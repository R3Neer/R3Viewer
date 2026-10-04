//! Versioned, bounded MessagePack transport independent of the Nushell SDK.
//!
//! A blocking `Read` cannot be interrupted by an atomic flag. Its owner must
//! arrange cancellation of the OS handle/worker. Checks here run between reads.
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{self, Write as _};
use std::io::{self, Read, Write};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

pub const MAGIC: &[u8; 4] = b"R3V1";
pub const VERSION: u16 = 1;
pub const MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_TOTAL_BYTES: usize = 128 * 1024 * 1024;
pub const BYTE_CHUNK_BYTES: usize = 64 * 1024;
pub const MAX_VALUE_DEPTH: usize = 64;
pub const MAX_VALUE_NODES: usize = 1_000_000;
pub const MAX_WIRE_DEPTH: usize = 256;
pub const MAX_WIRE_NODES: usize = 8_000_000;

#[derive(Debug)]
pub enum WireError {
    Io(io::Error),
    Invalid(&'static str),
    Codec(String),
    Cancelled,
}
pub type Result<T> = std::result::Result<T, WireError>;
impl fmt::Display for WireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "wire I/O: {e}"),
            Self::Invalid(e) => write!(f, "invalid wire: {e}"),
            Self::Codec(e) => write!(f, "invalid MessagePack: {e}"),
            Self::Cancelled => f.write_str("wire cancelled"),
        }
    }
}
impl std::error::Error for WireError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}
impl From<io::Error> for WireError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
fn check_cancel(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Relaxed) {
        Err(WireError::Cancelled)
    } else {
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Bound<T> {
    Included(T),
    Excluded(T),
    Unbounded,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum CellPathMember {
    String {
        value: String,
        optional: bool,
        insensitive: bool,
    },
    Index {
        value: u64,
        optional: bool,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Value {
    Nothing,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Binary(#[serde(with = "serde_bytes")] Vec<u8>),
    Date(String),
    Filesize(i64),
    Duration(i64),
    Record(Vec<(String, Value)>),
    List(Vec<Value>),
    IntRange {
        start: i64,
        step: i64,
        end: Bound<i64>,
    },
    FloatRange {
        start: f64,
        step: f64,
        end: Bound<f64>,
    },
    CellPath(Vec<CellPathMember>),
    Glob {
        pattern: String,
        no_expand: bool,
    },
    Error {
        message: String,
    },
    Closure,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ByteStreamKind {
    String,
    Binary,
    Unknown,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Origin {
    Absent,
    SingleValue,
    ListStream,
    ByteStream(ByteStreamKind),
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Start {
    pub version: u16,
    pub cwd: String,
    pub origin: Origin,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Frame {
    Start {
        version: u16,
        cwd: String,
        origin: Origin,
    },
    Value(Value),
    Bytes(#[serde(with = "serde_bytes")] Vec<u8>),
    Ping,
    End,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Payload {
    Absent,
    SingleValue(Value),
    ListStream(Vec<Value>),
    ByteStream {
        kind: ByteStreamKind,
        bytes: Vec<u8>,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub struct TypedInput {
    pub start: Start,
    pub payload: Payload,
}

/// Validate one value iteratively. The caller accumulates the returned node
/// count over a stream. Record keys are ordered, unique and never reordered.
pub fn validate_value(value: &Value, cancel: &AtomicBool) -> Result<usize> {
    let mut work = vec![(value, 1usize)];
    let mut nodes = 0usize;
    while let Some((value, depth)) = work.pop() {
        check_cancel(cancel)?;
        nodes += 1;
        if nodes > MAX_VALUE_NODES {
            return Err(WireError::Invalid("value node limit"));
        }
        if depth > MAX_VALUE_DEPTH {
            return Err(WireError::Invalid("value depth limit"));
        }
        match value {
            Value::Record(entries) => {
                if entries.len() > MAX_VALUE_NODES - nodes {
                    return Err(WireError::Invalid("value node limit"));
                }
                let mut keys = HashSet::new();
                for (key, child) in entries {
                    check_cancel(cancel)?;
                    if !keys.insert(key.as_str()) {
                        return Err(WireError::Invalid("duplicate record key"));
                    }
                    work.push((child, depth + 1));
                }
            }
            Value::List(children) => {
                if children.len() > MAX_VALUE_NODES - nodes {
                    return Err(WireError::Invalid("value node limit"));
                }
                for child in children {
                    check_cancel(cancel)?;
                    work.push((child, depth + 1));
                }
            }
            Value::Date(date) if !is_rfc3339(date) => {
                return Err(WireError::Invalid("date is not RFC3339"));
            }
            _ => {}
        }
    }
    Ok(nodes)
}

fn is_rfc3339(date: &str) -> bool {
    let b = date.as_bytes();
    if b.len() < 20
        || b[4] != b'-'
        || b[7] != b'-'
        || !matches!(b[10], b'T' | b't')
        || b[13] != b':'
        || b[16] != b':'
    {
        return false;
    }
    let number = |range: std::ops::Range<usize>| -> Option<u32> {
        b[range].iter().try_fold(0, |n, c| {
            c.is_ascii_digit()
                .then_some(n * 10 + u32::from(c.wrapping_sub(b'0')))
        })
    };
    let (Some(year), Some(month), Some(day), Some(hour), Some(minute), Some(second)) = (
        number(0..4),
        number(5..7),
        number(8..10),
        number(11..13),
        number(14..16),
        number(17..19),
    ) else {
        return false;
    };
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        _ => return false,
    };
    if day == 0 || day > days || hour > 23 || minute > 59 || second > 60 {
        return false;
    }
    let mut p = 19;
    if b[p] == b'.' {
        p += 1;
        let first = p;
        while p < b.len() && b[p].is_ascii_digit() {
            p += 1;
        }
        if p == first {
            return false;
        }
    }
    if p + 1 == b.len() && matches!(b[p], b'Z' | b'z') {
        return true;
    }
    if p + 6 != b.len() || !matches!(b[p], b'+' | b'-') || b[p + 3] != b':' {
        return false;
    }
    matches!((number(p+1..p+3), number(p+4..p+6)), (Some(h), Some(m)) if h <= 23 && m <= 59)
}

/// Iterative MessagePack scan BEFORE serde sees container lengths. No stack or
/// allocation is proportional to an untrusted declared container length.
/// Count wire nodes in one MessagePack frame payload, excluding magic and the
/// length prefix. Producers accumulate this count across every frame, including
/// Start, Ping and End. This validates the payload without deserializing it.
pub fn messagepack_node_count(data: &[u8], cancel: &AtomicBool) -> Result<usize> {
    check_cancel(cancel)?;
    if data.is_empty() || data.len() > MAX_FRAME_BYTES {
        return Err(WireError::Invalid("frame length limit"));
    }
    preflight(data, cancel)
}

fn preflight(data: &[u8], cancel: &AtomicBool) -> Result<usize> {
    let mut pos = 0usize;
    let mut stack = vec![1u64];
    let mut nodes = 0usize;
    fn take<'a>(data: &'a [u8], pos: &mut usize, n: usize) -> Result<&'a [u8]> {
        let end = pos
            .checked_add(n)
            .ok_or(WireError::Invalid("MessagePack length overflow"))?;
        let bytes = data
            .get(*pos..end)
            .ok_or(WireError::Invalid("truncated MessagePack"))?;
        *pos = end;
        Ok(bytes)
    }
    fn length(data: &[u8], pos: &mut usize, n: usize) -> Result<usize> {
        let mut value = 0usize;
        for byte in take(data, pos, n)? {
            value = (value << 8) | usize::from(*byte);
        }
        Ok(value)
    }
    while !stack.is_empty() {
        check_cancel(cancel)?;
        if *stack.last().expect("nonempty") == 0 {
            stack.pop();
            continue;
        }
        *stack.last_mut().expect("nonempty") -= 1;
        nodes += 1;
        if nodes > MAX_WIRE_NODES {
            return Err(WireError::Invalid("wire node limit"));
        }
        let marker = take(data, &mut pos, 1)?[0];
        let mut children = 0usize;
        let mut bytes = 0usize;
        match marker {
            0x00..=0x7f | 0xe0..=0xff | 0xc0 | 0xc2 | 0xc3 => {}
            0xc1 => return Err(WireError::Invalid("reserved MessagePack marker")),
            0x80..=0x8f => children = usize::from(marker & 15) * 2,
            0x90..=0x9f => children = usize::from(marker & 15),
            0xa0..=0xbf => bytes = usize::from(marker & 31),
            0xc4 | 0xd9 => bytes = length(data, &mut pos, 1)?,
            0xc5 | 0xda => bytes = length(data, &mut pos, 2)?,
            0xc6 | 0xdb => bytes = length(data, &mut pos, 4)?,
            0xc7..=0xc9 => {
                bytes = length(data, &mut pos, 1 << (marker - 0xc7))?
                    .checked_add(1)
                    .ok_or(WireError::Invalid("extension length overflow"))?;
            }
            0xca => bytes = 4,
            0xcb => bytes = 8,
            0xcc | 0xd0 => bytes = 1,
            0xcd | 0xd1 => bytes = 2,
            0xce | 0xd2 => bytes = 4,
            0xcf | 0xd3 => bytes = 8,
            0xd4..=0xd8 => bytes = (1usize << (marker - 0xd4)) + 1,
            0xdc => children = length(data, &mut pos, 2)?,
            0xdd => children = length(data, &mut pos, 4)?,
            0xde => {
                children = length(data, &mut pos, 2)?
                    .checked_mul(2)
                    .ok_or(WireError::Invalid("map length overflow"))?
            }
            0xdf => {
                children = length(data, &mut pos, 4)?
                    .checked_mul(2)
                    .ok_or(WireError::Invalid("map length overflow"))?
            }
        }
        take(data, &mut pos, bytes)?;
        if children != 0 {
            if children > data.len() - pos {
                return Err(WireError::Invalid(
                    "container length exceeds remaining bytes",
                ));
            }
            if children > MAX_WIRE_NODES - nodes {
                return Err(WireError::Invalid("wire node limit"));
            }
            if stack.len() >= MAX_WIRE_DEPTH {
                return Err(WireError::Invalid("wire depth limit"));
            }
            stack.push(children as u64);
        }
    }
    if pos != data.len() {
        return Err(WireError::Invalid(
            "multiple MessagePack objects or trailing bytes",
        ));
    }
    Ok(nodes)
}

pub struct Reader<R> {
    input: R,
    cancel: Arc<AtomicBool>,
    start: Option<Start>,
    total_bytes: usize,
    wire_nodes: usize,
    progress: Option<Arc<AtomicU64>>,
}
impl<R: Read> Reader<R> {
    pub fn new(input: R, cancel: Arc<AtomicBool>) -> Self {
        Self {
            input,
            cancel,
            start: None,
            total_bytes: 0,
            wire_nodes: 0,
            progress: None,
        }
    }
    /// Optional content progress counter for an owning worker's inactivity
    /// deadline. Ping never renews progress. No deadline is enforced here.
    pub fn with_progress(mut self, progress: Arc<AtomicU64>) -> Self {
        self.progress = Some(progress);
        self
    }
    fn mark_progress(&self) {
        if let Some(progress) = &self.progress {
            progress.fetch_add(1, Ordering::Relaxed);
        }
    }
    fn read_exact_checked(&mut self, data: &mut [u8]) -> Result<()> {
        let mut offset = 0;
        while offset < data.len() {
            check_cancel(&self.cancel)?;
            let until = data.len().min(offset + BYTE_CHUNK_BYTES);
            match self.input.read(&mut data[offset..until]) {
                Ok(0) => return Err(WireError::Invalid("unexpected EOF before End")),
                Ok(n) => offset += n,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e.into()),
            }
        }
        check_cancel(&self.cancel)
    }
    fn read_frame(&mut self) -> Result<Frame> {
        let mut prefix = [0; 4];
        self.read_exact_checked(&mut prefix)?;
        let len = u32::from_be_bytes(prefix) as usize;
        if len == 0 || len > MAX_FRAME_BYTES {
            return Err(WireError::Invalid("frame length limit"));
        }
        let total = self
            .total_bytes
            .checked_add(4 + len)
            .ok_or(WireError::Invalid("total length overflow"))?;
        if total > MAX_TOTAL_BYTES {
            return Err(WireError::Invalid("total byte limit"));
        }
        self.total_bytes = total;
        let mut bytes = vec![0; len];
        self.read_exact_checked(&mut bytes)?;
        let nodes = messagepack_node_count(&bytes, &self.cancel)?;
        self.wire_nodes += nodes;
        if self.wire_nodes > MAX_WIRE_NODES {
            return Err(WireError::Invalid("wire node limit"));
        }
        let mut deserializer = rmp_serde::Deserializer::new(bytes.as_slice());
        deserializer.set_max_depth(MAX_WIRE_DEPTH);
        let frame =
            Frame::deserialize(&mut deserializer).map_err(|e| WireError::Codec(e.to_string()))?;
        check_cancel(&self.cancel)?;
        Ok(frame)
    }
    /// Consume only magic and Start so a caller can reject conflicting input
    /// origins immediately, even while the producer remains open.
    pub fn read_start(&mut self) -> Result<Start> {
        if self.start.is_some() {
            return Err(WireError::Invalid("Start already read"));
        }
        let mut magic = [0; 4];
        self.read_exact_checked(&mut magic)?;
        if &magic != MAGIC {
            return Err(WireError::Invalid("wrong magic"));
        }
        self.total_bytes = MAGIC.len();
        let Frame::Start {
            version,
            cwd,
            origin,
        } = self.read_frame()?
        else {
            return Err(WireError::Invalid("first frame must be Start"));
        };
        if version != VERSION {
            return Err(WireError::Invalid("unsupported version"));
        }
        let start = Start {
            version,
            cwd,
            origin,
        };
        self.start = Some(start.clone());
        self.mark_progress();
        Ok(start)
    }
    /// Finish a finite input. End is mandatory and must be followed by EOF.
    pub fn finish(mut self) -> Result<TypedInput> {
        let start = self
            .start
            .take()
            .ok_or(WireError::Invalid("read_start must precede finish"))?;
        let mut values = Vec::new();
        let mut bytes = Vec::new();
        let mut value_nodes = 0usize;
        loop {
            match self.read_frame()? {
                Frame::Start { .. } => return Err(WireError::Invalid("duplicate Start")),
                Frame::Ping => {}
                Frame::Value(value) => {
                    match start.origin {
                        Origin::ListStream => {}
                        Origin::SingleValue if values.is_empty() => {}
                        _ => return Err(WireError::Invalid("Value does not match origin")),
                    }
                    value_nodes += validate_value(&value, &self.cancel)?;
                    if value_nodes > MAX_VALUE_NODES {
                        return Err(WireError::Invalid("value node limit"));
                    }
                    values.push(value);
                    self.mark_progress();
                }
                Frame::Bytes(chunk) => {
                    if !matches!(start.origin, Origin::ByteStream(_)) {
                        return Err(WireError::Invalid("Bytes does not match origin"));
                    }
                    if chunk.len() > BYTE_CHUNK_BYTES {
                        return Err(WireError::Invalid("byte chunk limit"));
                    }
                    // Total wire bytes were checked before each frame allocation.
                    bytes.extend_from_slice(&chunk);
                    self.mark_progress();
                }
                Frame::End => {
                    self.mark_progress();
                    break;
                }
            }
        }
        let mut trailing = [0];
        loop {
            check_cancel(&self.cancel)?;
            match self.input.read(&mut trailing) {
                Ok(0) => break,
                Ok(_) => return Err(WireError::Invalid("bytes after End")),
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e.into()),
            }
        }
        check_cancel(&self.cancel)?;
        let payload = match start.origin {
            Origin::Absent => Payload::Absent,
            Origin::SingleValue => {
                if values.len() != 1 {
                    return Err(WireError::Invalid(
                        "single_value requires exactly one Value",
                    ));
                }
                Payload::SingleValue(values.pop().expect("one value"))
            }
            Origin::ListStream => Payload::ListStream(values),
            Origin::ByteStream(kind) => {
                if kind == ByteStreamKind::String && std::str::from_utf8(&bytes).is_err() {
                    return Err(WireError::Invalid("String ByteStream is not UTF-8"));
                }
                Payload::ByteStream { kind, bytes }
            }
        };
        Ok(TypedInput { start, payload })
    }
}

pub fn write_magic(output: &mut impl Write) -> Result<()> {
    output.write_all(MAGIC)?;
    Ok(())
}
pub fn write_frame(output: &mut impl Write, frame: &Frame) -> Result<()> {
    write_frame_cancelable(output, frame, &AtomicBool::new(false))
}
/// Bounded encoding: serde cannot grow the temporary buffer past one frame.
/// Per-stream total/node budgets are the producer's responsibility too.
pub fn write_frame_cancelable(
    output: &mut impl Write,
    frame: &Frame,
    cancel: &AtomicBool,
) -> Result<()> {
    check_cancel(cancel)?;
    match frame {
        Frame::Start { version, .. } if *version != VERSION => {
            return Err(WireError::Invalid("unsupported version"));
        }
        Frame::Value(value) => {
            validate_value(value, cancel)?;
        }
        Frame::Bytes(bytes) if bytes.len() > BYTE_CHUNK_BYTES => {
            return Err(WireError::Invalid("byte chunk limit"));
        }
        _ => {}
    }
    struct Buffer<'a> {
        bytes: Vec<u8>,
        cancel: &'a AtomicBool,
    }
    impl Write for Buffer<'_> {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            if self.cancel.load(Ordering::Relaxed) {
                return Err(io::Error::other("cancelled"));
            }
            if buf.len() > MAX_FRAME_BYTES - self.bytes.len() {
                return Err(io::Error::other("frame length limit"));
            }
            self.bytes.extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut buffer = Buffer {
        bytes: Vec::new(),
        cancel,
    };
    rmp_serde::encode::write_named(&mut buffer, frame).map_err(|e| {
        if cancel.load(Ordering::Relaxed) {
            WireError::Cancelled
        } else {
            WireError::Codec(e.to_string())
        }
    })?;
    check_cancel(cancel)?;
    output.write_all(&(buffer.bytes.len() as u32).to_be_bytes())?;
    for chunk in buffer.bytes.chunks(BYTE_CHUNK_BYTES) {
        check_cancel(cancel)?;
        output.write_all(chunk)?;
    }
    Ok(())
}

impl TypedInput {
    /// Text strings at the root remain literal. All other values carry explicit
    /// type names; ranges/cellpaths/closures are descriptors, never evaluated.
    pub fn typed_text(&self) -> Result<String> {
        // TypedInput is public; validate callers' manually constructed values too.
        let cancel = AtomicBool::new(false);
        match &self.payload {
            Payload::SingleValue(value) => {
                validate_value(value, &cancel)?;
            }
            Payload::ListStream(values) => {
                let mut nodes = 0usize;
                for value in values {
                    nodes += validate_value(value, &cancel)?;
                    if nodes > MAX_VALUE_NODES {
                        return Err(WireError::Invalid("value node limit"));
                    }
                }
            }
            _ => {}
        }
        let mut text = String::new();
        match &self.payload {
            Payload::Absent => text.push_str("absent"),
            Payload::SingleValue(Value::String(value)) => text.push_str(value),
            Payload::SingleValue(value) => render_value(&mut text, value),
            Payload::ListStream(values) => {
                text.push_str("list_stream[");
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        text.push_str(", ");
                    }
                    render_value(&mut text, value);
                }
                text.push(']');
            }
            Payload::ByteStream {
                kind: ByteStreamKind::String,
                bytes,
            } => text.push_str(
                std::str::from_utf8(bytes)
                    .map_err(|_| WireError::Invalid("String ByteStream is not UTF-8"))?,
            ),
            Payload::ByteStream { kind, bytes } => {
                text.push_str(if *kind == ByteStreamKind::Binary {
                    "binary_stream(0x"
                } else {
                    "unknown_stream(0x"
                });
                hex(&mut text, bytes);
                text.push(')');
            }
        }
        Ok(text)
    }
}
fn hex(text: &mut String, bytes: &[u8]) {
    const HEX: &[u8] = b"0123456789abcdef";
    for byte in bytes {
        text.push(HEX[(byte >> 4) as usize] as char);
        text.push(HEX[(byte & 15) as usize] as char);
    }
}
fn render_value(text: &mut String, value: &Value) {
    match value {
        Value::Nothing => text.push_str("nothing"),
        Value::Bool(v) => {
            let _ = write!(text, "bool({v})");
        }
        Value::Int(v) => {
            let _ = write!(text, "int({v})");
        }
        Value::Float(v) => {
            let _ = write!(text, "float({v:?}; bits=0x{:016x})", v.to_bits());
        }
        Value::String(v) => {
            let _ = write!(text, "string({v:?})");
        }
        Value::Binary(v) => {
            text.push_str("binary(0x");
            hex(text, v);
            text.push(')');
        }
        Value::Date(v) => {
            let _ = write!(text, "date({v:?})");
        }
        Value::Filesize(v) => {
            let _ = write!(text, "filesize({v} bytes)");
        }
        Value::Duration(v) => {
            let _ = write!(text, "duration({v} ns)");
        }
        Value::Record(entries) => {
            text.push_str("record{");
            for (i, (key, value)) in entries.iter().enumerate() {
                if i > 0 {
                    text.push_str(", ");
                }
                let _ = write!(text, "{key:?}: ");
                render_value(text, value);
            }
            text.push('}');
        }
        Value::List(values) => {
            text.push_str("list[");
            for (i, value) in values.iter().enumerate() {
                if i > 0 {
                    text.push_str(", ");
                }
                render_value(text, value);
            }
            text.push(']');
        }
        Value::IntRange { start, step, end } => {
            let _ = write!(text, "int_range(start={start}, step={step}, end={end:?})");
        }
        Value::FloatRange { start, step, end } => {
            let _ = write!(
                text,
                "float_range(start={start:?}, step={step:?}, end={end:?})"
            );
        }
        Value::CellPath(members) => {
            let _ = write!(text, "cell_path({members:?})");
        }
        Value::Glob { pattern, no_expand } => {
            let _ = write!(text, "glob(pattern={pattern:?}, no_expand={no_expand})");
        }
        Value::Error { message } => {
            let _ = write!(text, "error({message:?})");
        }
        Value::Closure => text.push_str("closure(opaque)"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    fn cancel() -> Arc<AtomicBool> {
        Arc::new(AtomicBool::new(false))
    }
    fn start(origin: Origin) -> Frame {
        Frame::Start {
            version: VERSION,
            cwd: "D:/fixture".into(),
            origin,
        }
    }
    fn encode(frames: &[Frame]) -> Vec<u8> {
        let mut bytes = Vec::new();
        write_magic(&mut bytes).unwrap();
        for frame in frames {
            write_frame(&mut bytes, frame).unwrap();
        }
        bytes
    }
    fn decode(bytes: Vec<u8>) -> Result<TypedInput> {
        let mut reader = Reader::new(Cursor::new(bytes), cancel());
        reader.read_start()?;
        reader.finish()
    }
    fn raw_frame(stream: &mut Vec<u8>, bytes: &[u8]) {
        stream.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
        stream.extend_from_slice(bytes);
    }
    fn unchecked_encode(frames: &[Frame]) -> Vec<u8> {
        let mut stream = MAGIC.to_vec();
        for frame in frames {
            raw_frame(&mut stream, &rmp_serde::to_vec_named(frame).unwrap());
        }
        stream
    }
    #[test]
    fn absence_nothing_empty_values_have_distinct_origins() {
        let inputs = [
            decode(encode(&[start(Origin::Absent), Frame::End])).unwrap(),
            decode(encode(&[
                start(Origin::SingleValue),
                Frame::Value(Value::Nothing),
                Frame::End,
            ]))
            .unwrap(),
            decode(encode(&[
                start(Origin::SingleValue),
                Frame::Value(Value::List(vec![])),
                Frame::End,
            ]))
            .unwrap(),
            decode(encode(&[start(Origin::ListStream), Frame::End])).unwrap(),
            decode(encode(&[
                start(Origin::SingleValue),
                Frame::Value(Value::String(String::new())),
                Frame::End,
            ]))
            .unwrap(),
        ];
        let text: Vec<_> = inputs
            .iter()
            .map(|input| input.typed_text().unwrap())
            .collect();
        assert_eq!(text, ["absent", "nothing", "list[]", "list_stream[]", ""]);
    }
    #[test]
    fn all_descriptors_and_record_order_survive() {
        let value = Value::Record(vec![
            ("z".into(), Value::Int(i64::MIN)),
            (
                "a".into(),
                Value::List(vec![
                    Value::Bool(true),
                    Value::Int(i64::MAX),
                    Value::Binary(vec![0, 255]),
                    Value::Date("2024-02-29T23:59:60.125+02:30".into()),
                    Value::Filesize(-1),
                    Value::Duration(i64::MIN),
                    Value::IntRange {
                        start: 0,
                        step: -1,
                        end: Bound::Unbounded,
                    },
                    Value::FloatRange {
                        start: 0.0,
                        step: 0.5,
                        end: Bound::Excluded(1.0),
                    },
                    Value::CellPath(vec![
                        CellPathMember::String {
                            value: "x".into(),
                            optional: true,
                            insensitive: true,
                        },
                        CellPathMember::Index {
                            value: u64::MAX,
                            optional: false,
                        },
                    ]),
                    Value::Glob {
                        pattern: "*".into(),
                        no_expand: true,
                    },
                    Value::Error {
                        message: "oops".into(),
                    },
                    Value::Closure,
                ]),
            ),
        ]);
        let got = decode(encode(&[
            start(Origin::SingleValue),
            Frame::Value(value.clone()),
            Frame::End,
        ]))
        .unwrap();
        assert_eq!(got.payload, Payload::SingleValue(value));
        let text = got.typed_text().unwrap();
        assert!(text.find("\"z\"").unwrap() < text.find("\"a\"").unwrap());
        assert!(text.contains("Unbounded"));
        assert!(text.contains("closure(opaque)"));
    }
    #[test]
    fn float_payload_bits_are_preserved() {
        for bits in [
            0u64,
            1,
            0x8000000000000000,
            0x7ff0000000000000,
            0xfff0000000000000,
            0x7ff8000000000042,
            0xfff8000000000043,
            0x7ff0000000000001,
        ] {
            let got = decode(encode(&[
                start(Origin::SingleValue),
                Frame::Value(Value::Float(f64::from_bits(bits))),
                Frame::End,
            ]))
            .unwrap();
            let Payload::SingleValue(Value::Float(value)) = got.payload else {
                panic!()
            };
            assert_eq!(value.to_bits(), bits, "float bit pattern");
        }
    }
    #[test]
    fn utf8_is_validated_after_chunk_boundary_and_unknown_stays_unknown() {
        let mut first = vec![b'a'; BYTE_CHUNK_BYTES - 1];
        first.push(0xe2);
        let expected = format!("{}€", "a".repeat(BYTE_CHUNK_BYTES - 1));
        let input = decode(encode(&[
            start(Origin::ByteStream(ByteStreamKind::String)),
            Frame::Bytes(first),
            Frame::Bytes(vec![0x82, 0xac]),
            Frame::End,
        ]))
        .unwrap();
        assert_eq!(input.typed_text().unwrap(), expected);
        for bytes in [vec![], vec![b'a'], vec![0xff]] {
            let input = decode(encode(&[
                start(Origin::ByteStream(ByteStreamKind::Unknown)),
                Frame::Bytes(bytes.clone()),
                Frame::End,
            ]))
            .unwrap();
            assert_eq!(
                input.payload,
                Payload::ByteStream {
                    kind: ByteStreamKind::Unknown,
                    bytes
                }
            );
            assert!(input.typed_text().unwrap().starts_with("unknown_stream("));
        }
        assert!(
            decode(encode(&[
                start(Origin::ByteStream(ByteStreamKind::String)),
                Frame::Bytes(vec![0xff]),
                Frame::End
            ]))
            .is_err()
        );
        for kind in [ByteStreamKind::String, ByteStreamKind::Binary] {
            let input = decode(encode(&[start(Origin::ByteStream(kind)), Frame::End])).unwrap();
            assert_eq!(
                input.payload,
                Payload::ByteStream {
                    kind,
                    bytes: vec![]
                }
            );
        }
    }
    #[test]
    fn start_does_not_consume_payload_or_wait_for_eof() {
        struct StartOnly {
            bytes: Cursor<Vec<u8>>,
        }
        impl Read for StartOnly {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                assert!(
                    self.bytes.position() < self.bytes.get_ref().len() as u64,
                    "reader consumed after Start"
                );
                self.bytes.read(buf)
            }
        }
        let mut reader = Reader::new(
            StartOnly {
                bytes: Cursor::new(encode(&[start(Origin::SingleValue)])),
            },
            cancel(),
        );
        assert_eq!(reader.read_start().unwrap().origin, Origin::SingleValue);
    }
    #[test]
    fn state_machine_rejects_invalid_sequences() {
        for frames in [
            vec![Frame::Ping, Frame::End],
            vec![start(Origin::Absent), start(Origin::Absent), Frame::End],
            vec![start(Origin::SingleValue), Frame::End],
            vec![
                start(Origin::SingleValue),
                Frame::Value(Value::Nothing),
                Frame::Value(Value::Nothing),
                Frame::End,
            ],
            vec![
                start(Origin::Absent),
                Frame::Value(Value::Nothing),
                Frame::End,
            ],
            vec![start(Origin::ListStream), Frame::Bytes(vec![]), Frame::End],
            vec![
                start(Origin::ByteStream(ByteStreamKind::Binary)),
                Frame::Value(Value::Nothing),
                Frame::End,
            ],
            vec![start(Origin::Absent), Frame::Ping],
            vec![start(Origin::Absent), Frame::End, Frame::Ping],
        ] {
            assert!(decode(encode(&frames)).is_err(), "{frames:?}");
        }
        assert!(decode(encode(&[start(Origin::Absent), Frame::Ping, Frame::End])).is_ok());
    }
    #[test]
    fn truncated_frames_bad_magic_version_and_trailing_bytes_fail() {
        let stream = encode(&[
            start(Origin::SingleValue),
            Frame::Value(Value::String("x".into())),
            Frame::End,
        ]);
        for length in 0..stream.len() {
            assert!(
                decode(stream[..length].to_vec()).is_err(),
                "length={length}"
            );
        }
        let mut wrong = stream.clone();
        wrong[0] = b'X';
        assert!(decode(wrong).is_err());
        let mut trailing = stream;
        trailing.push(0);
        assert!(decode(trailing).is_err());
        let unknown = Frame::Start {
            version: VERSION + 1,
            cwd: "x".into(),
            origin: Origin::Absent,
        };
        assert!(decode(unchecked_encode(&[unknown, Frame::End])).is_err());
    }
    #[test]
    fn length_bombs_fail_before_allocating_or_reading_payload() {
        struct HeaderOnly(Cursor<Vec<u8>>);
        impl Read for HeaderOnly {
            fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
                assert!(
                    self.0.position() < self.0.get_ref().len() as u64,
                    "payload read attempted"
                );
                self.0.read(bytes)
            }
        }
        for size in [0, MAX_FRAME_BYTES as u32 + 1, u32::MAX] {
            let mut bytes = MAGIC.to_vec();
            bytes.extend_from_slice(&size.to_be_bytes());
            let mut reader = Reader::new(HeaderOnly(Cursor::new(bytes)), cancel());
            assert!(reader.read_start().is_err());
        }
        for bomb in [
            vec![0xdd, 255, 255, 255, 255],
            vec![0xdf, 255, 255, 255, 255],
            vec![0xdb, 255, 255, 255, 255],
            vec![0xc6, 255, 255, 255, 255],
            vec![0xc9, 255, 255, 255, 255],
        ] {
            assert!(preflight(&bomb, &AtomicBool::new(false)).is_err());
            let mut stream = encode(&[start(Origin::Absent)]);
            raw_frame(&mut stream, &bomb);
            assert!(decode(stream).is_err());
        }
    }
    #[test]
    fn public_node_count_matches_reader_and_rejects_invalid_payloads() {
        let cancel = AtomicBool::new(false);
        let frames = [
            start(Origin::ListStream),
            Frame::Ping,
            Frame::Value(Value::List(vec![Value::Int(4), Value::Nothing])),
            Frame::End,
        ];
        let mut expected = 0;
        for frame in &frames {
            let bytes = rmp_serde::to_vec_named(frame).unwrap();
            expected += messagepack_node_count(&bytes, &cancel).unwrap();
        }
        let mut reader = Reader::new(Cursor::new(encode(&frames)), super::tests::cancel());
        reader.read_start().unwrap();
        for _ in 1..frames.len() {
            reader.read_frame().unwrap();
        }
        assert_eq!(reader.wire_nodes, expected);
        assert_eq!(
            messagepack_node_count(&[0x92, 0xc0, 0xc0], &cancel).unwrap(),
            3
        );
        assert!(messagepack_node_count(&[], &cancel).is_err());
        assert!(messagepack_node_count(&[0xdd, 255, 255, 255, 255], &cancel).is_err());
        assert!(messagepack_node_count(&[0xc0, 0xc0], &cancel).is_err());
        let mut nested = vec![0x91; MAX_WIRE_DEPTH + 1];
        nested.push(0xc0);
        assert!(messagepack_node_count(&nested, &cancel).is_err());
        assert!(messagepack_node_count(&vec![0; MAX_FRAME_BYTES + 1], &cancel).is_err());
    }
    #[test]
    fn preflight_requires_exactly_one_object_and_limits_stack() {
        assert!(preflight(&[0xc1], &AtomicBool::new(false)).is_err());
        assert!(preflight(&[0xc0, 0xc0], &AtomicBool::new(false)).is_err());
        let mut nested = vec![0x91; MAX_WIRE_DEPTH + 1];
        nested.push(0xc0);
        assert!(preflight(&nested, &AtomicBool::new(false)).is_err());
        assert!(preflight(&[0x92, 0xc0], &AtomicBool::new(false)).is_err());
    }
    #[test]
    fn value_depth_and_duplicates_are_rejected_before_success() {
        fn nested(depth: usize) -> Value {
            let mut v = Value::Nothing;
            for _ in 1..depth {
                v = Value::List(vec![v]);
            }
            v
        }
        let cancel = AtomicBool::new(false);
        assert!(validate_value(&nested(MAX_VALUE_DEPTH), &cancel).is_ok());
        assert!(validate_value(&nested(MAX_VALUE_DEPTH + 1), &cancel).is_err());
        assert!(
            decode(unchecked_encode(&[
                start(Origin::SingleValue),
                Frame::Value(nested(MAX_VALUE_DEPTH + 1)),
                Frame::End
            ]))
            .is_err()
        );
        let dup = Value::Record(vec![
            ("x".into(), Value::Nothing),
            ("x".into(), Value::Int(1)),
        ]);
        assert!(
            decode(unchecked_encode(&[
                start(Origin::SingleValue),
                Frame::Value(dup),
                Frame::End
            ]))
            .is_err()
        );
        assert!(
            decode(encode(&[
                start(Origin::SingleValue),
                Frame::Value(nested(MAX_VALUE_DEPTH)),
                Frame::End
            ]))
            .is_ok()
        );
    }
    #[test]
    fn stream_node_and_total_budgets_accumulate() {
        // Two individually legal Values together cross the semantic budget.
        let value = Value::List(vec![Value::Nothing; MAX_VALUE_NODES / 2]);
        let stream = encode(&[
            start(Origin::ListStream),
            Frame::Value(value.clone()),
            Frame::Value(value),
            Frame::End,
        ]);
        assert!(matches!(
            decode(stream),
            Err(WireError::Invalid("value node limit"))
        ));
        let mut reader = Reader::new(
            Cursor::new(encode(&[start(Origin::Absent), Frame::Ping, Frame::End])),
            cancel(),
        );
        reader.read_start().unwrap();
        reader.total_bytes = MAX_TOTAL_BYTES - 4;
        assert!(matches!(
            reader.finish(),
            Err(WireError::Invalid("total byte limit"))
        ));
    }
    #[test]
    fn cancellation_checked_between_reads_and_value_traversal() {
        let flag = cancel();
        flag.store(true, Ordering::Relaxed);
        let mut reader = Reader::new(
            Cursor::new(encode(&[start(Origin::Absent), Frame::End])),
            flag,
        );
        assert!(matches!(reader.read_start(), Err(WireError::Cancelled)));
        let flag = AtomicBool::new(true);
        assert!(matches!(
            validate_value(&Value::Nothing, &flag),
            Err(WireError::Cancelled)
        ));
        assert!(matches!(
            write_frame_cancelable(&mut Vec::new(), &Frame::Ping, &flag),
            Err(WireError::Cancelled)
        ));
        assert!(matches!(
            preflight(&[0xc0], &flag),
            Err(WireError::Cancelled)
        ));
    }
    #[test]
    fn encoding_does_not_emit_prefix_if_frame_validation_fails() {
        let mut out = Vec::new();
        assert!(write_frame(&mut out, &Frame::Bytes(vec![0; BYTE_CHUNK_BYTES + 1])).is_err());
        assert!(out.is_empty());
        assert!(
            write_frame(
                &mut out,
                &Frame::Value(Value::String("x".repeat(MAX_FRAME_BYTES)))
            )
            .is_err()
        );
        assert!(out.is_empty());
    }
    #[test]
    fn ping_never_updates_the_content_progress_counter() {
        let progress = Arc::new(AtomicU64::new(0));
        let mut reader = Reader::new(
            Cursor::new(encode(&[
                start(Origin::Absent),
                Frame::Ping,
                Frame::Ping,
                Frame::End,
            ])),
            cancel(),
        )
        .with_progress(progress.clone());
        reader.read_start().unwrap();
        assert_eq!(progress.load(Ordering::Relaxed), 1);
        assert!(matches!(reader.read_frame().unwrap(), Frame::Ping));
        assert_eq!(progress.load(Ordering::Relaxed), 1);
        assert!(matches!(reader.read_frame().unwrap(), Frame::Ping));
        assert_eq!(progress.load(Ordering::Relaxed), 1);
        reader.finish().unwrap();
        assert_eq!(progress.load(Ordering::Relaxed), 2);
    }
    #[test]
    fn dates_retain_offsets_and_reject_invalid_calendar_values() {
        for date in [
            "2023-02-29T00:00:00Z",
            "2024-01-01T24:00:00Z",
            "2024-01-01T00:00:00",
            "2024-01-01T00:00:00.Z",
            "2024-01-01T00:00:00+25:00",
            "garbage",
        ] {
            assert!(
                validate_value(&Value::Date(date.into()), &AtomicBool::new(false)).is_err(),
                "{date}"
            );
        }
    }
}
