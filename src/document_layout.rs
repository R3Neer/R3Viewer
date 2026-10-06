//! Prepared textual window. No renderer, parser, TUI pump or optional color allowance.
#![allow(dead_code)] // Private b1 library; session adoption is D2c.
use crate::{
    document_consumer_budget::{Charge, ChargedVec, ConsumerBudget},
    document_consumer_profile::ConsumerProfile,
    document_flow::{Event, FlowHandle},
    document_model::{Document, NONE},
    document_navigation::State,
    document_text::{
        DocumentWork, Error, IndexProfile, Point, Relation, Result, SegmentedIndex, SegmentedText,
    },
};
use std::{
    mem::size_of,
    ops::Range,
    sync::{Arc, atomic::AtomicBool},
};
pub(crate) const WINDOW_BYTES: usize = 131072;
pub(crate) const FRAME_BYTES: usize = 262144;
pub(crate) const REFERENCE_WORK: usize = 8192;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Geometry {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub left: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReferenceReason {
    Geometry,
    Symbol,
    WidthZero,
    Width,
    Window,
    Work,
    Budget,
    Resource,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CellKind {
    Glyph,
    Resource(u32),
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct FirstComponent {
    pub block: u32,
    pub domain: u32,
    pub run: u32,
    pub local: usize,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Glyph {
    pub x: u16,
    pub row: u16,
    pub width: u16,
    pub start: u32,
    pub end: u32,
    pub kind: CellKind,
    pub styles: u32,
    pub first: Option<FirstComponent>,
    pub links: usize,
    pub relation_mask: u8,
    pub projection_part: u8,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Row {
    pub first: u32,
    pub count: u32,
}
pub(crate) struct SequenceSlot {
    index: SegmentedIndex,
    domain: u32,
    runs: Range<usize>,
    _metadata: Charge,
}
impl SequenceSlot {
    fn prepare(
        document: Arc<Document>,
        domain: u32,
        runs: Range<usize>,
        budget: &ConsumerBudget,
        cancel: &AtomicBool,
        profile: IndexProfile,
    ) -> Result<Arc<Self>> {
        let n = size_of::<Self>()
            .checked_sub(size_of::<SegmentedIndex>())
            .and_then(|n| n.checked_add(2 * size_of::<usize>()))
            .ok_or(Error::Budget)?;
        let metadata = budget.reserve(n)?;
        let text = SegmentedText::prepare_with_profile(
            document,
            domain,
            runs.clone(),
            budget,
            cancel,
            profile,
        )?;
        let index = SegmentedIndex::prepare_with_profile(text, budget, cancel, profile)?;
        Ok(Arc::new(Self {
            index,
            domain,
            runs,
            _metadata: metadata,
        }))
    }
    fn matches(&self, domain: u32, runs: &Range<usize>) -> bool {
        self.domain == domain && self.runs == *runs
    }
}
pub(crate) struct TextWindow {
    symbols: ChargedVec<u8>,
    glyphs: ChargedVec<Glyph>,
    rows: ChargedVec<Row>,
    slot: Option<Arc<SequenceSlot>>,
    _owner: Charge,
    bytes: usize,
    pub work: usize,
    pub preparation_work: usize,
    next: Option<crate::document_view::LayoutAnchor>,
}
impl TextWindow {
    pub(crate) fn symbols(&self) -> &str {
        std::str::from_utf8(self.symbols.as_slice()).expect("complete projected UTF8")
    }
    pub(crate) fn glyphs(&self) -> &[Glyph] {
        self.glyphs.as_slice()
    }
    pub(crate) fn rows(&self) -> &[Row] {
        self.rows.as_slice()
    }
    pub(crate) fn bytes(&self) -> usize {
        self.bytes
    }
    pub(crate) fn next_anchor(&self) -> Option<crate::document_view::LayoutAnchor> {
        self.next
    }
    pub(crate) fn row_anchor(&self, row: usize) -> Option<crate::document_view::LayoutAnchor> {
        let r = self.rows.as_slice().get(row)?;
        let g = self.glyphs.as_slice().get(r.first as usize)?;
        if r.count == 0 {
            return None;
        }
        let f = g.first?;
        Some(crate::document_view::LayoutAnchor {
            raw: crate::document_view::RawAnchor {
                block: f.block,
                run: f.run,
                byte: f.local,
            },
            projection_part: g.projection_part,
        })
    }
    pub(crate) fn slot_alias(&self) -> Option<Arc<SequenceSlot>> {
        self.slot.clone()
    } // Arc field is charged in the input object.
}
pub(crate) enum LayoutWindow {
    Text(Box<TextWindow>),
    Reference {
        reason: ReferenceReason,
        work: usize,
    },
}
impl LayoutWindow {
    pub(crate) fn bytes(&self) -> usize {
        match self {
            Self::Text(w) => w.bytes(),
            Self::Reference { .. } => 0,
        }
    }
    pub(crate) fn text(&self) -> Option<&TextWindow> {
        match self {
            Self::Text(w) => Some(w),
            _ => None,
        }
    }
}
pub(crate) struct WindowBuilder {
    window: TextWindow,
    column: usize,
    row: usize,
    row_start: usize,
    geometry: Geometry,
}
impl WindowBuilder {
    pub(crate) fn new(budget: &ConsumerBudget, g: Geometry, old_bytes: usize) -> Result<Self> {
        if g.width == 0 || g.height == 0 {
            return Err(Error::Budget);
        }
        let cells = usize::from(g.width)
            .checked_mul(usize::from(g.height))
            .ok_or(Error::Budget)?;
        let symbols = cells.checked_mul(4).ok_or(Error::Budget)?.max(16384);
        let requested = size_of::<TextWindow>()
            .checked_add(symbols)
            .and_then(|n| n.checked_add(cells.checked_mul(size_of::<Glyph>())?))
            .and_then(|n| n.checked_add((usize::from(g.height) + 1).checked_mul(size_of::<Row>())?))
            .ok_or(Error::Budget)?;
        if requested > WINDOW_BYTES
            || old_bytes
                .checked_add(requested)
                .is_none_or(|n| n > FRAME_BYTES)
        {
            return Err(Error::Budget);
        }
        let owner = budget.reserve(size_of::<TextWindow>())?;
        let symbols = ChargedVec::new(budget, symbols)?;
        let glyphs = ChargedVec::new(budget, cells)?;
        let rows = ChargedVec::new(budget, usize::from(g.height) + 1)?;
        let bytes = size_of::<TextWindow>()
            .checked_add(symbols.capacity())
            .and_then(|n| n.checked_add(glyphs.capacity().checked_mul(size_of::<Glyph>())?))
            .and_then(|n| n.checked_add(rows.capacity().checked_mul(size_of::<Row>())?))
            .ok_or(Error::Budget)?;
        if bytes > WINDOW_BYTES || old_bytes.checked_add(bytes).is_none_or(|n| n > FRAME_BYTES) {
            return Err(Error::Budget);
        }
        Ok(Self {
            window: TextWindow {
                symbols,
                glyphs,
                rows,
                slot: None,
                _owner: owner,
                bytes,
                work: 0,
                preparation_work: 0,
                next: None,
            },
            column: 0,
            row: 0,
            row_start: 0,
            geometry: g,
        })
    }
    pub(crate) fn retain_candidate(&mut self, slot: Option<Arc<SequenceSlot>>) {
        self.window.slot = slot;
    }
    fn newline(&mut self, w: &mut DocumentWork<'_>) -> Result<bool> {
        w.admit_structure(size_of::<Row>())?;
        self.window.rows.push(Row {
            first: u32::try_from(self.row_start).map_err(|_| Error::Invalid)?,
            count: u32::try_from(self.window.glyphs.as_slice().len() - self.row_start)
                .map_err(|_| Error::Invalid)?,
        })?;
        self.row += 1;
        self.column = 0;
        self.row_start = self.window.glyphs.as_slice().len();
        Ok(self.row < usize::from(self.geometry.height))
    }
    fn resource(
        &mut self,
        block: u32,
        run: u32,
        resource: u32,
        w: &mut DocumentWork<'_>,
    ) -> Result<bool> {
        if self.column > 0 && !self.newline(w)? {
            return Ok(false);
        }
        w.admit_structure(size_of::<Glyph>())?;
        self.window.glyphs.push(Glyph {
            x: 0,
            row: self.row as u16,
            width: 0,
            start: 0,
            end: 0,
            kind: CellKind::Resource(resource),
            styles: 0,
            first: Some(FirstComponent {
                block,
                domain: NONE,
                run,
                local: 0,
            }),
            links: 0,
            relation_mask: 0,
            projection_part: 0,
        })?;
        self.newline(w)
    }
    #[allow(clippy::too_many_arguments)] // Explicit shared ledger, cancellation and Work are not replaceable defaults.
    fn sequence(
        &mut self,
        slot: Arc<SequenceSlot>,
        start_run: u32,
        raw_byte: usize,
        projection_part: u8,
        end_anchor: bool,
        block: u32,
        budget: &ConsumerBudget,
        cancel: &AtomicBool,
        w: &mut DocumentWork<'_>,
    ) -> Result<bool> {
        let _points = budget.reserve_points(2 * size_of::<Point>(), 2)?;
        let index = &slot.index;
        let mut prep = DocumentWork::preparation(cancel);
        let projected =
            index
                .text()
                .locate_visual(start_run, raw_byte, projection_part, &mut prep)?;
        let mut point = index.seek_before_with_work(projected, &mut prep)?;
        while point.offset() < projected {
            let before = point.clone();
            let unit = index
                .step(&mut point, 4096, &mut prep)?
                .ok_or(Error::Invalid)?;
            if unit.projected.end > projected {
                point = before;
                break;
            }
        }
        if end_anchor {
            // A bounded suffix of the terminal logical line keeps the final glyph visible;
            // no full-document geometry pass and no dependency on checkpoint placement.
            // Half the cell envelope leaves padding room for wide cluster wrapping.
            let suffix = usize::from(self.geometry.width)
                .saturating_mul(usize::from(self.geometry.height))
                / 2;
            point = index.seek_cell_with_work(
                point.line(),
                point.column().saturating_sub(suffix.max(1)),
                &mut prep,
            )?;
        }
        self.window.preparation_work = self
            .window
            .preparation_work
            .checked_add(prep.used())
            .ok_or(Error::Work)?;
        // Installing a slot drops the previous candidate before preparing the next domain.
        self.window.slot = Some(slot.clone());
        loop {
            let before_raw = index.visual_anchor_with_work(&point, w)?;
            let Some(unit) = index.step(&mut point, 4096, w)? else {
                break;
            };
            if unit.oversized {
                return Err(Error::UnsupportedRun(u32::MAX));
            }
            if unit.newline {
                if !self.newline(w)? {
                    if point.offset() < index.text().len() {
                        let (run, byte, part) = index.visual_anchor_with_work(&point, w)?;
                        self.window.next = Some(crate::document_view::LayoutAnchor {
                            raw: crate::document_view::RawAnchor { block, run, byte },
                            projection_part: part,
                        });
                    }
                    return Ok(false);
                }
                continue;
            }
            let mut width = unit.width;
            let is_tab = unit.projected.end - unit.projected.start == 1 && {
                let mut tab = false;
                index
                    .text()
                    .visit_projected(unit.projected.clone(), w, |s| {
                        tab = s == "\t";
                        Ok(())
                    })?;
                tab
            };
            if is_tab {
                width = 8 - self.column % 8;
            }
            let kind = self
                .window
                .glyphs
                .as_slice()
                .last()
                .filter(|g| usize::from(g.row) == self.row);
            if width == 0 && kind.is_none() {
                return Err(Error::UnsupportedRun(u32::MAX - 1));
            }
            if width > usize::from(self.geometry.width) {
                return Err(Error::UnsupportedRun(u32::MAX - 2));
            }
            // Code uses horizontal viewport; paragraph wrapping remains local to the prepared row.
            let code = self.window.slot.as_ref().is_some_and(|s| {
                let d = &s.index.text().document().domains()[s.domain as usize];
                matches!(
                    s.index.text().document().blocks()[d.owner() as usize].kind(),
                    10 | 14
                )
            });
            let x = if code {
                let end = self.column.checked_add(width).ok_or(Error::Invalid)?;
                if self.column < self.geometry.left
                    || end
                        > self
                            .geometry
                            .left
                            .checked_add(usize::from(self.geometry.width))
                            .ok_or(Error::Invalid)?
                {
                    self.column = end;
                    continue;
                }
                self.column - self.geometry.left
            } else {
                if self.column.checked_add(width).ok_or(Error::Invalid)?
                    > usize::from(self.geometry.width)
                    && !self.newline(w)?
                {
                    self.window.next = Some(crate::document_view::LayoutAnchor {
                        raw: crate::document_view::RawAnchor {
                            block,
                            run: before_raw.0,
                            byte: before_raw.1,
                        },
                        projection_part: before_raw.2,
                    });
                    return Ok(false);
                }
                self.column
            };
            let start = self.window.symbols.as_slice().len();
            let old_glyph = if width == 0 {
                self.window.glyphs.as_slice().last().copied()
            } else {
                None
            };
            if let Some(old) = old_glyph
                && (old.end - old.start) as usize + unit.projected.len() > 16384
            {
                return Err(Error::UnsupportedRun(u32::MAX));
            }
            if unit.projected.len() > 16384 {
                return Err(Error::UnsupportedRun(u32::MAX));
            }
            if is_tab {
                for _ in 0..width {
                    w.admit_structure(1)?;
                    self.window.symbols.push(b' ')?;
                }
            } else {
                w.admit_structure(unit.projected.len())?;
                index
                    .text()
                    .visit_projected(unit.projected.clone(), w, |s| {
                        for &b in s.as_bytes() {
                            self.window.symbols.push(b)?;
                        }
                        Ok(())
                    })?;
            }
            let end = self.window.symbols.as_slice().len();
            let mut first = None;
            let mut styles = 0;
            let mut links = 0usize;
            let mut relation_mask = 0u8;
            index
                .text()
                .visit_coverage(unit.projected.clone(), w, |c| {
                    if first.is_none() {
                        first = Some(FirstComponent {
                            block,
                            domain: slot.domain,
                            run: c.run,
                            local: c.local.start,
                        });
                    }
                    styles |= c.styles;
                    links = links
                        .checked_add(usize::from(c.link.is_some()))
                        .ok_or(Error::Work)?;
                    relation_mask |= match c.relation {
                        Relation::ExactSource => 1,
                        Relation::ControlProjection => 2,
                        Relation::EnclosingToken => 4,
                        Relation::Synthetic => 8,
                    };
                    Ok(())
                })?;
            w.admit_structure(size_of::<Glyph>())?;
            if width == 0 {
                let last = self.window.glyphs.last_mut().ok_or(Error::Invalid)?;
                last.end = u32::try_from(end).map_err(|_| Error::Invalid)?;
                last.styles |= styles;
                last.links = last.links.checked_add(links).ok_or(Error::Work)?;
                last.relation_mask |= relation_mask;
            } else {
                self.window.glyphs.push(Glyph {
                    x: x as u16,
                    row: self.row as u16,
                    width: width as u16,
                    start: u32::try_from(start).map_err(|_| Error::Invalid)?,
                    end: u32::try_from(end).map_err(|_| Error::Invalid)?,
                    kind: CellKind::Glyph,
                    styles,
                    first,
                    links,
                    relation_mask,
                    projection_part: before_raw.2,
                })?;
            }
            self.column = self.column.checked_add(width).ok_or(Error::Invalid)?;
        }
        Ok(true)
    }
    fn finish(mut self, w: &mut DocumentWork<'_>) -> Result<TextWindow> {
        if self.row < usize::from(self.geometry.height) {
            self.newline(w)?;
        }
        self.window.work = w.used();
        Ok(self.window)
    }
}
// Copy admission occurs before the projected callback, not by a new Work allowance.

#[allow(clippy::too_many_arguments)] // Same explicit request authority and shared ledger throughout preparation.
pub(crate) fn build_window(
    mut builder: WindowBuilder,
    document: Arc<Document>,
    flow: &FlowHandle,
    selected: &State,
    old_slot: Option<&Arc<SequenceSlot>>,
    budget: &ConsumerBudget,
    cancel: &AtomicBool,
    profile: ConsumerProfile,
    end_anchor: bool,
) -> Result<LayoutWindow> {
    let mut w = DocumentWork::frame(cancel);
    w.admit_structure(REFERENCE_WORK)?;
    let mut prep = DocumentWork::preparation(cancel);
    let mut cursor = flow.index().cursor_at(selected.location(), &mut prep)?;
    let mut pending = None;
    let result = (|| {
        loop {
            let (event, location) = if let Some(p) = pending.take() {
                p
            } else {
                let ordinal = cursor.ordinal();
                let Some(event) = flow.index().next_event(&mut cursor, &mut w)? else {
                    break;
                };
                (event, flow.index().location(ordinal)?)
            };
            if let Event::Run { block, run } = event {
                w.admit_structure(size_of::<crate::document_model::Run>())?;
                let record = &document.runs()[run as usize];
                if record.kind() == 5 {
                    if !builder.resource(block, run, record.resource(), &mut w)? {
                        break;
                    }
                    continue;
                }
                let domain = record.domain();
                let mut first = record.ordinal() as usize;
                let mut last = first + 1;
                // Include only consecutive actual Flow Run events; child/resource barriers never disappear.
                let mut prior = location.ordinal();
                while prior > 0 {
                    let p = flow.index().location(prior - 1)?;
                    match flow.index().event_at(&p, &mut prep)? {
                        Event::Run { run: r, .. } => {
                            prep.admit_structure(size_of::<crate::document_model::Run>())?;
                            let r = &document.runs()[r as usize];
                            if r.domain() == domain
                                && r.kind() != 5
                                && (r.ordinal() as usize) + 1 == first
                            {
                                first -= 1;
                                prior -= 1;
                            } else {
                                break;
                            }
                        }
                        _ => break,
                    }
                }
                loop {
                    let ordinal = cursor.ordinal();
                    let Some(next) = flow.index().next_event(&mut cursor, &mut prep)? else {
                        break;
                    };
                    let l = flow.index().location(ordinal)?;
                    if let Event::Run { run: r, .. } = next {
                        prep.admit_structure(size_of::<crate::document_model::Run>())?;
                        let r = &document.runs()[r as usize];
                        if r.domain() == domain && r.kind() != 5 && r.ordinal() as usize == last {
                            last += 1;
                            continue;
                        }
                    }
                    pending = Some((next, l));
                    break;
                }
                let range = first..last;
                // Drop candidate slot before creating another; old current slot remains protected.
                let reuse = builder
                    .window
                    .slot
                    .as_ref()
                    .filter(|s| s.matches(domain, &range))
                    .cloned()
                    .or_else(|| old_slot.filter(|s| s.matches(domain, &range)).cloned());
                let slot = if let Some(slot) = reuse {
                    slot
                } else {
                    builder.window.slot = None;
                    SequenceSlot::prepare(
                        document.clone(),
                        domain,
                        range,
                        budget,
                        cancel,
                        profile.index,
                    )?
                };
                let byte = if run == selected.run() {
                    selected.raw_byte()
                } else {
                    0
                };
                if !builder.sequence(
                    slot,
                    run,
                    byte,
                    if run == selected.run() {
                        selected.projection_part()
                    } else {
                        0
                    },
                    end_anchor && run == selected.run(),
                    block,
                    budget,
                    cancel,
                    &mut w,
                )? {
                    break;
                }
            } else if matches!(event, Event::Leave(_))
                && builder.column > 0
                && !builder.newline(&mut w)?
            {
                if cursor.ordinal() < flow.index().event_count() {
                    let loc = flow.index().location(cursor.ordinal())?;
                    let next = flow.index().event_at(&loc, &mut prep)?;
                    builder.window.next = Some(crate::document_view::LayoutAnchor {
                        raw: match next {
                            Event::Run { block, run } => crate::document_view::RawAnchor {
                                block,
                                run,
                                byte: 0,
                            },
                            Event::Enter(block) | Event::Leave(block) => {
                                crate::document_view::RawAnchor {
                                    block,
                                    run: NONE,
                                    byte: 0,
                                }
                            }
                        },
                        projection_part: 0,
                    });
                }
                break;
            }
        }
        builder.window.preparation_work = builder
            .window
            .preparation_work
            .checked_add(prep.used())
            .ok_or(Error::Work)?;
        builder.finish(&mut w)
    })();
    match result {
        Ok(t) => Ok(LayoutWindow::Text(Box::new(t))),
        Err(Error::Cancelled) => Err(Error::Cancelled),
        Err(Error::Invalid) => Err(Error::Invalid),
        Err(e) => {
            let reason = match e {
                Error::Work => ReferenceReason::Work,
                Error::Budget => ReferenceReason::Window,
                Error::UnsupportedRun(k) if k == u32::MAX => ReferenceReason::Symbol,
                Error::UnsupportedRun(k) if k == u32::MAX - 1 => ReferenceReason::WidthZero,
                Error::UnsupportedRun(_) => ReferenceReason::Width,
                _ => ReferenceReason::Budget,
            };
            Ok(LayoutWindow::Reference {
                reason,
                work: w.used(),
            })
        }
    }
}

/// Background-only line positioning; no whole-document geometry index or UI traversal.
#[allow(clippy::too_many_arguments)] // No hidden owner, new allowance or implicit profile.
pub(crate) fn shift_line(
    document: Arc<Document>,
    flow: &FlowHandle,
    selected: &State,
    delta: i32,
    old_slot: Option<&Arc<SequenceSlot>>,
    budget: &ConsumerBudget,
    cancel: &AtomicBool,
    profile: ConsumerProfile,
) -> Result<(State, Option<Arc<SequenceSlot>>)> {
    if delta == 0 {
        return Ok((selected.clone(), old_slot.cloned()));
    }
    let mut prep = DocumentWork::preparation(cancel);
    let mut run = selected.run();
    let mut byte = selected.raw_byte();
    if run == NONE {
        let mut cursor = flow.index().cursor_at(selected.location(), &mut prep)?;
        loop {
            match flow.index().next_event(&mut cursor, &mut prep)? {
                Some(Event::Run { run: r, .. }) => {
                    run = r;
                    byte = 0;
                    break;
                }
                Some(_) => {}
                None => return Ok((selected.clone(), None)),
            }
        }
    }
    let record = &document.runs()[run as usize];
    prep.admit_structure(size_of_val_record())?;
    if record.kind() == 5 {
        return Ok((selected.clone(), None));
    }

    let domain = record.domain();
    let mut first = record.ordinal() as usize;
    let mut last = first + 1;
    let selected_location = flow.index().locate_run(run, cancel)?;
    let mut previous = selected_location.ordinal();
    while previous > 0 {
        let loc = flow.index().location(previous - 1)?;
        if let Event::Run { run: r, .. } = flow.index().event_at(&loc, &mut prep)? {
            prep.admit_structure(size_of_val_record())?;
            let rr = &document.runs()[r as usize];
            if rr.domain() == domain && rr.kind() != 5 && (rr.ordinal() as usize) + 1 == first {
                first -= 1;
                previous -= 1;
                continue;
            }
        }
        break;
    }
    let mut cursor = flow.index().cursor_at(&selected_location, &mut prep)?;
    let _first = flow.index().next_event(&mut cursor, &mut prep)?;
    loop {
        if let Some(Event::Run { run: r, .. }) = flow.index().next_event(&mut cursor, &mut prep)? {
            prep.admit_structure(size_of_val_record())?;
            let rr = &document.runs()[r as usize];
            if rr.domain() == domain && rr.kind() != 5 && rr.ordinal() as usize == last {
                last += 1;
                continue;
            }
        }
        break;
    }
    let range = first..last;
    let slot = if let Some(s) = old_slot.filter(|s| s.matches(domain, &range)) {
        s.clone()
    } else {
        SequenceSlot::prepare(
            document.clone(),
            domain,
            range,
            budget,
            cancel,
            profile.index,
        )?
    };
    let _point = budget.reserve_points(2 * size_of::<Point>(), 2)?;
    let offset =
        slot.index
            .text()
            .locate_visual(run, byte, selected.projection_part(), &mut prep)?;
    let mut p = slot.index.seek_before_with_work(offset, &mut prep)?;
    while p.offset() < offset {
        if slot.index.step(&mut p, 4096, &mut prep)?.is_none() {
            break;
        }
    }
    let target = p.line() as i64 + i64::from(delta);
    if target >= 0 && (target as usize) < slot.index.line_count() {
        let p = slot.index.seek_line_with_work(target as usize, &mut prep)?;
        if p.offset() < slot.index.text().len() {
            let (run, byte, part) = slot.index.visual_anchor_with_work(&p, &mut prep)?;
            return Ok((
                State::prepared_visual(flow.index(), run, byte, part, cancel)?,
                Some(slot),
            ));
        }
    }
    // Crossing a block/domain moves through actual Flow order; hidden callout bodies stay hidden.
    let ordinal = selected.location().ordinal();
    let mut at = ordinal;
    loop {
        at = if delta < 0 {
            if at == 0 {
                return Ok((selected.clone(), Some(slot)));
            }
            at - 1
        } else {
            at.checked_add(1).ok_or(Error::Invalid)?
        };
        if at >= flow.index().event_count() {
            return Ok((selected.clone(), Some(slot)));
        }
        let l = flow.index().location(at)?;
        if let Event::Run { run: r, .. } = flow.index().event_at(&l, &mut prep)? {
            prep.admit_structure(size_of_val_record())?;
            let rr = &document.runs()[r as usize];
            if rr.domain() != domain || rr.kind() == 5 {
                let text = if matches!(rr.kind(), 0 | 1 | 4 | 6 | 7) {
                    document.text(rr.text()).map_err(|_| Error::Invalid)?
                } else {
                    ""
                };
                let raw = if delta < 0 { text.len() } else { 0 };
                return Ok((State::prepared_run(flow.index(), r, raw, cancel)?, None));
            }
        }
    }
}
fn size_of_val_record() -> usize {
    size_of::<crate::document_model::Run>()
}
