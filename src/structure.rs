//! One session-owned controller for complete structural interpretation.
use crate::{
    structure_model::*,
    structure_protocol::{self as ipc, Frame, Message},
    worker_process::{ChildJob, ReapState, ReapTicket},
};
use std::{
    io,
    process::{Child, Command, Stdio},
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Tui,
    Inline,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StructureFailure {
    InvalidToml,
    SourceBudget,
    ArenaBudget,
    Depth,
    Timeout,
    InvalidIpc,
    WorkerExited,
    UnsupportedPlatform,
    ContainmentFailed,
    InvalidJson,
    UnicodeRepresentation,
    InvalidYaml,
    UnsupportedYamlVersion,
    InvalidYamlCoreTag,
    YamlRepresentation,
    InvalidXml,
    UnsupportedXmlVersion,
    UnsupportedXmlEncoding,
    UnsupportedXmlEntity,
    XmlRepresentation,
}
impl StructureFailure {
    pub fn description(self) -> &'static str {
        match self {
            Self::InvalidXml => "XML no válido; se conserva la fuente",
            Self::UnsupportedXmlVersion => "versión XML no soportada; se requiere 1.0",
            Self::UnsupportedXmlEncoding => "codificación XML no soportada; se requiere UTF-8",
            Self::UnsupportedXmlEntity => "entidad XML no procesada; se conserva la fuente",
            Self::XmlRepresentation => {
                "rango o representación XML no acreditado; se conserva la fuente"
            }
            Self::InvalidYaml => "YAML no válido; se conserva la fuente",
            Self::UnsupportedYamlVersion => "versión YAML explícita no soportada; se requiere 1.2",
            Self::InvalidYamlCoreTag => {
                "etiqueta Core YAML incompatible con contenido o forma; se conserva la fuente"
            }
            Self::YamlRepresentation => {
                "rango o representación YAML no acreditado; se conserva la fuente"
            }
            Self::InvalidJson => "JSON no válido; se conserva la fuente",
            Self::UnicodeRepresentation => {
                "surrogate Unicode aislado no representable; se conserva la fuente"
            }
            Self::InvalidToml => "TOML no válido; se conserva la fuente",
            Self::SourceBudget => "fuente demasiado grande para estructura",
            Self::ArenaBudget => "estructura supera el presupuesto",
            Self::Depth => "estructura supera profundidad 64",
            Self::Timeout => "interpretación estructural agotó su plazo",
            Self::InvalidIpc => "respuesta estructural no válida",
            Self::WorkerExited => "proceso estructural terminó inesperadamente",
            Self::UnsupportedPlatform => {
                "contención estructural pendiente de validar en esta plataforma"
            }
            Self::ContainmentFailed => "no se pudo contener o recoger el proceso estructural",
        }
    }
    pub(crate) fn decode(n: u8) -> Option<Self> {
        Some(match n {
            0 => Self::InvalidToml,
            1 => Self::SourceBudget,
            2 => Self::ArenaBudget,
            3 => Self::Depth,
            4 => Self::Timeout,
            5 => Self::InvalidIpc,
            6 => Self::WorkerExited,
            7 => Self::UnsupportedPlatform,
            8 => Self::ContainmentFailed,
            9 => Self::InvalidJson,
            10 => Self::UnicodeRepresentation,
            11 => Self::InvalidYaml,
            12 => Self::UnsupportedYamlVersion,
            13 => Self::InvalidYamlCoreTag,
            14 => Self::YamlRepresentation,
            15 => Self::InvalidXml,
            16 => Self::UnsupportedXmlVersion,
            17 => Self::UnsupportedXmlEncoding,
            18 => Self::UnsupportedXmlEntity,
            19 => Self::XmlRepresentation,
            _ => return None,
        })
    }
}
#[derive(Debug)]
pub enum StructureEvent {
    Ready {
        generation: u64,
        request_id: u64,
        structure: Arc<Structure>,
    },
    Unavailable {
        generation: u64,
        request_id: u64,
        reason: StructureFailure,
    },
}
struct Request {
    format: StructureFormat,
    generation: u64,
    id: u64,
    revision: u64,
    source: Arc<String>,
    deadline: Instant,
    syntax: ReapTicket,
}
struct Active {
    generation: u64,
    id: u64,
    cancel: Arc<AtomicBool>,
    ticket: ReapTicket,
}
struct State {
    generation: u64,
    revision: u64,
    source: Option<Arc<String>>,
    pending: Option<Request>,
    active: Option<Active>,
    latest: Option<u64>,
    failure: Option<StructureFailure>,
    event: Option<StructureEvent>,
    stopping: bool,
    cleanup_failed: bool,
}
struct Shared {
    format: StructureFormat,
    state: Mutex<State>,
    wake: Condvar,
    cancel: Arc<AtomicBool>,
}
impl Shared {
    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }
}
pub struct StructureClient {
    shared: Arc<Shared>,
    owner: Option<JoinHandle<()>>,
}
pub struct StructureShutdown {
    owner: Option<JoinHandle<()>>,
}
impl StructureShutdown {
    pub fn join(mut self) -> io::Result<()> {
        if let Some(owner) = self.owner.take() {
            owner
                .join()
                .map_err(|_| io::Error::other("structure controller panicked"))?;
        }
        Ok(())
    }
}
impl StructureClient {
    pub fn new(cancelled: Arc<AtomicBool>, format: StructureFormat) -> io::Result<Self> {
        let shared = Arc::new(Shared {
            format,
            state: Mutex::new(State {
                generation: 0,
                revision: 0,
                source: None,
                pending: None,
                active: None,
                latest: None,
                failure: None,
                event: None,
                stopping: false,
                cleanup_failed: false,
            }),
            wake: Condvar::new(),
            cancel: cancelled,
        });
        let background = shared.clone();
        let owner = thread::Builder::new()
            .name("structure-controller".into())
            .spawn(move || controller(background))?;
        Ok(Self {
            shared,
            owner: Some(owner),
        })
    }
    pub fn set_document(&self, source: Arc<String>, generation: u64) {
        let mut state = self.shared.lock();
        if state.stopping {
            return;
        }
        if state.generation != generation {
            state.failure = None;
        }
        if let Some(active) = &state.active {
            active.cancel.store(true, Ordering::Release);
        }
        state.generation = generation;
        state.revision = state.revision.wrapping_add(1);
        state.source = Some(source);
        state.pending = None;
        state.latest = None;
        state.event = None;
        drop(state);
        self.shared.wake.notify_one();
    }
    pub fn request(
        &self,
        generation: u64,
        request_id: u64,
        mode: Mode,
        syntax_reaped: ReapTicket,
    ) -> bool {
        let mut state = self.shared.lock();
        if state.stopping
            || state.cleanup_failed
            || state.generation != generation
            || state.failure.is_some()
            || state.source.is_none()
            || self.shared.cancel.load(Ordering::Acquire)
        {
            return false;
        }
        if let Some(active) = &state.active {
            active.cancel.store(true, Ordering::Release);
        }
        let source = Arc::clone(state.source.as_ref().unwrap());
        let revision = state.revision;
        state.latest = Some(request_id);
        state.event = None;
        state.pending = Some(Request {
            format: self.shared.format,
            generation,
            id: request_id,
            revision,
            source,
            deadline: Instant::now()
                + match mode {
                    Mode::Tui => Duration::from_secs(2),
                    Mode::Inline => Duration::from_secs(5),
                },
            syntax: syntax_reaped,
        });
        drop(state);
        self.shared.wake.notify_one();
        true
    }
    pub fn cancel_request(&self, generation: u64, request_id: u64) -> ReapTicket {
        let mut state = self.shared.lock();
        if state.cleanup_failed {
            let ticket = ReapTicket::pending();
            ticket.complete(false);
            return ticket;
        }
        let pending = state
            .pending
            .as_ref()
            .is_some_and(|r| r.generation == generation && r.id == request_id);
        if pending {
            state.pending = None;
            state.latest = None;
        }
        let ticket = if let Some(active) = &state.active {
            if (active.generation == generation && active.id == request_id) || pending {
                active.cancel.store(true, Ordering::Release);
                active.ticket.clone()
            } else {
                ReapTicket::reaped()
            }
        } else {
            ReapTicket::reaped()
        };
        drop(state);
        self.shared.wake.notify_one();
        ticket
    }
    pub fn clear_document(&self, generation: u64) {
        let mut state = self.shared.lock();
        if state.generation != generation {
            state.failure = None;
        }
        if let Some(active) = &state.active {
            active.cancel.store(true, Ordering::Release);
        }
        state.generation = generation;
        state.revision = state.revision.wrapping_add(1);
        state.source = None;
        state.pending = None;
        state.latest = None;
        state.event = None;
        drop(state);
        self.shared.wake.notify_one();
    }
    pub fn poll(&mut self) -> Option<StructureEvent> {
        self.shared.lock().event.take()
    }
    pub fn cancel(&self) {
        self.shared.cancel.store(true, Ordering::Release);
        self.stop();
    }
    fn stop(&self) {
        let mut state = self.shared.lock();
        state.stopping = true;
        state.pending = None;
        if let Some(active) = &state.active {
            active.cancel.store(true, Ordering::Release);
        }
        drop(state);
        self.shared.wake.notify_one();
    }
    pub fn shutdown(mut self) -> StructureShutdown {
        self.stop();
        StructureShutdown {
            owner: self.owner.take(),
        }
    }
}
impl Drop for StructureClient {
    fn drop(&mut self) {
        self.stop();
    }
}
#[derive(Debug)]
enum WorkError {
    Cancelled,
    Failure(StructureFailure),
}
fn status(shared: &Shared, request: &Request, cancel: &AtomicBool) -> Result<(), WorkError> {
    if cancel.load(Ordering::Acquire) || shared.cancel.load(Ordering::Acquire) {
        return Err(WorkError::Cancelled);
    }
    let state = shared.lock();
    if state.stopping
        || state.revision != request.revision
        || state.generation != request.generation
        || state.latest != Some(request.id)
    {
        return Err(WorkError::Cancelled);
    }
    drop(state);
    if Instant::now() >= request.deadline {
        return Err(WorkError::Failure(StructureFailure::Timeout));
    }
    Ok(())
}
fn controller(shared: Arc<Shared>) {
    loop {
        let (request, cancel, ticket) = {
            let mut state = shared.lock();
            loop {
                if state.stopping || shared.cancel.load(Ordering::Acquire) {
                    return;
                }
                if let Some(request) = state.pending.take() {
                    if state.source.is_none()
                        || state.revision != request.revision
                        || state.generation != request.generation
                        || state.latest != Some(request.id)
                    {
                        continue;
                    }
                    let cancel = Arc::new(AtomicBool::new(false));
                    let ticket = ReapTicket::pending();
                    state.active = Some(Active {
                        generation: request.generation,
                        id: request.id,
                        cancel: cancel.clone(),
                        ticket: ticket.clone(),
                    });
                    break (request, cancel, ticket);
                }
                state = shared
                    .wake
                    .wait_timeout(state, Duration::from_millis(25))
                    .unwrap_or_else(|e| e.into_inner())
                    .0;
            }
        };
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            perform(&shared, &request, &cancel)
        }));
        let (result, reaped) = outcome.unwrap_or((
            Err(WorkError::Failure(StructureFailure::WorkerExited)),
            false,
        ));
        ticket.complete(reaped);
        let mut state = shared.lock();
        state.active = None;
        if !reaped {
            state.cleanup_failed = true;
            state.pending = None;
            state.failure = Some(StructureFailure::ContainmentFailed);
            state.event = Some(StructureEvent::Unavailable {
                generation: state.generation,
                request_id: state.latest.unwrap_or(request.id),
                reason: StructureFailure::ContainmentFailed,
            });
            continue;
        }
        if state.stopping
            || state.revision != request.revision
            || state.generation != request.generation
            || state.latest != Some(request.id)
            || cancel.load(Ordering::Acquire)
            || shared.cancel.load(Ordering::Acquire)
        {
            continue;
        }
        match result {
            Ok(model) => {
                state.event = Some(StructureEvent::Ready {
                    generation: request.generation,
                    request_id: request.id,
                    structure: Arc::new(model),
                })
            }
            Err(WorkError::Failure(reason)) => {
                state.failure = Some(reason);
                state.event = Some(StructureEvent::Unavailable {
                    generation: request.generation,
                    request_id: request.id,
                    reason,
                });
            }
            Err(WorkError::Cancelled) => {}
        }
    }
}
struct Connection {
    child: Child,
    job: Option<ChildJob>,
    receive: Option<mpsc::Receiver<io::Result<Option<Frame>>>>,
    io: Option<JoinHandle<()>>,
    closed: bool,
}
impl Connection {
    fn spawn(request: &Request) -> Result<Self, (StructureFailure, bool)> {
        let executable =
            std::env::current_exe().map_err(|_| (StructureFailure::WorkerExited, true))?;
        let mut command = Command::new(executable);
        command
            .arg("--structure-worker")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(windows_sys::Win32::System::Threading::CREATE_NO_WINDOW);
        }
        let mut child = command
            .spawn()
            .map_err(|_| (StructureFailure::WorkerExited, true))?;
        let job = match ChildJob::attach(&child) {
            Ok(job) => job,
            Err(_) => {
                let _ = child.kill();
                let reaped = child.wait().is_ok();
                return Err((StructureFailure::ContainmentFailed, reaped));
            }
        };
        let mut input = child.stdin.take().unwrap();
        let mut output = child.stdout.take().unwrap();
        let (send, receive) = mpsc::sync_channel(1);
        let source = request.source.clone();
        let format = request.format;
        let generation = request.generation;
        let id = request.id;
        let io = thread::Builder::new()
            .name("structure-io".into())
            .spawn(move || {
                let result = (|| -> io::Result<()> {
                    let boot = ipc::read_frame(&mut output)?
                        .ok_or_else(|| io::Error::other("missing boot"))?;
                    if boot.generation != 0
                        || boot.request != 0
                        || !matches!(boot.message, Message::Boot)
                    {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid boot"));
                    }
                    let mut ack = |message: Message, expected: u32| -> io::Result<()> {
                        ipc::write_frame(&mut input, &Frame::new(generation, id, message))?;
                        let response = ipc::read_frame(&mut output)?
                            .ok_or_else(|| io::Error::other("missing ACK"))?;
                        if response.generation != generation
                            || response.request != id
                            || !matches!(response.message,Message::Ack{offset}if offset==expected)
                        {
                            return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid ACK"));
                        }
                        Ok(())
                    };
                    ack(
                        Message::Init {
                            format,
                            bytes: source.len() as u32,
                        },
                        0,
                    )?;
                    let mut offset = 0u32;
                    for bytes in source.as_bytes().chunks(ipc::CHUNK) {
                        let next = offset + bytes.len() as u32;
                        ack(
                            Message::Source {
                                offset,
                                bytes: bytes.to_vec(),
                            },
                            next,
                        )?;
                        offset = next;
                    }
                    ipc::write_frame(&mut input, &Frame::new(generation, id, Message::SourceEnd))?;
                    loop {
                        let response = ipc::read_frame(&mut output)?;
                        let eof = response.is_none();
                        if send.send(Ok(response)).is_err() {
                            return Ok(());
                        }
                        if eof {
                            return Ok(());
                        }
                    }
                })();
                if let Err(error) = result {
                    let _ = send.send(Err(error));
                }
            });
        match io {
            Ok(io) => Ok(Self {
                child,
                job: Some(job),
                receive: Some(receive),
                io: Some(io),
                closed: false,
            }),
            Err(_) => {
                job.close();
                let _ = child.kill();
                let reaped = child.wait().is_ok();
                Err((StructureFailure::WorkerExited, reaped))
            }
        }
    }
    fn next(
        &self,
        shared: &Shared,
        request: &Request,
        cancel: &AtomicBool,
    ) -> Result<Option<Frame>, WorkError> {
        loop {
            status(shared, request, cancel)?;
            match self
                .receive
                .as_ref()
                .unwrap()
                .recv_timeout(Duration::from_millis(5))
            {
                Ok(Ok(frame)) => {
                    status(shared, request, cancel)?;
                    return Ok(frame);
                }
                Ok(Err(error)) => {
                    return Err(WorkError::Failure(
                        if error.kind() == io::ErrorKind::InvalidData {
                            StructureFailure::InvalidIpc
                        } else {
                            StructureFailure::WorkerExited
                        },
                    ));
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(_) => return Err(WorkError::Failure(StructureFailure::WorkerExited)),
            }
        }
    }
    fn close(&mut self) -> bool {
        if self.closed {
            return true;
        }
        self.receive.take(); // release bounded send BEFORE joining
        if let Some(job) = self.job.take() {
            job.close();
        }
        let mut ok = true;
        match self.child.try_wait() {
            Ok(None) => {
                if self.child.kill().is_err() {
                    ok = false;
                }
            }
            Ok(Some(_)) => {}
            Err(_) => {
                ok = false;
                let _ = self.child.kill();
            }
        }
        if self.child.wait().is_err() {
            ok = false;
        }
        if let Some(io) = self.io.take()
            && io.join().is_err()
        {
            ok = false;
        }
        self.closed = true;
        ok
    }
}
impl Drop for Connection {
    fn drop(&mut self) {
        self.close();
    }
}
fn model_error(error: ModelError) -> WorkError {
    match error {
        ModelError::Cancelled => WorkError::Cancelled,
        ModelError::Timeout => WorkError::Failure(StructureFailure::Timeout),
        ModelError::SourceLimit => WorkError::Failure(StructureFailure::SourceBudget),
        ModelError::Budget => WorkError::Failure(StructureFailure::ArenaBudget),
        ModelError::Depth => WorkError::Failure(StructureFailure::Depth),
        ModelError::Invalid => WorkError::Failure(StructureFailure::InvalidIpc),
    }
}
fn perform(
    shared: &Shared,
    request: &Request,
    cancel: &AtomicBool,
) -> (Result<Structure, WorkError>, bool) {
    let ready = (|| {
        loop {
            status(shared, request, cancel)?;
            match request.syntax.state() {
                ReapState::Reaped => break,
                ReapState::Failed => {
                    return Err(WorkError::Failure(StructureFailure::ContainmentFailed));
                }
                ReapState::Pending => thread::sleep(Duration::from_millis(5)),
            }
        }
        if request.source.len() > MAX_STRUCTURE_SOURCE {
            return Err(WorkError::Failure(StructureFailure::SourceBudget));
        }
        if !crate::worker_process::supported() {
            return Err(WorkError::Failure(StructureFailure::UnsupportedPlatform));
        }
        Ok(())
    })();
    if let Err(error) = ready {
        return (Err(error), true);
    }
    let mut connection = match Connection::spawn(request) {
        Ok(c) => c,
        Err((reason, reaped)) => return (Err(WorkError::Failure(reason)), reaped),
    };
    let result = receive_model(&connection, shared, request, cancel);
    let result = result.and_then(|model| {
        loop {
            status(shared, request, cancel)?;
            match connection.child.try_wait() {
                Ok(Some(exit)) if exit.success() => break,
                Ok(Some(_)) | Err(_) => {
                    return Err(WorkError::Failure(StructureFailure::WorkerExited));
                }
                Ok(None) => thread::sleep(Duration::from_millis(2)),
            }
        }
        Ok(model)
    });
    let reaped = connection.close();
    (result, reaped)
}
fn receive_model(
    connection: &Connection,
    shared: &Shared,
    request: &Request,
    cancel: &AtomicBool,
) -> Result<Structure, WorkError> {
    receive_sequence(
        request.source.clone(),
        request.generation,
        request.id,
        request.format,
        || connection.next(shared, request, cancel),
        || status(shared, request, cancel),
    )
}
fn receive_sequence(
    source: Arc<String>,
    generation: u64,
    id: u64,
    format: StructureFormat,
    mut read: impl FnMut() -> Result<Option<Frame>, WorkError>,
    mut check: impl FnMut() -> Result<(), WorkError>,
) -> Result<Structure, WorkError> {
    let invalid = || WorkError::Failure(StructureFailure::InvalidIpc);
    let mut next = || -> Result<Option<Frame>, WorkError> {
        let frame = read()?;
        if frame
            .as_ref()
            .is_some_and(|f| f.generation != generation || f.request != id)
        {
            return Err(invalid());
        }
        Ok(frame)
    };
    let counts = match next()?.ok_or_else(invalid)?.message {
        Message::Begin(c) => c,
        Message::Failure(n) => {
            return Err(WorkError::Failure(
                StructureFailure::decode(n).ok_or_else(invalid)?,
            ));
        }
        _ => return Err(invalid()),
    };
    if counts.format != format
        || counts.nodes == 0
        || counts.nodes as usize + counts.entries as usize > MAX_STRUCTURE_ITEMS
        || counts.cooked as usize > MAX_STRUCTURE_BYTES
        || counts.declarations as usize > MAX_STRUCTURE_BYTES / 24
        || counts.parts as usize > MAX_STRUCTURE_BYTES / 12
        || counts.syntax_depth as usize > MAX_STRUCTURE_DEPTH
        || counts.key_components as usize > MAX_STRUCTURE_DEPTH
        || counts.xml_metadata > counts.nodes
        || (format == StructureFormat::Xml
            && (counts.xml_metadata != counts.nodes
                || counts.metadata != 0
                || counts.alias_edges != 0
                || counts.declarations != 0
                || counts.parts != 0
                || counts.key_components != 0))
        || (format != StructureFormat::Xml && counts.xml_metadata != 0)
        || counts.metadata > counts.nodes
        || counts.alias_edges > counts.nodes
        || (counts.format != StructureFormat::Yaml
            && (counts.metadata != 0 || counts.alias_edges != 0))
        || (counts.format == StructureFormat::Yaml
            && (counts.declarations != 0 || counts.parts != 0 || counts.key_components != 0))
    {
        return Err(invalid());
    }
    if format == StructureFormat::Xml
        && Structure::xml_projected_bytes(
            counts.nodes as usize,
            counts.entries as usize,
            counts.cooked as usize,
        )
        .is_none_or(|n| n > MAX_STRUCTURE_BYTES)
    {
        return Err(WorkError::Failure(StructureFailure::ArenaBudget));
    }
    let mut builder = StructureBuilder::new_for_format(source.clone(), generation, format)
        .map_err(model_error)?;
    let mut expected = Vec::<u32>::new();
    let transport_headroom = if matches!(format, StructureFormat::Yaml | StructureFormat::Xml) {
        4 * ipc::MAX_FRAME
    } else {
        0
    };
    builder
        .set_scratch_bytes(
            (counts.nodes as usize)
                .checked_mul(std::mem::size_of::<u32>())
                .and_then(|n| n.checked_add(transport_headroom))
                .ok_or_else(invalid)?,
        )
        .map_err(model_error)?;
    expected
        .try_reserve_exact(counts.nodes as usize)
        .map_err(|_| WorkError::Failure(StructureFailure::ArenaBudget))?;
    builder
        .set_scratch_bytes(expected.capacity() * std::mem::size_of::<u32>() + transport_headroom)
        .map_err(model_error)?;
    let mut nodes = 0u32;
    let mut expected_total = 0u64;
    let mut entries = 0u32;
    let mut parent = 0usize;
    let mut in_parent = 0u32;
    let mut cooked = 0u32;
    let mut declarations = 0u32;
    let mut parts = 0u32;
    let mut metadata = 0u32;
    let mut xml_metadata = 0u32;
    let mut aliases = 0u32;
    let mut phase = 0u8;
    loop {
        check()?;
        let frame = next()?.ok_or_else(invalid)?;
        let incoming_phase = match &frame.message {
            Message::Nodes { .. } => 1,
            Message::Entries { .. } => 2,
            Message::Cooked { .. } => 3,
            Message::Declaration { .. } => 4,
            Message::Metadata { .. } | Message::XmlMetadata { .. } => 5,
            Message::End => 6,
            Message::Failure(_) => phase,
            _ => return Err(invalid()),
        };
        if incoming_phase < phase {
            return Err(invalid());
        }
        phase = incoming_phase;
        match frame.message {
            Message::Nodes {
                offset,
                nodes: batch,
            } => {
                if offset != nodes
                    || nodes as usize + batch.len() > counts.nodes as usize
                    || batch.is_empty()
                {
                    return Err(invalid());
                }
                for node in batch {
                    let valid_kind = match format {
                        StructureFormat::Xml => node.spec.kind.is_xml(),
                        StructureFormat::Yaml => {
                            !node.spec.kind.is_xml()
                                && !matches!(
                                    node.spec.kind,
                                    StructureKind::ArrayOfTables
                                        | StructureKind::JsonNumber
                                        | StructureKind::OffsetDateTime
                                        | StructureKind::LocalDateTime
                                        | StructureKind::LocalDate
                                        | StructureKind::LocalTime
                                )
                        }
                        StructureFormat::Json => matches!(
                            node.spec.kind,
                            StructureKind::Mapping
                                | StructureKind::Sequence
                                | StructureKind::String
                                | StructureKind::Boolean
                                | StructureKind::Null
                                | StructureKind::JsonNumber
                        ),
                        StructureFormat::Toml => {
                            !node.spec.kind.is_xml()
                                && !matches!(
                                    node.spec.kind,
                                    StructureKind::Null
                                        | StructureKind::JsonNumber
                                        | StructureKind::YamlStream
                                        | StructureKind::YamlDocument
                                        | StructureKind::Alias
                                        | StructureKind::YamlTaggedScalar
                                )
                        }
                    };
                    if !valid_kind {
                        return Err(invalid());
                    }
                    builder.push_node(node.spec).map_err(model_error)?;
                    expected_total += node.entries as u64;
                    if expected_total > counts.entries as u64 {
                        return Err(invalid());
                    }
                    expected.push(node.entries);
                    nodes += 1;
                }
            }
            Message::Entries {
                parent: owner,
                entries: batch,
            } => {
                if nodes != counts.nodes || batch.is_empty() {
                    return Err(invalid());
                }
                while parent < expected.len() && expected[parent] == 0 {
                    parent += 1;
                }
                if owner.0 as usize != parent
                    || parent >= expected.len()
                    || in_parent as usize + batch.len() > expected[parent] as usize
                    || entries as usize + batch.len() > counts.entries as usize
                {
                    return Err(invalid());
                }
                let len = batch.len() as u32;
                builder
                    .extend_entries_chunk(owner, in_parent, batch)
                    .map_err(model_error)?;
                in_parent += len;
                entries += len;
                if in_parent == expected[parent] {
                    parent += 1;
                    in_parent = 0;
                }
            }
            Message::Cooked { offset, bytes } => {
                if nodes != counts.nodes
                    || entries != counts.entries
                    || offset != cooked
                    || bytes.is_empty()
                    || cooked as usize + bytes.len() > counts.cooked as usize
                {
                    return Err(invalid());
                }
                builder.append_cooked(&bytes).map_err(model_error)?;
                cooked += bytes.len() as u32;
            }
            Message::Declaration { spec, parts: batch } => {
                if nodes != counts.nodes
                    || entries != counts.entries
                    || cooked != counts.cooked
                    || declarations >= counts.declarations
                    || parts as usize + batch.len() > counts.parts as usize
                {
                    return Err(invalid());
                }
                parts += batch.len() as u32;
                builder.push_declaration(spec, batch).map_err(model_error)?;
                declarations += 1;
            }
            Message::Metadata { offset, records } => {
                if format != StructureFormat::Yaml
                    || nodes != counts.nodes
                    || entries != counts.entries
                    || cooked != counts.cooked
                    || declarations != counts.declarations
                    || parts != counts.parts
                    || offset != metadata
                    || records.is_empty()
                    || metadata as usize + records.len() > counts.metadata as usize
                {
                    return Err(invalid());
                }
                for record in records {
                    aliases += u32::from(record.alias.is_some());
                    builder.push_metadata(record).map_err(model_error)?;
                    metadata += 1;
                }
            }
            Message::XmlMetadata { offset, records } => {
                if format != StructureFormat::Xml
                    || nodes != counts.nodes
                    || entries != counts.entries
                    || cooked != counts.cooked
                    || offset != xml_metadata
                    || records.is_empty()
                    || xml_metadata as usize + records.len() > counts.xml_metadata as usize
                {
                    return Err(invalid());
                }
                for record in records {
                    builder.push_xml_metadata(record).map_err(model_error)?;
                    xml_metadata += 1;
                }
            }
            Message::Failure(n) => {
                return Err(WorkError::Failure(
                    StructureFailure::decode(n).ok_or_else(invalid)?,
                ));
            }
            Message::End => {
                if nodes != counts.nodes
                    || expected_total != counts.entries as u64
                    || entries != counts.entries
                    || cooked != counts.cooked
                    || declarations != counts.declarations
                    || parts != counts.parts
                    || xml_metadata != counts.xml_metadata
                    || metadata != counts.metadata
                    || aliases != counts.alias_edges
                {
                    return Err(invalid());
                }
                if next()?.is_some() {
                    return Err(invalid());
                }
                break;
            }
            _ => return Err(invalid()),
        }
        if builder
            .persistent_bytes()
            .checked_add(expected.capacity() * std::mem::size_of::<u32>() + transport_headroom)
            .is_none_or(|n| n > MAX_STRUCTURE_BYTES)
        {
            return Err(WorkError::Failure(StructureFailure::ArenaBudget));
        }
    }
    drop(expected);
    builder
        .set_scratch_bytes(transport_headroom)
        .map_err(model_error)?;
    let model = builder
        .finish_with(
            counts.root,
            counts.syntax_depth as usize,
            counts.key_components as usize,
            || check().is_err(),
        )
        .map_err(|error| {
            if error == ModelError::Cancelled {
                check().err().unwrap_or(WorkError::Cancelled)
            } else {
                model_error(error)
            }
        })?;
    check()?;
    Ok(model)
}

