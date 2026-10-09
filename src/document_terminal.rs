//! Closed fixed-viewport owner. Grid capacity is admitted before any allocation.
#![allow(dead_code)] // Renderer and interactive pump consume this owner in the next increment.
use ratatui::{Terminal, backend::Backend, buffer::Cell, layout::Rect};
use std::{
    mem::size_of,
    sync::atomic::{AtomicBool, Ordering},
};

pub(crate) const FRAMEWORK_LIMIT: usize = 2 * 1024 * 1024;
const WRITER_METADATA_RESERVE: usize = 8192;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Receipt {
    pub capacities: [usize; 2],
    pub symbol_heap: [usize; 2],
    pub live: usize,
    pub peak: usize,
    pub area: Rect,
}
#[derive(Debug)]
pub(crate) enum Error<E> {
    Budget,
    Allocation,
    Cancelled,
    Stale,
    Poisoned,
    Backend(E),
}

/// Never exposes mutable Terminal or backend to a renderer.
pub(crate) struct BudgetedTerminal<B: Backend> {
    terminal: Terminal<B>,
    limit: usize,
    metadata: usize,
    peak: usize,
    area: Rect,
    poisoned: bool,
    previous_window: usize,
    previous_payload: usize,
    previous_copied_bytes: usize,
    symbol_charge: Option<crate::document_consumer_budget::Charge>,
    #[cfg(test)]
    fail_reserve: Option<usize>,
}
impl<B: Backend> BudgetedTerminal<B> {
    pub(crate) fn new(backend: B) -> Result<Self, Error<B::Error>> {
        Self::with_limit(backend, FRAMEWORK_LIMIT)
    }
    fn with_limit(backend: B, limit: usize) -> Result<Self, Error<B::Error>> {
        let metadata = size_of::<Self>()
            .checked_add(WRITER_METADATA_RESERVE)
            .ok_or(Error::Budget)?;
        if metadata > limit {
            return Err(Error::Budget);
        }
        let terminal = Terminal::r3_empty_fixed(backend).map_err(Error::Backend)?;
        let mut owner = Self {
            terminal,
            limit,
            metadata,
            peak: metadata,
            area: Rect::default(),
            poisoned: false,
            previous_window: 0,
            previous_payload: 0,
            previous_copied_bytes: 0,
            symbol_charge: None,
            #[cfg(test)]
            fail_reserve: None,
        };
        owner.readback()?;
        Ok(owner)
    }
    pub(crate) fn receipt(&self) -> Result<Receipt, Error<B::Error>> {
        let actual = self.terminal.r3_buffer_storage().ok_or(Error::Budget)?;
        let capacities = actual.map(|s| s.cell_capacity);
        let symbol_heap = actual.map(|s| s.symbol_heap_bytes);
        let mut live = self.metadata;
        for i in 0..2 {
            live = live
                .checked_add(
                    capacities[i]
                        .checked_mul(size_of::<Cell>())
                        .ok_or(Error::Budget)?,
                )
                .and_then(|n| n.checked_add(symbol_heap[i]))
                .ok_or(Error::Budget)?;
        }
        Ok(Receipt {
            capacities,
            symbol_heap,
            live,
            peak: self.peak,
            area: self.area,
        })
    }
    fn readback(&mut self) -> Result<Receipt, Error<B::Error>> {
        let receipt = match self.receipt() {
            Ok(receipt) => receipt,
            Err(error) => {
                self.poisoned = true;
                return Err(error);
            }
        };
        self.peak = self.peak.max(receipt.live);
        if receipt.live > self.limit {
            self.poisoned = true;
            return Err(Error::Budget);
        }
        Ok(Receipt {
            peak: self.peak,
            ..receipt
        })
    }
    fn check(
        &self,
        cancel: &AtomicBool,
        current: &mut impl FnMut() -> bool,
    ) -> Result<(), Error<B::Error>> {
        if self.poisoned {
            return Err(Error::Poisoned);
        }
        if cancel.load(Ordering::Acquire) {
            return Err(Error::Cancelled);
        }
        let is_current = current();
        if cancel.load(Ordering::Acquire) {
            return Err(Error::Cancelled);
        }
        if !is_current {
            return Err(Error::Stale);
        }
        Ok(())
    }
    /// A rejected reserve preserves old visible geometry and any successful physical reservation.
    /// Only the enclosing frame owner supplies the binding check; no new deadline or worker.
    pub(crate) fn prepare_area(
        &mut self,
        area: Rect,
        cancel: &AtomicBool,
        mut current: impl FnMut() -> bool,
    ) -> Result<Receipt, Error<B::Error>> {
        self.check(cancel, &mut current)?;
        let cells = usize::from(area.width)
            .checked_mul(usize::from(area.height))
            .ok_or(Error::Budget)?;
        for i in 0..2 {
            let before = self.readback()?;
            if cells <= before.capacities[i] {
                continue;
            }
            // Current allocation stays physically alive while the new allocation is obtained.
            let peak = before
                .live
                .checked_add(cells.checked_mul(size_of::<Cell>()).ok_or(Error::Budget)?)
                .ok_or(Error::Budget)?;
            if peak > self.limit {
                return Err(Error::Budget);
            }
            self.check(cancel, &mut current)?;
            self.peak = self.peak.max(peak);
            #[cfg(test)]
            if self.fail_reserve == Some(i) {
                return Err(Error::Allocation);
            }
            let result = self.terminal.r3_reserve_buffer(i == 1, cells);
            // Read even if allocation failed, and retain the first successful reserve on failure.
            self.readback()?;
            result.map_err(|_| Error::Allocation)?;
        }
        self.check(cancel, &mut current)?;
        // A stable viewport must preserve the previous visible mirror and emit no clear.
        // Capacity has still been admitted/read back above; no allowance is renewed here.
        if area == self.area {
            return self.readback();
        }
        match self.terminal.r3_resize_reserved_fixed(area) {
            Ok(true) => {
                self.area = area;
                self.readback()
            }
            Ok(false) => {
                self.poisoned = true;
                Err(Error::Poisoned)
            }
            Err(error) => {
                self.poisoned = true;
                Err(Error::Backend(error))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    fn cancel() -> AtomicBool {
        AtomicBool::new(false)
    }
    #[test]
    fn actual_normal_geometry_and_exact_resize_peak() {
        let mut owner = BudgetedTerminal::new(TestBackend::new(121, 40)).unwrap();
        let first = owner
            .prepare_area(Rect::new(0, 0, 120, 40), &cancel(), || true)
            .unwrap();
        assert_eq!(first.capacities, [4800, 4800]);
        assert_eq!(first.live, owner.metadata + 460800);
        let grown = owner
            .prepare_area(Rect::new(0, 0, 121, 40), &cancel(), || true)
            .unwrap();
        assert_eq!(grown.capacities, [4840, 4840]);
        assert_eq!(grown.peak, owner.metadata + 695040);
        let small = owner
            .prepare_area(Rect::new(0, 0, 80, 24), &cancel(), || true)
            .unwrap();
        assert_eq!(small.capacities, grown.capacities);
        assert_eq!(small.live, grown.live);
    }
    #[test]
    fn second_reserve_budget_failure_retains_first_capacity_and_old_area() {
        let metadata = size_of::<BudgetedTerminal<TestBackend>>() + WRITER_METADATA_RESERVE;
        let mut owner =
            BudgetedTerminal::with_limit(TestBackend::new(120, 40), metadata + 460799).unwrap();
        assert!(matches!(
            owner.prepare_area(Rect::new(0, 0, 120, 40), &cancel(), || true),
            Err(Error::Budget)
        ));
        let kept = owner.receipt().unwrap();
        assert_eq!(kept.capacities, [4800, 0]);
        assert_eq!(kept.live, metadata + 230400);
        assert_eq!(kept.area, Rect::default());
    }
    #[test]
    fn synthetic_second_allocation_failure_preserves_first_physical_reserve() {
        let mut owner = BudgetedTerminal::new(TestBackend::new(120, 40)).unwrap();
        owner.fail_reserve = Some(1);
        assert!(matches!(
            owner.prepare_area(Rect::new(0, 0, 120, 40), &cancel(), || true),
            Err(Error::Allocation)
        ));
        assert_eq!(owner.receipt().unwrap().capacities, [4800, 0]);
        assert_eq!(owner.area, Rect::default());
        owner.fail_reserve = None;
        assert!(
            owner
                .prepare_area(Rect::new(0, 0, 120, 40), &cancel(), || true)
                .is_ok()
        );
    }
    #[test]
    fn cancelled_and_stale_never_publish_new_geometry() {
        let mut owner = BudgetedTerminal::new(TestBackend::new(120, 40)).unwrap();
        let cancel = cancel();
        let mut calls = 0;
        assert!(matches!(
            owner.prepare_area(Rect::new(0, 0, 120, 40), &cancel, || {
                calls += 1;
                if calls == 4 {
                    cancel.store(true, Ordering::Release);
                }
                true
            }),
            Err(Error::Cancelled)
        ));
        assert_eq!(owner.receipt().unwrap().capacities, [4800, 4800]);
        assert_eq!(owner.area, Rect::default());
        cancel.store(false, Ordering::Release);
        assert!(matches!(
            owner.prepare_area(Rect::new(0, 0, 120, 40), &cancel, || false),
            Err(Error::Stale)
        ));
        assert_eq!(owner.area, Rect::default());
    }
    #[test]
    fn zero_geometry_and_backend_growth_leave_fixed_grids() {
        let mut owner = BudgetedTerminal::new(TestBackend::new(121, 40)).unwrap();
        owner
            .prepare_area(Rect::new(0, 0, 80, 24), &cancel(), || true)
            .unwrap();
        let before = owner.receipt().unwrap();
        owner
            .terminal
            .draw(|f| assert_eq!(f.area(), Rect::new(0, 0, 80, 24)))
            .unwrap();
        assert_eq!(owner.receipt().unwrap(), before);
        let empty = owner
            .prepare_area(Rect::default(), &cancel(), || true)
            .unwrap();
        assert_eq!(empty.capacities, before.capacities);
        assert_eq!(empty.area, Rect::default());
    }
}

impl<B: Backend> BudgetedTerminal<B> {
    /// Closed entry; no mutable framework value escapes to the caller.
    pub(crate) fn draw_document(
        &mut self,
        view: &crate::document_view::DocumentView,
        config: &crate::config::PresentationConfig,
        cancel: &AtomicBool,
        mut current: impl FnMut(crate::preparation_owner::Binding) -> bool,
    ) -> Result<crate::document_renderer::WorkReceipt, RenderError<B::Error>> {
        use crate::{document_consumer_budget::Charge, document_renderer as renderer};
        use ratatui::buffer::CellDiffOption;
        use std::num::NonZeroU16;
        self.check(cancel, &mut || current(view.render_binding()))
            .map_err(RenderError::Owner)?;
        let previous = self.previous_frame();
        let (plan, permit) = renderer::prepare(view, config, self.area, &previous, cancel)
            .map_err(RenderError::Admission)?;
        if !permit.matches(&plan) {
            return Err(RenderError::Admission(renderer::Error::Invalid));
        }
        if self.area.width == 0 || self.area.height == 0 {
            return Ok(plan.work());
        } // Geometry Reference performs no backend IO.
        // The frame permit has prepaid both physical storage scans.
        let before = self.readback().map_err(RenderError::Owner)?;
        let new_heap: Charge = plan.reserve_heap().map_err(RenderError::Admission)?;
        let peak = before
            .live
            .checked_add(plan.heap())
            .ok_or(RenderError::Owner(Error::Budget))?;
        if peak > self.limit {
            return Err(RenderError::Owner(Error::Budget));
        }
        self.peak = self.peak.max(peak);
        self.check(cancel, &mut || current(plan.binding()))
            .map_err(RenderError::Owner)?;
        // Materialization cannot perform traversal, IO or admission.
        {
            let mut frame = self.terminal.get_frame();
            let grid = frame.buffer_mut();
            for (glyph, symbol) in plan.cells() {
                let x = plan.geometry().x + glyph.x;
                let y = plan.geometry().y + glyph.row;
                let style = renderer::style(glyph, plan.config());
                grid[(x, y)]
                    .set_symbol(symbol)
                    .set_style(style)
                    .set_diff_option(CellDiffOption::ForcedWidth(
                        NonZeroU16::new(glyph.width).expect("preadmitted width"),
                    ));
                for dx in 1..glyph.width {
                    grid[(x + dx, y)]
                        .set_symbol(" ")
                        .set_style(style)
                        .set_diff_option(CellDiffOption::Skip);
                }
            }
        }
        let after = match self.readback() {
            Ok(r) => r,
            Err(e) => {
                self.terminal.r3_reset_symbols();
                self.symbol_charge = None;
                return Err(RenderError::Owner(e));
            }
        };
        let measured = after.symbol_heap[0]
            .checked_add(after.symbol_heap[1])
            .ok_or(RenderError::Owner(Error::Budget))?;
        if measured > before.symbol_heap[0] + before.symbol_heap[1] + plan.heap() {
            self.terminal.r3_reset_symbols();
            self.symbol_charge = None;
            self.poisoned = true;
            return Err(RenderError::Owner(Error::Poisoned));
        }
        if let Err(e) = self.check(cancel, &mut || current(plan.binding())) {
            // Preserve previous's physical symbols and Consumer authority: they still
            // describe the visible terminal. Only the unsubmitted candidate is discarded.
            self.terminal.r3_reset_current_symbols();
            return Err(RenderError::Owner(e));
        }
        if let Err(e) = self.terminal.r3_apply_buffer_admitted() {
            self.terminal.r3_reset_symbols();
            self.symbol_charge = None;
            self.poisoned = true;
            return Err(RenderError::Owner(Error::Backend(e)));
        }
        // swap reset physically destroyed previous symbol allocations.
        self.symbol_charge = Some(new_heap);
        self.previous_window = plan.window_bytes();
        self.previous_payload = plan.payload();
        self.previous_copied_bytes = plan.copied_bytes();
        Ok(plan.work())
    }
}
#[derive(Debug)]
pub(crate) enum RenderError<E> {
    Owner(Error<E>),
    Admission(crate::document_renderer::Error),
}

/// Only the owner can construct previous-frame values; renderer receives immutable readback.
pub(crate) struct PreviousFrame {
    window: usize,
    payload: usize,
    copied_bytes: usize,
}
impl PreviousFrame {
    pub(crate) fn window(&self) -> usize {
        self.window
    }
    pub(crate) fn payload(&self) -> usize {
        self.payload
    }
    pub(crate) fn copied_bytes(&self) -> usize {
        self.copied_bytes
    }
}

impl<B: Backend> BudgetedTerminal<B> {
    pub(crate) fn previous_frame(&self) -> PreviousFrame {
        PreviousFrame {
            window: self.previous_window,
            payload: self.previous_payload,
            copied_bytes: self.previous_copied_bytes,
        }
    }
}
