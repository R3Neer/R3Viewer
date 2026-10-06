//! Safe visible text with sparse provenance checkpoints. Source bytes stay intact.
use crate::text_engine::{self, Identity, Index, Work};
use std::{
    ops::Range,
    sync::atomic::{AtomicBool, Ordering},
};
#[cfg(test)]
use unicode_segmentation::UnicodeSegmentation;

const CHECKPOINT_BYTES: usize = 4096;
const MAX_LINES: usize = 1_000_000;
const MAX_VISIBLE_BYTES: usize = 256 * 1024 * 1024;

#[derive(Debug)]
pub enum PrepareError {
    Cancelled,
    Limit(&'static str),
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)] // b0 receipt remains internal until session adoption.
pub(crate) struct SourceAllocationReceipt {
    pub original_capacity: usize,
    pub visible_capacity: usize,
    pub checkpoint_bytes: usize,
    pub line_bytes: usize,
    pub index_bytes: usize,
    pub index_growth_peak: usize,
    pub model_bytes: usize,
    pub known_heap_bytes: usize,
    pub original_arc_bytes: usize,
    pub identity_arc_bytes: usize,
}
#[derive(Debug)]
pub struct SafeTextModel {
    original: std::sync::Arc<String>,
    visible: String,
    // (source byte, visible byte), always at character boundaries.
    checkpoints: Vec<(usize, usize)>,
    lines: Vec<usize>,
    max_width: usize,
    cell_index: Index,
    identity: Identity,
}

pub(crate) fn engine_error(error: text_engine::Error) -> PrepareError {
    match error {
        text_engine::Error::Cancelled => PrepareError::Cancelled,
        text_engine::Error::Work => {
            PrepareError::Limit("trabajo de viewport excede256KiB; referencia requerida")
        }
        text_engine::Error::Invalid => PrepareError::Limit("identidad de texto no válida"),
    }
}
use crate::text_projection::{append as append_visible, len as replacement_len};

