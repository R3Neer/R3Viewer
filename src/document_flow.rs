//! Geometry-independent traversal of validated Items. No parser or layout in seek/draw.
use crate::{
    document_consumer_budget::{Charge, ChargedVec, ConsumerBudget},
    document_model::{Document, NONE},
    document_text::{DocumentWork, Error, Result},
    text_engine::Identity,
};
use std::{
    mem::size_of,
    sync::{Arc, atomic::AtomicBool},
};
const DEPTH: usize = 65;
/// Counts actual structural activity, not glyph/Source bytes or timing guarantees.
#[derive(Clone, Copy, Debug, Default)]
pub struct QueryStats {
    pub root_enters: usize,
    pub record_reads: usize,
    pub checkpoint_reads: usize,
}
fn increment(n: &mut usize) -> Result<()> {
    *n = n.checked_add(1).ok_or(Error::Invalid)?;
    Ok(())
}
#[derive(Clone, Copy, Debug, Default)]
struct Frame {
    block: u32,
    next: u32,
    stage: u32,
    enter_ordinal: u64,
}
#[derive(Clone, Debug)]
struct CursorState {
    stack: [Frame; DEPTH],
    depth: usize,
    ordinal: u64,
    identity: Identity,
    include_titles: bool,
}
/// Retained cursor owns its actual bytes and one global checkpoint slot; not clonable.
pub struct FlowCursor {
    state: CursorState,
    _charge: Charge,
}
impl FlowCursor {
    /// Borrowed actual stack after the current event; no parent/root replay or heap copy.
    pub(crate) fn context<'a>(
        &'a self,
        work: &mut DocumentWork<'_>,
    ) -> Result<impl DoubleEndedIterator<Item = (u32, u32, u32, u64)> + 'a> {
        work.admit_structure(
            self.state
                .depth
                .checked_mul(size_of::<Frame>())
                .ok_or(Error::Work)?,
        )?;
        Ok(self.state.stack[..self.state.depth]
            .iter()
            .map(|f| (f.block, f.next, f.stage, f.enter_ordinal)))
    }
    pub(crate) fn parent_item_ordinal(
        &self,
        document: &Document,
        block: u32,
        work: &mut DocumentWork<'_>,
    ) -> Result<u32> {
        work.admit_structure(self.state.depth * size_of::<Frame>())?;
        let depth = self.state.stack[..self.state.depth]
            .iter()
            .position(|f| f.block == block)
            .ok_or(Error::Invalid)?;
        let parent = self
            .state
            .stack
            .get(depth.checked_sub(1).ok_or(Error::Invalid)?)
            .ok_or(Error::Invalid)?;
        work.admit_structure(
            size_of::<crate::document_model::Block>() + size_of::<crate::document_model::Item>(),
        )?;
        let b = document
            .blocks()
            .get(parent.block as usize)
            .ok_or(Error::Invalid)?;
        let position = b
            .first_item()
            .checked_add(parent.next.checked_sub(1).ok_or(Error::Invalid)?)
            .ok_or(Error::Invalid)?;
        let item = document
            .items()
            .get(position as usize)
            .ok_or(Error::Invalid)?;
        if item.kind() != 0 || item.target() != block {
            return Err(Error::Invalid);
        }
        Ok(item.ordinal())
    }
    pub fn ordinal(&self) -> u64 {
        self.state.ordinal
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Enter(u32),
    Run { block: u32, run: u32 },
    Leave(u32),
}
#[derive(Clone, Debug)]
pub struct FlowLocation {
    identity: Identity,
    ordinal: u64,
}
impl FlowLocation {
    pub(crate) fn same(&self, other: &Self) -> bool {
        self.identity.same(&other.identity) && self.ordinal == other.ordinal
    }
    pub fn ordinal(&self) -> u64 {
        self.ordinal
    }
}
#[derive(Debug)]
pub struct Target {
    pub location: FlowLocation,
    pub block: u32,
    pub run: u32,
    pub raw_byte: usize,
    cursor: CursorState,
}
#[derive(Clone, Copy, Debug)]
pub struct FoldOverride {
    pub block: u32,
    pub expanded: bool,
}
fn expanded(
    document: &Document,
    block: u32,
    folds: &[FoldOverride],
    work: &mut DocumentWork<'_>,
) -> Result<bool> {
    let record = document
        .blocks()
        .get(block as usize)
        .ok_or(Error::Invalid)?;
    if record.kind() != 4 {
        return Ok(true);
    }
    let (mut low, mut high) = (0, folds.len());
    while low < high {
        work.admit_structure(size_of::<FoldOverride>())?;
        let mid = low + (high - low) / 2;
        if folds[mid].block < block {
            low = mid + 1;
        } else {
            high = mid;
        }
    }
    Ok(folds
        .get(low)
        .filter(|v| v.block == block)
        .map_or(record.aux2() != 2, |v| v.expanded))
}
struct Checkpoint {
    cursor: CursorState,
}
pub struct FlowIndex {
    document: Arc<Document>,
    identity: Identity,
    points: ChargedVec<Checkpoint>,
    folds: ChargedVec<FoldOverride>,
    first: Option<Target>,
    terminal: Option<Target>,
    events: u64,
    include_titles: bool,
    preparation_work: usize,
    _owned: Charge,
    budget: ConsumerBudget,
}

