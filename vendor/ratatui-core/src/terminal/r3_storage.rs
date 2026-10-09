//! Narrow storage hooks for R3Viewer's enclosing, budgeted fixed-viewport owner.
use std::collections::TryReserveError;

use super::{Terminal, TerminalOptions, Viewport};
use crate::{backend::Backend, layout::Rect};

/// Actual physical storage of one terminal buffer; does not copy cells or symbols.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct R3BufferStorage {
    /// Allocated vector capacity in cells, including unused slots.
    pub cell_capacity: usize,
    /// Sum of actual heap capacities of symbols, excluding inline and static symbols.
    pub symbol_heap_bytes: usize,
}

impl<B: Backend> Terminal<B> {
    /// Creates a fixed, zero-cell terminal. The enclosing owner admits storage before growth.
    /// This method does not query backend geometry or allocate nonempty cell grids.
    pub fn r3_empty_fixed(backend: B) -> Result<Self, B::Error> {
        Self::with_options(
            backend,
            TerminalOptions {
                viewport: Viewport::Fixed(Rect::default()),
            },
        )
    }

    /// Reads both real buffer capacities without cloning or allocating.
    /// Returns `None` on an arithmetic overflow.
    pub fn r3_buffer_storage(&self) -> Option<[R3BufferStorage; 2]> {
        let mut result = [R3BufferStorage {
            cell_capacity: 0,
            symbol_heap_bytes: 0,
        }; 2];
        for (buffer, slot) in self.buffers.iter().zip(result.iter_mut()) {
            slot.cell_capacity = buffer.content.capacity();
            slot.symbol_heap_bytes = buffer.content.iter().try_fold(0usize, |sum, cell| {
                sum.checked_add(cell.r3_symbol_heap_capacity())
            })?;
        }
        Some(result)
    }

    /// Reserves one buffer fallibly, without changing its length, viewport, or visible cells.
    /// `second` chooses physical buffer one rather than zero. The enclosing owner must admit
    /// transient old/new overlap before calling and read back capacity even after a later failure.
    pub fn r3_reserve_buffer(&mut self, second: bool, cells: usize) -> Result<(), TryReserveError> {
        let buffer = &mut self.buffers[usize::from(second)];
        buffer
            .content
            .try_reserve_exact(cells.saturating_sub(buffer.content.len()))
    }

    /// Resizes an already reserved fixed terminal without allocating cell grids.
    /// Returns `Ok(false)` without mutation if either buffer lacks capacity or viewport is not
    /// fixed. Backend errors remain fatal to the enclosing session, as in normal `resize`.
    pub fn r3_resize_reserved_fixed(&mut self, area: Rect) -> Result<bool, B::Error> {
        if !matches!(self.viewport, Viewport::Fixed(_))
            || self.buffers.iter().any(|buffer| {
                buffer.content.capacity() < usize::from(area.width) * usize::from(area.height)
            })
        {
            return Ok(false);
        }
        self.resize(area)?;
        self.viewport = Viewport::Fixed(area);
        Ok(true)
    }
}

/// Applies a closed admitted frame with wide preclears, retaining the normal
/// cursor/reset/swap/flush/count lifecycle. General apply/diff remain unchanged.
impl<B: Backend> Terminal<B> {
    /// Applies the admitted current buffer with borrowed preclears before changed wide cells.
    pub fn r3_apply_buffer_admitted(&mut self) -> Result<super::CompletedFrame<'_>, B::Error> {
        use crate::{buffer::CellDiffOption, layout::Position};
        let previous = &self.buffers[1 - self.current];
        let current = &self.buffers[self.current];
        let mut last_pos = None;
        // Current-grid continuation cells are real blanks with the base style.
        // A changed wide update clears them BEFORE printing its base symbol.
        let expanded = previous
            .diff_iter(current)
            .flat_map(|(x, y, cell)| {
                let width = match cell.diff_option {
                    CellDiffOption::ForcedWidth(w) => w.get(),
                    _ => 1,
                };
                (1..width)
                    .map(move |dx| (x + dx, y, &current[(x + dx, y)]))
                    .chain(std::iter::once((x, y, cell)))
            })
            .inspect(|(x, y, _)| {
                last_pos = Some(Position { x: *x, y: *y });
            });
        self.backend.draw(expanded)?;
        if let Some(pos) = last_pos {
            self.last_known_cursor_pos = pos
        }
        self.hide_cursor()?;
        self.swap_buffers();
        self.backend.flush()?;
        let completed = super::CompletedFrame {
            buffer: &self.buffers[1 - self.current],
            area: self.last_known_area,
            count: self.frame_count,
        };
        self.frame_count = self.frame_count.wrapping_add(1);
        Ok(completed)
    }
    /// Discards only the unsubmitted current frame. Retains the previous visible mirror.
    pub fn r3_reset_current_symbols(&mut self) {
        self.buffers[self.current].reset();
    }
    /// Internal cleanup of symbol allocations after rejected/failed draw. No IO.
    pub fn r3_reset_symbols(&mut self) {
        self.buffers[0].reset();
        self.buffers[1].reset();
    }
}
