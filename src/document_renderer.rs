//! Private first renderer: admitted text/spans, no roles/resources/pump.
#![allow(dead_code)]
use crate::{
    config::{PresentationConfig, Role},
    document_consumer_budget::Charge,
    document_layout::{CellKind, FRAME_BYTES, Glyph, TextWindow},
    document_text::{DOCUMENT_FRAME_WORK, DocumentWork},
    document_view::DocumentView,
    preparation_owner::Binding,
};
use ratatui::{
    buffer::Cell,
    layout::Rect,
    style::{Modifier, Style},
};
use std::{mem::size_of, sync::atomic::AtomicBool};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Error {
    Work {
        required: usize,
        limit: usize,
        parts: [usize; 7],
    },
    Frame,
    Symbol,
    Invalid,
    Resource,
    Consumer,
    Cancelled,
}
#[derive(Debug, Clone, Copy)]
pub(crate) struct WorkReceipt {
    pub layout: usize,
    pub preflight: usize,
    pub materialize: usize,
    pub readbacks: usize,
    pub diff: usize,
    pub reset: usize,
    pub formatter: usize,
    pub total: usize,
}
pub(crate) struct RenderPlan<'a> {
    window: &'a TextWindow,
    geometry: Rect,
    binding: Binding,
    config: &'a PresentationConfig,
    payload: usize,
    heap: usize,
    copied_bytes: usize,
    frame_bytes: usize,
    work: WorkReceipt,
    identity: usize,
    budget: crate::document_consumer_budget::ConsumerBudget,
    _metadata: Charge,
}
impl RenderPlan<'_> {
    pub(crate) fn window(&self) -> &TextWindow {
        self.window
    }
    pub(crate) fn geometry(&self) -> Rect {
        self.geometry
    }
    pub(crate) fn binding(&self) -> Binding {
        self.binding
    }
    pub(crate) fn config(&self) -> &PresentationConfig {
        self.config
    }
    pub(crate) fn payload(&self) -> usize {
        self.payload
    }
    pub(crate) fn heap(&self) -> usize {
        self.heap
    }
    pub(crate) fn copied_bytes(&self) -> usize {
        self.copied_bytes
    }
    pub(crate) fn frame_bytes(&self) -> usize {
        self.frame_bytes
    }
    pub(crate) fn work(&self) -> WorkReceipt {
        self.work
    }
    pub(crate) fn reserve_heap(&self) -> Result<Charge, Error> {
        self.budget.reserve(self.heap).map_err(|_| Error::Consumer)
    }
}
/// Linear authority: privately constructed, moved into one draw, never Clone.
pub(crate) struct MandatoryRenderPermit {
    binding: Binding,
    geometry: Rect,
    identity: usize,
    total: usize,
}
impl MandatoryRenderPermit {
    pub(crate) fn matches(&self, p: &RenderPlan<'_>) -> bool {
        self.binding == p.binding
            && self.geometry == p.geometry
            && self.identity == p.identity
            && self.total == p.work.total
    }
}
fn add(a: usize, b: usize) -> Result<usize, Error> {
    a.checked_add(b).ok_or(Error::Invalid)
}
fn mul(a: usize, b: usize) -> Result<usize, Error> {
    a.checked_mul(b).ok_or(Error::Invalid)
}
pub(crate) fn prepare<'a>(
    view: &'a DocumentView,
    config: &'a PresentationConfig,
    area: Rect,
    previous: &crate::document_terminal::PreviousFrame,
    cancel: &AtomicBool,
) -> Result<(RenderPlan<'a>, MandatoryRenderPermit), Error> {
    prepare_limit(view, config, area, previous, cancel, DOCUMENT_FRAME_WORK)
}
fn prepare_limit<'a>(
    view: &'a DocumentView,
    config: &'a PresentationConfig,
    area: Rect,
    previous: &crate::document_terminal::PreviousFrame,
    cancel: &AtomicBool,
    limit: usize,
) -> Result<(RenderPlan<'a>, MandatoryRenderPermit), Error> {
    let previous_window = previous.window();
    let previous_payload = previous.payload();
    let g = view.geometry();
    if g.x.checked_add(g.width).is_none()
        || g.y.checked_add(g.height).is_none()
        || area.width != g.width
        || area.height != g.height
        || area != Rect::new(g.x, g.y, g.width, g.height)
    {
        return Err(Error::Invalid);
    }
    let window = view.layout().text().ok_or(Error::Resource)?;
    let identity = window as *const TextWindow as usize;
    let mut w = DocumentWork::bounded(cancel, limit).map_err(|_| Error::Work {
        required: DOCUMENT_FRAME_WORK + 1,
        limit: DOCUMENT_FRAME_WORK,
        parts: [0; 7],
    })?;
    w.admit_structure(window.work).map_err(|e| match e {
        crate::document_text::Error::Cancelled => Error::Cancelled,
        _ => Error::Work {
            required: window.work,
            limit: DOCUMENT_FRAME_WORK,
            parts: [window.work, 0, 0, 0, 0, 0, 0],
        },
    })?;
    let mut payload = 0usize;
    let mut heap = 0usize;
    let mut copied_bytes = 0usize;
    let mut inspected = 0usize;
    for i in 0..window.glyphs().len() {
        // Read packed20 + class40, construct Glyph72 by value, check metadata/range.
        let cost = TextWindow::glyph_read_bytes() + 16;
        w.admit_structure(cost).map_err(|e| match e {
            crate::document_text::Error::Cancelled => Error::Cancelled,
            _ => Error::Work {
                required: DOCUMENT_FRAME_WORK + 1,
                limit: DOCUMENT_FRAME_WORK,
                parts: [0; 7],
            },
        })?;
        inspected = add(inspected, cost)?;
        let c = window.glyphs().get(i).ok_or(Error::Invalid)?;
        if c.kind != CellKind::Glyph {
            return Err(Error::Resource);
        }
        if c.width == 0 || c.row >= g.height || c.x.checked_add(c.width).is_none_or(|x| x > g.width)
        {
            return Err(Error::Invalid);
        }
        let s = window
            .symbols()
            .get(c.start as usize..c.end as usize)
            .ok_or(Error::Invalid)?;
        if s.is_empty() || s.len() > 16384 {
            return Err(Error::Symbol);
        }
        payload = add(payload, s.len())?;
        heap = add(heap, if s.len() > 24 { s.len().max(32) } else { 0 })?;
        copied_bytes = add(
            copied_bytes,
            if s.len() > 24 {
                s.len().max(32)
            } else {
                s.len()
            },
        )?;
    }
    let metadata = size_of::<RenderPlan<'_>>() + size_of::<MandatoryRenderPermit>();
    let frame_bytes = window
        .bytes()
        .checked_add(previous_window)
        .and_then(|n| n.checked_add(copied_bytes))
        .and_then(|n| n.checked_add(previous.copied_bytes()))
        .and_then(|n| n.checked_add(metadata))
        .ok_or(Error::Frame)?;
    if frame_bytes > FRAME_BYTES {
        return Err(Error::Frame);
    }
    let cells = mul(usize::from(area.width), usize::from(area.height))?;
    let readbacks = mul(mul(cells, 2 * size_of::<Cell>())?, 2)?;
    let diff = add(
        mul(cells, 2 * size_of::<Cell>())?,
        add(payload, previous_payload)?,
    )?;
    // Two resets cover success as well as error/cancel cleanup of both physical grids.
    let reset = mul(cells, 2 * size_of::<Cell>())?;
    // Upper bound: two updates per cell (preclear+base), ANSI descriptors each <=256 bytes.
    // Payload itself is charged separately, including old/new comparison and new writes.
    let formatter = add(mul(cells, 2 * 256)?, payload)?;
    let materialize = add(inspected, payload)?;
    let formatter = add(formatter, 256)?; // fixed FG/BG/attribute reset, cursor-hide and flush descriptors
    let parts = [
        window.work,
        inspected,
        materialize,
        readbacks,
        diff,
        reset,
        formatter,
    ];
    let total = parts.iter().try_fold(0usize, |n, &c| add(n, c))?;
    if total > limit {
        return Err(Error::Work {
            required: total,
            limit,
            parts,
        });
    }
    for cost in [materialize, readbacks, diff, reset, formatter] {
        w.admit_structure(cost).map_err(|e| match e {
            crate::document_text::Error::Cancelled => Error::Cancelled,
            _ => Error::Work {
                required: DOCUMENT_FRAME_WORK + 1,
                limit: DOCUMENT_FRAME_WORK,
                parts: [0; 7],
            },
        })?;
    }
    let work = WorkReceipt {
        layout: window.work,
        preflight: inspected,
        materialize,
        readbacks,
        diff,
        reset,
        formatter,
        total: w.used(),
    };
    let metadata_charge = view
        .budget()
        .reserve(metadata)
        .map_err(|_| Error::Consumer)?;
    let binding = view.render_binding();
    let p = RenderPlan {
        window,
        geometry: area,
        binding,
        config,
        payload,
        heap,
        copied_bytes,
        frame_bytes,
        work,
        identity,
        budget: view.budget().clone(),
        _metadata: metadata_charge,
    };
    let permit = MandatoryRenderPermit {
        binding,
        geometry: area,
        identity,
        total: work.total,
    };
    Ok((p, permit))
}
pub(crate) fn style(g: Glyph, config: &PresentationConfig) -> Style {
    if !config.coloured {
        return Style::default();
    }
    let linked = g.links > 0;
    let mut s = config.style(if g.styles & 8 != 0 || linked {
        Role::Accent
    } else {
        Role::Value
    });
    if g.styles & 1 != 0 {
        s = s.add_modifier(Modifier::ITALIC)
    }
    if g.styles & 2 != 0 {
        s = s.add_modifier(Modifier::BOLD)
    }
    if g.styles & 4 != 0 {
        s = s.add_modifier(Modifier::CROSSED_OUT)
    }
    if linked {
        s = s.add_modifier(Modifier::UNDERLINED)
    }
    s
}
#[cfg(test)]
#[path = "document_renderer_tests.rs"]
mod tests;
