//! Logical intradocument navigation; no geometry, worker, routing or Source reopening.
use crate::{
    document_consumer_budget::{Charge, ChargedVec, ConsumerBudget},
    document_flow::{Event, FlowIndex, FlowLocation},
    document_text::{DocumentWork, Error, Result},
};
use std::mem::size_of;
#[derive(Clone, Debug)]
pub struct State {
    location: FlowLocation,
    block: u32,
    run: u32,
    raw_byte: usize,
    position_pending: bool,
}
impl State {
    fn same_anchor(&self, other: &Self) -> bool {
        self.location.same(&other.location)
            && self.block == other.block
            && self.run == other.run
            && self.raw_byte == other.raw_byte
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
    pub fn position_pending(&self) -> bool {
        self.position_pending
    }
}
pub struct Navigator {
    index: FlowIndex,
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
        // Mandatory logical state/history is admitted before optional Flow density.
        let state_charge = budget.reserve(
            size_of::<Self>()
                .checked_add(2 * size_of::<State>())
                .ok_or(Error::Budget)?,
        )?;
        let history = ChargedVec::new(budget, 128)?;
        let index = FlowIndex::prepare(document, budget, slots, cancel)?;
        let current = index.first().map(|t| State {
            location: t.location.clone(),
            block: t.block,
            run: t.run,
            raw_byte: t.raw_byte,
            position_pending: false,
        });
        Ok(Self {
            index,
            current,
            history,
            _state_charge: state_charge,
        })
    }
    pub fn current(&self) -> Option<&State> {
        self.current.as_ref()
    }
    pub fn history_len(&self) -> usize {
        self.history.as_slice().len()
    }
    pub fn index(&self) -> &FlowIndex {
        &self.index
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
        if let Some(t) = self.index.first() {
            self.commit(
                State {
                    location: t.location.clone(),
                    block: t.block,
                    run: t.run,
                    raw_byte: 0,
                    position_pending: false,
                },
                false,
            )?;
        }
        Ok(())
    }
    pub fn end(&mut self) -> Result<()> {
        if let Some(t) = self.index.terminal() {
            self.commit(
                State {
                    location: t.location.clone(),
                    block: t.block,
                    run: t.run,
                    raw_byte: t.raw_byte,
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
        self.index.check_location(&location)?;
        let event = self.index.event_at(&location, work);
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
                position_pending: pending,
            },
            false,
        )
    }
    /// Construct a selected anchor through real traversal; callers cannot forge its fields.
    pub fn selected_run(&self, run: u32, cancel: &std::sync::atomic::AtomicBool) -> Result<State> {
        let location = self.index.locate_run(run, cancel)?;
        let block = self.index.block_for_run(run)?;
        Ok(State {
            location,
            block,
            run,
            raw_byte: 0,
            position_pending: false,
        })
    }
    /// A known anchor survives WorkOver, including its block/run/raw UTF8 position.
    pub fn seek_selected(
        &mut self,
        mut selected: State,
        work: &mut DocumentWork<'_>,
    ) -> Result<()> {
        self.index.check_location(&selected.location)?;
        match self.index.event_at(&selected.location, work) {
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
        self.index.check_location(&selected.location)?;
        match self.index.event_at(&selected.location, work) {
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
            self.index.check_location(&last.location)?;
        }
        if let Some(last) = self.history.pop() {
            self.current = Some(last);
        }
        Ok(())
    }
}
