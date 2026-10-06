//! Private binary protocol and lexical worker. No terminal or inferred file I/O.
use crate::document_consumer_budget::Charge;
#[cfg(test)]
use crate::document_consumer_budget::ConsumerBudget;
use crate::syntax::RetainedSyntaxInput;
use crate::syntax::{FallbackReason, RoleSpan, SyntaxRole, Viewport};
use std::io::{self, Read, Write};
use std::sync::Arc;

pub(crate) const MAX_SOURCE: usize = 128 * 1024 * 1024;
pub(crate) const MAX_LINE: usize = 65_536;
pub(crate) const MAX_FRAME: usize = 1024 * 1024;
pub(crate) const MAX_SPANS: usize = 50_000;
pub(crate) const MAX_VIEW_LINES: usize = 256;
pub(crate) const MAX_CHECKPOINTS: usize = 32;
#[cfg(windows)]
pub(crate) const MAX_OPS: usize = 262_144;
#[cfg(windows)]
pub(crate) const MAX_SCOPES: usize = 1024;
const HEADER: usize = 24;
const MAGIC: &[u8; 4] = b"R3SY";

#[derive(Debug)]
pub(crate) enum Message {
    Boot,
    Init {
        bytes: u32,
        language: String,
        spans: u32,
        checkpoints: u16,
    },
    Ack {
        offset: u32,
    },
    Chunk {
        offset: u32,
        bytes: Vec<u8>,
    },
    ChunkSlice {
        offset: u32,
        input: Arc<crate::syntax::RetainedSyntaxInput>,
        range: std::ops::Range<usize>,
    },
    SourceEnd,
    Ready,
    View(Viewport),
    All,
    Continue,
    Supersede(Viewport),
    Progress {
        line: u32,
        offset: u32,
    },
    Styles(Vec<RoleSpan>),
    Fallback(FallbackReason),
    Shutdown,
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
    io::Error::new(io::ErrorKind::InvalidData, "invalid syntax IPC")
}
struct Decoder<'a>(&'a [u8]);
impl<'a> Decoder<'a> {
    fn take(&mut self, size: usize) -> io::Result<&'a [u8]> {
        if size > self.0.len() {
            return Err(invalid());
        }
        let (a, b) = self.0.split_at(size);
        self.0 = b;
        Ok(a)
    }
    fn u8(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> io::Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> io::Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn viewport(&mut self) -> io::Result<Viewport> {
        let first_line = self.u32()?;
        let line_count = self.u16()?;
        if line_count == 0 || line_count as usize > MAX_VIEW_LINES {
            return Err(invalid());
        }
        Ok(Viewport {
            first_line,
            line_count,
        })
    }
}

