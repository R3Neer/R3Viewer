//! Prepared textual window. No renderer, parser, TUI pump or optional color allowance.
#![allow(dead_code)] // Private b1 library; session adoption is D2c.
use crate::document_presentation_anchor::{
    CellProgress, OriginMeta, PaintRole, PresentationAnchor, RefAnchor, RefField, RefOwner,
    ResourceAltAnchor, RoleAnchor, TableCellProgress, TableResume,
};
use crate::document_provider_gate::{IndexOwner, ProviderGate, RefOwner as ProviderRefOwner};
use crate::document_roles::{self as roles, Alignment, BlockRole, ResourceKind, TaskState};
use crate::{
    document_consumer_budget::{Charge, ChargedVec, ConsumerBudget},
    document_consumer_profile::ConsumerProfile,
    document_flow::{Event, FlowHandle},
    document_model::{Document, NONE},
    document_navigation::State,
    document_text::{DocumentWork, Error, IndexProfile, Point, Relation, Result, SegmentedIndex},
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
    Text(crate::document_presentation_anchor::PaintRole),
    Decoration(u32),
    RefText(u32),
    AltText(u32),
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
// Packed cells retain physical local byte + visual affinity; uniform immutable
// run/style metadata is stored once per consecutive class, not copied per ASCII.
#[derive(Clone, Copy)]
struct PackedGlyph {
    x: u16,
    row: u16,
    width: u16,
    len: u16,
    start: u32,
    local: u32,
    class: u16,
    part: u8,
}
#[derive(Clone, Copy, PartialEq, Eq)]
struct FirstBase {
    block: u32,
    domain: u32,
    run: u32,
}
#[derive(Clone, Copy, PartialEq, Eq)]
struct GlyphClass {
    kind: CellKind,
    styles: u32,
    first: Option<FirstBase>,
    links: usize,
    relation_mask: u8,
}
struct GlyphStorage {
    cells: ChargedVec<PackedGlyph>,
    classes: ChargedVec<GlyphClass>,
}
impl GlyphStorage {
    fn new(budget: &ConsumerBudget, cells: usize, classes: usize) -> Result<Self> {
        Ok(Self {
            cells: ChargedVec::new(budget, cells)?,
            classes: ChargedVec::new(budget, classes)?,
        })
    }
    fn capacity_bytes(&self) -> Result<usize> {
        self.cells
            .capacity()
            .checked_mul(size_of::<PackedGlyph>())
            .and_then(|n| {
                self.classes
                    .capacity()
                    .checked_mul(size_of::<GlyphClass>())
                    .and_then(|c| n.checked_add(c))
            })
            .ok_or(Error::Budget)
    }
    fn len(&self) -> usize {
        self.cells.as_slice().len()
    }
    fn get(&self, index: usize) -> Option<Glyph> {
        let p = self.cells.as_slice().get(index)?;
        let c = self.classes.as_slice().get(usize::from(p.class))?;
        Some(Glyph {
            x: p.x,
            row: p.row,
            width: p.width,
            start: p.start,
            end: p.start + u32::from(p.len),
            kind: c.kind,
            styles: c.styles,
            first: c.first.map(|f| FirstComponent {
                block: f.block,
                domain: f.domain,
                run: f.run,
                local: p.local as usize,
            }),
            links: c.links,
            relation_mask: c.relation_mask,
            projection_part: p.part,
        })
    }
    fn last(&self) -> Option<Glyph> {
        self.len().checked_sub(1).and_then(|i| self.get(i))
    }
    fn class_for(&mut self, g: &Glyph, w: &mut DocumentWork<'_>) -> Result<u16> {
        let class = GlyphClass {
            kind: g.kind,
            styles: g.styles,
            first: g.first.map(|f| FirstBase {
                block: f.block,
                domain: f.domain,
                run: f.run,
            }),
            links: g.links,
            relation_mask: g.relation_mask,
        };
        // A bounded one-record cache only, no linear search or uncharged hash table.
        w.admit_structure(size_of::<GlyphClass>())?;
        if self.classes.as_slice().last() == Some(&class) {
            return u16::try_from(self.classes.as_slice().len() - 1).map_err(|_| Error::Budget);
        }
        let id = u16::try_from(self.classes.as_slice().len()).map_err(|_| Error::Budget)?;
        w.admit_structure(size_of::<GlyphClass>())?;
        self.classes.push(class)?;
        Ok(id)
    }
    fn packed(&mut self, g: Glyph, w: &mut DocumentWork<'_>) -> Result<PackedGlyph> {
        let len = g.end.checked_sub(g.start).ok_or(Error::Invalid)?;
        let len = u16::try_from(len).map_err(|_| Error::Budget)?;
        let local = u32::try_from(g.first.map_or(0, |f| f.local)).map_err(|_| Error::Invalid)?;
        let class = self.class_for(&g, w)?;
        w.admit_structure(size_of::<PackedGlyph>())?;
        Ok(PackedGlyph {
            x: g.x,
            row: g.row,
            width: g.width,
            len,
            start: g.start,
            local,
            class,
            part: g.projection_part,
        })
    }
    fn push(&mut self, g: Glyph, w: &mut DocumentWork<'_>) -> Result<()> {
        let p = self.packed(g, w)?;
        self.cells.push(p).map_err(Into::into)
    }
    fn replace_last(&mut self, g: Glyph, w: &mut DocumentWork<'_>) -> Result<()> {
        let p = self.packed(g, w)?;
        *self.cells.last_mut().ok_or(Error::Invalid)? = p;
        Ok(())
    }
}
pub(crate) struct GlyphView<'a>(&'a GlyphStorage);
impl GlyphView<'_> {
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
    pub(crate) fn get(&self, index: usize) -> Option<Glyph> {
        self.0.get(index)
    }
    pub(crate) fn last(&self) -> Option<Glyph> {
        self.0.last()
    }
    pub(crate) fn iter(&self) -> impl DoubleEndedIterator<Item = Glyph> + ExactSizeIterator + '_ {
        (0..self.len()).map(|i| self.0.get(i).expect("closed packedcell/class indices"))
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Row {
    pub first: u32,
    pub count: u32,
}
enum SlotPayload {
    Domain {
        index: IndexOwner,
        domain: u32,
        runs: Range<usize>,
    },
    Ref(ProviderRefOwner),
}
pub(crate) struct SequenceSlot {
    payload: SlotPayload,
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
        gate: Option<&Arc<ProviderGate>>,
    ) -> Result<Arc<Self>> {
        let metadata = budget
            .reserve(size_of::<Self>() - size_of::<SegmentedIndex>() + 2 * size_of::<usize>())?;
        let text = crate::document_provider_gate::TextOwner::prepare(
            document,
            domain,
            runs.clone(),
            budget,
            cancel,
            profile,
            gate,
        )?;
        let index = text.into_index(budget, cancel, profile)?;
        Ok(Arc::new(Self {
            payload: SlotPayload::Domain {
                index,
                domain,
                runs,
            },
            _metadata: metadata,
        }))
    }
    fn prepare_ref(
        document: Arc<Document>,
        id: u32,
        budget: &ConsumerBudget,
        cancel: &AtomicBool,
        profile: IndexProfile,
        gate: Option<&Arc<ProviderGate>>,
    ) -> Result<Arc<Self>> {
        let metadata = budget
            .reserve(size_of::<Self>() - size_of::<SegmentedIndex>() + 2 * size_of::<usize>())?;
        let index = ProviderRefOwner::prepare(document, id, budget, cancel, profile, gate)?;
        Ok(Arc::new(Self {
            payload: SlotPayload::Ref(index),
            _metadata: metadata,
        }))
    }
    fn matches(&self, domain: u32, runs: &Range<usize>) -> bool {
        matches!(&self.payload,SlotPayload::Domain{domain:d,runs:r,..}if *d==domain&&r==runs)
    }
    fn matches_ref(&self, id: u32) -> bool {
        matches!(&self.payload,SlotPayload::Ref(r)if r.id()==id)
    }
    fn domain_index(&self) -> Result<&SegmentedIndex> {
        match &self.payload {
            SlotPayload::Domain { index, .. } => Ok(index),
            _ => Err(Error::Invalid),
        }
    }
    fn domain(&self) -> Result<u32> {
        match &self.payload {
            SlotPayload::Domain { domain, .. } => Ok(*domain),
            _ => Err(Error::Invalid),
        }
    }
}
pub(crate) struct TextWindow {
    symbols: ChargedVec<u8>,
    glyphs: GlyphStorage,
    rows: ChargedVec<Row>,
    slot: Option<Arc<SequenceSlot>>,
    _owner: Charge,
    bytes: usize,
    pub work: usize,
    pub preparation_work: usize,
    next: Option<crate::document_view::LayoutAnchor>,
    receipt: Option<crate::document_frame_work::FrameWorkReceipt>,
    origins: ChargedVec<OriginMeta>,
    anchors: ChargedVec<Option<PresentationAnchor>>,
    presentation_next: Option<PresentationAnchor>,
    auxiliary: usize,
}
impl TextWindow {
    pub(crate) fn frame_receipt(&self) -> &crate::document_frame_work::FrameWorkReceipt {
        self.receipt.as_ref().expect("finished layout receipt")
    }
    pub(crate) fn glyph_read_bytes() -> usize {
        size_of::<PackedGlyph>() + size_of::<GlyphClass>() + size_of::<Glyph>()
    }
    pub(crate) fn presentation_next(&self) -> Option<&PresentationAnchor> {
        self.presentation_next.as_ref()
    }
    pub(crate) fn presentation_row(&self, row: usize) -> Option<&PresentationAnchor> {
        self.anchors.as_slice().get(row)?.as_ref()
    }
    pub(crate) fn cell_anchor(
        &self,
        index: usize,
        w: &mut DocumentWork<'_>,
    ) -> Result<Option<PresentationAnchor>> {
        w.admit_structure(Self::glyph_read_bytes() + size_of::<OriginMeta>())?;
        let p = self
            .glyphs
            .cells
            .as_slice()
            .get(index)
            .ok_or(Error::Invalid)?;
        let g = self.glyphs.get(index).ok_or(Error::Invalid)?;
        Ok(match g.kind {
            CellKind::Decoration(id) => match *self
                .origins
                .as_slice()
                .get(id as usize)
                .ok_or(Error::Invalid)?
            {
                meta @ OriginMeta::Role { .. } => {
                    Some(PresentationAnchor::Role(meta.role_anchor(p.local as u16)?))
                }
                _ => return Err(Error::Invalid),
            },
            CellKind::AltText(id) => match *self
                .origins
                .as_slice()
                .get(id as usize)
                .ok_or(Error::Invalid)?
            {
                OriginMeta::Alt {
                    block,
                    run,
                    resource,
                    domain,
                    ..
                } => {
                    let first = g.first.ok_or(Error::Invalid)?;
                    Some(PresentationAnchor::ResourceAlt(
                        ResourceAltAnchor::from_provider(
                            block,
                            run,
                            resource,
                            domain,
                            crate::document_view::LayoutAnchor {
                                raw: crate::document_view::RawAnchor {
                                    block,
                                    run: first.run,
                                    byte: first.local,
                                },
                                projection_part: g.projection_part,
                            },
                        ),
                    ))
                }
                _ => return Err(Error::Invalid),
            },
            CellKind::RefText(id) => match *self
                .origins
                .as_slice()
                .get(id as usize)
                .ok_or(Error::Invalid)?
            {
                OriginMeta::Ref {
                    owner, text_ref, ..
                } => Some(PresentationAnchor::Ref(RefAnchor::from_provider(
                    owner,
                    text_ref,
                    p.local as usize,
                    p.part,
                ))),
                _ => return Err(Error::Invalid),
            },
            _ => g.first.map(|f| {
                PresentationAnchor::Domain(crate::document_view::LayoutAnchor {
                    raw: crate::document_view::RawAnchor {
                        block: f.block,
                        run: f.run,
                        byte: f.local,
                    },
                    projection_part: g.projection_part,
                })
            }),
        })
    }
    pub(crate) fn paint(&self, g: Glyph) -> Result<PaintRole> {
        match g.kind {
            CellKind::Text(p) => Ok(p),
            CellKind::Glyph => Ok(PaintRole::Value),
            CellKind::Decoration(id) | CellKind::RefText(id) | CellKind::AltText(id) => match self
                .origins
                .as_slice()
                .get(id as usize)
                .ok_or(Error::Invalid)?
            {
                OriginMeta::Role { paint, .. }
                | OriginMeta::Ref { paint, .. }
                | OriginMeta::Alt { paint, .. } => Ok(*paint),
            },
            _ => Err(Error::Invalid),
        }
    }
    pub(crate) fn symbols(&self) -> &str {
        std::str::from_utf8(self.symbols.as_slice()).expect("complete projected UTF8")
    }
    pub(crate) fn glyphs(&self) -> GlyphView<'_> {
        GlyphView(&self.glyphs)
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
        let g = (r.first as usize..r.first as usize + r.count as usize)
            .find_map(|i| self.glyphs.get(i).filter(|g| g.first.is_some()))?;
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
        receipt: crate::document_frame_work::FrameWorkReceipt,
    },
}
impl LayoutWindow {
    pub(crate) fn frame_receipt(&self) -> &crate::document_frame_work::FrameWorkReceipt {
        match self {
            Self::Text(w) => w.frame_receipt(),
            Self::Reference { receipt, .. } => receipt,
        }
    }
    pub(crate) fn reference(
        reason: ReferenceReason,
        g: Geometry,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        let mut op = crate::document_frame_work::LayoutWork::reference(cancel, g)?;
        op.work().admit_structure(REFERENCE_WORK)?;
        let work = op.spent();
        Ok(Self::Reference {
            reason,
            work,
            receipt: op.finish(),
        })
    }

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
    budget: ConsumerBudget,
    provider_gate: Option<Arc<ProviderGate>>,
    old_bytes: usize,
    prefix: [u8; 256],
    prefix_len: usize,
    hanging: [u8; 256],
    paint: PaintRole,
    prefix_owner: u32,
    marked: [u32; 65],
    _scratch: Charge,
    defer_rows: bool,
    cell_left: usize,
    cell_width: usize,
    table_progress: Option<ChargedVec<TableCellProgress>>,
    table_cell: usize,
    table_base: usize,
    table_columns: usize,
    resource_alt: Option<(u32, u32, u32, u32, u32)>,
    event_run: u32,
    event_ordinal: u64,
    role_resume: Option<RoleAnchor>,
}
impl WindowBuilder {
    pub(crate) fn with_provider_gate(mut self, gate: Option<Arc<ProviderGate>>) -> Self {
        self.provider_gate = gate;
        self
    }

