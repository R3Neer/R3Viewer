//! Shared cancelable Unicode mechanics. Never hand unadmitted clusters to widgets.
use std::{
    ops::Range,
    sync::atomic::{AtomicBool, Ordering},
};
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Error {
    Cancelled,
    Work,
    Invalid,
}
#[derive(Default, Debug, Clone)]
pub(crate) struct Counters {
    pub supplied: usize,
    pub context: usize,
    pub advanced: usize,
    pub width_reserved: usize,
    pub projection: usize,
}
pub(crate) struct Work<'a> {
    pub cancel: &'a AtomicBool,
    pub counters: Counters,
    limit: usize,
    used: usize,
}
impl<'a> Work<'a> {
    pub fn preparation(cancel: &'a AtomicBool) -> Self {
        Self {
            cancel,
            counters: Counters::default(),
            limit: usize::MAX,
            used: 0,
        }
    }
    pub fn frame(cancel: &'a AtomicBool) -> Self {
        Self {
            limit: 256 * 1024,
            ..Self::preparation(cancel)
        }
    }
    pub fn admit(&mut self, n: usize) -> Result<(), Error> {
        if self.cancel.load(Ordering::Relaxed) {
            return Err(Error::Cancelled);
        }
        let total = self.used.checked_add(n).ok_or(Error::Work)?;
        if total > self.limit {
            return Err(Error::Work);
        }
        self.used = total;
        Ok(())
    }
    pub fn used(&self) -> usize {
        self.used
    }
    pub fn permit(cancel: &'a AtomicBool, limit: usize) -> Self {
        Self {
            limit,
            ..Self::preparation(cancel)
        }
    }
    pub fn project(&mut self, n: usize) -> Result<(), Error> {
        self.admit(n)?;
        self.counters.projection += n;
        Ok(())
    }
}
/// Cursor is retained only at resolved boundaries; callers must keep its original text identity.
pub(crate) fn next(
    s: &str,
    cursor: &mut GraphemeCursor,
    w: &mut Work<'_>,
) -> Result<Option<usize>, Error> {
    let mut start = cursor.cur_cursor();
    loop {
        let left = s[..start].char_indices().next_back().map_or(0, |(i, _)| i);
        let mut end =
            s.len()
                .min(start.saturating_add(if w.limit == usize::MAX { 4092 } else { 4 }));
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        w.admit(end - left)?;
        w.counters.supplied += end - left;
        let before = cursor.cur_cursor();
        let result = cursor.next_boundary(&s[left..end], left);
        w.counters.advanced += cursor.cur_cursor().saturating_sub(before);
        w.admit(0)?;
        match result {
            Ok(v) => return Ok(v),
            Err(GraphemeIncomplete::NextChunk) => start = end,
            Err(GraphemeIncomplete::PreContext(n)) => {
                let mut left = n.saturating_sub(4096);
                while !s.is_char_boundary(left) {
                    left += 1;
                }
                w.admit(n - left)?;
                w.counters.context += n - left;
                cursor.provide_context(&s[left..n], left);
                w.admit(0)?;
            }
            _ => return Err(Error::Invalid),
        }
    }
}
pub(crate) fn width(s: &str, column: usize, w: &mut Work<'_>) -> Result<usize, Error> {
    if s == "\t" {
        w.admit(1)?;
        return Ok(8 - column % 8);
    }
    let mut failure = None;
    let result = unicode_width_adapter::width_cancelled(s, |n| {
        if let Err(e) = w.admit(n) {
            failure = Some(e);
            false
        } else {
            w.counters.width_reserved += n;
            true
        }
    });
    result.ok_or(failure.unwrap_or(Error::Cancelled))
}
pub(crate) fn control_in(s: &str, w: &mut Work<'_>) -> Result<bool, Error> {
    let mut at = 0;
    while at < s.len() {
        let mut end = s.len().min(at + 4096);
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        w.project(end - at)?;
        if s[at..end].chars().any(char::is_control) {
            return Ok(true);
        }
        at = end;
    }
    w.admit(0)?;
    Ok(false)
}
/// A borrowed, complete cluster; range is in its immutable domain, not decoded-source offsets.
#[derive(Debug, Clone)]
pub(crate) struct Unit {
    pub visible: Range<usize>,
    pub ascii: bool,
    pub safe_control: bool,
    pub range: Range<usize>,
    pub line: usize,
    pub column: usize,
    pub width: usize,
    pub newline: bool,
}
#[cfg(test)]
mod tests {
    use super::*;
    use unicode_width::UnicodeWidthStr;
    #[test]
    fn stateful_reverse_matches_original() {
        let fixtures = [
            "",
            "a",
            "e\u{301}",
            "👩🏽‍💻",
            "🇦🇧🇨",
            "\r\n",
            "لا",
            "가",
            "a\u{9b}x",
            "👩\u{301}‍💻",
        ];
        for s in fixtures {
            for chunk in [4, 5, 7, 32, 4096] {
                let mut state = unicode_width_adapter::WidthState::new();
                let mut end = s.len();
                while end > 0 {
                    let mut left = end.saturating_sub(chunk);
                    while !s.is_char_boundary(left) {
                        left += 1;
                    }
                    assert!(state.push_reverse_block(&s[left..end], |_| true));
                    end = left;
                }
                assert_eq!(state.finish(), s.width(), "{s:?} chunk{chunk}");
            }
        }
        let s = format!("👩{}‍💻{}", "\u{301}".repeat(8192), "🇦".repeat(8193));
        let cancel = AtomicBool::new(false);
        let mut w = Work::preparation(&cancel);
        assert_eq!(width(&s, 0, &mut w).unwrap(), s.width());
    }
    #[test]
    fn width_rejects_before_processing() {
        let cancel = AtomicBool::new(false);
        let mut w = Work::frame(&cancel);
        w.admit(256 * 1024 - 1).unwrap();
        assert_eq!(width(&"a".repeat(4096), 0, &mut w), Err(Error::Work));
        assert_eq!(w.counters.width_reserved, 0);
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Identity(std::sync::Arc<()>);
impl Identity {
    pub fn new() -> Self {
        Self(std::sync::Arc::new(()))
    }
    pub(crate) fn same(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.0, &other.0)
    }
}
#[derive(Clone, Debug)]
pub(crate) struct Point {
    identity: Identity,
    base: usize,
    end: usize,
    offset: usize,
    visible: usize,
    line: usize,
    column: usize,
    cursor: GraphemeCursor,
}
impl Point {
    pub fn offset(&self) -> usize {
        self.offset
    }
    pub fn line(&self) -> usize {
        self.line
    }
    pub fn column(&self) -> usize {
        self.column
    }
}
#[derive(Clone, Debug)]
struct Exception {
    unit: Unit,
    after: Point,
}
/// Sparse prepared index over one immutable piece. Identity comes from its owner.
#[derive(Debug)]
pub(crate) struct Index {
    identity: Identity,
    len: usize,
    projected: bool,
    last_column: usize,
    visible_bytes: usize,
    points: Vec<Point>,
    exceptions: Vec<Exception>,
    pub max_width: usize,
    pub lines: usize,
    pub allocated: usize,
    pub growth_peak: usize,
}
fn push_charged<T>(
    v: &mut Vec<T>,
    value: T,
    used: &mut usize,
    peak: &mut usize,
    limit: usize,
) -> Result<(), Error> {
    if v.len() == v.capacity() {
        let old = v
            .capacity()
            .checked_mul(std::mem::size_of::<T>())
            .ok_or(Error::Work)?;
        let requested = v.capacity().max(4).checked_mul(2).ok_or(Error::Work)?;
        let bytes = requested
            .checked_mul(std::mem::size_of::<T>())
            .ok_or(Error::Work)?;
        let admission = used.checked_add(bytes).ok_or(Error::Work)?;
        if admission > limit {
            return Err(Error::Work);
        }
        *peak = (*peak).max(admission);
        v.try_reserve_exact(requested - v.len())
            .map_err(|_| Error::Work)?;
        let actual = v
            .capacity()
            .checked_mul(std::mem::size_of::<T>())
            .ok_or(Error::Work)?;
        if actual > bytes {
            return Err(Error::Work);
        }
        *used = used
            .checked_sub(old)
            .and_then(|n| n.checked_add(actual))
            .ok_or(Error::Work)?;
    }
    v.push(value);
    Ok(())
}
impl Index {
    pub fn prepare(
        text: &str,
        identity: Identity,
        w: &mut Work<'_>,
        stride: usize,
        allocation_limit: usize,
    ) -> Result<Self, Error> {
        Self::prepare_inner(text, identity, w, stride, allocation_limit, 0, false)
    }
    pub fn prepare_projected(
        text: &str,
        identity: Identity,
        w: &mut Work<'_>,
        stride: usize,
        allocation_limit: usize,
        initial_column: usize,
    ) -> Result<Self, Error> {
        Self::prepare_inner(
            text,
            identity,
            w,
            stride,
            allocation_limit,
            initial_column,
            true,
        )
    }
    fn prepare_inner(
        text: &str,
        identity: Identity,
        w: &mut Work<'_>,
        stride: usize,
        allocation_limit: usize,
        initial_column: usize,
        projected: bool,
    ) -> Result<Self, Error> {
        let mut index = Self {
            identity,
            len: text.len(),
            projected,
            last_column: initial_column,
            visible_bytes: 0,
            points: Vec::new(),
            exceptions: Vec::new(),
            max_width: 0,
            lines: 1,
            allocated: 0,
            growth_peak: 0,
        };
        let mut at = Point {
            identity: index.identity.clone(),
            base: 0,
            end: text.len(),
            offset: 0,
            visible: 0,
            line: 0,
            column: initial_column,
            cursor: GraphemeCursor::new(0, text.len(), true),
        };
        push_charged(
            &mut index.points,
            at.clone(),
            &mut index.allocated,
            &mut index.growth_peak,
            allocation_limit,
        )?;
        let mut checkpoint = stride.max(4096);
        while at.offset < text.len() {
            let before = at.clone();
            let end =
                at.base + next(&text[at.base..at.end], &mut at.cursor, w)?.ok_or(Error::Invalid)?;
            let raw = &text[before.offset..end];
            let newline = raw == "\n" || raw == "\r\n";
            let (visible_len, cells, _) = if projected {
                projected_metrics(raw, at.column, w)?
            } else {
                (
                    raw.len(),
                    if newline {
                        0
                    } else {
                        width(raw, at.column, w)?
                    },
                    newline,
                )
            };
            let safe_control = projected && is_projected_control(raw);
            let ascii = if safe_control {
                raw.len() == 2
            } else {
                raw == "\r\n" || raw.len() == 1 && raw.as_bytes()[0].is_ascii()
            };
            let unit = Unit {
                visible: at.visible..at.visible + visible_len,
                ascii,
                safe_control,
                range: before.offset..end,
                line: at.line,
                column: at.column,
                width: cells,
                newline,
            };
            at.offset = end;
            at.visible += visible_len;
            if newline {
                index.max_width = index.max_width.max(at.column);
                at.line += 1;
                at.column = 0;
            } else {
                at.column = at.column.saturating_add(cells);
            }
            if visible_len > 16 * 1024 {
                if index
                    .points
                    .last()
                    .is_none_or(|p| p.offset != before.offset)
                {
                    push_charged(
                        &mut index.points,
                        before,
                        &mut index.allocated,
                        &mut index.growth_peak,
                        allocation_limit,
                    )?;
                }
                push_charged(
                    &mut index.exceptions,
                    Exception {
                        unit,
                        after: at.clone(),
                    },
                    &mut index.allocated,
                    &mut index.growth_peak,
                    allocation_limit,
                )?;
                push_charged(
                    &mut index.points,
                    at.clone(),
                    &mut index.allocated,
                    &mut index.growth_peak,
                    allocation_limit,
                )?;
                checkpoint = end.saturating_add(stride.max(4096));
            } else if end >= checkpoint {
                push_charged(
                    &mut index.points,
                    at.clone(),
                    &mut index.allocated,
                    &mut index.growth_peak,
                    allocation_limit,
                )?;
                checkpoint = end.saturating_add(stride.max(4096));
            }
        }
        index.max_width = index.max_width.max(at.column);
        index.lines = at.line + 1;
        index.last_column = at.column;
        index.visible_bytes = at.visible;
        w.admit(0)?;
        Ok(index)
    }
    fn validate(&self, text: &str, identity: &Identity) -> Result<(), Error> {
        if self.len != text.len() || !self.identity.same(identity) {
            Err(Error::Invalid)
        } else {
            Ok(())
        }
    }
    pub fn start_offset(
        &self,
        text: &str,
        identity: &Identity,
        offset: usize,
    ) -> Result<Point, Error> {
        self.validate(text, identity)?;
        let n = self
            .points
            .partition_point(|p| p.offset <= offset.min(self.len));
        Ok(self.points[n.saturating_sub(1)].clone())
    }
    pub fn line_entry(
        &self,
        text: &str,
        identity: &Identity,
        range: Range<usize>,
        line: usize,
    ) -> Result<Point, Error> {
        self.validate(text, identity)?;
        if range.start > range.end
            || range.end > text.len()
            || !text.is_char_boundary(range.start)
            || !text.is_char_boundary(range.end)
        {
            return Err(Error::Invalid);
        }
        Ok(Point {
            identity: self.identity.clone(),
            base: range.start,
            end: range.end,
            offset: range.start,
            visible: range.start,
            line,
            column: 0,
            cursor: GraphemeCursor::new(0, range.len(), true),
        })
    }
    pub fn start_cell(
        &self,
        text: &str,
        identity: &Identity,
        line: usize,
        column: usize,
    ) -> Result<Point, Error> {
        self.validate(text, identity)?;
        let zero = self.exceptions.partition_point(|e| {
            e.unit.line < line || e.unit.line == line && e.unit.column < column
        });
        if let Some(e) = self
            .exceptions
            .get(zero)
            .filter(|e| e.unit.line == line && e.unit.column == column && e.unit.width == 0)
        {
            return self.start_offset(text, identity, e.unit.range.start);
        }
        let n = self
            .points
            .partition_point(|p| p.line < line || p.line == line && p.column <= column);
        Ok(self.points[n.saturating_sub(1)].clone())
    }
    pub fn step(
        &self,
        text: &str,
        identity: &Identity,
        at: &mut Point,
        w: &mut Work<'_>,
    ) -> Result<Option<Unit>, Error> {
        self.validate(text, identity)?;
        if !self.identity.same(&at.identity)
            || at.base > at.offset
            || at.offset > at.end
            || at.end > self.len
            || at.cursor.cur_cursor() != at.offset - at.base
        {
            return Err(Error::Invalid);
        }
        w.admit(0)?;
        if at.offset >= at.end {
            return Ok(None);
        }
        let n = self
            .exceptions
            .partition_point(|e| e.unit.range.start < at.offset);
        if let Some(e) = self
            .exceptions
            .get(n)
            .filter(|e| e.unit.range.start == at.offset)
        {
            w.admit(std::mem::size_of::<Exception>())?;
            *at = e.after.clone();
            return Ok(Some(e.unit.clone()));
        }
        let start = at.offset;
        let end =
            at.base + next(&text[at.base..at.end], &mut at.cursor, w)?.ok_or(Error::Invalid)?;
        let raw = &text[start..end];
        let newline = raw == "\n" || raw == "\r\n";
        let (visible_len, cells, _) = if self.projected {
            projected_metrics(raw, at.column, w)?
        } else {
            (
                raw.len(),
                if newline {
                    0
                } else {
                    width(raw, at.column, w)?
                },
                newline,
            )
        };
        let safe_control = self.projected && is_projected_control(raw);
        let ascii = if safe_control {
            raw.len() == 2
        } else {
            raw == "\r\n" || raw.len() == 1 && raw.as_bytes()[0].is_ascii()
        };
        let unit = Unit {
            visible: at.visible..at.visible + visible_len,
            ascii,
            safe_control,
            range: start..end,
            line: at.line,
            column: at.column,
            width: cells,
            newline,
        };
        at.offset = end;
        at.visible += visible_len;
        if newline {
            at.line += 1;
            at.column = 0;
        } else {
            at.column = at.column.saturating_add(cells);
        }
        Ok(Some(unit))
    }
    pub fn snapshot_capacity(&self) -> usize {
        self.points.capacity() + self.exceptions.capacity()
    }
    #[cfg(test)]
    pub fn snapshot_count(&self) -> usize {
        self.points.len() + self.exceptions.len()
    }
    pub fn end_metrics(&self) -> (usize, usize, usize) {
        (self.lines - 1, self.last_column, self.visible_bytes)
    }
    pub fn start_visible(
        &self,
        text: &str,
        identity: &Identity,
        offset: usize,
    ) -> Result<Point, Error> {
        self.validate(text, identity)?;
        let n = self.points.partition_point(|p| p.visible <= offset);
        Ok(self.points[n.saturating_sub(1)].clone())
    }
    pub fn position(
        &self,
        text: &str,
        identity: &Identity,
        offset: usize,
        w: &mut Work<'_>,
    ) -> Result<(usize, usize), Error> {
        let target = offset.min(self.len);
        let mut at = self.start_offset(text, identity, target)?;
        while let Some(unit) = self.step(text, identity, &mut at, w)? {
            if target < unit.range.end {
                return Ok((unit.line, unit.column));
            }
            if target == unit.range.end {
                return Ok((at.line, at.column));
            }
        }
        Ok((at.line, at.column))
    }
}

/// Exact safe-control projection metrics without constructing the projected leaf.
pub(crate) fn projected_metrics(
    raw: &str,
    column: usize,
    w: &mut Work<'_>,
) -> Result<(usize, usize, bool), Error> {
    if raw == "\n" || raw == "\r\n" {
        w.project(raw.len())?;
        return Ok((raw.len(), 0, true));
    }
    if raw == "\t" {
        w.project(1)?;
        return Ok((1, 8 - column % 8, false));
    }
    if !control_in(raw, w)? {
        return Ok((raw.len(), width(raw, column, w)?, false));
    }
    let mut state = unicode_width_adapter::WidthState::new();
    let mut visible = 0usize;
    let mut end = raw.len();
    while end > 0 {
        let mut left = end.saturating_sub(4096);
        while !raw.is_char_boundary(left) {
            left += 1;
        }
        w.project(end - left)?;
        for (relative, ch) in raw[left..end].char_indices().rev() {
            let next = left + relative + ch.len_utf8();
            let mut bytes = [0u8; 8];
            let shown: &str = match ch {
                '\u{7f}' => "␡",
                '\0'..='\u{1f}' if ch != '\r' || !raw[next..].starts_with('\n') => {
                    char::from_u32(0x2400 + ch as u32)
                        .ok_or(Error::Invalid)?
                        .encode_utf8(&mut bytes)
                }
                '\u{80}'..='\u{9f}' => {
                    const HEX: &[u8; 16] = b"0123456789abcdef";
                    bytes.copy_from_slice(b"\\u{0000}");
                    bytes[5] = HEX[(ch as usize >> 4) & 15];
                    bytes[6] = HEX[ch as usize & 15];
                    std::str::from_utf8(&bytes).map_err(|_| Error::Invalid)?
                }
                _ => ch.encode_utf8(&mut bytes),
            };
            w.project(shown.len())?;
            visible = visible.checked_add(shown.len()).ok_or(Error::Work)?;
            let mut failure = None;
            if !state.push_reverse_block(shown, |n| match w.admit(n) {
                Ok(()) => {
                    w.counters.width_reserved += n;
                    true
                }
                Err(e) => {
                    failure = Some(e);
                    false
                }
            }) {
                return Err(failure.unwrap_or(Error::Cancelled));
            }
        }
        end = left;
    }
    Ok((visible, state.finish(), false))
}

pub(crate) fn is_projected_control(raw: &str) -> bool {
    let b = raw.as_bytes();
    (b.len() == 1 && (b[0] < 32 || b[0] == 127) && raw != "\t" && raw != "\n")
        || (b.len() == 2 && b[0] == 0xc2 && (0x80..=0x9f).contains(&b[1]))
}
#[cfg(test)]
mod integration_mechanics_tests {
    use super::*;
    use unicode_width::UnicodeWidthStr;
    #[test]
    fn projected_cluster_metrics_match_existing_safe_spelling() {
        let cancel = AtomicBool::new(false);
        for text in [
            "\x1b",
            "\u{009b}",
            "\u{7f}",
            "\r",
            "e\u{301}",
            "👩🏽‍💻",
            "\u{0301}",
        ] {
            let safe = crate::text::safe_text(text);
            let mut w = Work::preparation(&cancel);
            let (len, cells, newline) = projected_metrics(text, 0, &mut w).unwrap();
            assert_eq!(len, safe.len());
            assert_eq!(cells, safe.width());
            assert!(!newline);
        }
        let mut w = Work::preparation(&cancel);
        assert_eq!(projected_metrics("\t", 7, &mut w).unwrap(), (1, 1, false));
        assert_eq!(projected_metrics("\r\n", 7, &mut w).unwrap(), (2, 0, true));
    }
    #[test]
    fn rejects_foreign_checkpoint_and_finishes_line_entry() {
        let cancel = AtomicBool::new(false);
        let ia = Identity::new();
        let ib = Identity::new();
        let mut w = Work::preparation(&cancel);
        let a = Index::prepare("abc\ndef", ia.clone(), &mut w, 4096, usize::MAX).unwrap();
        let b = Index::prepare("xyz\nuvw", ib.clone(), &mut w, 4096, usize::MAX).unwrap();
        let mut foreign = b.start_offset("xyz\nuvw", &ib, 0).unwrap();
        assert!(matches!(
            a.step("abc\ndef", &ia, &mut foreign, &mut w),
            Err(Error::Invalid)
        ));
        let mut line = a.line_entry("abc\ndef", &ia, 0..3, 0).unwrap();
        while a
            .step("abc\ndef", &ia, &mut line, &mut w)
            .unwrap()
            .is_some()
        {}
        assert_eq!(line.offset(), 3);
        assert!(
            a.step("abc\ndef", &ia, &mut line, &mut w)
                .unwrap()
                .is_none()
        );
    }
}

#[cfg(test)]
mod unicode_query_context_tests {
    use super::*;
    use unicode_segmentation::UnicodeSegmentation;
    #[test]
    fn four_byte_query_resume_matches_upstream_mixed_ri_context() {
        let cancel = AtomicBool::new(false);
        for base in ["x🇦🇧🇨z", "👩🏽‍💻🇦🇧🇨x", "ée\u{301}👩🏽‍💻🇦🇧🇨x", "क्🇦🇧🇨x"]
        {
            for prefix in (0..66).chain(4080..4110) {
                let text = format!("{}{base}", "x".repeat(prefix));
                let expected: Vec<_> = text
                    .grapheme_indices(true)
                    .map(|(n, s)| n..n + s.len())
                    .collect();
                let id = Identity::new();
                let mut prep = Work::preparation(&cancel);
                let index = Index::prepare(&text, id.clone(), &mut prep, 4096, usize::MAX).unwrap();
                let mut at = index.start_offset(&text, &id, 0).unwrap();
                let mut query = Work::frame(&cancel);
                let mut got = Vec::new();
                while let Some(unit) = index.step(&text, &id, &mut at, &mut query).unwrap() {
                    got.push(unit.range);
                }
                assert_eq!(got, expected, "prefix={prefix} {base}");
            }
        }
    }
}

#[cfg(test)]
mod unicode_sparse_seek_probe {
    use super::*;
    #[test]
    fn maximum_borrowed_profile_stride_preserves_ascii_seek() {
        let cancel = AtomicBool::new(false);
        let text = "a".repeat(131072);
        let identity = Identity::new();
        let index = Index::prepare_projected(
            &text,
            identity.clone(),
            &mut Work::preparation(&cancel),
            (128usize * 1024 * 1024).div_ceil(4095),
            1024 * 1024,
            0,
        )
        .unwrap();
        let mut work = Work::frame(&cancel);
        let result = index.position(&text, &identity, 131070, &mut work);
        eprintln!(
            "sparse seek result={result:?} work={} Point={} index_alloc={} growth={}",
            work.used(),
            std::mem::size_of::<Point>(),
            index.allocated,
            index.growth_peak
        );
        assert_eq!(result, Ok((0, 131070)));
    }
}

#[cfg(test)]
mod unicode_projection_seek_probe {
    use super::*;
    #[test]
    fn maximum_borrowed_profile_projection_seek() {
        let cancel = AtomicBool::new(false);
        for scalar in ['\u{1b}', '\u{9b}', 'é', '\u{1f1e6}'] {
            let text = scalar.to_string().repeat(65536);
            let identity = Identity::new();
            let index = Index::prepare_projected(
                &text,
                identity.clone(),
                &mut Work::preparation(&cancel),
                (128usize * 1024 * 1024).div_ceil(4095),
                1024 * 1024,
                0,
            )
            .unwrap();
            let target = 32000 / scalar.len_utf8() * scalar.len_utf8();
            let mut work = Work::frame(&cancel);
            let result = index.position(&text, &identity, target, &mut work);
            eprintln!(
                "seek scalar={scalar:?} result={result:?} work={}",
                work.used()
            );
            if scalar == '\u{1b}' || scalar == '\u{9b}' {
                assert_eq!(result, Err(Error::Work));
            } else {
                assert!(result.is_ok());
            }
        }
    }
}
