//! R3MDv2 bounded codec. IO completion and process receipts belong to the controller.
use crate::{
    document_budget::{ActorArena, ArenaVec, Lease},
    document_model::*,
    document_obsidian::Control,
    document_source::DocumentSource,
};
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    sync::Arc,
};
pub(crate) const MAX_BODY: usize = 1_048_576;
pub(crate) const CHUNK: usize = 65_536;
const STRIDES: [usize; 10] = [64, 24, 16, 64, 40, 64, 80, 16, 16, 24];
fn bad() -> Error {
    Error::new(ErrorKind::InvalidIpc)
}
fn io(_: std::io::Error) -> Error {
    bad()
}
fn checked(ok: bool) -> Result<()> {
    if ok { Ok(()) } else { Err(bad()) }
}
fn u32_at(bytes: &[u8], at: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        bytes
            .get(at..at + 4)
            .ok_or_else(bad)?
            .try_into()
            .map_err(|_| bad())?,
    ))
}
fn u64_at(bytes: &[u8], at: usize) -> Result<u64> {
    Ok(u64::from_le_bytes(
        bytes
            .get(at..at + 8)
            .ok_or_else(bad)?
            .try_into()
            .map_err(|_| bad())?,
    ))
}
fn put32(out: &mut [u8], at: usize, n: u32) {
    out[at..at + 4].copy_from_slice(&n.to_le_bytes());
}
fn put64(out: &mut [u8], at: usize, n: u64) {
    out[at..at + 8].copy_from_slice(&n.to_le_bytes());
}
#[derive(Clone, Copy)]
pub(crate) struct Header {
    pub kind: u16,
    pub generation: u64,
    pub request: u64,
    pub body_len: usize,
}
impl Header {
    fn bytes(self) -> Result<[u8; 32]> {
        checked(self.body_len <= MAX_BODY)?;
        let mut out = [0; 32];
        out[..4].copy_from_slice(b"R3MD");
        out[4..6].copy_from_slice(&2u16.to_le_bytes());
        out[6..8].copy_from_slice(&self.kind.to_le_bytes());
        put64(&mut out, 8, self.generation);
        put64(&mut out, 16, self.request);
        put32(&mut out, 24, self.body_len as u32);
        Ok(out)
    }
    fn decode(bytes: &[u8; 32], output: bool, generation: u64, request: u64) -> Result<Self> {
        checked(
            &bytes[..4] == b"R3MD"
                && bytes[4..6] == 2u16.to_le_bytes()
                && u32_at(bytes, 28)? == 0
                && u64_at(bytes, 8)? == generation
                && u64_at(bytes, 16)? == request,
        )?;
        let kind = u16::from_le_bytes(bytes[6..8].try_into().map_err(|_| bad())?);
        let len = u32_at(bytes, 24)? as usize;
        let valid = if output {
            match kind {
                10 => len == 8,
                11 => len == 112,
                12 => (16 + 16..=16 + 2048 * 80).contains(&len) && len.is_multiple_of(4),
                13 => (9..=8 + CHUNK).contains(&len),
                14 => len == 88,
                15 => len == 16,
                _ => false,
            }
        } else {
            match kind {
                1 => len == 56,
                2 => (9..=8 + CHUNK).contains(&len),
                3 => len == 40,
                _ => false,
            }
        };
        checked(valid && len <= MAX_BODY)?;
        Ok(Self {
            kind,
            generation,
            request,
            body_len: len,
        })
    }
}
/// One frame buffer is paid by the fixed 3MiB transport reservation, including capacity overshoot.
pub(crate) struct FrameBuffer {
    data: Vec<u8>,
    _extra: Option<Lease>,
}
impl FrameBuffer {
    pub fn new(arena: &ActorArena) -> Result<Self> {
        let mut data = Vec::new();
        data.try_reserve_exact(MAX_BODY)
            .map_err(|_| Error::new(ErrorKind::Allocation))?;
        let extra = if data.capacity() > MAX_BODY {
            Some(arena.reserve(data.capacity() - MAX_BODY)?)
        } else {
            None
        };
        Ok(Self {
            data,
            _extra: extra,
        })
    }
    fn size(&mut self, n: usize) -> Result<&mut [u8]> {
        checked(n <= MAX_BODY)?;
        self.data.resize(n, 0);
        Ok(&mut self.data)
    }
    pub fn read<R: Read>(
        &mut self,
        input: &mut R,
        output: bool,
        generation: u64,
        request: u64,
    ) -> Result<Option<Header>> {
        let mut bytes = [0u8; 32];
        let n = loop {
            match input.read(&mut bytes[..1]) {
                Ok(n) => break n,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(io(e)),
            }
        };
        if n == 0 {
            return Ok(None);
        }
        input.read_exact(&mut bytes[1..]).map_err(io)?;
        let h = Header::decode(&bytes, output, generation, request)?;
        for chunk in self.size(h.body_len)?.chunks_mut(4096) {
            input.read_exact(chunk).map_err(io)?;
        }
        Ok(Some(h))
    }
    pub fn body(&self) -> &[u8] {
        &self.data
    }
}
fn emit<W: Write>(
    output: &mut W,
    kind: u16,
    generation: u64,
    request: u64,
    body: &[u8],
    hash: Option<&mut Sha256>,
    total: &mut u64,
) -> Result<()> {
    let h = Header {
        kind,
        generation,
        request,
        body_len: body.len(),
    };
    output.write_all(&h.bytes()?).map_err(io)?;
    output.write_all(body).map_err(io)?;
    if let Some(hash) = hash {
        hash.update(body);
    }
    *total = total.checked_add(32 + body.len() as u64).ok_or_else(bad)?;
    Ok(())
}
pub(crate) fn total_wire(counts: [u32; 11]) -> Result<u64> {
    let mut total = 304u64;
    for (&n, &stride) in counts[..10].iter().zip(STRIDES.iter()) {
        total = total
            .checked_add(n as u64 * stride as u64)
            .and_then(|n0| n0.checked_add((n as u64).div_ceil(2048) * 48))
            .ok_or_else(bad)?;
    }
    total = total
        .checked_add(counts[10] as u64)
        .and_then(|n| n.checked_add((counts[10] as u64).div_ceil(CHUNK as u64) * 40))
        .ok_or_else(bad)?;
    checked(total <= 134_217_728)?;
    Ok(total)
}
fn counts_from(body: &[u8], at: usize) -> Result<[u32; 11]> {
    let mut counts = [0; 11];
    for (i, n) in counts.iter_mut().enumerate() {
        *n = u32_at(body, at + 4 * i)?;
    }
    Ok(counts)
}
// Borrowed codec state avoids a second allocation or table staging copy.
#[allow(clippy::too_many_arguments)]
fn table<W: Write, const N: usize, T>(
    records: &[T],
    words: fn(&T) -> &[u32; N],
    kind: u32,
    buffer: &mut FrameBuffer,
    out: &mut W,
    generation: u64,
    request: u64,
    hash: &mut Sha256,
    total: &mut u64,
    ctl: &Control<'_>,
) -> Result<()> {
    for (batch, records) in records.chunks(2048).enumerate() {
        ctl.poll()?;
        let body = buffer.size(16 + records.len() * N * 4)?;
        put32(body, 0, kind);
        put32(body, 4, (batch * 2048) as u32);
        put32(body, 8, records.len() as u32);
        put32(body, 12, (N * 4) as u32);
        for (id, record) in records.iter().enumerate() {
            if id.is_multiple_of(64) {
                ctl.poll()?;
            }
            for (word, &value) in words(record).iter().enumerate() {
                put32(body, 16 + id * N * 4 + word * 4, value);
            }
        }
        emit(out, 12, generation, request, body, Some(hash), total)?;
    }
    Ok(())
}
pub(crate) fn send_boot<W: Write>(out: &mut W, generation: u64, request: u64) -> Result<()> {
    let mut body = [0; 8];
    put32(&mut body, 0, 3);
    put32(&mut body, 4, 2);
    emit(out, 10, generation, request, &body, None, &mut 0)
}
pub(crate) fn send_failure<W: Write>(
    out: &mut W,
    generation: u64,
    request: u64,
    e: Error,
) -> Result<()> {
    let code = match e.kind {
        ErrorKind::Parse => 1,
        ErrorKind::ArenaLimit => 2,
        ErrorKind::DepthLimit => 3,
        ErrorKind::RecordLimit => 4,
        ErrorKind::InvalidContext => 5,
        ErrorKind::Allocation => 6,
        _ => 7,
    };
    let mut body = [0; 16];
    for (i, n) in [code, e.table, e.id, e.source_offset]
        .into_iter()
        .enumerate()
    {
        put32(&mut body, i * 4, n);
    }
    emit(out, 15, generation, request, &body, None, &mut 0)?;
    out.flush().map_err(io)
}
pub(crate) fn send_model<W: Write>(
    c: &Candidate,
    out: &mut W,
    buffer: &mut FrameBuffer,
    ctl: &Control<'_>,
) -> Result<()> {
    let generation = c.source.generation();
    let request = c.source.request();
    let counts = c.counts();
    let expected = total_wire(counts)?;
    let mut total = 40u64;
    let mut hash = Sha256::new();
    let mut begin = [0; 112];
    put32(&mut begin, 0, c.profile as u32);
    put64(&mut begin, 8, c.source.len() as u64);
    begin[16..48].copy_from_slice(c.source.hash());
    for (i, &n) in counts.iter().enumerate() {
        put32(&mut begin, 48 + 4 * i, n);
    }
    put32(&mut begin, 92, c.max_depth);
    put64(&mut begin, 96, expected);
    emit(
        out,
        11,
        generation,
        request,
        &begin,
        Some(&mut hash),
        &mut total,
    )?;
    table(
        &c.blocks,
        Block::words,
        1,
        buffer,
        out,
        generation,
        request,
        &mut hash,
        &mut total,
        ctl,
    )?;
    table(
        &c.domains,
        Domain::words,
        2,
        buffer,
        out,
        generation,
        request,
        &mut hash,
        &mut total,
        ctl,
    )?;
    table(
        &c.items,
        Item::words,
        3,
        buffer,
        out,
        generation,
        request,
        &mut hash,
        &mut total,
        ctl,
    )?;
    table(
        &c.runs,
        Run::words,
        4,
        buffer,
        out,
        generation,
        request,
        &mut hash,
        &mut total,
        ctl,
    )?;
    table(
        &c.spans,
        Span::words,
        5,
        buffer,
        out,
        generation,
        request,
        &mut hash,
        &mut total,
        ctl,
    )?;
    table(
        &c.links,
        Link::words,
        6,
        buffer,
        out,
        generation,
        request,
        &mut hash,
        &mut total,
        ctl,
    )?;
    table(
        &c.resources,
        Resource::words,
        7,
        buffer,
        out,
        generation,
        request,
        &mut hash,
        &mut total,
        ctl,
    )?;
    table(
        &c.texts,
        TextRef::words,
        8,
        buffer,
        out,
        generation,
        request,
        &mut hash,
        &mut total,
        ctl,
    )?;
    table(
        &c.origins,
        Origin::words,
        9,
        buffer,
        out,
        generation,
        request,
        &mut hash,
        &mut total,
        ctl,
    )?;
    table(
        &c.attributes,
        Attribute::words,
        10,
        buffer,
        out,
        generation,
        request,
        &mut hash,
        &mut total,
        ctl,
    )?;
    for (i, chunk) in c.decoded.chunks(CHUNK).enumerate() {
        ctl.poll()?;
        let body = buffer.size(8 + chunk.len())?;
        put64(body, 0, (i * CHUNK) as u64);
        body[8..].copy_from_slice(chunk);
        emit(
            out,
            13,
            generation,
            request,
            body,
            Some(&mut hash),
            &mut total,
        )?;
    }
    let mut end = [0; 88];
    for (i, &n) in counts.iter().enumerate() {
        put32(&mut end, 4 * i, n);
    }
    put32(&mut end, 44, c.max_depth);
    put64(&mut end, 48, expected);
    end[56..].copy_from_slice(&hash.finalize());
    emit(out, 14, generation, request, &end, None, &mut total)?;
    checked(total == expected)?;
    out.flush().map_err(io)
}
fn append<const N: usize, T>(
    out: &mut ArenaVec<T>,
    body: &[u8],
    construct: fn([u32; N]) -> T,
    ctl: &Control<'_>,
) -> Result<()> {
    for record in body[16..].chunks_exact(N * 4) {
        ctl.poll()?;
        let mut words = [0; N];
        for (i, n) in words.iter_mut().enumerate() {
            *n = u32_at(record, 4 * i)?;
        }
        out.push(construct(words))?;
    }
    Ok(())
}
/// Returns quarantined validated data. The caller cannot publish before process/final-IO receipts.
pub(crate) fn receive_model<R: Read>(
    source: Arc<DocumentSource>,
    profile: Profile,
    arena: ActorArena,
    input: &mut R,
    ctl: &Control<'_>,
) -> Result<crate::document_validate::ValidatedCandidate> {
    let generation = source.generation();
    let request = source.request();
    let mut buffer = FrameBuffer::new(&arena)?;
    ctl.poll()?;
    let boot = buffer
        .read(input, true, generation, request)?
        .ok_or_else(bad)?;
    checked(boot.kind == 10 && u32_at(buffer.body(), 0)? == 3 && u32_at(buffer.body(), 4)? == 2)?;
    let begin = buffer
        .read(input, true, generation, request)?
        .ok_or_else(bad)?;
    if begin.kind == 15 {
        let body = buffer.body();
        let kind = match u32_at(body, 0)? {
            1 => ErrorKind::Parse,
            2 => ErrorKind::ArenaLimit,
            3 => ErrorKind::DepthLimit,
            4 => ErrorKind::RecordLimit,
            5 => ErrorKind::InvalidContext,
            6 => ErrorKind::Allocation,
            7 => ErrorKind::Internal,
            _ => return Err(bad()),
        };
        let table = u32_at(body, 4)?;
        let id = u32_at(body, 8)?;
        let offset = u32_at(body, 12)?;
        checked(
            table <= 10
                && (table != 0 || id == NONE)
                && (offset == NONE || source.text().is_char_boundary(offset as usize)),
        )?;
        let error = Error::new(kind).at(table, id, offset);
        checked(buffer.read(input, true, generation, request)?.is_none())?;
        return Err(error);
    }
    checked(begin.kind == 11)?;
    let body = buffer.body();
    checked(
        u32_at(body, 0)? == profile as u32
            && u32_at(body, 4)? == 0
            && u64_at(body, 8)? == source.len() as u64
            && body[16..48] == *source.hash()
            && u64_at(body, 104)? == 0,
    )?;
    let counts = counts_from(body, 48)?;
    let depth = u32_at(body, 92)?;
    checked(depth <= DEPTH_LIMIT)?;
    let expected = total_wire(counts)?;
    checked(u64_at(body, 96)? == expected)?;
    crate::document_validate::preflight(
        counts,
        arena
            .used()
            .saturating_sub(crate::document_budget::TRANSPORT_RESERVE),
    )?;
    let mut hash = Sha256::new();
    hash.update(body);
    let mut total = 184u64;
    let mut c = Candidate::empty(source, profile, arena);
    c.max_depth = depth;
    c.blocks.reserve_exact(counts[0] as usize)?;
    c.domains.reserve_exact(counts[1] as usize)?;
    c.items.reserve_exact(counts[2] as usize)?;
    c.runs.reserve_exact(counts[3] as usize)?;
    c.spans.reserve_exact(counts[4] as usize)?;
    c.links.reserve_exact(counts[5] as usize)?;
    c.resources.reserve_exact(counts[6] as usize)?;
    c.texts.reserve_exact(counts[7] as usize)?;
    c.origins.reserve_exact(counts[8] as usize)?;
    c.attributes.reserve_exact(counts[9] as usize)?;
    c.decoded.reserve_exact(counts[10] as usize)?;
    for table in 0..10 {
        let mut first = 0u32;
        while first < counts[table] {
            ctl.poll()?;
            let h = buffer
                .read(input, true, generation, request)?
                .ok_or_else(bad)?;
            let body = buffer.body();
            let count = (counts[table] - first).min(2048);
            checked(
                h.kind == 12
                    && u32_at(body, 0)? == table as u32 + 1
                    && u32_at(body, 4)? == first
                    && u32_at(body, 8)? == count
                    && u32_at(body, 12)? as usize == STRIDES[table]
                    && body.len() == 16 + count as usize * STRIDES[table],
            )?;
            hash.update(body);
            total = total.checked_add(32 + body.len() as u64).ok_or_else(bad)?;
            match table {
                0 => append(&mut c.blocks, body, Block::from_words, ctl)?,
                1 => append(&mut c.domains, body, Domain::from_words, ctl)?,
                2 => append(&mut c.items, body, Item::from_words, ctl)?,
                3 => append(&mut c.runs, body, Run::from_words, ctl)?,
                4 => append(&mut c.spans, body, Span::from_words, ctl)?,
                5 => append(&mut c.links, body, Link::from_words, ctl)?,
                6 => append(&mut c.resources, body, Resource::from_words, ctl)?,
                7 => append(&mut c.texts, body, TextRef::from_words, ctl)?,
                8 => append(&mut c.origins, body, Origin::from_words, ctl)?,
                9 => append(&mut c.attributes, body, Attribute::from_words, ctl)?,
                _ => return Err(bad()),
            }
            first += count;
        }
    }
    while c.decoded.len() < counts[10] as usize {
        ctl.poll()?;
        let h = buffer
            .read(input, true, generation, request)?
            .ok_or_else(bad)?;
        let body = buffer.body();
        let len = (counts[10] as usize - c.decoded.len()).min(CHUNK);
        checked(
            h.kind == 13 && body.len() == 8 + len && u64_at(body, 0)? == c.decoded.len() as u64,
        )?;
        hash.update(body);
        total = total.checked_add(32 + body.len() as u64).ok_or_else(bad)?;
        c.decoded.extend_from_slice(&body[8..])?;
    }
    let h = buffer
        .read(input, true, generation, request)?
        .ok_or_else(bad)?;
    let body = buffer.body();
    checked(
        h.kind == 14
            && counts_from(body, 0)? == counts
            && u32_at(body, 44)? == depth
            && u64_at(body, 48)? == expected
            && body[56..] == hash.finalize()[..],
    )?;
    total = total.checked_add(120).ok_or_else(bad)?;
    checked(total == expected && buffer.read(input, true, generation, request)?.is_none())?;
    ctl.poll()?;
    crate::document_validate::quarantine(c, depth, ctl)
}
pub(crate) fn send_source<W: Write>(
    source: &DocumentSource,
    profile: Profile,
    out: &mut W,
    ctl: &Control<'_>,
) -> Result<()> {
    let mut init = [0; 56];
    put32(&mut init, 0, profile as u32);
    put64(&mut init, 8, source.len() as u64);
    init[16..48].copy_from_slice(source.hash());
    let mut total = 0;
    emit(
        out,
        1,
        source.generation(),
        source.request(),
        &init,
        None,
        &mut total,
    )?;
    let mut buffer = [0; 8 + CHUNK];
    for (id, chunk) in source.text().as_bytes().chunks(CHUNK).enumerate() {
        ctl.poll()?;
        put64(&mut buffer, 0, (id * CHUNK) as u64);
        buffer[8..8 + chunk.len()].copy_from_slice(chunk);
        emit(
            out,
            2,
            source.generation(),
            source.request(),
            &buffer[..8 + chunk.len()],
            None,
            &mut total,
        )?;
    }
    let mut end = [0; 40];
    put64(&mut end, 0, source.len() as u64);
    end[8..].copy_from_slice(source.hash());
    emit(
        out,
        3,
        source.generation(),
        source.request(),
        &end,
        None,
        &mut total,
    )?;
    checked(
        total == source.len() as u64 + 40 * (source.len() as u64).div_ceil(CHUNK as u64) + 160,
    )?;
    out.flush().map_err(io)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::SourceOrigin;
    use std::{
        io::Cursor,
        sync::atomic::AtomicBool,
        time::{Duration, Instant},
    };
    #[test]
    fn predeclared_wire_header_and_zero_source_sizes() {
        let bytes = Header {
            kind: 10,
            generation: 0x0807060504030201,
            request: 0x1817161514131211,
            body_len: 8,
        }
        .bytes()
        .unwrap();
        assert_eq!(&bytes[..8], b"R3MD\x02\0\x0a\0");
        assert_eq!(&bytes[8..16], &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(&bytes[16..24], &[17, 18, 19, 20, 21, 22, 23, 24]);
        assert_eq!(&bytes[24..], &[8, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(total_wire([1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0]).unwrap(), 480);
        assert_eq!(134_217_728u64 + 40 * 2048 + 160, 134_299_808);
    }
    #[test]
    fn actual_table_adapter_traverses_wire_and_host_graph_validation() {
        let cancel = AtomicBool::new(false);
        let ctl = Control {
            cancel: &cancel,
            deadline: Instant::now() + Duration::from_secs(10),
        };
        let original = "| A |\n|---|\n| ![[dir\\Nota\\|100x145]] [[Nota\\|**Alias**]] |\n";
        let source = Arc::new(
            DocumentSource::new(
                Arc::new(original.to_owned()),
                SourceOrigin::Literal,
                19,
                23,
                &cancel,
            )
            .unwrap(),
        );
        let candidate = crate::document_parser::project(
            source.clone(),
            Profile::Extended,
            ActorArena::new(),
            &ctl,
        )
        .unwrap();
        let mut bytes = Vec::new();
        let mut buffer = FrameBuffer::new(&candidate.arena).unwrap();
        send_boot(&mut bytes, 19, 23).unwrap();
        send_model(&candidate, &mut bytes, &mut buffer, &ctl).unwrap();
        let received = receive_model(
            source.clone(),
            Profile::Extended,
            ActorArena::new(),
            &mut Cursor::new(&bytes),
            &ctl,
        )
        .unwrap();
        assert_eq!(received.counts(), candidate.counts());
        let link = &received.links[0];
        assert_eq!(received.text(link.target_decoded()).unwrap(), "Nota");
        let embed = &received.resources[0];
        assert_eq!(received.text(embed.input_decoded()).unwrap(), "dir\\Nota");
        assert_eq!((embed.width_value(), embed.height_value()), (100, 145));
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert_eq!(
            receive_model(
                source,
                Profile::Extended,
                ActorArena::new(),
                &mut Cursor::new(trailing),
                &ctl
            )
            .err()
            .unwrap()
            .kind,
            ErrorKind::InvalidIpc
        );
    }
    #[test]
    fn invalid_header_is_rejected_before_body_read() {
        struct HeaderOnly {
            bytes: Cursor<Vec<u8>>,
            body_read: bool,
        }
        impl Read for HeaderOnly {
            fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
                if self.bytes.position() >= 32 {
                    self.body_read = true;
                    panic!("body read after invalid preadmission");
                }
                self.bytes.read(out)
            }
        }
        let mut bytes = Header {
            kind: 12,
            generation: 1,
            request: 1,
            body_len: 32,
        }
        .bytes()
        .unwrap();
        put32(&mut bytes, 24, 1_048_577);
        let mut reader = HeaderOnly {
            bytes: Cursor::new(bytes.to_vec()),
            body_read: false,
        };
        let mut buffer = FrameBuffer::new(&ActorArena::new()).unwrap();
        assert_eq!(
            buffer.read(&mut reader, true, 1, 1).err().unwrap().kind,
            ErrorKind::InvalidIpc
        );
        assert!(!reader.body_read);
    }
}

pub(crate) fn receive_source<R: Read>(
    input: &mut R,
    generation: u64,
    request: u64,
    buffer: &mut FrameBuffer,
    ctl: &Control<'_>,
) -> Result<(Arc<DocumentSource>, Profile)> {
    let h = buffer
        .read(input, false, generation, request)?
        .ok_or_else(bad)?;
    checked(h.kind == 1)?;
    let body = buffer.body();
    let profile = Profile::from_word(u32_at(body, 0)?)?;
    checked(u32_at(body, 4)? == 0 && u64_at(body, 48)? == 0)?;
    let length = u64_at(body, 8)?;
    checked(length <= 134_217_728)?;
    let expected: [u8; 32] = body[16..48].try_into().map_err(|_| bad())?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(length as usize)
        .map_err(|_| Error::new(ErrorKind::Allocation))?;
    let mut hash = Sha256::new();
    while bytes.len() < length as usize {
        ctl.poll()?;
        let h = buffer
            .read(input, false, generation, request)?
            .ok_or_else(bad)?;
        let body = buffer.body();
        let n = (length as usize - bytes.len()).min(CHUNK);
        checked(h.kind == 2 && body.len() == 8 + n && u64_at(body, 0)? == bytes.len() as u64)?;
        hash.update(&body[8..]);
        bytes.extend_from_slice(&body[8..]);
    }
    let h = buffer
        .read(input, false, generation, request)?
        .ok_or_else(bad)?;
    let body = buffer.body();
    checked(
        h.kind == 3
            && u64_at(body, 0)? == length
            && body[8..] == expected
            && hash.finalize()[..] == expected,
    )?;
    checked(buffer.read(input, false, generation, request)?.is_none())?;
    crate::document_utf8::checked(&bytes, ctl)?;
    // Bounded UTF8 proof has completed for every byte; preserve the same allocation, never copy the source.
    let original = unsafe { String::from_utf8_unchecked(bytes) };
    let source = DocumentSource::new(
        Arc::new(original),
        crate::input::SourceOrigin::StandardInput,
        generation,
        request,
        ctl.cancel,
    )?;
    checked(*source.hash() == expected)?;
    Ok((Arc::new(source), profile))
}

#[cfg(test)]
mod hostile_tests {
    use super::*;
    use crate::input::SourceOrigin;
    use std::{
        io::Cursor,
        sync::atomic::AtomicBool,
        time::{Duration, Instant},
    };
    #[test]
    fn closed_failure_fields_reject_false_source_locations() {
        let cancel = AtomicBool::new(false);
        let ctl = Control {
            cancel: &cancel,
            deadline: Instant::now() + Duration::from_secs(10),
        };
        let source = Arc::new(
            DocumentSource::new(
                Arc::new("é".to_owned()),
                SourceOrigin::Literal,
                1,
                1,
                &cancel,
            )
            .unwrap(),
        );
        for (table, id, offset) in [(11, NONE, NONE), (0, 0, NONE), (0, NONE, 3), (0, NONE, 1)] {
            let mut bytes = Vec::new();
            send_boot(&mut bytes, 1, 1).unwrap();
            let mut body = [0; 16];
            for (i, value) in [1, table, id, offset].into_iter().enumerate() {
                put32(&mut body, 4 * i, value);
            }
            emit(&mut bytes, 15, 1, 1, &body, None, &mut 0).unwrap();
            assert_eq!(
                receive_model(
                    source.clone(),
                    Profile::Core,
                    ActorArena::new(),
                    &mut Cursor::new(bytes),
                    &ctl
                )
                .err()
                .unwrap()
                .kind,
                ErrorKind::InvalidIpc
            );
        }
    }
    #[test]
    fn binary_reader_asks_for_at_most_4096_and_accepts_fragmentation() {
        struct Bounded {
            bytes: Cursor<Vec<u8>>,
            calls: usize,
        }
        impl Read for Bounded {
            fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
                assert!(out.len() <= 4096);
                self.calls += 1;
                let n = out.len().min(3);
                self.bytes.read(&mut out[..n])
            }
        }
        let mut bytes = Vec::new();
        let body = vec![1u8; 8 + CHUNK];
        emit(&mut bytes, 2, 1, 1, &body, None, &mut 0).unwrap();
        let mut reader = Bounded {
            bytes: Cursor::new(bytes),
            calls: 0,
        };
        let mut buffer = FrameBuffer::new(&ActorArena::new()).unwrap();
        assert_eq!(
            buffer
                .read(&mut reader, false, 1, 1)
                .unwrap()
                .unwrap()
                .body_len,
            8 + CHUNK
        );
        assert_eq!(buffer.body(), body);
        assert!(reader.calls > 1000);
    }
    #[test]
    fn begin_minimum_arena_boundary_and_record_limit_are_exact() {
        let c = [1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 63_958_928];
        let (_, scratch) = crate::document_validate::preflight(c, 0).unwrap();
        assert_eq!(scratch, 4128);
        let mut plus = c;
        plus[10] += 1;
        assert_eq!(
            crate::document_validate::preflight(plus, 0)
                .unwrap_err()
                .kind,
            ErrorKind::ArenaLimit
        );
        let mut records = [0; 11];
        records[0] = 1;
        records[7] = 1_000_000;
        assert_eq!(
            crate::document_validate::preflight(records, 0)
                .unwrap_err()
                .kind,
            ErrorKind::RecordLimit
        );
    }
}