impl SafeTextModel {
    pub fn new(original: String, cancelled: &AtomicBool) -> Result<Self, PrepareError> {
        Self::from_shared(std::sync::Arc::new(original), cancelled)
    }
    /// Prepare the existing safe Source representation from the same snapshot.
    pub fn from_shared(
        original: std::sync::Arc<String>,
        cancelled: &AtomicBool,
    ) -> Result<Self, PrepareError> {
        if cancelled.load(Ordering::Relaxed) {
            return Err(PrepareError::Cancelled);
        }
        let mut visible = String::with_capacity(original.len());
        let mut checkpoints = vec![(0, 0)];
        let mut lines = vec![0];
        let mut next_checkpoint = CHECKPOINT_BYTES;
        let mut chars = original.char_indices().peekable();
        let mut count = 0usize;
        while let Some((offset, ch)) = chars.next() {
            if count.is_multiple_of(4096) && cancelled.load(Ordering::Relaxed) {
                return Err(PrepareError::Cancelled);
            }
            count += 1;
            if offset >= next_checkpoint {
                checkpoints.push((offset, visible.len()));
                next_checkpoint = offset.saturating_add(CHECKPOINT_BYTES);
            }
            let next = chars.peek().map(|(_, ch)| *ch);
            let piece_len = replacement_len(ch, next);
            if piece_len > MAX_VISIBLE_BYTES.saturating_sub(visible.len()) {
                return Err(PrepareError::Limit(
                    "texto visible excede256MiB; no se muestra un prefijo",
                ));
            }
            append_visible(&mut visible, ch, next);
            if ch == '\n' {
                if lines.len() >= MAX_LINES {
                    return Err(PrepareError::Limit(
                        "documento excede un millón de líneas; no se muestra un prefijo",
                    ));
                }
                lines.push(visible.len());
            }
        }
        if cancelled.load(Ordering::Relaxed) {
            return Err(PrepareError::Cancelled);
        }
        checkpoints.push((original.len(), visible.len()));
        let identity = Identity::new();
        let mut work = Work::preparation(cancelled);
        let cell_index = Index::prepare(
            &visible,
            identity.clone(),
            &mut work,
            CHECKPOINT_BYTES,
            usize::MAX,
        )
        .map_err(engine_error)?;
        let max_width = cell_index.max_width;
        Ok(Self {
            original,
            visible,
            checkpoints,
            lines,
            max_width,
            cell_index,
            identity,
        })
    }
    /// Capacities of the actual completed model; shared original is listed once, not new RSS proof.
    #[allow(dead_code)] // b0 library is not connected to the session yet.
    pub(crate) fn allocation_receipt(&self) -> Option<SourceAllocationReceipt> {
        let checkpoint_bytes = self
            .checkpoints
            .capacity()
            .checked_mul(std::mem::size_of::<(usize, usize)>())?;
        let line_bytes = self
            .lines
            .capacity()
            .checked_mul(std::mem::size_of::<usize>())?;
        let model_bytes = std::mem::size_of::<Self>();
        let original_arc_bytes = std::mem::size_of::<String>() + 2 * std::mem::size_of::<usize>();
        let identity_arc_bytes = 2 * std::mem::size_of::<usize>();
        let known_heap_bytes = original_arc_bytes
            .checked_add(identity_arc_bytes)?
            .checked_add(self.original.capacity())?
            .checked_add(self.visible.capacity())?
            .checked_add(checkpoint_bytes)?
            .checked_add(line_bytes)?
            .checked_add(self.cell_index.allocated)?;
        Some(SourceAllocationReceipt {
            original_capacity: self.original.capacity(),
            visible_capacity: self.visible.capacity(),
            checkpoint_bytes,
            line_bytes,
            index_bytes: self.cell_index.allocated,
            index_growth_peak: self.cell_index.growth_peak,
            model_bytes,
            known_heap_bytes,
            original_arc_bytes,
            identity_arc_bytes,
        })
    }
    /// Monotonic renderer cursor; one bounded checkpoint scan per visible row.
    pub(crate) fn identity(&self) -> Identity {
        self.identity.clone()
    }
    pub fn provenance_cursor(&self, visible_offset: usize) -> ProvenanceCursor<'_> {
        let target = visible_offset.min(self.visible.len());
        let index = self
            .checkpoints
            .partition_point(|(_, safe)| *safe <= target)
            .saturating_sub(1);
        let (raw, safe) = self.checkpoints[index];
        ProvenanceCursor {
            chars: self.original[raw..].char_indices().peekable(),
            base: raw,
            safe,
            source_end: self.original.len(),
            previous_end: 0,
        }
    }
    pub fn original_handle(&self) -> std::sync::Arc<String> {
        self.original.clone()
    }
    pub fn original(&self) -> &str {
        &self.original
    }
    pub fn visible(&self) -> &str {
        &self.visible
    }
    pub fn max_width(&self) -> usize {
        self.max_width
    }
    /// First checkpoint at/before the horizontal viewport. Long lines need not
    /// be scanned from byte zero on every draw; checkpoints are grapheme-aligned.
    pub fn viewport_start(&self, line: usize, left: usize) -> Option<(usize, usize)> {
        let cancel = AtomicBool::new(false);
        let mut work = Work::preparation(&cancel);
        self.viewport_start_with(line, left, &mut work).ok()
    }
    pub(crate) fn viewport_cursor(
        &self,
        line: usize,
        left: usize,
        work: &mut Work<'_>,
    ) -> Result<text_engine::Point, PrepareError> {
        let range = self
            .line_range(line)
            .ok_or(PrepareError::Limit("línea inválida"))?;
        let at = self
            .cell_index
            .start_cell(&self.visible, &self.identity, line, left)
            .map_err(engine_error)?;
        let at = if at.line() == line {
            at
        } else {
            self.cell_index
                .line_entry(&self.visible, &self.identity, range.clone(), line)
                .map_err(engine_error)?
        };
        work.admit(0).map_err(engine_error)?;
        Ok(at)
    }
    pub(crate) fn viewport_start_with(
        &self,
        line: usize,
        left: usize,
        work: &mut Work<'_>,
    ) -> Result<(usize, usize), PrepareError> {
        self.viewport_cursor(line, left, work)
            .map(|p| (p.offset(), p.column()))
    }

    pub(crate) fn next_unit(
        &self,
        at: &mut text_engine::Point,
        work: &mut Work<'_>,
    ) -> Result<Option<text_engine::Unit>, PrepareError> {
        self.cell_index
            .step(&self.visible, &self.identity, at, work)
            .map_err(engine_error)
    }
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
    pub fn line_range(&self, line: usize) -> Option<Range<usize>> {
        let start = *self.lines.get(line)?;
        let mut end = self
            .lines
            .get(line + 1)
            .copied()
            .unwrap_or(self.visible.len());
        if end > start && self.visible.as_bytes()[end - 1] == b'\n' {
            end -= 1;
        }
        if end > start && self.visible.as_bytes()[end - 1] == b'\r' {
            end -= 1;
        }
        Some(start..end)
    }
    pub fn line(&self, line: usize) -> Option<&str> {
        self.line_range(line).map(|r| &self.visible[r])
    }
    pub fn line_for_offset(&self, offset: usize) -> usize {
        self.lines
            .partition_point(|start| *start <= offset.min(self.visible.len()))
            .saturating_sub(1)
    }
    fn source_boundary_with(
        &self,
        visible_offset: usize,
        end: bool,
        work: &mut Work<'_>,
    ) -> Result<usize, PrepareError> {
        let target = visible_offset.min(self.visible.len());
        let checkpoint = self
            .checkpoints
            .partition_point(|(_, safe)| *safe <= target)
            .saturating_sub(1);
        let (raw, mut safe) = self.checkpoints[checkpoint];
        let mut chars = self.original[raw..].char_indices().peekable();
        while let Some((relative, ch)) = chars.next() {
            work.project(ch.len_utf8()).map_err(engine_error)?;
            let offset = raw + relative;
            if safe == target {
                return Ok(offset);
            }
            let len = replacement_len(ch, chars.peek().map(|(_, ch)| *ch));
            if safe + len > target {
                return Ok(if end { offset + ch.len_utf8() } else { offset });
            }
            safe += len;
        }
        Ok(self.original.len())
    }
    /// A partial match in a control placeholder maps to its complete source char.
    pub fn source_range(&self, visible: Range<usize>) -> Range<usize> {
        let cancel = AtomicBool::new(false);
        let mut work = Work::preparation(&cancel);
        self.source_range_with(visible, &mut work)
            .expect("valid source mapping")
    }
    pub(crate) fn source_range_with(
        &self,
        visible: Range<usize>,
        work: &mut Work<'_>,
    ) -> Result<Range<usize>, PrepareError> {
        Ok(self.source_boundary_with(visible.start, false, work)?
            ..self.source_boundary_with(visible.end, true, work)?)
    }
    fn visible_boundary_with(
        &self,
        source_offset: usize,
        end: bool,
        work: &mut Work<'_>,
    ) -> Result<usize, PrepareError> {
        let target = source_offset.min(self.original.len());
        let checkpoint = self
            .checkpoints
            .partition_point(|(raw, _)| *raw <= target)
            .saturating_sub(1);
        let (raw, mut safe) = self.checkpoints[checkpoint];
        let mut chars = self.original[raw..].char_indices().peekable();
        while let Some((relative, ch)) = chars.next() {
            work.project(ch.len_utf8()).map_err(engine_error)?;
            let offset = raw + relative;
            if offset == target {
                return Ok(safe);
            }
            let len = replacement_len(ch, chars.peek().map(|(_, ch)| *ch));
            if offset + ch.len_utf8() > target {
                return Ok(if end { safe + len } else { safe });
            }
            safe += len;
        }
        Ok(self.visible.len())
    }
    pub fn visible_range(&self, source: Range<usize>) -> Range<usize> {
        let cancel = AtomicBool::new(false);
        let mut work = Work::preparation(&cancel);
        self.visible_range_with(source, &mut work)
            .expect("valid visible mapping")
    }
    pub(crate) fn visible_range_with(
        &self,
        source: Range<usize>,
        work: &mut Work<'_>,
    ) -> Result<Range<usize>, PrepareError> {
        Ok(self.visible_boundary_with(source.start, false, work)?
            ..self.visible_boundary_with(source.end, true, work)?)
    }
    pub fn cell_for_offset(&self, offset: usize) -> usize {
        let cancel = AtomicBool::new(false);
        let mut work = Work::preparation(&cancel);
        self.cell_for_offset_with(offset, &mut work).unwrap_or(0)
    }
    pub(crate) fn cell_for_offset_with(
        &self,
        offset: usize,
        work: &mut Work<'_>,
    ) -> Result<usize, PrepareError> {
        self.cell_index
            .position(&self.visible, &self.identity, offset, work)
            .map(|(_, column)| column)
            .map_err(engine_error)
    }
}