fn write_bounded(out: &mut impl Write, bytes: &[u8]) -> io::Result<()> {
    for chunk in bytes.chunks(4096) {
        out.write_all(chunk)?;
    }
    Ok(())
}
pub(crate) fn write_frame(out: &mut impl Write, frame: &Frame) -> io::Result<()> {
    let (kind, len) = match &frame.message {
        Message::Boot => (0, 0),
        Message::Init {
            bytes,
            language,
            spans,
            checkpoints,
        } => {
            if *bytes as usize > MAX_SOURCE
                || language.is_empty()
                || language.len() > 256
                || *spans == 0
                || *spans as usize > MAX_SPANS
                || *checkpoints as usize > MAX_CHECKPOINTS
            {
                return Err(invalid());
            }
            (1, 12 + language.len())
        }
        Message::Ack { .. } => (2, 4),
        Message::Chunk { bytes, .. } => {
            if bytes.is_empty() || bytes.len() > MAX_LINE {
                return Err(invalid());
            }
            (3, 4 + bytes.len())
        }
        Message::ChunkSlice { input, range, .. } => {
            if range.start >= range.end || range.end > input.len() || range.len() > 4096 {
                return Err(invalid());
            }
            (3, 4 + range.len())
        }
        Message::SourceEnd => (4, 0),
        Message::Ready => (5, 0),
        Message::View(v) | Message::Supersede(v) => {
            if v.line_count == 0 || v.line_count as usize > MAX_VIEW_LINES {
                return Err(invalid());
            }
            (
                if matches!(frame.message, Message::View(_)) {
                    6
                } else {
                    9
                },
                6,
            )
        }
        Message::All => (7, 0),
        Message::Continue => (8, 0),
        Message::Progress { .. } => (10, 8),
        Message::Styles(v) => {
            if v.len() > MAX_SPANS {
                return Err(invalid());
            }
            (11, 4 + 9 * v.len())
        }
        Message::Fallback(_) => (12, 1),
        Message::Shutdown => (13, 0),
    };
    if HEADER + len > MAX_FRAME {
        return Err(invalid());
    }
    let mut header = [0u8; 28];
    header[..4].copy_from_slice(&((HEADER + len) as u32).to_le_bytes());
    header[4..8].copy_from_slice(MAGIC);
    header[8..10].copy_from_slice(&1u16.to_le_bytes());
    header[10] = kind;
    header[12..20].copy_from_slice(&frame.generation.to_le_bytes());
    header[20..28].copy_from_slice(&frame.request.to_le_bytes());
    write_bounded(out, &header)?;
    match &frame.message {
        Message::Init {
            bytes,
            language,
            spans,
            checkpoints,
        } => {
            let mut meta = [0u8; 12];
            meta[..4].copy_from_slice(&bytes.to_le_bytes());
            meta[4..8].copy_from_slice(&spans.to_le_bytes());
            meta[8..10].copy_from_slice(&checkpoints.to_le_bytes());
            meta[10..12].copy_from_slice(&(language.len() as u16).to_le_bytes());
            write_bounded(out, &meta)?;
            write_bounded(out, language.as_bytes())?;
        }
        Message::Ack { offset } => write_bounded(out, &offset.to_le_bytes())?,
        Message::Chunk { offset, bytes } => {
            write_bounded(out, &offset.to_le_bytes())?;
            write_bounded(out, bytes)?;
        }
        Message::ChunkSlice {
            offset,
            input,
            range,
        } => {
            write_bounded(out, &offset.to_le_bytes())?;
            write_bounded(out, &input.text().as_bytes()[range.clone()])?;
        }
        Message::View(v) | Message::Supersede(v) => {
            write_bounded(out, &v.first_line.to_le_bytes())?;
            write_bounded(out, &v.line_count.to_le_bytes())?;
        }
        Message::Progress { line, offset } => {
            write_bounded(out, &line.to_le_bytes())?;
            write_bounded(out, &offset.to_le_bytes())?;
        }
        Message::Styles(spans) => {
            write_bounded(out, &(spans.len() as u32).to_le_bytes())?;
            for span in spans {
                let mut row = [0u8; 9];
                row[..4].copy_from_slice(&span.start.to_le_bytes());
                row[4..8].copy_from_slice(&span.end.to_le_bytes());
                row[8] = span.role as u8;
                write_bounded(out, &row)?;
            }
        }
        Message::Fallback(reason) => write_bounded(out, &[*reason as u8])?,
        _ => {}
    }
    out.flush()
}
fn read_bounded(input: &mut impl Read, bytes: &mut [u8]) -> io::Result<()> {
    for chunk in bytes.chunks_mut(4096) {
        input.read_exact(chunk)?;
    }
    Ok(())
}
struct RoleCharge {
    _base: Charge,
    _extra: Option<Charge>,
}
/// Decoder-only ownership: allocation and its admission cannot be exchanged.
pub(crate) struct OwnedRoleAllocation {
    spans: Vec<RoleSpan>,
    input: Arc<RetainedSyntaxInput>,
    charge: Option<RoleCharge>,
    capacity_bytes: usize,
    generation: u64,
    request: u64,
}
impl OwnedRoleAllocation {
    pub(crate) fn spans(&self) -> &[RoleSpan] {
        &self.spans
    }
    pub(crate) fn input(&self) -> &RetainedSyntaxInput {
        &self.input
    }
    pub(crate) fn capacity_bytes(&self) -> usize {
        self.capacity_bytes
    }
    pub(crate) fn verify_owner(
        &self,
        expected: &Arc<RetainedSyntaxInput>,
        generation: u64,
        request: u64,
    ) -> io::Result<()> {
        let actual = self
            .spans
            .capacity()
            .checked_mul(std::mem::size_of::<RoleSpan>())
            .ok_or_else(invalid)?;
        if !Arc::ptr_eq(&self.input, expected)
            || generation != self.generation
            || request != self.request
            || actual != self.capacity_bytes
            || (actual != 0 && self.input.budget().is_some() && self.charge.is_none())
        {
            return Err(invalid());
        }
        Ok(())
    }
    /// Source compatibility only; never a Document admission escape.
    #[cfg(test)]
    pub(crate) fn legacy(
        spans: Vec<RoleSpan>,
        input: Arc<RetainedSyntaxInput>,
    ) -> io::Result<Self> {
        if input.budget().is_some() {
            return Err(invalid());
        }
        let capacity_bytes = spans
            .capacity()
            .checked_mul(std::mem::size_of::<RoleSpan>())
            .ok_or_else(invalid)?;
        Ok(Self {
            spans,
            input,
            charge: None,
            capacity_bytes,
            generation: 0,
            request: 0,
        })
    }
}
pub(crate) struct FrameReceipt {
    pub frame: Frame,
    roles: Option<OwnedRoleAllocation>,
}
impl FrameReceipt {
    pub(crate) fn take_roles(&mut self) -> Option<OwnedRoleAllocation> {
        self.roles.take()
    }
}
pub(crate) fn read_frame(input: &mut impl Read) -> io::Result<Frame> {
    Ok(read_frame_limited(input, None, MAX_SPANS)?.frame)
}
pub(crate) fn read_host_frame(
    input: &mut impl Read,
    owner: &Arc<RetainedSyntaxInput>,
    span_limit: usize,
) -> io::Result<FrameReceipt> {
    read_frame_limited(input, Some(owner), span_limit)
}
fn read_frame_limited(
    input: &mut impl Read,
    owner: Option<&Arc<RetainedSyntaxInput>>,
    span_limit: usize,
) -> io::Result<FrameReceipt> {
    let host = owner.is_some();
    let budget = owner.and_then(|v| v.budget());
    let mut prefix = [0u8; 4];
    read_bounded(input, &mut prefix)?;
    let length = u32::from_le_bytes(prefix) as usize;
    if !(HEADER..=MAX_FRAME).contains(&length) {
        return Err(invalid());
    }
    let mut header = [0u8; HEADER];
    read_bounded(input, &mut header)?;
    let mut h = Decoder(&header);
    if h.take(4)? != MAGIC || h.u16()? != 1 {
        return Err(invalid());
    }
    let kind = h.u8()?;
    if h.u8()? != 0 {
        return Err(invalid());
    }
    let generation = h.u64()?;
    let request = h.u64()?;
    if host && !matches!(kind, 0 | 2 | 5 | 10 | 11 | 12) {
        return Err(invalid());
    }
    let payload = length - HEADER;
    let legal = match kind {
        0 | 4 | 5 | 7 | 8 | 13 => payload == 0,
        1 => (13..=268).contains(&payload),
        2 => payload == 4,
        3 => (5..=4 + MAX_LINE).contains(&payload),
        6 | 9 => payload == 6,
        10 => payload == 8,
        11 => payload >= 4 && payload <= 4 + 9 * span_limit.min(MAX_SPANS),
        12 => payload == 1,
        _ => false,
    };
    if !legal {
        return Err(invalid());
    }
    let mut count_prefix = [0u8; 4];
    let mut role_charge = None;
    let mut role_requested = 0;
    if kind == 11 {
        read_bounded(input, &mut count_prefix)?;
        let count = u32::from_le_bytes(count_prefix) as usize;
        if count > span_limit.min(MAX_SPANS)
            || count.checked_mul(9).and_then(|n| n.checked_add(4)) != Some(payload)
        {
            return Err(invalid());
        }
        role_requested = count
            .checked_mul(std::mem::size_of::<RoleSpan>())
            .ok_or_else(invalid)?;
        role_charge = budget
            .map(|b| {
                b.reserve(role_requested)
                    .map(|charge| RoleCharge {
                        _base: charge,
                        _extra: None,
                    })
                    .map_err(|e| io::Error::new(io::ErrorKind::OutOfMemory, e))
            })
            .transpose()?;
    }
    let _body_charge = budget
        .map(|b| {
            b.reserve(payload)
                .map_err(|e| io::Error::new(io::ErrorKind::OutOfMemory, e))
        })
        .transpose()?;
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(payload).map_err(io::Error::other)?;
    let _body_extra = budget
        .map(|b| {
            b.reserve(bytes.capacity().saturating_sub(payload))
                .map_err(|e| io::Error::new(io::ErrorKind::OutOfMemory, e))
        })
        .transpose()?;
    bytes.resize(payload, 0);
    if kind == 11 {
        bytes[..4].copy_from_slice(&count_prefix);
        read_bounded(input, &mut bytes[4..])?;
    } else {
        read_bounded(input, &mut bytes)?;
    }
    let mut d = Decoder(&bytes);
    let mut owned_roles = None;
    let message = match kind {
        0 => Message::Boot,
        1 => {
            let bytes = d.u32()?;
            let spans = d.u32()?;
            let checkpoints = d.u16()?;
            let count = d.u16()? as usize;
            if bytes as usize > MAX_SOURCE
                || spans == 0
                || spans as usize > MAX_SPANS
                || checkpoints as usize > MAX_CHECKPOINTS
                || count == 0
                || count > 256
            {
                return Err(invalid());
            }
            let language = std::str::from_utf8(d.take(count)?)
                .map_err(|_| invalid())?
                .to_owned();
            Message::Init {
                bytes,
                language,
                spans,
                checkpoints,
            }
        }
        2 => Message::Ack { offset: d.u32()? },
        3 => {
            let offset = d.u32()?;
            if d.0.is_empty() || d.0.len() > MAX_LINE {
                return Err(invalid());
            }
            let bytes = d.take(d.0.len())?.to_vec();
            Message::Chunk { offset, bytes }
        }
        4 => Message::SourceEnd,
        5 => Message::Ready,
        6 => Message::View(d.viewport()?),
        7 => Message::All,
        8 => Message::Continue,
        9 => Message::Supersede(d.viewport()?),
        10 => Message::Progress {
            line: d.u32()?,
            offset: d.u32()?,
        },
        11 => {
            let count = d.u32()? as usize;
            if count > MAX_SPANS || count.checked_mul(9) != Some(d.0.len()) {
                return Err(invalid());
            }
            let mut spans = Vec::new();
            spans.try_reserve_exact(count).map_err(io::Error::other)?;
            let actual = spans
                .capacity()
                .checked_mul(std::mem::size_of::<RoleSpan>())
                .ok_or_else(invalid)?;
            if let Some(b) = budget {
                let extra = b
                    .reserve(actual.saturating_sub(role_requested))
                    .map_err(|e| io::Error::new(io::ErrorKind::OutOfMemory, e))?;
                if let Some(charge) = role_charge.as_mut() {
                    charge._extra = Some(extra);
                }
            }
            for _ in 0..count {
                let start = d.u32()?;
                let end = d.u32()?;
                let role = SyntaxRole::from_wire(d.u8()?).ok_or_else(invalid)?;
                if start >= end || end as usize > MAX_SOURCE {
                    return Err(invalid());
                }
                spans.push(RoleSpan { start, end, role });
            }
            if let Some(owner) = owner {
                owned_roles = Some(OwnedRoleAllocation {
                    spans,
                    input: owner.clone(),
                    charge: role_charge,
                    capacity_bytes: actual,
                    generation,
                    request,
                });
                Message::Styles(Vec::new()) // marker only; owned payload is private receipt above.
            } else {
                Message::Styles(spans)
            }
        }
        12 => Message::Fallback(FallbackReason::from_wire(d.u8()?).ok_or_else(invalid)?),
        13 => Message::Shutdown,
        _ => return Err(invalid()),
    };
    if !d.0.is_empty() {
        return Err(invalid());
    }
    Ok(FrameReceipt {
        frame: Frame {
            generation,
            request,
            message,
        },
        roles: owned_roles,
    })
}

