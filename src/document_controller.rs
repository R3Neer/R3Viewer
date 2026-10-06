//! Parent-owned one-shot document transaction; a failed receipt never publishes a Document.
use crate::{document_budget::ActorArena, document_model::*, document_source::DocumentSource};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
/// A stale worker cannot publish into a superseding source generation/request.
pub struct IdentityLatch {
    identity: Mutex<(u64, u64)>,
}
impl IdentityLatch {
    pub fn new(generation: u64, request: u64) -> Self {
        Self {
            identity: Mutex::new((generation, request)),
        }
    }
    pub fn update(&self, generation: u64, request: u64) -> Result<()> {
        *self
            .identity
            .lock()
            .map_err(|_| Error::new(ErrorKind::InvalidContext))? = (generation, request);
        Ok(())
    }
    fn matches(&self, source: &DocumentSource) -> bool {
        self.identity
            .lock()
            .map(|pair| *pair == (source.generation(), source.request()))
            .unwrap_or(false)
    }
}

static BUSY: AtomicBool = AtomicBool::new(false);
static PENDING: AtomicBool = AtomicBool::new(false);
struct BusyLease;
impl Drop for BusyLease {
    fn drop(&mut self) {
        BUSY.store(false, Ordering::Release);
    }
}
// Keep the small record-vector owners inline; boxing would add an uncharged allocation.
#[allow(clippy::large_enum_variant)]
pub enum ControllerPoll {
    Idle,
    Pending,
    Ready(Document),
    Failed(Error),
    CleanupComplete { original_failure: Option<Error> },
}
#[allow(clippy::large_enum_variant)]
enum Completion {
    Document(Result<Document>),
    Cleanup(Result<Option<Error>>),
}
/// UI methods never join a live task or hold the process-ownership mutex.
pub struct DocumentController {
    task: Option<std::thread::JoinHandle<Completion>>,
    cancel: Option<Arc<AtomicBool>>,
    external: Option<Arc<AtomicBool>>,
    identity: Option<Arc<IdentityLatch>>,
}
impl Default for DocumentController {
    fn default() -> Self {
        Self::new()
    }
}
impl DocumentController {
    pub fn new() -> Self {
        Self {
            task: None,
            cancel: None,
            external: None,
            identity: None,
        }
    }
    pub fn prepare(
        &mut self,
        executable: std::path::PathBuf,
        source: Arc<DocumentSource>,
        profile: Profile,
        arena: ActorArena,
        cancel: Arc<AtomicBool>,
        identity: Arc<IdentityLatch>,
    ) -> Result<()> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        if self.task.is_some() || PENDING.load(Ordering::Acquire) {
            return Err(Error::new(ErrorKind::PendingCleanup));
        }
        BUSY.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| Error::new(ErrorKind::PendingCleanup))?;
        let lease = BusyLease;
        let requested = Arc::new(AtomicBool::new(false));
        let stop = requested.clone();
        let external = cancel.clone();
        let latch = identity.clone();
        let task = std::thread::Builder::new()
            .name("document-control".to_owned())
            .spawn(move || {
                let _lease = lease;
                #[cfg(windows)]
                {
                    Completion::Document(platform::load(
                        &executable,
                        source,
                        profile,
                        arena,
                        cancel,
                        identity,
                        stop,
                        deadline,
                    ))
                }
                #[cfg(not(windows))]
                {
                    let _ = (
                        executable, source, profile, arena, cancel, identity, stop, deadline,
                    );
                    Completion::Document(Err(Error::new(ErrorKind::UnsupportedPlatform)))
                }
            })
            .map_err(|_| Error::new(ErrorKind::Allocation))?;
        self.cancel = Some(requested);
        self.external = Some(external);
        self.identity = Some(latch);
        self.task = Some(task);
        Ok(())
    }
    pub fn poll(&mut self) -> ControllerPoll {
        let Some(task) = self.task.as_ref() else {
            return ControllerPoll::Idle;
        };
        if !task.is_finished() {
            return ControllerPoll::Pending;
        }
        let requested = self.cancel.take();
        let external = self.external.take();
        let identity = self.identity.take();
        let Some(task) = self.task.take() else {
            return ControllerPoll::Failed(Error::new(ErrorKind::Internal));
        };
        match task.join() {
            Ok(Completion::Document(Ok(document))) => {
                if requested
                    .as_ref()
                    .is_some_and(|s| s.load(Ordering::Acquire))
                    || external.as_ref().is_some_and(|s| s.load(Ordering::Acquire))
                    || !identity
                        .as_ref()
                        .is_some_and(|l| l.matches(document.source()))
                {
                    ControllerPoll::Failed(Error::new(ErrorKind::Cancelled))
                } else {
                    ControllerPoll::Ready(document)
                }
            }
            Ok(Completion::Document(Err(error))) | Ok(Completion::Cleanup(Err(error))) => {
                ControllerPoll::Failed(error)
            }
            Ok(Completion::Cleanup(Ok(original_failure))) => {
                ControllerPoll::CleanupComplete { original_failure }
            }
            Err(_) => ControllerPoll::Failed(Error::new(ErrorKind::Internal)),
        }
    }
    pub fn cancel(&mut self) {
        if let Some(cancel) = self.cancel.as_ref() {
            cancel.store(true, Ordering::Release);
        }
    }
    pub fn pending_cleanup(&self) -> bool {
        PENDING.load(Ordering::Acquire)
    }
    pub fn retry_cleanup(&mut self) -> Result<()> {
        if self.task.is_some() {
            return Err(Error::new(ErrorKind::PendingCleanup));
        }
        BUSY.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| Error::new(ErrorKind::PendingCleanup))?;
        let lease = BusyLease;
        let task = std::thread::Builder::new()
            .name("document-reap".to_owned())
            .spawn(move || {
                let _lease = lease;
                #[cfg(windows)]
                {
                    Completion::Cleanup(platform::retry())
                }
                #[cfg(not(windows))]
                {
                    Completion::Cleanup(Ok(None))
                }
            })
            .map_err(|_| Error::new(ErrorKind::Allocation))?;
        self.task = Some(task);
        Ok(())
    }
}
impl Drop for DocumentController {
    fn drop(&mut self) {
        self.cancel(); /* The supervisor thread retains all process authority until its bounded cleanup completes or transfers Pending into SLOT. */
    }
}
#[cfg(windows)]
mod platform {
    use super::*;
    use crate::{document_obsidian::Control, document_protocol as ipc, worker_process::ChildJob};
    use std::{
        fs::File,
        io::{Read, Write},
        os::windows::{
            io::{AsRawHandle, FromRawHandle, IntoRawHandle},
            process::CommandExt,
        },
        process::{Child, Command, ExitStatus, Stdio},
        sync::{Mutex, OnceLock},
        thread::{self, JoinHandle},
        time::{Duration, Instant},
    };
    use windows_sys::Win32::{
        Foundation::{CloseHandle, HANDLE_FLAG_INHERIT, SetHandleInformation},
        Security::SECURITY_ATTRIBUTES,
        System::Pipes::CreatePipe,
    };

