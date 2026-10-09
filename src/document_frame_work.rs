//! Closed two-phase admission for one layout+draw operation. Holds capacity, never refunds work.
use crate::{
    document_layout::{Geometry, TextWindow, WINDOW_BYTES},
    document_text::{DOCUMENT_FRAME_WORK, DocumentWork, Error, Result},
};
use ratatui::buffer::Cell;
use std::{mem::size_of, sync::atomic::AtomicBool};
#[derive(Debug)]
pub(crate) struct RenderReserve {
    capacity: usize,
}
#[derive(Debug)]
pub(crate) struct FrameWorkReceipt {
    layout_spent: usize,
    reserve: RenderReserve,
    geometry: Geometry,
}
impl FrameWorkReceipt {
    pub(crate) fn geometry(&self) -> Geometry {
        self.geometry
    }
    pub(crate) fn layout_spent(&self) -> usize {
        self.layout_spent
    }
    #[cfg(test)]
    pub(crate) fn render_capacity(&self) -> usize {
        self.reserve.capacity
    }
    pub(crate) fn validate_spend(&self, render_spent: usize) -> Result<usize> {
        if render_spent > self.reserve.capacity {
            return Err(Error::Work);
        }
        self.layout_spent
            .checked_add(render_spent)
            .filter(|n| *n <= DOCUMENT_FRAME_WORK)
            .ok_or(Error::Work)
    }
    /// Explicit stable redraw replays the same mandatory layout charge within a new frame operation.
    pub(crate) fn replay(&self) -> Self {
        Self {
            layout_spent: self.layout_spent,
            reserve: RenderReserve {
                capacity: self.reserve.capacity,
            },
            geometry: self.geometry,
        }
    }
}
pub(crate) struct LayoutWork<'a> {
    work: DocumentWork<'a>,
    reserve: RenderReserve,
    geometry: Geometry,
}
impl<'a> LayoutWork<'a> {
    pub(crate) fn begin(cancel: &'a AtomicBool, g: Geometry) -> Result<Self> {
        let n = usize::from(g.width)
            .checked_mul(usize::from(g.height))
            .ok_or(Error::Work)?;
        let read = TextWindow::glyph_read_bytes()
            .checked_add(16 + 32)
            .ok_or(Error::Work)?;
        let per = 8 * size_of::<Cell>() + 512 + 2 * read;
        let capacity = n
            .checked_mul(per)
            .and_then(|c| c.checked_add(4 * WINDOW_BYTES + 256))
            .ok_or(Error::Work)?;
        let limit = DOCUMENT_FRAME_WORK
            .checked_sub(capacity)
            .ok_or(Error::Work)?;
        Ok(Self {
            work: DocumentWork::bounded(cancel, limit)?,
            reserve: RenderReserve { capacity },
            geometry: g,
        })
    }
    /// Rejected geometry emits only the bounded label; renderer still independently admits real grids.
    /// A rectangle whose grid cannot fit never gains a draw permit or backend I/O.
    pub(crate) fn reference(cancel: &'a AtomicBool, g: Geometry) -> Result<Self> {
        let n = usize::from(g.width)
            .checked_mul(usize::from(g.height))
            .ok_or(Error::Work)?;
        let read = TextWindow::glyph_read_bytes() + 16 + 32;
        let capacity = n
            .checked_mul(8 * size_of::<Cell>() + 512 + 2 * read)
            .and_then(|v| v.checked_add(4 * WINDOW_BYTES + 256))
            .unwrap_or(DOCUMENT_FRAME_WORK)
            .min(DOCUMENT_FRAME_WORK - crate::document_layout::REFERENCE_WORK);
        Ok(Self {
            work: DocumentWork::bounded(cancel, DOCUMENT_FRAME_WORK - capacity)?,
            reserve: RenderReserve { capacity },
            geometry: g,
        })
    }
    pub(crate) fn work(&mut self) -> &mut DocumentWork<'a> {
        &mut self.work
    }
    pub(crate) fn finish(self) -> FrameWorkReceipt {
        FrameWorkReceipt {
            layout_spent: self.work.used(),
            reserve: self.reserve,
            geometry: self.geometry,
        }
    }
    pub(crate) fn spent(&self) -> usize {
        self.work.used()
    }
}