#[cfg(test)]
mod receiver_tests {
    use super::*;
    fn frames() -> (Arc<String>, Vec<Frame>) {
        let source = Arc::new("{a: &x [*x], b: \"\\e\"}".to_owned());
        let (model, depth, keys) =
            crate::yaml_worker::project(source.clone(), 31, &AtomicBool::new(false)).unwrap();
        let mut bytes = Vec::new();
        crate::structure_worker::output(
            &model,
            StructureFormat::Yaml,
            depth,
            keys,
            31,
            9,
            &mut bytes,
        )
        .unwrap();
        let mut input = bytes.as_slice();
        let mut frames = Vec::new();
        while let Some(frame) = ipc::read_frame(&mut input).unwrap() {
            frames.push(frame);
        }
        (source, frames)
    }
    fn receive(source: Arc<String>, frames: Vec<Frame>) -> Result<Structure, WorkError> {
        let mut frames = frames.into_iter();
        receive_sequence(
            source,
            31,
            9,
            StructureFormat::Yaml,
            || Ok(frames.next()),
            || Ok(()),
        )
    }
    #[test]
    fn whole_sequence_roundtrip_revalidates_yaml_and_derives_parents() {
        let (source, frames) = frames();
        let model = receive(source.clone(), frames).unwrap();
        assert!(Arc::ptr_eq(&source, &model.source_handle()));
        assert_eq!(model.format(), StructureFormat::Yaml);
        let doc = model.root().entry(0).unwrap().value();
        assert_eq!(model.parent(doc.id()).unwrap().owner, model.root().id());
        let root = doc.entry(0).unwrap().value();
        let target = root.entry(0).unwrap().value();
        let alias = target.entry(0).unwrap().value();
        assert_eq!(alias.metadata().unwrap().alias.unwrap().target, target.id());
        assert_eq!(root.entry(1).unwrap().value().decoded(), Some("\u{1b}"));
    }
    #[test]
    fn malformed_phases_counts_identity_scope_alias_offsets_and_eof_never_publish() {
        for alteration in 0..14 {
            let (source, mut frames) = frames();
            match alteration {
                0 => {
                    if let Message::Begin(c) = &mut frames[0].message {
                        c.format = StructureFormat::Json;
                    }
                }
                1 => {
                    if let Message::Begin(c) = &mut frames[0].message {
                        c.metadata -= 1;
                    }
                }
                2 => {
                    if let Message::Begin(c) = &mut frames[0].message {
                        c.alias_edges += 1;
                    }
                }
                3 => frames[1].generation += 1,
                4 => {
                    let meta = frames
                        .iter()
                        .position(|f| matches!(f.message, Message::Metadata { .. }))
                        .unwrap();
                    frames.swap(1, meta);
                }
                5 => {
                    let cooked = frames
                        .iter()
                        .position(|f| matches!(f.message, Message::Cooked { .. }))
                        .unwrap();
                    let entries = frames
                        .iter()
                        .position(|f| matches!(f.message, Message::Entries { .. }))
                        .unwrap();
                    frames.swap(cooked, entries);
                }
                6 => {
                    if let Some(Frame {
                        message: Message::Metadata { offset, .. },
                        ..
                    }) = frames
                        .iter_mut()
                        .find(|f| matches!(f.message, Message::Metadata { .. }))
                    {
                        *offset = 1;
                    }
                }
                7 => {
                    if let Some(Frame {
                        message: Message::Metadata { records, .. },
                        ..
                    }) = frames
                        .iter_mut()
                        .find(|f| matches!(f.message, Message::Metadata { .. }))
                    {
                        records.last_mut().unwrap().document = 99;
                    }
                }
                8 => {
                    if let Some(Frame {
                        message: Message::Metadata { records, .. },
                        ..
                    }) = frames
                        .iter_mut()
                        .find(|f| matches!(f.message, Message::Metadata { .. }))
                    {
                        let m = records.iter_mut().find(|m| m.alias.is_some()).unwrap();
                        m.alias.as_mut().unwrap().target = m.node;
                    }
                }
                9 => {
                    if let Some(Frame {
                        message: Message::Entries { parent, .. },
                        ..
                    }) = frames
                        .iter_mut()
                        .find(|f| matches!(f.message, Message::Entries { .. }))
                    {
                        *parent = NodeId(u32::MAX);
                    }
                }
                10 => {
                    if let Some(Frame {
                        message: Message::Cooked { bytes, .. },
                        ..
                    }) = frames
                        .iter_mut()
                        .find(|f| matches!(f.message, Message::Cooked { .. }))
                    {
                        bytes[0] = 255;
                    }
                }
                11 => {
                    frames.pop();
                }
                12 => frames.push(Frame::new(31, 9, Message::End)),
                _ => {
                    if let Message::Begin(c) = &mut frames[0].message {
                        c.syntax_depth += 1;
                    }
                }
            }
            assert!(receive(source, frames).is_err(), "alteration {alteration}");
        }
    }
    #[test]
    fn cancellation_during_receive_and_failure_midstream_invalidate_everything() {
        let (source, frames) = frames();
        let mut frames = frames.into_iter();
        let mut ticks = 0;
        let result = receive_sequence(
            source,
            31,
            9,
            StructureFormat::Yaml,
            || Ok(frames.next()),
            || {
                ticks += 1;
                if ticks >= 2 {
                    Err(WorkError::Cancelled)
                } else {
                    Ok(())
                }
            },
        );
        assert!(matches!(result, Err(WorkError::Cancelled)));
        let (source, mut frames) = self::frames();
        frames.insert(
            2,
            Frame::new(31, 9, Message::Failure(StructureFailure::InvalidYaml as u8)),
        );
        assert!(matches!(
            receive(source, frames),
            Err(WorkError::Failure(StructureFailure::InvalidYaml))
        ));
    }
}

