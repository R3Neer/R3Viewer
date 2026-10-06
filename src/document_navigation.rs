//! Logical intradocument navigation; no geometry, worker, routing or Source reopening.
use crate::{
    document_consumer_budget::{Charge, ChargedVec, ConsumerBudget},
    document_flow::{Event, FlowHandle, FlowIndex, FlowLocation, SharedFlow},
    document_text::{DocumentWork, Error, Result},
};
use std::mem::size_of;
#[derive(Clone, Debug)]
pub struct State {
    location: FlowLocation,
    block: u32,
    run: u32,
    raw_byte: usize,
    projection_part: u8,
    position_pending: bool,
}
impl State {
    fn same_anchor(&self, other: &Self) -> bool {
        self.location.same(&other.location)
            && self.block == other.block
            && self.run == other.run
            && self.raw_byte == other.raw_byte
            && self.projection_part == other.projection_part
    }
    pub fn location(&self) -> &FlowLocation {
        &self.location
    }
    pub fn block(&self) -> u32 {
        self.block
    }
    pub fn run(&self) -> u32 {
        self.run
    }
    pub fn raw_byte(&self) -> usize {
        self.raw_byte
    }
    pub(crate) fn projection_part(&self) -> u8 {
        self.projection_part
    }
    pub fn position_pending(&self) -> bool {
        self.position_pending
    }
}
pub struct Navigator {
    index: SharedFlow,
    current: Option<State>,
    history: ChargedVec<State>,
    _state_charge: Charge,
}
impl Navigator {
    pub fn prepare(
        document: std::sync::Arc<crate::document_model::Document>,
        slots: usize,
        budget: &ConsumerBudget,
        cancel: &std::sync::atomic::AtomicBool,
    ) -> Result<Self> {
        if !budget.belongs_to(&document) {
            return Err(Error::Invalid);
        }
        let storage = NavigationStorage::new(budget)?;
        let flow = SharedFlow::new(FlowIndex::prepare(document, budget, slots, cancel)?)?;
        let current = flow.index().first().map(State::from_target);
        Ok(Self::from_storage(flow, current, storage))
    }
    pub(crate) fn from_storage(
        index: SharedFlow,
        current: Option<State>,
        storage: NavigationStorage,
    ) -> Self {
        Self {
            index,
            current,
            history: storage.history,
            _state_charge: storage.state_charge,
        }
    }
    pub(crate) fn share_flow(&self) -> Result<FlowHandle> {
        self.index.alias()
    }
    pub(crate) fn same_flow(&self, handle: &FlowHandle) -> bool {
        self.index.same(handle)
    }
    pub(crate) fn commit_prepared(
        &mut self,
        newflow: Option<SharedFlow>,
        selection: Option<State>,
    ) -> Result<()> {
        let flow = newflow.as_ref().unwrap_or(&self.index).index();
        if let Some(s) = &selection {
            flow.check_location(&s.location)?;
        }
        if let Some(f) = newflow {
            self.index = f;
        }
        self.current = selection;
        Ok(()) // History remains owned, raw anchors preserved; stale Back is rejected without pop.
    }
    #[allow(dead_code)] // b3 consumes this raw peek before background remapping, never destructive pop.
    pub(crate) fn raw_back(&self) -> Option<(u32, u32, usize)> {
        self.history
            .as_slice()
            .last()
            .map(|s| (s.block, s.run, s.raw_byte))
    }
    pub fn current(&self) -> Option<&State> {
        self.current.as_ref()
    }
    pub fn history_len(&self) -> usize {
        self.history.as_slice().len()
    }
    pub fn index(&self) -> &FlowIndex {
        self.index.index()
    }
    fn commit(&mut self, next: State, push: bool) -> Result<()> {
        if push
            && let Some(old) = &self.current
            && !old.same_anchor(&next)
        {
            self.history.push(old.clone())?;
        }
        self.current = Some(next);
        Ok(())
    }
    pub fn home(&mut self) -> Result<()> {
        if let Some(t) = self.index.index().first() {
            self.commit(
                State {
                    location: t.location.clone(),
                    block: t.block,
                    run: t.run,
                    raw_byte: 0,
                    projection_part: 0,
                    position_pending: false,
                },
                false,
            )?;
        }
        Ok(())
    }
    pub fn end(&mut self) -> Result<()> {
        if let Some(t) = self.index.index().terminal() {
            self.commit(
                State {
                    location: t.location.clone(),
                    block: t.block,
                    run: t.run,
                    raw_byte: t.raw_byte,
                    projection_part: 0,
                    position_pending: false,
                },
                false,
            )?;
        }
        Ok(())
    }
    /// caller supplies one frame work already including any Reference reserve.
    /// Work failure preserves a selected logical target; redraw must not call seek automatically.
    pub fn seek_ordinal(
        &mut self,
        location: FlowLocation,
        work: &mut DocumentWork<'_>,
    ) -> Result<()> {
        self.index.index().check_location(&location)?;
        let event = self.index.index().event_at(&location, work);
        let (block, run, pending) = match event {
            Ok(Event::Run { block, run }) => (block, run, false),
            Ok(Event::Enter(block) | Event::Leave(block)) => {
                (block, crate::document_model::NONE, false)
            }
            Err(Error::Work) => (
                crate::document_model::NONE,
                crate::document_model::NONE,
                true,
            ),
            Err(e) => return Err(e),
        };
        self.commit(
            State {
                location,
                block,
                run,
                raw_byte: 0,
                projection_part: 0,
                position_pending: pending,
            },
            false,
        )
    }
    /// Construct a selected anchor through real traversal; callers cannot forge its fields.
    pub fn selected_run(&self, run: u32, cancel: &std::sync::atomic::AtomicBool) -> Result<State> {
        let location = self.index.index().locate_run(run, cancel)?;
        let block = self.index.index().block_for_run(run)?;
        Ok(State {
            location,
            block,
            run,
            raw_byte: 0,
            projection_part: 0,
            position_pending: false,
        })
    }
    /// A known anchor survives WorkOver, including its block/run/raw UTF8 position.
    pub fn seek_selected(
        &mut self,
        mut selected: State,
        work: &mut DocumentWork<'_>,
    ) -> Result<()> {
        self.index.index().check_location(&selected.location)?;
        match self.index.index().event_at(&selected.location, work) {
            Ok(Event::Run { block, run }) if block == selected.block && run == selected.run => {
                selected.position_pending = false
            }
            Err(Error::Work) => selected.position_pending = true,
            Err(e) => return Err(e),
            _ => return Err(Error::Invalid),
        }
        self.commit(selected, false)
    }
    /// Explicit returnable transition, unlike Home/End/scroll/ordinary viewport seeking.
    pub fn jump_selected(
        &mut self,
        mut selected: State,
        work: &mut DocumentWork<'_>,
    ) -> Result<()> {
        self.index.index().check_location(&selected.location)?;
        match self.index.index().event_at(&selected.location, work) {
            Ok(Event::Run { block, run }) if block == selected.block && run == selected.run => {
                selected.position_pending = false
            }
            Err(Error::Work) => selected.position_pending = true,
            Err(e) => return Err(e),
            _ => return Err(Error::Invalid),
        }
        self.commit(selected, true)
    }
    pub fn back(&mut self) -> Result<()> {
        if let Some(last) = self.history.as_slice().last() {
            self.index.index().check_location(&last.location)?;
        }
        if let Some(last) = self.history.pop() {
            self.current = Some(last);
        }
        Ok(())
    }
}

