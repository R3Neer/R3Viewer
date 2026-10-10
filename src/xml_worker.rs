//! Source-only XML 1.0 inspection. DTD bodies remain opaque and inert.
#[path = "xml/lexical.rs"]
mod lexical;
#[path = "xml/profile.rs"]
mod profile;
use crate::{structure::StructureFailure, structure_model::*};
use lexical::{Control, Cursor, Kind, Result as XResult, Span, is_s, reserve};
use profile::{attrs, cooked_each, declaration, doctype, nc_name, qname, reference};
use std::{
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hasher},
    mem::size_of,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
const XML: &str = "http://www.w3.org/XML/1998/namespace";
const XMLNS: &str = "http://www.w3.org/2000/xmlns/";
const HEADROOM: usize = 4 * crate::structure_protocol::MAX_FRAME;
fn span(p: Span) -> SourceSpan {
    SourceSpan {
        start: p.start as u32,
        end: p.end as u32,
    }
}
fn raw(s: &str, p: Span) -> &str {
    &s[p.start..p.end]
}
// In-place heapsort has explicit cancellation checkpoints and no hidden scratch allocation.
fn checked_cmp(a: &str, b: &str, c: &mut Control) -> XResult<std::cmp::Ordering> {
    for (a, b) in a.as_bytes().chunks(4096).zip(b.as_bytes().chunks(4096)) {
        c.poll()?;
        let order = a.cmp(b);
        if order != std::cmp::Ordering::Equal {
            return Ok(order);
        }
    }
    Ok(a.len().cmp(&b.len()))
}
fn checked_sort<T: Copy>(
    items: &mut [T],
    c: &mut Control,
    mut cmp: impl FnMut(&T, &T, &mut Control) -> XResult<std::cmp::Ordering>,
) -> XResult<()> {
    fn down<T: Copy>(
        v: &mut [T],
        mut at: usize,
        end: usize,
        c: &mut Control,
        cmp: &mut impl FnMut(&T, &T, &mut Control) -> XResult<std::cmp::Ordering>,
    ) -> XResult<()> {
        loop {
            c.poll()?;
            let left = at * 2 + 1;
            if left >= end {
                return Ok(());
            }
            let mut child = left;
            if left + 1 < end && cmp(&v[left], &v[left + 1], c)?.is_lt() {
                child += 1
            }
            if !cmp(&v[at], &v[child], c)?.is_lt() {
                return Ok(());
            }
            v.swap(at, child);
            at = child;
        }
    }
    let len = items.len();
    for at in (0..len / 2).rev() {
        down(items, at, len, c, &mut cmp)?
    }
    for end in (1..len).rev() {
        items.swap(0, end);
        down(items, 0, end, c, &mut cmp)?
    }
    Ok(())
}
fn admit_item(nodes: usize, edges: usize, parent: bool) -> XResult<()> {
    if nodes
        .checked_add(edges)
        .and_then(|n| n.checked_add(1 + usize::from(parent)))
        .is_none_or(|n| n > MAX_STRUCTURE_ITEMS)
    {
        Err("Budget")
    } else {
        Ok(())
    }
}
fn only_s(s: &str, c: &mut Control) -> XResult<bool> {
    for b in s.as_bytes().chunks(4096) {
        c.poll()?;
        if !b.iter().copied().all(is_s) {
            return Ok(false);
        }
    }
    Ok(true)
}
fn contains_checked(s: &str, needle: &[u8], c: &mut Control) -> XResult<bool> {
    let bytes = s.as_bytes();
    for start in (0..bytes.len()).step_by(4096) {
        c.poll()?;
        let end = (start + 4096 + needle.len() - 1).min(bytes.len());
        if bytes[start..end].windows(needle.len()).any(|b| b == needle) {
            return Ok(true);
        }
    }
    Ok(false)
}
fn failure(e: &str) -> StructureFailure {
    match e {
        "Cancelled" => StructureFailure::WorkerExited,
        "Timeout" => StructureFailure::Timeout,
        "Budget" => StructureFailure::ArenaBudget,
        "Depth" => StructureFailure::Depth,
        "UnsupportedXmlVersion" => StructureFailure::UnsupportedXmlVersion,
        "UnsupportedXmlEncoding" => StructureFailure::UnsupportedXmlEncoding,
        "UnsupportedXmlEntity" => StructureFailure::UnsupportedXmlEntity,
        "XmlRepresentation" => StructureFailure::XmlRepresentation,
        _ => StructureFailure::InvalidXml,
    }
}
fn model_error(e: ModelError) -> &'static str {
    match e {
        ModelError::Cancelled => "Cancelled",
        ModelError::Timeout => "Timeout",
        ModelError::Budget | ModelError::SourceLimit => "Budget",
        ModelError::Depth => "Depth",
        ModelError::Invalid => "XmlRepresentation",
    }
}
enum Target<'a> {
    Build(&'a mut StructureBuilder),
    Verify(&'a Structure),
}
struct Sink<'a> {
    target: Target<'a>,
    next: u32,
    relations: Vec<(NodeId, NodeId)>,
}
impl Sink<'_> {
    fn base(&self) -> usize {
        match &self.target {
            Target::Build(b) => b.persistent_bytes(),
            Target::Verify(m) => m.persistent_bytes(),
        }
    }
    fn sync(&mut self, c: &mut Control) -> XResult<()> {
        c.base = self.base() + HEADROOM;
        if c.base
            .checked_add(c.own)
            .is_none_or(|n| n > MAX_STRUCTURE_BYTES)
        {
            return Err("Budget");
        }
        if let Target::Build(b) = &mut self.target {
            b.set_scratch_bytes(c.own + HEADROOM).map_err(model_error)?
        }
        Ok(())
    }
    fn text(&self, t: TextRef) -> &str {
        match &self.target {
            Target::Build(b) => b.xml_text(t),
            Target::Verify(m) => m.text_ref(t),
        }
    }
    fn norm(
        &mut self,
        s: &str,
        p: Span,
        attribute: bool,
        refs: bool,
        c: &mut Control,
    ) -> XResult<TextRef> {
        c.poll()?;
        if let Target::Verify(m) = &self.target {
            let text = m
                .node(NodeId(self.next))
                .and_then(|n| n.decoded())
                .ok_or("XmlRepresentation")?;
            let mut chars = text.chars();
            cooked_each(raw(s, p), attribute, refs, c, |x| {
                if chars.next() == Some(x) {
                    Ok(())
                } else {
                    Err("XmlRepresentation")
                }
            })?;
            if chars.next().is_some() {
                return Err("XmlRepresentation");
            }
            return m
                .node(NodeId(self.next))
                .and_then(|n| n.raw_text_ref())
                .ok_or("XmlRepresentation");
        }
        let mut count = 0usize;
        let mut identical = true;
        let mut original = raw(s, p).chars();
        cooked_each(raw(s, p), attribute, refs, c, |x| {
            count = count.checked_add(x.len_utf8()).ok_or("Budget")?;
            identical &= original.next() == Some(x);
            Ok(())
        })?;
        identical &= original.next().is_none();
        if identical {
            return Ok(TextRef::Source(span(p)));
        }
        self.sync(c)?;
        let Target::Build(b) = &mut self.target else {
            unreachable!()
        };
        b.reserve_cooked(count).map_err(model_error)?;
        let start = b.append_cooked(&[]).map_err(model_error)?.start;
        // Stack buffer emits normalized UTF8 without a second owned scalar copy.
        let mut buf = [0u8; 4096];
        let mut len = 0usize;
        cooked_each(raw(s, p), attribute, refs, c, |x| {
            if len + 4 > buf.len() {
                b.append_cooked(&buf[..len]).map_err(model_error)?;
                len = 0;
            }
            let mut bytes = [0; 4];
            let v = x.encode_utf8(&mut bytes).as_bytes();
            buf[len..len + v.len()].copy_from_slice(v);
            len += v.len();
            Ok(())
        })?;
        if len > 0 {
            b.append_cooked(&buf[..len]).map_err(model_error)?;
        }
        let end = start.checked_add(count as u32).ok_or("Budget")?;
        self.sync(c)?;
        Ok(TextRef::Cooked { start, end })
    }
    fn literal_xml(&mut self, c: &mut Control) -> XResult<TextRef> {
        if let Target::Verify(m) = &self.target {
            for x in m.raw_xml_metadata() {
                if let Some(t) = x.name.and_then(|n| n.namespace)
                    && m.text_ref(t) == XML
                {
                    return Ok(t);
                }
            }
            return Err("XmlRepresentation");
        }
        self.sync(c)?;
        let Target::Build(b) = &mut self.target else {
            unreachable!()
        };
        let r = b.append_cooked(XML.as_bytes()).map_err(model_error)?;
        self.sync(c)?;
        Ok(TextRef::Cooked {
            start: r.start,
            end: r.end,
        })
    }
    fn add(
        &mut self,
        kind: StructureKind,
        location: Option<SourceSpan>,
        text: Option<TextRef>,
        mut meta: XmlMetadata,
        parent: Option<NodeId>,
        c: &mut Control,
    ) -> XResult<NodeId> {
        self.sync(c)?;
        admit_item(self.next as usize, self.relations.len(), parent.is_some())?;
        let id = NodeId(self.next);
        meta.node = id;
        match &mut self.target {
            Target::Build(b) => {
                b.push_node(NodeSpec {
                    kind,
                    location,
                    text,
                })
                .map_err(model_error)?;
                b.push_xml_metadata(meta).map_err(model_error)?;
            }
            Target::Verify(m) => {
                let n = m.node(id).ok_or("XmlRepresentation")?;
                if n.kind() != kind
                    || (!kind.is_container() && n.source_location() != location)
                    || n.decoded().is_some() != text.is_some()
                {
                    return Err("XmlRepresentation");
                }
                if n.parent().map(|p| p.owner) != parent {
                    return Err("XmlRepresentation");
                }
            }
        }
        self.sync(c)?;
        if let Some(parent) = parent {
            reserve(&mut self.relations, 1, c)?;
            self.relations.push((parent, id));
        }
        self.next += 1;
        self.sync(c)?;
        Ok(id)
    }
    fn metadata(
        &mut self,
        meta: XmlMetadata,
        location: Option<SourceSpan>,
        c: &mut Control,
    ) -> XResult<()> {
        self.sync(c)?;
        match &mut self.target {
            Target::Build(b) => {
                if let Some(s) = location {
                    b.set_location(meta.node, s).map_err(model_error)?
                }
                b.replace_xml_metadata(meta).map_err(model_error)?;
            }
            Target::Verify(m) => {
                let n = m.node(meta.node).ok_or("XmlRepresentation")?;
                let got = *n.xml_metadata().ok_or("XmlRepresentation")?;
                if n.source_location() != location {
                    return Err("XmlRepresentation");
                }
                let mut expected = meta;
                let mut got_normal = got;
                if let (Some(e), Some(g)) = (expected.name.as_mut(), got_normal.name.as_mut()) {
                    if e.namespace.map(|t| m.text_ref(t)) != g.namespace.map(|t| m.text_ref(t)) {
                        return Err("XmlRepresentation");
                    }
                    e.namespace = None;
                    g.namespace = None;
                }
                if expected != got_normal {
                    return Err("XmlRepresentation");
                }
            }
        }
        self.sync(c)
    }
}
#[derive(Clone, Copy)]
struct Binding {
    key: Span,
    hash: u64,
    value: Option<TextRef>,
}
struct Scope {
    slots: Vec<Option<Binding>>,
    len: usize,
    undo: Vec<(Span, Option<TextRef>)>,
    random: RandomState,
}
impl Scope {
    fn new() -> Self {
        Self {
            slots: Vec::new(),
            len: 0,
            undo: Vec::new(),
            random: RandomState::new(),
        }
    }
    fn hash(&self, s: &str, key: Span, c: &mut Control) -> XResult<u64> {
        let mut h = self.random.build_hasher();
        for b in raw(s, key).as_bytes().chunks(4096) {
            c.poll()?;
            h.write(b)
        }
        Ok(h.finish())
    }
    fn slot(&self, s: &str, key: Span, hash: u64) -> usize {
        let mut i = hash as usize & (self.slots.len() - 1);
        loop {
            match self.slots[i] {
                None => return i,
                Some(b) if b.hash == hash && raw(s, b.key) == raw(s, key) => return i,
                _ => i = (i + 1) & (self.slots.len() - 1),
            }
        }
    }
    fn grow(&mut self, c: &mut Control) -> XResult<()> {
        let cap = self.slots.len().max(8).checked_mul(2).ok_or("Budget")?;
        let bytes = cap
            .checked_mul(size_of::<Option<Binding>>())
            .ok_or("Budget")?;
        c.charge(bytes)?;
        let mut slots = Vec::new();
        slots.try_reserve_exact(cap).map_err(|_| "Budget")?;
        slots.resize(cap, None);
        let actual = slots.capacity() * size_of::<Option<Binding>>();
        if actual > bytes {
            c.charge(actual - bytes)?
        } else {
            c.release(bytes - actual)
        }
        for b in self.slots.iter().flatten() {
            c.poll()?;
            let mut i = b.hash as usize & (cap - 1);
            while slots[i].is_some() {
                i = (i + 1) & (cap - 1)
            }
            slots[i] = Some(*b)
        }
        let old = self.slots.capacity() * size_of::<Option<Binding>>();
        self.slots = slots;
        c.release(old);
        Ok(())
    }
    fn set(
        &mut self,
        s: &str,
        key: Span,
        value: Option<TextRef>,
        c: &mut Control,
        record: bool,
    ) -> XResult<()> {
        if self.slots.is_empty() || self.len * 2 >= self.slots.len() {
            self.grow(c)?
        }
        let h = self.hash(s, key, c)?;
        let i = self.slot(s, key, h);
        let old = self.slots[i].and_then(|b| b.value);
        if record {
            reserve(&mut self.undo, 1, c)?;
            self.undo.push((key, old))
        }
        if self.slots[i].is_none() {
            self.len += 1
        }
        self.slots[i] = Some(Binding {
            key,
            hash: h,
            value,
        });
        Ok(())
    }
    fn get(&self, s: &str, key: Span, c: &mut Control) -> XResult<Option<TextRef>> {
        if self.slots.is_empty() {
            return Ok(None);
        }
        let h = self.hash(s, key, c)?;
        Ok(self.slots[self.slot(s, key, h)].and_then(|b| b.value))
    }
    fn restore(&mut self, s: &str, mark: usize, c: &mut Control) -> XResult<()> {
        while self.undo.len() > mark {
            let (k, v) = self.undo.pop().unwrap();
            self.set(s, k, v, c, false)?
        }
        Ok(())
    }
}
#[derive(Clone, Copy)]
struct Open {
    id: NodeId,
    meta: XmlMetadata,
    name: Span,
    scope: usize,
    start: usize,
}
fn xml_name(s: &str, name: Span, namespace: Option<TextRef>, c: &mut Control) -> XResult<XmlName> {
    let (p, l) = qname(raw(s, name), c)?;
    let local = Span {
        start: name.end - l.len(),
        end: name.end,
    };
    Ok(XmlName {
        qname: span(name),
        local: span(local),
        prefix: p.map(|p| {
            span(Span {
                start: name.start,
                end: name.start + p.len(),
            })
        }),
        namespace,
    })
}
fn resolve(
    s: &str,
    name: Span,
    element: bool,
    scope: &Scope,
    sink: &mut Sink,
    c: &mut Control,
    xml: &mut Option<TextRef>,
) -> XResult<Option<TextRef>> {
    let (p, _) = qname(raw(s, name), c)?;
    if p == Some("xmlns") {
        return Err("InvalidXml");
    }
    if p == Some("xml") {
        if xml.is_none() {
            *xml = Some(sink.literal_xml(c)?)
        }
        return Ok(*xml);
    }
    if p.is_none() && !element {
        return Ok(None);
    }
    let key = Span {
        start: name.start,
        end: name.start + p.map_or(0, str::len),
    };
    let v = scope.get(s, key, c)?.filter(|t| !sink.text(*t).is_empty());
    if p.is_some() && v.is_none() {
        return Err("InvalidXml");
    }
    Ok(v)
}
fn parse(s: &str, sink: &mut Sink, c: &mut Control) -> XResult<usize> {
    lexical::validate_chars(s, c)?;
    let mut cursor = Cursor::new(s);
    let base = cursor.pos;
    let mut reader = quick_xml::Reader::from_str(&s[base..]);
    reader.config_mut().enable_all_checks(true);
    let mut scope = Scope::new();
    let mut stack = Vec::<Open>::new();
    let mut roots = 0usize;
    let mut maxdepth = 0;
    let mut doc = XmlMetadata::new(NodeId(0));
    sink.add(StructureKind::XmlDocument, None, None, doc, None, c)?;
    let mut xml = None;
    while let Some(t) = cursor.next(c)? {
        sink.sync(c)?;
        c.poll()?;
        if matches!(t.kind, Kind::Start | Kind::Empty) && stack.len() + 1 > MAX_STRUCTURE_DEPTH {
            return Err("Depth");
        }
        let before = reader.buffer_position() as usize + base;
        let actual = match reader.read_event().map_err(|_| "InvalidXml")? {
            quick_xml::events::Event::Start(_) => Kind::Start,
            quick_xml::events::Event::End(_) => Kind::End,
            quick_xml::events::Event::Empty(_) => Kind::Empty,
            quick_xml::events::Event::Text(_) => Kind::Text,
            quick_xml::events::Event::GeneralRef(_) => Kind::Ref,
            quick_xml::events::Event::CData(_) => Kind::CData,
            quick_xml::events::Event::Comment(_) => Kind::Comment,
            quick_xml::events::Event::PI(_) => Kind::Pi,
            quick_xml::events::Event::Decl(_) => Kind::Decl,
            quick_xml::events::Event::DocType(_) => Kind::DocType,
            quick_xml::events::Event::Eof => return Err("XmlRepresentation"),
        };
        if actual != t.kind
            || before != t.span.start
            || reader.buffer_position() as usize + base != t.span.end
        {
            return Err("XmlRepresentation");
        }
        let parent = stack.last().map_or(NodeId(0), |o| o.id);
        match t.kind {
            Kind::Start | Kind::Empty => {
                let depth = stack.len() + 1;
                maxdepth = maxdepth.max(depth);
                if stack.is_empty() {
                    roots += 1;
                    if roots > 1 {
                        return Err("InvalidXml");
                    }
                }
                let (name, attributes) = attrs(s, t.payload, c)?;
                let allocated = attributes.capacity() * size_of::<profile::Attr>();
                let id = NodeId(sink.next);
                let mut meta = XmlMetadata::new(id);
                meta.name = Some(xml_name(s, name, None, c)?);
                meta.open_span = Some(span(t.span));
                meta.empty_form = t.kind == Kind::Empty;
                meta.attribute_count = attributes.len() as u32;
                sink.add(
                    StructureKind::XmlElement,
                    Some(span(t.span)),
                    None,
                    meta,
                    Some(parent),
                    c,
                )?;
                if stack.is_empty() {
                    doc.root_element = Some(id)
                }
                let mark = scope.undo.len();
                let mut ids = Vec::<NodeId>::new();
                let mut order = Vec::<usize>::new();
                for (i, a) in attributes.iter().enumerate() {
                    reserve(&mut order, 1, c)?;
                    order.push(i);
                    let q = raw(s, a.name);
                    let (p, local) = qname(q, c)?;
                    let binding = if q == "xmlns" {
                        Some(Span {
                            start: a.name.end,
                            end: a.name.end,
                        })
                    } else if p == Some("xmlns") {
                        Some(Span {
                            start: a.name.end - local.len(),
                            end: a.name.end,
                        })
                    } else {
                        None
                    };
                    let value = sink.norm(s, a.value, true, true, c)?;
                    let aid = NodeId(sink.next);
                    let mut am = XmlMetadata::new(aid);
                    am.name = Some(xml_name(s, a.name, None, c)?);
                    am.quoted_value = Some(span(a.quoted));
                    am.value_content = Some(span(a.value));
                    am.binding_prefix = binding.filter(|p| p.start != p.end).map(span);
                    sink.add(
                        if binding.is_some() {
                            StructureKind::XmlNamespaceDeclaration
                        } else {
                            StructureKind::XmlAttribute
                        },
                        Some(span(a.span)),
                        Some(value),
                        am,
                        Some(id),
                        c,
                    )?;
                    reserve(&mut ids, 1, c)?;
                    ids.push(aid);
                    if let Some(key) = binding {
                        let p = raw(s, key);
                        let v = sink.text(value);
                        if p == "xmlns"
                            || v == XMLNS
                            || (p == "xml" && v != XML)
                            || (p != "xml" && v == XML)
                            || (!p.is_empty() && v.is_empty())
                        {
                            return Err("InvalidXml");
                        }
                        scope.set(s, key, Some(value), c, true)?;
                    }
                }
                checked_sort(&mut order, c, |a, b, c| {
                    checked_cmp(raw(s, attributes[*a].name), raw(s, attributes[*b].name), c)
                })?;
                for pair in order.windows(2) {
                    c.poll()?;
                    if checked_cmp(
                        raw(s, attributes[pair[0]].name),
                        raw(s, attributes[pair[1]].name),
                        c,
                    )?
                    .is_eq()
                    {
                        return Err("InvalidXml");
                    }
                }
                meta.name = Some(xml_name(
                    s,
                    name,
                    resolve(s, name, true, &scope, sink, c, &mut xml)?,
                    c,
                )?);
                let mut names = Vec::<(Option<TextRef>, SourceSpan)>::new();
                for (a, aid) in attributes.iter().zip(&ids) {
                    let q = raw(s, a.name);
                    let (p, l) = qname(q, c)?;
                    let ns = q == "xmlns" || p == Some("xmlns");
                    let uri = if ns {
                        None
                    } else {
                        resolve(s, a.name, false, &scope, sink, c, &mut xml)?
                    };
                    let mut am = XmlMetadata::new(*aid);
                    am.name = Some(xml_name(s, a.name, uri, c)?);
                    am.quoted_value = Some(span(a.quoted));
                    am.value_content = Some(span(a.value));
                    am.binding_prefix = if q == "xmlns" {
                        None
                    } else if ns {
                        Some(span(Span {
                            start: a.name.end - l.len(),
                            end: a.name.end,
                        }))
                    } else {
                        None
                    };
                    sink.metadata(am, Some(span(a.span)), c)?;
                    if !ns {
                        reserve(&mut names, 1, c)?;
                        names.push((uri, am.name.unwrap().local));
                    }
                }
                checked_sort(&mut names, c, |(au, al), (bu, bl), c| {
                    let ordering = match (au, bu) {
                        (None, None) => std::cmp::Ordering::Equal,
                        (None, Some(_)) => std::cmp::Ordering::Less,
                        (Some(_), None) => std::cmp::Ordering::Greater,
                        (Some(a), Some(b)) => checked_cmp(sink.text(*a), sink.text(*b), c)?,
                    };
                    if ordering.is_eq() {
                        checked_cmp(&s[al.range()], &s[bl.range()], c)
                    } else {
                        Ok(ordering)
                    }
                })?;
                for pair in names.windows(2) {
                    c.poll()?;
                    if match (pair[0].0, pair[1].0) {
                        (None, None) => true,
                        (Some(a), Some(b)) => checked_cmp(sink.text(a), sink.text(b), c)?.is_eq(),
                        _ => false,
                    } && checked_cmp(&s[pair[0].1.range()], &s[pair[1].1.range()], c)?.is_eq()
                    {
                        return Err("InvalidXml");
                    }
                }
                c.release(names.capacity() * size_of::<(Option<TextRef>, SourceSpan)>());
                drop(names);
                c.release(order.capacity() * size_of::<usize>());
                drop(order);
                c.release(ids.capacity() * size_of::<NodeId>());
                drop(ids);
                c.release(allocated);
                drop(attributes);
                if t.kind == Kind::Start {
                    reserve(&mut stack, 1, c)?;
                    stack.push(Open {
                        id,
                        meta,
                        name,
                        scope: mark,
                        start: t.span.start,
                    })
                } else {
                    sink.metadata(meta, Some(span(t.span)), c)?;
                    scope.restore(s, mark, c)?
                }
            }
            Kind::End => {
                let mut end = t.payload.end;
                while end > t.payload.start && is_s(s.as_bytes()[end - 1]) {
                    end -= 1
                }
                let name = Span {
                    start: t.payload.start,
                    end,
                };
                qname(raw(s, name), c)?;
                let mut o = stack.pop().ok_or("InvalidXml")?;
                if raw(s, name) != raw(s, o.name) {
                    return Err("InvalidXml");
                }
                o.meta.close_span = Some(span(t.span));
                sink.metadata(
                    o.meta,
                    Some(span(Span {
                        start: o.start,
                        end: t.span.end,
                    })),
                    c,
                )?;
                scope.restore(s, o.scope, c)?;
            }
            kind => {
                let id = NodeId(sink.next);
                let mut meta = XmlMetadata::new(id);
                let mut payload = t.payload;
                let (kind, attr, refs) = match kind {
                    Kind::Text => {
                        if contains_checked(raw(s, payload), b"]]>", c)?
                            || stack.is_empty() && !only_s(raw(s, payload), c)?
                        {
                            return Err("InvalidXml");
                        }
                        (StructureKind::XmlText, false, false)
                    }
                    Kind::Ref => {
                        if stack.is_empty() {
                            return Err("InvalidXml");
                        };
                        reference(raw(s, payload), c)?;
                        meta.reference_kind = Some(if raw(s, payload).starts_with("#x") {
                            XmlReferenceKind::Hexadecimal
                        } else if raw(s, payload).starts_with('#') {
                            XmlReferenceKind::Decimal
                        } else {
                            XmlReferenceKind::Predefined
                        });
                        meta.value_content = Some(span(payload));
                        payload = t.span;
                        (StructureKind::XmlReference, false, true)
                    }
                    Kind::CData => {
                        if stack.is_empty() {
                            return Err("InvalidXml");
                        }
                        (StructureKind::XmlCData, false, false)
                    }
                    Kind::Comment => {
                        if contains_checked(raw(s, payload), b"--", c)?
                            || raw(s, payload).ends_with('-')
                        {
                            return Err("InvalidXml");
                        }
                        (StructureKind::XmlComment, false, false)
                    }
                    Kind::Pi => {
                        let mut e = payload.start;
                        while e < payload.end && !is_s(s.as_bytes()[e]) {
                            if e & 4095 == 0 {
                                c.poll()?
                            }
                            e += 1
                        }
                        let target = Span {
                            start: payload.start,
                            end: e,
                        };
                        if !nc_name(raw(s, target), c)?
                            || raw(s, target).eq_ignore_ascii_case("xml")
                        {
                            return Err("InvalidXml");
                        }
                        meta.target = Some(span(target));
                        while e < payload.end && is_s(s.as_bytes()[e]) {
                            if e & 4095 == 0 {
                                c.poll()?
                            }
                            e += 1
                        }
                        payload.start = e;
                        (StructureKind::XmlPi, false, false)
                    }
                    Kind::Decl => {
                        if t.span.start != base || roots != 0 {
                            return Err("InvalidXml");
                        };
                        declaration(s, payload, c)?;
                        (StructureKind::XmlDeclaration, false, false)
                    }
                    Kind::DocType => {
                        if roots != 0 || !stack.is_empty() || doc.has_doctype {
                            return Err("InvalidXml");
                        };
                        doctype(s, payload, c)?;
                        doc.has_doctype = true;
                        (StructureKind::XmlDocType, false, false)
                    }
                    _ => unreachable!(),
                };
                if meta.value_content.is_none() {
                    meta.value_content = Some(span(payload))
                }
                let text = sink.norm(s, payload, attr, refs, c)?;
                sink.add(kind, Some(span(t.span)), Some(text), meta, Some(parent), c)?;
                sink.metadata(meta, Some(span(t.span)), c)?;
            }
        }
    }
    if roots != 1 || !stack.is_empty() {
        return Err("InvalidXml");
    }
    if !matches!(
        reader.read_event().map_err(|_| "InvalidXml")?,
        quick_xml::events::Event::Eof
    ) || reader.buffer_position() as usize + base != s.len()
    {
        return Err("XmlRepresentation");
    }
    sink.metadata(doc, None, c)?;
    c.release(stack.capacity() * size_of::<Open>());
    drop(stack);
    c.release(
        scope.slots.capacity() * size_of::<Option<Binding>>()
            + scope.undo.capacity() * size_of::<(Span, Option<TextRef>)>(),
    );
    drop(scope);
    c.poll()?;
    Ok(maxdepth)
}
pub fn project(
    source: Arc<String>,
    generation: u64,
    cancel: &AtomicBool,
) -> std::result::Result<(Structure, usize, usize), StructureFailure> {
    if source.len() > MAX_STRUCTURE_SOURCE {
        return Err(StructureFailure::SourceBudget);
    }
    let mut stopped = || cancel.load(Ordering::Relaxed);
    let mut control = Control {
        deadline: Instant::now() + Duration::from_secs(5),
        cancel: &mut stopped,
        own: 0,
        base: 0,
        limit: MAX_STRUCTURE_BYTES,
    };
    let mut builder =
        StructureBuilder::new_for_format(source.clone(), generation, StructureFormat::Xml)
            .map_err(|e| failure(model_error(e)))?;
    let mut sink = Sink {
        target: Target::Build(&mut builder),
        next: 0,
        relations: Vec::new(),
    };
    let depth = parse(&source, &mut sink, &mut control).map_err(failure)?;
    checked_sort(&mut sink.relations, &mut control, |a, b, _| {
        Ok((a.0.0, a.1.0).cmp(&(b.0.0, b.1.0)))
    })
    .map_err(failure)?;
    sink.sync(&mut control).map_err(failure)?;
    let next = sink.next;
    let relations = std::mem::take(&mut sink.relations);
    drop(sink);
    let mut at = 0;
    for owner in 0..next {
        control.poll().map_err(failure)?;
        let start = at;
        while at < relations.len() && relations[at].0 == NodeId(owner) {
            at += 1
        }
        if start < at
            || owner == 0
            || builder
                .xml_metadata_mut(NodeId(owner))
                .map_err(|e| failure(model_error(e)))?
                .open_span
                .is_some()
        {
            builder
                .set_entries(
                    NodeId(owner),
                    relations[start..at].iter().map(|(_, n)| EntrySpec {
                        key: None,
                        value: *n,
                    }),
                )
                .map_err(|e| failure(model_error(e)))?;
        }
    }
    control.release(relations.capacity() * size_of::<(NodeId, NodeId)>());
    drop(relations);
    builder
        .set_scratch_bytes(HEADROOM)
        .map_err(|e| failure(model_error(e)))?;
    let model = builder
        .finish_with(NodeId(0), depth, 0, || cancel.load(Ordering::Relaxed))
        .map_err(|e| failure(model_error(e)))?;
    Ok((model, depth, 0))
}
pub(crate) fn validate_source(
    model: &Structure,
    cancel: &mut impl FnMut() -> bool,
) -> std::result::Result<(), ModelError> {
    validate_source_until(model, cancel, Instant::now() + Duration::from_secs(5))
}
fn validation_error(error: &str) -> ModelError {
    match error {
        "Cancelled" => ModelError::Cancelled,
        "Timeout" => ModelError::Timeout,
        "Budget" => ModelError::Budget,
        "Depth" => ModelError::Depth,
        _ => ModelError::Invalid,
    }
}
fn validate_source_until(
    model: &Structure,
    cancel: &mut impl FnMut() -> bool,
    deadline: Instant,
) -> std::result::Result<(), ModelError> {
    let mut control = Control {
        deadline,
        cancel,
        own: 0,
        base: model.persistent_bytes() + HEADROOM,
        limit: MAX_STRUCTURE_BYTES,
    };
    let mut sink = Sink {
        target: Target::Verify(model),
        next: 0,
        relations: Vec::new(),
    };
    let depth = parse(model.source(), &mut sink, &mut control).map_err(validation_error)?;
    if sink.next as usize != model.node_count() {
        return Err(ModelError::Invalid);
    }
    checked_sort(&mut sink.relations, &mut control, |a, b, _| {
        Ok((a.0.0, a.1.0).cmp(&(b.0.0, b.1.0)))
    })
    .map_err(validation_error)?;
    let mut at = 0;
    for owner in 0..model.node_count() {
        let node = model.node(NodeId(owner as u32)).unwrap();
        for entry in node.entries() {
            let relation = sink.relations.get(at).ok_or(ModelError::Invalid)?;
            if *relation != (node.id(), entry.value().id()) {
                return Err(ModelError::Invalid);
            }
            at += 1;
        }
    }
    if at != sink.relations.len() || depth > MAX_STRUCTURE_DEPTH {
        return Err(ModelError::Invalid);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn model(s: &str) -> Structure {
        project(Arc::new(s.to_owned()), 7, &AtomicBool::new(false))
            .unwrap()
            .0
    }
    #[test]
    fn expired_validation_is_timeout_and_cancellation_keeps_priority() {
        let m = model("<r><a/><b/></r>");
        assert_eq!(validate_source(&m, &mut || false), Ok(()));
        let expired = Instant::now();
        let error = validate_source_until(&m, &mut || false, expired).unwrap_err();
        assert_eq!(error, ModelError::Timeout);
        assert_eq!(failure(model_error(error)), StructureFailure::Timeout);
        assert_eq!(
            validate_source_until(&m, &mut || true, expired),
            Err(ModelError::Cancelled)
        );
    }
    #[test]
    fn real_relation_sort_preserves_timeout_and_cancel_cause() {
        let m = model("<r><a/><b/></r>");
        let relations: Vec<_> = (0..m.node_count())
            .flat_map(|owner| {
                m.node(NodeId(owner as u32))
                    .unwrap()
                    .entries()
                    .map(move |entry| (NodeId(owner as u32), entry.value().id()))
            })
            .collect();
        assert!(relations.len() >= 2);
        for (cancelled, expired, expected) in [
            (false, false, Ok(())),
            (false, true, Err(ModelError::Timeout)),
            (true, true, Err(ModelError::Cancelled)),
        ] {
            let mut values = relations.clone();
            values.reverse();
            let mut cancel = || cancelled;
            let mut control = Control {
                deadline: Instant::now()
                    + if expired {
                        Duration::ZERO
                    } else {
                        Duration::from_secs(5)
                    },
                cancel: &mut cancel,
                own: 0,
                base: m.persistent_bytes() + HEADROOM,
                limit: MAX_STRUCTURE_BYTES,
            };
            let result = checked_sort(&mut values, &mut control, |a, b, _| {
                Ok((a.0.0, a.1.0).cmp(&(b.0.0, b.1.0)))
            })
            .map_err(validation_error);
            assert_eq!(result, expected);
            if expected.is_ok() {
                assert!(
                    values
                        .windows(2)
                        .all(|pair| (pair[0].0.0, pair[0].1.0) <= (pair[1].0.0, pair[1].1.0))
                );
            }
        }
    }
    #[test]
    fn mixed_physical_nodes_namespaces_and_references() {
        let m = model(
            "<?xml version='1.0'?><r xmlns='urn:a&amp;b' xmlns:p='urn:a&#38;b' a='x&#xD;\r\ny'><p:x>hi&amp;<![CDATA[z&]]><!--c--><?pi d?></p:x><x xmlns=''/></r>",
        );
        assert_eq!(m.format(), StructureFormat::Xml);
        let root = m.root().entry(1).unwrap().value();
        assert_eq!(root.kind(), StructureKind::XmlElement);
        assert_eq!(root.xml_metadata().unwrap().attribute_count, 3);
        assert_eq!(
            m.text_ref(
                root.xml_metadata()
                    .unwrap()
                    .name
                    .unwrap()
                    .namespace
                    .unwrap()
            ),
            "urn:a&b"
        );
        let a = root.entry(2).unwrap().value();
        assert_eq!(a.decoded(), Some("x\r y"));
        let p = root.entry(3).unwrap().value();
        assert_eq!(p.entries().count(), 5);
        let reference = p.entry(1).unwrap().value();
        assert_eq!(reference.decoded(), Some("&"));
        assert_eq!(reference.kind(), StructureKind::XmlReference);
        let empty = root.entry(4).unwrap().value();
        assert!(empty.xml_metadata().unwrap().empty_form);
        assert!(
            empty
                .xml_metadata()
                .unwrap()
                .name
                .unwrap()
                .namespace
                .is_none()
        );
        assert!(m.persistent_bytes() < MAX_STRUCTURE_BYTES);
    }
    #[test]
    fn bom_and_exact_utf8_spans() {
        let s = "\u{feff}<é a='界'/>";
        let m = model(s);
        let e = m.root().entry(0).unwrap().value();
        let x = e.xml_metadata().unwrap();
        assert_eq!(e.source_location().unwrap().range(), 3..s.len());
        assert_eq!(&s[x.name.unwrap().qname.range()], "é");
        let a = e.entry(0).unwrap().value();
        assert_eq!(a.decoded(), Some("界"));
        assert_eq!(&s[a.source_location().unwrap().range()], "a='界'");
    }
    #[test]
    fn complete_failures_are_distinct() {
        for (s, want) in [
            ("<r>", StructureFailure::InvalidXml),
            ("<r/><r/>", StructureFailure::InvalidXml),
            ("<r>&custom;</r>", StructureFailure::UnsupportedXmlEntity),
            (
                "<?xml version='1.1'?><r/>",
                StructureFailure::UnsupportedXmlVersion,
            ),
            (
                "<?xml version='1.0' encoding='latin1'?><r/>",
                StructureFailure::UnsupportedXmlEncoding,
            ),
            (
                "<r xmlns:p='u' xmlns:q='u' p:a='' q:a=''/>",
                StructureFailure::InvalidXml,
            ),
        ] {
            assert_eq!(
                project(Arc::new(s.into()), 1, &AtomicBool::new(false)).unwrap_err(),
                want,
                "{s}"
            );
        }
    }
    #[test]
    fn opaque_doctype_is_inert_and_warned() {
        let m = model("<!DOCTYPE r SYSTEM 'file:///no-open.dtd' [<!ENTITY x 'unused'>]><r/>");
        assert!(m.root().xml_metadata().unwrap().has_doctype);
        let d = m.root().entry(0).unwrap().value();
        assert_eq!(d.kind(), StructureKind::XmlDocType);
        assert!(d.decoded().unwrap().contains("file:///no-open.dtd"));
    }
    #[test]
    fn element_depth_and_document_wrapper() {
        for n in [64, 65] {
            let s = format!("{}{}", "<r>".repeat(n), "</r>".repeat(n));
            let result = project(Arc::new(s), 1, &AtomicBool::new(false));
            assert_eq!(result.is_ok(), n == 64);
            if n == 65 {
                assert_eq!(result.unwrap_err(), StructureFailure::Depth)
            }
        }
    }
    #[test]
    fn actual_capacities_charge_growth_and_release() {
        let mut no = || false;
        let mut c = Control {
            deadline: Instant::now() + Duration::from_secs(1),
            cancel: &mut no,
            own: 0,
            base: MAX_STRUCTURE_BYTES - 256,
            limit: MAX_STRUCTURE_BYTES,
        };
        let mut v = Vec::<u64>::new();
        reserve(&mut v, 1, &mut c).unwrap();
        assert_eq!(c.own, v.capacity() * 8);
        let old = c.own;
        assert!(reserve(&mut v, 100, &mut c).is_err());
        assert_eq!(c.own, old);
    }
    #[test]
    fn cancellation_and_aggregated_admission() {
        assert!(project(Arc::new("<r/>".into()), 1, &AtomicBool::new(true)).is_err());
        assert!(Structure::xml_projected_bytes(1_000_000, 0, 0).unwrap() > MAX_STRUCTURE_BYTES);
    }
}

#[cfg(all(test, windows))]
mod native_job_gate {
    use std::{
        io::{Read, Write},
        process::{Command, Stdio},
        thread,
        time::{Duration, Instant},
    };
    #[test]
    #[ignore = "child of owned_job_committed_memory_boundary only"]
    fn owned_memory_child() {
        assert_eq!(
            std::env::var("R3VIEWER_OWNED_JOB_PROBE").as_deref(),
            Ok("xml-g5")
        );
        let mut signal = [0u8; 1];
        std::io::stdin().read_exact(&mut signal).unwrap();
        use windows_sys::Win32::System::JobObjects::*;
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        let ok = unsafe {
            QueryInformationJobObject(
                std::ptr::null_mut(),
                JobObjectExtendedLimitInformation,
                (&mut info as *mut JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                std::mem::size_of_val(&info) as u32,
                std::ptr::null_mut(),
            )
        };
        assert_ne!(ok, 0);
        assert_eq!(info.ProcessMemoryLimit, 512 * 1024 * 1024);
        assert_ne!(
            info.BasicLimitInformation.LimitFlags & JOB_OBJECT_LIMIT_PROCESS_MEMORY,
            0
        );
        let mut small = Vec::<u8>::new();
        small.try_reserve_exact(128 * 1024 * 1024).unwrap();
        small.resize(128 * 1024 * 1024, 0x7f);
        let mut large = Vec::<u8>::new();
        assert!(large.try_reserve_exact(600 * 1024 * 1024).is_err());
        println!(
            "OWNED_JOB_GATE limit={} positive={} rejected=600MiB",
            info.ProcessMemoryLimit,
            small.len()
        );
    }
    #[test]
    #[ignore = "explicit owned native Windows Job gate; allocates 128MiB in child"]
    fn owned_job_committed_memory_boundary() {
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "xml_worker::native_job_gate::owned_memory_child",
                "--ignored",
                "--nocapture",
            ])
            .env("R3VIEWER_OWNED_JOB_PROBE", "xml-g5")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let job = crate::worker_process::ChildJob::attach(&child).unwrap();
        child.stdin.take().unwrap().write_all(b"x").unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if Instant::now() > deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("owned child exceeded gate lease")
            }
            thread::sleep(Duration::from_millis(5))
        };
        let mut out = String::new();
        child
            .stdout
            .take()
            .unwrap()
            .take(4096)
            .read_to_string(&mut out)
            .unwrap();
        let mut err = String::new();
        child
            .stderr
            .take()
            .unwrap()
            .take(4096)
            .read_to_string(&mut err)
            .unwrap();
        job.close();
        assert!(status.success(), "{out} {err}");
        assert!(out.contains("OWNED_JOB_GATE limit=536870912 positive=134217728 rejected=600MiB"));
        println!("{out}");
    }
}

