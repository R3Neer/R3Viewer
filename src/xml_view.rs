//! XML physical navigation with bounded borrowed windows; no links or constructors.
use crate::{
    data::Cursor,
    structure_model::{NodeId, NodeRef, Structure, StructureKind},
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
pub const HISTORY_LIMIT: usize = 128;
pub const SESSION_BYTES: usize = 1024 * 1024;
pub const WINDOW_BYTES: usize = 128 * 1024;
const CHECKPOINTS: usize = 4096;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Cancelled,
    Budget,
    History,
    Invalid,
    Window,
    FrameSymbols,
    Work,
    Reference {
        start: usize,
        end: usize,
        visible_start: usize,
        visible_end: usize,
    },
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Cancelled => "cancelado",
            Self::Budget => "reserva XML de sesión agotada",
            Self::History => "historial XML: máximo 128 entradas previas; vuelve antes de navegar",
            Self::Invalid => "dirección XML no válida",
            Self::FrameSymbols => "referencia de viewport: símbolos/metadata exceden 256KiB",
            Self::Work => "referencia de viewport: trabajo excede 256KiB",
            Self::Reference { .. } => "referencia de viewport: símbolo supera 16KiB",
            Self::Window => "ventana XML supera su límite; el texto completo sigue en Fuente",
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
    Container,
    Inspector,
    QName,
    LocalName,
    NamespaceUri,
    ValueDecoded,
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
    pub position_pending: bool,
    pub last_hit: Option<crate::search::XmlSearchHit>,
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
                domain: Domain::Container,
                hit: None,
                position_pending: false,
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
        if self.current.domain != Domain::Container
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
            let _ = write!(out, "/{}", link.ordinal);
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

pub const SEARCH_DOMAINS: [Domain; 5] = [
    Domain::QName,
    Domain::LocalName,
    Domain::NamespaceUri,
    Domain::ValueDecoded,
    Domain::Metadata,
];
pub fn unit(structure: &Structure, id: NodeId, domain: Domain) -> Option<&str> {
    let node = structure.node(id)?;
    let meta = node.xml_metadata()?;
    let raw = |span: crate::structure_model::SourceSpan| &structure.source()[span.range()];
    match domain {
        Domain::QName => meta
            .name
            .map(|name| raw(name.qname))
            .or_else(|| meta.target.map(raw)),
        Domain::LocalName => {
            if node.kind() == StructureKind::XmlNamespaceDeclaration {
                meta.binding_prefix.map(raw)
            } else {
                meta.name.map(|name| raw(name.local))
            }
        }
        Domain::NamespaceUri => {
            if node.kind() == StructureKind::XmlNamespaceDeclaration {
                None
            } else {
                meta.name
                    .and_then(|name| name.namespace)
                    .map(|uri| structure.text_ref(uri))
            }
        }
        Domain::ValueDecoded => {
            if matches!(
                node.kind(),
                StructureKind::XmlAttribute
                    | StructureKind::XmlNamespaceDeclaration
                    | StructureKind::XmlText
                    | StructureKind::XmlReference
                    | StructureKind::XmlCData
            ) {
                node.decoded()
            } else {
                None
            }
        }
        Domain::Metadata => match node.kind() {
            StructureKind::XmlComment | StructureKind::XmlPi => node.decoded(),
            StructureKind::XmlDeclaration | StructureKind::XmlDocType => {
                node.source_location().map(raw)
            }
            _ => None,
        },
        _ => None,
    }
}
pub fn open_domain(node: NodeRef<'_>) -> Domain {
    if node.kind().is_container() {
        Domain::Container
    } else if matches!(
        node.kind(),
        StructureKind::XmlComment
            | StructureKind::XmlPi
            | StructureKind::XmlDeclaration
            | StructureKind::XmlDocType
    ) {
        Domain::Metadata
    } else {
        Domain::ValueDecoded
    }
}

pub struct Text<'a> {
    pieces: [Piece<'a>; 64],
    count: usize,
    searchable: [Option<HitRange>; 2],
}
impl<'a> Text<'a> {
    fn empty() -> Self {
        Self {
            pieces: [Piece::Text(""); 64],
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
    pub fn new(structure: &'a Structure, id: NodeId, domain: Domain) -> Result<Self, Error> {
        let node = structure.node(id).ok_or(Error::Invalid)?;
        let mut text = Self::empty();
        if domain != Domain::Inspector {
            if let Some(value) = unit(structure, id, domain) {
                text.add(Piece::Text(value));
                text.searchable[0] = Some(HitRange {
                    start: 0,
                    end: value.len(),
                });
            }
            return Ok(text);
        }
        text.add(Piece::Text("inspector XML\n"));
        text.add(Piece::Text(crate::data::file_kind_name(node.kind())));
        text.add(Piece::Text("\n"));
        for (domain, label) in [
            (Domain::QName, "QName"),
            (Domain::LocalName, "LocalName/prefijo declarado"),
            (Domain::NamespaceUri, "NamespaceUri"),
            (Domain::ValueDecoded, "ValueDecoded"),
            (Domain::Metadata, "Metadata"),
        ] {
            if let Some(value) = unit(structure, id, domain) {
                text.add(Piece::Text(label));
                text.add(Piece::Text(":\n"));
                text.add(Piece::Text(value));
                text.add(Piece::Text("\n"));
            }
        }
        if let Some(meta) = node.xml_metadata() {
            text.line_number("attributes: ", meta.attribute_count);
            text.add(Piece::Text(if meta.empty_form {
                "empty: true\n"
            } else {
                "empty: false\n"
            }));
            for (label, span) in [
                ("open span: ", meta.open_span),
                ("close span: ", meta.close_span),
                ("quoted value span: ", meta.quoted_value),
                ("value content span: ", meta.value_content),
                ("binding prefix span: ", meta.binding_prefix),
            ] {
                if let Some(span) = span {
                    let mut shown = Small::new();
                    let _ = writeln!(shown, "{label}{}..{}", span.start, span.end);
                    text.add(Piece::Small(shown));
                }
            }
            if let Some(reference) = meta.reference_kind {
                let mut shown = Small::new();
                let _ = writeln!(shown, "reference kind: {reference:?}");
                text.add(Piece::Small(shown));
            }
            if meta.has_doctype {
                text.add(Piece::Text(
                    "DOCTYPE inerte; no se abre ningún recurso externo\n",
                ));
            }
            if let Some(root) = meta.root_element {
                text.line_number("root node: ", root.0);
            }
        }
        if let Some(span) = node.source_location() {
            text.line_number("raw start: ", span.start);
            text.line_number("raw end: ", span.end);
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
#[derive(Clone)]
struct Point {
    piece: usize,
    cursor: crate::text_engine::Point,
}
struct IndexedPiece {
    index: crate::text_engine::Index,
    identity: crate::text_engine::Identity,
    raw: usize,
    visible: usize,
    line: usize,
    end_line: usize,
    last_column: usize,
    signature: usize,
    len: usize,
}
pub struct Pane {
    pub node: NodeId,
    pub domain: Domain,
    pieces: Vec<IndexedPiece>,
    structure_address: usize,
    source: Arc<String>,
    generation: u64,
    lines: usize,
    width: usize,
    last_column: usize,
    position_reference: Option<HitRange>,
    _charge: Charge,
}
fn engine_error(error: crate::text_engine::Error) -> Error {
    match error {
        crate::text_engine::Error::Cancelled => Error::Cancelled,
        crate::text_engine::Error::Work => Error::Work,
        crate::text_engine::Error::Invalid => Error::Invalid,
    }
}
fn signature(piece: &Piece<'_>) -> usize {
    match piece {
        Piece::Text(s) => s.as_ptr() as usize,
        Piece::Small(s) => {
            let mut hash = 0usize;
            for b in s.as_str().bytes() {
                hash = hash.wrapping_mul(16777619) ^ usize::from(b);
            }
            hash
        }
    }
}
impl Charge {
    fn shrink(&mut self, bytes: usize) -> Result<(), Error> {
        if bytes > self.bytes {
            return Err(Error::Budget);
        }
        self.used.fetch_sub(self.bytes - bytes, Ordering::AcqRel);
        self.bytes = bytes;
        Ok(())
    }
}
impl Pane {
    pub fn prepare(
        structure: &Structure,
        state: State,
        budget: &Budget,
        cancel: &AtomicBool,
    ) -> Result<Self, Error> {
        let text = Text::new(structure, state.node, state.domain)?;
        let available = SESSION_BYTES.saturating_sub(budget.used());
        let mut charge = budget.charge(available)?;
        let fixed = text
            .count
            .checked_mul(size_of::<IndexedPiece>() + 2 * size_of::<usize>())
            .ok_or(Error::Budget)?;
        if fixed > available {
            return Err(Error::Budget);
        }
        let mut pieces = Vec::new();
        pieces
            .try_reserve_exact(text.count)
            .map_err(|_| Error::Budget)?;
        if pieces.capacity() > text.count {
            return Err(Error::Budget);
        }
        let mut allocated = fixed;
        let mut raw = 0usize;
        let mut visible = 0usize;
        let mut line = 0usize;
        let mut column = 0usize;
        let mut width = 0;
        let mut snapshots = 0;
        let stride = text.raw_len().div_ceil(CHECKPOINTS - 1).max(4096);
        let mut work = crate::text_engine::Work::preparation(cancel);
        for piece in &text.pieces[..text.count] {
            let value = piece.text();
            let identity = crate::text_engine::Identity::new();
            let index = crate::text_engine::Index::prepare_projected(
                value,
                identity.clone(),
                &mut work,
                stride,
                available.saturating_sub(allocated),
                column,
            )
            .map_err(engine_error)?;
            snapshots += index.snapshot_capacity();
            if snapshots > CHECKPOINTS {
                return Err(Error::Budget);
            }
            allocated = allocated
                .checked_add(index.allocated)
                .ok_or(Error::Budget)?;
            width = width.max(index.max_width);
            let (extra_lines, last_column, visible_len) = index.end_metrics();
            pieces.push(IndexedPiece {
                index,
                identity,
                raw,
                visible,
                line,
                end_line: line + extra_lines,
                last_column,
                signature: signature(piece),
                len: value.len(),
            });
            raw += value.len();
            visible += visible_len;
            line += extra_lines;
            column = last_column;
        }
        check(cancel)?;
        charge.shrink(allocated)?;
        Ok(Self {
            node: state.node,
            domain: state.domain,
            pieces,
            structure_address: structure as *const Structure as usize,
            source: structure.source_handle(),
            generation: structure.generation(),
            lines: line + 1,
            width,
            last_column: column,
            position_reference: state.hit.filter(|_| state.position_pending),
            _charge: charge,
        })
    }
    pub(crate) fn reference(
        &self,
        structure: &Structure,
        error: Error,
    ) -> Option<crate::viewport_admission::Reference> {
        let (range, visible, reason) = match error {
            Error::Reference {
                start,
                end,
                visible_start,
                visible_end,
            } => (
                start..end,
                visible_start..visible_end,
                crate::viewport_admission::Reason::Symbol,
            ),
            Error::Work | Error::FrameSymbols => {
                let last = self.pieces.last()?;
                (
                    0..last.raw + last.len,
                    0..last.visible + last.index.end_metrics().2,
                    if error == Error::Work {
                        crate::viewport_admission::Reason::Work
                    } else {
                        crate::viewport_admission::Reason::FrameSymbols
                    },
                )
            }
            _ => return None,
        };
        let source = structure
            .node(self.node)
            .and_then(|node| node.source_location())
            .map_or(crate::viewport_admission::SourceRelation::None, |span| {
                crate::viewport_admission::SourceRelation::EnclosingNodeSpan(span.range())
            });
        Some(crate::viewport_admission::Reference {
            domain: crate::viewport_admission::Domain::Xml {
                node: self.node.0,
                domain: match self.domain {
                    Domain::Container => "Container",
                    Domain::Inspector => "Inspector",
                    Domain::QName => "QName",
                    Domain::LocalName => "LocalName",
                    Domain::NamespaceUri => "NamespaceUri",
                    Domain::ValueDecoded => "ValueDecoded",
                    Domain::Metadata => "Metadata",
                },
            },
            range,
            range_exact: true,
            visible,
            source,
            reason,
            identity: self.pieces.first()?.identity.clone(),
            generation: Some(self.generation),
            unpositioned_hit: self.position_reference.map(|hit| hit.start..hit.end),
        })
    }
    #[cfg(test)]
    pub(crate) fn maximum_profile_seek_gap_fixture(
        &mut self,
        structure: &Structure,
        cancel: &AtomicBool,
    ) {
        let text = Text::new(structure, self.node, self.domain).unwrap();
        self.validate(structure, &text).unwrap();
        for (ordinal, piece) in self.pieces.iter_mut().enumerate() {
            piece.index = crate::text_engine::Index::prepare_projected(
                text.pieces[ordinal].text(),
                piece.identity.clone(),
                &mut crate::text_engine::Work::preparation(cancel),
                (128usize * 1024 * 1024).div_ceil(CHECKPOINTS - 1),
                1024 * 1024,
                0,
            )
            .unwrap();
        }
    }
    pub(crate) fn retain_unpositioned_hit(&mut self, hit: HitRange) {
        self.position_reference = Some(hit);
    }
    pub(crate) fn resume_manual_position(&mut self) {
        self.position_reference = None;
    }
    pub fn line_count(&self) -> usize {
        self.lines
    }
    pub fn max_width(&self) -> usize {
        self.width
    }
    fn validate(&self, structure: &Structure, text: &Text<'_>) -> Result<(), Error> {
        if self.structure_address != structure as *const Structure as usize
            || self.generation != structure.generation()
            || !Arc::ptr_eq(&self.source, &structure.source_handle())
            || text.count != self.pieces.len()
        {
            return Err(Error::Invalid);
        }
        for (piece, index) in text.pieces[..text.count].iter().zip(&self.pieces) {
            if piece.text().len() != index.len || signature(piece) != index.signature {
                return Err(Error::Invalid);
            }
        }
        Ok(())
    }
    fn start(&self, text: &Text<'_>, line: usize, left: usize) -> Result<Point, Error> {
        let piece = self
            .pieces
            .iter()
            .position(|p| {
                p.line <= line && p.end_line >= line && (p.end_line > line || p.last_column >= left)
            })
            .unwrap_or_else(|| self.pieces.len().saturating_sub(1));
        let p = self.pieces.get(piece).ok_or(Error::Invalid)?;
        let cursor = p
            .index
            .start_cell(
                text.pieces[piece].text(),
                &p.identity,
                line.saturating_sub(p.line),
                left,
            )
            .map_err(engine_error)?;
        Ok(Point { piece, cursor })
    }
    pub fn position(
        &self,
        structure: &Structure,
        offset: usize,
        cancel: &AtomicBool,
    ) -> Result<(usize, usize), Error> {
        let text = Text::new(structure, self.node, self.domain)?;
        self.validate(structure, &text)?;
        let n = self.pieces.partition_point(|p| p.visible <= offset);
        let piece = n.saturating_sub(1);
        let p = self.pieces.get(piece).ok_or(Error::Invalid)?;
        let cursor = p
            .index
            .start_visible(
                text.pieces[piece].text(),
                &p.identity,
                offset.saturating_sub(p.visible),
            )
            .map_err(engine_error)?;
        let mut at = Point { piece, cursor };
        let mut work = crate::text_engine::Work::frame(cancel);
        let mut found = None;
        self.walk(&text, &mut at, &mut work, |value, unit| {
            if offset < unit.visible.end {
                found = Some((
                    unit.line,
                    unit.column
                        + if unit.ascii {
                            offset.saturating_sub(unit.visible.start)
                        } else {
                            0
                        },
                ));
                true
            } else {
                let _ = value;
                false
            }
        })?;
        Ok(found.unwrap_or((self.lines - 1, self.last_column)))
    }
    fn walk(
        &self,
        text: &Text<'_>,
        at: &mut Point,
        work: &mut crate::text_engine::Work<'_>,
        mut visit: impl FnMut(&str, crate::text_engine::Unit) -> bool,
    ) -> Result<(), Error> {
        while let Some(piece) = self.pieces.get(at.piece) {
            let value = text.pieces[at.piece].text();
            while let Some(mut unit) = piece
                .index
                .step(value, &piece.identity, &mut at.cursor, work)
                .map_err(engine_error)?
            {
                let raw = &value[unit.range.clone()];
                unit.range.start += piece.raw;
                unit.range.end += piece.raw;
                unit.visible.start += piece.visible;
                unit.visible.end += piece.visible;
                unit.line += piece.line;
                let mut tiny = Small::new();
                let shown = if unit.safe_control {
                    let ch = raw.chars().next().ok_or(Error::Invalid)?;
                    if ch == '\u{7f}' {
                        write!(tiny, "␡").map_err(|_| Error::Invalid)?;
                    } else if ch <= '\u{1f}' {
                        write!(
                            tiny,
                            "{}",
                            char::from_u32(0x2400 + ch as u32).ok_or(Error::Invalid)?
                        )
                        .map_err(|_| Error::Invalid)?;
                    } else {
                        write!(tiny, "\\u{{{:04x}}}", ch as u32).map_err(|_| Error::Invalid)?;
                    }
                    tiny.as_str()
                } else {
                    raw
                };
                if visit(shown, unit) {
                    return Ok(());
                }
            }
            at.piece += 1;
            if let Some(next) = self.pieces.get(at.piece) {
                at.cursor = next
                    .index
                    .start_offset(text.pieces[at.piece].text(), &next.identity, 0)
                    .map_err(engine_error)?;
            }
        }
        Ok(())
    }
    /// All rows are admitted before any leaf String or widget is constructed.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn windows(
        &self,
        structure: &Structure,
        top: usize,
        rows: usize,
        left: usize,
        width: usize,
        hit: Option<HitRange>,
        budget: &Budget,
        frame: &mut crate::viewport_admission::FrameBudget<'_>,
    ) -> Result<Windows, Error> {
        frame.reference_reserve().map_err(|reason| match reason {
            crate::viewport_admission::Reason::Work => Error::Work,
            _ => Error::FrameSymbols,
        })?;
        let text = Text::new(structure, self.node, self.domain)?;
        self.validate(structure, &text)?;
        if self.position_reference.is_some() {
            return Err(Error::Work);
        }
        let count = rows.min(self.lines.saturating_sub(top));
        let plan_bytes = count
            .checked_mul(size_of::<RowPlan>() + size_of::<Window>())
            .ok_or(Error::Budget)?;
        frame.reserve(plan_bytes).map_err(|_| Error::FrameSymbols)?;
        let plan_charge = budget.charge(plan_bytes)?;
        let mut plans = Vec::new();
        plans.try_reserve_exact(count).map_err(|_| Error::Budget)?;
        if plans.capacity() > count {
            return Err(Error::Budget);
        }
        let mut sequential = None;
        for line in top..top + count {
            let start = match sequential.take() {
                Some(point) => point,
                None => self.start(&text, line, left)?,
            };
            let mut at = start.clone();
            let before = frame.work.used();
            let right = left.saturating_add(width);
            let mut bytes = 0usize;
            let mut cells = 0usize;
            let mut failure = None;
            self.walk(&text, &mut at, &mut frame.work, |value, unit| {
                // walk advances the resolved cursor before visit. Stop on this row's
                // newline so sequential replay never consumes the next row's first unit.
                if unit.line > line || unit.line == line && (unit.newline || unit.column >= right) {
                    return true;
                }
                if unit.line == line
                    && !unit.newline
                    && (unit.column + unit.width > left || unit.width == 0 && unit.column >= left)
                {
                    if unit.visible.len() > crate::viewport_admission::SYMBOL_BYTES {
                        failure = Some(Error::Reference {
                            start: unit.range.start,
                            end: unit.range.end,
                            visible_start: unit.visible.start,
                            visible_end: unit.visible.end,
                        });
                        return true;
                    }
                    if unit.width > 0 {
                        let ascii = unit.ascii && value.len() > 1 && value != "\t";
                        let n = if ascii {
                            right.min(unit.column + unit.width) - left.max(unit.column)
                        } else {
                            1
                        };
                        let length = if value == "\t"
                            || unit.column < left
                            || unit.column + unit.width > right
                        {
                            right.min(unit.column + unit.width) - left.max(unit.column)
                        } else {
                            value.len()
                        };
                        cells += n;
                        bytes += length;
                        if cells > 2048 || bytes > WINDOW_BYTES {
                            failure = Some(Error::Window);
                            return true;
                        }
                    }
                }
                false
            })?;
            if let Some(error) = failure {
                return Err(error);
            }
            sequential = Some(at);
            let replay = frame.work.used() - before;
            frame
                .work
                .admit(replay.checked_add(bytes).ok_or(Error::Work)?)
                .map_err(engine_error)?;
            let allocated = bytes
                .checked_mul(2)
                .and_then(|n| {
                    cells
                        .checked_mul(size_of::<Cell>() + size_of::<ratatui::text::Span<'static>>())
                        .and_then(|meta| n.checked_add(meta))
                })
                .ok_or(Error::Budget)?;
            frame.reserve(allocated).map_err(|_| Error::FrameSymbols)?;
            plans.push(RowPlan {
                line,
                start,
                bytes,
                cells,
                replay,
                allocated,
            });
        }
        // Lease the entire frame before allocating its Strings; keep it through render.
        let total = plans
            .iter()
            .try_fold(0usize, |n, p| n.checked_add(p.allocated))
            .ok_or(Error::Budget)?;
        let charge = budget.charge(total)?;
        let mut windows = Vec::new();
        windows
            .try_reserve_exact(count)
            .map_err(|_| Error::Budget)?;
        if windows.capacity() > count {
            return Err(Error::Budget);
        }
        for plan in plans {
            let mut cells = Vec::new();
            cells
                .try_reserve_exact(plan.cells)
                .map_err(|_| Error::Budget)?;
            if cells.capacity() > plan.cells {
                return Err(Error::Budget);
            }
            let mut at = plan.start;
            let right = left.saturating_add(width);
            let mut copied = 0usize;
            let mut failure = None;
            let mut work =
                crate::text_engine::Work::permit(frame.work.cancel, plan.replay + plan.bytes);
            work.project(plan.bytes).map_err(engine_error)?;
            self.walk(&text, &mut at, &mut work, |value, unit| {
                // walk advances the resolved cursor before visit. Stop on this row's
                // newline so sequential replay never consumes the next row's first unit.
                if unit.line > plan.line
                    || unit.line == plan.line && (unit.newline || unit.column >= right)
                {
                    return true;
                }
                if unit.line == plan.line
                    && !unit.newline
                    && unit.width > 0
                    && unit.column + unit.width > left
                {
                    let ascii = unit.ascii && value.len() > 1 && value != "\t";
                    let a = left.saturating_sub(unit.column);
                    let b = right.saturating_sub(unit.column).min(unit.width);
                    let mut push = |shown: &str, range: Range<usize>| -> bool {
                        if frame.work.cancel.load(Ordering::Relaxed) {
                            failure = Some(Error::Cancelled);
                            return false;
                        }
                        if cells.len() >= plan.cells || copied + shown.len() > plan.bytes {
                            failure = Some(Error::Invalid);
                            return false;
                        }
                        let mut output = String::new();
                        if output.try_reserve_exact(shown.len()).is_err()
                            || output.capacity() > shown.len()
                        {
                            failure = Some(Error::Budget);
                            return false;
                        }
                        output.push_str(shown);
                        copied += output.capacity();
                        cells.push(Cell {
                            text: output,
                            matched: hit
                                .is_some_and(|h| h.start < range.end && h.end > range.start),
                        });
                        true
                    };
                    if ascii {
                        for i in a..b {
                            if !push(
                                &value[i..i + 1],
                                unit.visible.start + i..unit.visible.start + i + 1,
                            ) {
                                return true;
                            }
                        }
                    } else if value == "\t" || a > 0 || b < unit.width {
                        let length = b - a;
                        let blanks = [b' '; 2048];
                        let shown = std::str::from_utf8(&blanks[..length]).unwrap();
                        if !push(shown, unit.visible) {
                            return true;
                        }
                    } else if !push(value, unit.visible) {
                        return true;
                    }
                }
                false
            })?;
            if let Some(error) = failure {
                return Err(error);
            }
            if copied != plan.bytes || cells.len() != plan.cells {
                return Err(Error::Invalid);
            }
            windows.push(Window {
                cells,
                _charge: None,
            });
        }
        check(frame.work.cancel)?;
        Ok(Windows {
            rows: windows,
            _charges: Some((charge, plan_charge)),
        })
    }
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
        let mut frame = crate::viewport_admission::FrameBudget::new(cancel);
        let mut windows = self.windows(structure, line, 1, left, width, hit, budget, &mut frame)?;
        let mut window = windows.rows.pop().ok_or(Error::Invalid)?;
        window._charge = windows._charges.take();
        Ok(window)
    }
}
struct RowPlan {
    line: usize,
    start: Point,
    bytes: usize,
    cells: usize,
    replay: usize,
    allocated: usize,
}
pub struct Cell {
    pub text: String,
    pub matched: bool,
}
pub struct Window {
    pub cells: Vec<Cell>,
    _charge: Option<(Charge, Charge)>,
}
pub(crate) struct Windows {
    pub rows: Vec<Window>,
    _charges: Option<(Charge, Charge)>,
}

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
            crate::xml_worker::project(Arc::new(raw.to_owned()), 7, &AtomicBool::new(false))
                .unwrap()
                .0,
        )
    }
    #[test]
    fn physical_units_do_not_include_inspector_labels_or_dtd_expansion() {
        let s = model(
            "<?xml version=\"1.0\"?><!DOCTYPE p:r SYSTEM 'no-fetch'><p:r xmlns:p='u&amp;v' a='true'>x&amp;<![CDATA[y]]><!-- z --><?go data?></p:r>",
        );
        let cancel = AtomicBool::new(false);
        let ids: Vec<_> = Visits::new(&s, &cancel).map(Result::unwrap).collect();
        let names: Vec<_> = ids
            .iter()
            .filter_map(|id| unit(&s, *id, Domain::QName))
            .collect();
        assert_eq!(names, ["p:r", "xmlns:p", "a", "go"]);
        let elem = ids
            .iter()
            .copied()
            .find(|id| s.node(*id).unwrap().kind() == StructureKind::XmlElement)
            .unwrap();
        assert_eq!(unit(&s, elem, Domain::NamespaceUri), Some("u&v"));
        let ns = ids
            .iter()
            .copied()
            .find(|id| s.node(*id).unwrap().kind() == StructureKind::XmlNamespaceDeclaration)
            .unwrap();
        assert_eq!(unit(&s, ns, Domain::LocalName), Some("p"));
        assert_eq!(unit(&s, ns, Domain::NamespaceUri), None);
        assert_eq!(unit(&s, ns, Domain::ValueDecoded), Some("u&v"));
        let comment = ids
            .iter()
            .copied()
            .find(|id| s.node(*id).unwrap().kind() == StructureKind::XmlComment)
            .unwrap();
        assert_eq!(unit(&s, comment, Domain::ValueDecoded), None);
        assert_eq!(unit(&s, comment, Domain::Metadata), Some(" z "));
        let dtd = ids
            .iter()
            .copied()
            .find(|id| s.node(*id).unwrap().kind() == StructureKind::XmlDocType)
            .unwrap();
        assert!(
            unit(&s, dtd, Domain::Metadata)
                .unwrap()
                .starts_with("<!DOCTYPE")
        );
    }
    #[test]
    fn history_and_window_reservations_release_transactionally() {
        let s = model("<r><a/></r>");
        let cancel = AtomicBool::new(false);
        let mut nav = Navigator::new(s.clone()).unwrap();
        let initial = nav.current();
        let budget = nav.budget().clone();
        for _ in 0..128 {
            nav.commit(initial, &cancel).unwrap();
        }
        assert_eq!(nav.commit(initial, &cancel), Err(Error::History));
        assert_eq!(nav.history_len(), 128);
        assert_eq!(nav.current(), initial);
        for _ in 0..128 {
            assert!(nav.back())
        }
        assert!(!nav.back());
        let pane = Pane::prepare(
            &s,
            State {
                domain: Domain::Inspector,
                ..initial
            },
            &budget,
            &cancel,
        )
        .unwrap();
        assert!(budget.used() < SESSION_BYTES);
        drop(pane);
        drop(nav);
        assert_eq!(budget.used(), 0);
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

#[cfg(test)]
mod unicode_borrowed_integration_tests {
    use super::*;
    fn project(raw: String) -> Arc<Structure> {
        Arc::new(
            crate::xml_worker::project(Arc::new(raw), 9, &AtomicBool::new(false))
                .unwrap()
                .0,
        )
    }
    fn decoded(s: &Structure, value: &str) -> NodeId {
        Visits::new(s, &AtomicBool::new(false))
            .filter_map(Result::ok)
            .find(|id| s.node(*id).unwrap().decoded() == Some(value))
            .unwrap()
    }

    fn row_fixture(value: &str) -> (Arc<Structure>, NodeId) {
        let s = project(format!(
            "<r>{}</r>",
            value.replace('&', "&amp;").replace('<', "&lt;")
        ));
        // XML literal EOL normalization is performed by the existing worker;
        // numeric references would instead create separate reference nodes.
        let normalized = value.replace("\r\n", "\n").replace('\r', "\n");
        let node = decoded(&s, &normalized);
        (s, node)
    }
    #[test]
    fn sequential_short_empty_unicode_rows_preserve_every_first_unit() {
        let cancel = AtomicBool::new(false);
        let fixtures: &[(&str, &[&str])] = &[
            ("ab\ncd\nef", &["ab", "cd", "ef"]),
            ("\nx\ny", &["", "x", "y"]),
            ("a\nc\u{301}d\n🐱x", &["a", "c\u{301}d", "🐱x"]),
            ("ab\r\ncd\r\nef", &["ab", "cd", "ef"]),
            ("ab\n\ncd\n", &["ab", "", "cd", ""]),
        ];
        for &(value, expected) in fixtures {
            let (s, node) = row_fixture(value);
            let nav = Navigator::new(s.clone()).unwrap();
            let mut state = nav.current();
            state.node = node;
            state.domain = Domain::ValueDecoded;
            let pane = Pane::prepare(&s, state, nav.budget(), &cancel).unwrap();
            for top in 0..expected.len() {
                let mut frame = crate::viewport_admission::FrameBudget::new(&cancel);
                let windows = pane
                    .windows(
                        &s,
                        top,
                        expected.len() - top,
                        0,
                        80,
                        None,
                        nav.budget(),
                        &mut frame,
                    )
                    .unwrap();
                let actual: Vec<String> = windows
                    .rows
                    .iter()
                    .map(|r| r.cells.iter().map(|c| c.text.as_str()).collect())
                    .collect();
                assert_eq!(actual, expected[top..], "fixture {value:?} top {top}");
            }
        }
    }
    #[test]
    fn sequential_next_row_zero_width_giant_still_requires_reference() {
        let cancel = AtomicBool::new(false);
        let giant = "\u{301}".repeat(20_000);
        let value = format!("ab\n{giant}\ncd");
        let (s, node) = row_fixture(&value);
        let nav = Navigator::new(s.clone()).unwrap();
        let mut state = nav.current();
        state.node = node;
        state.domain = Domain::ValueDecoded;
        let pane = Pane::prepare(&s, state, nav.budget(), &cancel).unwrap();
        let mut frame = crate::viewport_admission::FrameBudget::new(&cancel);
        let windows = pane
            .windows(&s, 0, 1, 0, 80, None, nav.budget(), &mut frame)
            .unwrap();
        assert_eq!(
            windows.rows[0]
                .cells
                .iter()
                .map(|c| c.text.as_str())
                .collect::<String>(),
            "ab"
        );
        drop(windows);
        let mut frame = crate::viewport_admission::FrameBudget::new(&cancel);
        assert!(matches!(
            pane.windows(&s, 0, 2, 0, 80, None, nav.budget(), &mut frame),
            Err(Error::Reference { start: 3, .. })
        ));
    }
    #[test]
    fn sequential_metadata_piece_boundaries_preserve_labels_and_value() {
        let cancel = AtomicBool::new(false);
        let (s, node) = row_fixture("abc");
        let nav = Navigator::new(s.clone()).unwrap();
        let mut state = nav.current();
        state.node = node;
        state.domain = Domain::Inspector;
        let pane = Pane::prepare(&s, state, nav.budget(), &cancel).unwrap();
        let expected = ["inspector XML", "xml_text", "ValueDecoded:", "abc"];
        let mut frame = crate::viewport_admission::FrameBudget::new(&cancel);
        let windows = pane
            .windows(&s, 0, expected.len(), 0, 80, None, nav.budget(), &mut frame)
            .unwrap();
        let actual: Vec<String> = windows
            .rows
            .iter()
            .map(|r| r.cells.iter().map(|c| c.text.as_str()).collect())
            .collect();
        assert_eq!(actual, expected);
    }
    #[test]
    fn sequential_clipping_does_not_consume_next_row_or_giant() {
        let cancel = AtomicBool::new(false);
        let (s, node) = row_fixture("abcdef\nxyzw\npqrs");
        let nav = Navigator::new(s.clone()).unwrap();
        let mut state = nav.current();
        state.node = node;
        state.domain = Domain::ValueDecoded;
        let pane = Pane::prepare(&s, state, nav.budget(), &cancel).unwrap();
        for (left, width, expected) in [
            (0, 2, vec!["ab", "xy", "pq"]),
            (1, 2, vec!["bc", "yz", "qr"]),
            (4, 2, vec!["ef", "", ""]),
        ] {
            let mut frame = crate::viewport_admission::FrameBudget::new(&cancel);
            let windows = pane
                .windows(&s, 0, 3, left, width, None, nav.budget(), &mut frame)
                .unwrap();
            let actual: Vec<String> = windows
                .rows
                .iter()
                .map(|r| r.cells.iter().map(|c| c.text.as_str()).collect())
                .collect();
            assert_eq!(actual, expected, "left {left} width {width}");
        }
        let giant = format!("a{}", "\u{301}".repeat(20_000));
        let value = format!("ab\ncd\n{giant}");
        let (s, node) = row_fixture(&value);
        let nav = Navigator::new(s.clone()).unwrap();
        let mut state = nav.current();
        state.node = node;
        state.domain = Domain::ValueDecoded;
        let pane = Pane::prepare(&s, state, nav.budget(), &cancel).unwrap();
        let mut frame = crate::viewport_admission::FrameBudget::new(&cancel);
        let windows = pane
            .windows(&s, 0, 2, 0, 80, None, nav.budget(), &mut frame)
            .unwrap();
        assert_eq!(
            windows
                .rows
                .iter()
                .map(|r| r.cells.iter().map(|c| c.text.as_str()).collect::<String>())
                .collect::<Vec<_>>(),
            ["ab", "cd"]
        );
        drop(windows);
        let mut frame = crate::viewport_admission::FrameBudget::new(&cancel);
        assert!(matches!(
            pane.windows(&s, 0, 3, 0, 80, None, nav.budget(), &mut frame),
            Err(Error::Reference { start: 6, .. })
        ));
    }
    #[test]
    fn giant_decoded_piece_is_complete_and_reference_has_enclosing_provenance() {
        let cancel = AtomicBool::new(false);
        let giant = format!("a{}", "\u{301}".repeat(16_000_000));
        let s = project(format!("<r>{giant}</r>"));
        let node = decoded(&s, &giant);
        let nav = Navigator::new(s.clone()).unwrap();
        let mut state = nav.current();
        state.node = node;
        state.domain = Domain::ValueDecoded;
        let baseline = nav.budget().used();
        let pane = Pane::prepare(&s, state, nav.budget(), &cancel).unwrap();
        assert_eq!(pane.position(&s, 16000, &cancel).unwrap(), (0, 0));
        let mut frame = crate::viewport_admission::FrameBudget::new(&cancel);
        let error = match pane.windows(&s, 0, 1, 0, 80, None, nav.budget(), &mut frame) {
            Err(error) => error,
            Ok(_) => panic!("giant borrowed piece must never be copied to a symbol"),
        };
        let reference = pane.reference(&s, error).unwrap();
        assert_eq!(reference.range, 0..giant.len());
        assert!(matches!(
            reference.source,
            crate::viewport_admission::SourceRelation::EnclosingNodeSpan(_)
        ));
        assert_eq!(s.node(node).unwrap().decoded(), Some(giant.as_str()));
        drop(pane);
        assert_eq!(nav.budget().used(), baseline);
    }
    #[test]
    fn normal_multirow_window_is_admitted_as_one_frame_and_rejects_foreign_model() {
        let cancel = AtomicBool::new(false);
        let normal = format!("{}\n", "a".repeat(80)).repeat(24);
        let s = project(format!("<r>{normal}</r>"));
        let node = decoded(&s, &normal);
        let nav = Navigator::new(s.clone()).unwrap();
        let mut state = nav.current();
        state.node = node;
        state.domain = Domain::ValueDecoded;
        let pane = Pane::prepare(&s, state, nav.budget(), &cancel).unwrap();
        let baseline = nav.budget().used();
        assert!(pane.reference(&s, Error::Window).is_none());
        assert!(pane.reference(&s, Error::Budget).is_none());
        assert!(matches!(
            pane.reference(&s, Error::FrameSymbols).unwrap().reason,
            crate::viewport_admission::Reason::FrameSymbols
        ));
        let mut frame = crate::viewport_admission::FrameBudget::new(&cancel);
        let windows = pane
            .windows(&s, 0, 24, 0, 80, None, nav.budget(), &mut frame)
            .unwrap();
        assert_eq!(windows.rows.len(), 24);
        for row in &windows.rows {
            assert_eq!(
                row.cells
                    .iter()
                    .map(|cell| cell.text.as_str())
                    .collect::<String>(),
                "a".repeat(80)
            );
        }
        drop(windows);
        assert_eq!(nav.budget().used(), baseline);
        let foreign = crate::xml_worker::project(s.source_handle(), 9, &cancel)
            .unwrap()
            .0;
        assert!(matches!(
            pane.position(&foreign, 0, &cancel),
            Err(Error::Invalid)
        ));
    }
}
