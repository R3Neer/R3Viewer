//! Fixed, bounded IPC for the one-shot structure subprocess. Independent of Nu/syntax.
use crate::structure_model::*;
use std::io::{self, Read, Write};
pub(crate) const CHUNK: usize = 65_536;
pub(crate) const MAX_FRAME: usize = 1024 * 1024;
pub(crate) const BATCH: usize = 2048;
#[derive(Debug, Clone, Copy)]
pub(crate) struct Counts {
    pub nodes: u32,
    pub entries: u32,
    pub declarations: u32,
    pub parts: u32,
    pub cooked: u32,
    pub root: NodeId,
    pub syntax_depth: u32,
    pub key_components: u32,
}
#[derive(Debug, Clone, Copy)]
pub(crate) struct WireNode {
    pub spec: NodeSpec,
    pub entries: u32,
}
#[derive(Debug)]
pub(crate) enum Message {
    Boot,
    Init {
        bytes: u32,
    },
    Ack {
        offset: u32,
    },
    Source {
        offset: u32,
        bytes: Vec<u8>,
    },
    SourceEnd,
    Begin(Counts),
    Nodes {
        offset: u32,
        nodes: Vec<WireNode>,
    },
    Entries {
        parent: NodeId,
        entries: Vec<EntrySpec>,
    },
    Cooked {
        offset: u32,
        bytes: Vec<u8>,
    },
    Declaration {
        spec: DeclarationSpec,
        parts: Vec<DeclarationPart>,
    },
    End,
    Failure(u8),
}
#[derive(Debug)]
pub(crate) struct Frame {
    pub generation: u64,
    pub request: u64,
    pub message: Message,
}
impl Frame {
    pub(crate) fn new(generation: u64, request: u64, message: Message) -> Self {
        Self {
            generation,
            request,
            message,
        }
    }
}
fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid structure IPC")
}
struct Encode(Vec<u8>);
impl Encode {
    fn byte(&mut self, n: u8) {
        self.0.push(n)
    }
    fn word(&mut self, n: u32) {
        self.0.extend_from_slice(&n.to_le_bytes())
    }
    fn span(&mut self, s: SourceSpan) {
        self.word(s.start);
        self.word(s.end)
    }
    fn location(&mut self, s: Option<SourceSpan>) {
        self.byte(u8::from(s.is_some()));
        if let Some(s) = s {
            self.span(s)
        }
    }
    fn text(&mut self, t: Option<TextRef>) {
        match t {
            None => self.byte(0),
            Some(TextRef::Source(s)) => {
                self.byte(1);
                self.span(s)
            }
            Some(TextRef::Cooked { start, end }) => {
                self.byte(2);
                self.word(start);
                self.word(end)
            }
        }
    }
    fn entries(&mut self, entries: &[EntrySpec]) -> io::Result<()> {
        if entries.len() > BATCH {
            return Err(invalid());
        }
        self.word(entries.len() as u32);
        for e in entries {
            self.word(e.key.map_or(u32::MAX, |id| id.0));
            self.word(e.value.0);
        }
        Ok(())
    }
    fn bytes(&mut self, offset: u32, bytes: &[u8]) -> io::Result<()> {
        if bytes.len() > CHUNK {
            return Err(invalid());
        }
        self.word(offset);
        self.word(bytes.len() as u32);
        self.0.extend_from_slice(bytes);
        Ok(())
    }
}
pub(crate) fn write_frame(out: &mut impl Write, frame: &Frame) -> io::Result<()> {
    let mut body = Encode(Vec::new());
    body.0
        .try_reserve_exact(CHUNK + 512)
        .map_err(|_| invalid())?;
    let tag = match &frame.message {
        Message::Boot => 0,
        Message::Init { bytes } => {
            body.word(*bytes);
            1
        }
        Message::Ack { offset } => {
            body.word(*offset);
            2
        }
        Message::Source { offset, bytes } => {
            body.bytes(*offset, bytes)?;
            3
        }
        Message::SourceEnd => 4,
        Message::Begin(c) => {
            for n in [
                c.nodes,
                c.entries,
                c.declarations,
                c.parts,
                c.cooked,
                c.root.0,
                c.syntax_depth,
                c.key_components,
            ] {
                body.word(n);
            }
            5
        }
        Message::Nodes { offset, nodes } => {
            if nodes.len() > BATCH {
                return Err(invalid());
            }
            body.word(*offset);
            body.word(nodes.len() as u32);
            for n in nodes {
                body.byte(n.spec.kind as u8);
                body.text(n.spec.text);
                body.location(n.spec.location);
                body.word(n.entries);
            }
            6
        }
        Message::Entries { parent, entries } => {
            body.word(parent.0);
            body.entries(entries)?;
            7
        }
        Message::Cooked { offset, bytes } => {
            body.bytes(*offset, bytes)?;
            8
        }
        Message::Declaration { spec, parts } => {
            if parts.len() > MAX_STRUCTURE_DEPTH {
                return Err(invalid());
            }
            body.byte(spec.kind as u8);
            body.span(spec.span);
            body.span(spec.key_span);
            body.location(spec.value_span);
            body.word(spec.node.0);
            body.word(parts.len() as u32);
            for p in parts {
                body.span(p.span);
                body.word(p.node.0);
            }
            9
        }
        Message::End => 10,
        Message::Failure(reason) => {
            body.byte(*reason);
            11
        }
    };
    if body.0.len() > MAX_FRAME - 28 {
        return Err(invalid());
    }
    let mut header = [0u8; 28];
    header[..4].copy_from_slice(b"R3ST");
    header[4] = 1;
    header[6] = tag;
    header[8..16].copy_from_slice(&frame.generation.to_le_bytes());
    header[16..24].copy_from_slice(&frame.request.to_le_bytes());
    header[24..].copy_from_slice(&(body.0.len() as u32).to_le_bytes());
    out.write_all(&header)?;
    out.write_all(&body.0)?;
    out.flush()
}
struct Decode<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Decode<'a> {
    fn take(&mut self, n: usize) -> io::Result<&'a [u8]> {
        let end = self.at.checked_add(n).ok_or_else(invalid)?;
        let b = self.bytes.get(self.at..end).ok_or_else(invalid)?;
        self.at = end;
        Ok(b)
    }
    fn byte(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn word(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn span(&mut self) -> io::Result<SourceSpan> {
        Ok(SourceSpan {
            start: self.word()?,
            end: self.word()?,
        })
    }
    fn location(&mut self) -> io::Result<Option<SourceSpan>> {
        match self.byte()? {
            0 => Ok(None),
            1 => Ok(Some(self.span()?)),
            _ => Err(invalid()),
        }
    }
    fn text(&mut self) -> io::Result<Option<TextRef>> {
        match self.byte()? {
            0 => Ok(None),
            1 => Ok(Some(TextRef::Source(self.span()?))),
            2 => Ok(Some(TextRef::Cooked {
                start: self.word()?,
                end: self.word()?,
            })),
            _ => Err(invalid()),
        }
    }
    fn count(&mut self, max: usize) -> io::Result<usize> {
        let n = self.word()? as usize;
        if n > max {
            return Err(invalid());
        }
        Ok(n)
    }
    fn bytes(&mut self) -> io::Result<(u32, Vec<u8>)> {
        let offset = self.word()?;
        let len = self.count(CHUNK)?;
        let bytes = self.take(len)?;
        let mut owned = Vec::new();
        owned.try_reserve_exact(len).map_err(|_| invalid())?;
        owned.extend_from_slice(bytes);
        Ok((offset, owned))
    }
}
pub(crate) fn read_frame(input: &mut impl Read) -> io::Result<Option<Frame>> {
    let mut header = [0u8; 28];
    loop {
        match input.read(&mut header[..1]) {
            Ok(0) => return Ok(None),
            Ok(_) => break,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
    input.read_exact(&mut header[1..])?;
    if &header[..4] != b"R3ST" || header[4] != 1 || header[5] != 0 || header[7] != 0 {
        return Err(invalid());
    }
    let len = u32::from_le_bytes(header[24..28].try_into().unwrap()) as usize;
    if len > MAX_FRAME - 28 {
        return Err(invalid());
    }
    let mut body = Vec::new();
    body.try_reserve_exact(len).map_err(|_| invalid())?;
    body.resize(len, 0);
    input.read_exact(&mut body)?;
    let mut d = Decode {
        bytes: &body,
        at: 0,
    };
    let message = match header[6] {
        0 => Message::Boot,
        1 => Message::Init { bytes: d.word()? },
        2 => Message::Ack { offset: d.word()? },
        3 => {
            let (offset, bytes) = d.bytes()?;
            Message::Source { offset, bytes }
        }
        4 => Message::SourceEnd,
        5 => Message::Begin(Counts {
            nodes: d.word()?,
            entries: d.word()?,
            declarations: d.word()?,
            parts: d.word()?,
            cooked: d.word()?,
            root: NodeId(d.word()?),
            syntax_depth: d.word()?,
            key_components: d.word()?,
        }),
        6 => {
            let offset = d.word()?;
            let count = d.count(BATCH)?;
            let mut nodes = Vec::new();
            nodes.try_reserve_exact(count).map_err(|_| invalid())?;
            for _ in 0..count {
                let kind = match d.byte()? {
                    0 => StructureKind::Mapping,
                    1 => StructureKind::Sequence,
                    2 => StructureKind::ArrayOfTables,
                    3 => StructureKind::String,
                    4 => StructureKind::Boolean,
                    5 => StructureKind::Integer,
                    6 => StructureKind::Float,
                    7 => StructureKind::OffsetDateTime,
                    8 => StructureKind::LocalDateTime,
                    9 => StructureKind::LocalDate,
                    10 => StructureKind::LocalTime,
                    _ => return Err(invalid()),
                };
                nodes.push(WireNode {
                    spec: NodeSpec {
                        kind,
                        text: d.text()?,
                        location: d.location()?,
                    },
                    entries: d.word()?,
                });
            }
            Message::Nodes { offset, nodes }
        }
        7 => {
            let parent = NodeId(d.word()?);
            let count = d.count(BATCH)?;
            let mut entries = Vec::new();
            entries.try_reserve_exact(count).map_err(|_| invalid())?;
            for _ in 0..count {
                let key = d.word()?;
                entries.push(EntrySpec {
                    key: (key != u32::MAX).then_some(NodeId(key)),
                    value: NodeId(d.word()?),
                });
            }
            Message::Entries { parent, entries }
        }
        8 => {
            let (offset, bytes) = d.bytes()?;
            Message::Cooked { offset, bytes }
        }
        9 => {
            let kind = match d.byte()? {
                0 => DeclarationKind::Header,
                1 => DeclarationKind::ArrayOfTablesHeader,
                2 => DeclarationKind::Assignment,
                3 => DeclarationKind::InlineAssignment,
                _ => return Err(invalid()),
            };
            let spec = DeclarationSpec {
                kind,
                span: d.span()?,
                key_span: d.span()?,
                value_span: d.location()?,
                node: NodeId(d.word()?),
            };
            let count = d.count(MAX_STRUCTURE_DEPTH)?;
            let mut parts = Vec::new();
            parts.try_reserve_exact(count).map_err(|_| invalid())?;
            for _ in 0..count {
                parts.push(DeclarationPart {
                    span: d.span()?,
                    node: NodeId(d.word()?),
                });
            }
            Message::Declaration { spec, parts }
        }
        10 => Message::End,
        11 => Message::Failure(d.byte()?),
        _ => return Err(invalid()),
    };
    if d.at != body.len() {
        return Err(invalid());
    }
    Ok(Some(Frame {
        generation: u64::from_le_bytes(header[8..16].try_into().unwrap()),
        request: u64::from_le_bytes(header[16..24].try_into().unwrap()),
        message,
    }))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn split_unicode_and_truncation_preserve_transport_contract() {
        let frame = Frame::new(
            7,
            9,
            Message::Cooked {
                offset: 0,
                bytes: vec![0xe7],
            },
        );
        let mut encoded = Vec::new();
        write_frame(&mut encoded, &frame).unwrap();
        let decoded = read_frame(&mut encoded.as_slice()).unwrap().unwrap();
        assert!(matches!(decoded.message,Message::Cooked{offset:0,bytes}if bytes==[0xe7]));
        encoded.pop();
        assert!(read_frame(&mut encoded.as_slice()).is_err());
    }
    #[test]
    fn oversized_header_rejected_before_payload_allocation() {
        let mut bytes = [0u8; 28];
        bytes[..4].copy_from_slice(b"R3ST");
        bytes[4] = 1;
        bytes[24..].copy_from_slice(&((MAX_FRAME + 1) as u32).to_le_bytes());
        assert!(read_frame(&mut bytes.as_slice()).is_err());
    }
}