#[cfg(test)]
mod large_input_gate {
    use super::*;
    #[test]
    #[ignore = "explicit XML G5 large-input probe; sequential synthetic inputs"]
    fn large_fields_namespace_lists_and_whole_budget_failure() {
        let cases = [
            ("name", format!("<{} />", "a".repeat(1024 * 1024))),
            ("attribute", format!("<r a='{}'/>", "x".repeat(1024 * 1024))),
            (
                "namespace",
                format!("<r xmlns:p='urn:{}'><p:x/></r>", "x".repeat(1024 * 1024)),
            ),
            ("many_attributes", {
                let mut s = String::from("<r ");
                for i in 0..200_000 {
                    use std::fmt::Write;
                    write!(s, "a{i}='v' ").unwrap();
                }
                s.push_str("/>");
                s
            }),
            ("many_namespaces", {
                let mut s = String::from("<r ");
                for i in 0..200_000 {
                    use std::fmt::Write;
                    write!(s, "xmlns:p{i}='urn:{i}' ").unwrap();
                }
                s.push_str("/>");
                s
            }),
        ];
        for (name, s) in cases {
            let bytes = s.len();
            let started = Instant::now();
            let result = project(Arc::new(s), 1, &AtomicBool::new(false));
            match result {
                Ok((m, depth, _)) => {
                    println!(
                        "XML_G5 {name} bytes={bytes} Ready nodes={} entries={} persistent={} depth={depth} elapsed_ms={}",
                        m.node_count(),
                        m.entry_count(),
                        m.persistent_bytes(),
                        started.elapsed().as_millis()
                    );
                    assert!(m.persistent_bytes() <= MAX_STRUCTURE_BYTES)
                }
                Err(reason) => {
                    println!(
                        "XML_G5 {name} bytes={bytes} Unavailable={reason:?} elapsed_ms={}",
                        started.elapsed().as_millis()
                    );
                    assert!(
                        matches!(
                            reason,
                            StructureFailure::ArenaBudget | StructureFailure::Timeout
                        ),
                        "unexpected {name}: {reason:?}"
                    )
                }
            }
        }
    }
}