/// Called only for the executable's private --syntax-worker entry point.
pub fn run_private_stdio() -> io::Result<()> {
    #[cfg(windows)]
    {
        run_worker()
    }
    #[cfg(not(windows))]
    {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "syntax containment is not validated on this platform",
        ))
    }
}
#[cfg(windows)]
pub fn bundled_syntaxes() -> io::Result<syntect::parsing::SyntaxSet> {
    syntect::dumps::from_reader(std::io::Cursor::new(include_bytes!(concat!(
        env!("OUT_DIR"),
        "/bundled.syntax.pack"
    ))))
    .map_err(|_| invalid())
}

#[cfg(windows)]
fn run_worker() -> io::Result<()> {
    use std::sync::mpsc;
    let (send, receive) = mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("syntax-lease".into())
        .spawn(move || {
            let input = io::stdin();
            let mut input = input.lock();
            loop {
                let frame = match read_frame(&mut input) {
                    Ok(frame) => frame,
                    Err(_) => std::process::exit(75),
                };
                // Never park the EOF watcher behind a full request queue.
                if send.try_send(frame).is_err() {
                    std::process::exit(76);
                }
            }
        })?;
    let output = io::stdout();
    let mut output = output.lock();
    write_frame(&mut output, &Frame::new(0, 0, Message::Boot))?;
    let init = receive.recv().map_err(io::Error::other)?;
    let generation = init.generation;
    let (size, language, span_limit, checkpoint_limit) = match init.message {
        Message::Init {
            bytes,
            language,
            spans,
            checkpoints,
        } if init.request == 0 => (
            bytes as usize,
            language,
            spans as usize,
            checkpoints as usize,
        ),
        _ => return Err(invalid()),
    };
    let syntaxes = bundled_syntaxes()?;
    let extension = language.trim_start_matches('.').to_ascii_lowercase();
    let syntax = syntaxes
        .find_syntax_by_extension(&language)
        .or_else(|| syntaxes.find_syntax_by_extension(&extension))
        .or_else(|| {
            syntaxes
                .syntaxes()
                .iter()
                .find(|s| s.name.eq_ignore_ascii_case(&language))
        });
    let Some(syntax) = syntax else {
        return fail(&mut output, generation, 0, FallbackReason::UnknownLanguage);
    };
    let parser = syntect::parsing::ParseState::new(syntax);
    let mut source = Vec::new();
    if source.try_reserve_exact(size).is_err() {
        return fail(&mut output, generation, 0, FallbackReason::SourceBudget);
    }
    write_frame(
        &mut output,
        &Frame::new(generation, 0, Message::Ack { offset: 0 }),
    )?;
    let mut line_bytes = 0usize;
    let mut lines = 1usize;
    loop {
        let frame = receive.recv().map_err(io::Error::other)?;
        if frame.generation != generation || frame.request != 0 {
            return Err(invalid());
        }
        match frame.message {
            Message::Chunk { offset, bytes } => {
                if offset as usize != source.len()
                    || bytes.len() > size.saturating_sub(source.len())
                {
                    return Err(invalid());
                }
                for byte in &bytes {
                    line_bytes += 1;
                    if line_bytes > MAX_LINE {
                        return fail(&mut output, generation, 0, FallbackReason::LongLine);
                    }
                    if *byte == b'\n' {
                        line_bytes = 0;
                        lines += 1;
                        if lines > 1_000_000 {
                            return fail(&mut output, generation, 0, FallbackReason::SourceBudget);
                        }
                    }
                }
                source.extend(bytes);
                write_frame(
                    &mut output,
                    &Frame::new(
                        generation,
                        0,
                        Message::Ack {
                            offset: source.len() as u32,
                        },
                    ),
                )?;
            }
            Message::SourceEnd if source.len() == size => break,
            _ => return Err(invalid()),
        }
    }
    let source = String::from_utf8(source).map_err(|_| invalid())?;
    let mut engine = Engine::new(source, syntaxes, parser, span_limit, checkpoint_limit)?;
    write_frame(&mut output, &Frame::new(generation, 0, Message::Ready))?;
    loop {
        let request = receive.recv().map_err(io::Error::other)?;
        if request.generation != generation {
            return Err(invalid());
        }
        if matches!(request.message, Message::Shutdown) {
            return Ok(());
        }
        let mut request_id = request.request;
        let mut target = match request.message {
            Message::View(view) => Some(view),
            Message::All => None,
            _ => return Err(invalid()),
        };
        engine.seek(target.map_or(0, |v| v.first_line));
        let mut spans = Vec::new();
        loop {
            let end = target.map(|v| v.first_line as u64 + v.line_count as u64);
            if engine.offset >= engine.source.len()
                || end.is_some_and(|end| engine.line as u64 >= end)
            {
                write_frame(
                    &mut output,
                    &Frame::new(generation, request_id, Message::Styles(spans)),
                )?;
                break;
            }
            let emit = target.is_none_or(|v| engine.line >= v.first_line);
            if let Err(reason) = engine.step(emit, &mut spans) {
                return fail(&mut output, generation, request_id, reason);
            }
            write_frame(
                &mut output,
                &Frame::new(
                    generation,
                    request_id,
                    Message::Progress {
                        line: engine.line,
                        offset: engine.offset as u32,
                    },
                ),
            )?;
            let control = receive.recv().map_err(io::Error::other)?;
            if control.generation != generation {
                return Err(invalid());
            }
            match control.message {
                Message::Continue if control.request == request_id => {}
                Message::Supersede(view) => {
                    request_id = control.request;
                    target = Some(view);
                    engine.seek(view.first_line);
                    spans.clear();
                }
                Message::Shutdown => return Ok(()),
                _ => return Err(invalid()),
            }
        }
    }
}
#[cfg(windows)]
fn fail(
    out: &mut impl Write,
    generation: u64,
    request: u64,
    reason: FallbackReason,
) -> io::Result<()> {
    write_frame(
        out,
        &Frame::new(generation, request, Message::Fallback(reason)),
    )?;
    Err(io::Error::other("syntax worker fallback"))
}

