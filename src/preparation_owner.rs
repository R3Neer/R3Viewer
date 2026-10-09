//! Single process-wide preparation authority; typed output cannot commit before physical join.
#![allow(dead_code)] // Private preparation library; session adoption belongs to D2c/D3c.
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, Receiver, SyncSender, TryRecvError},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
static NEXT: AtomicU64 = AtomicU64::new(1);
static OCCUPIED: AtomicU64 = AtomicU64::new(0);
static PENDING: Mutex<Option<Box<dyn PendingTask>>> = Mutex::new(None);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Binding {
    pub input: u64,
    pub generation: u64,
    pub request: u64,
    pub revision: u64,
    pub digest: Option<[u8; 32]>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Cause {
    Busy,
    Pending,
    Cancelled,
    Stale,
    Spawn,
    SendFull,
    Disconnected,
    Panic,
    Missing,
    Poisoned,
    Limit(&'static str),
}
pub(crate) enum Outcome<O> {
    Ready(O),
    Cancelled,
    Limit(&'static str),
    Failed(Cause),
}
struct Completion<O> {
    binding: Binding,
    outcome: Outcome<O>,
}
struct FinalReceipt<O> {
    cause: Option<Cause>,
    retained: Option<Completion<O>>,
}
struct Permit {
    token: u64,
}
impl Drop for Permit {
    fn drop(&mut self) {
        let _ = OCCUPIED.compare_exchange(self.token, 0, Ordering::AcqRel, Ordering::Acquire);
    }
}
trait PendingTask: Send {
    fn finished(&self) -> bool;
    fn reap(self: Box<Self>) -> ReapReceipt;
}
// Named capture fixes the exact allocation layout charged by common Source factories.
struct WorkerCapture<I, O, F> {
    input: Arc<I>,
    stop: Arc<AtomicBool>,
    binding: Binding,
    sender: SyncSender<Completion<O>>,
    run: F,
}
impl<
    I: Send + Sync + 'static,
    O: Send + 'static,
    F: FnOnce(&I, &AtomicBool) -> Outcome<O> + Send + 'static,
> WorkerCapture<I, O, F>
{
    fn execute(self) -> FinalReceipt<O> {
        execute(self.input, self.stop, self.binding, self.sender, self.run)
    }
}
struct Parts<I, O> {
    input: Arc<I>,
    binding: Binding,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<FinalReceipt<O>>>,
    receiver: Receiver<Completion<O>>,
    candidate: Option<Completion<O>>,
    cause: Option<Cause>,
    closure_bytes: usize,
    _permit: Permit,
}
impl<I: Send + Sync + 'static, O: Send + 'static> PendingTask for Parts<I, O> {
    fn finished(&self) -> bool {
        self.handle.as_ref().is_none_or(JoinHandle::is_finished)
    }
    fn reap(mut self: Box<Self>) -> ReapReceipt {
        let mut worker_panicked = false;
        let receipt = self.handle.take().and_then(|h| match h.join() {
            Ok(r) => Some(r),
            Err(_) => {
                worker_panicked = true;
                self.cause.get_or_insert(Cause::Panic);
                None
            }
        });
        let mut worker_cause = receipt.as_ref().and_then(|r| r.cause);
        if worker_panicked {
            worker_cause = Some(Cause::Panic);
        }
        if let Some(r) = receipt {
            if let Some(c) = r.cause {
                self.cause.get_or_insert(c);
            }
            drop(r.retained);
        }
        ReapReceipt {
            binding: self.binding,
            cause: self.cause,
            worker_cause,
            joined: true,
        }
    }
}
/// Box was allocated before spawn; moving it into Pending requires no replacement allocation.
pub(crate) struct OwnedPreparation<I: Send + Sync + 'static, O: Send + 'static> {
    parts: Option<Box<Parts<I, O>>>,
}
impl<I: Send + Sync + 'static, O: Send + 'static> Drop for OwnedPreparation<I, O> {
    fn drop(&mut self) {
        if let Some(mut p) = self.parts.take() {
            p.stop.store(true, Ordering::Release);
            if p.finished() {
                p.reap();
            } else {
                p.cause.get_or_insert(Cause::Pending);
                let mut slot = PENDING.lock().unwrap_or_else(|e| e.into_inner());
                assert!(slot.is_none(), "single occupied authority");
                *slot = Some(p);
            }
        }
    }
}
/// A completed receipt records known allocation sizes; runtime/channel/thread internals remain unknown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ReapReceipt {
    pub binding: Binding,
    pub cause: Option<Cause>,
    pub worker_cause: Option<Cause>,
    pub joined: bool,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct OwnerReceipt {
    pub parts_bytes: usize,
    pub input_arc_bytes: usize,
    pub completion_bytes: usize,
    pub closure_bytes: usize,
    pub owner_value_bytes: usize,
    pub authority_box_bytes: usize,
    pub authority_value_bytes: usize,
    pub stop_arc_layout_estimate: usize,
    pub joined: bool,
}
impl<I: Send + Sync + 'static, O: Send + 'static> OwnedPreparation<I, O> {
    pub(crate) fn known_parts_bytes() -> usize {
        std::mem::size_of::<Parts<I, O>>()
    }
    pub(crate) fn known_owner_value_bytes() -> usize {
        std::mem::size_of::<Self>()
    }
    pub(crate) fn known_capture_layout<C>() -> usize {
        std::mem::size_of::<WorkerCapture<I, O, C>>()
    }
    pub(crate) fn known_final_receipt_bytes() -> usize {
        std::mem::size_of::<FinalReceipt<O>>()
    }
    pub(crate) fn known_worker_capture_bytes<
        F: FnOnce(&I, &AtomicBool) -> Outcome<O> + Send + 'static,
    >(
        _: &F,
    ) -> usize {
        std::mem::size_of::<WorkerCapture<I, O, F>>()
    }
    pub(crate) fn known_stop_arc_bytes() -> Option<usize> {
        let align = std::mem::align_of::<usize>();
        Some(
            std::mem::size_of::<AtomicBool>()
                .checked_add(2 * std::mem::size_of::<usize>())?
                .checked_add(align - 1)?
                / align
                * align,
        )
    }
    pub(crate) fn known_job_bytes<F: FnOnce(&I, &AtomicBool) -> Outcome<O> + Send + 'static>(
        run: &F,
    ) -> Option<usize> {
        // Known Rust layouts only: Parts Box, one channel completion, final receipt,
        // named capture and AtomicBool Arc. Allocator/channel/thread internals remain unknown.
        Self::known_parts_bytes()
            .checked_add(Self::known_completion_bytes())?
            .checked_add(Self::known_final_receipt_bytes())?
            .checked_add(Self::known_worker_capture_bytes(run))?
            .checked_add(Self::known_stop_arc_bytes()?)
    }
    pub(crate) fn known_spawn_bytes<F: FnOnce(&I, &AtomicBool) -> Outcome<O> + Send + 'static>(
        run: &F,
    ) -> Option<usize> {
        std::mem::size_of::<I>()
            .checked_add(2 * std::mem::size_of::<usize>())?
            .checked_add(Self::known_job_bytes(run)?)
    }
    pub(crate) fn known_completion_bytes() -> usize {
        std::mem::size_of::<Completion<O>>()
    }
    pub(crate) fn spawn(
        input: Arc<I>,
        binding: Binding,
        run: impl FnOnce(&I, &AtomicBool) -> Outcome<O> + Send + 'static,
    ) -> Result<Self, Cause> {
        Self::spawn_with(
            input,
            binding,
            run,
            |f| {
                thread::Builder::new()
                    .name("r3viewer-prepare".into())
                    .spawn(f)
            },
            None,
        )
    }
    fn spawn_with(
        input: Arc<I>,
        binding: Binding,
        run: impl FnOnce(&I, &AtomicBool) -> Outcome<O> + Send + 'static,
        spawn: impl FnOnce(
            Box<dyn FnOnce() -> FinalReceipt<O> + Send>,
        ) -> std::io::Result<JoinHandle<FinalReceipt<O>>>,
        seed: Option<Outcome<O>>,
    ) -> Result<Self, Cause> {
        let token = NEXT
            .try_update(Ordering::AcqRel, Ordering::Acquire, |v| v.checked_add(1))
            .map_err(|_| Cause::Limit("preparation token"))?;
        if OCCUPIED
            .compare_exchange(0, token, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Err(if global_pending() {
                Cause::Pending
            } else {
                Cause::Busy
            });
        }
        let permit = Permit { token };
        let (sender, receiver) = mpsc::sync_channel(1);
        let stop = Arc::new(AtomicBool::new(false));
        if let Some(outcome) = seed {
            sender
                .try_send(Completion { binding, outcome })
                .map_err(|_| Cause::SendFull)?;
        }
        let worker_input = input.clone();
        let worker_stop = stop.clone();
        let mut parts = Box::new(Parts {
            input,
            binding,
            stop,
            handle: None,
            receiver,
            candidate: None,
            cause: None,
            closure_bytes: 0,
            _permit: permit,
        });
        let capture = WorkerCapture {
            input: worker_input,
            stop: worker_stop,
            binding,
            sender,
            run,
        };
        let worker = move || capture.execute();
        parts.closure_bytes = std::mem::size_of_val(&worker);
        let handle = spawn(Box::new(worker)).map_err(|_| Cause::Spawn)?;
        parts.handle = Some(handle);
        Ok(Self { parts: Some(parts) })
    }
    pub(crate) fn binding(&self) -> Option<Binding> {
        self.parts.as_ref().map(|p| p.binding)
    }
    pub(crate) fn input(&self) -> Option<&I> {
        self.parts.as_ref().map(|p| p.input.as_ref())
    }
    pub(crate) fn stop(&mut self) {
        if let Some(p) = self.parts.as_mut() {
            p.stop.store(true, Ordering::Release);
            p.cause.get_or_insert(Cause::Cancelled);
        }
    }
    pub(crate) fn is_finished(&self) -> bool {
        self.parts.as_ref().is_none_or(|p| p.finished())
    }
    pub(crate) fn poll(&mut self) -> Result<bool, Cause> {
        let p = self.parts.as_mut().ok_or(Cause::Missing)?;
        if p.candidate.is_none() {
            match p.receiver.try_recv() {
                Ok(c) => p.candidate = Some(c),
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    if !p.finished() {
                        p.cause.get_or_insert(Cause::Disconnected);
                    }
                }
            }
        }
        Ok(p.finished())
    }
    /// Does not join a live thread. Expected identity and terminal intent are checked after final receipt.
    pub(crate) fn take_validated(
        &mut self,
        expected: Binding,
        exit_intent: bool,
    ) -> Result<Option<(Outcome<O>, OwnerReceipt)>, Cause> {
        self.poll()?;
        if !self.is_finished() {
            return Ok(None);
        }
        let mut p = self.parts.take().ok_or(Cause::Missing)?;
        let handle = p.handle.take().ok_or(Cause::Missing)?;
        let final_receipt = handle.join().map_err(|_| Cause::Panic)?;
        if let Some(c) = final_receipt.cause {
            p.cause.get_or_insert(c);
        }
        if final_receipt.retained.is_some() {
            p.cause.get_or_insert(Cause::Disconnected);
        }
        if exit_intent {
            p.cause.get_or_insert(Cause::Cancelled);
        }
        if p.binding != expected {
            p.cause.get_or_insert(Cause::Stale);
        }
        if let Some(c) = p.cause {
            return Err(c);
        }
        let c = p
            .candidate
            .take()
            .or_else(|| p.receiver.try_recv().ok())
            .ok_or(Cause::Missing)?;
        if c.binding != expected {
            return Err(Cause::Stale);
        }
        let receipt = OwnerReceipt {
            parts_bytes: std::mem::size_of::<Parts<I, O>>(),
            input_arc_bytes: std::mem::size_of::<I>() + 2 * std::mem::size_of::<usize>(),
            completion_bytes: std::mem::size_of::<Completion<O>>(),
            closure_bytes: p.closure_bytes,
            owner_value_bytes: std::mem::size_of::<Self>(),
            authority_box_bytes: 0,
            authority_value_bytes: 0,
            stop_arc_layout_estimate: 3 * std::mem::size_of::<usize>(),
            joined: true,
        };
        Ok(Some((c.outcome, receipt)))
    }
    /// Caller supplies one absolute cleanup deadline. On expiry Drop moves the exact owner to global Pending.
    pub(crate) fn cleanup(mut self, deadline: Instant) -> Result<ReapReceipt, Cause> {
        let maximum = Instant::now()
            .checked_add(Duration::from_secs(2))
            .ok_or(Cause::Limit("cleanup deadline"))?;
        let deadline = deadline.min(maximum);
        self.stop();
        while !self.is_finished() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(1));
        }
        if !self.is_finished() {
            return Err(Cause::Pending);
        }
        self.parts.take().map(|p| p.reap()).ok_or(Cause::Missing)
    }
}
fn execute<I: Send + Sync + 'static, O: Send + 'static>(
    input: Arc<I>,
    stop: Arc<AtomicBool>,
    binding: Binding,
    sender: SyncSender<Completion<O>>,
    run: impl FnOnce(&I, &AtomicBool) -> Outcome<O>,
) -> FinalReceipt<O> {
    let outcome = run(&input, &stop);
    let completion = Completion { binding, outcome };
    match sender.try_send(completion) {
        Ok(()) => FinalReceipt {
            cause: None,
            retained: None,
        },
        Err(mpsc::TrySendError::Full(c)) => FinalReceipt {
            cause: Some(Cause::SendFull),
            retained: Some(c),
        },
        Err(mpsc::TrySendError::Disconnected(c)) => FinalReceipt {
            cause: Some(Cause::Disconnected),
            retained: Some(c),
        },
    }
}
pub(crate) fn global_pending() -> bool {
    PENDING.lock().unwrap_or_else(|e| e.into_inner()).is_some()
}
pub(crate) fn global_busy() -> bool {
    OCCUPIED.load(Ordering::Acquire) != 0
}
/// Retry only physical reap; never creates a task and never promotes its retained result.
pub(crate) fn retry_pending() -> Result<Option<ReapReceipt>, Cause> {
    let task = {
        let mut slot = PENDING.lock().map_err(|_| Cause::Poisoned)?;
        if slot.as_ref().is_none_or(|p| !p.finished()) {
            return Ok(None);
        }
        slot.take()
    };
    Ok(task.map(|task| task.reap()))
}

