//! Complete viewport admission precedes every text copy and widget construction.
use crate::{
    presentation::{PrepareError, SafeTextModel},
    text_engine::{Error, Identity, Point, Work},
};
use ratatui::{
    style::Style,
    text::{Line, Span},
};
use std::{mem::size_of, ops::Range, sync::atomic::AtomicBool};
pub(crate) const SYMBOL_BYTES: usize = 16 * 1024;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Reason {
    Symbol,
    FrameSymbols,
    Work,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Domain {
    Source,
    NuValue { key: bool },
    FileValue { node: u32 },
    Yaml { node: u32, metadata: bool },
    Xml { node: u32, domain: &'static str },
    Preview,
}
#[derive(Clone, Debug)]
pub(crate) enum SourceRelation {
    Exact(Range<usize>),
    ControlProjection(Range<usize>),
    EnclosingNodeSpan(Range<usize>),
    None,
}
#[derive(Clone, Debug)]
pub(crate) struct Reference {
    pub domain: Domain,
    pub range: Range<usize>,
    pub range_exact: bool,
    pub visible: Range<usize>,
    pub source: SourceRelation,
    pub reason: Reason,
    pub identity: Identity,
    pub generation: Option<u64>,
    pub unpositioned_hit: Option<Range<usize>>,
}
impl Reference {
    pub fn message(&self) -> String {
        let relation = match &self.source {
            SourceRelation::Exact(r) => format!("origen exacto {}..{}", r.start, r.end),
            SourceRelation::ControlProjection(r) => {
                format!("proyección de controles; origen {}..{}", r.start, r.end)
            }
            SourceRelation::EnclosingNodeSpan(r) => {
                format!("span contenedor de Fuente {}..{}", r.start, r.end)
            }
            SourceRelation::None => "sin span literal de Fuente".to_owned(),
        };
        let position = self.unpositioned_hit.as_ref().map_or(String::new(), |hit| {
            format!(
                "; coincidencia visible {}..{} seleccionada; posición visual no calculada",
                hit.start, hit.end
            )
        });
        let _owner = &self.identity;
        let coordinates = if self.range_exact {
            format!(
                "rango {}..{}; visible {}..{}",
                self.range.start, self.range.end, self.visible.start, self.visible.end
            )
        } else {
            format!(
                "cota de descriptor {} bytes; rango no calculado",
                self.range.end
            )
        };
        let reason = match self.reason {
            Reason::Symbol => "símbolo supera 16KiB",
            Reason::FrameSymbols => "símbolos/metadata superan 256KiB",
            Reason::Work => "trabajo de consulta supera 256KiB",
        };
        format!(
            "referencia de viewport: {reason}; dominio {:?}; {coordinates}; {relation}{position}{}",
            self.domain,
            self.generation
                .map_or(String::new(), |g| format!("; generación {g}"))
        )
    }
    pub fn text(
        model: &SafeTextModel,
        domain: Domain,
        visible: Range<usize>,
        reason: Reason,
        cancel: &AtomicBool,
    ) -> Result<Self, PrepareError> {
        let raw = model.source_range_with(visible.clone(), &mut Work::preparation(cancel))?;
        let source = if domain == Domain::Source {
            if raw.len() == visible.len() {
                SourceRelation::Exact(raw.clone())
            } else {
                SourceRelation::ControlProjection(raw.clone())
            }
        } else {
            SourceRelation::None
        };
        Ok(Self {
            domain,
            range: raw,
            range_exact: true,
            visible,
            source,
            reason,
            identity: model.identity(),
            generation: None,
            unpositioned_hit: None,
        })
    }
}
pub(crate) struct FrameBudget<'a> {
    pub work: Work<'a>,
    bytes: usize,
    reference_reserved: bool,
    pub failed: Option<Reference>,
}
impl<'a> FrameBudget<'a> {
    pub fn new(cancel: &'a AtomicBool) -> Self {
        Self {
            work: Work::frame(cancel),
            bytes: 0,
            reference_reserved: false,
            failed: None,
        }
    }
    pub fn reserve(&mut self, bytes: usize) -> Result<(), Reason> {
        let next = self.bytes.checked_add(bytes).ok_or(Reason::FrameSymbols)?;
        if next > 256 * 1024 {
            return Err(Reason::FrameSymbols);
        }
        self.bytes = next;
        Ok(())
    }
    #[cfg(test)]
    pub fn symbol(&mut self, bytes: usize) -> Result<(), Reason> {
        if bytes > SYMBOL_BYTES {
            return Err(Reason::Symbol);
        }
        self.reserve(bytes)
    }
    pub fn release(&mut self, bytes: usize) {
        self.bytes = self
            .bytes
            .checked_sub(bytes)
            .expect("owned viewport reservation");
    }
    pub fn reference_reserve(&mut self) -> Result<(), Reason> {
        if !self.reference_reserved {
            self.reserve(4096)?;
            self.work.admit(8192).map_err(|_| Reason::Work)?;
            self.reference_reserved = true;
        }
        Ok(())
    }
    pub fn preview_failure(&mut self, range: Range<usize>, reason: Reason) {
        if self.failed.is_none() {
            self.failed = Some(Reference {
                domain: Domain::Preview,
                visible: range.clone(),
                range,
                range_exact: false,
                source: SourceRelation::None,
                reason,
                identity: Identity::new(),
                generation: None,
                unpositioned_hit: None,
            });
        }
    }
    fn preview_known_failure(&mut self, range: Range<usize>, reason: Reason) {
        self.preview_failure(range, reason);
        if let Some(reference) = &mut self.failed {
            reference.range_exact = true;
        }
    }
    /// Bounded preview formatter: its upper bound is reserved before invocation.
    pub fn preview(
        &mut self,
        max_chars: usize,
        width: usize,
        make: impl FnOnce() -> String,
    ) -> String {
        let upper = match max_chars.checked_mul(36).and_then(|n| n.checked_add(1024)) {
            Some(n) => n,
            None => {
                self.preview_failure(0..max_chars, Reason::FrameSymbols);
                return String::new();
            }
        };
        self.preview_bytes(upper, width, make)
    }
    pub fn preview_bytes(
        &mut self,
        upper: usize,
        width: usize,
        make: impl FnOnce() -> String,
    ) -> String {
        if self.failed.is_some() {
            return String::new();
        }
        if let Err(reason) = self.reserve(upper) {
            self.preview_failure(0..upper, reason);
            return String::new();
        }
        if self.work.admit(upper).is_err() {
            self.release(upper);
            self.preview_failure(0..upper, Reason::Work);
            return String::new();
        }
        let text = make();
        if text.capacity() > upper {
            drop(text);
            self.release(upper);
            self.preview_failure(0..upper, Reason::FrameSymbols);
            return String::new();
        }
        let result = self.label(&text, width);
        drop(text);
        self.release(upper);
        result
    }
    /// No cluster is copied until segmentation, width and growth are admitted.
    pub fn label(&mut self, text: &str, width: usize) -> String {
        if self.failed.is_some() || self.work.cancel.load(std::sync::atomic::Ordering::Relaxed) {
            return String::new();
        }
        let mut cursor = unicode_segmentation::GraphemeCursor::new(0, text.len(), true);
        let mut byte = 0;
        let mut column = 0;
        let mut output = String::new();
        let mut charged = 0usize;
        while byte < text.len() && column < width {
            let end = match crate::text_engine::next(text, &mut cursor, &mut self.work) {
                Ok(Some(n)) => n,
                Ok(None) => break,
                Err(Error::Cancelled) => return String::new(),
                Err(_) => {
                    self.preview_known_failure(0..text.len(), Reason::Work);
                    break;
                }
            };
            let raw = &text[byte..end];
            if raw.len() > SYMBOL_BYTES {
                self.preview_known_failure(byte..end, Reason::Symbol);
                break;
            }
            let shown = if raw == "\r\n" {
                "��"
            } else if raw.chars().next().is_some_and(char::is_control) {
                "�"
            } else {
                raw
            };
            let cells = match crate::text_engine::width(shown, column, &mut self.work) {
                Ok(n) => n,
                Err(Error::Cancelled) => return String::new(),
                Err(_) => {
                    self.preview_known_failure(byte..end, Reason::Work);
                    break;
                }
            };
            if cells > 0 {
                let length = if column + cells > width {
                    width - column
                } else {
                    shown.len()
                };
                let needed = match output.len().checked_add(length) {
                    Some(n) => n,
                    None => {
                        self.preview_known_failure(byte..end, Reason::FrameSymbols);
                        break;
                    }
                };
                if needed > output.capacity() {
                    let capacity = needed.max(output.capacity().max(8).saturating_mul(2));
                    let next = match capacity.checked_mul(3) {
                        Some(n) => n,
                        None => {
                            self.preview_known_failure(byte..end, Reason::FrameSymbols);
                            break;
                        }
                    };
                    if let Err(reason) = self.reserve(next) {
                        self.preview_known_failure(byte..end, reason);
                        break;
                    }
                    if output.try_reserve_exact(capacity - output.len()).is_err()
                        || output.capacity() > capacity
                    {
                        self.release(next);
                        self.preview_known_failure(byte..end, Reason::FrameSymbols);
                        break;
                    }
                    self.release(charged);
                    charged = next;
                }
                if self.work.project(length).is_err() {
                    self.preview_known_failure(byte..end, Reason::Work);
                    break;
                }
                if column + cells > width {
                    output.extend(std::iter::repeat_n(' ', length));
                } else {
                    output.push_str(shown);
                }
            }
            column += cells;
            byte = end;
        }
        if self.failed.is_some() {
            drop(output);
            self.release(charged);
            String::new()
        } else {
            output
        }
    }
}
#[derive(Debug)]
pub(crate) enum TextError {
    Prepare(PrepareError),
    Reference(Box<Reference>),
}
impl TextError {
    fn reference(value: Reference) -> Self {
        // Fixed-size metadata only, included in the 4096-byte reference reserve.
        Self::Reference(Box::new(value))
    }
}
impl From<PrepareError> for TextError {
    fn from(error: PrepareError) -> Self {
        Self::Prepare(error)
    }
}
struct Row {
    line: usize,
    start: Point,
    cells: usize,
    bytes: usize,
    replay: usize,
}
pub(crate) struct TextPlan {
    owner: Identity,
    rows: Vec<Row>,
    domain: Domain,
    left: usize,
    right: usize,
}
impl TextPlan {
    pub fn prepare(
        model: &SafeTextModel,
        domain: Domain,
        top: usize,
        count: usize,
        window: Range<usize>,
        frame: &mut FrameBudget<'_>,
    ) -> Result<Self, TextError> {
        let cancel = frame.work.cancel;
        if let Err(reason) = frame.reference_reserve() {
            return Err(TextError::reference(Reference::text(
                model,
                domain.clone(),
                model.line_range(top).unwrap_or(0..0),
                reason,
                cancel,
            )?));
        }
        let count = count.min(model.line_count().saturating_sub(top));
        let left = window.start;
        let right = window.end;
        let fallback = model.line_range(top).unwrap_or(0..0);
        let reference = |range, reason| {
            Reference::text(model, domain.clone(), range, reason, frame.work.cancel)
                .map(TextError::reference)
                .map_err(TextError::Prepare)
        };
        let metadata = count
            .checked_mul(size_of::<Row>() + size_of::<Line<'static>>())
            .ok_or(TextError::Prepare(PrepareError::Limit(
                "desbordamiento de viewport",
            )))?;
        if let Err(reason) = frame.reserve(metadata) {
            return Err(reference(fallback, reason)?);
        }
        let mut rows = Vec::new();
        rows.try_reserve_exact(count)
            .map_err(|_| TextError::Prepare(PrepareError::Limit("memoria de viewport")))?;
        if rows.capacity() > count {
            return Err(reference(fallback, Reason::FrameSymbols)?);
        }
        for line in top..top + count {
            let range = model.line_range(line).unwrap();
            let before = frame.work.used();
            let start = match model.viewport_cursor(line, left, &mut frame.work) {
                Ok(p) => p,
                Err(PrepareError::Cancelled) => {
                    return Err(TextError::Prepare(PrepareError::Cancelled));
                }
                Err(_) => return Err(reference(range, Reason::Work)?),
            };
            let mut cursor = start.clone();
            let mut provenance = model.provenance_cursor(start.offset());
            let mut cells = 0usize;
            let mut bytes = 0usize;
            loop {
                let unit = match model.next_unit(&mut cursor, &mut frame.work) {
                    Ok(Some(u)) => u,
                    Ok(None) => break,
                    Err(PrepareError::Cancelled) => {
                        return Err(TextError::Prepare(PrepareError::Cancelled));
                    }
                    Err(_) => return Err(reference(range.clone(), Reason::Work)?),
                };
                if unit.range.start >= range.end || unit.newline || unit.column >= right {
                    break;
                }
                if unit.column + unit.width > left || unit.width == 0 && unit.column >= left {
                    if unit.range.len() > SYMBOL_BYTES {
                        return Err(reference(unit.range, Reason::Symbol)?);
                    }
                    if unit.width > 0 {
                        let shown = if model.visible().get(unit.range.clone()) == Some("\t")
                            || unit.column < left
                            || unit.column + unit.width > right
                        {
                            right.min(unit.column + unit.width) - left.max(unit.column)
                        } else {
                            unit.range.len()
                        };
                        provenance.source_range_with(unit.range, &mut frame.work)?;
                        cells += 1;
                        bytes = bytes.checked_add(shown).ok_or(TextError::Prepare(
                            PrepareError::Limit("desbordamiento de viewport"),
                        ))?;
                    }
                }
            }
            let replay = frame.work.used() - before;
            if frame
                .work
                .admit(
                    replay
                        .checked_add(bytes)
                        .ok_or(TextError::Prepare(PrepareError::Limit(
                            "desbordamiento de viewport",
                        )))?,
                )
                .is_err()
            {
                return Err(reference(range.clone(), Reason::Work)?);
            }
            if let Err(reason) = frame.reserve(bytes * 2 + cells * size_of::<Span<'static>>()) {
                return Err(reference(range, reason)?);
            }
            rows.push(Row {
                line,
                start,
                cells,
                bytes,
                replay,
            });
        }
        Ok(Self {
            owner: model.identity(),
            rows,
            domain,
            left,
            right,
        })
    }
    pub fn materialize(
        self,
        model: &SafeTextModel,
        cancel: &AtomicBool,
        mut style: impl FnMut(Range<usize>, Range<usize>) -> Style,
    ) -> Result<Vec<Line<'static>>, TextError> {
        if !self.owner.same(&model.identity()) {
            return Err(TextError::Prepare(PrepareError::Limit(
                "identidad de viewport no válida",
            )));
        }
        let mut result = Vec::new();
        result
            .try_reserve_exact(self.rows.len())
            .map_err(|_| TextError::Prepare(PrepareError::Limit("memoria de viewport")))?;
        if result.capacity() > self.rows.len() {
            return Err(TextError::Prepare(PrepareError::Limit(
                "capacidad de viewport",
            )));
        }
        for row in self.rows {
            let range = model.line_range(row.line).unwrap();
            let mut spans = Vec::new();
            spans
                .try_reserve_exact(row.cells)
                .map_err(|_| TextError::Prepare(PrepareError::Limit("memoria de viewport")))?;
            if spans.capacity() > row.cells {
                return Err(TextError::Prepare(PrepareError::Limit(
                    "capacidad de viewport",
                )));
            }
            let mut cursor = row.start;
            let mut provenance = model.provenance_cursor(cursor.offset());
            let mut work = Work::permit(cancel, row.replay + row.bytes);
            work.project(row.bytes)
                .map_err(|_| TextError::Prepare(PrepareError::Cancelled))?;
            let mut copied = 0;
            while let Some(unit) = model.next_unit(&mut cursor, &mut work)? {
                if unit.range.start >= range.end || unit.newline || unit.column >= self.right {
                    break;
                }
                if unit.width > 0 && unit.column + unit.width > self.left {
                    let raw = provenance.source_range_with(unit.range.clone(), &mut work)?;
                    let shown_len = if &model.visible()[unit.range.clone()] == "\t"
                        || unit.column < self.left
                        || unit.column + unit.width > self.right
                    {
                        self.right.min(unit.column + unit.width) - self.left.max(unit.column)
                    } else {
                        unit.range.len()
                    };
                    if shown_len > SYMBOL_BYTES
                        || spans.len() >= row.cells
                        || copied + shown_len > row.bytes
                    {
                        return Err(TextError::reference(Reference::text(
                            model,
                            self.domain.clone(),
                            unit.range,
                            Reason::FrameSymbols,
                            cancel,
                        )?));
                    }
                    let mut shown = String::new();
                    shown.try_reserve_exact(shown_len).map_err(|_| {
                        TextError::Prepare(PrepareError::Limit("memoria de símbolo"))
                    })?;
                    if shown.capacity() > shown_len {
                        return Err(TextError::Prepare(PrepareError::Limit(
                            "capacidad de símbolo",
                        )));
                    }
                    if &model.visible()[unit.range.clone()] == "\t"
                        || unit.column < self.left
                        || unit.column + unit.width > self.right
                    {
                        shown.extend(std::iter::repeat_n(' ', shown_len));
                    } else {
                        shown.push_str(&model.visible()[unit.range.clone()]);
                    }
                    copied += shown.capacity();
                    spans.push(Span::styled(shown, style(raw, unit.range)));
                }
            }
            if copied != row.bytes || spans.len() != row.cells {
                return Err(TextError::Prepare(PrepareError::Limit(
                    "plan de viewport obsoleto",
                )));
            }
            result.push(Line::from(spans));
        }
        if cancel.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(TextError::Prepare(PrepareError::Cancelled));
        }
        Ok(result)
    }
}

