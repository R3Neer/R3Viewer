//! Session-owned asynchronous syntax controller. UI calls never perform pipe I/O.
use crate::{
    syntax_process::ChildJob,
    syntax_worker::{self, Frame, Message},
    worker_process::ReapTicket,
};
use std::{
    collections::VecDeque,
    fmt, io,
    mem::size_of,
    process::{Child, Command, Stdio},
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SyntaxRole {
    Value,
    Secondary,
    Accent,
    Success,
    Process,
    Heading,
}
impl SyntaxRole {
    pub(crate) fn from_wire(value: u8) -> Option<Self> {
        Some(match value {
            0 => Self::Value,
            1 => Self::Secondary,
            2 => Self::Accent,
            3 => Self::Success,
            4 => Self::Process,
            5 => Self::Heading,
            _ => return None,
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoleSpan {
    pub start: u32,
    pub end: u32,
    pub role: SyntaxRole,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Viewport {
    pub first_line: u32,
    pub line_count: u16,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FallbackReason {
    SourceBudget,
    LongLine,
    UnknownLanguage,
    StartupTimeout,
    LineTimeout,
    RequestTimeout,
    SpanBudget,
    InvalidIpc,
    WorkerExited,
    Cancelled,
    UnsupportedPlatform,
    ContainmentFailed,
    GrammarError,
    OperationBudget,
}
impl FallbackReason {
    pub(crate) fn from_wire(value: u8) -> Option<Self> {
        Some(match value {
            0 => Self::SourceBudget,
            1 => Self::LongLine,
            2 => Self::UnknownLanguage,
            3 => Self::StartupTimeout,
            4 => Self::LineTimeout,
            5 => Self::RequestTimeout,
            6 => Self::SpanBudget,
            7 => Self::InvalidIpc,
            8 => Self::WorkerExited,
            9 => Self::Cancelled,
            10 => Self::UnsupportedPlatform,
            11 => Self::ContainmentFailed,
            12 => Self::GrammarError,
            13 => Self::OperationBudget,
            _ => return None,
        })
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::SourceBudget => "syntax source budget exceeded",
            Self::LongLine => "syntax line exceeds 64 KiB",
            Self::UnknownLanguage => "unknown syntax language",
            Self::StartupTimeout => "syntax initialization timed out",
            Self::LineTimeout => "syntax line processing timed out",
            Self::RequestTimeout => "syntax request timed out",
            Self::SpanBudget => "syntax style budget exceeded",
            Self::InvalidIpc => "invalid syntax worker response",
            Self::WorkerExited => "syntax worker exited",
            Self::Cancelled => "syntax cancelled",
            Self::UnsupportedPlatform => {
                "syntax memory containment is not validated on this platform"
            }
            Self::ContainmentFailed => "syntax worker containment failed",
            Self::GrammarError => "syntax grammar failed",
            Self::OperationBudget => "syntax parser operation budget exceeded",
        }
    }
}
impl fmt::Display for FallbackReason {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.write_str(self.description())
    }
}
impl std::error::Error for FallbackReason {}
#[derive(Debug, Clone)]
pub struct SyntaxLimits {
    pub source_bytes: usize,
    pub spans: usize,
    pub checkpoints: usize,
    pub style_cache_bytes: usize,
    pub startup: Duration,
    pub per_line: Duration,
    pub viewport_total: Duration,
    pub inline_total: Duration,
}
impl Default for SyntaxLimits {
    fn default() -> Self {
        Self {
            source_bytes: syntax_worker::MAX_SOURCE,
            spans: syntax_worker::MAX_SPANS,
            checkpoints: syntax_worker::MAX_CHECKPOINTS,
            style_cache_bytes: 8 * 1024 * 1024,
            startup: Duration::from_secs(2),
            per_line: Duration::from_millis(500),
            viewport_total: Duration::from_secs(2),
            inline_total: Duration::from_secs(5),
        }
    }
}
impl SyntaxLimits {
    fn bounded(mut self) -> Self {
        self.source_bytes = self.source_bytes.min(syntax_worker::MAX_SOURCE);
        self.spans = self.spans.clamp(1, syntax_worker::MAX_SPANS);
        self.checkpoints = self.checkpoints.min(syntax_worker::MAX_CHECKPOINTS);
        self.style_cache_bytes = self.style_cache_bytes.min(8 * 1024 * 1024);
        self.startup = self.startup.min(Duration::from_secs(2));
        self.per_line = self.per_line.min(Duration::from_millis(500));
        self.viewport_total = self.viewport_total.min(Duration::from_secs(2));
        self.inline_total = self.inline_total.min(Duration::from_secs(5));
        self
    }
}
#[derive(Debug)]
pub enum SyntaxEvent {
    Ready {
        generation: u64,
    },
    Styles {
        generation: u64,
        request: u64,
        viewport: Option<Viewport>,
        spans: Vec<RoleSpan>,
    },
    Plain {
        generation: u64,
        reason: FallbackReason,
    },
}
#[derive(Debug, Clone, Copy, Default)]
pub struct SyntaxMemory {
    pub controller_source_bytes: usize,
    pub pending_source_bytes: usize,
    pub child_source_bytes: usize,
    pub style_cache_bytes: usize,
    pub pending_style_bytes: usize,
}

struct Document {
    source: Arc<String>,
    language: String,
    generation: u64,
    revision: u64,
}
enum Update {
    Set(Document),
    Clear(u64),
}
#[derive(Clone, Copy)]
struct Request {
    generation: u64,
    id: u64,
    viewport: Option<Viewport>,
}
struct Latched {
    generation: u64,
    reason: FallbackReason,
    delivered: bool,
}
struct State {
    generation: u64,
    revision: u64,
    has_document: bool,
    sequence: u64,
    update: Option<Update>,
    request: Option<Request>,
    last_request: u64,
    event: Option<SyntaxEvent>,
    plain: Option<Latched>,
    stopping: bool,
    pause: Option<(u64, ReapTicket)>,
    memory: SyntaxMemory,
}
struct Shared {
    state: Mutex<State>,
    wake: Condvar,
    cancelled: Arc<AtomicBool>,
}
impl Shared {
    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }
    fn latch(&self, generation: u64, reason: FallbackReason) {
        let mut state = self.lock();
        if state.generation == generation && state.plain.is_none() {
            state.event = None;
            state.request = None;
            state.plain = Some(Latched {
                generation,
                reason,
                delivered: false,
            });
            state.memory.pending_style_bytes = 0;
        }
    }
    fn publish(&self, event: SyntaxEvent) {
        let mut state = self.lock();
        let (generation, request) = match &event {
            SyntaxEvent::Ready { generation } => (*generation, None),
            SyntaxEvent::Styles {
                generation,
                request,
                ..
            } => (*generation, Some(*request)),
            SyntaxEvent::Plain { .. } => unreachable!(),
        };
        if state.generation == generation
            && state.plain.is_none()
            && request.is_none_or(|id| id == state.last_request)
        {
            state.memory.pending_style_bytes = match &event {
                SyntaxEvent::Styles { spans, .. } => spans.capacity() * size_of::<RoleSpan>(),
                _ => 0,
            };
            state.event = Some(event);
        }
    }
    fn take_current_request(&self, doc: &Document) -> Result<Option<Request>, WaitError> {
        let mut state = self.lock();
        // This check and take share the lock with Set/Clear and Request. A replacement
        // published after wait's last status check must keep its own request pending.
        if state.stopping || self.cancelled.load(Ordering::Relaxed) {
            return Err(WaitError::Stopped);
        }
        if state.revision != doc.revision
            || state.generation != doc.generation
            || !state.has_document
            || state.update.is_some()
            || state
                .request
                .as_ref()
                .is_some_and(|request| request.generation != doc.generation)
        {
            return Err(WaitError::Changed);
        }
        if let Some(plain) = &state.plain {
            return Err(WaitError::Failure(plain.reason));
        }
        Ok(state.request.take())
    }
    fn status(&self, revision: u64) -> Result<(), WaitError> {
        if self.cancelled.load(Ordering::Relaxed) {
            return Err(WaitError::Stopped);
        }
        let state = self.lock();
        if state.stopping {
            Err(WaitError::Stopped)
        } else if state.revision != revision {
            Err(WaitError::Changed)
        } else {
            Ok(())
        }
    }
}

pub struct SyntaxClient {
    shared: Arc<Shared>,
    owner: Option<JoinHandle<()>>,
}
pub struct SyntaxShutdown {
    owner: Option<JoinHandle<()>>,
}
impl SyntaxShutdown {
    /// Call after the terminal session has restored its modes.
    pub fn join(mut self) -> io::Result<()> {
        if let Some(owner) = self.owner.take() {
            owner
                .join()
                .map_err(|_| io::Error::other("syntax controller panicked"))?;
        }
        Ok(())
    }
}
impl SyntaxClient {
    pub fn new(limits: SyntaxLimits, cancelled: Arc<AtomicBool>) -> io::Result<Self> {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                generation: 0,
                revision: 0,
                has_document: false,
                sequence: 0,
                update: None,
                request: None,
                last_request: 0,
                event: None,
                plain: None,
                stopping: false,
                pause: None,
                memory: Default::default(),
            }),
            wake: Condvar::new(),
            cancelled,
        });
        let background = Arc::clone(&shared);
        let limits = limits.bounded();
        let owner = thread::Builder::new()
            .name("syntax-controller".into())
            .spawn(move || controller(background, limits))?;
        Ok(Self {
            shared,
            owner: Some(owner),
        })
    }
    /// A different document must receive a new generation; a latched failure cannot reset in place.
    pub fn set_document(&self, source: Arc<String>, language: String, generation: u64) {
        let mut state = self.shared.lock();
        if state.stopping {
            return;
        }
        if state.generation != generation {
            state.plain = None;
        }
        state.generation = generation;
        state.has_document = true;
        state.revision = state.revision.wrapping_add(1);
        state.event = None;
        state.request = None;
        state.memory.pending_style_bytes = 0;
        state.memory.pending_source_bytes = source.len();
        let revision = state.revision;
        state.update = Some(Update::Set(Document {
            source,
            language,
            generation,
            revision,
        }));
        drop(state);
        self.shared.wake.notify_one();
    }
    /// Suspend this generation and receive confirmation after child and IPC are reaped.
    /// The caller resumes only after the structural worker's own receipt is Reaped.
    pub fn pause_document(&self, generation: u64) -> ReapTicket {
        let mut state = self.shared.lock();
        if state.stopping {
            let ticket = ReapTicket::pending();
            ticket.complete(false);
            return ticket;
        }
        if state.generation != generation {
            return ReapTicket::reaped();
        }
        let ticket = state
            .pause
            .as_ref()
            .map(|(_, ticket)| ticket.clone())
            .unwrap_or_else(ReapTicket::pending);
        state.has_document = false;
        state.revision = state.revision.wrapping_add(1);
        state.update = Some(Update::Clear(state.revision));
        state.request = None;
        state.event = None;
        state.memory.pending_source_bytes = 0;
        state.memory.pending_style_bytes = 0;
        state.pause = Some((state.revision, ticket.clone()));
        drop(state);
        self.shared.wake.notify_one();
        ticket
    }
    pub fn clear_document(&self, generation: u64) {
        let mut state = self.shared.lock();
        if state.stopping {
            return;
        }
        if state.generation != generation {
            state.plain = None;
        }
        state.generation = generation;
        state.has_document = false;
        state.revision = state.revision.wrapping_add(1);
        state.update = Some(Update::Clear(state.revision));
        state.request = None;
        state.event = None;
        state.memory.pending_source_bytes = 0;
        state.memory.pending_style_bytes = 0;
        drop(state);
        self.shared.wake.notify_one();
    }
    /// Returns zero when ignored, including before Set or after Clear.
    /// A viewport may be queued while a valid Set is still initializing.
    pub fn request_viewport(&self, generation: u64, viewport: Viewport) -> u64 {
        self.request(generation, Some(viewport))
    }
    fn request(&self, generation: u64, viewport: Option<Viewport>) -> u64 {
        let mut state = self.shared.lock();
        if state.stopping
            || !state.has_document
            || state.generation != generation
            || state.plain.is_some()
        {
            return 0;
        }
        let Some(id) = state.sequence.checked_add(1) else {
            drop(state);
            self.shared.latch(generation, FallbackReason::InvalidIpc);
            return 0;
        };
        state.sequence = id;
        state.last_request = id;
        state.request = Some(Request {
            generation,
            id,
            viewport,
        });
        drop(state);
        self.shared.wake.notify_one();
        id
    }
    pub fn poll(&mut self) -> Option<SyntaxEvent> {
        let mut state = self.shared.lock();
        if let Some(plain) = state.plain.as_mut() {
            if !plain.delivered {
                plain.delivered = true;
                return Some(SyntaxEvent::Plain {
                    generation: plain.generation,
                    reason: plain.reason,
                });
            }
            return None;
        }
        state.memory.pending_style_bytes = 0;
        state.event.take()
    }
    /// Accounting reports retained source copies/cache, not an exact heap bound.
    pub fn memory_usage(&self) -> SyntaxMemory {
        self.shared.lock().memory
    }
    pub fn cancel(&self) {
        self.shared.cancelled.store(true, Ordering::Relaxed);
        self.shared.wake.notify_one();
    }
    pub fn shutdown(mut self) -> SyntaxShutdown {
        self.stop();
        SyntaxShutdown {
            owner: self.owner.take(),
        }
    }
    fn stop(&self) {
        let mut state = self.shared.lock();
        state.stopping = true;
        state.update = None;
        state.request = None;
        state.memory.pending_source_bytes = 0;
        drop(state);
        self.shared.wake.notify_one();
    }
}
impl Drop for SyntaxClient {
    fn drop(&mut self) {
        self.stop();
    }
}