#[cfg(test)]
mod source_item_gates {
    use super::*;
    #[test]
    fn physical_nodes_and_edges_exact_quota_and_overflow() {
        assert!(admit_item(499_999, 499_999, true).is_ok());
        assert_eq!(admit_item(500_000, 499_999, true), Err("Budget"));
        assert_eq!(admit_item(usize::MAX, 1, false), Err("Budget"));
    }
    #[test]
    #[ignore = "explicit XML G5 source128MiB boundary; one owned buffer reused"]
    fn exact_source_limit_and_one_byte_over() {
        let mut s = String::with_capacity(MAX_STRUCTURE_SOURCE + 1);
        s.push_str("<r/>");
        s.extend(std::iter::repeat_n(' ', MAX_STRUCTURE_SOURCE - 4));
        let source = Arc::new(s);
        let started = Instant::now();
        let result = project(source.clone(), 1, &AtomicBool::new(false));
        match &result {
            Ok((m, _, _)) => println!(
                "XML_SOURCE_LIMIT bytes={} Ready nodes={} persistent={} elapsed_ms={}",
                source.len(),
                m.node_count(),
                m.persistent_bytes(),
                started.elapsed().as_millis()
            ),
            Err(e) => {
                println!(
                    "XML_SOURCE_LIMIT bytes={} Unavailable={e:?} elapsed_ms={}",
                    source.len(),
                    started.elapsed().as_millis()
                );
                assert!(matches!(
                    e,
                    StructureFailure::Timeout | StructureFailure::ArenaBudget
                ))
            }
        }
        drop(result);
        let mut s = match Arc::try_unwrap(source) {
            Ok(s) => s,
            Err(_) => panic!("owned source retained"),
        };
        s.push(' ');
        assert_eq!(
            project(Arc::new(s), 1, &AtomicBool::new(false)).unwrap_err(),
            StructureFailure::SourceBudget
        );
        println!(
            "XML_SOURCE_LIMIT bytes={} Unavailable=SourceBudget",
            MAX_STRUCTURE_SOURCE + 1
        );
    }
}
