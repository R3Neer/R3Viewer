//! Streaming YAML serialization inspector; references are never expanded.
use crate::{structure::StructureFailure as Failure, structure_model::*};
use saphyr_parser::{Event, Parser, ScalarStyle, Span, Tag};
use std::{
    borrow::Cow,
    mem::size_of,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
fn error(e: ModelError) -> Failure {
    match e {
        ModelError::SourceLimit => Failure::SourceBudget,
        ModelError::Budget => Failure::ArenaBudget,
        ModelError::Depth => Failure::Depth,
        ModelError::Cancelled => Failure::WorkerExited,
        ModelError::Invalid => Failure::YamlRepresentation,
    }
}
fn check(cancel: &AtomicBool) -> Result<(), Failure> {
    if cancel.load(Ordering::Acquire) {
        Err(Failure::WorkerExited)
    } else {
        Ok(())
    }
}
fn digits(bytes: &[u8], radix: u8, cancelled: &mut impl FnMut() -> bool) -> Result<bool, Failure> {
    if bytes.is_empty() {
        return Ok(false);
    }
    for (i, b) in bytes.iter().enumerate() {
        if i % 1024 == 0 && cancelled() {
            return Err(Failure::WorkerExited);
        }
        let valid = match radix {
            8 => matches!(b, b'0'..=b'7'),
            16 => b.is_ascii_hexdigit(),
            _ => b.is_ascii_digit(),
        };
        if !valid {
            return Ok(false);
        }
    }
    Ok(true)
}
fn integer(s: &str, c: &mut impl FnMut() -> bool) -> Result<bool, Failure> {
    if let Some(s) = s.strip_prefix("0o") {
        return digits(s.as_bytes(), 8, c);
    }
    if let Some(s) = s.strip_prefix("0x") {
        return digits(s.as_bytes(), 16, c);
    }
    let s = s.strip_prefix(['+', '-']).unwrap_or(s);
    digits(s.as_bytes(), 10, c)
}
fn float(s: &str, c: &mut impl FnMut() -> bool) -> Result<bool, Failure> {
    let s = s.strip_prefix(['+', '-']).unwrap_or(s);
    if matches!(s, ".inf" | ".Inf" | ".INF") {
        return Ok(true);
    }
    // NaN signs are not part of the Core production.
    let bytes = s.as_bytes();
    let mut at = 0;
    let mut before = 0;
    let mut after = 0;
    while bytes.get(at).is_some_and(u8::is_ascii_digit) {
        if at % 1024 == 0 && c() {
            return Err(Failure::WorkerExited);
        }
        at += 1;
        before += 1;
    }
    if bytes.get(at) == Some(&b'.') {
        at += 1;
        while bytes.get(at).is_some_and(u8::is_ascii_digit) {
            if at % 1024 == 0 && c() {
                return Err(Failure::WorkerExited);
            }
            at += 1;
            after += 1;
        }
    }
    if before + after == 0 {
        return Ok(false);
    }
    if matches!(bytes.get(at), Some(b'e' | b'E')) {
        at += 1;
        if matches!(bytes.get(at), Some(b'+' | b'-')) {
            at += 1;
        }
        let start = at;
        while bytes.get(at).is_some_and(u8::is_ascii_digit) {
            if at % 1024 == 0 && c() {
                return Err(Failure::WorkerExited);
            }
            at += 1;
        }
        if at == start {
            return Ok(false);
        }
    }
    Ok(at == bytes.len())
}
pub(crate) fn scalar_kind(
    s: &str,
    style: YamlStyle,
    tag: Option<&str>,
    c: &mut impl FnMut() -> bool,
) -> Result<StructureKind, Failure> {
    use StructureKind::*;
    if c() {
        return Err(Failure::WorkerExited);
    }
    let null = matches!(s, "" | "~" | "null" | "Null" | "NULL");
    let boolean = matches!(s, "true" | "True" | "TRUE" | "false" | "False" | "FALSE");
    let numeric_nan = matches!(s, ".nan" | ".NaN" | ".NAN");
    if let Some(tag) = tag {
        return match tag {
            "!" | "tag:yaml.org,2002:str" => Ok(String),
            "tag:yaml.org,2002:null" if null => Ok(Null),
            "tag:yaml.org,2002:bool" if boolean => Ok(Boolean),
            "tag:yaml.org,2002:int" if integer(s, c)? => Ok(Integer),
            "tag:yaml.org,2002:float" if numeric_nan || float(s, c)? => Ok(Float),
            "tag:yaml.org,2002:null"
            | "tag:yaml.org,2002:bool"
            | "tag:yaml.org,2002:int"
            | "tag:yaml.org,2002:float"
            | "tag:yaml.org,2002:seq"
            | "tag:yaml.org,2002:map" => Err(Failure::InvalidYamlCoreTag),
            _ => Ok(YamlTaggedScalar),
        };
    }
    if style != YamlStyle::Plain {
        return Ok(String);
    }
    Ok(if null {
        Null
    } else if boolean {
        Boolean
    } else if integer(s, c)? {
        Integer
    } else if numeric_nan || float(s, c)? {
        Float
    } else {
        String
    })
}
fn tag_key(tag: &Tag) -> &'static str {
    for key in [
        "!",
        "tag:yaml.org,2002:str",
        "tag:yaml.org,2002:null",
        "tag:yaml.org,2002:bool",
        "tag:yaml.org,2002:int",
        "tag:yaml.org,2002:float",
        "tag:yaml.org,2002:seq",
        "tag:yaml.org,2002:map",
    ] {
        if key.strip_prefix(tag.handle.as_str()) == Some(tag.suffix.as_str()) {
            return key;
        }
    }
    "custom"
}
pub(crate) fn validate_shape(kind: StructureKind, tag: Option<&str>) -> Result<(), Failure> {
    if let Some(tag) = tag {
        let valid = match tag {
            "tag:yaml.org,2002:seq" => kind == StructureKind::Sequence,
            "tag:yaml.org,2002:map" => kind == StructureKind::Mapping,
            "tag:yaml.org,2002:str"
            | "tag:yaml.org,2002:null"
            | "tag:yaml.org,2002:bool"
            | "tag:yaml.org,2002:int"
            | "tag:yaml.org,2002:float" => false,
            _ => true,
        };
        if !valid {
            return Err(Failure::InvalidYamlCoreTag);
        }
    }
    Ok(())
}
struct CharIndex {
    points: Vec<u32>,
    chars: usize,
}
impl CharIndex {
    fn build(source: &str, other: usize, cancel: &AtomicBool) -> Result<Self, Failure> {
        let mut points = Vec::new();
        reserve(&mut points, source.len() / 1024 + 2, other).map_err(error)?;
        let mut chars = 0;
        for (i, (byte, _)) in source.char_indices().enumerate() {
            if i % 1024 == 0 {
                check(cancel)?;
                points.push(byte as u32);
            }
            chars = i + 1;
        }
        if chars == 0 || chars % 1024 == 0 {
            points.push(source.len() as u32);
        }
        Ok(Self { points, chars })
    }
    fn byte(&self, source: &str, index: usize) -> Result<u32, Failure> {
        if index > self.chars {
            return Err(Failure::YamlRepresentation);
        }
        if index == self.chars {
            return Ok(source.len() as u32);
        }
        let base = *self
            .points
            .get(index / 1024)
            .ok_or(Failure::YamlRepresentation)? as usize;
        let add = source[base..]
            .char_indices()
            .nth(index % 1024)
            .ok_or(Failure::YamlRepresentation)?
            .0;
        Ok((base + add) as u32)
    }
    fn span(&self, s: &str, span: Span) -> Result<SourceSpan, Failure> {
        let start = self.byte(s, span.start.index())?;
        let end = self.byte(s, span.end.index())?;
        if start > end {
            return Err(Failure::YamlRepresentation);
        }
        Ok(SourceSpan { start, end })
    }
}
struct Frame {
    node: NodeId,
    kind: StructureKind,
    pending: Option<NodeId>,
    entries: Vec<EntrySpec>,
    open: Option<u32>,
}
struct Project<'a> {
    builder: StructureBuilder,
    source: Arc<String>,
    index: CharIndex,
    frames: Vec<Frame>,
    anchors: Vec<(u32, NodeId)>,
    document: u32,
    depth: usize,
    max_depth: usize,
    cancel: &'a AtomicBool,
    owned: usize,
}
impl Project<'_> {
    fn scratch(&self) -> usize {
        self.index.points.capacity() * 4
            + self.frames.capacity() * size_of::<Frame>()
            + self.anchors.capacity() * size_of::<(u32, NodeId)>()
            + self
                .frames
                .iter()
                .map(|f| f.entries.capacity() * size_of::<EntrySpec>())
                .sum::<usize>()
            + self.owned
            + crate::structure_protocol::MAX_FRAME * 4
    }
    fn charge(&mut self) -> Result<(), Failure> {
        self.builder
            .set_scratch_bytes(self.scratch())
            .map_err(error)
    }
    fn attach(&mut self, node: NodeId) -> Result<(), Failure> {
        if let Some(f) = self.frames.last_mut()
            && f.kind == StructureKind::Mapping
            && f.pending.is_none()
        {
            f.pending = Some(node);
            return Ok(());
        }
        let scratch = self.scratch();
        let f = self.frames.last_mut().ok_or(Failure::YamlRepresentation)?;
        let other = self.builder.persistent_bytes() + scratch
            - f.entries.capacity() * size_of::<EntrySpec>();
        reserve(&mut f.entries, 1, other).map_err(error)?;
        f.entries.push(EntrySpec {
            key: f.pending.take(),
            value: node,
        });
        self.charge()
    }
    fn metadata(
        &mut self,
        node: NodeId,
        style: Option<YamlStyle>,
        anchor: usize,
        tag: Option<&Tag>,
        alias: Option<AliasTarget>,
        explicit: bool,
    ) -> Result<(), Failure> {
        let anchor = u32::try_from(anchor).map_err(|_| Failure::YamlRepresentation)?;
        let uri = if let Some(tag) = tag {
            let mut start = None;
            let mut end = 0;
            for part in [&tag.handle, &tag.suffix] {
                for chunk in part.as_bytes().chunks(65_536) {
                    check(self.cancel)?;
                    let r = self.builder.append_cooked(chunk).map_err(error)?;
                    start.get_or_insert(r.start);
                    end = r.end;
                }
            }
            // Concatenate expanded parts exactly; the non-specific ! URI remains '!'.
            Some(TextRef::Cooked {
                start: start.unwrap_or(end),
                end,
            })
        } else {
            None
        };
        self.builder
            .push_metadata(YamlMetadata {
                node,
                document: self.document,
                style,
                anchor: (anchor != 0).then_some(anchor),
                tag: uri,
                alias,
                explicit_start: explicit,
                synthetic_root: false,
            })
            .map_err(error)?;
        if anchor != 0 {
            if self.anchors.last().is_some_and(|a| a.0 >= anchor) {
                return Err(Failure::YamlRepresentation);
            }
            let other = self.builder.persistent_bytes() + self.scratch()
                - self.anchors.capacity() * size_of::<(u32, NodeId)>();
            reserve(&mut self.anchors, 1, other).map_err(error)?;
            self.anchors.push((anchor, node));
            self.charge()?;
        }
        Ok(())
    }
    fn container(
        &mut self,
        kind: StructureKind,
        mark: Span,
        anchor: usize,
        tag: Option<&Tag>,
        explicit: bool,
    ) -> Result<(), Failure> {
        if matches!(kind, StructureKind::Mapping | StructureKind::Sequence) {
            self.depth += 1;
            self.max_depth = self.max_depth.max(self.depth);
            if self.depth > 64 {
                return Err(Failure::Depth);
            }
        }
        let start = self.index.byte(&self.source, mark.start.index())?;
        for f in &mut self.frames {
            if f.open == Some(start) {
                f.open = None;
            }
        }
        let expect = match kind {
            StructureKind::Mapping => Some(b'{'),
            StructureKind::Sequence => Some(b'['),
            _ => None,
        };
        let open =
            if expect.is_some() && self.source.as_bytes().get(start as usize).copied() == expect {
                Some(start)
            } else {
                None
            };
        let node = self
            .builder
            .push_node(NodeSpec {
                kind,
                text: None,
                location: None,
            })
            .map_err(error)?;
        if kind != StructureKind::YamlStream {
            self.attach(node)?;
            self.metadata(node, None, anchor, tag, None, explicit)?;
        }
        self.frames.push(Frame {
            node,
            kind,
            pending: None,
            entries: Vec::new(),
            open,
        });
        self.charge()
    }
    fn close(&mut self, kind: StructureKind, mark: Span) -> Result<NodeId, Failure> {
        let f = self.frames.last().ok_or(Failure::YamlRepresentation)?;
        if f.kind != kind || f.pending.is_some() {
            return Err(Failure::YamlRepresentation);
        }
        if let Some(start) = f.open {
            let mut at = self.index.byte(&self.source, mark.start.index())? as usize;
            let close = if kind == StructureKind::Mapping {
                b'}'
            } else {
                b']'
            };
            let bytes = self.source.as_bytes();
            if kind == StructureKind::Mapping && bytes.get(at) == Some(&b',') {
                at += 1;
                let mut comment = false;
                while let Some(&b) = bytes.get(at) {
                    if at.is_multiple_of(1024) {
                        check(self.cancel)?;
                    }
                    if comment {
                        if matches!(b, b'\r' | b'\n') {
                            comment = false;
                        }
                        at += 1;
                    } else if matches!(b, b' ' | b'\t' | b'\r' | b'\n') {
                        at += 1;
                    } else if b == b'#' {
                        comment = true;
                        at += 1;
                    } else {
                        break;
                    }
                }
            }
            if at > start as usize && bytes.get(at) == Some(&close) {
                self.builder
                    .set_location(
                        f.node,
                        SourceSpan {
                            start,
                            end: u32::try_from(at + 1).map_err(|_| Failure::YamlRepresentation)?,
                        },
                    )
                    .map_err(error)?;
            }
        }
        let f = self.frames.pop().unwrap();
        // Retain its live allocation charge while set_entries copies entries.
        let held = f.entries.capacity() * size_of::<EntrySpec>();
        self.builder
            .set_scratch_bytes(self.scratch() + held)
            .map_err(error)?;
        self.builder
            .set_entries(f.node, f.entries.iter().copied())
            .map_err(error)?;
        drop(f.entries);
        if matches!(kind, StructureKind::Mapping | StructureKind::Sequence) {
            self.depth -= 1;
        }
        self.charge()?;
        Ok(f.node)
    }
}
pub fn project(
    source: Arc<String>,
    generation: u64,
    cancel: &AtomicBool,
) -> Result<(Structure, usize, usize), Failure> {
    check(cancel)?;
    let builder =
        StructureBuilder::new_for_format(source.clone(), generation, StructureFormat::Yaml)
            .map_err(error)?;
    let index = CharIndex::build(
        &source,
        builder.persistent_bytes()
            + 4 * crate::structure_protocol::MAX_FRAME
            + 67 * size_of::<Frame>(),
        cancel,
    )?;
    let mut frames = Vec::new();
    reserve(
        &mut frames,
        67,
        builder.persistent_bytes()
            + index.points.capacity() * 4
            + 4 * crate::structure_protocol::MAX_FRAME,
    )
    .map_err(error)?;
    let mut p = Project {
        builder,
        source: source.clone(),
        index,
        frames,
        anchors: Vec::new(),
        document: u32::MAX,
        depth: 0,
        max_depth: 0,
        cancel,
        owned: 0,
    };
    p.charge()?;
    let mut parser = Parser::new_from_str(&source);
    let mut root = None;
    let mut ended = false;
    while let Some(event) = parser.next_event() {
        check(cancel)?;
        let (event, mark) = event.map_err(|e| {
            if e.info() == "R3Viewer YAML profile requires version 1.2" {
                Failure::UnsupportedYamlVersion
            } else {
                Failure::InvalidYaml
            }
        })?;
        let tag_charge = |tag: &Option<Cow<'_, Tag>>| {
            tag.as_ref().map_or(0, |t| {
                if let Cow::Owned(t) = t {
                    t.handle.capacity() + t.suffix.capacity()
                } else {
                    0
                }
            })
        };
        p.owned = match &event {
            Event::Scalar(Cow::Owned(s), _, _, tag) => s.capacity() + tag_charge(tag),
            Event::Scalar(_, _, _, tag)
            | Event::SequenceStart(_, tag)
            | Event::MappingStart(_, tag) => tag_charge(tag),
            _ => 0,
        };
        p.charge()?;
        match &event {
            Event::StreamStart => p.container(StructureKind::YamlStream, mark, 0, None, false)?,
            Event::DocumentStart(explicit) => {
                p.document = p.document.wrapping_add(1);
                p.anchors.clear();
                p.container(StructureKind::YamlDocument, mark, 0, None, *explicit)?;
            }
            Event::DocumentEnd => {
                p.close(StructureKind::YamlDocument, mark)?;
            }
            Event::SequenceStart(anchor, tag) | Event::MappingStart(anchor, tag) => {
                let kind = if matches!(&event, Event::SequenceStart(..)) {
                    StructureKind::Sequence
                } else {
                    StructureKind::Mapping
                };
                // Expanded URI is evaluated from two borrowed parts, no Display normalization.
                validate_shape(kind, tag.as_ref().map(|t| tag_key(t)))?;
                p.container(kind, mark, *anchor, tag.as_deref(), false)?;
            }
            Event::SequenceEnd => {
                p.close(StructureKind::Sequence, mark)?;
            }
            Event::MappingEnd => {
                p.close(StructureKind::Mapping, mark)?;
            }
            Event::Alias(anchor) => {
                let a = u32::try_from(*anchor).map_err(|_| Failure::YamlRepresentation)?;
                let i = p
                    .anchors
                    .binary_search_by_key(&a, |a| a.0)
                    .map_err(|_| Failure::InvalidYaml)?;
                let target = p.anchors[i].1;
                let location = p.index.span(&source, mark)?;
                let location = source
                    .get(location.range())
                    .filter(|s| s.starts_with('*'))
                    .map(|_| location);
                let n = p
                    .builder
                    .push_node(NodeSpec {
                        kind: StructureKind::Alias,
                        text: None,
                        location,
                    })
                    .map_err(error)?;
                p.attach(n)?;
                p.metadata(
                    n,
                    None,
                    0,
                    None,
                    Some(AliasTarget { anchor: a, target }),
                    false,
                )?;
            }
            Event::Scalar(value, style, anchor, tag) => {
                let style = match style {
                    ScalarStyle::Plain => YamlStyle::Plain,
                    ScalarStyle::SingleQuoted => YamlStyle::SingleQuoted,
                    ScalarStyle::DoubleQuoted => YamlStyle::DoubleQuoted,
                    ScalarStyle::Literal => YamlStyle::Literal,
                    ScalarStyle::Folded => YamlStyle::Folded,
                };
                let synthetic = style == YamlStyle::Plain && value.is_empty();
                let location = if synthetic {
                    None
                } else {
                    Some(p.index.span(&source, mark)?)
                };
                let text =
                    if location.is_some_and(|s| source.get(s.range()) == Some(value.as_ref())) {
                        TextRef::Source(location.unwrap())
                    } else {
                        let mut start = None;
                        let mut end = 0;
                        for chunk in value.as_bytes().chunks(65_536) {
                            check(cancel)?;
                            let r = p.builder.append_cooked(chunk).map_err(error)?;
                            start.get_or_insert(r.start);
                            end = r.end;
                        }
                        if start.is_none() {
                            let r = p.builder.append_cooked(&[]).map_err(error)?;
                            end = r.end;
                        }
                        TextRef::Cooked {
                            start: start.unwrap_or(end),
                            end,
                        }
                    };
                // URI allocation is retained only in the cooked arena; small classification match avoids concatenated scratch.
                let tag_class = tag.as_ref().map(|t| tag_key(t));
                let kind = scalar_kind(value, style, tag_class, &mut || {
                    cancel.load(Ordering::Acquire)
                })?;
                let n = p
                    .builder
                    .push_node(NodeSpec {
                        kind,
                        text: Some(text),
                        location,
                    })
                    .map_err(error)?;
                if synthetic
                    && p.frames
                        .last()
                        .is_some_and(|f| f.kind == StructureKind::YamlDocument)
                {
                    p.builder
                        .mark_document_synthetic(p.frames.last().unwrap().node)
                        .map_err(error)?;
                }
                p.attach(n)?;
                p.metadata(n, Some(style), *anchor, tag.as_deref(), None, false)?;
            }
            Event::StreamEnd => {
                root = Some(p.close(StructureKind::YamlStream, mark)?);
                ended = true;
            }
            Event::Nothing => return Err(Failure::YamlRepresentation),
        }
        drop(event);
        p.owned = 0;
        p.charge()?;
        if ended {
            break;
        }
    }
    if !ended || !p.frames.is_empty() {
        return Err(Failure::InvalidYaml);
    }
    let syntax = p.max_depth;
    drop(p.index);
    drop(p.frames);
    drop(p.anchors);
    p.builder
        .set_scratch_bytes(4 * crate::structure_protocol::MAX_FRAME)
        .map_err(error)?;
    let model = p
        .builder
        .finish(root.ok_or(Failure::InvalidYaml)?, syntax, 0, cancel)
        .map_err(error)?;
    Ok((model, syntax, 0))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn model(s: &str) -> Structure {
        project(Arc::new(s.to_owned()), 19, &AtomicBool::new(false))
            .unwrap()
            .0
    }
    fn root(s: &Structure) -> crate::structure_model::NodeRef<'_> {
        s.root().entry(0).unwrap().value().entry(0).unwrap().value()
    }
    #[test]
    fn core_resolution_uses_expanded_uri_decoded_and_preserves_original() {
        for (raw, kind, decoded) in [
            ("! true", StructureKind::String, "true"),
            ("!!str", StructureKind::String, ""),
            ("!custom", StructureKind::YamlTaggedScalar, ""),
            ("!!int \"\\u0031\\u0032\"", StructureKind::Integer, "12"),
            (
                "!<tag:yaml.org,2002:int> '12'",
                StructureKind::Integer,
                "12",
            ),
            (
                "%TAG !! tag:example.org,custom:\n---\n!!int 12",
                StructureKind::YamlTaggedScalar,
                "12",
            ),
            ("!!float .inf", StructureKind::Float, ".inf"),
            ("!!str null", StructureKind::String, "null"),
            ("'true'", StructureKind::String, "true"),
        ] {
            let s = model(raw);
            let n = root(&s);
            assert_eq!(n.kind(), kind, "{raw}");
            assert_eq!(n.decoded(), Some(decoded));
            assert_eq!(s.source(), raw);
        }
        for raw in ["!!int nope", "!!int []", "!!seq true", "!!map []"] {
            assert_eq!(
                project(Arc::new(raw.into()), 1, &AtomicBool::new(false)).unwrap_err(),
                Failure::InvalidYamlCoreTag
            );
        }
        for raw in ["%YAML 1.1\n---\nx", "---\nx\n...\n%YAML 1.3\n---\ny"] {
            assert_eq!(
                project(Arc::new(raw.into()), 1, &AtomicBool::new(false)).unwrap_err(),
                Failure::UnsupportedYamlVersion
            );
        }
    }
    #[test]
    fn supplemental_custom_tag_category_and_original_raw_interval() {
        let s = model("!custom true\n");
        let n = root(&s);
        assert_eq!(n.kind(), StructureKind::YamlTaggedScalar);
        assert_eq!(n.decoded(), Some("true"));
        assert_eq!(s.text_ref(n.metadata().unwrap().tag.unwrap()), "!custom");
        assert_eq!(n.source_location(), Some(SourceSpan { start: 8, end: 12 }));
        assert_eq!(root(&model("true")).kind(), StructureKind::Boolean);
        assert_eq!(root(&model("! true")).kind(), StructureKind::String);
        assert_eq!(root(&model("!!bool true")).kind(), StructureKind::Boolean);
        let s = model("---\n!!str\n");
        let doc = s.root().entry(0).unwrap().value();
        assert!(doc.metadata().unwrap().synthetic_root);
        assert_eq!(root(&s).kind(), StructureKind::String);
        assert_eq!(root(&s).source_location(), None);
        assert!(
            model("{a: 1, 'a': 2}")
                .duplicate_diagnostic()
                .unwrap()
                .contains("scalar-string decodificadas")
        );
    }
    #[test]
    fn stream_empty_documents_complex_keys_and_alias_cycles_remain_finite() {
        assert_eq!(model("").root().entries().len(), 0);
        let empty = model("---\n...\n---\n\n...");
        assert_eq!(empty.root().entries().len(), 2);
        for d in empty.root().entries() {
            let n = d.value().entry(0).unwrap().value();
            assert_eq!(n.kind(), StructureKind::Null);
            assert_eq!(n.decoded(), Some(""));
            assert_eq!(n.source_location(), None);
        }
        let s = model("? [a, {b: c}]\n: &x [*x]\n");
        let n = root(&s);
        let e = n.entry(0).unwrap();
        assert_eq!(e.key().unwrap().kind(), StructureKind::Sequence);
        assert_eq!(
            s.parent(e.key().unwrap().id()).unwrap().part,
            ParentPart::Key
        );
        let target = e.value();
        let alias = target.entry(0).unwrap().value();
        assert_eq!(alias.kind(), StructureKind::Alias);
        assert_eq!(alias.metadata().unwrap().alias.unwrap().target, target.id());
        assert_eq!(alias.decoded(), None);
        assert_eq!(s.alias_name(alias.id()), Some("x"));
        let s = model("a: &n one\nb: *n\nc: &n two\nd: *n");
        let r = root(&s);
        let first = r
            .entry(1)
            .unwrap()
            .value()
            .metadata()
            .unwrap()
            .alias
            .unwrap();
        let second = r
            .entry(3)
            .unwrap()
            .value()
            .metadata()
            .unwrap()
            .alias
            .unwrap();
        assert_ne!(first.target, second.target);
        assert_eq!(
            project(
                Arc::new("---\na: &n one\n...\n---\nb: *n".into()),
                1,
                &AtomicBool::new(false)
            )
            .unwrap_err(),
            Failure::InvalidYaml
        );
    }
    #[test]
    fn depth_counts_complex_key_containers_and_alias_is_terminal() {
        for depth in [64, 65] {
            let s = format!("{}0{}", "[".repeat(depth), "]".repeat(depth));
            let result = project(Arc::new(s), 1, &AtomicBool::new(false));
            assert_eq!(result.is_ok(), depth == 64);
            if depth == 65 {
                assert_eq!(result.unwrap_err(), Failure::Depth);
            }
        }
        let s = model("{a: 1, 'a': 2, 1: x, 1: y}");
        assert_eq!(s.duplicates().extra_occurrences, 1);
        assert_eq!(
            project(Arc::new("[1]".into()), 1, &AtomicBool::new(true)).unwrap_err(),
            Failure::WorkerExited
        );
    }
    #[test]
    fn independent_flow_oracle_42_selections() {
        {
            let s = model("[]");
            let n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 0, end: 2 }),
                "flow-empty-sequence selection 0"
            );
            assert_eq!(&s.source()[n.source_location().unwrap().range()], "[]");
        }
        {
            let s = model("{}\n");
            let n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 0, end: 2 }),
                "flow-empty-mapping selection 0"
            );
            assert_eq!(&s.source()[n.source_location().unwrap().range()], "{}");
        }
        {
            let s = model("[{}, [], {k: [x, y]}]\n");
            let n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 0, end: 21 }),
                "flow-nested-empty selection 0"
            );
            assert_eq!(
                &s.source()[n.source_location().unwrap().range()],
                "[{}, [], {k: [x, y]}]"
            );
        }
        {
            let s = model("[{}, [], {k: [x, y]}]\n");
            let mut n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            n = n.entry(0).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 1, end: 3 }),
                "flow-nested-empty selection 1"
            );
            assert_eq!(&s.source()[n.source_location().unwrap().range()], "{}");
        }
        {
            let s = model("[{}, [], {k: [x, y]}]\n");
            let mut n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            n = n.entry(1).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 5, end: 7 }),
                "flow-nested-empty selection 2"
            );
            assert_eq!(&s.source()[n.source_location().unwrap().range()], "[]");
        }
        {
            let s = model("[{}, [], {k: [x, y]}]\n");
            let mut n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            n = n.entry(2).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 9, end: 20 }),
                "flow-nested-empty selection 3"
            );
            assert_eq!(
                &s.source()[n.source_location().unwrap().range()],
                "{k: [x, y]}"
            );
        }
        {
            let s = model("[{}, [], {k: [x, y]}]\n");
            let mut n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            n = n.entry(2).unwrap().value();
            n = n.entry(0).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 13, end: 19 }),
                "flow-nested-empty selection 4"
            );
            assert_eq!(&s.source()[n.source_location().unwrap().range()], "[x, y]");
        }
        {
            let s = model("# é 🦊\r\n[\r\n  界,\r\n  {k: [🦊]}\r\n]\r\n");
            let n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 11, end: 38 }),
                "flow-multiline-crlf-unicode selection 0"
            );
            assert_eq!(
                &s.source()[n.source_location().unwrap().range()],
                "[\r\n  界,\r\n  {k: [🦊]}\r\n]"
            );
        }
        {
            let s = model("# é 🦊\r\n[\r\n  界,\r\n  {k: [🦊]}\r\n]\r\n");
            let mut n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            n = n.entry(1).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 24, end: 35 }),
                "flow-multiline-crlf-unicode selection 1"
            );
            assert_eq!(
                &s.source()[n.source_location().unwrap().range()],
                "{k: [🦊]}"
            );
        }
        {
            let s = model("# é 🦊\r\n[\r\n  界,\r\n  {k: [🦊]}\r\n]\r\n");
            let mut n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            n = n.entry(1).unwrap().value();
            n = n.entry(0).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 28, end: 34 }),
                "flow-multiline-crlf-unicode selection 2"
            );
            assert_eq!(&s.source()[n.source_location().unwrap().range()], "[🦊]");
        }
        {
            let s = model("{? [a, b] : {x: y}, ? {k: v} : []}\n");
            let n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 0, end: 34 }),
                "flow-complex-keys selection 0"
            );
            assert_eq!(
                &s.source()[n.source_location().unwrap().range()],
                "{? [a, b] : {x: y}, ? {k: v} : []}"
            );
        }
        {
            let s = model("{? [a, b] : {x: y}, ? {k: v} : []}\n");
            let mut n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            n = n.entry(0).unwrap().key().unwrap();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 3, end: 9 }),
                "flow-complex-keys selection 1"
            );
            assert_eq!(&s.source()[n.source_location().unwrap().range()], "[a, b]");
        }
        {
            let s = model("{? [a, b] : {x: y}, ? {k: v} : []}\n");
            let mut n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            n = n.entry(0).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 12, end: 18 }),
                "flow-complex-keys selection 2"
            );
            assert_eq!(&s.source()[n.source_location().unwrap().range()], "{x: y}");
        }
        {
            let s = model("{? [a, b] : {x: y}, ? {k: v} : []}\n");
            let mut n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            n = n.entry(1).unwrap().key().unwrap();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 22, end: 28 }),
                "flow-complex-keys selection 3"
            );
            assert_eq!(&s.source()[n.source_location().unwrap().range()], "{k: v}");
        }
        {
            let s = model("{? [a, b] : {x: y}, ? {k: v} : []}\n");
            let mut n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            n = n.entry(1).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 31, end: 33 }),
                "flow-complex-keys selection 4"
            );
            assert_eq!(&s.source()[n.source_location().unwrap().range()], "[]");
        }
        {
            let s = model("{a: b,} # exterior\n");
            let n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 0, end: 7 }),
                "flow-mapping-trailing-comma selection 0"
            );
            assert_eq!(&s.source()[n.source_location().unwrap().range()], "{a: b,}");
        }
        {
            let s = model("{a: b, # interior\r\n \t} # exterior\r\n");
            let n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 0, end: 22 }),
                "flow-mapping-comma-comment selection 0"
            );
            assert_eq!(
                &s.source()[n.source_location().unwrap().range()],
                "{a: b, # interior\r\n \t}"
            );
        }
        {
            let s = model("[a, # interior\n ] # exterior\n");
            let n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 0, end: 17 }),
                "flow-sequence-trailing-comma selection 0"
            );
            assert_eq!(
                &s.source()[n.source_location().unwrap().range()],
                "[a, # interior\n ]"
            );
        }
        {
            let s = model("[a, # first\n {b: c, # inner\n }, # final\n] # out\n");
            let n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 0, end: 41 }),
                "flow-comment-nested selection 0"
            );
            assert_eq!(
                &s.source()[n.source_location().unwrap().range()],
                "[a, # first\n {b: c, # inner\n }, # final\n]"
            );
        }
        {
            let s = model("[a, # first\n {b: c, # inner\n }, # final\n] # out\n");
            let mut n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            n = n.entry(1).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 13, end: 30 }),
                "flow-comment-nested selection 1"
            );
            assert_eq!(
                &s.source()[n.source_location().unwrap().range()],
                "{b: c, # inner\n }"
            );
        }
        {
            let s = model("!!seq &anchor [a, {b: c}] # tail\n");
            let n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 14, end: 25 }),
                "flow-properties-outside selection 0"
            );
            assert_eq!(
                &s.source()[n.source_location().unwrap().range()],
                "[a, {b: c}]"
            );
        }
        {
            let s = model("!!seq &anchor [a, {b: c}] # tail\n");
            let mut n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            n = n.entry(1).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 18, end: 24 }),
                "flow-properties-outside selection 1"
            );
            assert_eq!(&s.source()[n.source_location().unwrap().range()], "{b: c}");
        }
        {
            let s = model("&a [*a]\n");
            let n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 3, end: 7 }),
                "flow-alias-cycle selection 0"
            );
            assert_eq!(&s.source()[n.source_location().unwrap().range()], "[*a]");
        }
        {
            let s = model("[&x {a: b}, *x]\n");
            let n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 0, end: 15 }),
                "flow-properties-inside selection 0"
            );
            assert_eq!(
                &s.source()[n.source_location().unwrap().range()],
                "[&x {a: b}, *x]"
            );
        }
        {
            let s = model("[&x {a: b}, *x]\n");
            let mut n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            n = n.entry(0).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 4, end: 10 }),
                "flow-properties-inside selection 1"
            );
            assert_eq!(&s.source()[n.source_location().unwrap().range()], "{a: b}");
        }
        {
            let s = model("[\"[]{}\", {k: \"]},#\"}, '[x]']\n");
            let n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 0, end: 28 }),
                "flow-quoted-delimiters selection 0"
            );
            assert_eq!(
                &s.source()[n.source_location().unwrap().range()],
                "[\"[]{}\", {k: \"]},#\"}, '[x]']"
            );
        }
        {
            let s = model("[\"[]{}\", {k: \"]},#\"}, '[x]']\n");
            let mut n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            n = n.entry(1).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 9, end: 20 }),
                "flow-quoted-delimiters selection 1"
            );
            assert_eq!(
                &s.source()[n.source_location().unwrap().range()],
                "{k: \"]},#\"}"
            );
        }
        {
            let s = model("%YAML 1.2\n---\n[]\n...\n---\n{}\n...\n");
            let n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 14, end: 16 }),
                "flow-two-documents selection 0"
            );
            assert_eq!(&s.source()[n.source_location().unwrap().range()], "[]");
        }
        {
            let s = model("%YAML 1.2\n---\n[]\n...\n---\n{}\n...\n");
            let n = s.root().entry(1).unwrap().value().entry(0).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 25, end: 27 }),
                "flow-two-documents selection 1"
            );
            assert_eq!(&s.source()[n.source_location().unwrap().range()], "{}");
        }
        {
            let s = model("[a: b]\n");
            let n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 0, end: 6 }),
                "flow-implicit-mapping-in-sequence selection 0"
            );
            assert_eq!(&s.source()[n.source_location().unwrap().range()], "[a: b]");
        }
        {
            let s = model("[a: b]\n");
            let mut n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            n = n.entry(0).unwrap().value();
            assert_eq!(
                n.source_location(),
                None,
                "flow-implicit-mapping-in-sequence selection 1"
            );
        }
        {
            let s = model("{a: b}: c\n");
            let n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            assert_eq!(
                n.source_location(),
                None,
                "flow-block-parent-first-map-key selection 0"
            );
        }
        {
            let s = model("{a: b}: c\n");
            let mut n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            n = n.entry(0).unwrap().key().unwrap();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 0, end: 6 }),
                "flow-block-parent-first-map-key selection 1"
            );
            assert_eq!(&s.source()[n.source_location().unwrap().range()], "{a: b}");
        }
        {
            let s = model("[a, b]: c\n");
            let n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            assert_eq!(
                n.source_location(),
                None,
                "flow-block-parent-first-sequence-key selection 0"
            );
        }
        {
            let s = model("[a, b]: c\n");
            let mut n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            n = n.entry(0).unwrap().key().unwrap();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 0, end: 6 }),
                "flow-block-parent-first-sequence-key selection 1"
            );
            assert_eq!(&s.source()[n.source_location().unwrap().range()], "[a, b]");
        }
        {
            let s = model("outer:\n  one: []\n  two: {x: y}\n");
            let n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            assert_eq!(n.source_location(), None, "flow-block-values selection 0");
        }
        {
            let s = model("outer:\n  one: []\n  two: {x: y}\n");
            let mut n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            n = n.entry(0).unwrap().value();
            assert_eq!(n.source_location(), None, "flow-block-values selection 1");
        }
        {
            let s = model("outer:\n  one: []\n  two: {x: y}\n");
            let mut n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            n = n.entry(0).unwrap().value();
            n = n.entry(0).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 14, end: 16 }),
                "flow-block-values selection 2"
            );
            assert_eq!(&s.source()[n.source_location().unwrap().range()], "[]");
        }
        {
            let s = model("outer:\n  one: []\n  two: {x: y}\n");
            let mut n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            n = n.entry(0).unwrap().value();
            n = n.entry(1).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 24, end: 30 }),
                "flow-block-values selection 3"
            );
            assert_eq!(&s.source()[n.source_location().unwrap().range()], "{x: y}");
        }
        {
            let s = model("[[], []]\n");
            let n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 0, end: 8 }),
                "flow-repeated-identical selection 0"
            );
            assert_eq!(
                &s.source()[n.source_location().unwrap().range()],
                "[[], []]"
            );
        }
        {
            let s = model("[[], []]\n");
            let mut n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            n = n.entry(0).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 1, end: 3 }),
                "flow-repeated-identical selection 1"
            );
            assert_eq!(&s.source()[n.source_location().unwrap().range()], "[]");
        }
        {
            let s = model("[[], []]\n");
            let mut n = s.root().entry(0).unwrap().value().entry(0).unwrap().value();
            n = n.entry(1).unwrap().value();
            assert_eq!(
                n.source_location(),
                Some(SourceSpan { start: 5, end: 7 }),
                "flow-repeated-identical selection 2"
            );
            assert_eq!(&s.source()[n.source_location().unwrap().range()], "[]");
        }
    }
    #[test]
    fn independent_original_82_states_and_53_scalar_spans() {
        {
            let source = "~\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("null-tilde: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("~"), "null-tilde");
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 0, end: 1 }),
                    "null-tilde root"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "~");
            }
        }
        {
            let source = "null\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("null-word: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("null"), "null-word");
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 0, end: 4 }),
                    "null-word root"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "null");
            }
        }
        {
            let source = "NULL\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("null-upper: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("NULL"), "null-upper");
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 0, end: 4 }),
                    "null-upper root"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "NULL");
            }
        }
        {
            let source = "true\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("bool-true: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("true"), "bool-true");
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 0, end: 4 }),
                    "bool-true root"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "true");
            }
        }
        {
            let source = "FALSE\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("bool-false: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("FALSE"), "bool-false");
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 0, end: 5 }),
                    "bool-false root"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "FALSE");
            }
        }
        {
            let source = "0\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("integer-zero: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("0"), "integer-zero");
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 0, end: 1 }),
                    "integer-zero root"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "0");
            }
        }
        {
            let source = "012\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result
                .unwrap_or_else(|e| panic!("integer-leading-zero: {e:?}"))
                .0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("012"), "integer-leading-zero");
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 0, end: 3 }),
                    "integer-leading-zero root"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "012");
            }
        }
        {
            let source = "-12\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result
                .unwrap_or_else(|e| panic!("integer-negative: {e:?}"))
                .0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("-12"), "integer-negative");
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 0, end: 3 }),
                    "integer-negative root"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "-12");
            }
        }
        {
            let source = "0xFF\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("integer-hex: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("0xFF"), "integer-hex");
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 0, end: 4 }),
                    "integer-hex root"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "0xFF");
            }
        }
        {
            let source = "0o17\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("integer-octal: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("0o17"), "integer-octal");
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 0, end: 4 }),
                    "integer-octal root"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "0o17");
            }
        }
        {
            let source = "1.\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("float-dot: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("1."), "float-dot");
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 0, end: 2 }),
                    "float-dot root"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "1.");
            }
        }
        {
            let source = "1e+999999\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("float-exponent: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("1e+999999"), "float-exponent");
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 0, end: 9 }),
                    "float-exponent root"
                );
                assert_eq!(
                    &s.source()[n.source_location().unwrap().range()],
                    "1e+999999"
                );
            }
        }
        {
            let source = "-.Inf\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("float-inf: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("-.Inf"), "float-inf");
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 0, end: 5 }),
                    "float-inf root"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "-.Inf");
            }
        }
        {
            let source = ".NaN\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("float-nan: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some(".NaN"), "float-nan");
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 0, end: 4 }),
                    "float-nan root"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], ".NaN");
            }
        }
        {
            let source = "yes\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("string-yes: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("yes"), "string-yes");
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 0, end: 3 }),
                    "string-yes root"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "yes");
            }
        }
        {
            let source = "on\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("string-on: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("on"), "string-on");
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 0, end: 2 }),
                    "string-on root"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "on");
            }
        }
        {
            let source = "2026-10-05\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("string-date: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("2026-10-05"), "string-date");
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 0, end: 10 }),
                    "string-date root"
                );
                assert_eq!(
                    &s.source()[n.source_location().unwrap().range()],
                    "2026-10-05"
                );
            }
        }
        {
            let source = "1_000\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result
                .unwrap_or_else(|e| panic!("string-underscore: {e:?}"))
                .0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("1_000"), "string-underscore");
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 0, end: 5 }),
                    "string-underscore root"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "1_000");
            }
        }
        {
            let source = "\"null\"\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("quoted-null: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("null"), "quoted-null");
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 0, end: 6 }),
                    "quoted-null root"
                );
                assert_eq!(
                    &s.source()[n.source_location().unwrap().range()],
                    "\"null\""
                );
            }
        }
        {
            let source = "\"\"\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("quoted-empty: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some(""), "quoted-empty");
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 0, end: 2 }),
                    "quoted-empty root"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "\"\"");
            }
        }
        {
            let source = "'it''s 界'\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("single-quote: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("it's 界"), "single-quote");
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 0, end: 11 }),
                    "single-quote root"
                );
                assert_eq!(
                    &s.source()[n.source_location().unwrap().range()],
                    "'it''s 界'"
                );
            }
        }
        {
            let source_owned = format!("{}\n", "9".repeat(4300));
            let source = source_owned.as_str();
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("integer-4300: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(
                    root(&s).decoded(),
                    Some("9".repeat(4300).as_str()),
                    "integer-4300"
                );
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan {
                        start: 0,
                        end: 4300
                    }),
                    "integer-4300 root"
                );
                assert_eq!(
                    &s.source()[n.source_location().unwrap().range()],
                    "9".repeat(4300).as_str()
                );
            }
        }
        {
            let source_owned = format!("{}\n", "9".repeat(4301));
            let source = source_owned.as_str();
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("integer-4301: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(
                    root(&s).decoded(),
                    Some("9".repeat(4301).as_str()),
                    "integer-4301"
                );
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan {
                        start: 0,
                        end: 4301
                    }),
                    "integer-4301 root"
                );
                assert_eq!(
                    &s.source()[n.source_location().unwrap().range()],
                    "9".repeat(4301).as_str()
                );
            }
        }
        {
            let source_owned = format!("{}\n", "9".repeat(65536));
            let source = source_owned.as_str();
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("integer-65536: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(
                    root(&s).decoded(),
                    Some("9".repeat(65536).as_str()),
                    "integer-65536"
                );
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan {
                        start: 0,
                        end: 65536
                    }),
                    "integer-65536 root"
                );
                assert_eq!(
                    &s.source()[n.source_location().unwrap().range()],
                    "9".repeat(65536).as_str()
                );
            }
        }
        {
            let source = "";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("empty-stream: {e:?}")).0;
            assert_eq!(s.source(), source);
            assert_eq!(s.root().entries().len(), 0);
        }
        {
            let source = "---\n...\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("empty-document: {e:?}")).0;
            assert_eq!(s.source(), source);
            assert_eq!(s.root().entries().len(), 1);
        }
        {
            let source = "a:\nb: \"\"\nc: null\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("empty-value: {e:?}")).0;
            assert_eq!(s.source(), source);
            {
                let n = root(&s)
                    .entries()
                    .find(|e| e.key().and_then(|k| k.decoded()) == Some("b"))
                    .unwrap()
                    .value();
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 6, end: 8 }),
                    "empty-value b.value"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "\"\"");
            }
            {
                let n = root(&s)
                    .entries()
                    .find(|e| e.key().and_then(|k| k.decoded()) == Some("c"))
                    .unwrap()
                    .value();
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 12, end: 16 }),
                    "empty-value c.value"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "null");
            }
        }
        {
            let source = "---\n...\n--- null\n...\n--- \"\"\n...\n--- []\n...\n--- {}\n...\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("documents: {e:?}")).0;
            assert_eq!(s.source(), source);
            assert_eq!(s.root().entries().len(), 5);
            {
                let n = s.root().entry(1).unwrap().value().entry(0).unwrap().value();
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 12, end: 16 }),
                    "documents doc1.root"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "null");
            }
            {
                let n = s.root().entry(2).unwrap().value().entry(0).unwrap().value();
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 25, end: 27 }),
                    "documents doc2.root"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "\"\"");
            }
            {
                let n = s.root().entry(3).unwrap().value().entry(0).unwrap().value();
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 36, end: 38 }),
                    "documents doc3.root"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "[]");
            }
            {
                let n = s.root().entry(4).unwrap().value().entry(0).unwrap().value();
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 47, end: 49 }),
                    "documents doc4.root"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "{}");
            }
        }
        {
            let source = "? [a, b]\n: sequence-key\n? {x: y}\n: mapping-key\n? null\n: null-key\na.b: dotted\n\"a\": first\n\"\\u0061\": second\n<<: merge-is-ordinary\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result
                .unwrap_or_else(|e| panic!("ordered-complex-keys: {e:?}"))
                .0;
            assert_eq!(s.source(), source);
            {
                let n = root(&s).entry(0).unwrap().key().unwrap();
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 2, end: 8 }),
                    "ordered-complex-keys pair0.key"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "[a, b]");
            }
            {
                let n = root(&s).entry(1).unwrap().key().unwrap();
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 26, end: 32 }),
                    "ordered-complex-keys pair1.key"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "{x: y}");
            }
            {
                let n = root(&s).entry(5).unwrap().key().unwrap();
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 88, end: 96 }),
                    "ordered-complex-keys pair5.key"
                );
                assert_eq!(
                    &s.source()[n.source_location().unwrap().range()],
                    "\"\\u0061\""
                );
            }
        }
        {
            let source = "!!int \"12\"\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("tag-int-quoted: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("12"), "tag-int-quoted");
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 6, end: 10 }),
                    "tag-int-quoted scalar"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "\"12\"");
            }
        }
        {
            let source = "!!int \"\\u0031\\u0032\"\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result
                .unwrap_or_else(|e| panic!("tag-int-escaped: {e:?}"))
                .0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("12"), "tag-int-escaped");
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 6, end: 20 }),
                    "tag-int-escaped scalar"
                );
                assert_eq!(
                    &s.source()[n.source_location().unwrap().range()],
                    "\"\\u0031\\u0032\""
                );
            }
        }
        {
            let source = "!!str true\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("tag-str-plain: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("true"), "tag-str-plain");
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 6, end: 10 }),
                    "tag-str-plain scalar"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "true");
            }
        }
        {
            let source = "! true\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result
                .unwrap_or_else(|e| panic!("tag-nonspecific: {e:?}"))
                .0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("true"), "tag-nonspecific");
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 2, end: 6 }),
                    "tag-nonspecific scalar"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "true");
            }
        }
        {
            let source = "!custom true\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("tag-custom: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("true"), "tag-custom");
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 8, end: 12 }),
                    "tag-custom scalar"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "true");
            }
        }
        {
            let source = "!<tag:yaml.org,2002:int> \"12\"\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result
                .unwrap_or_else(|e| panic!("tag-verbatim-standard: {e:?}"))
                .0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("12"), "tag-verbatim-standard");
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 25, end: 29 }),
                    "tag-verbatim-standard scalar"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "\"12\"");
            }
        }
        {
            let source = "%TAG !! tag:example.test,2026:\n---\n!!int \"12\"\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result
                .unwrap_or_else(|e| panic!("tag-redefined-standard: {e:?}"))
                .0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("12"), "tag-redefined-standard");
            }
            {
                let n = root(&s);
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 41, end: 45 }),
                    "tag-redefined-standard scalar"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "\"12\"");
            }
        }
        {
            let source = "!!int \"abc\"\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            assert!(result.is_err(), "tag-int-invalid must be unavailable");
        }
        {
            let source = "!!bool \"12\"\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            assert!(result.is_err(), "tag-bool-invalid must be unavailable");
        }
        {
            let source = "!!null \"true\"\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            assert!(result.is_err(), "tag-null-invalid must be unavailable");
        }
        {
            let source = "!!seq x\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            assert!(result.is_err(), "tag-seq-scalar must be unavailable");
        }
        {
            let source = "!!map []\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            assert!(result.is_err(), "tag-map-sequence must be unavailable");
        }
        {
            let source = "!!str [a]\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            assert!(result.is_err(), "tag-scalar-collection must be unavailable");
        }
        {
            let source =
                "%TAG !a! tag:a.test,2026:\n%TAG !b! tag:b.test,2026:\n---\n[!a!one x, !b!two y]\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result
                .unwrap_or_else(|e| panic!("tag-retains-handles: {e:?}"))
                .0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("xy"), "tag-retains-handles");
            }
        }
        {
            let source = "%TAG !e! tag:first.test,2026:\n--- !e!kind x\n...\n%TAG !e! tag:second.test,2026:\n--- !e!kind y\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result
                .unwrap_or_else(|e| panic!("tag-document-reset: {e:?}"))
                .0;
            assert_eq!(s.source(), source);
            assert_eq!(s.root().entries().len(), 2);
        }
        {
            let source = "%TAG !e! tag:first.test,2026:\n--- !e!kind x\n...\n--- !e!kind y\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            assert!(
                result.is_err(),
                "tag-missing-next-document must be unavailable"
            );
        }
        {
            let source = "%TAG !e! tag:example.test,2026:\n--- !e!%41 x\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result
                .unwrap_or_else(|e| panic!("tag-suffix-pct41: {e:?}"))
                .0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("x"), "tag-suffix-pct41");
            }
        }
        {
            let source = "%TAG !e! tag:example.test,2026:%41/\n--- !e!kind x\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result
                .unwrap_or_else(|e| panic!("tag-prefix-pct41: {e:?}"))
                .0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("x"), "tag-prefix-pct41");
            }
        }
        {
            let source = "!<tag:example.test,2026:%41> x\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result
                .unwrap_or_else(|e| panic!("tag-verbatim-pct41: {e:?}"))
                .0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("x"), "tag-verbatim-pct41");
            }
        }
        {
            let source = "%TAG !e! tag:example.test,2026:\n--- !e!A x\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("tag-suffix-A: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("x"), "tag-suffix-A");
            }
        }
        {
            let source = "%TAG !e! tag:example.test,2026:A/\n--- !e!kind x\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("tag-prefix-A: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("x"), "tag-prefix-A");
            }
        }
        {
            let source = "!<tag:example.test,2026:A> x\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("tag-verbatim-A: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("x"), "tag-verbatim-A");
            }
        }
        {
            let source = "%TAG !e! tag:example.test,2026:\n--- !e!%C3%A9 x\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result
                .unwrap_or_else(|e| panic!("tag-suffix-pctC3pctA9: {e:?}"))
                .0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("x"), "tag-suffix-pctC3pctA9");
            }
        }
        {
            let source = "%TAG !e! tag:example.test,2026:%C3%A9/\n--- !e!kind x\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result
                .unwrap_or_else(|e| panic!("tag-prefix-pctC3pctA9: {e:?}"))
                .0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("x"), "tag-prefix-pctC3pctA9");
            }
        }
        {
            let source = "!<tag:example.test,2026:%C3%A9> x\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result
                .unwrap_or_else(|e| panic!("tag-verbatim-pctC3pctA9: {e:?}"))
                .0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("x"), "tag-verbatim-pctC3pctA9");
            }
        }
        {
            let source = "%TAG !e! tag:example.test,2026:\n--- !e!%E7%95%8C x\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result
                .unwrap_or_else(|e| panic!("tag-suffix-pctE7pct95pct8C: {e:?}"))
                .0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("x"), "tag-suffix-pctE7pct95pct8C");
            }
        }
        {
            let source = "%TAG !e! tag:example.test,2026:%E7%95%8C/\n--- !e!kind x\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result
                .unwrap_or_else(|e| panic!("tag-prefix-pctE7pct95pct8C: {e:?}"))
                .0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("x"), "tag-prefix-pctE7pct95pct8C");
            }
        }
        {
            let source = "!<tag:example.test,2026:%E7%95%8C> x\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result
                .unwrap_or_else(|e| panic!("tag-verbatim-pctE7pct95pct8C: {e:?}"))
                .0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(
                    root(&s).decoded(),
                    Some("x"),
                    "tag-verbatim-pctE7pct95pct8C"
                );
            }
        }
        {
            let source = "!<tag:yaml.org,2002:%69nt> \"12\"\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result
                .unwrap_or_else(|e| panic!("tag-percent-standard-lookalike: {e:?}"))
                .0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(
                    root(&s).decoded(),
                    Some("12"),
                    "tag-percent-standard-lookalike"
                );
            }
        }
        {
            let source = "%YAML 1.1\n---\na: 1\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            assert!(result.is_err(), "version-1-1 must be unavailable");
        }
        {
            let source = "%YAML 1.2\n---\na: 1\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("version-1-2: {e:?}")).0;
            assert_eq!(s.source(), source);
        }
        {
            let source = "%YAML 1.3\n---\na: 1\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            assert!(result.is_err(), "version-1-3 must be unavailable");
        }
        {
            let source = "%YAML 2.0\n---\na: 1\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            assert!(result.is_err(), "version-2-0 must be unavailable");
        }
        {
            let source = "%YAML 1.2\n---\na: 1\n...\n%YAML 1.1\n---\nb: 2\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            assert!(result.is_err(), "version-late-invalid must be unavailable");
        }
        {
            let source = "- &a first\n- *a\n- &a second\n- *a\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("alias-rebind: {e:?}")).0;
            assert_eq!(s.source(), source);
            {
                let n = (0..s.node_count())
                    .filter_map(|i| s.node(NodeId(i as u32)))
                    .find(|n| {
                        n.kind() == StructureKind::Alias
                            && n.source_location() == Some(SourceSpan { start: 13, end: 15 })
                    })
                    .unwrap();
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 13, end: 15 }),
                    "alias-rebind alias1"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "*a");
            }
            {
                let n = (0..s.node_count())
                    .filter_map(|i| s.node(NodeId(i as u32)))
                    .find(|n| {
                        n.kind() == StructureKind::Alias
                            && n.source_location() == Some(SourceSpan { start: 30, end: 32 })
                    })
                    .unwrap();
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 30, end: 32 }),
                    "alias-rebind alias3"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "*a");
            }
        }
        {
            let source = "&a [*a]\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("alias-cycle: {e:?}")).0;
            assert_eq!(s.source(), source);
            {
                let n = (0..s.node_count())
                    .filter_map(|i| s.node(NodeId(i as u32)))
                    .find(|n| {
                        n.kind() == StructureKind::Alias
                            && n.source_location() == Some(SourceSpan { start: 4, end: 6 })
                    })
                    .unwrap();
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 4, end: 6 }),
                    "alias-cycle alias"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "*a");
            }
        }
        {
            let source = "- &a [x, y]\n- *a\n- *a\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("alias-dag: {e:?}")).0;
            assert_eq!(s.source(), source);
        }
        {
            let source = "[*a, &a x]\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            assert!(result.is_err(), "alias-forward must be unavailable");
        }
        {
            let source = "--- &a x\n...\n--- *a\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            assert!(result.is_err(), "alias-document-scope must be unavailable");
        }
        {
            let source = "# 前置\r\na: \"界\\U0001F98A\\u0041\" # 外部\r\nb: 'é'\r\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("spans-crlf: {e:?}")).0;
            assert_eq!(s.source(), source);
            {
                let n = root(&s)
                    .entries()
                    .find(|e| e.key().and_then(|k| k.decoded()) == Some("a"))
                    .unwrap()
                    .value();
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 13, end: 34 }),
                    "spans-crlf a.value"
                );
                assert_eq!(
                    &s.source()[n.source_location().unwrap().range()],
                    "\"界\\U0001F98A\\u0041\""
                );
            }
            {
                let n = root(&s)
                    .entries()
                    .find(|e| e.key().and_then(|k| k.decoded()) == Some("b"))
                    .unwrap()
                    .value();
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 48, end: 53 }),
                    "spans-crlf b.value"
                );
                assert_eq!(&s.source()[n.source_location().unwrap().range()], "'é'");
            }
        }
        {
            let source = "k: |-\n  界\n  🦊\nnext: z\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result
                .unwrap_or_else(|e| panic!("block-literal-strip: {e:?}"))
                .0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("界\n🦊"), "block-literal-strip");
            }
            {
                let n = root(&s)
                    .entries()
                    .find(|e| e.key().and_then(|k| k.decoded()) == Some("k"))
                    .unwrap()
                    .value();
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 3, end: 19 }),
                    "block-literal-strip k.value"
                );
                assert_eq!(
                    &s.source()[n.source_location().unwrap().range()],
                    "|-\n  界\n  🦊\n"
                );
            }
        }
        {
            let source = "k: |\n  a\nnext: z\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result
                .unwrap_or_else(|e| panic!("block-literal-clip: {e:?}"))
                .0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("a\n"), "block-literal-clip");
            }
            {
                let n = root(&s)
                    .entries()
                    .find(|e| e.key().and_then(|k| k.decoded()) == Some("k"))
                    .unwrap()
                    .value();
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 3, end: 9 }),
                    "block-literal-clip k.value"
                );
                assert_eq!(
                    &s.source()[n.source_location().unwrap().range()],
                    "|\n  a\n"
                );
            }
        }
        {
            let source = "k: |+\n  a\n\nnext: z\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result
                .unwrap_or_else(|e| panic!("block-literal-keep: {e:?}"))
                .0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("a\n\n"), "block-literal-keep");
            }
            {
                let n = root(&s)
                    .entries()
                    .find(|e| e.key().and_then(|k| k.decoded()) == Some("k"))
                    .unwrap()
                    .value();
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 3, end: 11 }),
                    "block-literal-keep k.value"
                );
                assert_eq!(
                    &s.source()[n.source_location().unwrap().range()],
                    "|+\n  a\n\n"
                );
            }
        }
        {
            let source = "k: >-\n  first\n  second\nnext: z\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("block-folded: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("first second"), "block-folded");
            }
            {
                let n = root(&s)
                    .entries()
                    .find(|e| e.key().and_then(|k| k.decoded()) == Some("k"))
                    .unwrap()
                    .value();
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 3, end: 23 }),
                    "block-folded k.value"
                );
                assert_eq!(
                    &s.source()[n.source_location().unwrap().range()],
                    ">-\n  first\n  second\n"
                );
            }
        }
        {
            let source = "k: |2-\n  a\nnext: z\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("block-indent: {e:?}")).0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("a"), "block-indent");
            }
            {
                let n = root(&s)
                    .entries()
                    .find(|e| e.key().and_then(|k| k.decoded()) == Some("k"))
                    .unwrap()
                    .value();
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 3, end: 11 }),
                    "block-indent k.value"
                );
                assert_eq!(
                    &s.source()[n.source_location().unwrap().range()],
                    "|2-\n  a\n"
                );
            }
        }
        {
            let source = "k: |- # header comment\n  a\nnext: z\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result
                .unwrap_or_else(|e| panic!("block-header-comment: {e:?}"))
                .0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(root(&s).decoded(), Some("a"), "block-header-comment");
            }
            {
                let n = root(&s)
                    .entries()
                    .find(|e| e.key().and_then(|k| k.decoded()) == Some("k"))
                    .unwrap()
                    .value();
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 3, end: 27 }),
                    "block-header-comment k.value"
                );
                assert_eq!(
                    &s.source()[n.source_location().unwrap().range()],
                    "|- # header comment\n  a\n"
                );
            }
        }
        {
            let source = "escaped: \"\\e TITLE\\x9b\"\nglyph: \"literal ␛\"\nc1_text: \"literal \\\\u{009b}\"\nkey: \"\\u754c é 🦊\"\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result
                .unwrap_or_else(|e| panic!("controls-search: {e:?}"))
                .0;
            assert_eq!(s.source(), source);
            {
                let n = root(&s)
                    .entries()
                    .find(|e| e.key().and_then(|k| k.decoded()) == Some("escaped"))
                    .unwrap()
                    .value();
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 9, end: 23 }),
                    "controls-search escaped.value"
                );
                assert_eq!(
                    &s.source()[n.source_location().unwrap().range()],
                    "\"\\e TITLE\\x9b\""
                );
            }
            {
                let n = root(&s)
                    .entries()
                    .find(|e| e.key().and_then(|k| k.decoded()) == Some("key"))
                    .unwrap()
                    .value();
                assert_eq!(
                    n.source_location(),
                    Some(SourceSpan { start: 79, end: 96 }),
                    "controls-search unicode.value"
                );
                assert_eq!(
                    &s.source()[n.source_location().unwrap().range()],
                    "\"\\u754c é 🦊\""
                );
            }
        }
        {
            let source = "[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[x]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result.unwrap_or_else(|e| panic!("depth-64: {e:?}")).0;
            assert_eq!(s.source(), source);
        }
        {
            let source = "? [[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[x]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]\n: v\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result
                .unwrap_or_else(|e| panic!("complex-key-depth-64: {e:?}"))
                .0;
            assert_eq!(s.source(), source);
        }
        {
            let source = "[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[x]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            assert!(result.is_err(), "depth-65 must be unavailable");
        }
        {
            let source = "? [[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[x]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]\n: v\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            assert!(result.is_err(), "complex-key-depth-65 must be unavailable");
        }
        {
            let source_owned = format!("\"{}END\"\n", "L".repeat(65536));
            let source = source_owned.as_str();
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            let s = result
                .unwrap_or_else(|e| panic!("token-large-quoted: {e:?}"))
                .0;
            assert_eq!(s.source(), source);
            if root(&s).decoded().is_some() {
                assert_eq!(
                    root(&s).decoded(),
                    Some(format!("{}END", "L".repeat(65536)).as_str()),
                    "token-large-quoted"
                );
            }
        }
        {
            let source = "---\na: 1\n...\n---\n[unterminated\n";
            let result = project(Arc::new(source.to_owned()), 23, &AtomicBool::new(false));
            assert!(result.is_err(), "stream-malformed-late must be unavailable");
        }
    }
}