#[cfg(test)]
mod unicode_admission_tests {
    use super::*;
    use crate::search::{Direction, SearchOutcome, find_source};
    #[test]
    fn complete_source_references_whole_cluster_before_any_row_materializes() {
        let cancel = AtomicBool::new(false);
        let giant = format!("a{}", "\u{301}".repeat(20000));
        let original = format!("ordinary\n{giant}\nafter");
        let model = SafeTextModel::new(original.clone(), &cancel).unwrap();
        let mut frame = FrameBudget::new(&cancel);
        let Err(TextError::Reference(reference)) =
            TextPlan::prepare(&model, Domain::Source, 0, 3, 0..80, &mut frame)
        else {
            panic!("whole viewport must reference the complete oversized cluster");
        };
        assert_eq!(reference.reason, Reason::Symbol);
        assert_eq!(reference.range, 9..9 + giant.len());
        assert_eq!(reference.visible, reference.range);
        assert!(matches!(reference.source, SourceRelation::Exact(_)));
        assert_eq!(model.original(), original);
        let SearchOutcome::SourceFound { original, visible } =
            find_source(&model, "\u{301}\u{301}", None, Direction::Forward, &cancel)
        else {
            panic!()
        };
        assert_eq!(original, 10..14);
        assert_eq!(visible, original);
        assert_eq!(model.cell_for_offset(12), 0);
    }
    #[test]
    fn symbol_exact_and_plus_one_zero_cluster_are_not_silent_clipping() {
        let cancel = AtomicBool::new(false);
        let exact = format!("é{}", "\u{301}".repeat(8191));
        assert_eq!(exact.len(), 16384);
        let model = SafeTextModel::new(exact.clone(), &cancel).unwrap();
        let mut frame = FrameBudget::new(&cancel);
        let lines = TextPlan::prepare(&model, Domain::Source, 0, 1, 0..1, &mut frame)
            .unwrap()
            .materialize(&model, &cancel, |_, _| Style::default())
            .unwrap();
        assert_eq!(lines[0].spans[0].content, exact);
        for text in [
            format!("a{}", "\u{301}".repeat(8192)),
            "\u{301}".repeat(8193),
        ] {
            let model = SafeTextModel::new(text.clone(), &cancel).unwrap();
            let mut frame = FrameBudget::new(&cancel);
            let Err(TextError::Reference(reference)) =
                TextPlan::prepare(&model, Domain::Source, 0, 1, 0..1, &mut frame)
            else {
                panic!()
            };
            assert_eq!(reference.reason, Reason::Symbol);
            assert_eq!(reference.range, 0..text.len());
        }
    }
    #[test]
    fn stale_plan_rejects_foreign_owner_before_line_lookup_or_copy() {
        let cancel = AtomicBool::new(false);
        let a = SafeTextModel::new("one\ntwo\nthree".into(), &cancel).unwrap();
        let b = SafeTextModel::new("".into(), &cancel).unwrap();
        let mut frame = FrameBudget::new(&cancel);
        let plan =
            TextPlan::prepare(&a, Domain::NuValue { key: false }, 1, 2, 0..80, &mut frame).unwrap();
        assert!(matches!(
            plan.materialize(&b, &cancel, |_, _| panic!(
                "no style callback for stale owner"
            )),
            Err(TextError::Prepare(_))
        ));
    }
    #[test]
    fn opaque_preview_formatter_is_not_invoked_after_admission_failure() {
        let cancel = AtomicBool::new(false);
        let mut frame = FrameBudget::new(&cancel);
        frame.reserve(256 * 1024).unwrap();
        assert_eq!(
            frame.preview(32, 16, || panic!("formatter must remain uncalled")),
            ""
        );
        assert_eq!(frame.failed.unwrap().reason, Reason::FrameSymbols);
    }
    #[test]
    fn exact_aggregate_quota_and_plus_one() {
        let cancel = AtomicBool::new(false);
        let mut frame = FrameBudget::new(&cancel);
        assert_eq!(frame.symbol(16384), Ok(()));
        assert_eq!(frame.symbol(16385), Err(Reason::Symbol));
        assert_eq!(frame.reserve(256 * 1024 - 16384), Ok(()));
        assert_eq!(frame.reserve(1), Err(Reason::FrameSymbols));
    }
}