    #[cfg(test)]
    pub(super) static FAULTS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    #[cfg(test)]
    pub(super) static BLOCKED_PID: std::sync::atomic::AtomicU32 =
        std::sync::atomic::AtomicU32::new(0);
    #[cfg(test)]
    fn take_fault(bit: u32, once: bool) -> bool {
        if once {
            FAULTS.fetch_and(!bit, Ordering::AcqRel) & bit != 0
        } else {
            FAULTS.load(Ordering::Acquire) & bit != 0
        }
    }
    fn receipt_close(file: &mut Option<File>, role: &str) -> Result<()> {
        #[cfg(test)]
        if role == "document-source" && (take_fault(8, true) || take_fault(16, false)) {
            return Err(issue());
        }
        let _ = role;
        close(file)
    }
    fn issue() -> Error {
        Error::new(ErrorKind::Containment)
    }
    fn close(file: &mut Option<File>) -> Result<()> {
        if let Some(value) = file.take() {
            let raw = value.into_raw_handle();
            if unsafe { CloseHandle(raw) } == 0 {
                *file = Some(unsafe { File::from_raw_handle(raw) });
                return Err(issue());
            }
        }
        Ok(())
    }
    fn start_pipe() -> (Option<File>, Option<File>, Result<()>) {
        let mut read = std::ptr::null_mut();
        let mut write = std::ptr::null_mut();
        let sa = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: std::ptr::null_mut(),
            bInheritHandle: 1,
        };
        if unsafe { CreatePipe(&mut read, &mut write, &sa, 0) } == 0 {
            return (None, None, Err(issue()));
        }
        let read = unsafe { File::from_raw_handle(read) };
        let write = unsafe { File::from_raw_handle(write) };
        if unsafe { SetHandleInformation(write.as_raw_handle(), HANDLE_FLAG_INHERIT, 0) } == 0 {
            return (Some(read), Some(write), Err(issue()));
        }
        (Some(read), Some(write), Ok(()))
    }
    struct Receipt<T> {
        result: Result<T>,
        file: Option<File>,
        closed: bool,
    }
    struct ThreadOwned<T> {
        handle: Option<JoinHandle<Receipt<T>>>,
        mailbox: Arc<Mutex<Option<File>>>,
        receipt: Option<Receipt<T>>,
        start_error: Option<Error>,
        never_started: bool,
        role: &'static str,
        _source_lease: Option<Arc<DocumentSource>>,
    }
    impl<T: Send + 'static> ThreadOwned<T> {
        fn launch(
            file: File,
            name: &'static str,
            source_lease: Option<Arc<DocumentSource>>,
            work: impl FnOnce(&mut File) -> Result<T> + Send + 'static,
        ) -> Self {
            let mailbox = Arc::new(Mutex::new(Some(file)));
            let lease = mailbox.clone();
            let builder = thread::Builder::new().name(name.to_owned());
            #[cfg(test)]
            let builder = {
                let bit = match name {
                    "document-source" => 1,
                    "document-model" => 2,
                    "document-errors" => 4,
                    _ => 0,
                };
                if bit != 0 && take_fault(bit, true) {
                    builder.stack_size(usize::MAX - 4095)
                } else {
                    builder
                }
            };
            let started = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                builder.spawn(move || {
                    let mut file = lease.lock().ok().and_then(|mut slot| slot.take());
                    let result = if let Some(ref mut stream) = file {
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| work(stream)))
                            .unwrap_or_else(|_| Err(Error::new(ErrorKind::Internal)))
                    } else {
                        Err(issue())
                    };
                    #[cfg(test)]
                    let result = if name == "document-model" && take_fault(32, true) {
                        Err(Error::new(ErrorKind::InvalidIpc))
                    } else {
                        result
                    };
                    let closed = receipt_close(&mut file, name).is_ok();
                    Receipt {
                        result,
                        file,
                        closed,
                    }
                })
            }))
            .unwrap_or_else(|_| Err(std::io::Error::other("thread setup failed")));
            match started {
                Ok(handle) => Self {
                    handle: Some(handle),
                    mailbox,
                    receipt: None,
                    start_error: None,
                    never_started: false,
                    role: name,
                    _source_lease: source_lease,
                },
                Err(_) => Self {
                    handle: None,
                    mailbox,
                    receipt: None,
                    start_error: Some(issue()),
                    never_started: true,
                    role: name,
                    _source_lease: source_lease,
                },
            }
        }
        fn collect(&mut self) -> Result<bool> {
            if self.handle.as_ref().is_some_and(JoinHandle::is_finished) {
                let handle = self.handle.take().ok_or_else(issue)?;
                match handle.join() {
                    Ok(receipt) => self.receipt = Some(receipt),
                    Err(_) => {
                        self.start_error = Some(Error::new(ErrorKind::Internal));
                    }
                }
            }
            if let Some(error) = self.start_error {
                return Err(error);
            }
            Ok(self.handle.is_none() && self.receipt.is_some())
        }
        fn cleanup(&mut self) -> Result<bool> {
            let mut error = None;
            let _ = self.collect();
            if self.handle.is_none() {
                if let Ok(mut mailbox) = self.mailbox.lock() {
                    if let Err(e) = close(&mut mailbox) {
                        error.get_or_insert(e);
                    }
                } else {
                    error.get_or_insert(issue());
                }
                if let Some(receipt) = self.receipt.as_mut()
                    && !receipt.closed
                {
                    if let Err(e) = receipt_close(&mut receipt.file, self.role) {
                        error.get_or_insert(e);
                    } else {
                        receipt.closed = true;
                    }
                }
            }
            let mailbox_closed = self.mailbox.lock().map(|m| m.is_none()).unwrap_or(false);
            let receipt_closed = self.receipt.as_ref().is_some_and(|r| r.closed)
                || (self.never_started && self.handle.is_none() && mailbox_closed);
            if let Some(e) = error {
                return Err(e);
            }
            Ok(self.handle.is_none() && mailbox_closed && receipt_closed)
        }
        fn failure(&self) -> Option<Error> {
            self.start_error
                .or_else(|| {
                    self.receipt
                        .as_ref()
                        .and_then(|r| r.result.as_ref().err().copied())
                })
                .or_else(|| self.receipt.as_ref().filter(|r| !r.closed).map(|_| issue()))
        }
    }
    struct Pending {
        child: Option<Child>,
        job: Option<ChildJob>,
        start_read: Option<File>,
        start_write: Option<File>,
        input: Option<File>,
        output: Option<File>,
        errors: Option<File>,
        writer: Option<ThreadOwned<()>>,
        reader: Option<ThreadOwned<crate::document_validate::ValidatedCandidate>>,
        stderr: Option<ThreadOwned<()>>,
        source: Arc<DocumentSource>,
        arena: ActorArena,
        stop: Arc<AtomicBool>,
        status: Option<ExitStatus>,
        sticky: Option<Error>,
    }
    impl Pending {
        fn mark(&mut self, result: Result<()>) {
            if let Err(e) = result {
                self.sticky.get_or_insert(e);
            }
        }
        fn collect(&mut self) {
            if let Some(writer) = self.writer.as_mut() {
                if let Err(e) = writer.collect() {
                    self.sticky.get_or_insert(e);
                }
                if let Some(e) = writer.failure() {
                    self.sticky.get_or_insert(e);
                }
            }
            if let Some(reader) = self.reader.as_mut() {
                if let Err(e) = reader.collect() {
                    self.sticky.get_or_insert(e);
                }
                if let Some(e) = reader.failure() {
                    self.sticky.get_or_insert(e);
                }
            }
            if let Some(stderr) = self.stderr.as_mut() {
                if let Err(e) = stderr.collect() {
                    self.sticky.get_or_insert(e);
                }
                if let Some(e) = stderr.failure() {
                    self.sticky.get_or_insert(e);
                }
            }
        }
        fn cleanup(&mut self) -> bool {
            self.stop.store(true, Ordering::Release);
            let deadline = Instant::now() + Duration::from_secs(2);
            let result = close(&mut self.start_read);
            self.mark(result);
            let result = close(&mut self.start_write);
            self.mark(result);
            let result = close(&mut self.input);
            self.mark(result);
            let result = close(&mut self.output);
            self.mark(result);
            let result = close(&mut self.errors);
            self.mark(result);
            if self.status.is_none() {
                if let Some(job) = self.job.as_ref()
                    && job.terminate(75).is_err()
                {
                    self.sticky.get_or_insert(issue());
                }
                if let Some(child) = self.child.as_mut()
                    && child.kill().is_err()
                {
                    self.sticky.get_or_insert(issue());
                }
            }
            loop {
                if self.status.is_none()
                    && let Some(child) = self.child.as_mut()
                {
                    match child.try_wait() {
                        Ok(Some(status)) => match child.wait() {
                            Ok(again) if again == status => self.status = Some(status),
                            _ => {
                                self.sticky.get_or_insert(issue());
                            }
                        },
                        Ok(None) => {}
                        Err(_) => {
                            self.sticky.get_or_insert(issue());
                        }
                    }
                }
                let mut threads_closed = true;
                if let Some(t) = self.writer.as_mut() {
                    match t.cleanup() {
                        Ok(done) => threads_closed &= done,
                        Err(e) => {
                            self.sticky.get_or_insert(e);
                            threads_closed = false;
                        }
                    }
                }
                if let Some(t) = self.reader.as_mut() {
                    match t.cleanup() {
                        Ok(done) => threads_closed &= done,
                        Err(e) => {
                            self.sticky.get_or_insert(e);
                            threads_closed = false;
                        }
                    }
                }
                if let Some(t) = self.stderr.as_mut() {
                    match t.cleanup() {
                        Ok(done) => threads_closed &= done,
                        Err(e) => {
                            self.sticky.get_or_insert(e);
                            threads_closed = false;
                        }
                    }
                }
                self.collect();
                let active_zero = if let Some(job) = self.job.as_ref() {
                    match job.active_processes() {
                        Ok(0) => true,
                        Ok(_) => {
                            if job.terminate(75).is_err() {
                                self.sticky.get_or_insert(issue());
                            }
                            false
                        }
                        Err(_) => {
                            self.sticky.get_or_insert(issue());
                            false
                        }
                    }
                } else {
                    true
                };
                let streams_closed = self.start_read.is_none()
                    && self.start_write.is_none()
                    && self.input.is_none()
                    && self.output.is_none()
                    && self.errors.is_none();
                if (self.child.is_none() || self.status.is_some())
                    && threads_closed
                    && streams_closed
                    && active_zero
                {
                    if let Some(job) = self.job.take() {
                        match job.close_checked() {
                            Ok(()) => {}
                            Err(failure) => {
                                let _ = failure.error;
                                self.job = Some(failure.job);
                                self.sticky.get_or_insert(issue());
                            }
                        }
                    }
                    if self.job.is_none() {
                        self.child = None;
                        return true;
                    }
                }
                if Instant::now() >= deadline {
                    return false;
                }
                if self.status.is_none()
                    && let Some(child) = self.child.as_mut()
                    && child.kill().is_err()
                {
                    self.sticky.get_or_insert(issue());
                }
                thread::sleep(Duration::from_millis(2));
            }
        }
    }
    static SLOT: OnceLock<Mutex<Option<Pending>>> = OnceLock::new();
    fn slot() -> &'static Mutex<Option<Pending>> {
        SLOT.get_or_init(|| Mutex::new(None))
    }

    pub fn retry() -> Result<Option<Error>> {
        let mut slot = slot()
            .lock()
            .map_err(|_| Error::new(ErrorKind::PendingCleanup))?;
        if let Some(p) = slot.as_mut() {
            if p.cleanup() {
                let original = p.sticky;
                *slot = None;
                PENDING.store(false, Ordering::Release);
                Ok(original)
            } else {
                Err(Error::new(ErrorKind::PendingCleanup))
            }
        } else {
            Ok(None)
        }
    }
    // The absolute request deadline and three independent ownership tokens are explicit.
    #[allow(clippy::too_many_arguments)]
    pub fn load(
        executable: &std::path::Path,
        source: Arc<DocumentSource>,
        profile: Profile,
        arena: ActorArena,
        cancel: Arc<AtomicBool>,
        identity: Arc<IdentityLatch>,
        requested: Arc<AtomicBool>,
        deadline: Instant,
    ) -> Result<Document> {
        let mut guard = slot()
            .lock()
            .map_err(|_| Error::new(ErrorKind::PendingCleanup))?;
        if guard.is_some() {
            return Err(Error::new(ErrorKind::PendingCleanup));
        }
        if cancel.load(Ordering::Acquire) || requested.load(Ordering::Acquire) {
            return Err(Error::new(ErrorKind::Cancelled));
        }
        if !identity.matches(&source) {
            return Err(Error::new(ErrorKind::Cancelled));
        }
        if Instant::now() >= deadline {
            return Err(Error::new(ErrorKind::Timeout));
        }
        let (read, write, pipe_setup) = start_pipe();
        let mut p = Pending {
            child: None,
            job: None,
            start_read: read,
            start_write: write,
            input: None,
            output: None,
            errors: None,
            writer: None,
            reader: None,
            stderr: None,
            source: source.clone(),
            arena: arena.clone(),
            stop: Arc::new(AtomicBool::new(false)),
            status: None,
            sticky: None,
        };
        let setup = (|| -> Result<()> {
            pipe_setup?;
            let handle = p.start_read.as_ref().ok_or_else(issue)?.as_raw_handle() as usize;
            let child = Command::new(executable)
                .args([
                    "--document-worker",
                    &source.generation().to_string(),
                    &source.request().to_string(),
                    &handle.to_string(),
                ])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .creation_flags(0x08000000)
                .spawn()
                .map_err(|_| Error::new(ErrorKind::WorkerExited))?;
            p.child = Some(child);
            let child = p.child.as_mut().ok_or_else(issue)?;
            p.input = child
                .stdin
                .take()
                .map(|stream| unsafe { File::from_raw_handle(stream.into_raw_handle()) });
            p.output = child
                .stdout
                .take()
                .map(|stream| unsafe { File::from_raw_handle(stream.into_raw_handle()) });
            p.errors = child
                .stderr
                .take()
                .map(|stream| unsafe { File::from_raw_handle(stream.into_raw_handle()) });
            match ChildJob::attach_verified(child) {
                Ok(job) => p.job = Some(job),
                Err(failure) => {
                    let _ = failure.error;
                    p.job = failure.job;
                    return Err(issue());
                }
            }
            close(&mut p.start_read)?;
            #[cfg(test)]
            if take_fault(64, false) {
                let job = p.job.as_ref().ok_or_else(issue)?;
                let child = p.child.as_mut().ok_or_else(issue)?;
                if job.active_processes().map_err(|_| issue())? != 1
                    || child.try_wait().map_err(|_| issue())?.is_some()
                {
                    return Err(issue());
                }
                BLOCKED_PID.store(child.id(), Ordering::Release);
                while take_fault(64, false)
                    && !requested.load(Ordering::Acquire)
                    && !cancel.load(Ordering::Acquire)
                    && Instant::now() < deadline
                {
                    thread::sleep(Duration::from_millis(1));
                }
                BLOCKED_PID.store(0, Ordering::Release);
                if requested.load(Ordering::Acquire) || cancel.load(Ordering::Acquire) {
                    return Err(Error::new(ErrorKind::Cancelled));
                }
            }
            if Instant::now() >= deadline {
                return Err(Error::new(ErrorKind::Timeout));
            }
            let stop = p.stop.clone();
            let lease = source.clone();
            p.writer = Some(ThreadOwned::launch(
                p.input.take().ok_or_else(issue)?,
                "document-source",
                Some(source.clone()),
                move |file| {
                    let ctl = Control {
                        cancel: &stop,
                        deadline,
                    };
                    ipc::send_source(&lease, profile, file, &ctl)
                },
            ));
            let stop = p.stop.clone();
            let lease = source.clone();
            let receiver_arena = arena.clone();
            p.reader = Some(ThreadOwned::launch(
                p.output.take().ok_or_else(issue)?,
                "document-model",
                None,
                move |file| {
                    let ctl = Control {
                        cancel: &stop,
                        deadline,
                    };
                    ipc::receive_model(lease, profile, receiver_arena, file, &ctl)
                },
            ));
            p.stderr = Some(ThreadOwned::launch(
                p.errors.take().ok_or_else(issue)?,
                "document-errors",
                None,
                move |file| {
                    let mut buffer = [0; 4096];
                    let mut total = 0usize;
                    let mut overflow = false;
                    loop {
                        match file.read(&mut buffer) {
                            Ok(0) => break,
                            Ok(n) => {
                                total = total.saturating_add(n);
                                overflow |= total > 65_536;
                            }
                            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                            Err(_) => return Err(Error::new(ErrorKind::InvalidIpc)),
                        }
                    }
                    if overflow {
                        Err(Error::new(ErrorKind::InvalidIpc))
                    } else {
                        Ok(())
                    }
                },
            ));
            // Observation barrier only: real writer/readers are owned. Existing
            // fault16 acts on the writer receipt; no close semantics changed.
            #[cfg(test)]
            if take_fault(128, false) {
                let child = p.child.as_mut().ok_or_else(issue)?;
                if p.writer.is_none()
                    || p.reader.is_none()
                    || p.stderr.is_none()
                    || child.try_wait().map_err(|_| issue())?.is_some()
                {
                    return Err(issue());
                }
                BLOCKED_PID.store(child.id(), Ordering::Release);
                while take_fault(128, false)
                    && !requested.load(Ordering::Acquire)
                    && !cancel.load(Ordering::Acquire)
                    && Instant::now() < deadline
                {
                    thread::sleep(Duration::from_millis(1));
                }
                BLOCKED_PID.store(0, Ordering::Release);
                if requested.load(Ordering::Acquire) || cancel.load(Ordering::Acquire) {
                    return Err(Error::new(ErrorKind::Cancelled));
                }
            }
            p.collect();
            if p.sticky.is_some() {
                return Err(p.sticky.ok_or_else(issue)?);
            }
            p.start_write
                .as_mut()
                .ok_or_else(issue)?
                .write_all(&[1])
                .map_err(|_| issue())?;
            close(&mut p.start_write)?;
            Ok(())
        })();
        p.mark(setup);
        while p.sticky.is_none() {
            p.collect();
            if cancel.load(Ordering::Acquire)
                || requested.load(Ordering::Acquire)
                || !identity.matches(&source)
            {
                p.sticky.get_or_insert(Error::new(ErrorKind::Cancelled));
                break;
            }
            if Instant::now() >= deadline {
                p.sticky.get_or_insert(Error::new(ErrorKind::Timeout));
                break;
            }
            if p.status.is_none()
                && let Some(child) = p.child.as_mut()
            {
                match child.try_wait() {
                    Ok(Some(status)) => match child.wait() {
                        Ok(again) if again == status => p.status = Some(status),
                        _ => {
                            p.sticky.get_or_insert(issue());
                        }
                    },
                    Ok(None) => {}
                    Err(_) => {
                        p.sticky.get_or_insert(issue());
                    }
                }
            }
            let done = p
                .writer
                .as_ref()
                .is_some_and(|t| t.handle.is_none() && t.receipt.is_some())
                && p.reader
                    .as_ref()
                    .is_some_and(|t| t.handle.is_none() && t.receipt.is_some())
                && p.stderr
                    .as_ref()
                    .is_some_and(|t| t.handle.is_none() && t.receipt.is_some());
            if done && p.status.is_some() {
                break;
            }
            thread::sleep(Duration::from_millis(2));
        }
        if p.status.is_some_and(|s| !s.success()) {
            p.sticky.get_or_insert(Error::new(ErrorKind::WorkerExited));
        }
        let closed = p.cleanup();
        let error = p.sticky;
        if !closed {
            PENDING.store(true, Ordering::Release);
            *guard = Some(p);
            return Err(error.unwrap_or_else(|| Error::new(ErrorKind::PendingCleanup)));
        }
        if let Some(error) = error {
            return Err(error);
        }
        if cancel.load(Ordering::Acquire)
            || requested.load(Ordering::Acquire)
            || !identity.matches(&source)
        {
            return Err(Error::new(ErrorKind::Cancelled));
        }
        let candidate = p
            .reader
            .as_mut()
            .and_then(|r| r.receipt.take())
            .ok_or_else(issue)?
            .result?;
        if !Arc::ptr_eq(&candidate.source, &source) {
            return Err(Error::new(ErrorKind::InvalidContext));
        }
        let _ = (&p.source, &p.arena);
        Ok(Document::from_validated(candidate))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{document_obsidian::Control, input::SourceOrigin};
    use std::time::{Duration, Instant};
    fn completed(identity: Arc<IdentityLatch>) -> DocumentController {
        let cancel = Arc::new(AtomicBool::new(false));
        let source = Arc::new(
            DocumentSource::new(
                Arc::new("text".to_owned()),
                SourceOrigin::Literal,
                2,
                10,
                &cancel,
            )
            .unwrap(),
        );
        let ctl = Control {
            cancel: &cancel,
            deadline: Instant::now() + Duration::from_secs(10),
        };
        let candidate =
            crate::document_parser::project(source, Profile::Core, ActorArena::new(), &ctl)
                .unwrap();
        let depth = candidate.max_depth;
        let proof = crate::document_validate::quarantine(candidate, depth, &ctl).unwrap();
        let task =
            std::thread::spawn(move || Completion::Document(Ok(Document::from_validated(proof))));
        let deadline = Instant::now() + Duration::from_secs(1);
        while !task.is_finished() {
            assert!(Instant::now() < deadline);
            std::thread::yield_now();
        }
        DocumentController {
            task: Some(task),
            cancel: Some(Arc::new(AtomicBool::new(false))),
            external: Some(cancel),
            identity: Some(identity),
        }
    }
    #[test]
    fn finished_result_is_not_published_after_cancel_before_poll() {
        let mut c = completed(Arc::new(IdentityLatch::new(2, 10)));
        c.cancel();
        assert!(matches!(
            c.poll(),
            ControllerPoll::Failed(Error {
                kind: ErrorKind::Cancelled,
                ..
            })
        ));
    }
    #[test]
    fn finished_result_is_not_published_after_identity_update_before_poll() {
        let identity = Arc::new(IdentityLatch::new(2, 10));
        let mut c = completed(identity.clone());
        identity.update(2, 20).unwrap();
        assert!(matches!(
            c.poll(),
            ControllerPoll::Failed(Error {
                kind: ErrorKind::Cancelled,
                ..
            })
        ));
    }
    #[test]
    fn finished_result_is_not_published_after_external_cancel_before_poll() {
        let mut c = completed(Arc::new(IdentityLatch::new(2, 10)));
        c.external.as_ref().unwrap().store(true, Ordering::Release);
        assert!(matches!(
            c.poll(),
            ControllerPoll::Failed(Error {
                kind: ErrorKind::Cancelled,
                ..
            })
        ));
    }
    #[test]
    fn paired_identity_has_no_torn_tuple() {
        let identity = IdentityLatch::new(1, 10);
        identity.update(2, 20).unwrap();
        let source = DocumentSource::new(
            Arc::new("x".to_owned()),
            SourceOrigin::Literal,
            2,
            10,
            &AtomicBool::new(false),
        )
        .unwrap();
        assert!(!identity.matches(&source));
    }
}

