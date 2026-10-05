//! Private binary protocol and lexical worker. No terminal or inferred file I/O.
use crate::syntax::{FallbackReason, RoleSpan, SyntaxRole, Viewport};
use std::io::{self, Read, Write};

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
fn append_view(out: &mut Vec<u8>, view: Viewport) {
    out.extend(view.first_line.to_le_bytes());
    out.extend(view.line_count.to_le_bytes());
}

pub(crate) fn write_frame(out: &mut impl Write, frame: &Frame) -> io::Result<()> {
    let (kind, mut payload) = (
        match &frame.message {
            Message::Boot => 0,
            Message::Init { .. } => 1,
            Message::Ack { .. } => 2,
            Message::Chunk { .. } => 3,
            Message::SourceEnd => 4,
            Message::Ready => 5,
            Message::View(_) => 6,
            Message::All => 7,
            Message::Continue => 8,
            Message::Supersede(_) => 9,
            Message::Progress { .. } => 10,
            Message::Styles(_) => 11,
            Message::Fallback(_) => 12,
            Message::Shutdown => 13,
        },
        Vec::new(),
    );
    match &frame.message {
        Message::Init {
            bytes,
            language,
            spans,
            checkpoints,
        } => {
            if *bytes as usize > MAX_SOURCE
                || language.is_empty()
                || language.len() > 256
                || *spans as usize > MAX_SPANS
                || *spans == 0
                || *checkpoints as usize > MAX_CHECKPOINTS
            {
                return Err(invalid());
            }
            payload.extend(bytes.to_le_bytes());
            payload.extend(spans.to_le_bytes());
            payload.extend(checkpoints.to_le_bytes());
            payload.extend((language.len() as u16).to_le_bytes());
            payload.extend(language.as_bytes());
        }
        Message::Ack { offset } => payload.extend(offset.to_le_bytes()),
        Message::Chunk { offset, bytes } => {
            if bytes.is_empty() || bytes.len() > MAX_LINE {
                return Err(invalid());
            }
            payload.extend(offset.to_le_bytes());
            payload.extend(bytes);
        }
        Message::View(view) | Message::Supersede(view) => {
            if view.line_count == 0 || view.line_count as usize > MAX_VIEW_LINES {
                return Err(invalid());
            }
            append_view(&mut payload, *view);
        }
        Message::Progress { line, offset } => {
            payload.extend(line.to_le_bytes());
            payload.extend(offset.to_le_bytes());
        }
        Message::Styles(spans) => {
            if spans.len() > MAX_SPANS {
                return Err(invalid());
            }
            payload.extend((spans.len() as u32).to_le_bytes());
            for span in spans {
                payload.extend(span.start.to_le_bytes());
                payload.extend(span.end.to_le_bytes());
                payload.push(span.role as u8);
            }
        }
        Message::Fallback(reason) => payload.push(*reason as u8),
        _ => {}
    }
    if HEADER + payload.len() > MAX_FRAME {
        return Err(invalid());
    }
    out.write_all(&((HEADER + payload.len()) as u32).to_le_bytes())?;
    out.write_all(MAGIC)?;
    out.write_all(&1_u16.to_le_bytes())?;
    out.write_all(&[kind, 0])?;
    out.write_all(&frame.generation.to_le_bytes())?;
    out.write_all(&frame.request.to_le_bytes())?;
    out.write_all(&payload)?;
    out.flush()
}

pub(crate) fn read_frame(input: &mut impl Read) -> io::Result<Frame> {
    let mut prefix = [0; 4];
    input.read_exact(&mut prefix)?;
    let length = u32::from_le_bytes(prefix) as usize;
    // Validate wire length before any allocation.
    if !(HEADER..=MAX_FRAME).contains(&length) {
        return Err(invalid());
    }
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(length).map_err(io::Error::other)?;
    bytes.resize(length, 0);
    input.read_exact(&mut bytes)?;
    let mut d = Decoder(&bytes);
    if d.take(4)? != MAGIC || d.u16()? != 1 {
        return Err(invalid());
    }
    let kind = d.u8()?;
    if d.u8()? != 0 {
        return Err(invalid());
    }
    let generation = d.u64()?;
    let request = d.u64()?;
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
            for _ in 0..count {
                let start = d.u32()?;
                let end = d.u32()?;
                let role = SyntaxRole::from_wire(d.u8()?).ok_or_else(invalid)?;
                if start >= end || end as usize > MAX_SOURCE {
                    return Err(invalid());
                }
                spans.push(RoleSpan { start, end, role });
            }
            Message::Styles(spans)
        }
        12 => Message::Fallback(FallbackReason::from_wire(d.u8()?).ok_or_else(invalid)?),
        13 => Message::Shutdown,
        _ => return Err(invalid()),
    };
    if !d.0.is_empty() {
        return Err(invalid());
    }
    Ok(Frame {
        generation,
        request,
        message,
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
        "/toml.syntax.pack"
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
}
