//! Source viewport uses the retained model on the SAME Source preparation authority.
#![allow(dead_code)]
use crate::{
    document_budget::{ArenaVec, Lease},
    document_layout::Geometry,
    preparation_owner::{Binding, Cause, Outcome, OwnedPreparation, PreparationAuthority},
    presentation::PrepareError,
    source_preparation::{SourcePreparationTaskInput, SourcePrepared},
    source_shared::{SharedSource, SourceLease},
    viewport_admission::{FrameBudget, SYMBOL_BYTES},
};
use std::{
    mem::size_of,
    ops::Range,
    sync::{Arc, atomic::AtomicBool},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ViewportKey {
    binding: Binding,
    query_revision: u64,
    action_revision: u64,
    frame_revision: u64,
}
impl ViewportKey {
    pub(crate) fn binding(&self) -> Binding {
        self.binding
    }
}
pub(crate) struct SourceViewportInput {
    lease: SourceLease,
    geometry: Geometry,
    top: usize,
    original_selection: Option<Range<usize>>,
    key: ViewportKey,
    _metadata: Lease,
    supplement: Option<Lease>,
}
impl SourceViewportInput {
    // Independent binding/query/action/frame revisions remain explicit at this closed boundary.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        shared: &SharedSource,
        geometry: Geometry,
        top: usize,
        original_selection: Option<Range<usize>>,
        revision: u64,
        query_revision: u64,
        action_revision: u64,
        frame_revision: u64,
    ) -> Result<Self, Cause> {
        if let Some(r) = &original_selection
            && (r.start > r.end
                || r.end > shared.model().original().len()
                || !shared.model().original().is_char_boundary(r.start)
                || !shared.model().original().is_char_boundary(r.end))
        {
            return Err(Cause::Stale);
        }
        if top >= shared.model().line_count() && top != 0 {
            return Err(Cause::Stale);
        }
        let lease = shared.try_lease()?;
        let run: fn(&SourcePreparationTaskInput, &AtomicBool) -> Outcome<SourcePrepared> =
            run_viewport;
        let bytes =
            OwnedPreparation::<SourcePreparationTaskInput, SourcePrepared>::known_spawn_bytes(&run)
                .and_then(|n| n.checked_sub(size_of::<SourceLease>()))
                .ok_or(Cause::Limit("Source viewport arithmetic"))?;
        let metadata = lease
            .arena()
            .reserve(bytes)
            .map_err(|_| Cause::Limit("Source viewport metadata"))?;
        let key = ViewportKey {
            binding: lease.binding(revision),
            query_revision,
            action_revision,
            frame_revision,
        };
        Ok(Self {
            lease,
            geometry,
            top,
            original_selection,
            key,
            _metadata: metadata,
            supplement: None,
        })
    }
    pub(crate) fn key(&self) -> ViewportKey {
        self.key
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct SourceGlyph {
    start: u32,
    visible_start: u32,
    source_start: u32,
    pub row: u16,
    pub column: u16,
    pub width: u16,
    length: u16,
    visible_length: u16,
    source_length: u16,
}
impl SourceGlyph {
    pub(crate) fn symbol_range(&self) -> Range<usize> {
        self.start as usize..self.start as usize + usize::from(self.length)
    }
    pub(crate) fn visible_range(&self) -> Range<usize> {
        self.visible_start as usize..self.visible_start as usize + usize::from(self.visible_length)
    }
    pub(crate) fn source_range(&self) -> Range<usize> {
        self.source_start as usize..self.source_start as usize + usize::from(self.source_length)
    }
}
pub(crate) struct SourceViewportPrepared {
    lease: SourceLease,
    key: ViewportKey,
    geometry: Geometry,
    top: usize,
    glyphs: ArenaVec<SourceGlyph>,
    symbols: ArenaVec<u8>,
    selection: Option<(Range<usize>, Range<usize>, usize)>,
    work: usize,
    frame_bytes: usize,
    _metadata: Lease,
}
impl SourceViewportPrepared {
    pub(crate) fn key(&self) -> ViewportKey {
        self.key
    }
    pub(crate) fn model(&self) -> &crate::presentation::SafeTextModel {
        self.lease.model()
    }
    pub(crate) fn context(&self) -> &Arc<crate::source_preparation::PreparedSourceContext> {
        self.lease.context()
    }
    pub(crate) fn glyphs(&self) -> &[SourceGlyph] {
        &self.glyphs
    }
    pub(crate) fn symbols(&self) -> &str {
        std::str::from_utf8(&self.symbols).expect("admitted safe UTF8")
    }
    pub(crate) fn selection(&self) -> Option<&(Range<usize>, Range<usize>, usize)> {
        self.selection.as_ref()
    }
    pub(crate) fn work(&self) -> usize {
        self.work
    }
    pub(crate) fn frame_bytes(&self) -> usize {
        self.frame_bytes
    }
}
fn error(e: PrepareError) -> Cause {
    match e {
        PrepareError::Cancelled => Cause::Cancelled,
        PrepareError::Limit(s) => Cause::Limit(s),
    }
}
fn checked_u32(n: usize) -> Result<u32, Cause> {
    u32::try_from(n).map_err(|_| Cause::Limit("Source position"))
}
// Approved calibration applies only to this Common phase, never to Work::frame/Legacy.
pub(crate) const SOURCE_COMMON_WORK: usize = 512 * 1024;
fn common_frame_budget(cancel: &AtomicBool) -> FrameBudget<'_> {
    let mut frame = FrameBudget::new(cancel);
    frame.work = crate::text_engine::Work::permit(cancel, SOURCE_COMMON_WORK);
    frame
}
fn prepare(
    input: &SourceViewportInput,
    stop: &AtomicBool,
) -> Result<SourceViewportPrepared, Cause> {
    prepare_with_frame(input, stop, common_frame_budget(stop))
}
fn prepare_with_frame(
    input: &SourceViewportInput,
    _stop: &AtomicBool,
    mut frame: FrameBudget<'_>,
) -> Result<SourceViewportPrepared, Cause> {
    frame
        .work
        .admit(size_of::<SourceViewportInput>())
        .map_err(|e| {
            if e == crate::text_engine::Error::Cancelled {
                Cause::Cancelled
            } else {
                Cause::Limit("SourceWork")
            }
        })?;
    let lease = input.lease.try_alias()?;
    let output_bytes = size_of::<SourceViewportPrepared>();
    frame
        .reserve(output_bytes)
        .map_err(|_| Cause::Limit("SourceFrame"))?;
    let metadata = lease
        .arena()
        .reserve(
            output_bytes
                .checked_sub(size_of::<SourceLease>())
                .ok_or(Cause::Limit("Source output arithmetic"))?,
        )
        .map_err(|_| Cause::Limit("Source viewport output"))?;
    let model = lease.model();
    let selection = match &input.original_selection {
        Some(raw) => {
            let visible = model
                .visible_range_metered_with(raw.clone(), &mut frame.work)
                .map_err(error)?;
            let real = model
                .source_range_metered_with(visible.clone(), &mut frame.work)
                .map_err(error)?;
            let column = model
                .cell_for_offset_metered_with(visible.start, &mut frame.work)
                .map_err(error)?;
            Some((real, visible, column))
        }
        None => None,
    };
    let cells = usize::from(input.geometry.width)
        .checked_mul(usize::from(input.geometry.height))
        .ok_or(Cause::Limit("Source geometry"))?;
    let glyph_bytes = cells
        .checked_mul(size_of::<SourceGlyph>())
        .ok_or(Cause::Limit("Source geometry"))?;
    frame
        .reserve(glyph_bytes)
        .map_err(|_| Cause::Limit("SourceFrame"))?;
    let mut glyphs = ArenaVec::new(lease.arena().clone());
    glyphs
        .reserve_exact(cells)
        .map_err(|_| Cause::Limit("Source glyph allocation"))?;
    if glyphs.capacity() != cells {
        return Err(Cause::Limit("Source glyph capacity"));
    }
    let mut symbols = ArenaVec::new(lease.arena().clone());
    let mut frame_bytes = output_bytes + glyph_bytes;
    let right = input
        .geometry
        .left
        .checked_add(usize::from(input.geometry.width))
        .ok_or(Cause::Limit("Source geometry"))?;
    if input.geometry.width > 0 && input.geometry.height > 0 {
        for row in
            0..usize::from(input.geometry.height).min(model.line_count().saturating_sub(input.top))
        {
            frame.work.admit(0).map_err(|_| Cause::Cancelled)?;
            let line = input.top + row;
            let mut cursor = model
                .viewport_cursor_metered(line, input.geometry.left, &mut frame.work)
                .map_err(error)?;
            let mut provenance = model
                .provenance_cursor_with(cursor.offset(), &mut frame.work)
                .map_err(error)?;
            while let Some(unit) = model
                .next_unit_metered(&mut cursor, &mut frame.work)
                .map_err(error)?
            {
                if unit.newline || unit.line != line || unit.column >= right {
                    break;
                }
                if unit.width == 0 || unit.column + unit.width <= input.geometry.left {
                    continue;
                }
                if unit.range.len() > SYMBOL_BYTES {
                    return Err(Cause::Limit("SourceSymbol16KiB"));
                }
                let left = unit.column.max(input.geometry.left);
                let end = (unit.column + unit.width).min(right);
                let clipped = unit.column < input.geometry.left || unit.column + unit.width > right;
                let raw = provenance
                    .source_range_with(unit.range.clone(), &mut frame.work)
                    .map_err(error)?;
                let text = &model.visible()[unit.range.clone()];
                let bytes = if clipped || text == "\t" {
                    end - left
                } else {
                    text.len()
                };
                let needed = symbols
                    .len()
                    .checked_add(bytes)
                    .ok_or(Cause::Limit("Source symbols"))?;
                if needed > symbols.capacity() {
                    let target = needed.max(symbols.capacity().saturating_mul(2)).max(16);
                    frame
                        .reserve(target)
                        .map_err(|_| Cause::Limit("SourceFrame"))?;
                    frame.work.admit(symbols.len()).map_err(|e| {
                        if e == crate::text_engine::Error::Cancelled {
                            Cause::Cancelled
                        } else {
                            Cause::Limit("SourceWork")
                        }
                    })?;
                    let old = symbols.capacity();
                    symbols
                        .reserve_exact(target)
                        .map_err(|_| Cause::Limit("Source symbols allocation"))?;
                    if symbols.capacity() != target {
                        return Err(Cause::Limit("Source symbols capacity"));
                    }
                    frame.release(old);
                    frame_bytes = frame_bytes - old + target;
                }
                frame
                    .work
                    .admit(size_of::<SourceGlyph>() + bytes)
                    .map_err(|e| {
                        if e == crate::text_engine::Error::Cancelled {
                            Cause::Cancelled
                        } else {
                            Cause::Limit("SourceWork")
                        }
                    })?;
                let start = checked_u32(symbols.len())?;
                if clipped || text == "\t" {
                    for _ in 0..bytes {
                        symbols
                            .push(b' ')
                            .map_err(|_| Cause::Limit("Source symbols"))?;
                    }
                } else {
                    symbols
                        .extend_from_slice(text.as_bytes())
                        .map_err(|_| Cause::Limit("Source symbols"))?;
                }
                glyphs
                    .push(SourceGlyph {
                        row: row as u16,
                        column: (left - input.geometry.left) as u16,
                        width: (end - left) as u16,
                        start,
                        length: u16::try_from(bytes)
                            .map_err(|_| Cause::Limit("Source symbol length"))?,
                        visible_start: checked_u32(unit.range.start)?,
                        visible_length: u16::try_from(unit.range.len())
                            .map_err(|_| Cause::Limit("Source visible length"))?,
                        source_start: checked_u32(raw.start)?,
                        source_length: u16::try_from(raw.len())
                            .map_err(|_| Cause::Limit("Source raw length"))?,
                    })
                    .map_err(|_| Cause::Limit("Source glyph count"))?;
            }
        }
    }
    frame.work.admit(0).map_err(|_| Cause::Cancelled)?;
    let work = frame.work.used();
    Ok(SourceViewportPrepared {
        lease,
        key: input.key,
        geometry: input.geometry,
        top: input.top,
        glyphs,
        symbols,
        selection,
        work,
        frame_bytes,
        _metadata: metadata,
    })
}
fn run_viewport(input: &SourcePreparationTaskInput, stop: &AtomicBool) -> Outcome<SourcePrepared> {
    let SourcePreparationTaskInput::Viewport(input) = input else {
        return Outcome::Failed(Cause::Stale);
    };
    match prepare(input, stop) {
        Ok(page) => Outcome::Ready(SourcePrepared::Viewport(Box::new(page))),
        Err(Cause::Cancelled) => Outcome::Cancelled,
        Err(Cause::Limit(s)) => Outcome::Limit(s),
        Err(e) => Outcome::Failed(e),
    }
}
pub(crate) fn start_source_viewport<DI: Send + Sync + 'static, DO: Send + 'static>(
    input: SourceViewportInput,
    authority: &mut PreparationAuthority<DI, DO>,
) -> Result<(), Cause> {
    if !authority
        .actor_arena()
        .is_some_and(|a| a.same(input.lease.arena()))
    {
        return Err(Cause::Stale);
    }
    let binding = input.key.binding;
    let run: fn(&SourcePreparationTaskInput, &AtomicBool) -> Outcome<SourcePrepared> = run_viewport;
    authority.start_source_owner(move || {
        OwnedPreparation::spawn(
            Arc::new(SourcePreparationTaskInput::Viewport(input)),
            binding,
            run,
        )
    })
}

#[cfg(test)]
pub(crate) fn start_source_viewport_controlled<DI: Send + Sync + 'static, DO: Send + 'static>(
    mut input: SourceViewportInput,
    authority: &mut PreparationAuthority<DI, DO>,
    after_prepare: impl FnOnce() + Send + 'static,
) -> Result<(), Cause> {
    if !authority
        .actor_arena()
        .is_some_and(|a| a.same(input.lease.arena()))
    {
        return Err(Cause::Stale);
    }
    let binding = input.key.binding;
    let run = move |input: &SourcePreparationTaskInput, stop: &AtomicBool| {
        let SourcePreparationTaskInput::Viewport(input) = input else {
            return Outcome::Failed(Cause::Stale);
        };
        let result = prepare(input, stop);
        after_prepare();
        match result {
            Ok(page) => Outcome::Ready(SourcePrepared::Viewport(Box::new(page))),
            Err(Cause::Cancelled) => Outcome::Cancelled,
            Err(Cause::Limit(s)) => Outcome::Limit(s),
            Err(e) => Outcome::Failed(e),
        }
    };
    let normal: fn(&SourcePreparationTaskInput, &AtomicBool) -> Outcome<SourcePrepared> =
        run_viewport;
    let actual =
        OwnedPreparation::<SourcePreparationTaskInput, SourcePrepared>::known_spawn_bytes(&run)
            .ok_or(Cause::Limit("test capture"))?;
    let baseline =
        OwnedPreparation::<SourcePreparationTaskInput, SourcePrepared>::known_spawn_bytes(&normal)
            .ok_or(Cause::Limit("test capture"))?;
    if actual > baseline {
        input.supplement = Some(
            input
                .lease
                .arena()
                .reserve(actual - baseline)
                .map_err(|_| Cause::Limit("test capture Arena"))?,
        );
    }
    authority.start_source_owner(move || {
        OwnedPreparation::spawn(
            Arc::new(SourcePreparationTaskInput::Viewport(input)),
            binding,
            run,
        )
    })
}

#[cfg(test)]
pub(crate) fn start_source_viewport_diagnostic<DI: Send + Sync + 'static, DO: Send + 'static>(
    input: SourceViewportInput,
    authority: &mut PreparationAuthority<DI, DO>,
    limit: usize,
) -> Result<(), Cause> {
    if !authority
        .actor_arena()
        .is_some_and(|a| a.same(input.lease.arena()))
    {
        return Err(Cause::Stale);
    }
    let binding = input.key.binding;
    let run = move |input: &SourcePreparationTaskInput, stop: &AtomicBool| {
        let SourcePreparationTaskInput::Viewport(input) = input else {
            return Outcome::Failed(Cause::Stale);
        };
        let mut frame = FrameBudget::new(stop);
        frame.work = crate::text_engine::Work::permit(stop, limit);
        match prepare_with_frame(input, stop, frame) {
            Ok(p) => {
                eprintln!(
                    "SC-W01 COMPONENT Index {:?}",
                    crate::text_engine::common_lookup_counters()
                );
                Outcome::Ready(SourcePrepared::Viewport(Box::new(p)))
            }
            Err(Cause::Cancelled) => Outcome::Cancelled,
            Err(Cause::Limit(s)) => Outcome::Limit(s),
            Err(e) => Outcome::Failed(e),
        }
    };
    let normal: fn(&SourcePreparationTaskInput, &AtomicBool) -> Outcome<SourcePrepared> =
        run_viewport;
    assert_eq!(
        OwnedPreparation::<SourcePreparationTaskInput, SourcePrepared>::known_spawn_bytes(&run),
        OwnedPreparation::<SourcePreparationTaskInput, SourcePrepared>::known_spawn_bytes(&normal)
    );
    authority.start_source_owner(move || {
        OwnedPreparation::spawn(
            Arc::new(SourcePreparationTaskInput::Viewport(input)),
            binding,
            run,
        )
    })
}