#[cfg(windows)]
struct Checkpoint {
    line: u32,
    offset: usize,
    parser: syntect::parsing::ParseState,
    stack: syntect::parsing::ScopeStack,
}
#[cfg(windows)]
struct Engine {
    source: String,
    syntaxes: syntect::parsing::SyntaxSet,
    parser: syntect::parsing::ParseState,
    stack: syntect::parsing::ScopeStack,
    initial: Checkpoint,
    checkpoints: std::collections::VecDeque<Checkpoint>,
    line: u32,
    offset: usize,
    span_limit: usize,
    checkpoint_limit: usize,
    roles: Vec<(syntect::parsing::Scope, SyntaxRole)>,
    #[cfg(test)]
    scope_trace: Option<Vec<(usize, usize, Vec<String>)>>,
}
#[cfg(windows)]
impl Engine {
    fn new(
        source: String,
        syntaxes: syntect::parsing::SyntaxSet,
        parser: syntect::parsing::ParseState,
        span_limit: usize,
        checkpoint_limit: usize,
    ) -> io::Result<Self> {
        use syntect::parsing::{Scope, ScopeStack};
        let stack = ScopeStack::new();
        let initial = Checkpoint {
            line: 0,
            offset: 0,
            parser: parser.clone(),
            stack: stack.clone(),
        };
        let roles = [
            ("comment", SyntaxRole::Secondary),
            ("string", SyntaxRole::Success),
            ("keyword", SyntaxRole::Accent),
            ("storage", SyntaxRole::Accent),
            ("constant.numeric", SyntaxRole::Process),
            ("entity.name", SyntaxRole::Heading),
            ("support", SyntaxRole::Heading),
            ("type", SyntaxRole::Heading),
        ]
        .into_iter()
        .map(|(scope, role)| Scope::new(scope).map(|scope| (scope, role)))
        .collect::<Result<Vec<_>, _>>()
        .map_err(io::Error::other)?;
        Ok(Self {
            source,
            syntaxes,
            parser,
            stack,
            initial,
            checkpoints: Default::default(),
            line: 0,
            offset: 0,
            span_limit,
            checkpoint_limit,
            roles,
            #[cfg(test)]
            scope_trace: None,
        })
    }
    fn seek(&mut self, first: u32) {
        let best = self
            .checkpoints
            .iter()
            .enumerate()
            .filter(|(_, c)| c.line <= first)
            .max_by_key(|(_, c)| c.line)
            .map(|(i, _)| i);
        if let Some(i) = best {
            let c = self.checkpoints.remove(i).unwrap();
            self.line = c.line;
            self.offset = c.offset;
            self.parser = c.parser.clone();
            self.stack = c.stack.clone();
            self.checkpoints.push_back(c);
        } else {
            self.line = 0;
            self.offset = 0;
            self.parser = self.initial.parser.clone();
            self.stack = self.initial.stack.clone();
        }
    }
    fn role(&self) -> SyntaxRole {
        let comment = self.roles[0].0;
        if self
            .stack
            .as_slice()
            .iter()
            .any(|scope| comment.is_prefix_of(*scope))
        {
            return SyntaxRole::Secondary;
        }
        for scope in self.stack.as_slice().iter().rev() {
            for (prefix, role) in &self.roles[1..] {
                if prefix.is_prefix_of(*scope) {
                    return *role;
                }
            }
        }
        SyntaxRole::Value
    }
    fn step(&mut self, emit: bool, spans: &mut Vec<RoleSpan>) -> Result<(), FallbackReason> {
        let length = self.source[self.offset..]
            .find('\n')
            .map_or(self.source.len() - self.offset, |at| at + 1);
        if length > MAX_LINE {
            return Err(FallbackReason::LongLine);
        }
        let text = &self.source[self.offset..self.offset + length];
        let changes = self
            .parser
            .parse_line(text, &self.syntaxes)
            .map_err(|_| FallbackReason::GrammarError)?;
        if changes.len() > MAX_OPS
            || changes
                .iter()
                .any(|(at, _)| *at > text.len() || !text.is_char_boundary(*at))
            || changes.windows(2).any(|pair| pair[0].0 > pair[1].0)
        {
            return Err(FallbackReason::OperationBudget);
        }
        // RangeIterator supplies the op applicable BEFORE each region, including empty ones.
        for (range, op) in syntect::easy::ScopeRangeIterator::new(&changes, text) {
            self.stack
                .apply(op)
                .map_err(|_| FallbackReason::GrammarError)?;
            if self.stack.len() > MAX_SCOPES {
                return Err(FallbackReason::OperationBudget);
            }
            #[cfg(test)]
            if !range.is_empty()
                && let Some(trace) = &mut self.scope_trace
            {
                trace.push((
                    self.offset + range.start,
                    self.offset + range.end,
                    self.stack
                        .as_slice()
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                ));
            }
            if emit && !range.is_empty() {
                let role = self.role();
                let start = (self.offset + range.start) as u32;
                let end = (self.offset + range.end) as u32;
                if let Some(last) = spans.last_mut()
                    && last.end == start
                    && last.role == role
                {
                    last.end = end;
                } else {
                    if spans.len() >= self.span_limit {
                        return Err(FallbackReason::SpanBudget);
                    }
                    spans.push(RoleSpan { start, end, role });
                }
            }
        }
        self.line += 1;
        self.offset += length;
        if self.checkpoint_limit > 0
            && self.line.is_multiple_of(256)
            && !self.checkpoints.iter().any(|c| c.line == self.line)
        {
            if self.checkpoints.len() >= self.checkpoint_limit {
                self.checkpoints.pop_front();
            }
            self.checkpoints.push_back(Checkpoint {
                line: self.line,
                offset: self.offset,
                parser: self.parser.clone(),
                stack: self.stack.clone(),
            });
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frame_rejects_lengths_and_counts_before_reserving() {
        for length in [0, 23, MAX_FRAME as u32 + 1, u32::MAX] {
            assert_eq!(
                read_frame(&mut &length.to_le_bytes()[..])
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::InvalidData
            );
        }
        let frame = Frame::new(
            7,
            8,
            Message::Styles(vec![RoleSpan {
                start: 0,
                end: 2,
                role: SyntaxRole::Success,
            }]),
        );
        let mut bytes = Vec::new();
        write_frame(&mut bytes, &frame).unwrap();
        bytes[28..32].copy_from_slice(&(MAX_SPANS as u32 + 1).to_le_bytes());
        assert!(read_frame(&mut &bytes[..]).is_err());
    }
    #[test]
    fn binary_protocol_roundtrips_and_rejects_role_and_truncation() {
        let frame = Frame::new(
            3,
            4,
            Message::Styles(vec![RoleSpan {
                start: 4,
                end: 9,
                role: SyntaxRole::Accent,
            }]),
        );
        let mut bytes = Vec::new();
        write_frame(&mut bytes, &frame).unwrap();
        let value = read_frame(&mut &bytes[..]).unwrap();
        assert_eq!(value.generation, 3);
        assert_eq!(value.request, 4);
        assert!(
            matches!(value.message,Message::Styles(spans) if spans==vec![RoleSpan{start:4,end:9,role:SyntaxRole::Accent}])
        );
        assert!(read_frame(&mut &bytes[..bytes.len() - 1]).is_err());
        *bytes.last_mut().unwrap() = 255;
        assert!(read_frame(&mut &bytes[..]).is_err());
    }
    #[cfg(windows)]
    fn engine(source: String, limit: usize, checkpoints: usize) -> Engine {
        let syntax = syntect::parsing::SyntaxSet::load_defaults_newlines();
        let parser =
            syntect::parsing::ParseState::new(syntax.find_syntax_by_extension("rs").unwrap());
        Engine::new(source, syntax, parser, limit, checkpoints).unwrap()
    }
    #[cfg(windows)]
    #[test]
    fn roles_cover_unicode_and_follow_multiline_context() {
        let source =
            "/*\r\nlet café = 42; 界 👩‍💻 e\u{301}\r\n*/ let n = 42; let s = \"é界\";\n".to_string();
        let mut e = engine(source.clone(), MAX_SPANS, 32);
        let mut spans = Vec::new();
        while e.offset < e.source.len() {
            e.step(true, &mut spans).unwrap();
        }
        assert_eq!(spans.first().unwrap().start, 0);
        assert_eq!(spans.last().unwrap().end as usize, source.len());
        assert!(spans.windows(2).all(|s| s[0].end == s[1].start));
        assert!(
            spans
                .iter()
                .all(|s| source.is_char_boundary(s.start as usize)
                    && source.is_char_boundary(s.end as usize))
        );
        let comment = source.find("café").unwrap();
        assert!(spans.iter().any(|s| s.role == SyntaxRole::Secondary
            && s.start as usize <= comment
            && s.end as usize > comment));
        assert!(spans.iter().any(|s| s.role == SyntaxRole::Accent));
        assert!(spans.iter().any(|s| s.role == SyntaxRole::Process));
        assert!(spans.iter().any(|s| s.role == SyntaxRole::Success));
    }
    #[cfg(windows)]
    #[test]
    fn checkpoint_eviction_reparses_context_and_preserves_clear_restore() {
        use syntect::parsing::{ClearAmount, Scope, ScopeStackOp};
        let source = format!("/*\n{}*/ let x = 42;\n", "inside é界\n".repeat(9000));
        let mut e = engine(source, MAX_SPANS, 32);
        let mut discarded = Vec::new();
        while e.offset < e.source.len() {
            e.step(false, &mut discarded).unwrap();
        }
        assert_eq!(e.checkpoints.len(), 32);
        assert!(e.checkpoints.iter().all(|c| c.line > 256));
        e.seek(300);
        let mut spans = Vec::new();
        while e.line <= 300 {
            e.step(e.line == 300, &mut spans).unwrap();
        }
        assert!(spans.iter().all(|s| s.role == SyntaxRole::Secondary));
        let comment = Scope::new("comment.block.rust").unwrap();
        e.stack.push(comment);
        e.stack
            .apply(&ScopeStackOp::Clear(ClearAmount::All))
            .unwrap();
        e.checkpoints.clear();
        e.checkpoints.push_back(Checkpoint {
            line: 256,
            offset: 0,
            parser: e.parser.clone(),
            stack: e.stack.clone(),
        });
        e.seek(256);
        e.stack.apply(&ScopeStackOp::Restore).unwrap();
        assert!(e.stack.as_slice().contains(&comment));
    }
    #[cfg(windows)]
    #[test]
    fn line_and_span_limits_fail_whole_result() {
        let mut e = engine(
            format!("let x = 1;{}\n", " ".repeat(MAX_LINE)),
            MAX_SPANS,
            32,
        );
        assert_eq!(e.step(true, &mut Vec::new()), Err(FallbackReason::LongLine));
        let mut e = engine("let x = 1; let s = \"hello\";\n".into(), 1, 32);
        assert_eq!(
            e.step(true, &mut Vec::new()),
            Err(FallbackReason::SpanBudget)
        );
        let mut e = engine(format!("//{}\n", "x".repeat(MAX_LINE - 3)), MAX_SPANS, 32);
        e.step(true, &mut Vec::new()).unwrap();
        assert_eq!(e.offset, MAX_LINE);
    }
    #[cfg(windows)]
    struct ShellAssertion {
        line: usize,
        start: usize,
        end: usize,
        scopes: &'static [&'static str],
    }
    #[cfg(windows)]
    struct ShellEndStack {
        line: usize,
        scopes: &'static [&'static str],
    }
    #[cfg(windows)]
    struct ShellFixture {
        id: &'static str,
        language: &'static str,
        source: &'static str,
        assertions: &'static [ShellAssertion],
        end_stacks: &'static [ShellEndStack],
    }
    #[cfg(windows)]
    include!("../resources/syntax/shell/oracle-fixtures.rs");