/// Immutable shared allocation; the original FlowIndex body/points keep their existing charges.
struct FlowAllocation {
    index: FlowIndex,
    _arc_metadata: Charge,
}
pub(crate) struct SharedFlow {
    inner: Arc<FlowAllocation>,
}
pub(crate) struct FlowHandle {
    inner: Arc<FlowAllocation>,
    _alias: Charge,
}
impl SharedFlow {
    pub(crate) fn new(index: FlowIndex) -> Result<Self> {
        let extra = size_of::<FlowAllocation>()
            .checked_sub(size_of::<FlowIndex>())
            .and_then(|n| n.checked_add(2 * size_of::<usize>()))
            .ok_or(Error::Budget)?;
        let charge = index.budget.reserve(extra)?;
        Ok(Self {
            inner: Arc::new(FlowAllocation {
                index,
                _arc_metadata: charge,
            }),
        })
    }
    pub(crate) fn index(&self) -> &FlowIndex {
        &self.inner.index
    }
    pub(crate) fn alias(&self) -> Result<FlowHandle> {
        let charge = self.index().budget.reserve(size_of::<FlowHandle>())?;
        Ok(FlowHandle {
            inner: self.inner.clone(),
            _alias: charge,
        })
    }
    pub(crate) fn same(&self, handle: &FlowHandle) -> bool {
        Arc::ptr_eq(&self.inner, &handle.inner)
    }
}
impl FlowHandle {
    pub(crate) fn index(&self) -> &FlowIndex {
        &self.inner.index
    }
    pub(crate) fn try_alias(&self) -> Result<Self> {
        let charge = self.index().budget.reserve(size_of::<Self>())?;
        Ok(Self {
            inner: self.inner.clone(),
            _alias: charge,
        })
    }
    #[cfg(test)]
    pub(crate) fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}