#[cfg(all(test, windows))]
mod native_fault_tests {
    use super::*;
    use crate::input::SourceOrigin;
    use std::{
        path::PathBuf,
        thread,
        time::{Duration, Instant},
    };
    fn executable() -> PathBuf {
        PathBuf::from(
            std::env::var_os("R3VIEWER_DOCUMENT_TEST_EXE")
                .expect("native fault controls require explicit private worker path"),
        )
    }
    fn finish(controller: &mut DocumentController) -> ControllerPoll {
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            match controller.poll() {
                ControllerPoll::Pending => {}
                done => return done,
            }
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(2));
        }
    }
    fn request(controller: &mut DocumentController) -> std::sync::Weak<DocumentSource> {
        let cancel = Arc::new(AtomicBool::new(false));
        let source = Arc::new(
            DocumentSource::new(
                Arc::new("source remains complete".to_owned()),
                SourceOrigin::Literal,
                7,
                11,
                &cancel,
            )
            .unwrap(),
        );
        let weak = Arc::downgrade(&source);
        controller
            .prepare(
                executable(),
                source,
                Profile::Core,
                ActorArena::new(),
                cancel,
                Arc::new(IdentityLatch::new(7, 11)),
            )
            .unwrap();
        weak
    }
    #[test]
    fn real_thread_start_failures_keep_streams_and_child_until_cleanup() {
        for bit in [1, 2, 4] {
            platform::FAULTS.store(bit, Ordering::Release);
            let mut controller = DocumentController::new();
            let weak = request(&mut controller);
            assert!(matches!(finish(&mut controller), ControllerPoll::Failed(_)));
            assert!(!controller.pending_cleanup());
            assert!(weak.upgrade().is_none());
            platform::FAULTS.store(0, Ordering::Release);
        }
    }
    #[test]
    fn late_reader_receipt_after_valid_stream_never_publishes_document() {
        platform::FAULTS.store(32, Ordering::Release);
        let mut controller = DocumentController::new();
        let weak = request(&mut controller);
        assert!(matches!(
            finish(&mut controller),
            ControllerPoll::Failed(Error {
                kind: ErrorKind::InvalidIpc,
                ..
            })
        ));
        assert!(!controller.pending_cleanup());
        assert!(weak.upgrade().is_none());
        platform::FAULTS.store(0, Ordering::Release);
    }
    #[test]
    fn one_shot_writer_close_failure_is_sticky_after_successful_cleanup() {
        platform::FAULTS.store(8, Ordering::Release);
        let mut controller = DocumentController::new();
        let weak = request(&mut controller);
        assert!(matches!(
            finish(&mut controller),
            ControllerPoll::Failed(Error {
                kind: ErrorKind::Containment,
                ..
            })
        ));
        assert!(!controller.pending_cleanup());
        assert!(weak.upgrade().is_none());
        platform::FAULTS.store(0, Ordering::Release);
    }
    #[test]
    fn persistent_writer_close_retains_lease_blocks_next_spawn_and_retry_keeps_first_failure() {
        platform::FAULTS.store(16, Ordering::Release);
        let mut controller = DocumentController::new();
        let weak = request(&mut controller);
        assert!(matches!(
            finish(&mut controller),
            ControllerPoll::Failed(Error {
                kind: ErrorKind::Containment,
                ..
            })
        ));
        assert!(controller.pending_cleanup());
        assert!(weak.upgrade().is_some());
        let cancel = Arc::new(AtomicBool::new(false));
        let source = Arc::new(
            DocumentSource::new(
                Arc::new("next".to_owned()),
                SourceOrigin::Literal,
                8,
                12,
                &cancel,
            )
            .unwrap(),
        );
        assert_eq!(
            controller
                .prepare(
                    executable(),
                    source,
                    Profile::Core,
                    ActorArena::new(),
                    cancel,
                    Arc::new(IdentityLatch::new(8, 12))
                )
                .unwrap_err()
                .kind,
            ErrorKind::PendingCleanup
        );
        platform::FAULTS.store(0, Ordering::Release);
        controller.retry_cleanup().unwrap();
        assert!(matches!(
            finish(&mut controller),
            ControllerPoll::CleanupComplete {
                original_failure: Some(Error {
                    kind: ErrorKind::Containment,
                    ..
                })
            }
        ));
        assert!(!controller.pending_cleanup());
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn cancel_live_verified_job_blocked_before_start_reaps_and_permits_next_request() {
        platform::FAULTS.store(64, Ordering::Release);
        let mut controller = DocumentController::new();
        let weak = request(&mut controller);
        let deadline = Instant::now() + Duration::from_secs(5);
        while platform::BLOCKED_PID.load(Ordering::Acquire) == 0 {
            assert!(Instant::now() < deadline);
            assert!(matches!(controller.poll(), ControllerPoll::Pending));
            thread::sleep(Duration::from_millis(1));
        }
        assert_ne!(platform::BLOCKED_PID.load(Ordering::Acquire), 0);
        controller.cancel();
        assert!(matches!(
            finish(&mut controller),
            ControllerPoll::Failed(Error {
                kind: ErrorKind::Cancelled,
                ..
            })
        ));
        assert!(!controller.pending_cleanup());
        assert!(weak.upgrade().is_none());
        platform::FAULTS.store(0, Ordering::Release);
        let weak = request(&mut controller);
        assert!(matches!(finish(&mut controller), ControllerPoll::Ready(_)));
        assert!(!controller.pending_cleanup());
        assert!(weak.upgrade().is_none());
    }
    #[test]
    fn expired_absolute_request_deadline_never_spawns_worker() {
        let cancel = Arc::new(AtomicBool::new(false));
        let source = Arc::new(
            DocumentSource::new(
                Arc::new("text".to_owned()),
                SourceOrigin::Literal,
                1,
                1,
                &cancel,
            )
            .unwrap(),
        );
        let error = platform::load(
            &executable(),
            source,
            Profile::Core,
            ActorArena::new(),
            cancel,
            Arc::new(IdentityLatch::new(1, 1)),
            Arc::new(AtomicBool::new(false)),
            Instant::now() - Duration::from_secs(1),
        )
        .err()
        .unwrap();
        assert_eq!(error.kind, ErrorKind::Timeout);
        assert!(!PENDING.load(Ordering::Acquire));
    }

    // D3c caller controls: cfg-only suffix, original controller/oracles unchanged.
    struct CallerFaultOwner;
    impl CallerFaultOwner {
        fn reap(&self) -> Option<Error> {
            platform::FAULTS.store(0, Ordering::Release);
            let mut owner = DocumentController::new();
            if !owner.pending_cleanup() {
                return None;
            }
            owner
                .retry_cleanup()
                .expect("owned harness retry, never caller retry");
            let ControllerPoll::CleanupComplete { original_failure } = finish(&mut owner) else {
                panic!("owned physical cleanup failed");
            };
            assert!(!owner.pending_cleanup());
            original_failure
        }
    }
    impl Drop for CallerFaultOwner {
        fn drop(&mut self) {
            platform::FAULTS.store(0, Ordering::Release);
            let mut owner = DocumentController::new();
            if owner.pending_cleanup() && owner.retry_cleanup().is_ok() {
                let deadline = Instant::now() + Duration::from_secs(15);
                while matches!(owner.poll(), ControllerPoll::Pending) && Instant::now() < deadline {
                    thread::sleep(Duration::from_millis(2));
                }
                if owner.pending_cleanup() {
                    eprintln!("OWN D3c harness cleanup still Pending; not a passing receipt");
                }
            }
        }
    }
    fn wait_own_blocked() {
        let deadline = Instant::now() + Duration::from_secs(5);
        while platform::BLOCKED_PID.load(Ordering::Acquire) == 0 {
            assert!(
                Instant::now() < deadline,
                "own verified job beforeSTART witness"
            );
            thread::sleep(Duration::from_millis(1));
        }
    }
    #[test]
    fn co09_caller_sticky_containment_pending_never_returns_source_or_retries() {
        let owner = CallerFaultOwner;
        let config = crate::document_output::tests::config();
        platform::FAULTS.store(16, Ordering::Release);
        let original = Arc::new("# original remains\n".to_owned());
        let weak = Arc::downgrade(&original);
        let error = crate::document_output::prepare(
            executable(),
            original,
            SourceOrigin::Literal,
            80,
            &config,
            Arc::new(AtomicBool::new(false)),
            crate::document_output::OUTPUT_LIMIT,
        )
        .unwrap_err();
        assert_eq!(error.kind, crate::document_output::ErrorKind::Preparation);
        assert!(error.cleanup_pending);
        assert_eq!(error.original_failure.unwrap().kind, ErrorKind::Containment);
        assert!(PENDING.load(Ordering::Acquire));
        assert!(weak.upgrade().is_some());
        let error2 = crate::document_output::prepare(
            executable(),
            Arc::new("next".into()),
            SourceOrigin::Literal,
            80,
            &config,
            Arc::new(AtomicBool::new(false)),
            crate::document_output::OUTPUT_LIMIT,
        )
        .unwrap_err();
        assert!(error2.cleanup_pending);
        assert!(PENDING.load(Ordering::Acquire));
        assert_eq!(owner.reap().unwrap().kind, ErrorKind::Containment);
        assert!(weak.upgrade().is_none());
        eprintln!(
            "CO09 caller noPrepared/Pending/stickyContainment; harness physicalReap then sourceLease dropped; no caller retry/spawn"
        );
    }
    #[test]
    fn co10_caller_cancel_live_before_start_with_close_pending_preserves_130() {
        let owner = CallerFaultOwner;
        let config = crate::document_output::tests::config();
        let exe = executable();
        let cancel = Arc::new(AtomicBool::new(false));
        let signal = cancel.clone();
        let original = Arc::new("# cancel original\n".to_owned());
        let weak = Arc::downgrade(&original);
        platform::FAULTS.store(16 | 128, Ordering::Release);
        let call = thread::spawn(move || {
            crate::document_output::prepare(
                exe,
                original,
                SourceOrigin::Literal,
                80,
                &config,
                cancel,
                crate::document_output::OUTPUT_LIMIT,
            )
        });
        wait_own_blocked();
        signal.store(true, Ordering::Release);
        let error = call.join().unwrap().unwrap_err();
        assert_eq!(error.kind, crate::document_output::ErrorKind::Cancelled);
        assert_eq!(error.exit_code(), 130);
        assert!(error.cleanup_pending);
        assert_eq!(error.original_failure.unwrap().kind, ErrorKind::Cancelled);
        assert!(PENDING.load(Ordering::Acquire));
        assert!(weak.upgrade().is_some());
        assert_eq!(owner.reap().unwrap().kind, ErrorKind::Cancelled);
        assert!(weak.upgrade().is_none());
        eprintln!(
            "CO10 verifiedBeforeSTART cancel130/Pending/noPrepared; harness physicalReap preserves Cancelled cause"
        );
    }
    #[test]
    fn co11_caller_busy_without_pending_never_falls_back_or_cancels_other_owner() {
        let _fault_owner = CallerFaultOwner;
        let config = crate::document_output::tests::config();
        platform::FAULTS.store(64, Ordering::Release);
        let mut controller = DocumentController::new();
        let weak = request(&mut controller);
        wait_own_blocked();
        assert!(!controller.pending_cleanup());
        let error = crate::document_output::prepare(
            executable(),
            Arc::new("second request".into()),
            SourceOrigin::Literal,
            80,
            &config,
            Arc::new(AtomicBool::new(false)),
            crate::document_output::OUTPUT_LIMIT,
        )
        .unwrap_err();
        assert_eq!(error.kind, crate::document_output::ErrorKind::Preparation);
        assert!(!error.cleanup_pending);
        assert_eq!(
            error.original_failure.unwrap().kind,
            ErrorKind::PendingCleanup
        );
        assert!(matches!(controller.poll(), ControllerPoll::Pending));
        assert_ne!(platform::BLOCKED_PID.load(Ordering::Acquire), 0);
        controller.cancel();
        assert!(matches!(
            finish(&mut controller),
            ControllerPoll::Failed(Error {
                kind: ErrorKind::Cancelled,
                ..
            })
        ));
        assert!(!controller.pending_cleanup());
        assert!(weak.upgrade().is_none());
        platform::FAULTS.store(0, Ordering::Release);
        eprintln!(
            "CO11 BUSY/noPending caller noPrepared; previous own request remained blocked until harness cancel"
        );
    }
}
