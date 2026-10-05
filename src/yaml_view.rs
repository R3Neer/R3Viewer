//! YAML serialization navigation and borrowed, bounded presentation.
//! Alias edges are followed only by an explicit action, never by traversal.
use crate::{
    data::Cursor,
    structure_model::{NodeId, NodeRef, ParentPart, Structure, StructureKind, YamlStyle},
};
use std::{
    fmt::{self, Write},
    mem::size_of,
    ops::Range,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete};
pub const HISTORY_LIMIT: usize = 128;
pub const SESSION_BYTES: usize = 1024 * 1024;
pub const WINDOW_BYTES: usize = 128 * 1024;
const CHECKPOINTS: usize = 4096;
const CLUSTER_BYTES: usize = 16 * 1024;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Cancelled,
    Budget,
    History,
    Invalid,
    Window,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Cancelled => "cancelado",
            Self::Budget => "reserva YAML de sesión agotada",
            Self::History => "historial YAML: máximo 128 entradas previas; vuelve antes de navegar",
            Self::Invalid => "dirección YAML no válida",
            Self::Window => "ventana YAML supera su límite; el texto completo sigue en Fuente",
        })
    }
}
pub(crate) fn check_search(cancel: &AtomicBool) -> Result<(), Error> {
    check(cancel)
}
fn check(cancel: &AtomicBool) -> Result<(), Error> {
    if cancel.load(Ordering::Relaxed) {
        Err(Error::Cancelled)
    } else {
        Ok(())
    }
}
/// The model already charged this one shared reservation. UI/search consume it;
/// each allocation obtains a lease before reserve, and releases after drop.
#[derive(Clone)]
pub struct Budget {
    used: Arc<AtomicUsize>,
    limit: usize,
}
pub struct Charge {
    used: Arc<AtomicUsize>,
    bytes: usize,
}
impl Budget {
    pub fn new(structure: &Structure) -> Result<Self, Error> {
        if structure.session_budget_bytes() < SESSION_BYTES {
            return Err(Error::Budget);
        }
        Ok(Self {
            used: Arc::new(AtomicUsize::new(0)),
            limit: SESSION_BYTES,
        })
    }
    pub fn charge(&self, bytes: usize) -> Result<Charge, Error> {
        let mut old = self.used.load(Ordering::Relaxed);
        loop {
            let new = old
                .checked_add(bytes)
                .filter(|n| *n <= self.limit)
                .ok_or(Error::Budget)?;
            match self
                .used
                .compare_exchange_weak(old, new, Ordering::AcqRel, Ordering::Relaxed)
            {
                Ok(_) => {
                    return Ok(Charge {
                        used: self.used.clone(),
                        bytes,
                    });
                }
                Err(n) => old = n,
            }
        }
    }
    pub fn used(&self) -> usize {
        self.used.load(Ordering::Relaxed)
    }
}
impl Drop for Charge {
    fn drop(&mut self) {
        self.used.fetch_sub(self.bytes, Ordering::AcqRel);
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Domain {
    #[default]
    Decoded,
    Metadata,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HitRange {
    pub start: usize,
    pub end: usize,
}
impl HitRange {
    pub fn range(self) -> Range<usize> {
        self.start..self.end
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub node: NodeId,
    pub cursor: Cursor,
    pub domain: Domain,
    pub hit: Option<HitRange>,
    pub last_hit: Option<crate::search::YamlSearchHit>,
}
pub struct Navigator {
    structure: Arc<Structure>,
    current: State,
    previous: [Option<State>; HISTORY_LIMIT],
    count: usize,
    budget: Budget,
    _charge: Charge,
}
impl Navigator {
    pub fn new(structure: Arc<Structure>) -> Result<Self, Error> {
        let budget = Budget::new(&structure)?;
        let charge = budget.charge(size_of::<Self>() + 32768)?;
        let node = structure.root().id();
        Ok(Self {
            structure,
            current: State {
                node,
                cursor: Cursor::default(),
                domain: Domain::Decoded,
                hit: None,
                last_hit: None,
            },
            previous: [None; HISTORY_LIMIT],
            count: 0,
            budget,
            _charge: charge,
        })
    }
    pub fn structure(&self) -> &Arc<Structure> {
        &self.structure
    }
    pub fn budget(&self) -> &Budget {
        &self.budget
    }
    pub fn current(&self) -> State {
        self.current
    }
    pub fn current_mut(&mut self) -> &mut State {
        &mut self.current
    }
    pub fn node(&self) -> NodeRef<'_> {
        self.structure
            .node(self.current.node)
            .expect("validated node")
    }
    pub fn history_len(&self) -> usize {
        self.count
    }
    pub fn can_navigate(&self) -> Result<(), Error> {
        if self.count == HISTORY_LIMIT {
            Err(Error::History)
        } else {
            Ok(())
        }
    }
    pub fn commit(&mut self, state: State, cancel: &AtomicBool) -> Result<(), Error> {
        self.can_navigate()?;
        check(cancel)?;
        if self.structure.node(state.node).is_none() {
            return Err(Error::Invalid);
        }
        self.previous[self.count] = Some(self.current);
        self.count += 1;
        self.current = state;
        Ok(())
    }
    pub fn child(&self, row: usize, column: usize) -> Option<NodeId> {
        let entry = self.node().entry(row)?;
        if column == 0 {
            entry.key().or(Some(entry.value())).map(|n| n.id())
        } else {
            Some(entry.value().id())
        }
    }
    pub fn following_alias(&self) -> Option<NodeId> {
        self.node().metadata()?.alias.map(|alias| alias.target)
    }
    pub fn return_state(&self) -> Option<State> {
        self.count.checked_sub(1).and_then(|i| self.previous[i])
    }
    pub fn back(&mut self) -> bool {
        if self.count == 0 {
            return false;
        }
        self.count -= 1;
        self.current = self.previous[self.count].take().expect("return state");
        true
    }
    pub fn selected_location(&self) -> Option<crate::structure_model::SourceSpan> {
        let node = self.node();
        if self.current.domain == Domain::Metadata
            || !node.kind().is_container()
            || node.entries().len() == 0
        {
            return node.source_location();
        }
        self.child(self.current.cursor.row, self.current.cursor.column)
            .and_then(|id| self.structure.node(id)?.source_location())
    }
    pub fn label(&self) -> Small {
        let mut edges = [None; 66];
        let mut n = self.current.node;
        let mut len = 0;
        while let Some(link) = self.structure.parent(n) {
            if len == edges.len() {
                break;
            }
            edges[len] = Some(link);
            len += 1;
            n = link.owner
        }
        let mut out = Small::new();
        let _ = write!(out, "$");
        for link in edges[..len].iter().rev().flatten() {
            let _ = write!(
                out,
                "/{}{}",
                link.ordinal,
                if link.part == ParentPart::Key {
                    "K"
                } else {
                    "V"
                }
            );
        }
        out
    }
    pub fn frame(&mut self, rows: usize, columns: usize) {
        let count = self.node().entries().len();
        let c = &mut self.current.cursor;
        c.row = c.row.min(count.saturating_sub(1));
        c.column = c.column.min(1);
        let rows = rows.max(1);
        if c.row < c.top {
            c.top = c.row
        }
        if c.row >= c.top.saturating_add(rows) {
            c.top = c.row.saturating_sub(rows - 1)
        }
        let cols = if columns <= 1 { 1 } else { 2 };
        if c.column < c.left {
            c.left = c.column
        }
        if c.column >= c.left + cols {
            c.left = c.column + 1 - cols
        }
    }
}
#[derive(Clone, Copy)]
pub struct Small {
    bytes: [u8; 128],
    len: usize,
}
impl Small {
    fn new() -> Self {
        Self {
            bytes: [0; 128],
            len: 0,
        }
    }
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes[..self.len]).expect("UTF8 small")
    }
}
impl Write for Small {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let end = self
            .len
            .checked_add(s.len())
            .filter(|n| *n <= self.bytes.len())
            .ok_or(fmt::Error)?;
        self.bytes[self.len..end].copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}
#[derive(Clone, Copy)]
enum Piece<'a> {
    Text(&'a str),
    Small(Small),
}
impl Piece<'_> {
    fn text(&self) -> &str {
        match self {
            Self::Text(s) => s,
            Self::Small(s) => s.as_str(),
        }
    }
}
pub struct Text<'a> {
    pieces: [Piece<'a>; 32],
    count: usize,
    searchable: [Option<HitRange>; 2],
}
impl<'a> Text<'a> {
    fn empty() -> Self {
        Self {
            pieces: [Piece::Text(""); 32],
            count: 0,
            searchable: [None; 2],
        }
    }
    fn add(&mut self, piece: Piece<'a>) {
        assert!(self.count < self.pieces.len());
        self.pieces[self.count] = piece;
        self.count += 1;
    }
    fn line_number(&mut self, prefix: &str, value: u32) {
        let mut n = Small::new();
        let _ = writeln!(n, "{prefix}{value}");
        self.add(Piece::Small(n));
    }
    pub fn new(structure: &'a Structure, node: NodeId, domain: Domain) -> Result<Self, Error> {
        let node = structure.node(node).ok_or(Error::Invalid)?;
        let mut text = Self::empty();
        if domain == Domain::Decoded
            && let Some(value) = node.decoded()
        {
            text.add(Piece::Text(value));
            text.searchable[0] = Some(HitRange {
                start: 0,
                end: value.len(),
            });
            return Ok(text);
        }
        text.add(Piece::Text("inspector de serialización YAML\n"));
        let mut kind = Small::new();
        let _ = writeln!(kind, "tipo: {}", crate::data::file_kind_name(node.kind()));
        text.add(Piece::Small(kind));
        if let Some(meta) = node.metadata() {
            text.line_number("documento: ", meta.document);
            if let Some(style) = meta.style {
                text.add(Piece::Text(match style {
                    YamlStyle::Plain => "estilo: plain\n",
                    YamlStyle::SingleQuoted => "estilo: single quoted\n",
                    YamlStyle::DoubleQuoted => "estilo: double quoted\n",
                    YamlStyle::Literal => "estilo: literal\n",
                    YamlStyle::Folded => "estilo: folded\n",
                }));
            }
            if let Some(anchor) = meta.anchor {
                text.line_number("anchor ID: ", anchor)
            }
            if let Some(tag) = meta.tag {
                text.add(Piece::Text("tag URI:\n"));
                let start = text.raw_len();
                text.add(Piece::Text(structure.text_ref(tag)));
                text.searchable[0] = Some(HitRange {
                    start,
                    end: text.raw_len(),
                });
                text.add(Piece::Text("\n"));
            }
            if let Some(alias) = meta.alias {
                text.line_number("alias anchor ID: ", alias.anchor);
                text.line_number("target node: ", alias.target.0);
                if let Some(name) = structure.alias_name(node.id()) {
                    text.add(Piece::Text("alias name:\n"));
                    text.add(Piece::Text(name));
                    text.add(Piece::Text("\n"));
                }
                if let Some(span) = node.source_location() {
                    text.add(Piece::Text("alias raw token:\n"));
                    text.add(Piece::Text(&structure.source()[span.range()]));
                    text.add(Piece::Text("\n"));
                }
                let start = text.raw_len();
                text.add(Piece::Text("alias *"));
                if let Some(name) = structure.alias_name(node.id()) {
                    text.add(Piece::Text(name));
                }
                text.add(Piece::Text(" -> anchor#"));
                let mut id = Small::new();
                let _ = write!(id, "{}", alias.anchor);
                text.add(Piece::Small(id));
                text.searchable[1] = Some(HitRange {
                    start,
                    end: text.raw_len(),
                });
                text.add(Piece::Text("\n"));
                text.add(Piece::Text(
                    "Enter sigue explícitamente el target; búsqueda no lo expande\n",
                ));
            }
            if node.kind() == StructureKind::YamlDocument {
                text.add(Piece::Text(if meta.explicit_start {
                    "document start: explicit\n"
                } else {
                    "document start: implicit\n"
                }));
                if meta.synthetic_root {
                    text.add(Piece::Text(
                        "document root: absent token (null implícito)\n",
                    ));
                }
            }
        }
        Ok(text)
    }
    pub fn raw_len(&self) -> usize {
        self.pieces[..self.count]
            .iter()
            .map(|p| p.text().len())
            .sum()
    }
    pub fn search_ranges(&self) -> [Option<HitRange>; 2] {
        self.searchable
    }
    pub fn raw_bytes(&self) -> impl Iterator<Item = u8> + '_ {
        self.pieces[..self.count]
            .iter()
            .flat_map(|piece| piece.text().bytes())
    }
    /// Translate domain bytes to presentation bytes without copying the domain.
    pub fn visible_range(&self, range: HitRange, cancel: &AtomicBool) -> Result<HitRange, Error> {
        self.project_range(range, |_| check(cancel))
    }
    /// The byte threshold is crossed even when UTF8 character starts never align.
    fn project_range(
        &self,
        range: HitRange,
        mut poll: impl FnMut(usize) -> Result<(), Error>,
    ) -> Result<HitRange, Error> {
        let mut raw: usize = 0;
        let mut visible = 0;
        let mut start = None;
        let mut end = None;
        for piece in &self.pieces[..self.count] {
            let text = piece.text();
            let mut next_check = 0;
            for (index, ch) in text.char_indices() {
                if index >= next_check {
                    poll(raw)?;
                    next_check = index.saturating_add(1024);
                }
                if raw == range.start {
                    start = Some(visible);
                }
                if raw == range.end {
                    end = Some(visible);
                }
                if let (Some(start), Some(end)) = (start, end) {
                    return Ok(HitRange { start, end });
                }
                let next = index + ch.len_utf8();
                visible += match ch {
                    '\n' | '\t' => 1,
                    '\r' if text[next..].starts_with('\n') => 1,
                    '\0'..='\u{1f}' | '\u{7f}' => 3,
                    '\u{80}'..='\u{9f}' => 8,
                    _ => ch.len_utf8(),
                };
                raw += ch.len_utf8();
            }
        }
        if raw == range.start {
            start = Some(visible);
        }
        if raw == range.end {
            end = Some(visible);
        }
        poll(raw)?;
        match (start, end) {
            (Some(start), Some(end)) => Ok(HitRange { start, end }),
            _ => Err(Error::Invalid),
        }
    }
    pub fn chars(&self) -> SafeChars<'_> {
        SafeChars {
            text: self,
            piece: 0,
            byte: 0,
            emission: Small::new(),
            emit: 0,
            offset: 0,
            ticks: 0,
        }
    }
}
/// Identical safe control spelling to SafeTextModel, without creating that model.
pub struct SafeChars<'a> {
    text: &'a Text<'a>,
    piece: usize,
    byte: usize,
    emission: Small,
    emit: usize,
    offset: usize,
    ticks: usize,
}
impl SafeChars<'_> {
    pub fn next_byte(&mut self, cancel: &AtomicBool) -> Result<Option<(usize, u8)>, Error> {
        loop {
            if self.ticks.is_multiple_of(1024) {
                check(cancel)?
            }
            self.ticks += 1;
            if self.emit < self.emission.len {
                let result = (self.offset, self.emission.bytes[self.emit]);
                self.offset += 1;
                self.emit += 1;
                return Ok(Some(result));
            }
            if self.piece == self.text.count {
                return Ok(None);
            }
            let s = self.text.pieces[self.piece].text();
            if self.byte == s.len() {
                self.piece += 1;
                self.byte = 0;
                continue;
            }
            let ch = s[self.byte..].chars().next().unwrap();
            let next = self.byte + ch.len_utf8();
            self.emission = Small::new();
            match ch {
                '\n' | '\t' => {
                    let _ = write!(self.emission, "{ch}");
                }
                '\r' if s[next..].starts_with('\n') => {
                    let _ = write!(self.emission, "{ch}");
                }
                '\u{7f}' => {
                    let _ = write!(self.emission, "␡");
                }
                '\0'..='\u{1f}' => {
                    let _ = write!(
                        self.emission,
                        "{}",
                        char::from_u32(0x2400 + ch as u32).unwrap()
                    );
                }
                '\u{80}'..='\u{9f}' => {
                    let _ = write!(self.emission, "\\u{{{:04x}}}", ch as u32);
                }
                _ => {
                    let _ = write!(self.emission, "{ch}");
                }
            }
            self.byte = next;
            self.emit = 0;
        }
    }
}
#[derive(Clone, Copy, Default)]
struct Point {
    piece: usize,
    byte: usize,
    visible: usize,
    line: usize,
    cell: usize,
}
pub struct Pane {
    pub node: NodeId,
    pub domain: Domain,
    points: Vec<Point>,
    lines: usize,
    width: usize,
    _charge: Charge,
}
fn boundary(
    s: &str,
    start: usize,
    cursor: &mut GraphemeCursor,
    cancel: &AtomicBool,
) -> Result<usize, Error> {
    let mut chunk = start;
    loop {
        check(cancel)?;
        let mut end = (chunk + 1024).min(s.len());
        while !s.is_char_boundary(end) {
            end -= 1
        }
        match cursor.next_boundary(&s[chunk..end], chunk) {
            Ok(Some(n)) => {
                return if n - start > CLUSTER_BYTES {
                    Err(Error::Window)
                } else {
                    Ok(n)
                };
            }
            Ok(None) => return Ok(s.len()),
            Err(GraphemeIncomplete::NextChunk) => {
                chunk = end;
                if chunk - start > CLUSTER_BYTES {
                    return Err(Error::Window);
                }
            }
            Err(GraphemeIncomplete::PreContext(n)) => {
                let mut left = n.saturating_sub(1024);
                while !s.is_char_boundary(left) {
                    left += 1
                }
                cursor.provide_context(&s[left..n], left);
            }
            _ => return Err(Error::Invalid),
        }
    }
}
fn safe_cluster(raw: &str) -> Result<Small, Error> {
    let mut out = Small::new();
    if raw.len() > out.bytes.len() {
        return Err(Error::Window);
    }
    let cancel = AtomicBool::new(false);
    let mut text = Text::empty();
    text.add(Piece::Text(raw));
    let mut chars = text.chars();
    while let Some((_, b)) = chars.next_byte(&cancel)? {
        if out.len == out.bytes.len() {
            return Err(Error::Window);
        }
        out.bytes[out.len] = b;
        out.len += 1;
    }
    Ok(out)
}
impl Pane {
    pub fn prepare(
        structure: &Structure,
        state: State,
        budget: &Budget,
        cancel: &AtomicBool,
    ) -> Result<Self, Error> {
        let text = Text::new(structure, state.node, state.domain)?;
        let charge = budget.charge(CHECKPOINTS * size_of::<Point>() + 16384)?;
        let mut points = Vec::new();
        points
            .try_reserve_exact(CHECKPOINTS)
            .map_err(|_| Error::Budget)?;
        if points.capacity() > CHECKPOINTS {
            return Err(Error::Budget);
        }
        let stride = (text.raw_len() / 2048).max(4096);
        let mut at = Point::default();
        points.push(at);
        let mut raw_total = 0;
        let mut next = stride;
        let mut width = 0;
        for piece in 0..text.count {
            let s = text.pieces[piece].text();
            at.piece = piece;
            at.byte = 0;
            let mut graphemes = GraphemeCursor::new(0, s.len(), true);
            while at.byte < s.len() {
                check(cancel)?;
                if raw_total >= next {
                    if points.len() == CHECKPOINTS {
                        return Err(Error::Window);
                    }
                    points.push(at);
                    next = raw_total.saturating_add(stride);
                }
                let end = boundary(s, at.byte, &mut graphemes, cancel)?;
                let raw = &s[at.byte..end];
                let (len, cells, newline) = if raw == "\n" || raw == "\r\n" {
                    (raw.len(), 0, true)
                } else if raw.chars().any(char::is_control) {
                    let safe = safe_cluster(raw)?;
                    (
                        safe.len,
                        crate::presentation::cell_width(safe.as_str(), at.cell),
                        false,
                    )
                } else {
                    (
                        raw.len(),
                        crate::presentation::cell_width(raw, at.cell),
                        false,
                    )
                };
                at.visible += len;
                if newline {
                    width = width.max(at.cell);
                    at.line += 1;
                    at.cell = 0
                } else {
                    at.cell += cells
                }
                raw_total += end - at.byte;
                at.byte = end;
            }
        }
        width = width.max(at.cell);
        Ok(Self {
            node: state.node,
            domain: state.domain,
            points,
            lines: at.line + 1,
            width,
            _charge: charge,
        })
    }
    pub fn line_count(&self) -> usize {
        self.lines
    }
    pub fn max_width(&self) -> usize {
        self.width
    }
    fn start(&self, line: usize, left: usize) -> Point {
        let i = self
            .points
            .partition_point(|p| p.line < line || (p.line == line && p.cell <= left));
        self.points[i.saturating_sub(1)]
    }
    pub fn position(
        &self,
        structure: &Structure,
        offset: usize,
        cancel: &AtomicBool,
    ) -> Result<(usize, usize), Error> {
        let text = Text::new(structure, self.node, self.domain)?;
        let i = self.points.partition_point(|p| p.visible <= offset);
        let mut at = self.points[i.saturating_sub(1)];
        let mut found = None;
        self.walk(&text, &mut at, cancel, |value, p, _, _| {
            if offset < p.visible + value.len() {
                found = Some((
                    p.line,
                    p.cell
                        + if value.is_ascii() {
                            offset.saturating_sub(p.visible)
                        } else {
                            0
                        },
                ));
                true
            } else {
                false
            }
        })?;
        Ok(found.unwrap_or((at.line, at.cell)))
    }
    fn walk(
        &self,
        text: &Text<'_>,
        at: &mut Point,
        cancel: &AtomicBool,
        mut visit: impl FnMut(&str, Point, usize, bool) -> bool,
    ) -> Result<(), Error> {
        while at.piece < text.count {
            let s = text.pieces[at.piece].text();
            let mut graphemes = GraphemeCursor::new(at.byte, s.len(), true);
            while at.byte < s.len() {
                let end = boundary(s, at.byte, &mut graphemes, cancel)?;
                let raw = &s[at.byte..end];
                let newline = raw == "\n" || raw == "\r\n";
                let safe;
                let value = if raw.chars().any(char::is_control) && !newline {
                    safe = safe_cluster(raw)?;
                    safe.as_str()
                } else {
                    raw
                };
                let cells = if newline {
                    0
                } else {
                    crate::presentation::cell_width(value, at.cell)
                };
                if visit(value, *at, cells, newline) {
                    return Ok(());
                }
                at.visible += value.len();
                if newline {
                    at.line += 1;
                    at.cell = 0
                } else {
                    at.cell += cells
                }
                at.byte = end;
            }
            at.piece += 1;
            at.byte = 0;
        }
        Ok(())
    }
    /// Lease includes cell/String capacities, temporary fragments, and the UI's
    /// one matching Span vector. It lives until the widget finishes rendering.
    #[allow(clippy::too_many_arguments)]
    pub fn window(
        &self,
        structure: &Structure,
        line: usize,
        left: usize,
        width: usize,
        hit: Option<HitRange>,
        budget: &Budget,
        cancel: &AtomicBool,
    ) -> Result<Window, Error> {
        let charge = budget.charge(WINDOW_BYTES * 3)?;
        let text = Text::new(structure, self.node, self.domain)?;
        let mut at = self.start(line, left);
        let mut cells = Vec::new();
        cells.try_reserve_exact(2048).map_err(|_| Error::Budget)?;
        if cells.capacity() > 2048 {
            return Err(Error::Budget);
        }
        let right = left.saturating_add(width);
        let mut bytes = 0usize;
        let mut failed = false;
        self.walk(&text, &mut at, cancel, |value, p, w, newline| {
            if p.line > line || p.line == line && p.cell >= right {
                return true;
            }
            if p.line == line && !newline && w > 0 && p.cell + w > left {
                // C1 placeholders are ASCII fragments: keep their individual
                // cells and byte ranges when scrolling/highlighting inside one.
                if value.is_ascii() && value.len() > 1 && value != "\t" {
                    for (index, byte) in value.bytes().enumerate() {
                        let cell = p.cell + index;
                        if cell < left || cell >= right {
                            continue;
                        }
                        if cells.len() == 2048 || bytes == WINDOW_BYTES {
                            failed = true;
                            return true;
                        }
                        let mut shown = String::new();
                        if shown.try_reserve_exact(1).is_err() || shown.capacity() > 1 {
                            failed = true;
                            return true;
                        }
                        shown.push(byte as char);
                        bytes += shown.capacity();
                        cells.push(Cell {
                            text: shown,
                            matched: hit.is_some_and(|h| {
                                h.start < p.visible + index + 1 && h.end > p.visible + index
                            }),
                        });
                    }
                    return false;
                }
                let length = value.len().max(w);
                if cells.len() == 2048 || bytes.saturating_add(length) > WINDOW_BYTES {
                    failed = true;
                    return true;
                }
                let mut shown = String::new();
                if shown.try_reserve_exact(length).is_err() || shown.capacity() > length {
                    failed = true;
                    return true;
                }
                if value == "\t" {
                    shown.extend(std::iter::repeat_n(
                        ' ',
                        right.min(p.cell + w) - p.cell.max(left),
                    ));
                } else if p.cell < left || p.cell + w > right {
                    if value.is_ascii() {
                        let a = left.saturating_sub(p.cell).min(value.len());
                        let b = right.saturating_sub(p.cell).min(value.len());
                        shown.push_str(&value[a..b]);
                    } else {
                        shown.extend(std::iter::repeat_n(
                            ' ',
                            right.min(p.cell + w) - p.cell.max(left),
                        ));
                    }
                } else {
                    shown.push_str(value)
                }
                bytes += shown.capacity();
                cells.push(Cell {
                    text: shown,
                    matched: hit
                        .is_some_and(|h| h.start < p.visible + value.len() && h.end > p.visible),
                });
            }
            false
        })?;
        if failed {
            return Err(Error::Window);
        }
        Ok(Window {
            cells,
            _charge: charge,
        })
    }
}
pub struct Cell {
    pub text: String,
    pub matched: bool,
}
pub struct Window {
    pub cells: Vec<Cell>,
    _charge: Charge,
}
/// Key-first serialization DFS. Fixed frames; references are terminal leaves.
pub struct Visits<'a> {
    structure: &'a Structure,
    stack: [Option<(NodeId, usize, u8)>; 67],
    len: usize,
    cancel: &'a AtomicBool,
}
impl<'a> Visits<'a> {
    pub fn new(structure: &'a Structure, cancel: &'a AtomicBool) -> Self {
        let mut stack = [None; 67];
        stack[0] = Some((structure.root().id(), 0, 0));
        Self {
            structure,
            stack,
            len: 1,
            cancel,
        }
    }
}
impl Iterator for Visits<'_> {
    type Item = Result<NodeId, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.len == 0 {
                return None;
            }
            if let Err(e) = check(self.cancel) {
                self.len = 0;
                return Some(Err(e));
            }
            let (id, next, stage) = self.stack[self.len - 1]?;
            let node = self.structure.node(id)?;
            if stage == 0 {
                self.stack[self.len - 1] = Some((id, 0, 1));
                return Some(Ok(id));
            }
            let Some(entry) = node.entry(next) else {
                self.len -= 1;
                continue;
            };
            let child = if stage == 1 {
                self.stack[self.len - 1] = Some((id, next, 2));
                entry.key()
            } else {
                self.stack[self.len - 1] = Some((id, next + 1, 1));
                Some(entry.value())
            };
            if let Some(child) = child {
                if self.len == self.stack.len() {
                    self.len = 0;
                    return Some(Err(Error::Invalid));
                }
                self.stack[self.len] = Some((child.id(), 0, 0));
                self.len += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn model(raw: &str) -> Arc<Structure> {
        Arc::new(
            crate::yaml_worker::project(Arc::new(raw.to_owned()), 1, &AtomicBool::new(false))
                .unwrap()
                .0,
        )
    }
    fn first_scalar(s: &Structure) -> NodeId {
        Visits::new(s, &AtomicBool::new(false))
            .filter_map(Result::ok)
            .find(|id| s.node(*id).unwrap().decoded().is_some())
            .unwrap()
    }
    #[test]
    fn history_has_128_previous_states_and_rejection_preserves_every_return() {
        let structure = model("&a [*a]");
        let alias = Visits::new(&structure, &AtomicBool::new(false))
            .filter_map(Result::ok)
            .find(|id| structure.node(*id).unwrap().kind() == StructureKind::Alias)
            .unwrap();
        let mut nav = Navigator::new(structure).unwrap();
        nav.current_mut().node = alias;
        let first = nav.current();
        let target = nav.following_alias().unwrap();
        let cancel = AtomicBool::new(false);
        for index in 0..HISTORY_LIMIT {
            let mut next = nav.current();
            next.node = if index % 2 == 0 { target } else { alias };
            next.cursor.row = index;
            nav.commit(next, &cancel).unwrap();
        }
        let current = nav.current();
        assert_eq!(nav.history_len(), 128);
        assert_eq!(nav.commit(first, &cancel), Err(Error::History));
        assert_eq!(nav.current(), current);
        for _ in 0..128 {
            assert!(nav.back())
        }
        assert_eq!(nav.current(), first);
        assert!(!nav.back());
        assert_eq!(Visits::new(nav.structure(), &cancel).count(), 4); // stream/doc/sequence/alias, no semantic cycle expansion
    }
    #[test]
    fn borrowed_windows_keep_unicode_control_cells_and_offsets_without_scalar_copies() {
        let structure = model("\"界e\\u0301👩‍💻\\t\\e␛\\u009bEND\"\n");
        let node = first_scalar(&structure);
        let nav = Navigator::new(structure.clone()).unwrap();
        let state = State {
            node,
            cursor: Default::default(),
            domain: Domain::Decoded,
            hit: None,
            last_hit: None,
        };
        let cancel = AtomicBool::new(false);
        let baseline = nav.budget().used();
        let pane = Pane::prepare(&structure, state, nav.budget(), &cancel).unwrap();
        assert!(pane.points.capacity() <= CHECKPOINTS);
        let window = pane
            .window(&structure, 0, 0, 100, None, nav.budget(), &cancel)
            .unwrap();
        let rendered: String = window.cells.iter().map(|c| c.text.as_str()).collect();
        assert!(rendered.contains("界e\u{301}👩‍💻"));
        assert!(rendered.contains("␛␛\\u{009b}END"));
        assert!(!rendered.contains('\x1b'));
        drop(window);
        let offset = crate::text::safe_text(structure.node(node).unwrap().decoded().unwrap())
            .find("u{009b}")
            .unwrap();
        let (_, left) = pane.position(&structure, offset, &cancel).unwrap();
        let cropped = pane
            .window(&structure, 0, left, 3, None, nav.budget(), &cancel)
            .unwrap();
        assert_eq!(
            cropped
                .cells
                .iter()
                .map(|c| c.text.as_str())
                .collect::<String>(),
            "u{0"
        );
        drop(cropped);
        drop(pane);
        assert_eq!(nav.budget().used(), baseline);
    }
    #[test]
    fn million_byte_leaf_uses_fixed_checkpoints_and_shared_admission_before_window() {
        let raw = format!("\"{}END\"", "x".repeat(1_000_000));
        let structure = model(&raw);
        let node = first_scalar(&structure);
        let nav = Navigator::new(structure.clone()).unwrap();
        let state = State {
            node,
            cursor: Default::default(),
            domain: Domain::Decoded,
            hit: None,
            last_hit: None,
        };
        let cancel = AtomicBool::new(false);
        let pane = Pane::prepare(&structure, state, nav.budget(), &cancel).unwrap();
        assert_eq!(pane.max_width(), 1_000_003);
        assert!(pane.points.len() < 4096);
        let window = pane
            .window(&structure, 0, 999_999, 4, None, nav.budget(), &cancel)
            .unwrap();
        assert_eq!(
            window
                .cells
                .iter()
                .map(|c| c.text.as_str())
                .collect::<String>(),
            "xEND"
        );
        drop(window);
        let used = nav.budget().used();
        let hold = nav.budget().charge(SESSION_BYTES - used - 1).unwrap();
        assert!(matches!(
            pane.window(&structure, 0, 0, 10, None, nav.budget(), &cancel),
            Err(Error::Budget)
        ));
        assert!(nav.budget().used() <= SESSION_BYTES);
        drop(hold);
        assert!(matches!(
            Pane::prepare(&structure, state, nav.budget(), &AtomicBool::new(true)),
            Err(Error::Cancelled)
        ));
    }
    #[test]
    fn active_projection_cancel_does_not_require_aligned_utf8_byte_indices() {
        for ch in ['é', '\u{009b}'] {
            let value = format!("a{}", ch.to_string().repeat(5000));
            let mut text = Text::empty();
            text.add(Piece::Text(&value));
            let range = HitRange {
                start: value.len() - ch.len_utf8(),
                end: value.len(),
            };
            let cancel = AtomicBool::new(false);
            let mut checkpoints = Vec::new();
            let projected = text.project_range(range, |offset| {
                checkpoints.push(offset);
                if offset > 0 {
                    cancel.store(true, Ordering::Relaxed);
                }
                check(&cancel)
            });
            assert_eq!(projected, Err(Error::Cancelled));
            // Cancellation switches on inside traversal, at the first crossed byte
            // threshold. A final-only poll would be 10001, and fails this oracle.
            assert_eq!(checkpoints, [0, 1025]);
            assert!(checkpoints[1] < range.start);
            cancel.store(false, Ordering::Relaxed);
            let visible = text.visible_range(range, &cancel).unwrap();
            let end = if ch == 'é' {
                value.len()
            } else {
                1 + 5000 * 8
            };
            let size = if ch == 'é' { 2 } else { 8 };
            assert_eq!(
                visible,
                HitRange {
                    start: end - size,
                    end
                }
            );
        }
    }
}