impl CursorState {
    fn root(identity: Identity, include_titles: bool) -> Self {
        let mut stack = [Frame::default(); DEPTH];
        stack[0].block = 0;
        Self {
            stack,
            depth: 1,
            ordinal: 0,
            identity,
            include_titles,
        }
    }
    fn next(
        &mut self,
        document: &Document,
        work: &mut DocumentWork<'_>,
        stats: &mut QueryStats,
        folds: &[FoldOverride],
    ) -> Result<Option<Event>> {
        if self.depth == 0 {
            work.admit_structure(0)?;
            return Ok(None);
        }
        loop {
            work.admit_structure(32 + size_of::<crate::document_model::Block>())?;
            increment(&mut stats.record_reads)?;
            let at = self.depth.checked_sub(1).ok_or(Error::Invalid)?;
            let frame = self.stack[at];
            let block = document
                .blocks()
                .get(frame.block as usize)
                .ok_or(Error::Invalid)?;
            let event = match frame.stage {
                0 => {
                    self.stack[at].stage =
                        if self.include_titles && block.kind() == 4 && block.aux1() != NONE {
                            3
                        } else {
                            1
                        };
                    if self.stack[at].stage == 1 && !expanded(document, frame.block, folds, work)? {
                        self.stack[at].next = block.item_count();
                    }
                    if frame.block == 0 {
                        increment(&mut stats.root_enters)?;
                    }
                    Event::Enter(frame.block)
                }
                1 => {
                    if frame.next < block.item_count() {
                        let offset = block
                            .first_item()
                            .checked_add(frame.next)
                            .ok_or(Error::Invalid)?;
                        work.admit_structure(size_of::<crate::document_model::Item>())?;
                        let item = document
                            .items()
                            .get(offset as usize)
                            .ok_or(Error::Invalid)?;
                        self.stack[at].next = frame.next.checked_add(1).ok_or(Error::Invalid)?;
                        match item.kind() {
                            0 => {
                                if self.depth == DEPTH {
                                    return Err(Error::Invalid);
                                }
                                self.stack[self.depth] = Frame {
                                    block: item.target(),
                                    next: 0,
                                    stage: 0,
                                    enter_ordinal: self.ordinal,
                                };
                                self.depth += 1;
                                continue;
                            }
                            1 => Event::Run {
                                block: frame.block,
                                run: item.target(),
                            },
                            _ => return Err(Error::Invalid),
                        }
                    } else {
                        self.stack[at].stage = 2;
                        continue;
                    }
                }
                3 => {
                    work.admit_structure(size_of::<crate::document_model::Domain>())?;
                    let title = document
                        .domains()
                        .get(block.aux1() as usize)
                        .ok_or(Error::Invalid)?;
                    if frame.next < title.run_count() {
                        let run = title
                            .first_run()
                            .checked_add(frame.next)
                            .ok_or(Error::Invalid)?;
                        self.stack[at].next = frame.next.checked_add(1).ok_or(Error::Invalid)?;
                        Event::Run {
                            block: frame.block,
                            run,
                        }
                    } else {
                        self.stack[at].stage = 1;
                        self.stack[at].next = if expanded(document, frame.block, folds, work)? {
                            0
                        } else {
                            block.item_count()
                        };
                        continue;
                    }
                }
                2 => {
                    self.depth -= 1;
                    Event::Leave(frame.block)
                }
                _ => return Err(Error::Invalid),
            };
            self.ordinal = self.ordinal.checked_add(1).ok_or(Error::Invalid)?;
            return Ok(Some(event));
        }
    }
}
impl FlowIndex {
    /// Fork the actual current cursor under its own charge; never replay Root to obtain context.
    pub(crate) fn copy_current(
        &self,
        cursor: &FlowCursor,
        work: &mut DocumentWork<'_>,
    ) -> Result<FlowCursor> {
        if !cursor.state.identity.same(&self.identity) {
            return Err(Error::Invalid);
        }
        work.admit_structure(size_of::<CursorState>())?;
        let charge = self.budget.reserve_points(size_of::<FlowCursor>(), 1)?;
        Ok(FlowCursor {
            state: cursor.state.clone(),
            _charge: charge,
        })
    }
    pub(crate) fn expanded(&self, block: u32, work: &mut DocumentWork<'_>) -> Result<bool> {
        expanded(&self.document, block, self.folds.as_slice(), work)
    }
    pub fn fixed_requested_bytes() -> usize {
        size_of::<Self>() + 4 * size_of::<CursorState>() + 2 * size_of::<usize>()
    }
    pub fn checkpoint_size() -> usize {
        size_of::<Checkpoint>()
    }
    pub fn prepare(
        document: Arc<Document>,
        budget: &ConsumerBudget,
        slots: usize,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        Self::prepare_with_folds(document, budget, slots, &[], cancel)
    }
    pub fn prepare_with_folds(
        document: Arc<Document>,
        budget: &ConsumerBudget,
        slots: usize,
        overrides: &[FoldOverride],
        cancel: &AtomicBool,
    ) -> Result<Self> {
        Self::prepare_mode(document, budget, slots, overrides, cancel, false)
    }
    pub(crate) fn prepare_visible(
        document: Arc<Document>,
        budget: &ConsumerBudget,
        slots: usize,
        overrides: &[FoldOverride],
        cancel: &AtomicBool,
    ) -> Result<Self> {
        Self::prepare_mode(document, budget, slots, overrides, cancel, true)
    }
    fn prepare_mode(
        document: Arc<Document>,
        budget: &ConsumerBudget,
        slots: usize,
        overrides: &[FoldOverride],
        cancel: &AtomicBool,
        include_titles: bool,
    ) -> Result<Self> {
        if !budget.belongs_to(&document) || !(2..=4096).contains(&slots) {
            return Err(Error::Invalid);
        }
        // Object, identity allocation and two temporary full cursors are admitted first.
        let owned = budget.reserve_points(Self::fixed_requested_bytes(), 2)?;
        let identity = Identity::new();
        let mut cursor = CursorState::root(identity.clone(), include_titles);
        let mut work = DocumentWork::preparation(cancel);
        let mut folds = ChargedVec::new(budget, overrides.len())?;
        let mut previous = None;
        for item in overrides {
            work.admit_structure(
                size_of::<FoldOverride>() + size_of::<crate::document_model::Block>(),
            )?;
            if previous.is_some_and(|p| p >= item.block)
                || document
                    .blocks()
                    .get(item.block as usize)
                    .is_none_or(|b| b.kind() != 4)
            {
                return Err(Error::Invalid);
            }
            folds.push(*item)?;
            previous = Some(item.block);
        }
        let mut first = None;
        let mut terminal = None;
        loop {
            work.admit_structure(size_of::<CursorState>())?;
            let before = cursor.clone();
            let Some(event) = cursor.next(
                &document,
                &mut work,
                &mut QueryStats::default(),
                folds.as_slice(),
            )?
            else {
                break;
            };
            if let Event::Enter(block) = event
                && matches!(
                    document.blocks()[block as usize].kind(),
                    3 | 4 | 6 | 11 | 12
                )
            {
                work.admit_structure(2 * size_of::<Target>())?;
                let target = Target {
                    location: FlowLocation {
                        identity: identity.clone(),
                        ordinal: cursor.ordinal - 1,
                    },
                    block,
                    run: NONE,
                    raw_byte: 0,
                    cursor: before.clone(),
                };
                if first.is_none() {
                    first = Some(Target {
                        location: target.location.clone(),
                        block: target.block,
                        run: target.run,
                        raw_byte: target.raw_byte,
                        cursor: target.cursor.clone(),
                    });
                }
                terminal = Some(target);
            }
            if let Event::Run { block, run } = event {
                work.admit_structure(size_of::<crate::document_model::Run>())?;
                let record = document.runs().get(run as usize).ok_or(Error::Invalid)?;
                let len = match record.kind() {
                    2 | 3 | 5 => Some(0),
                    _ if record.text() != NONE => {
                        work.admit_structure(size_of::<crate::document_model::TextRef>())?;
                        Some(document.text(record.text())?.len())
                    }
                    _ => None,
                };
                if let Some(len) = len.filter(|n| *n > 0 || matches!(record.kind(), 2 | 3 | 5)) {
                    work.admit_structure(2 * size_of::<Target>())?;
                    let location = FlowLocation {
                        identity: identity.clone(),
                        ordinal: cursor.ordinal - 1,
                    };
                    let start = Target {
                        location: location.clone(),
                        block,
                        run,
                        raw_byte: 0,
                        cursor: before.clone(),
                    };
                    if first.is_none() {
                        first = Some(start);
                    }
                    terminal = Some(Target {
                        location,
                        block,
                        run,
                        raw_byte: len,
                        cursor: before,
                    });
                }
            }
        }
        let events = cursor.ordinal;
        let stride = events.div_ceil((slots - 1) as u64).max(1);
        let capacity = usize::try_from(events.div_ceil(stride))
            .map_err(|_| Error::Budget)?
            .checked_add(1)
            .ok_or(Error::Budget)?
            .min(slots);
        let mut points = ChargedVec::new_points(budget, capacity, 1)?;
        cursor = CursorState::root(identity.clone(), include_titles);
        work.admit_structure(size_of::<Checkpoint>())?;
        points.push(Checkpoint {
            cursor: cursor.clone(),
        })?;
        let mut threshold = stride;
        while cursor
            .next(
                &document,
                &mut work,
                &mut QueryStats::default(),
                folds.as_slice(),
            )?
            .is_some()
        {
            if cursor.ordinal >= threshold && cursor.ordinal < events {
                work.admit_structure(size_of::<Checkpoint>())?;
                points.push(Checkpoint {
                    cursor: cursor.clone(),
                })?;
                threshold = cursor.ordinal.checked_add(stride).ok_or(Error::Invalid)?;
            }
        }
        if cursor.ordinal != events {
            return Err(Error::Invalid);
        }
        work.admit_structure(0)?;
        Ok(Self {
            document,
            identity,
            points,
            folds,
            first,
            terminal,
            events,
            include_titles,
            preparation_work: work.used(),
            _owned: owned,
            budget: budget.clone(),
        })
    }
    pub(crate) fn check_location(&self, location: &FlowLocation) -> Result<()> {
        if location.identity.same(&self.identity) && location.ordinal < self.events {
            Ok(())
        } else {
            Err(Error::Invalid)
        }
    }
    pub(crate) fn block_for_run(&self, run: u32) -> Result<u32> {
        let record = self
            .document
            .runs()
            .get(run as usize)
            .ok_or(Error::Invalid)?;
        let domain = self
            .document
            .domains()
            .get(record.domain() as usize)
            .ok_or(Error::Invalid)?;
        if !matches!(domain.kind(), 0..=2) {
            return Err(Error::Invalid);
        }
        Ok(domain.owner())
    }
    pub(crate) fn validate_run_byte(
        &self,
        run: u32,
        raw_byte: usize,
        work: &mut DocumentWork<'_>,
    ) -> Result<()> {
        work.admit_structure(size_of::<crate::document_model::Run>())?;
        let record = self
            .document
            .runs()
            .get(run as usize)
            .ok_or(Error::Invalid)?;
        if matches!(record.kind(), 2 | 3 | 5) {
            return if raw_byte == 0 {
                Ok(())
            } else {
                Err(Error::Invalid)
            };
        }
        work.admit_structure(size_of::<crate::document_model::TextRef>() + 1)?;
        let text = self
            .document
            .text(record.text())
            .map_err(|_| Error::Invalid)?;
        if raw_byte > text.len() || !text.is_char_boundary(raw_byte) {
            return Err(Error::Invalid);
        }
        Ok(())
    }
    pub(crate) fn validate_visual_part(
        &self,
        run: u32,
        byte: usize,
        part: u8,
        work: &mut DocumentWork<'_>,
    ) -> Result<()> {
        crate::document_text::validate_visual_part(&self.document, run, byte, part, work)
    }
    pub fn event_count(&self) -> u64 {
        self.events
    }
    pub fn preparation_work(&self) -> usize {
        self.preparation_work
    }
    pub fn first(&self) -> Option<&Target> {
        self.first.as_ref()
    }
    pub fn terminal(&self) -> Option<&Target> {
        self.terminal.as_ref()
    }
    pub fn location(&self, ordinal: u64) -> Result<FlowLocation> {
        if ordinal >= self.events {
            Err(Error::Invalid)
        } else {
            Ok(FlowLocation {
                identity: self.identity.clone(),
                ordinal,
            })
        }
    }
    fn resolve(
        &self,
        location: &FlowLocation,
        work: &mut DocumentWork<'_>,
        stats: &mut QueryStats,
    ) -> Result<CursorState> {
        if !location.identity.same(&self.identity) || location.ordinal >= self.events {
            return Err(Error::Invalid);
        }
        work.admit_structure(size_of::<CursorState>())?;
        for endpoint in [&self.first, &self.terminal].into_iter().flatten() {
            if endpoint.location.ordinal == location.ordinal {
                work.admit_structure(0)?;
                return Ok(endpoint.cursor.clone());
            }
        }
        let points = self.points.as_slice();
        let (mut low, mut high) = (0, points.len());
        while low < high {
            work.admit_structure(size_of::<Checkpoint>())?;
            increment(&mut stats.checkpoint_reads)?;
            let mid = low + (high - low) / 2;
            if points[mid].cursor.ordinal <= location.ordinal {
                low = mid + 1;
            } else {
                high = mid;
            }
        }
        let i = low.checked_sub(1).ok_or(Error::Invalid)?;
        let mut cursor = points[i].cursor.clone();
        while cursor.ordinal < location.ordinal {
            if cursor
                .next(&self.document, work, stats, self.folds.as_slice())?
                .is_none()
            {
                return Err(Error::Invalid);
            }
        }
        Ok(cursor)
    }