pub fn highlight_inline(
    source: Arc<String>,
    language: String,
    limits: SyntaxLimits,
    cancelled: Arc<AtomicBool>,
) -> Result<Vec<RoleSpan>, FallbackReason> {
    let limits = limits.bounded();
    let mut client = SyntaxClient::new(limits.clone(), Arc::clone(&cancelled))
        .map_err(|_| FallbackReason::WorkerExited)?;
    client.set_document(source, language, 1);
    client.request(1, None);
    let stop = Instant::now() + limits.startup + limits.inline_total + Duration::from_secs(1);
    let result = loop {
        if cancelled.load(Ordering::Relaxed) {
            break Err(FallbackReason::Cancelled);
        }
        match client.poll() {
            Some(SyntaxEvent::Styles { spans, .. }) => break Ok(spans),
            Some(SyntaxEvent::Plain { reason, .. }) => break Err(reason),
            _ => {}
        }
        if Instant::now() >= stop {
            break Err(FallbackReason::RequestTimeout);
        }
        thread::sleep(Duration::from_millis(2));
    };
    let shutdown = client.shutdown();
    if shutdown.join().is_err() {
        return Err(FallbackReason::WorkerExited);
    }
    result
}

#[derive(Debug)]
enum WaitError {
    Changed,
    Stopped,
    Failure(FallbackReason),
}
struct Connection {
    child: Child,
    job: Option<ChildJob>,
    send: Option<SyncSender<Frame>>,
    receive: Receiver<io::Result<Frame>>,
    io: Option<JoinHandle<()>>,
    closed: bool,
}
impl Connection {
    fn spawn() -> Result<Self, FallbackReason> {
        let executable = std::env::current_exe().map_err(|_| FallbackReason::WorkerExited)?;
        let mut command = Command::new(executable);
        command
            .arg("--syntax-worker")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(windows_sys::Win32::System::Threading::CREATE_NO_WINDOW);
        }
        let (send, incoming) = mpsc::sync_channel::<Frame>(1);
        let (finished, receive) = mpsc::sync_channel(1);
        let child = command.spawn().map_err(|_| FallbackReason::WorkerExited)?;
        let mut connection = Self {
            child,
            job: None,
            send: Some(send),
            receive,
            io: None,
            closed: false,
        };
        connection.job = Some(
            ChildJob::attach(&connection.child).map_err(|_| FallbackReason::ContainmentFailed)?,
        );
        let mut input = connection
            .child
            .stdin
            .take()
            .ok_or(FallbackReason::WorkerExited)?;
        let mut output = connection
            .child
            .stdout
            .take()
            .ok_or(FallbackReason::WorkerExited)?;
        connection.io = Some(
            thread::Builder::new()
                .name("syntax-ipc".into())
                .spawn(move || {
                    let mut result = syntax_worker::read_frame(&mut output);
                    loop {
                        let failed = result.is_err();
                        if finished.try_send(result).is_err() || failed {
                            break;
                        }
                        let Ok(frame) = incoming.recv() else {
                            break;
                        };
                        result = syntax_worker::write_frame(&mut input, &frame)
                            .and_then(|()| syntax_worker::read_frame(&mut output));
                    }
                })
                .map_err(|_| FallbackReason::WorkerExited)?,
        );
        Ok(connection)
    }
    fn wait(
        &self,
        shared: &Shared,
        revision: u64,
        per_line: Instant,
        total: Instant,
        timeout: FallbackReason,
    ) -> Result<Frame, WaitError> {
        loop {
            shared.status(revision)?;
            let now = Instant::now();
            if now >= total {
                return Err(WaitError::Failure(timeout));
            }
            if now >= per_line {
                return Err(WaitError::Failure(FallbackReason::LineTimeout));
            }
            match self.receive.recv_timeout(Duration::from_millis(2)) {
                Ok(Ok(frame)) => {
                    shared.status(revision)?;
                    if Instant::now() >= total {
                        return Err(WaitError::Failure(timeout));
                    }
                    if Instant::now() >= per_line {
                        return Err(WaitError::Failure(FallbackReason::LineTimeout));
                    }
                    return Ok(frame);
                }
                Ok(Err(error)) => {
                    return Err(WaitError::Failure(
                        if error.kind() == io::ErrorKind::InvalidData {
                            FallbackReason::InvalidIpc
                        } else {
                            FallbackReason::WorkerExited
                        },
                    ));
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(_) => return Err(WaitError::Failure(FallbackReason::WorkerExited)),
            }
        }
    }
    fn exchange(
        &self,
        frame: Frame,
        shared: &Shared,
        revision: u64,
        per_line: Instant,
        total: Instant,
        timeout: FallbackReason,
    ) -> Result<Frame, WaitError> {
        self.send
            .as_ref()
            .unwrap()
            .try_send(frame)
            .map_err(|_| WaitError::Failure(FallbackReason::InvalidIpc))?;
        self.wait(shared, revision, per_line, total, timeout)
    }
    fn check_alive(&mut self) -> Result<(), WaitError> {
        match self.child.try_wait() {
            Ok(None) => Ok(()),
            Ok(Some(_)) | Err(_) => Err(WaitError::Failure(FallbackReason::WorkerExited)),
        }
    }
    fn close(&mut self) -> io::Result<()> {
        if self.closed {
            return Ok(());
        }
        self.send.take();
        let mut failure = None;
        // Job close and explicit kill target only the owned child. Pipes release the I/O reader.
        if let Some(job) = self.job.take() {
            job.close();
        }
        match self.child.try_wait() {
            Ok(None) => {
                if let Err(error) = self.child.kill() {
                    failure = Some(error);
                }
            }
            Ok(Some(_)) => {}
            Err(error) => {
                failure = Some(error);
                let _ = self.child.kill();
            }
        }
        if let Err(error) = self.child.wait() {
            failure.get_or_insert(error);
        }
        if let Some(io) = self.io.take()
            && io.join().is_err()
        {
            failure.get_or_insert_with(|| io::Error::other("syntax IPC thread panicked"));
        }
        self.closed = true;
        if let Some(error) = failure {
            Err(error)
        } else {
            Ok(())
        }
    }
}
impl Drop for Connection {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

fn check_frame(frame: Frame, generation: u64, request: u64) -> Result<Message, WaitError> {
    if frame.generation != generation || frame.request != request {
        return Err(WaitError::Failure(FallbackReason::InvalidIpc));
    }
    match frame.message {
        Message::Fallback(reason) => Err(WaitError::Failure(reason)),
        message => Ok(message),
    }
}
fn preflight(doc: &Document, limits: &SyntaxLimits, shared: &Shared) -> Result<(), WaitError> {
    if doc.source.len() > limits.source_bytes || doc.language.is_empty() || doc.language.len() > 256
    {
        return Err(WaitError::Failure(FallbackReason::SourceBudget));
    }
    let mut line = 0;
    let mut lines = 1;
    for (index, byte) in doc.source.bytes().enumerate() {
        if index.is_multiple_of(65536) {
            shared.status(doc.revision)?;
        }
        line += 1;
        if line > syntax_worker::MAX_LINE {
            return Err(WaitError::Failure(FallbackReason::LongLine));
        }
        if byte == b'\n' {
            line = 0;
            lines += 1;
            if lines > 1_000_000 {
                return Err(WaitError::Failure(FallbackReason::SourceBudget));
            }
        }
    }
    Ok(())
}
fn initialize(
    connection: &Connection,
    doc: &Document,
    limits: &SyntaxLimits,
    shared: &Shared,
    deadline: Instant,
) -> Result<(), WaitError> {
    let boot = connection.wait(
        shared,
        doc.revision,
        deadline,
        deadline,
        FallbackReason::StartupTimeout,
    )?;
    if boot.generation != 0 || boot.request != 0 || !matches!(boot.message, Message::Boot) {
        return Err(WaitError::Failure(FallbackReason::InvalidIpc));
    }
    let response = connection.exchange(
        Frame::new(
            doc.generation,
            0,
            Message::Init {
                bytes: doc.source.len() as u32,
                language: doc.language.clone(),
                spans: limits.spans as u32,
                checkpoints: limits.checkpoints as u16,
            },
        ),
        shared,
        doc.revision,
        (Instant::now() + limits.per_line).min(deadline),
        deadline,
        FallbackReason::StartupTimeout,
    )?;
    if !matches!(
        check_frame(response, doc.generation, 0)?,
        Message::Ack { offset: 0 }
    ) {
        return Err(WaitError::Failure(FallbackReason::InvalidIpc));
    }
    let mut offset = 0;
    for bytes in doc.source.as_bytes().chunks(syntax_worker::MAX_LINE) {
        let next = offset + bytes.len();
        let response = connection.exchange(
            Frame::new(
                doc.generation,
                0,
                Message::Chunk {
                    offset: offset as u32,
                    bytes: bytes.to_vec(),
                },
            ),
            shared,
            doc.revision,
            (Instant::now() + limits.per_line).min(deadline),
            deadline,
            FallbackReason::StartupTimeout,
        )?;
        if !matches!(check_frame(response,doc.generation,0)?,Message::Ack{offset} if offset as usize==next)
        {
            return Err(WaitError::Failure(FallbackReason::InvalidIpc));
        }
        offset = next;
    }
    let response = connection.exchange(
        Frame::new(doc.generation, 0, Message::SourceEnd),
        shared,
        doc.revision,
        (Instant::now() + limits.per_line).min(deadline),
        deadline,
        FallbackReason::StartupTimeout,
    )?;
    if !matches!(check_frame(response, doc.generation, 0)?, Message::Ready) {
        return Err(WaitError::Failure(FallbackReason::InvalidIpc));
    }
    Ok(())
}
fn validate_spans(
    spans: &[RoleSpan],
    doc: &Document,
    request: Request,
    limits: &SyntaxLimits,
) -> Result<(), WaitError> {
    if spans.len() > limits.spans {
        return Err(WaitError::Failure(FallbackReason::SpanBudget));
    }
    // Byte bounds are original snapshot coordinates; no ANSI or safe-text transformations here.
    let mut previous = 0;
    for span in spans {
        let start = span.start as usize;
        let end = span.end as usize;
        if start >= end
            || end > doc.source.len()
            || start < previous
            || !doc.source.is_char_boundary(start)
            || !doc.source.is_char_boundary(end)
        {
            return Err(WaitError::Failure(FallbackReason::InvalidIpc));
        }
        previous = end;
    }
    let expected = if let Some(view) = request.viewport {
        let mut offset = 0usize;
        let mut start = None;
        let mut end = None;
        for (line, part) in doc.source.split_inclusive('\n').enumerate() {
            if line as u64 == view.first_line as u64 {
                start = Some(offset);
            }
            if line as u64 == view.first_line as u64 + view.line_count as u64 {
                end = Some(offset);
                break;
            }
            offset += part.len();
        }
        start.unwrap_or(doc.source.len())..end.unwrap_or(doc.source.len())
    } else {
        0..doc.source.len()
    };
    let mut cursor = expected.start;
    for span in spans {
        if span.start as usize != cursor || span.end as usize > expected.end {
            return Err(WaitError::Failure(FallbackReason::InvalidIpc));
        }
        cursor = span.end as usize;
    }
    if cursor != expected.end {
        return Err(WaitError::Failure(FallbackReason::InvalidIpc));
    }
    Ok(())
}
fn run_request(
    connection: &Connection,
    doc: &Document,
    mut request: Request,
    limits: &SyntaxLimits,
    shared: &Shared,
) -> Result<(Request, Vec<RoleSpan>), WaitError> {
    if request
        .viewport
        .is_some_and(|v| v.line_count == 0 || v.line_count as usize > syntax_worker::MAX_VIEW_LINES)
    {
        return Err(WaitError::Failure(FallbackReason::InvalidIpc));
    }
    let duration = |request: Request| {
        if request.viewport.is_some() {
            limits.viewport_total
        } else {
            limits.inline_total
        }
    };
    let mut total = Instant::now() + duration(request);
    let message = request.viewport.map_or(Message::All, Message::View);
    let mut phase_deadline = Instant::now() + limits.per_line;
    let mut response = connection.exchange(
        Frame::new(doc.generation, request.id, message),
        shared,
        doc.revision,
        phase_deadline,
        total,
        FallbackReason::RequestTimeout,
    )?;
    let mut progress = None;
    loop {
        match check_frame(response, doc.generation, request.id)? {
            Message::Styles(spans) => {
                validate_spans(&spans, doc, request, limits)?;
                if Instant::now() >= total {
                    return Err(WaitError::Failure(FallbackReason::RequestTimeout));
                }
                if Instant::now() >= phase_deadline {
                    return Err(WaitError::Failure(FallbackReason::LineTimeout));
                }
                return Ok((request, spans));
            }
            Message::Progress { line, offset } => {
                if line == 0
                    || line > 1_000_000
                    || offset as usize > doc.source.len()
                    || !doc.source.is_char_boundary(offset as usize)
                    || progress.is_some_and(|(last_line, last_offset)| {
                        line != last_line + 1 || offset <= last_offset
                    })
                {
                    return Err(WaitError::Failure(FallbackReason::InvalidIpc));
                }
                progress = Some((line, offset));
                // Check OLD request's budget before replacing it; progress never renews total.
                if Instant::now() >= total {
                    return Err(WaitError::Failure(FallbackReason::RequestTimeout));
                }
                let newer = shared.take_current_request(doc)?;
                let message = if let Some(newer) = newer {
                    let Some(view) = newer.viewport else {
                        return Err(WaitError::Failure(FallbackReason::InvalidIpc));
                    };
                    if view.line_count == 0
                        || view.line_count as usize > syntax_worker::MAX_VIEW_LINES
                    {
                        return Err(WaitError::Failure(FallbackReason::InvalidIpc));
                    }
                    request = newer;
                    total = Instant::now() + duration(request);
                    progress = None;
                    Message::Supersede(view)
                } else {
                    Message::Continue
                };
                phase_deadline = Instant::now() + limits.per_line;
                response = connection.exchange(
                    Frame::new(doc.generation, request.id, message),
                    shared,
                    doc.revision,
                    phase_deadline,
                    total,
                    FallbackReason::RequestTimeout,
                )?;
            }
            _ => return Err(WaitError::Failure(FallbackReason::InvalidIpc)),
        }
    }
}
struct Cached {
    viewport: Viewport,
    spans: Vec<RoleSpan>,
}
const MAX_CACHE_ENTRIES: usize = 256;
struct StyleCache {
    entries: VecDeque<Cached>,
    budget: usize,
}
impl StyleCache {
    fn new(budget: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            budget,
        }
    }
    fn bytes(&self) -> usize {
        self.entries.capacity() * size_of::<Cached>()
            + self
                .entries
                .iter()
                .map(|entry| entry.spans.capacity() * size_of::<RoleSpan>())
                .sum::<usize>()
    }
    fn clear(&mut self) {
        self.entries = VecDeque::new();
    }
    fn get(&mut self, viewport: Viewport) -> Option<Vec<RoleSpan>> {
        let index = self
            .entries
            .iter()
            .position(|entry| entry.viewport == viewport)?;
        let entry = self.entries.remove(index).unwrap();
        let spans = entry.spans.clone();
        self.entries.push_back(entry);
        Some(spans)
    }
    fn insert(&mut self, viewport: Viewport, spans: &[RoleSpan]) {
        // Empty responses need no cache entry; retained metadata is charged for all other entries.
        let cost = size_of::<Cached>() + std::mem::size_of_val(spans);
        if spans.is_empty() || cost > self.budget {
            return;
        }
        if let Some(index) = self
            .entries
            .iter()
            .position(|entry| entry.viewport == viewport)
        {
            self.entries.remove(index);
        }
        self.entries.push_back(Cached {
            viewport,
            spans: spans.to_vec(),
        });
        self.entries.shrink_to_fit();
        while self.entries.len() > MAX_CACHE_ENTRIES || self.bytes() > self.budget {
            self.entries.pop_front();
            if self.entries.is_empty() {
                self.clear();
                break;
            }
            self.entries.shrink_to_fit();
        }
    }
}
fn close_connection(connection: &mut Option<Connection>) -> bool {
    connection
        .take()
        .is_none_or(|mut connection| connection.close().is_ok())
}
fn controller(shared: Arc<Shared>, limits: SyntaxLimits) {
    let mut connection = None;
    let mut cleanup_failed = false;
    let mut document: Option<Document> = None;
    let mut cache = StyleCache::new(limits.style_cache_bytes);
    loop {
        if shared.cancelled.load(Ordering::Relaxed) {
            let generation = shared.lock().generation;
            shared.latch(generation, FallbackReason::Cancelled);
            break;
        }
        if shared.lock().stopping {
            break;
        }
        let update = {
            let mut state = shared.lock();
            let update = state.update.take();
            if update.is_some() {
                state.memory.pending_source_bytes = 0;
            }
            update
        };
        if let Some(update) = update {
            let processed_revision = match &update {
                Update::Set(doc) => doc.revision,
                Update::Clear(revision) => *revision,
            };
            if !close_connection(&mut connection) {
                cleanup_failed = true;
                let generation = shared.lock().generation;
                shared.latch(generation, FallbackReason::WorkerExited);
                break;
            }
            {
                let mut state = shared.lock();
                if state
                    .pause
                    .as_ref()
                    .is_some_and(|(revision, _)| *revision <= processed_revision)
                    && let Some((_, ticket)) = state.pause.take()
                {
                    ticket.complete(true);
                }
            }
            document = None;
            cache.clear();
            {
                let mut state = shared.lock();
                state.memory.controller_source_bytes = 0;
                state.memory.child_source_bytes = 0;
                state.memory.style_cache_bytes = 0;
            }
            if let Update::Set(doc) = update {
                {
                    let mut state = shared.lock();
                    state.memory.controller_source_bytes = doc.source.len();
                }
                if shared.lock().plain.is_some() {
                    continue;
                }
                let deadline = Instant::now() + limits.startup;
                let initialized = (|| {
                    preflight(&doc, &limits, &shared)?;
                    if !crate::syntax_process::supported() {
                        return Err(WaitError::Failure(FallbackReason::UnsupportedPlatform));
                    }
                    connection = Some(Connection::spawn().map_err(WaitError::Failure)?);
                    shared.lock().memory.child_source_bytes = doc.source.len();
                    initialize(
                        connection.as_ref().unwrap(),
                        &doc,
                        &limits,
                        &shared,
                        deadline,
                    )
                })();
                match initialized {
                    Ok(()) => {
                        shared.lock().memory.child_source_bytes = doc.source.len();
                        shared.publish(SyntaxEvent::Ready {
                            generation: doc.generation,
                        });
                        document = Some(doc);
                    }
                    Err(WaitError::Failure(reason)) => {
                        shared.latch(doc.generation, reason);
                        if !close_connection(&mut connection) {
                            cleanup_failed = true;
                            break;
                        }
                        {
                            let mut state = shared.lock();
                            state.memory.controller_source_bytes = 0;
                            state.memory.child_source_bytes = 0;
                        }
                    }
                    Err(WaitError::Changed) => {
                        if !close_connection(&mut connection) {
                            cleanup_failed = true;
                            break;
                        }
                        {
                            let mut state = shared.lock();
                            state.memory.controller_source_bytes = 0;
                            state.memory.child_source_bytes = 0;
                        }
                        continue;
                    }
                    Err(WaitError::Stopped) => {
                        shared.latch(doc.generation, FallbackReason::Cancelled);
                        break;
                    }
                }
            }
        }
        // Detect death even without a request. Never serve cached styles from a dead worker.
        if connection
            .as_mut()
            .is_some_and(|worker| worker.check_alive().is_err())
        {
            if let Some(doc) = document.as_ref() {
                shared.latch(doc.generation, FallbackReason::WorkerExited);
            }
            cache.clear();
            if !close_connection(&mut connection) {
                cleanup_failed = true;
                break;
            }
            document = None;
            let mut state = shared.lock();
            state.memory = SyntaxMemory {
                pending_source_bytes: state.memory.pending_source_bytes,
                ..Default::default()
            };
            continue;
        }
        let request = {
            let mut state = shared.lock();
            // Update and document identity are checked in the same lock as take, including
            // replacement arriving immediately after Ready. A newer request stays pending.
            if state.update.is_some()
                || document
                    .as_ref()
                    .is_some_and(|doc| doc.revision != state.revision)
            {
                drop(state);
                continue;
            }
            if document.is_some() && state.plain.is_none() {
                state.request.take()
            } else {
                None
            }
        };
        if let (Some(request), Some(doc), Some(worker)) =
            (request, document.as_ref(), connection.as_mut())
        {
            if request.generation != doc.generation || shared.lock().plain.is_some() {
                continue;
            }
            if let Some(spans) = request.viewport.and_then(|view| cache.get(view)) {
                shared.publish(SyntaxEvent::Styles {
                    generation: doc.generation,
                    request: request.id,
                    viewport: request.viewport,
                    spans,
                });
                continue;
            }
            let result = run_request(worker, doc, request, &limits, &shared).and_then(|result| {
                worker.check_alive()?;
                Ok(result)
            });
            match result {
                Ok((request, spans)) => {
                    if let Some(viewport) = request.viewport {
                        cache.insert(viewport, &spans);
                        shared.lock().memory.style_cache_bytes = cache.bytes();
                    }
                    shared.publish(SyntaxEvent::Styles {
                        generation: doc.generation,
                        request: request.id,
                        viewport: request.viewport,
                        spans,
                    });
                }
                Err(WaitError::Failure(reason)) => {
                    shared.latch(doc.generation, reason);
                    cache.clear();
                    if !close_connection(&mut connection) {
                        cleanup_failed = true;
                        break;
                    }
                    document = None;
                    let mut state = shared.lock();
                    state.memory = SyntaxMemory {
                        pending_source_bytes: state.memory.pending_source_bytes,
                        ..Default::default()
                    };
                }
                Err(WaitError::Changed) => {}
                Err(WaitError::Stopped) => {
                    shared.latch(doc.generation, FallbackReason::Cancelled);
                    break;
                }
            }
            continue;
        }
        let state = shared.lock();
        if state.update.is_none() && state.request.is_none() && !state.stopping {
            let _ = shared
                .wake
                .wait_timeout(state, Duration::from_millis(25))
                .unwrap_or_else(|e| e.into_inner());
        }
    }
    let closed = close_connection(&mut connection);
    let reaped = closed && !cleanup_failed;
    let mut state = shared.lock();
    if let Some((_, ticket)) = state.pause.take() {
        ticket.complete(reaped);
    }
    state.stopping = true;
    state.memory = Default::default();
    state.update = None;
    state.request = None;
}
#[cfg(test)]
mod tests {
    use super::*;
    // No controller is spawned: barriers force the exact wait-status/Progress-take
    // interleaving against the same public UI methods and production transaction.
    fn dormant_client() -> SyntaxClient {
        SyntaxClient {
            shared: Arc::new(Shared {
                state: Mutex::new(State {
                    generation: 0,
                    revision: 0,
                    has_document: false,
                    sequence: 0,
                    update: None,
                    request: None,
                    last_request: 0,
                    event: None,
                    plain: None,
                    stopping: false,
                    pause: None,
                    memory: Default::default(),
                }),
                wake: Condvar::new(),
                cancelled: Arc::new(AtomicBool::new(false)),
            }),
            owner: None,
        }
    }
    #[test]
    fn requests_require_an_active_or_pending_document() {
        let client = dormant_client();
        let view = Viewport {
            first_line: 0,
            line_count: 1,
        };
        assert_eq!(client.request_viewport(0, view), 0);
        client.clear_document(7);
        assert_eq!(client.request_viewport(7, view), 0);
        client.set_document(Arc::new("x\n".into()), "rs".into(), 8);
        assert_ne!(client.request_viewport(8, view), 0);
        client.clear_document(8);
        assert_eq!(client.request_viewport(8, view), 0);
        assert!(client.shared.lock().request.is_none());
    }
    #[test]
    fn progress_take_preserves_replacement_after_last_wait_status() {
        use std::sync::Barrier;
        let client = dormant_client();
        let view = Viewport {
            first_line: 0,
            line_count: 1,
        };
        client.set_document(Arc::new("old\n".into()), "rs".into(), 1);
        let Some(Update::Set(old)) = client.shared.lock().update.take() else {
            panic!("missing doc")
        };
        let before = Barrier::new(2);
        let after = Barrier::new(2);
        let latest = thread::scope(|scope| {
            let ui = scope.spawn(|| {
                before.wait();
                client.set_document(Arc::new("new\n".into()), "rs".into(), 2);
                let id = client.request_viewport(2, view);
                after.wait();
                id
            });
            assert!(client.shared.status(old.revision).is_ok());
            before.wait();
            after.wait();
            assert!(matches!(
                client.shared.take_current_request(&old),
                Err(WaitError::Changed)
            ));
            ui.join().unwrap()
        });
        assert_eq!(client.shared.lock().request.as_ref().unwrap().id, latest);
        let Some(Update::Set(new)) = client.shared.lock().update.take() else {
            panic!("missing replacement")
        };
        assert_eq!(
            client
                .shared
                .take_current_request(&new)
                .unwrap()
                .unwrap()
                .id,
            latest
        );
        // Same-document supersede is still consumed, and cancellation never consumes it.
        let supersede = client.request_viewport(
            2,
            Viewport {
                first_line: 1,
                line_count: 1,
            },
        );
        assert_eq!(
            client
                .shared
                .take_current_request(&new)
                .unwrap()
                .unwrap()
                .id,
            supersede
        );
        let pending = client.request_viewport(2, view);
        client.cancel();
        assert!(matches!(
            client.shared.take_current_request(&new),
            Err(WaitError::Stopped)
        ));
        assert_eq!(client.shared.lock().request.as_ref().unwrap().id, pending);
    }
    #[test]
    fn cache_charges_metadata_bounds_entries_and_evicts_lru() {
        let view = |first_line| Viewport {
            first_line,
            line_count: 1,
        };
        let spans = [RoleSpan {
            start: 0,
            end: 1,
            role: SyntaxRole::Value,
        }];
        let cost = size_of::<Cached>() + size_of::<RoleSpan>();
        let mut cache = StyleCache::new(cost * 2);
        for first in 0..20_000 {
            cache.insert(view(first), &[]);
        }
        assert!(cache.entries.is_empty());
        assert_eq!(cache.bytes(), 0);
        cache.insert(view(1), &spans);
        assert_eq!(cache.bytes(), cost);
        cache.insert(view(2), &spans);
        assert!(cache.get(view(1)).is_some());
        cache.insert(view(3), &spans);
        assert!(cache.get(view(2)).is_none());
        assert!(cache.get(view(1)).is_some());
        assert!(cache.bytes() <= cost * 2);
        let mut large = StyleCache::new(8 * 1024 * 1024);
        for first in 0..20_000 {
            large.insert(view(first), &spans);
        }
        assert_eq!(large.entries.len(), MAX_CACHE_ENTRIES);
        assert!(large.bytes() > 0 && large.bytes() <= large.budget);
        assert!(large.get(view(0)).is_none());
        large.clear();
        assert_eq!(large.bytes(), 0);
    }
    #[test]
    fn current_generation_fallback_overrides_newer_styles_and_latches() {
        let mut client =
            SyntaxClient::new(Default::default(), Arc::new(AtomicBool::new(false))).unwrap();
        {
            let mut state = client.shared.lock();
            state.generation = 10;
            state.last_request = 2;
        }
        client.shared.publish(SyntaxEvent::Styles {
            generation: 10,
            request: 2,
            viewport: None,
            spans: vec![RoleSpan {
                start: 0,
                end: 1,
                role: SyntaxRole::Accent,
            }],
        });
        // Failure from request1 still invalidates the generation, including request2's result.
        client.shared.latch(10, FallbackReason::LineTimeout);
        client.shared.publish(SyntaxEvent::Ready { generation: 10 });
        assert!(matches!(
            client.poll(),
            Some(SyntaxEvent::Plain {
                generation: 10,
                reason: FallbackReason::LineTimeout
            })
        ));
        assert!(client.poll().is_none());
        assert_eq!(
            client.request_viewport(
                10,
                Viewport {
                    first_line: 0,
                    line_count: 1
                }
            ),
            0
        );
        client.clear_document(11);
        client.shared.publish(SyntaxEvent::Ready { generation: 11 });
        assert!(matches!(
            client.poll(),
            Some(SyntaxEvent::Ready { generation: 11 })
        ));
        client.shutdown().join().unwrap();
    }
    #[test]
    fn spans_reject_utf8_midpoints_order_and_incomplete_inline() {
        let doc = Document {
            source: Arc::new("é界".into()),
            language: "rs".into(),
            generation: 1,
            revision: 1,
        };
        let request = Request {
            generation: 1,
            id: 1,
            viewport: None,
        };
        let invalid = vec![RoleSpan {
            start: 1,
            end: 5,
            role: SyntaxRole::Value,
        }];
        assert!(validate_spans(&invalid, &doc, request, &Default::default()).is_err());
        assert!(
            validate_spans(
                &[RoleSpan {
                    start: 0,
                    end: 2,
                    role: SyntaxRole::Value
                }],
                &doc,
                request,
                &Default::default()
            )
            .is_err()
        );
        assert!(
            validate_spans(
                &[RoleSpan {
                    start: 0,
                    end: 5,
                    role: SyntaxRole::Value
                }],
                &doc,
                request,
                &Default::default()
            )
            .is_ok()
        );
    }
}
