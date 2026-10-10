//! Strict pull projection preserving every object occurrence and raw number.
use crate::{
    structure::StructureFailure as Failure, structure_model::*, structure_protocol::CHUNK,
};
use jiter::{Jiter, Peek};
use std::{
    mem::size_of,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
fn error(e: ModelError) -> Failure {
    match e {
        ModelError::Cancelled => Failure::WorkerExited,
        ModelError::Timeout => Failure::Timeout,
        ModelError::SourceLimit => Failure::SourceBudget,
        ModelError::Budget => Failure::ArenaBudget,
        ModelError::Depth => Failure::Depth,
        ModelError::Invalid => Failure::InvalidIpc,
    }
}
fn span(start: usize, end: usize) -> SourceSpan {
    SourceSpan {
        start: start as u32,
        end: end as u32,
    }
}
fn check(cancel: &AtomicBool) -> Result<(), Failure> {
    if cancel.load(Ordering::Acquire) {
        Err(Failure::WorkerExited)
    } else {
        Ok(())
    }
}
fn whitespace(raw: &[u8], mut at: usize) -> usize {
    while raw
        .get(at)
        .is_some_and(|b| matches!(b, b' ' | b'\r' | b'\n' | b'\t'))
    {
        at += 1;
    }
    at
}
fn hex4(raw: &[u8], at: usize) -> Result<u16, Failure> {
    let bytes = raw.get(at..at + 4).ok_or(Failure::InvalidJson)?;
    let mut n = 0;
    for &b in bytes {
        n = n * 16
            + match b {
                b'0'..=b'9' => (b - b'0') as u16,
                b'a'..=b'f' => (b - b'a' + 10) as u16,
                b'A'..=b'F' => (b - b'A' + 10) as u16,
                _ => return Err(Failure::InvalidJson),
            };
    }
    Ok(n)
}
/// Local token-only preflight: no cooked allocation or document grammar replacement.
fn string_span(raw: &[u8], start: usize, cancel: &AtomicBool) -> Result<SourceSpan, Failure> {
    if raw.get(start) != Some(&b'"') {
        return Err(Failure::InvalidJson);
    }
    let mut at = start + 1;
    let mut lone = false;
    let mut last_check = start;
    loop {
        if at - last_check >= 4096 {
            check(cancel)?;
            last_check = at;
        }
        match raw.get(at).copied().ok_or(Failure::InvalidJson)? {
            b'"' => {
                check(cancel)?;
                return if lone {
                    Err(Failure::UnicodeRepresentation)
                } else {
                    Ok(span(start, at + 1))
                };
            }
            b'\\' => {
                at += 1;
                match raw.get(at).copied().ok_or(Failure::InvalidJson)? {
                    b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => at += 1,
                    b'u' => {
                        let n = hex4(raw, at + 1)?;
                        at += 5;
                        if (0xD800..=0xDBFF).contains(&n) {
                            if raw.get(at..at + 2) == Some(b"\\u") {
                                let next = hex4(raw, at + 2)?;
                                if (0xDC00..=0xDFFF).contains(&next) {
                                    at += 6;
                                } else {
                                    lone = true;
                                }
                            } else {
                                lone = true;
                            }
                        } else if (0xDC00..=0xDFFF).contains(&n) {
                            lone = true;
                        }
                    }
                    _ => return Err(Failure::InvalidJson),
                }
            }
            0..=31 => return Err(Failure::InvalidJson),
            _ => at += 1,
        }
    }
}
struct Projector<'a> {
    raw: &'a str,
    parser: Jiter<'a>,
    builder: StructureBuilder,
    cancel: &'a AtomicBool,
    max_depth: usize,
}
impl Projector<'_> {
    fn push(
        &mut self,
        kind: StructureKind,
        text: Option<TextRef>,
        location: SourceSpan,
        scratch: usize,
    ) -> Result<NodeId, Failure> {
        check(self.cancel)?;
        self.builder.set_scratch_bytes(scratch).map_err(error)?;
        self.builder
            .push_node(NodeSpec {
                kind,
                text,
                location: Some(location),
            })
            .map_err(error)
    }
    fn string(&mut self, location: SourceSpan, scratch: usize) -> Result<NodeId, Failure> {
        // The main cursor already consumed this token. Decode only its borrowed raw range.
        let raw = &self.raw[location.range()];
        let mut local = Jiter::new(raw.as_bytes());
        let cooked = local.next_str().map_err(|_| Failure::InvalidJson)?;
        let interior = span(location.start as usize + 1, location.end as usize - 1);
        let text = if &self.raw[interior.range()] == cooked {
            TextRef::Source(interior)
        } else {
            self.builder.set_scratch_bytes(scratch).map_err(error)?;
            let mut start = None;
            let mut end = 0;
            for chunk in cooked.as_bytes().chunks(CHUNK) {
                check(self.cancel)?;
                let range = self.builder.append_cooked(chunk).map_err(error)?;
                start.get_or_insert(range.start);
                end = range.end;
            }
            let start = if let Some(start) = start {
                start
            } else {
                let range = self.builder.append_cooked(&[]).map_err(error)?;
                end = range.end;
                range.start
            };
            TextRef::Cooked { start, end }
        };
        local.finish().map_err(|_| Failure::InvalidJson)?;
        self.push(StructureKind::String, Some(text), location, scratch)
    }
    fn next_key(&mut self, first: bool) -> Result<Option<SourceSpan>, Failure> {
        let bytes = self.raw.as_bytes();
        let mut at = whitespace(bytes, self.parser.current_index());
        if first {
            if bytes.get(at) != Some(&b'{') {
                return Err(Failure::InvalidJson);
            }
            at = whitespace(bytes, at + 1);
        } else if bytes.get(at) == Some(&b',') {
            at = whitespace(bytes, at + 1);
        }
        let expected = if bytes.get(at) == Some(&b'"') {
            Some(string_span(bytes, at, self.cancel)?)
        } else {
            None
        };
        let key = if first {
            self.parser.next_object_bytes()
        } else {
            self.parser.next_key_bytes()
        }
        .map_err(|_| Failure::InvalidJson)?;
        match (key, expected) {
            (None, None) => Ok(None),
            (Some(raw), Some(expected)) => {
                let interior = (expected.start as usize + 1)..(expected.end as usize - 1);
                if raw.as_ptr() != bytes[interior.clone()].as_ptr() || raw.len() != interior.len() {
                    return Err(Failure::InvalidIpc);
                }
                Ok(Some(expected))
            }
            _ => Err(Failure::InvalidIpc),
        }
    }
    fn grow_entries(&self, entries: &mut Vec<EntrySpec>, scratch: usize) -> Result<usize, Failure> {
        reserve(entries, 1, self.builder.persistent_bytes() + scratch).map_err(error)?;
        Ok(scratch + entries.capacity() * size_of::<EntrySpec>())
    }
    fn value(&mut self, depth: usize, scratch: usize) -> Result<NodeId, Failure> {
        check(self.cancel)?;
        if depth > MAX_STRUCTURE_DEPTH {
            return Err(Failure::Depth);
        }
        self.max_depth = self.max_depth.max(depth);
        let peek = self.parser.peek().map_err(|_| Failure::InvalidJson)?;
        let start = whitespace(self.raw.as_bytes(), self.parser.current_index());
        if peek == Peek::Object || peek == Peek::Array {
            if depth >= MAX_STRUCTURE_DEPTH {
                return Err(Failure::Depth);
            }
            self.max_depth = self.max_depth.max(depth + 1);
            let object = peek == Peek::Object;
            let kind = if object {
                StructureKind::Mapping
            } else {
                StructureKind::Sequence
            };
            let id = self.push(kind, None, span(start, start), scratch)?;
            let mut entries = Vec::new();
            if object {
                let mut next = self.next_key(true)?;
                while let Some(key_span) = next {
                    let held = self.grow_entries(&mut entries, scratch)?;
                    let key = self.string(key_span, held)?;
                    let value = self.value(depth + 1, held)?;
                    entries.push(EntrySpec {
                        key: Some(key),
                        value,
                    });
                    next = self.next_key(false)?;
                }
            } else {
                let mut next = self.parser.next_array().map_err(|_| Failure::InvalidJson)?;
                while next.is_some() {
                    let held = self.grow_entries(&mut entries, scratch)?;
                    let value = self.value(depth + 1, held)?;
                    entries.push(EntrySpec { key: None, value });
                    next = self.parser.array_step().map_err(|_| Failure::InvalidJson)?;
                }
            }
            let location = span(start, self.parser.current_index());
            self.builder.set_location(id, location).map_err(error)?;
            self.builder
                .set_scratch_bytes(scratch + entries.capacity() * size_of::<EntrySpec>())
                .map_err(error)?;
            self.builder.set_entries(id, entries).map_err(error)?;
            self.builder.set_scratch_bytes(scratch).map_err(error)?;
            return Ok(id);
        }
        if peek == Peek::String {
            let location = string_span(self.raw.as_bytes(), start, self.cancel)?;
            let raw = self.parser.next_bytes().map_err(|_| Failure::InvalidJson)?;
            let interior = (location.start as usize + 1)..(location.end as usize - 1);
            if raw.as_ptr() != self.raw.as_bytes()[interior.clone()].as_ptr()
                || raw.len() != interior.len()
            {
                return Err(Failure::InvalidIpc);
            }
            return self.string(location, scratch);
        }
        let kind = if peek == Peek::Null {
            self.parser.next_null().map_err(|_| Failure::InvalidJson)?;
            StructureKind::Null
        } else if peek == Peek::True || peek == Peek::False {
            self.parser.next_bool().map_err(|_| Failure::InvalidJson)?;
            StructureKind::Boolean
        } else if peek.is_num() {
            self.parser
                .next_number_bytes()
                .map_err(|_| Failure::InvalidJson)?;
            StructureKind::JsonNumber
        } else {
            return Err(Failure::InvalidJson);
        };
        let location = span(start, self.parser.current_index());
        self.push(kind, Some(TextRef::Source(location)), location, scratch)
    }
}
pub(crate) fn project(
    source: Arc<String>,
    generation: u64,
    cancel: &AtomicBool,
) -> Result<(Structure, usize, usize), Failure> {
    let builder =
        StructureBuilder::new_for_format(source.clone(), generation, StructureFormat::Json)
            .map_err(error)?;
    let mut p = Projector {
        raw: &source,
        parser: Jiter::new(source.as_bytes()),
        builder,
        cancel,
        max_depth: 0,
    };
    let root = p.value(0, 0)?;
    p.parser.finish().map_err(|_| Failure::InvalidJson)?;
    let depth = p.max_depth;
    p.builder.set_scratch_bytes(0).map_err(error)?;
    let model = p.builder.finish(root, depth, 0, cancel).map_err(error)?;
    Ok((model, depth, 0))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn parse(raw: &str) -> Result<Structure, Failure> {
        project(Arc::new(raw.to_owned()), 1, &AtomicBool::new(false)).map(|v| v.0)
    }
    #[test]
    fn roots_raw_numbers_and_strict_eof() {
        for raw in [
            "null",
            "true",
            "0",
            "\"\"",
            "[]",
            "{}",
            "9007199254740993",
            "1e400",
            "-0",
            "1.000",
        ] {
            assert!(parse(raw).is_ok(), "{raw}");
        }
        for raw in [
            "",
            "null false",
            "01",
            "+1",
            "1.",
            "1e",
            "NaN",
            "Infinity",
            "[1,]",
            "{}x",
            "{\"a\":1,}",
        ] {
            assert_eq!(parse(raw).unwrap_err(), Failure::InvalidJson, "{raw}");
        }
        for length in [4300, 4301, 65536] {
            let raw = "9".repeat(length);
            let model = parse(&raw).unwrap();
            assert_eq!(model.root().leaf().unwrap().text(), raw);
        }
    }
    #[test]
    fn ordered_duplicates_raw_spans_and_unicode() {
        let raw = r#"{"b":0,"a":1,"\u0061":2,"a":3,"a.b":"\uD83E\uDD8A"}"#;
        let model = parse(raw).unwrap();
        assert_eq!(model.duplicates().extra_occurrences, 2);
        assert_eq!(
            model
                .root()
                .entries()
                .map(|e| e.key().unwrap().leaf().unwrap().text())
                .collect::<Vec<_>>(),
            ["b", "a", "a", "a", "a.b"]
        );
        assert_eq!(
            &raw[model
                .root()
                .entry(4)
                .unwrap()
                .source_location()
                .unwrap()
                .range()],
            r#""a.b":"\uD83E\uDD8A""#
        );
        let leaf = model.root().entry(4).unwrap().value();
        assert_eq!(leaf.leaf().unwrap().text(), "🦊");
        assert_eq!(
            &raw[leaf.source_location().unwrap().range()],
            r#""\uD83E\uDD8A""#
        );
        assert_eq!(model.declarations().len(), 0);
    }
    #[test]
    fn lone_surrogates_and_invalid_tokens_have_distinct_diagnostics() {
        for raw in [
            r#""\uD800""#,
            r#""\uDC00""#,
            r#"{"\uD800":0}"#,
            r#""\uD800\u0041""#,
        ] {
            assert_eq!(
                parse(raw).unwrap_err(),
                Failure::UnicodeRepresentation,
                "{raw}"
            );
        }
        for raw in [
            r#""\q""#,
            r#""\uD800\uZZZZ""#,
            r#""\uD800"#,
            "\"a\n\"",
            r#""\uZZZZ""#,
        ] {
            assert_eq!(parse(raw).unwrap_err(), Failure::InvalidJson, "{raw}");
        }
    }
    #[test]
    fn nested_spans_and_depth_limits() {
        for depth in [64, 65] {
            let raw = format!("{}0{}", "[".repeat(depth), "]".repeat(depth));
            assert_eq!(parse(&raw).is_ok(), depth == 64);
        }
        assert_eq!(
            parse(&format!("{}{}", "[".repeat(65), "]".repeat(65))).unwrap_err(),
            Failure::Depth
        );
        let raw = " \r\n{\"é\": [true, {\"x\":null}]} \t";
        let model = parse(raw).unwrap();
        assert_eq!(
            &raw[model.root().source_location().unwrap().range()],
            "{\"é\": [true, {\"x\":null}]}"
        );
    }
}
