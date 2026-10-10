#[path = "syntax_retained.rs"]
mod retained;
pub(crate) use retained::{
    OperationContext, RetainedRoleSet, RetainedSyntaxEvent, RetainedSyntaxInput,
};
// Session-owned asynchronous syntax controller. UI calls never perform pipe I/O.
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
    /// Capacity of the last Vec handed to the legacy caller; its lifetime is caller-owned.
    pub legacy_handoff_bytes: usize,
}

struct Document {
    _metadata: Option<crate::document_consumer_budget::Charge>,
    source: Arc<RetainedSyntaxInput>,
    context: OperationContext,
    limits: SyntaxLimits,
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
    event: Option<RetainedSyntaxEvent>,
    context: OperationContext,
    cleanup: CleanupStatus,
    retry_cleanup: bool,
    plain: Option<Latched>,
    stopping: bool,
    pause: Option<(u64, ReapTicket)>,
    memory: SyntaxMemory,
    limits: SyntaxLimits,
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
    #[cfg(test)]
    fn publish(&self, event: RetainedSyntaxEvent) {
        self.publish_at(event, Instant::now());
    }
    #[cfg(test)]
    fn publish_at(&self, event: RetainedSyntaxEvent, now: Instant) {
        self.publish_checked(event, now, None);
    }
    fn publish_for(&self, doc: &Document, event: RetainedSyntaxEvent) {
        self.publish_checked(event, Instant::now(), Some((doc.revision, &doc.context)));
    }
    fn publish_checked(
        &self,
        event: RetainedSyntaxEvent,
        now: Instant,
        binding: Option<(u64, &OperationContext)>,
    ) {
        let mut state = self.lock();
        if state.stopping
            || self.cancelled.load(Ordering::Acquire)
            || binding.is_some_and(|(revision, context)| {
                state.revision != revision || !state.context.same(context)
            })
        {
            return;
        }
        let (generation, request) = match &event {
            RetainedSyntaxEvent::Ready { generation } => (*generation, None),
            RetainedSyntaxEvent::Styles {
                generation,
                request,
                ..
            } => (*generation, Some(*request)),
            RetainedSyntaxEvent::Plain { .. } => unreachable!(),
        };
        if let Err(reason) = state.context.check(now) {
            if state.generation == generation && state.plain.is_none() {
                state.event = None;
                state.plain = Some(Latched {
                    generation,
                    reason,
                    delivered: false,
                });
            }
            return;
        }
        if state.generation == generation
            && state.plain.is_none()
            && request.is_none_or(|id| id == state.last_request)
        {
            state.memory.pending_style_bytes = match &event {
                RetainedSyntaxEvent::Styles { roles, .. } => roles.retained_bytes(),
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
        if let Err(reason) = doc.context.check(Instant::now()) {
            return Err(WaitError::Failure(reason));
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
        if let Err(reason) = state.context.check(Instant::now()) {
            return Err(WaitError::Failure(reason));
        }
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
                context: OperationContext::LegacySource,
                cleanup: CleanupStatus::Reaped { failed: false },
                retry_cleanup: false,
                plain: None,
                stopping: false,
                pause: None,
                memory: Default::default(),
                limits: limits.clone().bounded(),
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
        let limits = self.shared.lock().limits.clone();
        let _ = self.set_retained_document(
            RetainedSyntaxInput::legacy(source),
            language,
            generation,
            limits,
            OperationContext::LegacySource,
        );
    }
    pub(crate) fn set_retained_document(
        &self,
        source: Arc<RetainedSyntaxInput>,
        language: String,
        generation: u64,
        limits: SyntaxLimits,
        context: OperationContext,
    ) -> Result<(), FallbackReason> {
        context.check(Instant::now())?;
        if source.budget().is_some() != matches!(context, OperationContext::Document { .. }) {
            return Err(FallbackReason::InvalidIpc);
        }
        let metadata = source
            .budget()
            .map(|budget| {
                let bytes = size_of::<Document>()
                    .checked_add(size_of::<Shared>() + size_of::<State>() + 4 * size_of::<usize>())
                    .and_then(|n| n.checked_add(language.capacity()))
                    .ok_or(FallbackReason::SourceBudget)?;
                budget
                    .reserve(bytes)
                    .map_err(|_| FallbackReason::SourceBudget)
            })
            .transpose()?;
        let mut state = self.shared.lock();
        if state.stopping {
            return Err(FallbackReason::Cancelled);
        }
        if state.cleanup == CleanupStatus::Pending || PENDING_PRESENT.load(Ordering::Acquire) {
            return Err(FallbackReason::ContainmentFailed);
        }
        let revision = state
            .revision
            .checked_add(1)
            .ok_or(FallbackReason::InvalidIpc)?;
        if state.generation != generation {
            state.plain = None;
        }
        state.generation = generation;
        state.context = context.clone();
        state.has_document = true;
        state.revision = revision;
        state.event = None;
        state.request = None;
        state.memory.pending_style_bytes = 0;
        state.memory.pending_source_bytes = source.len();
        let revision = state.revision;
        state.update = Some(Update::Set(Document {
            _metadata: metadata,
            source,
            context,
            limits: limits.bounded(),
            language,
            generation,
            revision,
        }));
        drop(state);
        self.shared.wake.notify_one();
        Ok(())
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
    #[allow(dead_code)] // Retained request entry is exercised here; D4b consumer follows.
    pub(crate) fn request_retained(
        &self,
        generation: u64,
        viewport: Option<Viewport>,
        context: &OperationContext,
    ) -> u64 {
        self.request_context(generation, viewport, Some(context))
    }
    fn request(&self, generation: u64, viewport: Option<Viewport>) -> u64 {
        self.request_context(generation, viewport, None)
    }
    fn request_context(
        &self,
        generation: u64,
        viewport: Option<Viewport>,
        expected: Option<&OperationContext>,
    ) -> u64 {
        let mut state = self.shared.lock();
        if expected.is_some_and(|e| !state.context.same(e))
            || expected.is_none() && !matches!(state.context, OperationContext::LegacySource)
        {
            return 0;
        }
        if state.stopping
            || state.cleanup == CleanupStatus::Pending
            || PENDING_PRESENT.load(Ordering::Acquire)
            || !state.has_document
            || state.generation != generation
            || state.plain.is_some()
        {
            return 0;
        }
        if let Err(reason) = state.context.check(Instant::now()) {
            drop(state);
            self.shared.latch(generation, reason);
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
        self.poll_retained().map(|event| match event {
            RetainedSyntaxEvent::Ready { generation } => SyntaxEvent::Ready { generation },
            RetainedSyntaxEvent::Plain { generation, reason } => {
                SyntaxEvent::Plain { generation, reason }
            }
            RetainedSyntaxEvent::Styles {
                generation,
                request,
                viewport,
                roles,
            } => {
                if roles.input().budget().is_some() {
                    SyntaxEvent::Plain {
                        generation,
                        reason: FallbackReason::InvalidIpc,
                    }
                } else {
                    SyntaxEvent::Styles {
                        generation,
                        request,
                        viewport,
                        spans: {
                            let copy = roles.spans().to_vec();
                            self.shared.lock().memory.legacy_handoff_bytes =
                                copy.capacity() * size_of::<RoleSpan>();
                            copy
                        },
                    }
                }
            }
        })
    }
    pub(crate) fn poll_retained(&mut self) -> Option<RetainedSyntaxEvent> {
        self.poll_retained_at(Instant::now())
    }
    fn poll_retained_at(&mut self, now: Instant) -> Option<RetainedSyntaxEvent> {
        let mut state = self.shared.lock();
        let authority = if self.shared.cancelled.load(Ordering::Acquire) {
            Err(FallbackReason::Cancelled)
        } else {
            state.context.check(now)
        };
        if state.event.is_some()
            && let Err(reason) = authority
        {
            state.event = None;
            if state.plain.is_none() {
                state.plain = Some(Latched {
                    generation: state.generation,
                    reason,
                    delivered: false,
                });
            }
        }
        if let Some(plain) = state.plain.as_mut() {
            if !plain.delivered {
                plain.delivered = true;
                return Some(RetainedSyntaxEvent::Plain {
                    generation: plain.generation,
                    reason: plain.reason,
                });
            }
            return None;
        }
        state.memory.pending_style_bytes = 0;
        state.event.take()
    }
    #[cfg(test)]
    pub(crate) fn own_controller_thread(&self) -> Option<std::thread::ThreadId> {
        self.owner.as_ref().map(|owner| owner.thread().id())
    }
    #[cfg(test)]
    pub(crate) fn own_close_pipe_fault(enabled: bool) {
        CLOSE_PIPE_FAULT.store(enabled, Ordering::Release);
    }
    /// Scheduler adopts the already-owned client without resetting its generation.
    #[cfg(test)]
    pub(crate) fn current_generation(&self) -> u64 {
        self.shared.lock().generation
    }
    /// Accounting reports retained source copies/cache, not an exact heap bound.
    pub fn cleanup_status(&self) -> CleanupStatus {
        if PENDING_PRESENT.load(Ordering::Acquire) {
            CleanupStatus::Pending
        } else {
            self.shared.lock().cleanup
        }
    }
    pub fn retry_cleanup(&self) {
        self.shared.lock().retry_cleanup = true;
        self.shared.wake.notify_one();
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupStatus {
    Pending,
    Reaped { failed: bool },
}
struct Outgoing {
    frame: Frame,
    deadline: Instant,
    context: OperationContext,
    revision: u64,
}
struct IoFiles {
    input: Option<std::fs::File>,
    output: Option<std::fs::File>,
}
struct IoReceipt {
    files: IoFiles,
    error: Option<io::Error>,
}
#[cfg(test)]
static CLOSE_PIPE_FAULT: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
static IO_START_FAULT: AtomicBool = AtomicBool::new(false);
fn close_file(file: &mut Option<std::fs::File>) -> io::Result<()> {
    #[cfg(test)]
    if file.is_some() && CLOSE_PIPE_FAULT.load(Ordering::Acquire) {
        return Err(io::Error::other("injected syntax pipe close"));
    }
    #[cfg(windows)]
    {
        use std::os::windows::io::{FromRawHandle, IntoRawHandle};
        if let Some(value) = file.take() {
            let raw = value.into_raw_handle();
            if unsafe { windows_sys::Win32::Foundation::CloseHandle(raw) } == 0 {
                *file = Some(unsafe { std::fs::File::from_raw_handle(raw) });
                return Err(io::Error::last_os_error());
            }
        }
    }
    #[cfg(not(windows))]
    {
        file.take();
    }
    Ok(())
}
struct Connection {
    owner_shared: Arc<Shared>,
    child: Child,
    job: Option<ChildJob>,
    send: Option<SyncSender<Outgoing>>,
    receive: Receiver<io::Result<syntax_worker::FrameReceipt>>,
    io: Option<JoinHandle<IoReceipt>>,
    mailbox: Arc<Mutex<Option<IoFiles>>>,
    receipt: Option<IoReceipt>,
    input: Arc<RetainedSyntaxInput>,
    _metadata: Option<crate::document_consumer_budget::Charge>,
    status: Option<std::process::ExitStatus>,
    sticky: Option<io::Error>,
    closed: bool,
    io_started: bool,
    cleanup_deadline: Option<Instant>,
    pending_next: Option<Box<Connection>>,
    _spawn_lease: SpawnLease,
}
static SYNTAX_ACTIVE: AtomicBool = AtomicBool::new(false);
static PENDING_PRESENT: AtomicBool = AtomicBool::new(false);
struct SpawnLease;
impl Drop for SpawnLease {
    fn drop(&mut self) {
        SYNTAX_ACTIVE.store(false, Ordering::Release);
    }
}
static PENDING: std::sync::OnceLock<Mutex<Option<Connection>>> = std::sync::OnceLock::new();
fn pending_slot() -> &'static Mutex<Option<Connection>> {
    PENDING.get_or_init(|| Mutex::new(None))
}
impl Connection {
    fn spawn(
        doc: &Document,
        shared: &Arc<Shared>,
    ) -> Result<Self, (FallbackReason, Option<Box<Self>>)> {
        #[cfg(test)]
        let spawn_started = Instant::now();
        if pending_slot()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_some()
        {
            return Err((FallbackReason::ContainmentFailed, None));
        }
        doc.context.check(Instant::now()).map_err(|e| (e, None))?;
        // Conservative host-owned control/mailbox allocation margin. No claim about OS thread stacks.
        let metadata = doc
            .source
            .budget()
            .map(|b| {
                b.reserve(
                    size_of::<Self>()
                        + size_of::<Shared>()
                        + 2 * size_of::<usize>()
                        + size_of::<Outgoing>()
                        + 4096,
                )
                .map_err(|_| (FallbackReason::SourceBudget, None))
            })
            .transpose()?;
        let executable =
            std::env::current_exe().map_err(|_| (FallbackReason::WorkerExited, None))?;
        #[cfg(test)]
        let executable = std::env::var_os("R3VIEWER_SYNTAX_TEST_EXE")
            .map(std::path::PathBuf::from)
            .unwrap_or(executable);
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
        let (send, incoming) = mpsc::sync_channel::<Outgoing>(1);
        let (finished, receive) = mpsc::sync_channel(1);
        if SYNTAX_ACTIVE
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Err((FallbackReason::ContainmentFailed, None));
        }
        let spawn_lease = SpawnLease;
        let child = command
            .spawn()
            .map_err(|_| (FallbackReason::WorkerExited, None))?;
        #[cfg(test)]
        eprintln!(
            "OWN-SPAWN CreateProcess-return pid={} elapsed={:?}",
            child.id(),
            spawn_started.elapsed()
        );
        let mut connection = Self {
            owner_shared: shared.clone(),
            child,
            job: None,
            send: Some(send),
            receive,
            io: None,
            mailbox: Arc::new(Mutex::new(None)),
            receipt: None,
            input: doc.source.clone(),
            _metadata: metadata,
            status: None,
            sticky: None,
            closed: false,
            io_started: false,
            cleanup_deadline: None,
            pending_next: None,
            _spawn_lease: spawn_lease,
        };
        #[cfg(windows)]
        {
            use std::os::windows::io::{FromRawHandle, IntoRawHandle};
            let input = connection
                .child
                .stdin
                .take()
                .map(|v| unsafe { std::fs::File::from_raw_handle(v.into_raw_handle()) });
            let output = connection
                .child
                .stdout
                .take()
                .map(|v| unsafe { std::fs::File::from_raw_handle(v.into_raw_handle()) });
            *connection.mailbox.lock().unwrap_or_else(|e| e.into_inner()) =
                Some(IoFiles { input, output });
        }
        #[cfg(not(windows))]
        {
            return Err((
                FallbackReason::UnsupportedPlatform,
                Some(Box::new(connection)),
            ));
        }
        match ChildJob::attach_verified(&connection.child) {
            Ok(job) => connection.job = Some(job),
            Err(failure) => {
                connection.job = failure.job;
                connection.sticky = Some(failure.error);
                return Err((
                    FallbackReason::ContainmentFailed,
                    Some(Box::new(connection)),
                ));
            }
        }
        #[cfg(test)]
        if IO_START_FAULT.load(Ordering::Acquire) {
            connection.sticky = Some(io::Error::other("injected syntax IPC thread setup"));
            return Err((FallbackReason::WorkerExited, Some(Box::new(connection))));
        }
        #[cfg(test)]
        eprintln!(
            "OWN-SPAWN attach-verified pid={} elapsed={:?}",
            connection.child.id(),
            spawn_started.elapsed()
        );
        let mailbox = connection.mailbox.clone();
        let input = connection.input.clone();
        let state = shared.clone();
        let revision = doc.revision;
        let cap = doc.limits.spans;
        let boot_context = doc.context.clone();
        let boot_deadline = doc.context.phase(Instant::now(), doc.limits.startup);
        // The mailbox, not a FnOnce capture, owns pipes on failed thread startup.
        let started = thread::Builder::new()
            .name("syntax-ipc".into())
            .spawn(move || {
                let mut files = mailbox
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .take()
                    .unwrap_or(IoFiles {
                        input: None,
                        output: None,
                    });
                let result =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> io::Result<()> {
                        let Some(output) = files.output.as_mut() else {
                            return Err(io::Error::other("missing syntax output"));
                        };
                        // Each individual Write checks host authority, not merely enqueue time.
                        struct Checked<'a> {
                            inner: &'a mut std::fs::File,
                            context: &'a OperationContext,
                            deadline: Instant,
                            state: &'a Shared,
                            revision: u64,
                        }
                        impl io::Write for Checked<'_> {
                            fn write(&mut self, b: &[u8]) -> io::Result<usize> {
                                self.context
                                    .check(Instant::now())
                                    .map_err(io::Error::other)?;
                                if self.state.status(self.revision).is_err()
                                    || Instant::now() >= self.deadline
                                {
                                    return Err(io::Error::new(
                                        io::ErrorKind::TimedOut,
                                        "syntax write expired",
                                    ));
                                }
                                io::Write::write(self.inner, &b[..b.len().min(4096)])
                            }
                            fn flush(&mut self) -> io::Result<()> {
                                self.context
                                    .check(Instant::now())
                                    .map_err(io::Error::other)?;
                                if Instant::now() >= self.deadline {
                                    return Err(io::Error::new(
                                        io::ErrorKind::TimedOut,
                                        "syntax flush expired",
                                    ));
                                }
                                io::Write::flush(self.inner)
                            }
                        }
                        impl io::Read for Checked<'_> {
                            fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
                                self.context
                                    .check(Instant::now())
                                    .map_err(io::Error::other)?;
                                if self.state.status(self.revision).is_err()
                                    || Instant::now() >= self.deadline
                                {
                                    return Err(io::Error::new(
                                        io::ErrorKind::TimedOut,
                                        "syntax read expired",
                                    ));
                                }
                                let size = b.len().min(4096);
                                let n = io::Read::read(self.inner, &mut b[..size])?;
                                self.context
                                    .check(Instant::now())
                                    .map_err(io::Error::other)?;
                                if Instant::now() >= self.deadline {
                                    return Err(io::Error::new(
                                        io::ErrorKind::TimedOut,
                                        "syntax read completed late",
                                    ));
                                }
                                Ok(n)
                            }
                        }

                        let boot = syntax_worker::read_host_frame(
                            &mut Checked {
                                inner: output,
                                context: &boot_context,
                                deadline: boot_deadline,
                                state: &state,
                                revision,
                            },
                            &input,
                            cap,
                        );
                        let bad = boot.is_err();
                        if finished.try_send(boot).is_err() || bad {
                            return Ok(());
                        }
                        while let Ok(packet) = incoming.recv() {
                            if state.status(revision).is_err() || packet.revision != revision {
                                break;
                            }
                            packet
                                .context
                                .check(Instant::now())
                                .map_err(io::Error::other)?;
                            if Instant::now() >= packet.deadline {
                                return Err(io::Error::new(
                                    io::ErrorKind::TimedOut,
                                    "syntax queued operation expired",
                                ));
                            }
                            let Some(writer) = files.input.as_mut() else {
                                return Err(io::Error::other("missing syntax input"));
                            };
                            let result = syntax_worker::write_frame(
                                &mut Checked {
                                    inner: writer,
                                    context: &packet.context,
                                    deadline: packet.deadline,
                                    state: &state,
                                    revision,
                                },
                                &packet.frame,
                            )
                            .and_then(|()| {
                                syntax_worker::read_host_frame(
                                    &mut Checked {
                                        inner: output,
                                        context: &packet.context,
                                        deadline: packet.deadline,
                                        state: &state,
                                        revision,
                                    },
                                    &input,
                                    cap,
                                )
                            });
                            let bad = result.is_err();
                            if finished.try_send(result).is_err() || bad {
                                break;
                            }
                        }
                        Ok(())
                    }));
                let error = match result {
                    Ok(Ok(())) => None,
                    Ok(Err(e)) => Some(e),
                    Err(_) => Some(io::Error::other("syntax IPC panic")),
                };
                // Returning Files retains failed-close authority until supervisor joins and checks closes.
                IoReceipt { files, error }
            });
        match started {
            Ok(io) => {
                connection.io = Some(io);
                connection.io_started = true;
            }
            Err(e) => {
                connection.sticky = Some(e);
                return Err((FallbackReason::WorkerExited, Some(Box::new(connection))));
            }
        }
        Ok(connection)
    }
    fn wait(
        &self,
        shared: &Shared,
        revision: u64,
        per_line: Instant,
        total: Instant,
        timeout: FallbackReason,
    ) -> Result<syntax_worker::FrameReceipt, WaitError> {
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
                Ok(Err(e)) => {
                    return Err(WaitError::Failure(
                        if e.kind() == io::ErrorKind::InvalidData {
                            FallbackReason::InvalidIpc
                        } else if e.kind() == io::ErrorKind::OutOfMemory {
                            FallbackReason::SpanBudget
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
    ) -> Result<syntax_worker::FrameReceipt, WaitError> {
        shared.status(revision)?;
        if Instant::now() >= total {
            return Err(WaitError::Failure(timeout));
        }
        if Instant::now() >= per_line {
            return Err(WaitError::Failure(FallbackReason::LineTimeout));
        }
        let context = shared.lock().context.clone();
        context.check(Instant::now()).map_err(WaitError::Failure)?;
        self.send
            .as_ref()
            .ok_or(WaitError::Failure(FallbackReason::WorkerExited))?
            .try_send(Outgoing {
                frame,
                deadline: total.min(per_line),
                context,
                revision,
            })
            .map_err(|_| WaitError::Failure(FallbackReason::InvalidIpc))?;
        self.wait(shared, revision, per_line, total, timeout)
    }
    fn check_alive(&mut self) -> Result<(), WaitError> {
        match self.child.try_wait() {
            Ok(None) => Ok(()),
            _ => Err(WaitError::Failure(FallbackReason::WorkerExited)),
        }
    }
    fn mark(&mut self, result: io::Result<()>) {
        if let Err(e) = result {
            self.sticky.get_or_insert(e);
        }
    }
    /// Physical proof and sticky errors are separate. Retain all authority if proof is missing.
    fn close(&mut self) -> bool {
        if self.closed {
            return true;
        }
        let deadline = *self
            .cleanup_deadline
            .get_or_insert_with(|| Instant::now() + Duration::from_secs(2));
        self.send.take();
        if self.status.is_none() {
            let r = self.job.as_ref().map_or(Ok(()), |j| j.terminate(75));
            self.mark(r);
            let r = self.child.kill();
            self.mark(r);
        }
        loop {
            if self.status.is_none() {
                match self.child.try_wait() {
                    Ok(Some(status)) => match self.child.wait() {
                        Ok(again) if again == status => self.status = Some(status),
                        Ok(_) => self.mark(Err(io::Error::other("syntax inconsistent wait"))),
                        Err(e) => self.mark(Err(e)),
                    },
                    Ok(None) => {}
                    Err(e) => self.mark(Err(e)),
                }
            }
            if self.io.as_ref().is_some_and(JoinHandle::is_finished)
                && let Some(handle) = self.io.take()
            {
                match handle.join() {
                    Ok(receipt) => {
                        if let Some(e) = receipt.error.as_ref() {
                            self.sticky
                                .get_or_insert_with(|| io::Error::new(e.kind(), e.to_string()));
                        }
                        self.receipt = Some(receipt);
                    }
                    Err(_) => self.mark(Err(io::Error::other("syntax IPC panic without receipt"))),
                }
            }
            let mut streams = true;
            if self.io.is_none() {
                let mailbox = self.mailbox.clone();
                let mut guard = mailbox.lock().unwrap_or_else(|e| e.into_inner());
                if let Some(files) = guard.as_mut() {
                    let r = close_file(&mut files.input);
                    self.mark(r);
                    let r = close_file(&mut files.output);
                    self.mark(r);
                    streams &= files.input.is_none() && files.output.is_none();
                    if streams {
                        guard.take();
                    }
                }
                if let Some(receipt) = self.receipt.as_mut() {
                    let a = close_file(&mut receipt.files.input);
                    let b = close_file(&mut receipt.files.output);
                    streams &= receipt.files.input.is_none() && receipt.files.output.is_none();
                    self.mark(a);
                    self.mark(b);
                }
            } else {
                streams = false;
            }
            // A join-panic without the receipt cannot claim the thread-owned files closed.
            streams &= self.receipt.is_some()
                || !self.io_started
                    && self
                        .mailbox
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .is_none()
                    && self.io.is_none();
            let active = match self.job.as_ref() {
                None => true,
                Some(job) => match job.active_processes() {
                    Ok(0) => true,
                    Ok(_) => {
                        let r = job.terminate(75);
                        self.mark(r);
                        false
                    }
                    Err(e) => {
                        self.mark(Err(e));
                        false
                    }
                },
            };
            if self.status.is_some() && self.io.is_none() && streams && active {
                if let Some(job) = self.job.take() {
                    match job.close_checked() {
                        Ok(()) => {}
                        Err(failure) => {
                            self.job = Some(failure.job);
                            self.mark(Err(failure.error));
                        }
                    }
                }
                if self.job.is_none() {
                    while self.receive.try_recv().is_ok() {}
                    self.closed = true;
                    return true;
                }
            }
            if Instant::now() >= deadline {
                return false;
            }
            thread::sleep(Duration::from_millis(2));
        }
    }
}

fn check_frame(
    frame: syntax_worker::FrameReceipt,
    generation: u64,
    request: u64,
) -> Result<Message, WaitError> {
    let frame = frame.frame;
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
        if index.is_multiple_of(4096) {
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
    #[cfg(test)]
    let phase_started = Instant::now();
    #[cfg(test)]
    eprintln!(
        "OWN-PHASE initialize-enter gen={} startup_remaining={:?}",
        doc.generation,
        deadline.saturating_duration_since(Instant::now())
    );
    let boot = connection.wait(
        shared,
        doc.revision,
        deadline,
        deadline,
        FallbackReason::StartupTimeout,
    )?;
    if boot.frame.generation != 0
        || boot.frame.request != 0
        || !matches!(boot.frame.message, Message::Boot)
    {
        return Err(WaitError::Failure(FallbackReason::InvalidIpc));
    }
    #[cfg(test)]
    eprintln!("OWN-PHASE Boot received {:?}", phase_started.elapsed());
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
    #[cfg(test)]
    eprintln!("OWN-PHASE Init/Ack received {:?}", phase_started.elapsed());
    let mut offset = 0;
    for bytes in doc.source.as_bytes().chunks(4096) {
        let next = offset + bytes.len();
        let response = connection.exchange(
            Frame::new(
                doc.generation,
                0,
                Message::ChunkSlice {
                    offset: offset as u32,
                    input: doc.source.clone(),
                    range: offset..next,
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
    #[cfg(test)]
    eprintln!(
        "OWN-PHASE all SourceChunks/Ack received {:?}",
        phase_started.elapsed()
    );
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
        if let Some(range) = doc.source.viewport_range(view.first_line, view.line_count) {
            range
        } else {
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
        }
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
) -> Result<(Request, Arc<RetainedRoleSet>), WaitError> {
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
    let mut total = doc.context.phase(Instant::now(), duration(request));
    let message = request.viewport.map_or(Message::All, Message::View);
    let mut phase_deadline = doc
        .context
        .phase(Instant::now(), limits.per_line)
        .min(total);
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
        let allocation = response.take_roles();
        match check_frame(response, doc.generation, request.id)? {
            Message::Styles(marker) => {
                if !marker.is_empty() {
                    return Err(WaitError::Failure(FallbackReason::InvalidIpc));
                }
                let allocation =
                    allocation.ok_or(WaitError::Failure(FallbackReason::InvalidIpc))?;
                allocation
                    .verify_owner(&doc.source, doc.generation, request.id)
                    .map_err(|_| WaitError::Failure(FallbackReason::InvalidIpc))?;
                validate_spans(allocation.spans(), doc, request, limits)?;
                if Instant::now() >= total {
                    return Err(WaitError::Failure(FallbackReason::RequestTimeout));
                }
                if Instant::now() >= phase_deadline {
                    return Err(WaitError::Failure(FallbackReason::LineTimeout));
                }
                let roles = RetainedRoleSet::from_receipt(
                    allocation,
                    &doc.source,
                    doc.generation,
                    request.id,
                )
                .map_err(WaitError::Failure)?;
                return Ok((request, roles));
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
                    total = doc.context.phase(Instant::now(), duration(request));
                    progress = None;
                    Message::Supersede(view)
                } else {
                    Message::Continue
                };
                phase_deadline = doc
                    .context
                    .phase(Instant::now(), limits.per_line)
                    .min(total);
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
    roles: Arc<RetainedRoleSet>,
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
                .map(|e| e.roles.retained_bytes())
                .sum::<usize>()
    }
    fn clear(&mut self) {
        self.entries = VecDeque::new();
    }
    fn get(&mut self, viewport: Viewport) -> Option<Arc<RetainedRoleSet>> {
        let at = self.entries.iter().position(|e| e.viewport == viewport)?;
        let e = self.entries.remove(at)?;
        let roles = e.roles.clone();
        self.entries.push_back(e);
        Some(roles)
    }
    fn insert_retained(&mut self, viewport: Viewport, roles: Arc<RetainedRoleSet>) {
        // Document cache is introduced with D4b snapshot profile. Until then it is disabled,
        // avoiding an uncharged VecDeque in this D4a host API.
        if roles.input().budget().is_some() {
            return;
        }
        let cost = size_of::<Cached>() + roles.retained_bytes();
        if roles.spans().is_empty() || cost > self.budget {
            return;
        }
        if let Some(at) = self.entries.iter().position(|e| e.viewport == viewport) {
            self.entries.remove(at);
        }
        self.entries.push_back(Cached { viewport, roles });
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
    #[cfg(test)]
    fn insert(&mut self, viewport: Viewport, spans: &[RoleSpan]) {
        let input = RetainedSyntaxInput::legacy(Arc::new(String::new()));
        let roles = RetainedRoleSet::legacy(spans.to_vec(), input).unwrap();
        self.insert_retained(viewport, roles);
    }
}
fn close_connection(connection: &mut Option<Connection>, shared: &Shared) -> bool {
    if let Some(owner) = connection.as_mut() {
        if !owner.close() {
            shared.lock().cleanup = CleanupStatus::Pending;
            return false;
        }
        let failed = owner.sticky.is_some();
        shared.lock().cleanup = CleanupStatus::Reaped { failed };
        connection.take();
    }
    true
}
/// Owns the same controller thread while a failed cleanup is parked. A retry only
/// closes retained physical authority; it never republishes the failed request.
fn retry_pending_connection(caller: &Arc<Shared>) {
    let mut slot = pending_slot().lock().unwrap_or_else(|e| e.into_inner());
    let Some(owner) = slot.as_mut() else {
        return;
    };
    // A live owner's retry may never be stolen by another Shared/controller.
    // Orphan backstop is authorized only by the actual owner's terminal stop intent.
    if !Arc::ptr_eq(caller, &owner.owner_shared) && !owner.owner_shared.lock().stopping {
        return;
    }
    owner.cleanup_deadline = None;
    if !owner.close() {
        return;
    }
    let failed = owner.sticky.is_some();
    let original = owner.owner_shared.clone();
    let next = owner.pending_next.take().map(|b| *b);
    *slot = next;
    PENDING_PRESENT.store(slot.is_some(), Ordering::Release);
    let mut state = original.lock();
    state.cleanup = if slot.is_some() {
        CleanupStatus::Pending
    } else {
        CleanupStatus::Reaped { failed }
    };
    if slot.is_none() {
        state.memory = Default::default();
    }
}
fn controller(shared: Arc<Shared>, limits: SyntaxLimits) {
    let mut connection = None;
    loop {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            controller_run(shared.clone(), limits.clone(), &mut connection)
        }));
        if result.is_err() {
            let generation = shared.lock().generation;
            shared.latch(generation, FallbackReason::WorkerExited);
        }
        if !close_connection(&mut connection, &shared)
            && let Some(owner) = connection.take()
        {
            let mut slot = pending_slot().lock().unwrap_or_else(|e| e.into_inner());
            let mut owner = owner;
            owner.pending_next = slot.take().map(Box::new);
            *slot = Some(owner);
            PENDING_PRESENT.store(true, Ordering::Release);
        }
        if shared.lock().stopping || shared.cancelled.load(Ordering::Acquire) {
            return;
        }
        // Failure ended this run but not the session-owned thread. The failed Set/request
        // were discarded by controller_run, the Plain latch remains sticky for generation.
        // No Set/Init is allowed while Pending; only a new explicit generation after reap.
        while PENDING_PRESENT.load(Ordering::Acquire) {
            let mut state = shared.lock();
            if state.stopping || shared.cancelled.load(Ordering::Acquire) {
                return;
            }
            if std::mem::take(&mut state.retry_cleanup) {
                drop(state);
                retry_pending_connection(&shared);
            } else {
                let _ = shared
                    .wake
                    .wait_timeout(state, Duration::from_millis(25))
                    .unwrap_or_else(|e| e.into_inner());
            }
        }
    }
}
fn controller_run(shared: Arc<Shared>, limits: SyntaxLimits, connection: &mut Option<Connection>) {
    let mut cleanup_failed = false;
    let mut document: Option<Document> = None;
    let mut cache = StyleCache::new(limits.style_cache_bytes);
    loop {
        let retry = {
            let mut state = shared.lock();
            std::mem::take(&mut state.retry_cleanup)
        };
        if retry {
            retry_pending_connection(&shared);
        }
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
            if !close_connection(connection, &shared) {
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
                let deadline = doc.context.phase(Instant::now(), doc.limits.startup);
                let initialized = (|| {
                    #[cfg(test)]
                    let preflight_started = Instant::now();
                    preflight(&doc, &doc.limits, &shared)?;
                    #[cfg(test)]
                    eprintln!(
                        "OWN-PREFLIGHT gen={} bytes={} elapsed={:?}",
                        doc.generation,
                        doc.source.len(),
                        preflight_started.elapsed()
                    );
                    if !crate::syntax_process::supported() {
                        return Err(WaitError::Failure(FallbackReason::UnsupportedPlatform));
                    }
                    match Connection::spawn(&doc, &shared) {
                        Ok(owner) => *connection = Some(owner),
                        Err((reason, owner)) => {
                            *connection = owner.map(|b| *b);
                            return Err(WaitError::Failure(reason));
                        }
                    }
                    shared.lock().memory.child_source_bytes = doc.source.len();
                    initialize(
                        connection.as_ref().unwrap(),
                        &doc,
                        &doc.limits,
                        &shared,
                        deadline,
                    )
                })();
                match initialized {
                    Ok(()) => {
                        shared.lock().memory.child_source_bytes = doc.source.len();
                        shared.publish_for(
                            &doc,
                            RetainedSyntaxEvent::Ready {
                                generation: doc.generation,
                            },
                        );
                        document = Some(doc);
                    }
                    Err(WaitError::Failure(reason)) => {
                        shared.latch(doc.generation, reason);
                        if !close_connection(connection, &shared) {
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
                        if !close_connection(connection, &shared) {
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
        if let Some(doc) = document.as_ref() {
            if let Err(reason) = doc.context.check(Instant::now()) {
                shared.latch(doc.generation, reason);
            }
            if shared.lock().plain.is_some() {
                if !close_connection(connection, &shared) {
                    cleanup_failed = true;
                    break;
                }
                document = None;
                cache.clear();
                let mut state = shared.lock();
                state.memory.controller_source_bytes = 0;
                state.memory.child_source_bytes = 0;
                state.memory.style_cache_bytes = 0;
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
            if !close_connection(connection, &shared) {
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
                shared.publish_for(
                    doc,
                    RetainedSyntaxEvent::Styles {
                        generation: doc.generation,
                        request: request.id,
                        viewport: request.viewport,
                        roles: spans,
                    },
                );
                continue;
            }
            let result =
                run_request(worker, doc, request, &doc.limits, &shared).and_then(|result| {
                    worker.check_alive()?;
                    Ok(result)
                });
            match result {
                Ok((request, spans)) => {
                    if let Some(viewport) = request.viewport {
                        cache.insert_retained(viewport, spans.clone());
                        shared.lock().memory.style_cache_bytes = cache.bytes();
                    }
                    shared.publish_for(
                        doc,
                        RetainedSyntaxEvent::Styles {
                            generation: doc.generation,
                            request: request.id,
                            viewport: request.viewport,
                            roles: spans,
                        },
                    );
                }
                Err(WaitError::Failure(reason)) => {
                    shared.latch(doc.generation, reason);
                    cache.clear();
                    if !close_connection(connection, &shared) {
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
    let closed = close_connection(connection, &shared);
    let reaped = closed && !cleanup_failed;
    let mut state = shared.lock();
    if let Some((_, ticket)) = state.pause.take() {
        ticket.complete(reaped);
    }
    // Preserve explicit exit intent; cleanup failure itself parks this same controller.
    if !cleanup_failed {
        state.stopping = true;
    }
    state.has_document = false;
    if reaped {
        state.memory = Default::default();
    }
    state.update = None;
    state.request = None;
}
#[cfg(test)]
pub(crate) use retained::fixture_document;
#[cfg(test)]
mod tests {
    use super::*;
    // No controller is spawned: barriers force the exact wait-status/Progress-take
    // interleaving against the same public UI methods and production transaction.
    pub(super) fn dormant_client() -> SyntaxClient {
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
                    context: OperationContext::LegacySource,
                    cleanup: CleanupStatus::Reaped { failed: false },
                    retry_cleanup: false,
                    plain: None,
                    stopping: false,
                    pause: None,
                    memory: Default::default(),
                    limits: SyntaxLimits::default(),
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
        client.shared.publish(RetainedSyntaxEvent::Styles {
            generation: 10,
            request: 2,
            viewport: None,
            roles: RetainedRoleSet::legacy(
                vec![RoleSpan {
                    start: 0,
                    end: 1,
                    role: SyntaxRole::Accent,
                }],
                RetainedSyntaxInput::legacy(Arc::new("x".into())),
            )
            .unwrap(),
        });
        // Failure from request1 still invalidates the generation, including request2's result.
        client.shared.latch(10, FallbackReason::LineTimeout);
        client
            .shared
            .publish(RetainedSyntaxEvent::Ready { generation: 10 });
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
        client
            .shared
            .publish(RetainedSyntaxEvent::Ready { generation: 11 });
        assert!(matches!(
            client.poll(),
            Some(SyntaxEvent::Ready { generation: 11 })
        ));
        client.shutdown().join().unwrap();
    }
    #[test]
    fn spans_reject_utf8_midpoints_order_and_incomplete_inline() {
        let doc = Document {
            _metadata: None,
            source: RetainedSyntaxInput::legacy(Arc::new("é界".into())),
            context: OperationContext::LegacySource,
            limits: SyntaxLimits::default(),
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
    #[test]
    fn late_poll_discards_color_but_does_not_mutate_original_and_context_is_atomic() {
        let mut client = dormant_client();
        let doc = retained::fixture_document();
        let budget = crate::document_consumer_budget::ConsumerBudget::new(&doc).unwrap();
        let now = Instant::now();
        let cancel = Arc::new(AtomicBool::new(false));
        let context = OperationContext::document(9, now + Duration::from_secs(5), cancel.clone());
        let input = RetainedSyntaxInput::copy_document("é\n", &doc, &budget, &context).unwrap();
        client
            .set_retained_document(
                input.clone(),
                "Rust".into(),
                7,
                SyntaxLimits::default(),
                context.clone(),
            )
            .unwrap();
        let request = client.request_retained(7, None, &context);
        assert_ne!(request, 0);
        let wrong = OperationContext::document(10, now + Duration::from_secs(5), cancel);
        assert_eq!(client.request_retained(7, None, &wrong), 0);
        let roles = RetainedRoleSet::empty_for_test(input);
        client.shared.publish_at(
            RetainedSyntaxEvent::Styles {
                generation: 7,
                request,
                viewport: None,
                roles,
            },
            now + Duration::from_secs(4),
        );
        assert!(client.shared.lock().event.is_some());
        assert!(matches!(
            client.poll_retained_at(now + Duration::from_secs(5)),
            Some(RetainedSyntaxEvent::Plain {
                reason: FallbackReason::RequestTimeout,
                ..
            })
        ));
        assert_eq!(doc.source().text(), "text\n");
        assert!(client.shared.lock().event.is_none());
    }
    #[test]
    fn publication_rejects_replacement_revision_before_new_request_and_late_global_cancel() {
        let mut client = dormant_client();
        client.set_document(Arc::new("old\n".into()), "Rust".into(), 1);
        let Some(Update::Set(old)) = client.shared.lock().update.take() else {
            panic!("doc")
        };
        let request = client.request(1, None);
        client.set_document(Arc::new("new\n".into()), "Rust".into(), 1);
        client
            .shared
            .publish_for(&old, RetainedSyntaxEvent::Ready { generation: 1 });
        assert!(client.shared.lock().event.is_none());
        let roles = RetainedRoleSet::empty_for_test(old.source.clone());
        client.shared.publish_for(
            &old,
            RetainedSyntaxEvent::Styles {
                generation: 1,
                request,
                viewport: None,
                roles,
            },
        );
        assert!(client.shared.lock().event.is_none());
        let Some(Update::Set(current)) = client.shared.lock().update.take() else {
            panic!("doc")
        };
        client
            .shared
            .publish_for(&current, RetainedSyntaxEvent::Ready { generation: 1 });
        assert!(client.shared.lock().event.is_some());
        client.shared.cancelled.store(true, Ordering::Release);
        assert!(matches!(
            client.poll_retained(),
            Some(RetainedSyntaxEvent::Plain {
                reason: FallbackReason::Cancelled,
                ..
            })
        ));
        assert!(client.shared.lock().event.is_none());
    }
    #[cfg(windows)]
    #[test]
    #[ignore = "requires explicit previously built native worker path"]
    fn native_source_and_retained_requests_share_one_owner_and_reap() {
        let path = std::env::var_os("R3VIEWER_SYNTAX_TEST_EXE")
            .expect("native syntax test requires previously built worker path");
        assert!(std::path::Path::new(&path).is_file());
        let mut client =
            SyntaxClient::new(SyntaxLimits::default(), Arc::new(AtomicBool::new(false))).unwrap();
        let text = Arc::new("let x = \"é\";\n".to_owned());
        client.set_document(text.clone(), "Rust".into(), 1);
        let request = client.request(1, None);
        let until = Instant::now() + Duration::from_secs(8);
        let mut found = false;
        let source_started = Instant::now();
        let mut source_failure = None;
        while Instant::now() < until {
            match client.poll() {
                Some(SyntaxEvent::Styles {
                    request: id, spans, ..
                }) => {
                    assert_eq!(id, request);
                    assert_eq!(spans.first().unwrap().start, 0);
                    assert_eq!(spans.last().unwrap().end as usize, text.len());
                    found = true;
                    break;
                }
                Some(SyntaxEvent::Plain { reason, .. }) => {
                    source_failure = Some(reason);
                    break;
                }
                _ => thread::sleep(Duration::from_millis(2)),
            }
        }
        eprintln!(
            "OWN-SOURCE elapsed={:?} found={} reason={:?}",
            source_started.elapsed(),
            found,
            source_failure
        );
        if !found {
            let shared = client.shared.clone();
            let joined = client.shutdown().join().is_ok();
            let state = shared.lock().cleanup;
            eprintln!(
                "OWN-SOURCE cleanup joined={} state={:?} active={} pending={}",
                joined,
                state,
                SYNTAX_ACTIVE.load(Ordering::Acquire),
                PENDING_PRESENT.load(Ordering::Acquire)
            );
            assert!(found, "legacy source {:?}", source_failure);
            return;
        }
        let pause = client.pause_document(1);
        while pause.state() == crate::worker_process::ReapState::Pending && Instant::now() < until {
            thread::sleep(Duration::from_millis(2));
        }
        assert!(matches!(
            client.cleanup_status(),
            CleanupStatus::Reaped { .. }
        ));
        let doc = retained::fixture_document();
        let budget = crate::document_consumer_budget::ConsumerBudget::new(&doc).unwrap();
        let context = OperationContext::document(
            2,
            Instant::now() + Duration::from_secs(5),
            Arc::new(AtomicBool::new(false)),
        );
        let input = RetainedSyntaxInput::copy_document("# Á\n", &doc, &budget, &context).unwrap();
        let weak = Arc::downgrade(&input);
        client
            .set_retained_document(
                input.clone(),
                "Nushell".into(),
                2,
                SyntaxLimits::default(),
                context.clone(),
            )
            .unwrap();
        drop(input);
        let request = client.request_retained(2, None, &context);
        let until = Instant::now() + Duration::from_secs(6);
        let mut output = None;
        let mut retained_failure = None;
        while Instant::now() < until {
            match client.poll_retained() {
                Some(RetainedSyntaxEvent::Styles {
                    request: id, roles, ..
                }) => {
                    assert_eq!(id, request);
                    output = Some(roles);
                    break;
                }
                Some(RetainedSyntaxEvent::Plain { reason, .. }) => {
                    retained_failure = Some(reason);
                    break;
                }
                _ => thread::sleep(Duration::from_millis(2)),
            }
        }
        let shared = client.shared.clone();
        let joined = client.shutdown().join().is_ok();
        eprintln!(
            "OWN-RETAINED cleanup joined={} state={:?} active={} pending={} reason={:?}",
            joined,
            shared.lock().cleanup,
            SYNTAX_ACTIVE.load(Ordering::Acquire),
            PENDING_PRESENT.load(Ordering::Acquire),
            retained_failure
        );
        assert!(joined);
        let roles = output.expect("retained response after explicit cleanup");
        assert!(
            roles
                .spans()
                .iter()
                .filter(|s| s.start < 4)
                .all(|s| s.role == SyntaxRole::Secondary)
        );
        assert!(weak.upgrade().is_some());
        drop(roles);
        assert!(weak.upgrade().is_none());
        assert!(!SYNTAX_ACTIVE.load(Ordering::Acquire));
        assert!(!PENDING_PRESENT.load(Ordering::Acquire));
    }

    #[cfg(windows)]
    #[test]
    #[ignore = "requires explicit previously built native worker path"]
    fn native_setup_failure_keeps_child_job_and_both_pipes_until_cleanup() {
        assert!(std::env::var_os("R3VIEWER_SYNTAX_TEST_EXE").is_some());
        let client = dormant_client();
        client.set_document(Arc::new("x\n".into()), "Rust".into(), 1);
        let Some(Update::Set(doc)) = client.shared.lock().update.take() else {
            panic!("document")
        };
        IO_START_FAULT.store(true, Ordering::Release);
        let result = Connection::spawn(&doc, &client.shared);
        IO_START_FAULT.store(false, Ordering::Release);
        let Err((reason, Some(mut owner))) = result else {
            panic!("injected startup must retain ownership")
        };
        let live = owner.job.as_ref().unwrap().active_processes().unwrap() == 1
            && owner.child.try_wait().unwrap().is_none();
        let pipes = owner
            .mailbox
            .lock()
            .unwrap()
            .as_ref()
            .is_some_and(|f| f.input.is_some() && f.output.is_some());
        let closed = owner.close();
        let waited = owner.status.is_some();
        let sticky = owner.sticky.is_some();
        if !closed {
            let mut slot = pending_slot().lock().unwrap();
            owner.pending_next = slot.take().map(Box::new);
            *slot = Some(*owner);
            PENDING_PRESENT.store(true, Ordering::Release);
        } else {
            drop(owner);
        }
        assert_eq!(reason, FallbackReason::WorkerExited);
        assert!(live && pipes && closed && waited && sticky);
        assert!(!SYNTAX_ACTIVE.load(Ordering::Acquire));
    }
    #[cfg(windows)]
    #[test]
    #[ignore = "requires explicit previously built native worker path"]
    fn native_active_cancel_failed_close_pending_blocks_spawn_and_explicit_retry_is_sticky() {
        assert!(std::env::var_os("R3VIEWER_SYNTAX_TEST_EXE").is_some());
        let client = dormant_client();
        let document = retained::fixture_document();
        let budget = crate::document_consumer_budget::ConsumerBudget::new(&document).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let context =
            OperationContext::document(71, Instant::now() + Duration::from_secs(8), cancel.clone());
        let input =
            RetainedSyntaxInput::copy_document("# retained\n", &document, &budget, &context)
                .unwrap();
        let weak = Arc::downgrade(&input);
        client
            .set_retained_document(input, "Nushell".into(), 7, SyntaxLimits::default(), context)
            .unwrap();
        let Some(Update::Set(doc)) = client.shared.lock().update.take() else {
            panic!("document")
        };
        let mut owner = match Connection::spawn(&doc, &client.shared) {
            Ok(v) => v,
            Err((e, mut owned)) => {
                if let Some(o) = owned.as_mut() {
                    o.close();
                }
                panic!("spawn {e}")
            }
        };
        let live = owner.job.as_ref().unwrap().active_processes().unwrap() == 1
            && owner.child.try_wait().unwrap().is_none();
        cancel.store(true, Ordering::Release);
        let deadline = Instant::now() + Duration::from_secs(2);
        let cancelled = matches!(
            owner.wait(
                &client.shared,
                doc.revision,
                deadline,
                deadline,
                FallbackReason::RequestTimeout
            ),
            Err(WaitError::Failure(FallbackReason::Cancelled))
        );
        CLOSE_PIPE_FAULT.store(true, Ordering::Release);
        let closed = owner.close();
        CLOSE_PIPE_FAULT.store(false, Ordering::Release);
        let physical_wait = owner.status.is_some();
        let blocked_files = owner
            .receipt
            .as_ref()
            .is_some_and(|r| r.files.input.is_some() || r.files.output.is_some());
        let sticky = owner.sticky.is_some();
        drop(doc);
        *pending_slot().lock().unwrap() = Some(owner);
        PENDING_PRESENT.store(true, Ordering::Release);
        let retained = weak.upgrade().is_some();
        let second = dormant_client();
        let generation_before = second.current_generation();
        let charged_before = budget.used();
        second.set_document(Arc::new("x\n".into()), "Rust".into(), 2);
        let denied = second.shared.lock().update.is_none()
            && second.current_generation() == generation_before
            && second.request(2, None) == 0
            && budget.used() == charged_before;
        // This component control has a manually-owned Connection, not a controller thread.
        // Mark the actual owner terminally stopped before exercising orphan backstop.
        // Real SAME-thread recovery is covered separately by SA14, never by this reaper.
        let original_shared = client.shared.clone();
        client.shutdown().join().unwrap();
        let retry =
            SyntaxClient::new(SyntaxLimits::default(), Arc::new(AtomicBool::new(false))).unwrap();
        retry.retry_cleanup();
        let until = Instant::now() + Duration::from_secs(4);
        while PENDING_PRESENT.load(Ordering::Acquire) && Instant::now() < until {
            thread::sleep(Duration::from_millis(2));
        }
        let resolved = !PENDING_PRESENT.load(Ordering::Acquire);
        let failed = matches!(
            original_shared.lock().cleanup,
            CleanupStatus::Reaped { failed: true }
        );
        retry.shutdown().join().unwrap();
        assert!(
            live && cancelled
                && !closed
                && physical_wait
                && blocked_files
                && sticky
                && retained
                && denied
                && resolved
                && failed
        );
        assert!(weak.upgrade().is_none());
        assert!(!SYNTAX_ACTIVE.load(Ordering::Acquire));
    }
}

#[cfg(test)]
#[path = "syntax_same_client_r2_tests.rs"]
mod same_client_r2_tests;