pub(crate) struct NavigationStorage {
    history: ChargedVec<State>,
    state_charge: Charge,
}
impl NavigationStorage {
    pub(crate) fn new(budget: &ConsumerBudget) -> Result<Self> {
        let state_charge = budget.reserve(
            size_of::<Navigator>()
                .checked_add(2 * size_of::<State>())
                .ok_or(Error::Budget)?,
        )?;
        let history = ChargedVec::new(budget, 128)?;
        Ok(Self {
            history,
            state_charge,
        })
    }
}
impl State {
    pub(crate) fn from_target(t: &crate::document_flow::Target) -> Self {
        Self {
            location: t.location.clone(),
            block: t.block,
            run: t.run,
            raw_byte: t.raw_byte,
            projection_part: 0,
            position_pending: false,
        }
    }
    pub(crate) fn prepared_run(
        index: &FlowIndex,
        run: u32,
        raw_byte: usize,
        cancel: &std::sync::atomic::AtomicBool,
    ) -> Result<Self> {
        Self::prepared_visual(index, run, raw_byte, 0, cancel)
    }
    pub(crate) fn prepared_visual(
        index: &FlowIndex,
        run: u32,
        raw_byte: usize,
        projection_part: u8,
        cancel: &std::sync::atomic::AtomicBool,
    ) -> Result<Self> {
        let mut work = DocumentWork::preparation(cancel);
        index.validate_run_byte(run, raw_byte, &mut work)?;
        index.validate_visual_part(run, raw_byte, projection_part, &mut work)?;
        Ok(Self {
            location: index.locate_run(run, cancel)?,
            block: index.block_for_run(run)?,
            run,
            raw_byte,
            projection_part,
            position_pending: false,
        })
    }
    pub(crate) fn prepared_role(
        index: &FlowIndex,
        block: u32,
        cancel: &std::sync::atomic::AtomicBool,
    ) -> Result<Self> {
        Ok(Self {
            location: index.locate_block(block, cancel)?,
            block,
            run: crate::document_model::NONE,
            raw_byte: 0,
            projection_part: 0,
            position_pending: false,
        })
    }
}
