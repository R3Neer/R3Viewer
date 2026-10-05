//! Safe visible text with sparse provenance checkpoints. Source bytes stay intact.
use std::{
    ops::Range,
    sync::atomic::{AtomicBool, Ordering},
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

const CHECKPOINT_BYTES: usize = 4096;
const MAX_LINES: usize = 1_000_000;
const MAX_VISIBLE_BYTES: usize = 256 * 1024 * 1024;

#[derive(Debug)]
pub enum PrepareError {
    Cancelled,
    Limit(&'static str),
}

#[derive(Debug)]
pub struct SafeTextModel {
    original: std::sync::Arc<String>,
    visible: String,
    // (source byte, visible byte), always at character boundaries.
    checkpoints: Vec<(usize, usize)>,
    lines: Vec<usize>,
    max_width: usize,
    cell_checkpoints: Vec<CellCheckpoint>,
}

#[derive(Debug)]
struct CellCheckpoint {
    line: usize,
    visible: usize,
    column: usize,
}

fn replacement_len(ch: char, next: Option<char>) -> usize {
    match ch {
        '\n' | '\t' => 1,
        '\r' if next == Some('\n') => 1,
        '\u{7f}' | '\u{0}'..='\u{1f}' => 3,
        '\u{80}'..='\u{9f}' => 8,
        _ => ch.len_utf8(),
    }
}
fn append_visible(out: &mut String, ch: char, next: Option<char>) {
    match ch {
        '\n' | '\t' => out.push(ch),
        '\r' if next == Some('\n') => out.push(ch),
        '\u{7f}' => out.push('␡'),
        '\u{0}'..='\u{1f}' => out.push(char::from_u32(0x2400 + ch as u32).unwrap()),
        '\u{80}'..='\u{9f}' => {
            use std::fmt::Write;
            write!(out, "\\u{{{:04x}}}", ch as u32).unwrap();
        }
        _ => out.push(ch),
    }
}

impl SafeTextModel {
    pub fn new(original: String, cancelled: &AtomicBool) -> Result<Self, PrepareError> {
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
        let mut max_width = 0;
        let mut cell_checkpoints = Vec::new();
        let mut visited = 0usize;
        for (line_index, line) in visible.split('\n').enumerate() {
            if cancelled.load(Ordering::Relaxed) {
                return Err(PrepareError::Cancelled);
            }
            let line = line.strip_suffix('\r').unwrap_or(line);
            let mut column = 0usize;
            let mut next = CHECKPOINT_BYTES;
            for (byte, grapheme) in line.grapheme_indices(true) {
                if visited.is_multiple_of(4096) && cancelled.load(Ordering::Relaxed) {
                    return Err(PrepareError::Cancelled);
                }
                visited += 1;
                if byte >= next {
                    cell_checkpoints.push(CellCheckpoint {
                        line: line_index,
                        visible: lines[line_index] + byte,
                        column,
                    });
                    next = byte.saturating_add(CHECKPOINT_BYTES);
                }
                column = column.saturating_add(cell_width(grapheme, column));
            }
            max_width = max_width.max(column);
        }
        if cancelled.load(Ordering::Relaxed) {
            return Err(PrepareError::Cancelled);
        }
        Ok(Self {
            original: std::sync::Arc::new(original),
            visible,
            checkpoints,
            lines,
            max_width,
            cell_checkpoints,
        })
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
        let start = *self.lines.get(line)?;
        let index = self
            .cell_checkpoints
            .partition_point(|cp| cp.line < line || cp.line == line && cp.column <= left);
        if let Some(cp) = index
            .checked_sub(1)
            .and_then(|i| self.cell_checkpoints.get(i))
            .filter(|cp| cp.line == line)
        {
            Some((cp.visible, cp.column))
        } else {
            Some((start, 0))
        }
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
    fn source_boundary(&self, visible_offset: usize, end: bool) -> usize {
        let target = visible_offset.min(self.visible.len());
        let checkpoint = self
            .checkpoints
            .partition_point(|(_, safe)| *safe <= target)
            .saturating_sub(1);
        let (raw, mut safe) = self.checkpoints[checkpoint];
        let mut chars = self.original[raw..].char_indices().peekable();
        while let Some((relative, ch)) = chars.next() {
            let offset = raw + relative;
            if safe == target {
                return offset;
            }
            let len = replacement_len(ch, chars.peek().map(|(_, ch)| *ch));
            if safe + len > target {
                return if end { offset + ch.len_utf8() } else { offset };
            }
            safe += len;
        }
        self.original.len()
    }
    /// A partial match in a control placeholder maps to its complete source char.
    pub fn source_range(&self, visible: Range<usize>) -> Range<usize> {
        self.source_boundary(visible.start, false)..self.source_boundary(visible.end, true)
    }
    fn visible_boundary(&self, source_offset: usize, end: bool) -> usize {
        let target = source_offset.min(self.original.len());
        let checkpoint = self
            .checkpoints
            .partition_point(|(raw, _)| *raw <= target)
            .saturating_sub(1);
        let (raw, mut safe) = self.checkpoints[checkpoint];
        let mut chars = self.original[raw..].char_indices().peekable();
        while let Some((relative, ch)) = chars.next() {
            let offset = raw + relative;
            if offset == target {
                return safe;
            }
            let len = replacement_len(ch, chars.peek().map(|(_, ch)| *ch));
            if offset + ch.len_utf8() > target {
                return if end { safe + len } else { safe };
            }
            safe += len;
        }
        self.visible.len()
    }
    pub fn visible_range(&self, source: Range<usize>) -> Range<usize> {
        self.visible_boundary(source.start, false)..self.visible_boundary(source.end, true)
    }
    pub fn cell_for_offset(&self, offset: usize) -> usize {
        let line = self.line_for_offset(offset);
        let range = self.line_range(line).unwrap();
        let target = offset.min(range.end);
        let index = self
            .cell_checkpoints
            .partition_point(|cp| cp.visible <= target);
        let (start, mut col) = index
            .checked_sub(1)
            .and_then(|i| self.cell_checkpoints.get(i))
            .filter(|cp| cp.line == line)
            .map_or((range.start, 0), |cp| (cp.visible, cp.column));
        for (byte, grapheme) in self.visible[start..range.end].grapheme_indices(true) {
            if target < start + byte + grapheme.len() {
                break;
            }
            col += cell_width(grapheme, col);
        }
        col
    }
}

pub fn cell_width(grapheme: &str, column: usize) -> usize {
    if grapheme == "\t" {
        8 - column % 8
    } else {
        UnicodeWidthStr::width(grapheme)
    }
}

/// Whole graphemes only; wide fragments become blank cells, tabs retain origin.
pub fn slice_cells(line: &str, left: usize, width: usize) -> String {
    let right = left.saturating_add(width);
    let mut column: usize = 0;
    let mut result = String::new();
    for grapheme in line.graphemes(true) {
        if column >= right {
            break;
        }
        let end = column.saturating_add(cell_width(grapheme, column));
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
    fn empty_and_cancelled() {
        let text = model("");
        assert_eq!(text.line(0), Some(""));
        assert!(matches!(
            SafeTextModel::new("x".into(), &AtomicBool::new(true)),
            Err(PrepareError::Cancelled)
        ));
    }
}