    #[cfg(windows)]
    #[test]
    fn shell_oracles_use_real_engine_limits_scopes_and_seek() {
        let mut assertions = 0;
        let mut end_stacks = 0;
        for fixture in SHELL_ORACLES {
            let set = bundled_syntaxes().unwrap();
            let parser = syntect::parsing::ParseState::new(
                set.find_syntax_by_name(fixture.language).unwrap(),
            );
            let mut engine = Engine::new(
                fixture.source.into(),
                set,
                parser,
                MAX_SPANS,
                MAX_CHECKPOINTS,
            )
            .unwrap();
            engine.scope_trace = Some(Vec::new());
            let mut roles = Vec::new();
            let mut offsets = Vec::new();
            while engine.offset < engine.source.len() {
                offsets.push(engine.offset);
                engine
                    .step(true, &mut roles)
                    .unwrap_or_else(|error| panic!("{}: {error:?}", fixture.id));
                for expected in fixture
                    .end_stacks
                    .iter()
                    .filter(|e| e.line + 1 == engine.line as usize)
                {
                    assert_eq!(
                        engine
                            .stack
                            .as_slice()
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>(),
                        expected.scopes,
                        "{} endStack line {}",
                        fixture.id,
                        expected.line
                    );
                    end_stacks += 1;
                }
            }
            let trace = engine.scope_trace.take().unwrap();
            let mut offset = 0;
            for (start, end, _) in &trace {
                assert_eq!(*start, offset, "{} trace continuity", fixture.id);
                assert!(
                    *end > *start
                        && fixture.source.is_char_boundary(*start)
                        && fixture.source.is_char_boundary(*end)
                );
                offset = *end;
            }
            assert_eq!(offset, fixture.source.len());
            assert_eq!(roles.first().unwrap().start, 0);
            assert_eq!(roles.last().unwrap().end as usize, fixture.source.len());
            assert!(roles.windows(2).all(|pair| pair[0].end == pair[1].start));
            for expected in fixture.assertions {
                let start = offsets[expected.line] + expected.start;
                let end = offsets[expected.line] + expected.end;
                let mut position = start;
                for (a, b, scopes) in trace.iter().filter(|(a, b, _)| *b > start && *a < end) {
                    assert_eq!(
                        (*a).max(start),
                        position,
                        "{} assertion coverage",
                        fixture.id
                    );
                    assert_eq!(
                        scopes, expected.scopes,
                        "{} interval {start}..{end}",
                        fixture.id
                    );
                    position = (*b).min(end);
                }
                assert_eq!(position, end);
                assertions += 1;
            }
            // The same Engine seek/step path must reconstruct spans and stacks.
            engine.seek(0);
            engine.scope_trace = Some(Vec::new());
            let mut replay = Vec::new();
            while engine.offset < engine.source.len() {
                engine.step(true, &mut replay).unwrap();
            }
            assert_eq!(roles, replay, "{} role replay", fixture.id);
            assert_eq!(
                trace,
                engine.scope_trace.take().unwrap(),
                "{} scope replay",
                fixture.id
            );
        }
        assert_eq!(SHELL_ORACLES.len(), 84);
        assert_eq!(assertions, 269);
        assert_eq!(end_stacks, 16);
        eprintln!(
            "real Engine: 84 documents, 269 exact intervals, 16 endStacks, replay/roles/limits PASS"
        );
    }
}