#[cfg(test)]
mod xml_transport_tests {
    use super::*;
    fn frames() -> (Arc<String>, Vec<Frame>) {
        let source =
            Arc::new("<r xmlns:p='urn:a&amp;b' a='x&#xD;y'><p:x/>text&amp;</r>".to_owned());
        let (m, d, k) =
            crate::xml_worker::project(source.clone(), 31, &AtomicBool::new(false)).unwrap();
        let mut bytes = Vec::new();
        crate::structure_worker::output(&m, StructureFormat::Xml, d, k, 31, 9, &mut bytes).unwrap();
        let mut input = bytes.as_slice();
        let mut out = Vec::new();
        while let Some(f) = ipc::read_frame(&mut input).unwrap() {
            out.push(f)
        }
        (source, out)
    }
    fn receive(source: Arc<String>, frames: Vec<Frame>) -> Result<Structure, WorkError> {
        let mut frames = frames.into_iter();
        receive_sequence(
            source,
            31,
            9,
            StructureFormat::Xml,
            || Ok(frames.next()),
            || Ok(()),
        )
    }
    #[test]
    fn xml_roundtrip_source_semantics_and_parent_identity() {
        let (s, f) = frames();
        let m = receive(s.clone(), f).unwrap();
        assert!(Arc::ptr_eq(&s, &m.source_handle()));
        let r = m.root().entry(0).unwrap().value();
        assert_eq!(r.xml_metadata().unwrap().attribute_count, 2);
        assert_eq!(r.entry(1).unwrap().value().decoded(), Some("x\ry"));
        assert_eq!(r.entry(2).unwrap().value().parent().unwrap().owner, r.id());
    }
    #[test]
    fn forged_xml_counts_phases_metadata_ranges_namespaces_and_tail_rejected() {
        for alteration in 0..13 {
            let (s, mut f) = frames();
            match alteration {
                0 => {
                    if let Message::Begin(c) = &mut f[0].message {
                        c.nodes = 1_000_000;
                        c.entries = 0;
                        c.xml_metadata = c.nodes
                    }
                }
                1 => {
                    if let Message::Begin(c) = &mut f[0].message {
                        c.xml_metadata -= 1
                    }
                }
                2 => {
                    if let Message::Begin(c) = &mut f[0].message {
                        c.metadata = 1
                    }
                }
                3 => {
                    let at = f
                        .iter()
                        .position(|f| matches!(f.message, Message::XmlMetadata { .. }))
                        .unwrap();
                    f.swap(1, at)
                }
                4 => {
                    let at = f
                        .iter()
                        .position(|f| matches!(f.message, Message::XmlMetadata { .. }))
                        .unwrap();
                    if let Message::XmlMetadata { records, .. } = &mut f[at].message {
                        records[1].has_doctype = true
                    }
                }
                5 => {
                    let at = f
                        .iter()
                        .position(|f| matches!(f.message, Message::XmlMetadata { .. }))
                        .unwrap();
                    if let Message::XmlMetadata { records, .. } = &mut f[at].message {
                        records[1].attribute_count = 0
                    }
                }
                6 => {
                    let at = f
                        .iter()
                        .position(|f| matches!(f.message, Message::XmlMetadata { .. }))
                        .unwrap();
                    if let Message::XmlMetadata { records, .. } = &mut f[at].message {
                        records[1].name.as_mut().unwrap().local.start += 1
                    }
                }
                7 => {
                    let at = f
                        .iter()
                        .position(|f| matches!(f.message, Message::XmlMetadata { .. }))
                        .unwrap();
                    if let Message::XmlMetadata { records, .. } = &mut f[at].message {
                        records[4].name.as_mut().unwrap().namespace = None
                    }
                }
                8 => {
                    let at = f
                        .iter()
                        .position(|f| matches!(f.message, Message::Cooked { .. }))
                        .unwrap();
                    if let Message::Cooked { bytes, .. } = &mut f[at].message {
                        bytes[0] = b'z'
                    }
                }
                9 => {
                    let at = f
                        .iter()
                        .position(|f| matches!(f.message, Message::XmlMetadata { .. }))
                        .unwrap();
                    if let Message::XmlMetadata { offset, .. } = &mut f[at].message {
                        *offset = 1
                    }
                }
                10 => f.push(Frame::new(31, 9, Message::End)),
                11 => {
                    f.pop();
                }
                12 => f[1].generation += 1,
                _ => unreachable!(),
            }
            assert!(receive(s, f).is_err(), "alteration {alteration}");
        }
    }
    #[test]
    fn xml_aggregate_budget_rejected_before_reading_payload() {
        let s = Arc::new("<r/>".into());
        let c = ipc::Counts {
            format: StructureFormat::Xml,
            nodes: 1_000_000,
            entries: 0,
            declarations: 0,
            parts: 0,
            cooked: 0,
            root: NodeId(0),
            syntax_depth: 1,
            key_components: 0,
            metadata: 0,
            alias_edges: 0,
            xml_metadata: 1_000_000,
        };
        let mut first = Some(Frame::new(31, 9, Message::Begin(c)));
        let result = receive_sequence(
            s,
            31,
            9,
            StructureFormat::Xml,
            || {
                Ok(first
                    .take()
                    .expect("payload read after rejected Begin")
                    .into())
            },
            || Ok(()),
        );
        assert!(matches!(
            result,
            Err(WorkError::Failure(StructureFailure::ArenaBudget))
        ));
    }
}