    pub(crate) fn new(budget: &ConsumerBudget, g: Geometry, old_bytes: usize) -> Result<Self> {
        if g.width == 0 || g.height == 0 {
            return Err(Error::Budget);
        }
        let cells = usize::from(g.width)
            .checked_mul(usize::from(g.height))
            .ok_or(Error::Budget)?;
        let symbols = cells.checked_mul(4).ok_or(Error::Budget)?.max(16384);
        let classes = cells.min(16);
        let requested = size_of::<TextWindow>()
            .checked_add(symbols)
            .and_then(|n| n.checked_add(cells.checked_mul(size_of::<PackedGlyph>())?))
            .and_then(|n| n.checked_add(classes.checked_mul(size_of::<GlyphClass>())?))
            .and_then(|n| n.checked_add((usize::from(g.height) + 1).checked_mul(size_of::<Row>())?))
            .and_then(|n| n.checked_add(16 * size_of::<OriginMeta>()))
            .and_then(|n| {
                n.checked_add(usize::from(g.height) * size_of::<Option<PresentationAnchor>>())
            })
            .ok_or(Error::Budget)?;
        if requested > WINDOW_BYTES
            || old_bytes
                .checked_add(requested)
                .is_none_or(|n| n > FRAME_BYTES)
        {
            return Err(Error::Budget);
        }
        let owner = budget.reserve(size_of::<TextWindow>())?;
        let scratch = budget.reserve(size_of::<Self>() - size_of::<TextWindow>())?;
        let symbols = ChargedVec::new(budget, symbols)?;
        let glyphs = GlyphStorage::new(budget, cells, classes)?;
        let rows = ChargedVec::new(budget, usize::from(g.height) + 1)?;
        let origins = ChargedVec::new(budget, 16)?;
        let mut anchors = ChargedVec::new(budget, usize::from(g.height))?;
        for _ in 0..g.height {
            anchors.push(None)?;
        }
        let bytes = size_of::<TextWindow>()
            .checked_add(symbols.capacity())
            .and_then(|n| n.checked_add(glyphs.capacity_bytes().ok()?))
            .and_then(|n| n.checked_add(rows.capacity().checked_mul(size_of::<Row>())?))
            .and_then(|n| n.checked_add(origins.capacity() * size_of::<OriginMeta>()))
            .and_then(|n| {
                n.checked_add(anchors.capacity() * size_of::<Option<PresentationAnchor>>())
            })
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
                receipt: None,
                origins,
                anchors,
                presentation_next: None,
                auxiliary: 0,
            },
            column: 0,
            row: 0,
            row_start: 0,
            geometry: g,
            budget: budget.clone(),
            provider_gate: None,
            old_bytes,
            prefix: [0; 256],
            prefix_len: 0,
            hanging: [0; 256],
            paint: PaintRole::Value,
            prefix_owner: 0,
            marked: [NONE; 65],
            _scratch: scratch,
            defer_rows: false,
            cell_left: 0,
            cell_width: usize::from(g.width),
            table_progress: None,
            table_cell: 0,
            table_base: 0,
            table_columns: 0,
            resource_alt: None,
            event_run: NONE,
            event_ordinal: 0,
            role_resume: None,
        })
    }
    #[cfg(test)]
    pub(crate) fn storage_receipt(&self) -> [usize; 8] {
        [
            size_of::<PackedGlyph>(),
            size_of::<GlyphClass>(),
            self.window.bytes,
            self.window.symbols.capacity(),
            self.window.glyphs.cells.capacity(),
            self.window.glyphs.classes.capacity(),
            self.window.rows.capacity(),
            size_of::<Row>(),
        ]
    }
    pub(crate) fn retain_candidate(&mut self, slot: Option<Arc<SequenceSlot>>) {
        self.window.slot = slot;
    }
    fn newline(&mut self, w: &mut DocumentWork<'_>) -> Result<bool> {
        w.admit_structure(size_of::<Row>())?;
        if !self.defer_rows {
            self.window.rows.push(Row {
                first: u32::try_from(self.row_start).map_err(|_| Error::Invalid)?,
                count: u32::try_from(self.window.glyphs.len() - self.row_start)
                    .map_err(|_| Error::Invalid)?,
            })?;
        }
        self.row += 1;
        self.prefix = self.hanging;
        self.column = 0;
        self.row_start = self.window.glyphs.len();
        Ok(self.row < usize::from(self.geometry.height))
    }
    fn refresh_bytes(&mut self) -> Result<()> {
        self.window.bytes = size_of::<TextWindow>()
            + self.window.symbols.capacity()
            + self.window.glyphs.capacity_bytes()?
            + self.window.rows.capacity() * size_of::<Row>()
            + self.window.origins.capacity() * size_of::<OriginMeta>()
            + self.window.anchors.capacity() * size_of::<Option<PresentationAnchor>>()
            + self.window.auxiliary;
        if self.window.bytes > WINDOW_BYTES
            || self
                .old_bytes
                .checked_add(self.window.bytes)
                .is_none_or(|n| n > FRAME_BYTES)
        {
            return Err(Error::Budget);
        }
        Ok(())
    }
    fn ensure_capacity(&mut self, symbols: usize, w: &mut DocumentWork<'_>) -> Result<()> {
        self.refresh_bytes()?;
        if self.window.glyphs.classes.as_slice().len() == self.window.glyphs.classes.capacity() {
            let old = self.window.glyphs.classes.capacity() * size_of::<GlyphClass>();
            let room = WINDOW_BYTES
                .min(FRAME_BYTES.saturating_sub(self.old_bytes))
                .saturating_sub(self.window.bytes)
                + old;
            let capacity = (self.window.glyphs.classes.capacity() * 2)
                .max(1)
                .min(self.window.glyphs.cells.capacity());
            self.window
                .glyphs
                .classes
                .grow_copy(&self.budget, capacity, room, w)?;
            self.refresh_bytes()?;
        }
        if self
            .window
            .symbols
            .as_slice()
            .len()
            .checked_add(symbols)
            .is_none_or(|n| n > self.window.symbols.capacity())
        {
            let old = self.window.symbols.capacity();
            let room = WINDOW_BYTES
                .min(FRAME_BYTES.saturating_sub(self.old_bytes))
                .saturating_sub(self.window.bytes)
                + old;
            let needed = self
                .window
                .symbols
                .as_slice()
                .len()
                .checked_add(symbols)
                .ok_or(Error::Budget)?;
            self.window
                .symbols
                .grow_copy(&self.budget, (old * 2).max(needed), room, w)?;
            self.refresh_bytes()?;
        }
        Ok(())
    }
    fn origin(&mut self, meta: OriginMeta, w: &mut DocumentWork<'_>) -> Result<u32> {
        w.admit_structure(size_of::<OriginMeta>())?;
        if self.window.origins.as_slice().last() == Some(&meta) {
            return u32::try_from(self.window.origins.as_slice().len() - 1)
                .map_err(|_| Error::Budget);
        }
        if self.window.origins.as_slice().len() == self.window.origins.capacity() {
            self.refresh_bytes()?;
            let old = self.window.origins.capacity() * size_of::<OriginMeta>();
            let room = WINDOW_BYTES
                .min(FRAME_BYTES.saturating_sub(self.old_bytes))
                .saturating_sub(self.window.bytes)
                + old;
            self.window.origins.grow_copy(
                &self.budget,
                self.window.origins.capacity() * 2,
                room,
                w,
            )?;
        }
        let id = u32::try_from(self.window.origins.as_slice().len()).map_err(|_| Error::Budget)?;
        w.admit_structure(size_of::<OriginMeta>())?;
        self.window.origins.push(meta)?;
        self.refresh_bytes()?;
        Ok(id)
    }
    fn content_anchor(
        &mut self,
        mut anchor: PresentationAnchor,
        w: &mut DocumentWork<'_>,
    ) -> Result<()> {
        w.admit_structure(8)?;
        if let PresentationAnchor::Domain(a) = anchor
            && let Some((block, run, resource, domain, _)) = self.resource_alt
        {
            anchor = PresentationAnchor::ResourceAlt(ResourceAltAnchor::from_provider(
                block, run, resource, domain, a,
            ));
        }
        if let PresentationAnchor::Domain(a) = &anchor
            && let Some(progress) = self.table_progress.as_mut()
        {
            let at = (self.row - self.table_base) * self.table_columns + self.table_cell;
            if let Some(v) = progress.as_mut_slice().get_mut(at)
                && matches!(v.progress, CellProgress::Done)
            {
                w.admit_structure(size_of::<TableCellProgress>())?;
                v.cell = a.raw.block;
                v.progress = CellProgress::At(*a);
            }
        }

        if let Some(place) = self.window.anchors.as_mut_slice().get_mut(self.row)
            && place
                .as_ref()
                .is_none_or(|a| matches!(a, PresentationAnchor::Role(_)))
        {
            w.admit_structure(size_of::<PresentationAnchor>())?;
            *place = Some(anchor);
        }
        Ok(())
    }
    fn decoration(
        &mut self,
        text: &str,
        block: u32,
        phase: u8,
        w: &mut DocumentWork<'_>,
    ) -> Result<bool> {
        let id = self.origin(
            OriginMeta::Role {
                block,
                run: self.event_run,
                ordinal: self.event_ordinal,
                phase,
                paint: PaintRole::Secondary,
            },
            w,
        )?;
        let skip = self
            .role_resume
            .filter(|r| r.block() == block && r.phase() == phase)
            .map_or(0, |r| usize::from(r.offset()));
        for (offset, &byte) in text.as_bytes().iter().enumerate().skip(skip) {
            if !byte.is_ascii() || byte.is_ascii_control() {
                return Err(Error::Invalid);
            }
            if self.column >= self.cell_width && !self.newline(w)? {
                self.window.presentation_next = Some(PresentationAnchor::Role(
                    OriginMeta::Role {
                        block,
                        run: self.event_run,
                        ordinal: self.event_ordinal,
                        phase,
                        paint: PaintRole::Secondary,
                    }
                    .role_anchor(offset as u16)?,
                ));
                return Ok(false);
            }
            self.ensure_capacity(1, w)?;
            w.admit_structure(1)?;
            let start = self.window.symbols.as_slice().len();
            self.window.symbols.push(byte)?;
            let g = Glyph {
                x: (self.column + self.cell_left) as u16,
                row: self.row as u16,
                width: 1,
                start: start as u32,
                end: (start + 1) as u32,
                kind: CellKind::Decoration(id),
                styles: 0,
                first: None,
                links: 0,
                relation_mask: 0,
                projection_part: 0,
            };
            self.window.glyphs.push(g, w)?;
            self.window
                .glyphs
                .cells
                .last_mut()
                .ok_or(Error::Invalid)?
                .local = offset as u32;
            if self.window.anchors.as_slice()[self.row].is_none() {
                self.window.anchors.as_mut_slice()[self.row] = Some(PresentationAnchor::Role(
                    OriginMeta::Role {
                        block,
                        run: self.event_run,
                        ordinal: self.event_ordinal,
                        phase,
                        paint: PaintRole::Secondary,
                    }
                    .role_anchor(offset as u16)?,
                ));
            }
            self.column += 1;
        }
        Ok(true)
    }
    fn ensure_prefix(&mut self, w: &mut DocumentWork<'_>) -> Result<()> {
        if self.column == 0 && self.prefix_len > 0 {
            if self.prefix_len >= self.cell_width {
                return Err(Error::UnsupportedRun(u32::MAX - 2));
            }
            let prefix = self.prefix;
            let text =
                std::str::from_utf8(&prefix[..self.prefix_len]).map_err(|_| Error::Invalid)?;
            self.decoration(text, self.prefix_owner, 0, w)?;
        }
        Ok(())
    }
    fn append_prefix(&mut self, text: &str, hang: bool) -> Result<()> {
        let end = self
            .prefix_len
            .checked_add(text.len())
            .filter(|n| *n <= self.prefix.len())
            .ok_or(Error::UnsupportedRun(u32::MAX - 2))?;
        self.prefix[self.prefix_len..end].copy_from_slice(text.as_bytes());
        if hang {
            self.hanging[self.prefix_len..end].fill(b' ');
        } else {
            self.hanging[self.prefix_len..end].copy_from_slice(text.as_bytes());
        }
        self.prefix_len = end;
        Ok(())
    }
    fn context(
        &mut self,
        document: &Document,
        cursor: &crate::document_flow::FlowCursor,
        block: u32,
        continued: bool,
        w: &mut DocumentWork<'_>,
    ) -> Result<()> {
        self.prefix_len = 0;
        self.prefix.fill(0);
        self.hanging.fill(0);
        self.prefix_owner = block;
        self.paint = PaintRole::Value;
        let mut parent: Option<(u32, u32)> = None;
        let mut list_depth = 0;
        for (depth, (id, next, stage, _)) in cursor.context(w)?.enumerate() {
            let role = roles::block_role(document, id, w)?;
            match role {
                BlockRole::Quote => self.append_prefix("> ", false)?,
                BlockRole::Callout { .. } if stage == 1 => self.append_prefix("| ", false)?,
                BlockRole::List { .. } => {
                    list_depth += 1;
                    if list_depth > 1 {
                        self.append_prefix("  ", false)?;
                    }
                }
                BlockRole::Item { task } => {
                    let seen = self.marked[depth] == id || continued;
                    self.marked[depth] = id;
                    let (parent_id, parent_next) = parent.ok_or(Error::Invalid)?;
                    if let BlockRole::List { ordered, start, .. } =
                        roles::block_role(document, parent_id, w)?
                    {
                        let b = &document.blocks()[parent_id as usize];
                        w.admit_structure(size_of::<crate::document_model::Item>())?;
                        let item = document
                            .items()
                            .get((b.first_item() + parent_next.saturating_sub(1)) as usize)
                            .ok_or(Error::Invalid)?;
                        if item.target() != id || item.kind() != 0 {
                            return Err(Error::Invalid);
                        }
                        let mut digits = [0u8; 20];
                        let mut buf = [0u8; 28];
                        let mut len;
                        if ordered {
                            let d = ascii_number(
                                u64::from(start)
                                    .checked_add(u64::from(item.ordinal()))
                                    .ok_or(Error::Invalid)?,
                                &mut digits,
                            );
                            buf[..d.len()].copy_from_slice(d.as_bytes());
                            len = d.len();
                            buf[len..len + 2].copy_from_slice(b". ");
                            len += 2;
                        } else {
                            buf[..2].copy_from_slice(b"- ");
                            len = 2;
                        }
                        let task = match task {
                            TaskState::Absent => "",
                            TaskState::Unchecked => "[ ] ",
                            TaskState::Checked => "[x] ",
                        };
                        buf[len..len + task.len()].copy_from_slice(task.as_bytes());
                        len += task.len();
                        if seen {
                            buf[..len].fill(b' ');
                        }
                        self.append_prefix(
                            std::str::from_utf8(&buf[..len]).map_err(|_| Error::Invalid)?,
                            true,
                        )?;
                    }
                }
                BlockRole::Heading { level } if id == block => {
                    self.paint = PaintRole::Heading;
                    let mut marker = [b'#'; 7];
                    let len = (level as usize).min(6) + 1;
                    marker[len - 1] = b' ';
                    if continued || self.marked[depth] == id {
                        marker[..len].fill(b' ');
                    }
                    self.marked[depth] = id;
                    self.append_prefix(
                        std::str::from_utf8(&marker[..len]).map_err(|_| Error::Invalid)?,
                        true,
                    )?;
                }
                BlockRole::Code { .. } | BlockRole::Frontmatter if id == block => {
                    self.paint = PaintRole::Code
                }
                BlockRole::Row { header: true } => self.paint = PaintRole::Heading,
                _ => {}
            }
            parent = Some((id, next));
        }
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    fn table_row(
        &mut self,
        document: Arc<Document>,
        flow: &FlowHandle,
        cursor: &crate::document_flow::FlowCursor,
        row: u32,
        selected: Option<crate::document_view::LayoutAnchor>,
        resume: Option<&TableResume>,
        budget: &ConsumerBudget,
        cancel: &AtomicBool,
        profile: IndexProfile,
        w: &mut DocumentWork<'_>,
    ) -> Result<bool> {
        if self.column > 0 && !self.newline(w)? {
            return Ok(false);
        }
        let record = &document.blocks()[row as usize];
        w.admit_structure(size_of::<crate::document_model::Block>())?;
        let BlockRole::Row { header } = roles::block_role(&document, row, w)? else {
            return Err(Error::Invalid);
        };
        let BlockRole::Table { columns } = roles::block_role(&document, record.parent(), w)? else {
            return Err(Error::Invalid);
        };
        let n = columns as usize;
        let threshold = n
            .checked_mul(8)
            .and_then(|v| v.checked_add(n.saturating_sub(1) * 3))
            .ok_or(Error::Budget)?;
        let vertical = usize::from(self.geometry.width) < threshold;
        let row_ordinal = cursor
            .context(w)?
            .find(|(b, _, _, _)| *b == row)
            .map(|(_, _, _, ordinal)| ordinal)
            .ok_or(Error::Invalid)?;
        if self
            .role_resume
            .is_some_and(|a| a.block() == row && a.phase() == 50)
            && !vertical
        {
            self.event_ordinal = row_ordinal;
            self.event_run = NONE;
            let available =
                usize::from(self.geometry.width).saturating_sub(n.saturating_sub(1) * 3);
            let unit = available / n;
            for i in 0..usize::from(self.geometry.width) {
                let ch = if i == unit || i == 2 * unit + 3 {
                    "+"
                } else {
                    "-"
                };
                if !self.decoration(ch, row, 50, w)? {
                    return Ok(false);
                }
            }
            return self.newline(w);
        }
        let mut local_cursor = flow.index().copy_current(cursor, w)?;
        let base = self.row;
        let height = usize::from(self.geometry.height) - base;
        let mut progress = ChargedVec::new(
            budget,
            if vertical {
                height
            } else {
                height.checked_mul(n).ok_or(Error::Budget)?
            },
        )?;
        if progress.capacity() * size_of::<TableCellProgress>() + self.window.bytes > WINDOW_BYTES {
            return Err(Error::Budget);
        }
        for _ in 0..progress.capacity() {
            w.admit_structure(size_of::<TableCellProgress>())?;
            progress.push(TableCellProgress {
                cell: NONE,
                progress: CellProgress::Done,
            })?;
        }
        self.table_progress = Some(progress);
        self.table_base = base;
        self.table_columns = if vertical { 1 } else { n };
        let mut continuation = ChargedVec::new(budget, if vertical { 1 } else { n })?;
        let mut max_row = base;
        let mut incomplete = false;
        let mut started = selected.is_none() && resume.is_none();
        let mut next_cell = None;
        let mut left = 0usize;
        let available = usize::from(self.geometry.width).saturating_sub(n.saturating_sub(1) * 3);
        let unit = available.checked_div(n).unwrap_or(0);
        let residual = if n == 0 { 0 } else { available % n };
        for item in &document.items()
            [record.first_item() as usize..(record.first_item() + record.item_count()) as usize]
        {
            w.admit_structure(size_of::<crate::document_model::Item>())?;
            if item.kind() != 0 {
                return Err(Error::Invalid);
            }
            let cell = item.target();
            let c = &document.blocks()[cell as usize];
            w.admit_structure(size_of::<crate::document_model::Block>())?;
            if c.kind() != 9 {
                return Err(Error::Invalid);
            }
            let ordinal = item.ordinal() as usize;
            if !vertical && let Some(p) = self.table_progress.as_mut() {
                for r in 0..height {
                    let at = r * self.table_columns + ordinal;
                    w.admit_structure(size_of::<TableCellProgress>())?;
                    p.as_mut_slice()[at].cell = cell;
                }
            }
            let mut stored = None;
            if let Some(resume) = resume {
                for p in resume.cells() {
                    w.admit_structure(size_of::<TableCellProgress>())?;
                    if p.cell == cell {
                        stored = Some(p.progress);
                        break;
                    }
                }
            }
            if selected.is_some_and(|a| a.raw.block == cell)
                || resume.is_some_and(|r| r.selected() == cell)
            {
                started = true;
            }
            let previous = !started && stored.is_none();
            let start = match stored {
                Some(CellProgress::Done) => {
                    if !vertical {
                        w.admit_structure(size_of::<TableCellProgress>())?;
                        continuation.push(TableCellProgress {
                            cell,
                            progress: CellProgress::Done,
                        })?;
                    }
                    left += unit + usize::from(ordinal < residual) + 3;
                    continue;
                }
                Some(CellProgress::At(a)) => Some(a),
                None if previous => {
                    if !vertical {
                        w.admit_structure(size_of::<TableCellProgress>())?;
                        continuation.push(TableCellProgress {
                            cell,
                            progress: CellProgress::Done,
                        })?;
                    }
                    left += unit + usize::from(ordinal < residual) + 3;
                    continue;
                }
                None => selected.filter(|a| a.raw.block == cell),
            };
            loop {
                if let Some(ordinal) = local_cursor
                    .context(w)?
                    .find(|(b, _, _, _)| *b == cell)
                    .map(|(_, _, _, ordinal)| ordinal)
                {
                    self.event_ordinal = ordinal;
                    self.event_run = NONE;
                    break;
                }
                if flow.index().next_event(&mut local_cursor, w)?.is_none() {
                    return Err(Error::Invalid);
                }
            }
            self.prefix_len = 0;
            self.prefix.fill(0);
            self.hanging.fill(0);
            self.paint = if header {
                PaintRole::Heading
            } else {
                PaintRole::Value
            };
            self.prefix_owner = cell;
            self.table_cell = if vertical { 0 } else { ordinal };
            self.cell_left = if vertical { 0 } else { left };
            self.cell_width = if vertical {
                usize::from(self.geometry.width)
            } else {
                unit + usize::from(ordinal < residual)
            };
            if !vertical {
                self.row = base;
                self.column = 0;
                self.defer_rows = true;
            }
            if vertical {
                if self.row >= usize::from(self.geometry.height) {
                    next_cell = Some(cell);
                    incomplete = true;
                    break;
                }
                let mut digits = [0; 20];
                let number = ascii_number(u64::from(item.ordinal()) + 1, &mut digits);
                let mut prefix = [0u8; 40];
                let label = b"column ";
                prefix[..label.len()].copy_from_slice(label);
                prefix[label.len()..label.len() + number.len()].copy_from_slice(number.as_bytes());
                let end = label.len() + number.len();
                prefix[end..end + 2].copy_from_slice(b": ");
                self.append_prefix(
                    std::str::from_utf8(&prefix[..end + 2]).map_err(|_| Error::Invalid)?,
                    true,
                )?;
            }
            let domain = c.domain();
            let d = &document.domains()[domain as usize];
            w.admit_structure(size_of::<crate::document_model::Domain>())?;
            let first_glyph = self.window.glyphs.len();
            let mut completed = true;
            self.window.next = None;
            if d.run_count() > 0 {
                self.window.slot = None;
                let slot = SequenceSlot::prepare(
                    document.clone(),
                    domain,
                    0..d.run_count() as usize,
                    budget,
                    cancel,
                    profile,
                    self.provider_gate.as_ref(),
                )?;
                let anchor = start.unwrap_or(crate::document_view::LayoutAnchor {
                    raw: crate::document_view::RawAnchor {
                        block: cell,
                        run: d.first_run(),
                        byte: 0,
                    },
                    projection_part: 0,
                });
                completed = self.sequence(
                    slot,
                    anchor.raw.run,
                    anchor.raw.byte,
                    anchor.projection_part,
                    false,
                    cell,
                    budget,
                    cancel,
                    w,
                )?;
            }
            if !vertical || !completed {
                let progress = if let Some(a) = self.window.next {
                    CellProgress::At(a)
                } else {
                    CellProgress::Done
                };
                w.admit_structure(size_of::<TableCellProgress>())?;
                continuation.push(TableCellProgress { cell, progress })?;
            }
            max_row = max_row.max(self.row + usize::from(self.column > 0));
            if !vertical {
                let alignment = match roles::block_role(&document, cell, w)? {
                    BlockRole::Cell { alignment } => alignment,
                    _ => return Err(Error::Invalid),
                };
                for r in base..max_row.min(usize::from(self.geometry.height)) {
                    let mut end = left;
                    for i in first_glyph..self.window.glyphs.len() {
                        w.admit_structure(TextWindow::glyph_read_bytes())?;
                        let g = self.window.glyphs.get(i).ok_or(Error::Invalid)?;
                        if g.row as usize == r {
                            end = end.max(usize::from(g.x) + usize::from(g.width));
                        }
                    }
                    let used = end.saturating_sub(left);
                    let shift = match alignment {
                        Alignment::Center => self.cell_width.saturating_sub(used) / 2,
                        Alignment::Right => self.cell_width.saturating_sub(used),
                        _ => 0,
                    };
                    if shift > 0 {
                        for p in &mut self.window.glyphs.cells.as_mut_slice()[first_glyph..] {
                            w.admit_structure(size_of::<PackedGlyph>())?;
                            if p.row as usize == r {
                                p.x = p.x.checked_add(shift as u16).ok_or(Error::Invalid)?;
                            }
                        }
                    }
                }
            } else if completed && self.column > 0 {
                completed = self.newline(w)?;
            }
            incomplete |= !completed;
            left += unit + usize::from(ordinal < residual) + 3;
            if vertical && !completed && self.window.next.is_some() {
                break;
            }
        }
        self.defer_rows = false;
        self.cell_left = 0;
        self.cell_width = usize::from(self.geometry.width);
        self.prefix_len = 0;
        self.paint = PaintRole::Value;
        self.column = 0;
        if !vertical {
            self.row = max_row.min(usize::from(self.geometry.height));
            for _ in base..self.row {
                w.admit_structure(size_of::<Row>())?;
                self.window.rows.push(Row { first: 0, count: 0 })?;
            }
        }
        if let Some(progress) = self.table_progress.take() {
            for r in base..max_row.min(usize::from(self.geometry.height)) {
                let start = (r - base) * self.table_columns;
                let values = &progress.as_slice()[start..start + self.table_columns];
                w.admit_structure(std::mem::size_of_val(values))?;
                let Some(chosen) = values
                    .iter()
                    .find(|p| matches!(p.progress, CellProgress::At(_)))
                else {
                    continue;
                };
                let bytes = TableResume::requested_bytes(values.len())?;
                if self
                    .window
                    .bytes
                    .checked_add(bytes)
                    .and_then(|n| {
                        n.checked_add(progress.capacity() * size_of::<TableCellProgress>())
                    })
                    .is_none_or(|n| n > WINDOW_BYTES)
                {
                    return Err(Error::Budget);
                }
                let a = TableResume::new(&document, row, chosen.cell, values, budget, w)?;
                self.window.auxiliary = self
                    .window
                    .auxiliary
                    .checked_add(a.bytes())
                    .ok_or(Error::Budget)?;
                self.window.anchors.as_mut_slice()[r] = Some(PresentationAnchor::Table(a));
                self.refresh_bytes()?;
            }
        }
        if incomplete {
            w.admit_structure(std::mem::size_of_val(continuation.as_slice()))?;
            if let Some(chosen) = continuation
                .as_slice()
                .iter()
                .find(|p| matches!(p.progress, CellProgress::At(_)))
            {
                let peak = TableResume::requested_bytes(continuation.as_slice().len())?
                    .checked_add(continuation.capacity() * size_of::<TableCellProgress>())
                    .and_then(|v| v.checked_add(self.window.bytes))
                    .ok_or(Error::Budget)?;
                if peak > WINDOW_BYTES {
                    return Err(Error::Budget);
                }
                let a = TableResume::new(
                    &document,
                    row,
                    chosen.cell,
                    continuation.as_slice(),
                    budget,
                    w,
                )?;
                self.window.auxiliary = self
                    .window
                    .auxiliary
                    .checked_add(a.bytes())
                    .ok_or(Error::Budget)?;
                self.refresh_bytes()?;
                self.window.presentation_next = Some(PresentationAnchor::Table(a));
            } else if let Some(cell) = next_cell {
                let c = &document.blocks()[cell as usize];
                let d = &document.domains()[c.domain() as usize];
                self.window.presentation_next = Some(PresentationAnchor::Domain(
                    crate::document_view::LayoutAnchor {
                        raw: crate::document_view::RawAnchor {
                            block: cell,
                            run: d.first_run(),
                            byte: 0,
                        },
                        projection_part: 0,
                    },
                ));
            }
            self.window.next = None;
            return Ok(false);
        }
        self.event_ordinal = row_ordinal;
        self.event_run = NONE;
        if header && !vertical && self.row < usize::from(self.geometry.height) {
            for i in 0..usize::from(self.geometry.width) {
                let ch = if i == unit || i == 2 * unit + 3 {
                    "+"
                } else {
                    "-"
                };
                if !self.decoration(ch, row, 50, w)? {
                    return Ok(false);
                }
            }
            if !self.newline(w)? {
                return Ok(false);
            }
        } else if vertical && self.row < usize::from(self.geometry.height) && !self.newline(w)? {
            return Ok(false);
        }
        Ok(self.row < usize::from(self.geometry.height))
    }
    #[allow(clippy::too_many_arguments)]
    fn ref_text(
        &mut self,
        document: Arc<Document>,
        owner: RefOwner,
        id: u32,
        resume: Option<RefAnchor>,
        budget: &ConsumerBudget,
        cancel: &AtomicBool,
        profile: IndexProfile,
        w: &mut DocumentWork<'_>,
    ) -> Result<bool> {
        let anchor = RefAnchor::checked(
            &document,
            owner,
            id,
            resume.map_or(0, |a| a.local()),
            resume.map_or(0, |a| a.part()),
            w,
        )?;
        let slot = if let Some(s) = self.window.slot.as_ref().filter(|s| s.matches_ref(id)) {
            s.clone()
        } else {
            self.window.slot = None;
            SequenceSlot::prepare_ref(
                document.clone(),
                id,
                budget,
                cancel,
                profile,
                self.provider_gate.as_ref(),
            )?
        };
        let SlotPayload::Ref(index) = &slot.payload else {
            return Err(Error::Invalid);
        };
        let _points = budget.reserve_points(2 * size_of::<Point>(), 2)?;
        let mut point = index.seek_visual(anchor.local(), anchor.part(), w)?;
        self.window.slot = Some(slot.clone());
        let meta = self.origin(
            OriginMeta::Ref {
                owner,
                text_ref: id,
                paint: PaintRole::Secondary,
            },
            w,
        )?;
        loop {
            let (local, part) = index.visual_local(&point, w)?;
            let before = RefAnchor::checked(&document, owner, id, local, part, w)?;
            let Some(resolved) = index.step_resolved(&mut point, 4, w)? else {
                break;
            };
            let unit = resolved.unit();
            if unit.oversized {
                return Err(Error::UnsupportedRun(u32::MAX));
            }
            if unit.newline {
                if !self.newline(w)? {
                    let (local, part) = index.visual_local(&point, w)?;
                    self.window.presentation_next = Some(PresentationAnchor::Ref(
                        RefAnchor::checked(&document, owner, id, local, part, w)?,
                    ));
                    return Ok(false);
                }
                continue;
            }
            let mut width = unit.width;
            let mut tab = false;
            resolved.visit_projected(w, |s| {
                tab = s == "\t";
                Ok(())
            })?;
            if tab {
                width = 8 - point.column().saturating_sub(unit.width) % 8;
            }
            let compatible = self
                .window
                .glyphs
                .last()
                .filter(|g| g.row as usize == self.row && g.kind == CellKind::RefText(meta));
            if width == 0 && compatible.is_none() {
                return Err(Error::UnsupportedRun(u32::MAX - 1));
            }
            if width > self.cell_width {
                return Err(Error::UnsupportedRun(u32::MAX - 2));
            }
            if self.column + width > self.cell_width && !self.newline(w)? {
                self.window.presentation_next = Some(PresentationAnchor::Ref(before));
                return Ok(false);
            }
            self.ensure_capacity(unit.projected.len().max(width), w)?;
            self.content_anchor(PresentationAnchor::Ref(before), w)?;
            let start = self.window.symbols.as_slice().len();
            w.admit_structure(if tab { width } else { unit.projected.len() })?;
            if tab {
                for _ in 0..width {
                    self.window.symbols.push(b' ')?;
                }
            } else {
                resolved.visit_projected(w, |s| {
                    for &b in s.as_bytes() {
                        self.window.symbols.push(b)?;
                    }
                    Ok(())
                })?;
            }
            let end = self.window.symbols.as_slice().len();
            let mut mask = 0;
            resolved.visit_coverage(w, |c| {
                mask |= match c.relation {
                    Relation::ExactSource => 1,
                    Relation::ControlProjection => 2,
                    Relation::EnclosingToken => 4,
                    Relation::Synthetic => 8,
                };
                Ok(())
            })?;
            if width == 0 {
                let mut old = compatible.ok_or(Error::Invalid)?;
                if end - old.start as usize > 16384 {
                    return Err(Error::UnsupportedRun(u32::MAX));
                }
                old.end = end as u32;
                old.relation_mask |= mask;
                let local = self
                    .window
                    .glyphs
                    .cells
                    .as_slice()
                    .last()
                    .ok_or(Error::Invalid)?
                    .local;
                self.window.glyphs.replace_last(old, w)?;
                self.window
                    .glyphs
                    .cells
                    .last_mut()
                    .ok_or(Error::Invalid)?
                    .local = local;
            } else {
                self.window.glyphs.push(
                    Glyph {
                        x: (self.column + self.cell_left) as u16,
                        row: self.row as u16,
                        width: width as u16,
                        start: start as u32,
                        end: end as u32,
                        kind: CellKind::RefText(meta),
                        styles: 0,
                        first: None,
                        links: 0,
                        relation_mask: mask,
                        projection_part: part,
                    },
                    w,
                )?;
                self.window
                    .glyphs
                    .cells
                    .last_mut()
                    .ok_or(Error::Invalid)?
                    .local = local as u32;
            }
            self.column += width;
        }
        Ok(true)
    }
    #[allow(clippy::too_many_arguments)]
    fn resource(
        &mut self,
        document: Arc<Document>,
        block: u32,
        run: u32,
        resource: u32,
        resume: Option<&PresentationAnchor>,
        budget: &ConsumerBudget,
        cancel: &AtomicBool,
        profile: IndexProfile,
        w: &mut DocumentWork<'_>,
    ) -> Result<bool> {
        let ref_resume = match resume {
            Some(PresentationAnchor::Ref(a)) if matches!(a.owner(),RefOwner::Resource{resource:r,..}if r==resource) => {
                Some(*a)
            }
            _ => None,
        };
        let alt_resume = match resume {
            Some(PresentationAnchor::ResourceAlt(a)) if a.resource() == resource => Some(*a),
            _ => None,
        };
        let d = roles::resource_descriptor(&document, resource, w)?;
        if d.block_intent() && self.column > 0 && !self.newline(w)? {
            return Ok(false);
        }
        let label = match d.kind() {
            ResourceKind::Image => "[Image pending: ",
            ResourceKind::Mermaid => "[Mermaid pending: ",
            ResourceKind::MathInline | ResourceKind::MathBlock => "[Math pending: ",
            ResourceKind::WikiEmbed => "[Wiki embed pending: ",
        };
        let input = d.input_ref();
        let alt = if d.kind() == ResourceKind::Image {
            d.alt_domain()
        } else {
            None
        };
        let title = d.title_ref();
        let block_intent = d.block_intent();
        let role_phase = self
            .role_resume
            .filter(|a| a.block() == block && a.run() == run)
            .map(|a| a.phase());
        let skip = ref_resume.map(|a| a.field());
        if skip.is_none()
            && alt_resume.is_none()
            && role_phase.is_none_or(|p| p <= 1)
            && !self.decoration(label, block, 1, w)?
        {
            return Ok(false);
        }
        if alt_resume.is_none()
            && role_phase.is_none_or(|p| p <= 1)
            && (skip.is_none() || skip == Some(RefField::Input))
        {
            let owner = RefOwner::Resource {
                block,
                run,
                resource,
                field: RefField::Input,
            };
            if !self.ref_text(
                document.clone(),
                owner,
                input,
                ref_resume.filter(|a| a.field() == RefField::Input),
                budget,
                cancel,
                profile,
                w,
            )? {
                return Ok(false);
            }
        }
        if let Some(domain) = alt
            && role_phase.is_none_or(|p| p <= 2)
            && (skip.is_none() || skip == Some(RefField::Input))
        {
            if alt_resume.is_none() && !self.decoration("; alt: ", block, 2, w)? {
                return Ok(false);
            }
            self.window.slot = None;
            let record = &document.domains()[domain as usize];
            w.admit_structure(size_of::<crate::document_model::Domain>())?;
            if record.run_count() > 0 {
                let slot = SequenceSlot::prepare(
                    document.clone(),
                    domain,
                    0..record.run_count() as usize,
                    budget,
                    cancel,
                    profile,
                    self.provider_gate.as_ref(),
                )?;
                let start = alt_resume.map(|a| a.position()).unwrap_or(
                    crate::document_view::LayoutAnchor {
                        raw: crate::document_view::RawAnchor {
                            block,
                            run: record.first_run(),
                            byte: 0,
                        },
                        projection_part: 0,
                    },
                );
                let meta = self.origin(
                    OriginMeta::Alt {
                        block,
                        run,
                        resource,
                        domain,
                        paint: PaintRole::Value,
                    },
                    w,
                )?;
                self.resource_alt = Some((block, run, resource, domain, meta));
                let completed = self.sequence(
                    slot,
                    start.raw.run,
                    start.raw.byte,
                    start.projection_part,
                    false,
                    block,
                    budget,
                    cancel,
                    w,
                )?;
                if !completed && let Some(a) = self.window.next {
                    self.window.presentation_next = Some(PresentationAnchor::ResourceAlt(
                        ResourceAltAnchor::checked(&document, block, run, resource, domain, a, w)?,
                    ));
                }
                self.resource_alt = None;
                if !completed {
                    return Ok(false);
                }
            }
        }
        if let Some(id) = title.filter(|_| role_phase.is_none_or(|p| p <= 3)) {
            if skip != Some(RefField::Title) && !self.decoration("; title: ", block, 3, w)? {
                return Ok(false);
            }
            if !self.ref_text(
                document.clone(),
                RefOwner::Resource {
                    block,
                    run,
                    resource,
                    field: RefField::Title,
                },
                id,
                ref_resume.filter(|a| a.field() == RefField::Title),
                budget,
                cancel,
                profile,
                w,
            )? {
                return Ok(false);
            }
        }
        if !self.decoration("]", block, 4, w)? {
            return Ok(false);
        }
        if block_intent {
            return self.newline(w);
        }
        Ok(true)
    }
    #[allow(clippy::too_many_arguments)]
    fn enter(
        &mut self,
        document: Arc<Document>,
        flow: &FlowHandle,
        cursor: &crate::document_flow::FlowCursor,
        block: u32,
        resume: Option<&PresentationAnchor>,
        budget: &ConsumerBudget,
        cancel: &AtomicBool,
        profile: IndexProfile,
        w: &mut DocumentWork<'_>,
    ) -> Result<bool> {
        let ref_resume = match resume {
            Some(PresentationAnchor::Ref(a))
                if a.block() == block && matches!(a.owner(), RefOwner::BlockRole { .. }) =>
            {
                Some(*a)
            }
            _ => None,
        };
        let role = roles::block_role(&document, block, w)?;
        match role {
            BlockRole::Callout { kind_text, .. } => {
                if self.column > 0 && !self.newline(w)? {
                    return Ok(false);
                }
                if ref_resume.is_none()
                    && self
                        .role_resume
                        .is_none_or(|r| r.block() != block || r.phase() <= 10)
                    && !self.decoration("[callout ", block, 10, w)?
                {
                    return Ok(false);
                }
                if self
                    .role_resume
                    .is_none_or(|r| r.block() != block || r.phase() <= 10)
                    && !self.ref_text(
                        document.clone(),
                        RefOwner::BlockRole {
                            block,
                            field: RefField::Kind,
                        },
                        kind_text,
                        ref_resume,
                        budget,
                        cancel,
                        profile,
                        w,
                    )?
                {
                    return Ok(false);
                }
                if !self.decoration(
                    if flow.index().expanded(block, w)? {
                        " -] "
                    } else {
                        " +] "
                    },
                    block,
                    11,
                    w,
                )? {
                    return Ok(false);
                }
            }
            BlockRole::Code {
                fenced: true,
                info: Some(info),
                mermaid,
                ..
            } => {
                let resource_resume = match resume {
                    Some(PresentationAnchor::Ref(a)) if matches!(a.owner(),RefOwner::Resource{block:b,..}if b==block) => {
                        Some(*a)
                    }
                    _ => None,
                };
                if resource_resume.is_none()
                    && !roles::text_ref_view(&document, info, w)?
                        .semantic_text()
                        .is_empty()
                {
                    if self.column > 0 && !self.newline(w)? {
                        return Ok(false);
                    }
                    if ref_resume.is_none()
                        && self
                            .role_resume
                            .is_none_or(|r| r.block() != block || r.phase() <= 20)
                        && !self.decoration("[code: ", block, 20, w)?
                    {
                        return Ok(false);
                    }
                    if self
                        .role_resume
                        .is_none_or(|r| r.block() != block || r.phase() <= 20)
                        && !self.ref_text(
                            document.clone(),
                            RefOwner::BlockRole {
                                block,
                                field: RefField::Info,
                            },
                            info,
                            ref_resume,
                            budget,
                            cancel,
                            profile,
                            w,
                        )?
                    {
                        return Ok(false);
                    }
                    if !self.decoration("]", block, 21, w)? || !self.newline(w)? {
                        return Ok(false);
                    }
                }
                if let Some(resource) = mermaid {
                    if !self.resource(document.clone(),block,NONE,resource,resume.filter(|a|matches!(a,PresentationAnchor::Ref(r)if matches!(r.owner(),RefOwner::Resource{..}))),budget,cancel,profile,w)?{return Ok(false);}
                    if self.column > 0 && !self.newline(w)? {
                        return Ok(false);
                    }
                }
            }
            BlockRole::Footnote { label } => {
                if self.column > 0 && !self.newline(w)? {
                    return Ok(false);
                }
                if ref_resume.is_none()
                    && self
                        .role_resume
                        .is_none_or(|r| r.block() != block || r.phase() <= 30)
                    && !self.decoration("[^", block, 30, w)?
                {
                    return Ok(false);
                }
                if self
                    .role_resume
                    .is_none_or(|r| r.block() != block || r.phase() <= 30)
                    && !self.ref_text(
                        document.clone(),
                        RefOwner::BlockRole {
                            block,
                            field: RefField::Label,
                        },
                        label,
                        ref_resume,
                        budget,
                        cancel,
                        profile,
                        w,
                    )?
                {
                    return Ok(false);
                }
                if !self.decoration("]: ", block, 31, w)? {
                    return Ok(false);
                }
            }
            BlockRole::Rule => {
                if self.column > 0 && !self.newline(w)? {
                    return Ok(false);
                }
                if !self.decoration("---", block, 40, w)? || !self.newline(w)? {
                    return Ok(false);
                }
            }
            BlockRole::Paragraph
            | BlockRole::Heading { .. }
            | BlockRole::HtmlInert
            | BlockRole::Frontmatter => {
                let parent = document.blocks()[block as usize].parent();
                let footnote = parent != NONE
                    && matches!(
                        roles::block_role(&document, parent, w)?,
                        BlockRole::Footnote { .. }
                    );
                if self.column > 0 && !footnote && !self.newline(w)? {
                    return Ok(false);
                }
            }
            BlockRole::Item { .. } => {
                let b = &document.blocks()[block as usize];
                w.admit_structure(size_of::<crate::document_model::Block>())?;
                if let BlockRole::List { tight: false, .. } =
                    roles::block_role(&document, b.parent(), w)?
                    && cursor.parent_item_ordinal(&document, block, w)? > 0
                    && !self.newline(w)?
                {
                    return Ok(false);
                }
            }
            _ => {}
        }
        Ok(true)
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
        let index = slot.domain_index()?;
        w.admit_structure(size_of::<crate::document_model::Block>())?;
        let code = matches!(
            index
                .text()
                .document()
                .blocks()
                .get(block as usize)
                .ok_or(Error::Invalid)?
                .kind(),
            10 | 14
        );
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
            let Some(resolved) = index.step_resolved(&mut point, 4, w)? else {
                break;
            };
            let unit = resolved.unit();
            if unit.oversized {
                return Err(Error::UnsupportedRun(u32::MAX));
            }
            if unit.newline {
                self.ensure_prefix(w)?;
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
            let source_column = point.column().saturating_sub(unit.width);
            let is_tab = unit.projected.end - unit.projected.start == 1 && {
                let mut tab = false;
                resolved.visit_projected(w, |s| {
                    tab = s == "\t";
                    Ok(())
                })?;
                tab
            };
            if is_tab {
                width = 8 - source_column % 8;
            }
            if !matches!(
                index.text().document().blocks()[block as usize].kind(),
                10 | 14
            ) {
                self.ensure_prefix(w)?;
            }
            let kind = self
                .window
                .glyphs
                .last()
                .filter(|g| usize::from(g.row) == self.row);
            if width == 0
                && kind.is_none_or(|g| {
                    !matches!(
                        g.kind,
                        CellKind::Glyph | CellKind::Text(_) | CellKind::AltText(_)
                    ) || g
                        .first
                        .is_none_or(|f| f.domain != slot.domain().unwrap_or(NONE))
                })
            {
                return Err(Error::UnsupportedRun(u32::MAX - 1));
            }
            if width > self.cell_width.saturating_sub(self.prefix_len) {
                return Err(Error::UnsupportedRun(u32::MAX - 2));
            }
            // Horizontal policy belongs to the actual Block, including ResourceAlt domains.
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
                self.column - self.geometry.left + self.cell_left
            } else {
                if self.column.checked_add(width).ok_or(Error::Invalid)? > self.cell_width
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
                self.ensure_prefix(w)?;
                self.column + self.cell_left
            };
            self.ensure_capacity(unit.projected.len().max(width), w)?;
            self.content_anchor(
                PresentationAnchor::Domain(crate::document_view::LayoutAnchor {
                    raw: crate::document_view::RawAnchor {
                        block,
                        run: before_raw.0,
                        byte: before_raw.1,
                    },
                    projection_part: before_raw.2,
                }),
                w,
            )?;
            let start = self.window.symbols.as_slice().len();
            let old_glyph = if width == 0 {
                self.window.glyphs.last()
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
                resolved.visit_projected(w, |s| {
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
            resolved.visit_coverage(w, |c| {
                if first.is_none() {
                    first = Some(FirstComponent {
                        block,
                        domain: slot.domain()?,
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
            if width == 0 {
                let mut last = self.window.glyphs.last().ok_or(Error::Invalid)?;
                last.end = u32::try_from(end).map_err(|_| Error::Invalid)?;
                last.styles |= styles;
                last.links = last.links.checked_add(links).ok_or(Error::Work)?;
                last.relation_mask |= relation_mask;
                self.window.glyphs.replace_last(last, w)?;
            } else {
                self.window.glyphs.push(
                    Glyph {
                        x: x as u16,
                        row: self.row as u16,
                        width: width as u16,
                        start: u32::try_from(start).map_err(|_| Error::Invalid)?,
                        end: u32::try_from(end).map_err(|_| Error::Invalid)?,
                        kind: if let Some((_, _, _, _, meta)) = self.resource_alt {
                            CellKind::AltText(meta)
                        } else if self.paint == PaintRole::Value {
                            CellKind::Glyph
                        } else {
                            CellKind::Text(self.paint)
                        },
                        styles,
                        first,
                        links,
                        relation_mask,
                        projection_part: before_raw.2,
                    },
                    w,
                )?;
            }
            self.column = self.column.checked_add(width).ok_or(Error::Invalid)?;
        }
        Ok(true)
    }
    fn finish(mut self, w: &mut DocumentWork<'_>) -> Result<TextWindow> {
        if self.row < usize::from(self.geometry.height) {
            self.newline(w)?;
        }
        self.refresh_bytes()?;
        if self.window.presentation_next.is_none() {
            self.window.presentation_next = self.window.next.map(PresentationAnchor::Domain);
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
    resume: Option<&PresentationAnchor>,
) -> Result<LayoutWindow> {
    let mut operation = crate::document_frame_work::LayoutWork::begin(cancel, builder.geometry)?;
    let w = operation.work();
    w.admit_structure(REFERENCE_WORK)?;
    let mut prep = DocumentWork::preparation(cancel);
    let mut cursor = flow.index().cursor_at(selected.location(), &mut prep)?;
    let mut pending = None;
    let mut rendered_row = None;
    builder.role_resume = match resume {
        Some(PresentationAnchor::Role(a)) => Some(*a),
        _ => None,
    };
    let result = (|| {
        if selected.run() != NONE && selected.raw_byte() == 0 && resume.is_none() {
            for (block, _, _, ordinal) in cursor.context(w)? {
                if matches!(
                    roles::block_role(&document, block, w)?,
                    BlockRole::Code { .. } | BlockRole::Footnote { .. }
                ) {
                    builder.event_ordinal = ordinal;
                    builder.event_run = NONE;
                    if !builder.enter(
                        document.clone(),
                        flow,
                        &cursor,
                        block,
                        None,
                        budget,
                        cancel,
                        profile.index,
                        w,
                    )? {
                        return builder.finish(w);
                    }
                }
            }
        }

        loop {
            let (event, location) = if let Some(p) = pending.take() {
                p
            } else {
                let ordinal = cursor.ordinal();
                let Some(event) = flow.index().next_event(&mut cursor, w)? else {
                    break;
                };
                (event, flow.index().location(ordinal)?)
            };
            builder.event_ordinal = location.ordinal();
            builder.event_run = match event {
                Event::Run { run, .. } => run,
                _ => NONE,
            };
            let table_row = match event {
                Event::Enter(block) if document.blocks()[block as usize].kind() == 8 => Some(block),
                Event::Run { block, .. }
                    if document.blocks()[block as usize].kind() == 9 && rendered_row.is_none() =>
                {
                    Some(document.blocks()[block as usize].parent())
                }
                _ => None,
            };
            if let Some(row) = table_row {
                let table_resume = match resume {
                    Some(PresentationAnchor::Table(t)) if t.row() == row => Some(t.as_ref()),
                    _ => None,
                };
                let raw = if selected.run() != NONE
                    && document.blocks()[selected.block() as usize].kind() == 9
                    && document.blocks()[selected.block() as usize].parent() == row
                {
                    Some(crate::document_view::LayoutAnchor {
                        raw: crate::document_view::RawAnchor {
                            block: selected.block(),
                            run: selected.run(),
                            byte: selected.raw_byte(),
                        },
                        projection_part: selected.projection_part(),
                    })
                } else {
                    None
                };
                rendered_row = Some(row);
                if !builder.table_row(
                    document.clone(),
                    flow,
                    &cursor,
                    row,
                    raw,
                    table_resume,
                    budget,
                    cancel,
                    profile.index,
                    w,
                )? {
                    break;
                }
                continue;
            }
            if let Some(row) = rendered_row {
                if matches!(event,Event::Leave(b)if b==row) {
                    rendered_row = None;
                }
                continue;
            }
            if let Event::Enter(block) = event
                && !builder.enter(
                    document.clone(),
                    flow,
                    &cursor,
                    block,
                    resume,
                    budget,
                    cancel,
                    profile.index,
                    w,
                )?
            {
                break;
            }
            if let Event::Run { block, run } = event {
                builder.context(
                    &document,
                    &cursor,
                    block,
                    run == selected.run() && selected.raw_byte() > 0,
                    w,
                )?;
                w.admit_structure(size_of::<crate::document_model::Run>())?;
                let record = &document.runs()[run as usize];
                if record.kind() == 5 {
                    if !builder.resource(
                        document.clone(),
                        block,
                        run,
                        record.resource(),
                        resume.filter(|a| a.block() == block),
                        budget,
                        cancel,
                        profile.index,
                        w,
                    )? {
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
                        builder.provider_gate.as_ref(),
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
                    w,
                )? {
                    break;
                }
            } else if matches!(event, Event::Leave(_))
                && builder.column > 0
                && !builder.newline(w)?
            {
                break;
            }
        }
        if builder.row >= usize::from(builder.geometry.height)
            && builder.window.presentation_next.is_none()
            && builder.window.next.is_none()
            && let Some(a) = following_anchor(flow, &mut cursor, pending.take(), rendered_row, w)?
        {
            builder.window.next = Some(a.layout_selection());
            builder.window.presentation_next = Some(a);
        }
        builder.window.preparation_work = builder
            .window
            .preparation_work
            .checked_add(prep.used())
            .ok_or(Error::Work)?;
        builder.finish(w)
    })();
    match result {
        Ok(mut t) => {
            t.receipt = Some(operation.finish());
            Ok(LayoutWindow::Text(Box::new(t)))
        }
        Err(Error::Cancelled) => Err(Error::Cancelled),
        Err(Error::Invalid) => Err(Error::Invalid),
        Err(Error::ProvidersBusy) => Err(Error::ProvidersBusy),
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
                receipt: operation.finish(),
            })
        }
    }
}

/// Background-only line positioning; no whole-document geometry index or UI traversal.
#[allow(clippy::too_many_arguments)] // No hidden owner, new allowance or implicit profile.
pub(crate) fn shift_line_scoped(
    document: Arc<Document>,
    flow: &FlowHandle,
    selected: &State,
    delta: i32,
    old_slot: Option<&Arc<SequenceSlot>>,
    budget: &ConsumerBudget,
    cancel: &AtomicBool,
    profile: ConsumerProfile,
    gate: Option<&Arc<ProviderGate>>,
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
            gate,
        )?
    };
    let _point = budget.reserve_points(2 * size_of::<Point>(), 2)?;
    let offset = slot.domain_index()?.text().locate_visual(
        run,
        byte,
        selected.projection_part(),
        &mut prep,
    )?;
    let mut p = slot
        .domain_index()?
        .seek_before_with_work(offset, &mut prep)?;
    while p.offset() < offset {
        if slot
            .domain_index()?
            .step(&mut p, 4096, &mut prep)?
            .is_none()
        {
            break;
        }
    }
    let target = p.line() as i64 + i64::from(delta);
    if target >= 0 && (target as usize) < slot.domain_index()?.line_count() {
        let p = slot
            .domain_index()?
            .seek_line_with_work(target as usize, &mut prep)?;
        if p.offset() < slot.domain_index()?.text().len() {
            let (run, byte, part) = slot
                .domain_index()?
                .visual_anchor_with_work(&p, &mut prep)?;
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

#[cfg(test)]
pub(crate) fn check_storage_roundtrip(budget: &ConsumerBudget) {
    let cancel = AtomicBool::new(false);
    let baseline = budget.used();
    let mut w = DocumentWork::frame(&cancel);
    let mut storage = GlyphStorage::new(budget, 2, 1).unwrap();
    let g = Glyph {
        x: 7,
        row: 3,
        width: 2,
        start: 11,
        end: 20,
        kind: CellKind::Glyph,
        styles: 6,
        first: Some(FirstComponent {
            block: 3,
            domain: 4,
            run: 9,
            local: 12345,
        }),
        links: 2,
        relation_mask: 5,
        projection_part: 3,
    };
    storage.push(g, &mut w).unwrap();
    let copy = storage.get(0).unwrap();
    assert_eq!(
        (copy.x, copy.row, copy.width, copy.start, copy.end),
        (7, 3, 2, 11, 20)
    );
    assert_eq!(
        (
            copy.styles,
            copy.links,
            copy.relation_mask,
            copy.projection_part
        ),
        (6, 2, 5, 3)
    );
    let f = copy.first.unwrap();
    assert_eq!((f.block, f.domain, f.run, f.local), (3, 4, 9, 12345));
    let before = w.used();
    let mut changed = g;
    changed.styles = 7;
    assert!(matches!(storage.push(changed, &mut w), Err(Error::Budget)));
    assert_eq!(storage.len(), 1);
    assert_eq!(storage.get(0).unwrap().styles, 6);
    assert!(w.used() > before);
    cancel.store(true, std::sync::atomic::Ordering::Release);
    assert!(matches!(storage.push(g, &mut w), Err(Error::Cancelled)));
    assert_eq!(storage.len(), 1);
    drop(storage);
    assert_eq!(budget.used(), baseline);
}

fn ascii_number(mut n: u64, b: &mut [u8; 20]) -> &str {
    let mut at = b.len();
    loop {
        at -= 1;
        b[at] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    std::str::from_utf8(&b[at..]).expect("ASCII")
}

fn following_anchor(
    flow: &FlowHandle,
    cursor: &mut crate::document_flow::FlowCursor,
    mut pending: Option<(Event, crate::document_flow::FlowLocation)>,
    mut skip_row: Option<u32>,
    w: &mut DocumentWork<'_>,
) -> Result<Option<PresentationAnchor>> {
    loop {
        let (event, ordinal) = if let Some((event, location)) = pending.take() {
            (event, location.ordinal())
        } else {
            let ordinal = cursor.ordinal();
            let Some(event) = flow.index().next_event(cursor, w)? else {
                return Ok(None);
            };
            (event, ordinal)
        };
        if let Some(row) = skip_row {
            if matches!(event,Event::Leave(b)if b==row) {
                skip_row = None;
            }
            continue;
        }
        match event {
            Event::Leave(_) => continue,
            Event::Enter(block) => {
                return Ok(Some(PresentationAnchor::Role(
                    OriginMeta::Role {
                        block,
                        run: NONE,
                        ordinal,
                        phase: 0,
                        paint: PaintRole::Secondary,
                    }
                    .role_anchor(0)?,
                )));
            }
            Event::Run { block, run } => {
                return Ok(Some(PresentationAnchor::Domain(
                    crate::document_view::LayoutAnchor {
                        raw: crate::document_view::RawAnchor {
                            block,
                            run,
                            byte: 0,
                        },
                        projection_part: 0,
                    },
                )));
            }
        }
    }
}

impl SequenceSlot {
    pub(crate) fn inspect_provider(&self) -> crate::document_search::inspect_hit::ProviderRef<'_> {
        match &self.payload {
            SlotPayload::Domain { index, .. } => {
                crate::document_search::inspect_hit::ProviderRef::Runs(index)
            }
            SlotPayload::Ref(index) => {
                crate::document_search::inspect_hit::ProviderRef::TextRef(index)
            }
        }
    }
    pub(crate) fn inspect_budget(&self) -> &ConsumerBudget {
        self._metadata.budget()
    }
}

#[allow(clippy::too_many_arguments)]
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
    shift_line_scoped(
        document, flow, selected, delta, old_slot, budget, cancel, profile, None,
    )
}
impl SequenceSlot {
    pub(crate) fn provider_gate(&self) -> Option<&Arc<ProviderGate>> {
        match &self.payload {
            SlotPayload::Domain { index, .. } => index.gate(),
            SlotPayload::Ref(index) => index.gate(),
        }
    }
}

impl TextWindow {
    pub(crate) fn take_provider_cache(&mut self) -> Option<Arc<SequenceSlot>> {
        self.slot.take()
    }
}