#[cfg(test)]
mod retained_codec_tests {
    use super::*;
    struct Probe {
        bytes: std::io::Cursor<Vec<u8>>,
        largest: usize,
        read: usize,
    }
    impl Read for Probe {
        fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
            self.largest = self.largest.max(b.len());
            let n = self.bytes.read(b)?;
            self.read += n;
            Ok(n)
        }
    }
    #[test]
    fn bounded_reads_and_scatter_write_keep_wire_identical() {
        let mut wire = Vec::new();
        let frame = Frame::new(
            19,
            23,
            Message::Chunk {
                offset: 7,
                bytes: vec![b'x'; MAX_LINE],
            },
        );
        write_frame(&mut wire, &frame).unwrap();
        assert_eq!(&wire[..4], &((HEADER + 4 + MAX_LINE) as u32).to_le_bytes());
        assert_eq!(&wire[4..8], b"R3SY");
        assert_eq!(wire[10], 3);
        assert_eq!(&wire[28..32], &7u32.to_le_bytes());
        let mut reader = Probe {
            bytes: std::io::Cursor::new(wire),
            largest: 0,
            read: 0,
        };
        let f = read_frame(&mut reader).unwrap();
        assert!(matches!(f.message,Message::Chunk{bytes,..}if bytes.len()==MAX_LINE));
        assert!(reader.largest <= 4096);
        struct Writer {
            largest: usize,
            total: usize,
        }
        impl Write for Writer {
            fn write(&mut self, b: &[u8]) -> io::Result<usize> {
                self.largest = self.largest.max(b.len());
                self.total += b.len();
                Ok(b.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let mut writer = Writer {
            largest: 0,
            total: 0,
        };
        write_frame(&mut writer, &frame).unwrap();
        assert!(writer.largest <= 4096);
        assert_eq!(writer.total, 28 + 4 + MAX_LINE);
    }
    #[test]
    fn host_wrong_direction_and_dynamic_styles_reject_before_body() {
        for frame in [
            Frame::new(
                1,
                1,
                Message::Chunk {
                    offset: 0,
                    bytes: vec![0; 4096],
                },
            ),
            Frame::new(
                1,
                1,
                Message::Styles(vec![
                    RoleSpan {
                        start: 0,
                        end: 1,
                        role: SyntaxRole::Value
                    };
                    2
                ]),
            ),
        ] {
            let mut wire = Vec::new();
            write_frame(&mut wire, &frame).unwrap();
            let mut reader = Probe {
                bytes: std::io::Cursor::new(wire),
                largest: 0,
                read: 0,
            };
            assert!(
                read_host_frame(
                    &mut reader,
                    &RetainedSyntaxInput::legacy(Arc::new(String::new())),
                    1
                )
                .is_err()
            );
            assert_eq!(reader.read, 28);
        }
        let frame = Frame::new(
            1,
            1,
            Message::Styles(vec![RoleSpan {
                start: 0,
                end: 1,
                role: SyntaxRole::Value,
            }]),
        );
        let mut wire = Vec::new();
        write_frame(&mut wire, &frame).unwrap();
        wire[28..32].copy_from_slice(&2u32.to_le_bytes());
        let mut reader = Probe {
            bytes: std::io::Cursor::new(wire),
            largest: 0,
            read: 0,
        };
        assert!(
            read_host_frame(
                &mut reader,
                &RetainedSyntaxInput::legacy(Arc::new(String::new())),
                1
            )
            .is_err()
        );
        assert_eq!(reader.read, 32);
    }
    #[test]
    fn raw_body_and_decoded_role_peak_are_both_admitted() {
        let doc = crate::syntax::fixture_document();
        let budget = ConsumerBudget::new(&doc).unwrap();
        let context = crate::syntax::OperationContext::document(
            1,
            std::time::Instant::now() + std::time::Duration::from_secs(5),
            Arc::new(std::sync::atomic::AtomicBool::new(false)),
        );
        let owner = RetainedSyntaxInput::copy_document("x", &doc, &budget, &context).unwrap();
        let base = budget.used();
        let role_bytes = std::mem::size_of::<RoleSpan>();
        let body_bytes = 13;
        let block = budget
            .reserve(
                crate::document_consumer_budget::CONSUMER_LIMIT - base - role_bytes - body_bytes
                    + 1,
            )
            .unwrap();
        let mut wire = Vec::new();
        write_frame(
            &mut wire,
            &Frame::new(
                1,
                1,
                Message::Styles(vec![RoleSpan {
                    start: 0,
                    end: 1,
                    role: SyntaxRole::Value,
                }]),
            ),
        )
        .unwrap();
        let mut reader = Probe {
            bytes: std::io::Cursor::new(wire.clone()),
            largest: 0,
            read: 0,
        };
        let err = read_host_frame(&mut reader, &owner, 1).err().unwrap();
        assert_eq!(err.kind(), io::ErrorKind::OutOfMemory);
        assert_eq!(reader.read, 32);
        drop(block);
        assert_eq!(budget.used(), base);
        let receipt = read_host_frame(&mut wire.as_slice(), &owner, 1).unwrap();
        assert!(budget.peak() >= base + role_bytes + body_bytes);
        assert_eq!(budget.used(), base + role_bytes);
        drop(receipt);
        assert_eq!(budget.used(), base);
    }
}

#[cfg(all(test, windows))]
mod d4_primary_roles {
    use super::*;
    #[test]
    fn dc_primary_roles_beforeoutput_fixtures_preserve_bytes_and_isolate_blocks() {
        let fixtures = [
            (
                "NU-COMMENT",
                "Nushell",
                include_str!("../tests/fixtures/syntax-d4/NU-COMMENT.txt"),
                2,
                8,
                SyntaxRole::Secondary,
            ),
            (
                "NU-STRING",
                "Nushell",
                include_str!("../tests/fixtures/syntax-d4/NU-STRING.txt"),
                9,
                11,
                SyntaxRole::Success,
            ),
            (
                "NU-NUMBER",
                "Nushell",
                include_str!("../tests/fixtures/syntax-d4/NU-NUMBER.txt"),
                8,
                10,
                SyntaxRole::Process,
            ),
            (
                "PS-COMMENT",
                "PowerShell",
                include_str!("../tests/fixtures/syntax-d4/PS-COMMENT.txt"),
                2,
                8,
                SyntaxRole::Secondary,
            ),
            (
                "PS-STRING",
                "PowerShell",
                include_str!("../tests/fixtures/syntax-d4/PS-STRING.txt"),
                1,
                3,
                SyntaxRole::Success,
            ),
            (
                "PS-NUMBER",
                "PowerShell",
                include_str!("../tests/fixtures/syntax-d4/PS-NUMBER.txt"),
                0,
                2,
                SyntaxRole::Process,
            ),
            (
                "PS-BLOCK-COMMENT",
                "PowerShell",
                include_str!("../tests/fixtures/syntax-d4/PS-BLOCK-COMMENT.txt"),
                8,
                14,
                SyntaxRole::Secondary,
            ),
            (
                "PS-NEXT-BLOCK",
                "PowerShell",
                include_str!("../tests/fixtures/syntax-d4/PS-NEXT-BLOCK.txt"),
                1,
                3,
                SyntaxRole::Success,
            ),
            (
                "RUST-COMMENT",
                "Rust",
                include_str!("../tests/fixtures/syntax-d4/RUST-COMMENT.txt"),
                3,
                9,
                SyntaxRole::Secondary,
            ),
            (
                "RUST-STRING",
                "Rust",
                include_str!("../tests/fixtures/syntax-d4/RUST-STRING.txt"),
                9,
                11,
                SyntaxRole::Success,
            ),
            (
                "RUST-NUMBER",
                "Rust",
                include_str!("../tests/fixtures/syntax-d4/RUST-NUMBER.txt"),
                8,
                10,
                SyntaxRole::Process,
            ),
            (
                "RUST-BLOCK-COMMENT",
                "Rust",
                include_str!("../tests/fixtures/syntax-d4/RUST-BLOCK-COMMENT.txt"),
                8,
                14,
                SyntaxRole::Secondary,
            ),
            (
                "RUST-NEXT-BLOCK",
                "Rust",
                include_str!("../tests/fixtures/syntax-d4/RUST-NEXT-BLOCK.txt"),
                9,
                11,
                SyntaxRole::Success,
            ),
        ];
        for (id, language, text, start, end, expected) in fixtures {
            let syntaxes = bundled_syntaxes().unwrap();
            let parser =
                syntect::parsing::ParseState::new(syntaxes.find_syntax_by_name(language).unwrap());
            let mut engine = Engine::new(text.to_owned(), syntaxes, parser, MAX_SPANS, 32).unwrap();
            let mut spans = Vec::new();
            while engine.offset < engine.source.len() {
                engine.step(true, &mut spans).unwrap();
            }
            assert_eq!(spans.first().map(|s| s.start), Some(0), "{id}");
            assert_eq!(
                spans.last().map(|s| s.end as usize),
                Some(text.len()),
                "{id}"
            );
            assert!(
                spans.windows(2).all(|pair| pair[0].end == pair[1].start),
                "{id}"
            );
            let covered = spans
                .iter()
                .filter(|span| (span.end as usize) > start && (span.start as usize) < end)
                .collect::<Vec<_>>();
            assert!(!covered.is_empty(), "{id}");
            assert!(
                covered.iter().all(|span| span.role == expected),
                "{id}: {covered:?}, expected {expected:?}"
            );
        }
    }
}
