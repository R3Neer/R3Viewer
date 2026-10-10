//! Private TOML1.1 one-shot worker. Input is a source snapshot, never a path.
use crate::{
    structure::StructureFailure,
    structure_model::*,
    structure_protocol::{self as ipc},
};
use std::{
    iter::Peekable,
    ops::Range,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use toml_edit::{Document, Item, Key, Table, Value};
use toml_parser::{
    Source,
    lexer::{Lexer, Token, TokenKind as K},
};
#[derive(Debug, Clone)]
struct SharedSource(Arc<String>);
impl AsRef<str> for SharedSource {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
fn failure(error: ModelError) -> StructureFailure {
    match error {
        ModelError::Cancelled => StructureFailure::WorkerExited,
        ModelError::Timeout => StructureFailure::Timeout,
        ModelError::SourceLimit => StructureFailure::SourceBudget,
        ModelError::Budget => StructureFailure::ArenaBudget,
        ModelError::Depth => StructureFailure::Depth,
        ModelError::Invalid => StructureFailure::InvalidIpc,
    }
}
fn invalid() -> StructureFailure {
    StructureFailure::InvalidIpc
}
fn location(range: Range<usize>) -> Result<SourceSpan, StructureFailure> {
    Ok(SourceSpan {
        start: u32::try_from(range.start).map_err(|_| invalid())?,
        end: u32::try_from(range.end).map_err(|_| invalid())?,
    })
}
fn checked(
    builder: &StructureBuilder,
    scratch: usize,
    cancel: &AtomicBool,
) -> Result<(), StructureFailure> {
    if cancel.load(Ordering::Acquire) {
        return Err(StructureFailure::WorkerExited);
    }
    if builder
        .persistent_bytes()
        .checked_add(scratch)
        .is_none_or(|n| n > MAX_STRUCTURE_BYTES)
    {
        return Err(StructureFailure::ArenaBudget);
    }
    Ok(())
}
fn interpreted(
    builder: &mut StructureBuilder,
    raw: &str,
    cooked: &str,
    origin: SourceSpan,
    scratch: usize,
    cancel: &AtomicBool,
) -> Result<TextRef, StructureFailure> {
    let whole = origin.range();
    let mut candidates = [0..0, 0..0, 0..0, 0..0, 0..0];
    let mut used = 0;
    let mut candidate = |range| {
        candidates[used] = range;
        used += 1;
    };
    candidate(whole.clone());
    for quotes in [1, 3] {
        if whole.len() >= quotes * 2 {
            let start = whole.start + quotes;
            let end = whole.end - quotes;
            if raw.is_char_boundary(start) && raw.is_char_boundary(end) {
                candidate(start..end);
                if raw[start..end].starts_with("\r\n") {
                    candidate(start + 2..end);
                } else if raw[start..end].starts_with('\n') {
                    candidate(start + 1..end);
                }
            }
        }
    }
    for range in candidates.into_iter().take(used) {
        if raw.get(range.clone()) == Some(cooked) {
            return Ok(TextRef::Source(location(range)?));
        }
    }
    let mut begin = None;
    let mut end = 0;
    for bytes in cooked.as_bytes().chunks(ipc::CHUNK) {
        checked(builder, scratch, cancel)?;
        builder.set_scratch_bytes(scratch).map_err(failure)?;
        let range = builder.append_cooked(bytes).map_err(failure)?;
        begin.get_or_insert(range.start);
        end = range.end;
        checked(builder, scratch, cancel)?;
    }
    // Empty interpreted values need a valid empty cooked interval too.
    let start = if let Some(start) = begin {
        start
    } else {
        let range = builder.append_cooked(&[]).map_err(failure)?;
        end = range.end;
        range.start
    };
    Ok(TextRef::Cooked { start, end })
}
fn push(
    builder: &mut StructureBuilder,
    spec: NodeSpec,
    scratch: usize,
    cancel: &AtomicBool,
) -> Result<NodeId, StructureFailure> {
    checked(builder, scratch, cancel)?;
    builder.set_scratch_bytes(scratch).map_err(failure)?;
    let id = builder.push_node(spec).map_err(failure)?;
    checked(builder, scratch, cancel)?;
    Ok(id)
}
fn key_node(
    builder: &mut StructureBuilder,
    raw: &str,
    key: &Key,
    scratch: usize,
    cancel: &AtomicBool,
) -> Result<NodeId, StructureFailure> {
    let origin = location(key.span().ok_or_else(invalid)?)?;
    let text = interpreted(builder, raw, key.get(), origin, scratch, cancel)?;
    push(
        builder,
        NodeSpec {
            kind: StructureKind::String,
            text: Some(text),
            location: Some(origin),
        },
        scratch,
        cancel,
    )
}
fn entries_buffer(
    count: usize,
    builder: &StructureBuilder,
    scratch: usize,
) -> Result<(Vec<EntrySpec>, usize), StructureFailure> {
    let bytes = count
        .checked_mul(std::mem::size_of::<EntrySpec>())
        .ok_or(StructureFailure::ArenaBudget)?;
    if builder
        .persistent_bytes()
        .checked_add(scratch)
        .and_then(|n| n.checked_add(bytes))
        .is_none_or(|n| n > MAX_STRUCTURE_BYTES)
    {
        return Err(StructureFailure::ArenaBudget);
    }
    let mut entries = Vec::new();
    entries
        .try_reserve_exact(count)
        .map_err(|_| StructureFailure::ArenaBudget)?;
    let held = scratch + entries.capacity() * std::mem::size_of::<EntrySpec>();
    Ok((entries, held))
}
fn table(
    builder: &mut StructureBuilder,
    raw: &str,
    table: &Table,
    depth: usize,
    scratch: usize,
    cancel: &AtomicBool,
) -> Result<NodeId, StructureFailure> {
    if depth > MAX_STRUCTURE_DEPTH {
        return Err(StructureFailure::Depth);
    }
    let id = push(
        builder,
        NodeSpec {
            kind: StructureKind::Mapping,
            text: None,
            location: None,
        },
        scratch,
        cancel,
    )?;
    let (mut entries, held) = entries_buffer(table.len(), builder, scratch)?;
    for (name, item) in table.iter() {
        let key = key_node(
            builder,
            raw,
            table.key(name).ok_or_else(invalid)?,
            held,
            cancel,
        )?;
        let value = item_node(builder, raw, item, depth + 1, held, cancel)?;
        entries.push(EntrySpec {
            key: Some(key),
            value,
        });
    }
    builder.set_scratch_bytes(held).map_err(failure)?;
    builder.set_entries(id, entries).map_err(failure)?;
    builder.set_scratch_bytes(scratch).map_err(failure)?;
    checked(builder, scratch, cancel)?;
    Ok(id)
}
fn item_node(
    builder: &mut StructureBuilder,
    raw: &str,
    item: &Item,
    depth: usize,
    scratch: usize,
    cancel: &AtomicBool,
) -> Result<NodeId, StructureFailure> {
    if depth > MAX_STRUCTURE_DEPTH {
        return Err(StructureFailure::Depth);
    }
    match item {
        Item::Table(t) => table(builder, raw, t, depth, scratch, cancel),
        Item::Value(v) => value(builder, raw, v, depth, scratch, cancel),
        Item::ArrayOfTables(a) => {
            let id = push(
                builder,
                NodeSpec {
                    kind: StructureKind::ArrayOfTables,
                    text: None,
                    location: None,
                },
                scratch,
                cancel,
            )?;
            let (mut entries, held) = entries_buffer(a.len(), builder, scratch)?;
            for t in a.iter() {
                entries.push(EntrySpec {
                    key: None,
                    value: table(builder, raw, t, depth + 1, held, cancel)?,
                });
            }
            builder.set_scratch_bytes(held).map_err(failure)?;
            builder.set_entries(id, entries).map_err(failure)?;
            builder.set_scratch_bytes(scratch).map_err(failure)?;
            checked(builder, scratch, cancel)?;
            Ok(id)
        }
        Item::None => Err(invalid()),
    }
}
fn value(
    builder: &mut StructureBuilder,
    raw: &str,
    v: &Value,
    depth: usize,
    scratch: usize,
    cancel: &AtomicBool,
) -> Result<NodeId, StructureFailure> {
    if depth > MAX_STRUCTURE_DEPTH {
        return Err(StructureFailure::Depth);
    }
    match v {
        Value::Array(a) => {
            let span = location(a.span().ok_or_else(invalid)?)?;
            let id = push(
                builder,
                NodeSpec {
                    kind: StructureKind::Sequence,
                    text: None,
                    location: Some(span),
                },
                scratch,
                cancel,
            )?;
            let (mut entries, held) = entries_buffer(a.len(), builder, scratch)?;
            for v in a.iter() {
                entries.push(EntrySpec {
                    key: None,
                    value: value(builder, raw, v, depth + 1, held, cancel)?,
                });
            }
            builder.set_scratch_bytes(held).map_err(failure)?;
            builder.set_entries(id, entries).map_err(failure)?;
            builder.set_scratch_bytes(scratch).map_err(failure)?;
            checked(builder, scratch, cancel)?;
            Ok(id)
        }
        Value::InlineTable(t) => {
            let origin = if t.is_dotted() {
                None
            } else {
                Some(location(t.span().ok_or_else(invalid)?)?)
            };
            let id = push(
                builder,
                NodeSpec {
                    kind: StructureKind::Mapping,
                    text: None,
                    location: origin,
                },
                scratch,
                cancel,
            )?;
            let (mut entries, held) = entries_buffer(t.len(), builder, scratch)?;
            for (name, v) in t.iter() {
                let key = key_node(builder, raw, t.key(name).ok_or_else(invalid)?, held, cancel)?;
                entries.push(EntrySpec {
                    key: Some(key),
                    value: value(builder, raw, v, depth + 1, held, cancel)?,
                });
            }
            builder.set_scratch_bytes(held).map_err(failure)?;
            builder.set_entries(id, entries).map_err(failure)?;
            builder.set_scratch_bytes(scratch).map_err(failure)?;
            checked(builder, scratch, cancel)?;
            Ok(id)
        }
        _ => {
            let origin = location(v.span().ok_or_else(invalid)?)?;
            let kind = match v {
                Value::String(_) => StructureKind::String,
                Value::Boolean(_) => StructureKind::Boolean,
                Value::Integer(_) => StructureKind::Integer,
                Value::Float(_) => StructureKind::Float,
                Value::Datetime(d) => {
                    let d = d.value();
                    match (d.date.is_some(), d.time.is_some(), d.offset.is_some()) {
                        (true, true, true) => StructureKind::OffsetDateTime,
                        (true, true, false) => StructureKind::LocalDateTime,
                        (true, false, false) => StructureKind::LocalDate,
                        (false, true, false) => StructureKind::LocalTime,
                        _ => return Err(invalid()),
                    }
                }
                _ => return Err(invalid()),
            };
            let text = if let Value::String(s) = v {
                interpreted(builder, raw, s.value(), origin, scratch, cancel)?
            } else {
                TextRef::Source(origin)
            };
            push(
                builder,
                NodeSpec {
                    kind,
                    text: Some(text),
                    location: Some(origin),
                },
                scratch,
                cancel,
            )
        }
    }
}
struct Decl {
    spec: DeclarationSpec,
    parts: Vec<DeclarationPart>,
}
type ScanResult = (Vec<Decl>, Vec<Option<SourceSpan>>, usize, usize);
struct Scanner<'a> {
    raw: &'a str,
    tokens: Peekable<Lexer<'a>>,
    model: &'a Structure,
    selected: Vec<(NodeId, usize)>,
    transient_bytes: usize,
    declarations: Vec<Decl>,
    part_bytes: usize,
    syntax: usize,
    key_max: usize,
    cancel: &'a AtomicBool,
    first: Vec<Option<SourceSpan>>,
}
fn token_span(t: Token) -> Result<SourceSpan, StructureFailure> {
    location(t.span().start()..t.span().end())
}
impl<'a> Scanner<'a> {
    fn bytes(&self) -> usize {
        self.model.persistent_bytes()
            + self.first.capacity() * std::mem::size_of::<Option<SourceSpan>>()
            + self.declarations.capacity() * std::mem::size_of::<Decl>()
            + self.part_bytes
            + self.transient_bytes
            + self.selected.capacity() * std::mem::size_of::<(NodeId, usize)>()
    }
    fn budget(&self) -> Result<(), StructureFailure> {
        if self.cancel.load(Ordering::Acquire) {
            return Err(StructureFailure::WorkerExited);
        }
        let bytes = self.bytes();
        if bytes > MAX_STRUCTURE_BYTES {
            return Err(StructureFailure::ArenaBudget);
        }
        Ok(())
    }
    fn skip(&mut self) {
        while self
            .tokens
            .peek()
            .is_some_and(|t| matches!(t.kind(), K::Whitespace | K::Comment | K::Newline))
        {
            self.tokens.next();
        }
    }
    fn whitespace(&mut self) {
        while self
            .tokens
            .peek()
            .is_some_and(|t| t.kind() == K::Whitespace)
        {
            self.tokens.next();
        }
    }
    fn take(&mut self, k: K) -> Result<Token, StructureFailure> {
        let t = self.tokens.next().ok_or_else(invalid)?;
        if t.kind() != k {
            return Err(invalid());
        }
        Ok(t)
    }
    fn keys(&mut self, stop: K) -> Result<(SourceSpan, Vec<(Key, SourceSpan)>), StructureFailure> {
        self.whitespace();
        let start = self.tokens.peek().ok_or_else(invalid)?.span().start();
        let mut end = start;
        let mut parts = Vec::new();
        let mut key_bytes = 0;
        let previous_transient = self.transient_bytes;
        while let Some(t) = self.tokens.peek().copied() {
            if t.kind() == stop {
                break;
            }
            self.tokens.next();
            if t.kind() == K::Whitespace {
                continue;
            }
            end = t.span().end();
            if t.kind() == K::Dot {
                continue;
            }
            if !matches!(t.kind(), K::Atom | K::BasicString | K::LiteralString) {
                return Err(invalid());
            }
            if parts.len() >= MAX_STRUCTURE_DEPTH {
                return Err(StructureFailure::Depth);
            }
            let raw = token_span(t)?;
            // Key parsing is temporary, bounded conservatively by twice its raw bytes.
            let charged = key_bytes + raw.range().len().saturating_mul(2);
            let other = self.bytes() - self.transient_bytes + previous_transient + charged;
            reserve(&mut parts, 1, other).map_err(failure)?;
            self.transient_bytes = previous_transient
                + charged
                + parts.capacity() * std::mem::size_of::<(Key, SourceSpan)>();
            self.budget()?;
            key_bytes = charged;
            let key = self.raw[raw.range()]
                .parse::<Key>()
                .map_err(|_| invalid())?;
            parts.push((key, raw));
        }
        self.key_max = self.key_max.max(parts.len());
        if parts.is_empty() {
            return Err(invalid());
        }
        Ok((location(start..end)?, parts))
    }
    fn resolve(
        &mut self,
        mut base: NodeId,
        keys: Vec<(Key, SourceSpan)>,
        new_aot: bool,
    ) -> Result<(NodeId, Vec<DeclarationPart>), StructureFailure> {
        let count = keys.len();
        let key_scratch = keys.capacity() * std::mem::size_of::<(Key, SourceSpan)>()
            + keys
                .iter()
                .map(|(_, span)| span.range().len() * 2)
                .sum::<usize>();
        let mut parts = Vec::new();
        reserve(&mut parts, count, self.bytes()).map_err(failure)?;
        self.transient_bytes += parts.capacity() * std::mem::size_of::<DeclarationPart>();
        for (i, (key, span)) in keys.into_iter().enumerate() {
            let parent = self.model.node(base).ok_or_else(invalid)?;
            if parent.kind() != StructureKind::Mapping {
                return Err(invalid());
            }
            let child = parent
                .entries()
                .find(|e| {
                    e.key()
                        .and_then(|k| k.leaf())
                        .is_some_and(|k| k.text() == key.get())
                })
                .ok_or_else(invalid)?
                .value();
            base = child.id();
            parts.push(DeclarationPart { span, node: base });
            if child.kind() == StructureKind::ArrayOfTables {
                let selected = if new_aot && i + 1 == count {
                    let next = self
                        .selected
                        .iter()
                        .find(|(id, _)| *id == base)
                        .map_or(0, |(_, n)| n + 1);
                    if let Some((_, n)) = self.selected.iter_mut().find(|(id, _)| *id == base) {
                        *n = next;
                    } else {
                        let other = self.bytes()
                            - self.selected.capacity() * std::mem::size_of::<(NodeId, usize)>();
                        reserve(&mut self.selected, 1, other).map_err(failure)?;
                        self.selected.push((base, next));
                    }
                    next
                } else {
                    self.selected
                        .iter()
                        .find(|(id, _)| *id == base)
                        .ok_or_else(invalid)?
                        .1
                };
                base = child.entry(selected).ok_or_else(invalid)?.value().id();
            }
        }
        self.transient_bytes -= key_scratch;
        Ok((base, parts))
    }
    fn add(
        &mut self,
        kind: DeclarationKind,
        key: SourceSpan,
        value: Option<SourceSpan>,
        node: NodeId,
        parts: Vec<DeclarationPart>,
        span: SourceSpan,
    ) -> Result<(), StructureFailure> {
        self.budget()?;
        for part in &parts {
            let current = &mut self.first[part.node.0 as usize];
            if current.is_none_or(|old| part.span.start < old.start) {
                *current = Some(part.span);
            }
        }
        let other = self.bytes() - self.declarations.capacity() * std::mem::size_of::<Decl>();
        reserve(&mut self.declarations, 1, other).map_err(failure)?;
        self.part_bytes += parts.capacity() * std::mem::size_of::<DeclarationPart>();
        self.transient_bytes -= parts.capacity() * std::mem::size_of::<DeclarationPart>();
        self.declarations.push(Decl {
            spec: DeclarationSpec {
                kind,
                span,
                key_span: key,
                value_span: value,
                node,
            },
            parts,
        });
        self.budget()
    }
    fn value(&mut self, node: NodeId, depth: usize) -> Result<SourceSpan, StructureFailure> {
        self.budget()?;
        self.whitespace();
        let first = *self.tokens.peek().ok_or_else(invalid)?;
        let view = self.model.node(node).ok_or_else(invalid)?;
        let span = match first.kind() {
            K::LeftSquareBracket => {
                if depth >= MAX_STRUCTURE_DEPTH {
                    return Err(StructureFailure::Depth);
                }
                self.syntax = self.syntax.max(depth + 1);
                let start = self.take(K::LeftSquareBracket)?.span().start();
                let mut i = 0;
                loop {
                    self.skip();
                    if self.tokens.peek().ok_or_else(invalid)?.kind() == K::RightSquareBracket {
                        break;
                    }
                    let child = view.entry(i).ok_or_else(invalid)?.value().id();
                    self.value(child, depth + 1)?;
                    i += 1;
                    self.skip();
                    if self.tokens.peek().ok_or_else(invalid)?.kind() == K::Comma {
                        self.take(K::Comma)?;
                    } else {
                        break;
                    }
                }
                if i != view.entries().len() {
                    return Err(invalid());
                }
                let end = self.take(K::RightSquareBracket)?.span().end();
                location(start..end)?
            }
            K::LeftCurlyBracket => {
                if depth >= MAX_STRUCTURE_DEPTH {
                    return Err(StructureFailure::Depth);
                }
                self.syntax = self.syntax.max(depth + 1);
                let start = self.take(K::LeftCurlyBracket)?.span().start();
                loop {
                    self.skip();
                    if self.tokens.peek().ok_or_else(invalid)?.kind() == K::RightCurlyBracket {
                        break;
                    }
                    let (key, keys) = self.keys(K::Equals)?;
                    self.take(K::Equals)?;
                    let (child, parts) = self.resolve(node, keys, false)?;
                    let value = self.value(child, depth + 1)?;
                    self.add(
                        DeclarationKind::InlineAssignment,
                        key,
                        Some(value),
                        child,
                        parts,
                        SourceSpan {
                            start: key.start,
                            end: value.end,
                        },
                    )?;
                    self.skip();
                    if self.tokens.peek().ok_or_else(invalid)?.kind() == K::Comma {
                        self.take(K::Comma)?;
                    } else {
                        break;
                    }
                }
                let end = self.take(K::RightCurlyBracket)?.span().end();
                location(start..end)?
            }
            _ => {
                let start = first.span().start();
                let mut end = start;
                while let Some(t) = self.tokens.peek().copied() {
                    if matches!(
                        t.kind(),
                        K::Comment
                            | K::Newline
                            | K::Comma
                            | K::RightSquareBracket
                            | K::RightCurlyBracket
                            | K::Eof
                    ) {
                        break;
                    }
                    self.tokens.next();
                    if t.kind() != K::Whitespace {
                        end = t.span().end();
                    }
                }
                location(start..end)?
            }
        };
        if view.source_location() != Some(span) {
            return Err(invalid());
        }
        Ok(span)
    }
    fn scan(mut self) -> Result<ScanResult, StructureFailure> {
        let mut context = self.model.root().id();
        loop {
            self.budget()?;
            self.skip();
            let token = *self.tokens.peek().ok_or_else(invalid)?;
            if token.kind() == K::Eof {
                break;
            }
            if token.kind() == K::LeftSquareBracket {
                let start = self.take(K::LeftSquareBracket)?.span().start();
                let aot = self.tokens.peek().ok_or_else(invalid)?.kind() == K::LeftSquareBracket;
                if aot {
                    self.take(K::LeftSquareBracket)?;
                }
                let (key, keys) = self.keys(K::RightSquareBracket)?;
                let mut end = self.take(K::RightSquareBracket)?.span().end();
                if aot {
                    end = self.take(K::RightSquareBracket)?.span().end();
                }
                let (node, parts) = self.resolve(self.model.root().id(), keys, aot)?;
                if self.model.node(node).ok_or_else(invalid)?.kind() != StructureKind::Mapping {
                    return Err(invalid());
                }
                context = node;
                self.add(
                    if aot {
                        DeclarationKind::ArrayOfTablesHeader
                    } else {
                        DeclarationKind::Header
                    },
                    key,
                    None,
                    node,
                    parts,
                    location(start..end)?,
                )?;
            } else {
                let (key, keys) = self.keys(K::Equals)?;
                self.take(K::Equals)?;
                let (node, parts) = self.resolve(context, keys, false)?;
                let value = self.value(node, 0)?;
                self.add(
                    DeclarationKind::Assignment,
                    key,
                    Some(value),
                    node,
                    parts,
                    SourceSpan {
                        start: key.start,
                        end: value.end,
                    },
                )?;
            }
            self.whitespace();
            if self.tokens.peek().is_some_and(|t| t.kind() == K::Comment) {
                self.tokens.next();
            }
            if !self
                .tokens
                .peek()
                .is_some_and(|t| matches!(t.kind(), K::Newline | K::Eof))
            {
                return Err(invalid());
            }
        }
        self.declarations
            .sort_unstable_by_key(|d| d.spec.span.start);
        Ok((self.declarations, self.first, self.syntax, self.key_max))
    }
}
pub(crate) fn project(
    source: Arc<String>,
    generation: u64,
    cancel: &AtomicBool,
) -> Result<(Structure, usize, usize), StructureFailure> {
    let document =
        Document::parse(SharedSource(source.clone())).map_err(|_| StructureFailure::InvalidToml)?;
    let mut builder = StructureBuilder::new(source.clone(), generation).map_err(failure)?;
    let root = table(&mut builder, &source, document.as_table(), 0, 0, cancel)?;
    let model = builder.finish(root, 0, 0, cancel).map_err(failure)?;
    let mut first = Vec::new();
    reserve(&mut first, model.node_count(), model.persistent_bytes()).map_err(failure)?;
    first.resize(model.node_count(), None);
    let scanner = Scanner {
        raw: &source,
        tokens: Source::new(&source).lex().peekable(),
        model: &model,
        selected: Vec::new(),
        transient_bytes: 0,
        declarations: Vec::new(),
        part_bytes: 0,
        syntax: 0,
        key_max: 0,
        cancel,
        first,
    };
    let (declarations, first, syntax, key_max) = scanner.scan()?;
    let mut builder = StructureBuilder::from_structure(model);
    let mut scratch = declarations.capacity() * std::mem::size_of::<Decl>()
        + declarations
            .iter()
            .map(|d| d.parts.capacity() * std::mem::size_of::<DeclarationPart>())
            .sum::<usize>();
    builder
        .set_scratch_bytes(scratch + first.capacity() * std::mem::size_of::<Option<SourceSpan>>())
        .map_err(failure)?;
    builder.order_mappings(&first).map_err(failure)?;
    drop(first);
    for declaration in declarations {
        checked(&builder, scratch, cancel)?;
        builder.set_scratch_bytes(scratch).map_err(failure)?;
        let consumed = declaration.parts.capacity() * std::mem::size_of::<DeclarationPart>();
        builder
            .push_declaration(declaration.spec, declaration.parts)
            .map_err(failure)?;
        scratch -= consumed;
    }
    builder.set_scratch_bytes(0).map_err(failure)?;
    let model = builder
        .finish(root, syntax, key_max, cancel)
        .map_err(failure)?;
    Ok((model, syntax, key_max))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn physical_headers_order_and_multiline_are_complete() {
        let raw = "root=1\r\n[a.b]\r\nx=\"\"\"\r\n[[fake]]\r\n\"\"\"\r\n[empty]\r\n[a]\r\ny=2";
        let (model, _, _) = project(Arc::new(raw.to_owned()), 3, &AtomicBool::new(false)).unwrap();
        let keys = model
            .root()
            .entries()
            .map(|e| e.key().unwrap().leaf().unwrap().text())
            .collect::<Vec<_>>();
        assert_eq!(keys, ["root", "a", "empty"]);
        let texts = model
            .declarations()
            .map(|d| &raw[d.source_location().range()])
            .collect::<Vec<_>>();
        assert_eq!(texts.len(), 6);
        assert_eq!(texts[1], "[a.b]");
        assert_eq!(texts[3], "[empty]");
        assert!(
            model
                .root()
                .entry(1)
                .unwrap()
                .value()
                .source_location()
                .is_none()
        );
    }
    #[test]
    fn aot_ordinal_and_inline_pairs_have_original_locations() {
        let raw = "[[a]]\nx={v=1}\n[[a]]\nx={v=2}";
        let (model, _, _) = project(Arc::new(raw.to_owned()), 1, &AtomicBool::new(false)).unwrap();
        let a = model.root().entry(0).unwrap().value();
        assert_eq!(a.entries().len(), 2);
        let ids = model
            .declarations()
            .filter(|d| d.kind() == DeclarationKind::ArrayOfTablesHeader)
            .map(|d| d.node().id())
            .collect::<Vec<_>>();
        assert_eq!(
            ids,
            [
                a.entry(0).unwrap().value().id(),
                a.entry(1).unwrap().value().id()
            ]
        );
        assert_eq!(model.declarations().len(), 6);
    }
    #[test]
    fn raw_numbers_dates_and_decoded_strings_have_distinct_authority() {
        let raw = "number=1.234567890123456789\ntime=07:32:00.123456789012\ntext=\"\\u754c\"";
        let (model, _, _) = project(Arc::new(raw.to_owned()), 1, &AtomicBool::new(false)).unwrap();
        let leaves = model
            .root()
            .entries()
            .map(|e| e.value().leaf().unwrap().text())
            .collect::<Vec<_>>();
        assert_eq!(
            leaves,
            ["1.234567890123456789", "07:32:00.123456789012", "界"]
        );
        assert_eq!(
            &raw[model
                .root()
                .entry(2)
                .unwrap()
                .value()
                .source_location()
                .unwrap()
                .range()],
            "\"\\u754c\""
        );
    }
    #[test]
    fn explicit_64_and_65_logical_limits() {
        for depth in [64, 65] {
            let key = (0..depth)
                .map(|i| format!("k{i}"))
                .collect::<Vec<_>>()
                .join(".");
            for raw in [format!("{key}=0"), format!("[{key}]")] {
                let result = project(Arc::new(raw), 1, &AtomicBool::new(false));
                assert_eq!(result.is_ok(), depth == 64);
            }
        }
    }
}