/// Closed typed owner: this enum, including all input/output leases, survives Pending.
pub(crate) enum ActiveOwner<DI: Send + Sync + 'static, DO: Send + 'static> {
    Source(crate::source_preparation::SourcePreparationOwner),
    Document(OwnedPreparation<DI, DO>),
}
enum AuthorityState<DI: Send + Sync + 'static, DO: Send + 'static> {
    Idle,
    Running(ActiveOwner<DI, DO>),
    Pending(ActiveOwner<DI, DO>),
}
pub(crate) enum PreparedCompletion<DO> {
    Source(
        Outcome<crate::source_preparation::SourcePrepared>,
        OwnerReceipt,
    ),
    Document(Outcome<DO>, OwnerReceipt),
}
struct AuthorityStorage<DI: Send + Sync + 'static, DO: Send + 'static> {
    phase: AuthorityState<DI, DO>,
    metadata: Option<crate::document_budget::Lease>,
}
impl<DI: Send + Sync + 'static, DO: Send + 'static> PendingTask for AuthorityStorage<DI, DO> {
    fn finished(&self) -> bool {
        self.phase.finished()
    }
    fn reap(self: Box<Self>) -> ReapReceipt {
        let Self { phase, metadata } = *self;
        let receipt = match phase {
            AuthorityState::Running(o) | AuthorityState::Pending(o) => o.reap(),
            AuthorityState::Idle => unreachable!(),
        };
        drop(metadata);
        receipt
    }
}
pub(crate) struct PreparationAuthority<DI: Send + Sync + 'static, DO: Send + 'static> {
    state: Option<Box<AuthorityStorage<DI, DO>>>,
}
impl<DI: Send + Sync + 'static, DO: Send + 'static> Default for PreparationAuthority<DI, DO> {
    fn default() -> Self {
        Self {
            state: Some(Box::new(AuthorityStorage {
                phase: AuthorityState::Idle,
                metadata: None,
            })),
        }
    }
}
impl<DI: Send + Sync + 'static, DO: Send + 'static> ActiveOwner<DI, DO> {
    fn finished(&self) -> bool {
        match self {
            Self::Source(o) => o.is_finished(),
            Self::Document(o) => o.is_finished(),
        }
    }
    fn stop(&mut self) {
        match self {
            Self::Source(o) => o.stop(),
            Self::Document(o) => o.stop(),
        }
    }
    fn poll(&mut self) -> Result<bool, Cause> {
        match self {
            Self::Source(o) => o.poll(),
            Self::Document(o) => o.poll(),
        }
    }
    fn reap(self) -> ReapReceipt {
        match self {
            Self::Source(mut o) => o.parts.take().expect("retained source").reap(),
            Self::Document(mut o) => o.parts.take().expect("retained document").reap(),
        }
    }
}
impl<DI: Send + Sync + 'static, DO: Send + 'static> PendingTask for AuthorityState<DI, DO> {
    fn finished(&self) -> bool {
        match self {
            Self::Idle => true,
            Self::Running(o) | Self::Pending(o) => o.finished(),
        }
    }
    fn reap(self: Box<Self>) -> ReapReceipt {
        match *self {
            Self::Running(o) | Self::Pending(o) => o.reap(),
            Self::Idle => unreachable!("only a live typed owner enters global Pending"),
        }
    }
}
impl<DI: Send + Sync + 'static, DO: Send + 'static> Drop for PreparationAuthority<DI, DO> {
    fn drop(&mut self) {
        if let Some(mut state) = self.state.take() {
            match &mut state.phase {
                AuthorityState::Idle => return,
                AuthorityState::Running(o) | AuthorityState::Pending(o) => o.stop(),
            }
            if state.finished() {
                state.reap();
            } else {
                let old = std::mem::replace(&mut state.phase, AuthorityState::Idle);
                state.phase = match old {
                    AuthorityState::Running(o) | AuthorityState::Pending(o) => {
                        AuthorityState::Pending(o)
                    }
                    AuthorityState::Idle => unreachable!(),
                };
                let mut slot = PENDING.lock().unwrap_or_else(|e| e.into_inner());
                assert!(slot.is_none(), "single occupied authority");
                *slot = Some(state); // Same Box allocated before spawn; the complete typed enum remains owned.
            }
        }
    }
}
impl<DI: Send + Sync + 'static, DO: Send + 'static> PreparationAuthority<DI, DO> {
    pub(crate) fn known_authority_bytes() -> Option<usize> {
        std::mem::size_of::<AuthorityStorage<DI, DO>>().checked_add(std::mem::size_of::<Self>())
    }
    /// The single existing Authority Box is charged once and travels intact into Pending.
    pub(crate) fn in_arena(arena: &crate::document_budget::ActorArena) -> Result<Self, Cause> {
        let bytes = std::mem::size_of::<AuthorityStorage<DI, DO>>()
            .checked_add(std::mem::size_of::<Self>())
            .ok_or(Cause::Limit("authority metadata"))?;
        let metadata = arena
            .reserve(bytes)
            .map_err(|_| Cause::Limit("authority Arena"))?;
        Ok(Self {
            state: Some(Box::new(AuthorityStorage {
                phase: AuthorityState::Idle,
                metadata: Some(metadata),
            })),
        })
    }
    pub(crate) fn actor_arena(&self) -> Option<&crate::document_budget::ActorArena> {
        self.state
            .as_ref()?
            .metadata
            .as_ref()
            .map(crate::document_budget::Lease::arena)
    }
    fn authority_receipt(mut r: OwnerReceipt) -> OwnerReceipt {
        r.authority_box_bytes = std::mem::size_of::<AuthorityStorage<DI, DO>>();
        r.authority_value_bytes = std::mem::size_of::<Self>();
        r
    }
    fn idle_state(&mut self) -> Result<&mut AuthorityState<DI, DO>, Cause> {
        let s = self
            .state
            .as_deref_mut()
            .map(|s| &mut s.phase)
            .ok_or(Cause::Missing)?;
        match s {
            AuthorityState::Idle => Ok(s),
            AuthorityState::Running(_) => Err(Cause::Busy),
            AuthorityState::Pending(_) => Err(Cause::Pending),
        }
    }
    pub(crate) fn start_source_owner(
        &mut self,
        factory: impl FnOnce() -> Result<crate::source_preparation::SourcePreparationOwner, Cause>,
    ) -> Result<(), Cause> {
        let state = self.idle_state()?;
        let o = factory()?;
        *state = AuthorityState::Running(ActiveOwner::Source(o));
        Ok(())
    }
    /// b1's closed factory will call this; b0 does not manufacture a Document result.
    pub(crate) fn start_document_owner(
        &mut self,
        factory: impl FnOnce() -> Result<OwnedPreparation<DI, DO>, Cause>,
    ) -> Result<(), Cause> {
        let state = self.idle_state()?;
        let o = factory()?;
        *state = AuthorityState::Running(ActiveOwner::Document(o));
        Ok(())
    }
    pub(crate) fn poll(&mut self) -> Result<bool, Cause> {
        match self
            .state
            .as_deref_mut()
            .map(|s| &mut s.phase)
            .ok_or(Cause::Missing)?
        {
            AuthorityState::Running(o) | AuthorityState::Pending(o) => o.poll(),
            AuthorityState::Idle => Err(Cause::Missing),
        }
    }
    pub(crate) fn take_validated(
        &mut self,
        expected: Binding,
        exit_intent: bool,
    ) -> Result<Option<PreparedCompletion<DO>>, Cause> {
        let state = self
            .state
            .as_deref_mut()
            .map(|s| &mut s.phase)
            .ok_or(Cause::Missing)?;
        let owner = match state {
            AuthorityState::Running(o) => o,
            AuthorityState::Pending(_) => return Err(Cause::Pending),
            AuthorityState::Idle => return Err(Cause::Missing),
        };
        if !owner.poll()? {
            return Ok(None);
        }
        let result = match owner {
            ActiveOwner::Source(o) => o
                .take_validated(expected, exit_intent)
                .map(|r| r.map(|(v, r)| PreparedCompletion::Source(v, Self::authority_receipt(r)))),
            ActiveOwner::Document(o) => o.take_validated(expected, exit_intent).map(|r| {
                r.map(|(v, r)| PreparedCompletion::Document(v, Self::authority_receipt(r)))
            }),
        };
        *state = AuthorityState::Idle;
        result
    }
    pub(crate) fn stop(&mut self) {
        if let Some(AuthorityState::Running(o) | AuthorityState::Pending(o)) =
            self.state.as_deref_mut().map(|s| &mut s.phase)
        {
            o.stop();
        }
    }
    pub(crate) fn is_finished(&self) -> bool {
        self.state.as_deref().is_none_or(PendingTask::finished)
    }
    pub(crate) fn cleanup(&mut self, deadline: Instant) -> Result<ReapReceipt, Cause> {
        let maximum = Instant::now()
            .checked_add(Duration::from_secs(2))
            .ok_or(Cause::Limit("cleanup deadline"))?;
        let deadline = deadline.min(maximum);
        self.stop();
        while !self.is_finished() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(1));
        }
        let state = self
            .state
            .as_deref_mut()
            .map(|s| &mut s.phase)
            .ok_or(Cause::Missing)?;
        let old = std::mem::replace(state, AuthorityState::Idle);
        match old {
            AuthorityState::Running(o) | AuthorityState::Pending(o) => {
                if o.finished() {
                    Ok(o.reap())
                } else {
                    *state = AuthorityState::Pending(o);
                    Err(Cause::Pending)
                }
            }
            AuthorityState::Idle => Err(Cause::Missing),
        }
    }
    pub(crate) fn retry_cleanup(&mut self) -> Result<Option<ReapReceipt>, Cause> {
        let state = self
            .state
            .as_deref_mut()
            .map(|s| &mut s.phase)
            .ok_or(Cause::Missing)?;
        match state {
            AuthorityState::Pending(o) if o.finished() => {
                let old = std::mem::replace(state, AuthorityState::Idle);
                if let AuthorityState::Pending(o) = old {
                    Ok(Some(o.reap()))
                } else {
                    unreachable!()
                }
            }
            AuthorityState::Pending(_) => Ok(None),
            _ => Err(Cause::Missing),
        }
    }
}

#[cfg(test)]
#[path = "preparation_owner_tests.rs"]
mod tests;