/// Maps adjoining visible intervals without rescanning a source prefix.
pub struct ProvenanceCursor<'a> {
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    base: usize,
    safe: usize,
    source_end: usize,
    previous_end: usize,
}
impl ProvenanceCursor<'_> {
    pub fn source_range(&mut self, visible: Range<usize>) -> Range<usize> {
        let cancel = AtomicBool::new(false);
        let mut work = Work::preparation(&cancel);
        self.source_range_with(visible, &mut work)
            .expect("valid monotonic mapping")
    }
    fn boundary(
        &mut self,
        target: usize,
        end: bool,
        work: &mut Work<'_>,
    ) -> Result<usize, PrepareError> {
        loop {
            let Some(&(relative, ch)) = self.chars.peek() else {
                return Ok(self.source_end);
            };
            work.project(ch.len_utf8()).map_err(engine_error)?;
            let raw = self.base + relative;
            if self.safe == target {
                return Ok(raw);
            }
            let mut lookahead = self.chars.clone();
            lookahead.next();
            let length = replacement_len(ch, lookahead.peek().map(|(_, ch)| *ch));
            if self.safe + length > target {
                return Ok(if end { raw + ch.len_utf8() } else { raw });
            }
            self.safe += length;
            self.chars.next();
        }
    }
    pub(crate) fn source_range_with(
        &mut self,
        visible: Range<usize>,
        work: &mut Work<'_>,
    ) -> Result<Range<usize>, PrepareError> {
        assert!(
            visible.start >= self.previous_end && visible.end >= visible.start,
            "provenance intervals must be monotonic"
        );
        let start = self.boundary(visible.start, false, work)?;
        let end = self.boundary(visible.end, true, work)?;
        self.previous_end = visible.end;
        Ok(start..end)
    }
}