    pub(crate) fn cursor_at(
        &self,
        location: &FlowLocation,
        w: &mut DocumentWork<'_>,
    ) -> Result<FlowCursor> {
        let charge = self.budget.reserve_points(size_of::<FlowCursor>(), 1)?;
        let state = self.resolve(location, w, &mut QueryStats::default())?;
        Ok(FlowCursor {
            state,
            _charge: charge,
        })
    }
    pub(crate) fn overrides(&self) -> &[FoldOverride] {
        self.folds.as_slice()
    }
    pub(crate) fn locate_block(&self, block: u32, cancel: &AtomicBool) -> Result<FlowLocation> {
        let mut cursor = self.start()?;
        let mut w = DocumentWork::preparation(cancel);
        loop {
            let ordinal = cursor.ordinal();
            match self.next_event(&mut cursor, &mut w)? {
                Some(Event::Enter(b)) if b == block => return self.location(ordinal),
                Some(_) => {}
                None => return Err(Error::Invalid),
            }
        }
    }
    pub fn event_at(&self, location: &FlowLocation, work: &mut DocumentWork<'_>) -> Result<Event> {
        self.event_at_with_stats(location, work, &mut QueryStats::default())
    }
    pub fn event_at_with_stats(
        &self,
        location: &FlowLocation,
        work: &mut DocumentWork<'_>,
        stats: &mut QueryStats,
    ) -> Result<Event> {
        let mut c = self.resolve(location, work, stats)?;
        c.next(&self.document, work, stats, self.folds.as_slice())?
            .ok_or(Error::Invalid)
    }
    pub fn locate_run(&self, run: u32, cancel: &AtomicBool) -> Result<FlowLocation> {
        if self.document.runs().get(run as usize).is_none() {
            return Err(Error::Invalid);
        }
        let _scratch = self.budget.reserve(size_of::<CursorState>())?;
        let mut c = CursorState::root(self.identity.clone(), self.include_titles);
        let mut w = DocumentWork::preparation(cancel);
        while let Some(event) = c.next(
            &self.document,
            &mut w,
            &mut QueryStats::default(),
            self.folds.as_slice(),
        )? {
            if matches!(event,Event::Run{run:r,..} if r==run) {
                return self.location(c.ordinal - 1);
            }
        }
        Err(Error::Invalid)
    }
    pub fn next_event(
        &self,
        cursor: &mut FlowCursor,
        work: &mut DocumentWork<'_>,
    ) -> Result<Option<Event>> {
        if !cursor.state.identity.same(&self.identity) {
            return Err(Error::Invalid);
        }
        work.admit_structure(size_of::<CursorState>())?;
        let mut candidate = cursor.state.clone();
        let event = candidate.next(
            &self.document,
            work,
            &mut QueryStats::default(),
            self.folds.as_slice(),
        )?;
        cursor.state = candidate;
        Ok(event)
    }
    pub fn start(&self) -> Result<FlowCursor> {
        let charge = self.budget.reserve_points(size_of::<FlowCursor>(), 1)?;
        Ok(FlowCursor {
            state: CursorState::root(self.identity.clone(), self.include_titles),
            _charge: charge,
        })
    }
}
#[cfg(test)]
#[path = "document_flow_tests.rs"]
mod tests;