pub fn cell_width(grapheme: &str, column: usize) -> usize {
    if grapheme == "\t" {
        8 - column % 8
    } else {
        text_engine::width(
            grapheme,
            column,
            &mut Work::preparation(&AtomicBool::new(false)),
        )
        .expect("immutable width without cancellation or quota")
    }
}

/// Whole graphemes only; wide fragments become blank cells, tabs retain origin.
pub fn slice_cells(line: &str, left: usize, width: usize) -> String {
    let right = left.saturating_add(width);
    let mut column: usize = 0;
    let mut result = String::new();
    let cancel = AtomicBool::new(false);
    let mut work = Work::frame(&cancel);
    let mut cursor = unicode_segmentation::GraphemeCursor::new(0, line.len(), true);
    let mut byte = 0;
    loop {
        let end_byte = match text_engine::next(line, &mut cursor, &mut work) {
            Ok(Some(end)) => end,
            Ok(None) => break,
            Err(_) => return "referencia de viewport".into(),
        };
        let grapheme = &line[byte..end_byte];
        byte = end_byte;
        if grapheme.len() > crate::viewport_admission::SYMBOL_BYTES {
            return "referencia de viewport".into();
        }
        if column >= right {
            break;
        }
        let Ok(cells) = text_engine::width(grapheme, column, &mut work) else {
            return "referencia de viewport".into();
        };
        let end = column.saturating_add(cells);
        if end > column && end > left {
            if grapheme == "\t" || column < left || end > right {
                result.extend(std::iter::repeat_n(
                    ' ',
                    end.min(right).saturating_sub(column.max(left)),
                ));
            } else {
                result.push_str(grapheme);
            }
        }
        column = end;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn model(s: &str) -> SafeTextModel {
        SafeTextModel::new(s.into(), &AtomicBool::new(false)).unwrap()
    }
    #[test]
    fn provenance_control_crlf_and_literal_placeholder() {
        let text = model("\x1bXfoo\r\n␛foo\u{009b}\t界e\u{301}");
        assert_eq!(text.visible(), "␛Xfoo\r\n␛foo\\u{009b}\t界e\u{301}");
        assert_eq!(text.source_range(4..7), 2..5);
        assert_eq!(text.visible_range(2..5), 4..7);
        assert_eq!(text.cell_for_offset(4), 2);
        assert_eq!(text.line(0), Some("␛Xfoo"));
        assert_eq!(text.line_count(), 2);
        let start = text.visible().find("009b").unwrap();
        let raw = text.source_range(start..start + 4);
        assert_eq!(&text.original()[raw.clone()], "\u{009b}");
        assert_eq!(&text.visible()[text.visible_range(raw)], "\\u{009b}");
        assert_eq!(text.original(), "\x1bXfoo\r\n␛foo\u{009b}\t界e\u{301}");
    }
    #[test]
    fn sparse_checkpoints_cover_expansion_at_boundary_and_end() {
        let source = format!("{}\u{009b}\x1b🦊end", "á".repeat(4095));
        let text = model(&source);
        let raw = source.find("end").unwrap();
        let safe = text.visible().find("end").unwrap();
        assert_eq!(text.source_range(safe..safe + 3), raw..raw + 3);
        assert_eq!(text.visible_range(raw..raw + 3), safe..safe + 3);
        assert_eq!(
            text.source_range(text.visible.len()..text.visible.len()),
            source.len()..source.len()
        );
        assert_eq!(text.checkpoints.len(), 4);
    }
    #[test]
    fn cells_preserve_graphemes_and_tab_coordinates() {
        assert_eq!(slice_cells("A界B", 2, 2), " B");
        assert_eq!(slice_cells("e\u{301}👩‍💻x", 1, 2), "👩‍💻");
        assert_eq!(slice_cells("a\tb", 6, 3), "  b");
        let text = model("\x1b\tfoo");
        assert_eq!(text.cell_for_offset(text.visible().find("foo").unwrap()), 8);
    }
    #[test]
    fn interior_combining_and_zwj_offsets_point_to_grapheme_start() {
        let text = model("e\u{301}👩‍💻x");
        assert_eq!(text.cell_for_offset(1), 0);
        assert_eq!(text.cell_for_offset(3), 1);
        assert_eq!(text.cell_for_offset(7), 1);
        assert_eq!(text.cell_for_offset(10), 1);
        assert_eq!(text.cell_for_offset(14), 3);
    }
    #[test]
    fn width_and_checkpoints_use_cells_including_tabs_and_unicode() {
        let text = model(&format!(
            "{}end\n{}e\u{301}👩‍💻end",
            "\t".repeat(20),
            "\t".repeat(5000)
        ));
        assert_eq!(text.max_width(), 40006);
        assert_eq!(text.viewport_start(0, 160), Some((0, 0)));
        let (offset, column) = text.viewport_start(1, 39990).unwrap();
        assert!(column <= 39990 && 39990 - column < 8 * CHECKPOINT_BYTES);
        assert!(text.visible().is_char_boundary(offset));
        let target = text.visible().rfind("end").unwrap();
        assert_eq!(text.cell_for_offset(target), 40003);
        assert_eq!(text.line_for_offset(offset), 1);
    }
    #[test]
    fn monotonic_cursor_matches_control_and_unicode_provenance() {
        let text = model(&format!(
            "{}\u{009b}e\u{301}👩‍💻\x1b界\r\nend",
            "á".repeat(5000)
        ));
        for start in [0, 4090, text.visible().find("009b").unwrap()] {
            let mut cursor = text.provenance_cursor(start);
            for (byte, grapheme) in text.visible()[start..].grapheme_indices(true) {
                let range = start + byte..start + byte + grapheme.len();
                assert_eq!(cursor.source_range(range.clone()), text.source_range(range));
            }
        }
    }
    #[test]
    fn empty_and_cancelled() {
        let text = model("");
        assert_eq!(text.line(0), Some(""));
        assert!(matches!(
            SafeTextModel::new("x".into(), &AtomicBool::new(true)),
            Err(PrepareError::Cancelled)
        ));
    }
}
