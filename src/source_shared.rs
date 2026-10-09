//! Charged ownership of the same Source Box and its original context, before the session pump.
#![allow(dead_code)]
use crate::{
    document_budget::{ActorArena, Lease},
    document_consumer_budget::Charge,
    document_model::Document,
    document_view::DocumentView,
    preparation_owner::{Binding, Cause, Outcome, PreparationAuthority, PreparedCompletion},
    presentation::{SafeTextModel, SourceAllocationReceipt},
    source_preparation::{PreparedSourceContext, PreparedSourceModel, SourcePrepared},
};
use std::{
    mem::size_of,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
fn reserve(arena: &ActorArena, bytes: usize) -> Result<Lease, Cause> {
    arena
        .reserve(bytes)
        .map_err(|_| Cause::Limit("Source ActorArena"))
}
enum ScopeOrigin {
    PreDocument,
    Document {
        document: Arc<Document>,
        metadata: Arc<Charge>,
        binding: Binding,
    },
}
struct ScopeAllocation {
    context: Arc<PreparedSourceContext>,
    arena: ActorArena,
    origin: ScopeOrigin,
    _metadata: Lease,
}
pub(crate) struct SourceScope {
    inner: Arc<ScopeAllocation>,
    _handle: Lease,
}
impl SourceScope {
    pub(crate) fn from_hash(
        context: Arc<PreparedSourceContext>,
        arena: &ActorArena,
    ) -> Result<Self, Cause> {
        if Arc::strong_count(&context) != 1 || context.is_document_context() {
            return Err(Cause::Stale);
        }
        let bytes = context
            .input_layout_bytes()
            .checked_add(context.context_layout_bytes())
            .and_then(|n| n.checked_add(context.source_layout_bytes()))
            .and_then(|n| n.checked_add(context.origin_capacity()))
            .and_then(|n| n.checked_add(context.input().origin_capacity()))
            .ok_or(Cause::Limit("Source context arithmetic"))?;
        Self::make(context, arena, ScopeOrigin::PreDocument, bytes)
    }
    pub(crate) fn from_view(view: &DocumentView, arena: &ActorArena) -> Result<Self, Cause> {
        let document = view.inspect_document();
        let context = view.inspect_source_context();
        let metadata = view.inspect_source_metadata();
        if !document.actor_arena().same(arena)
            || !view.budget().same(metadata.budget())
            || !view.budget().belongs_to(document)
            || !Arc::ptr_eq(context.source(), &document.source_handle())
        {
            return Err(Cause::Stale);
        }
        Self::make(
            context.clone(),
            arena,
            ScopeOrigin::Document {
                document: document.clone(),
                metadata: metadata.clone(),
                binding: view.render_binding(),
            },
            0,
        )
    }
    fn make(
        context: Arc<PreparedSourceContext>,
        arena: &ActorArena,
        origin: ScopeOrigin,
        context_bytes: usize,
    ) -> Result<Self, Cause> {
        let bytes = size_of::<ScopeAllocation>()
            .checked_add(2 * size_of::<usize>())
            .and_then(|n| n.checked_add(context_bytes))
            .ok_or(Cause::Limit("Source scope arithmetic"))?;
        let metadata = reserve(arena, bytes)?;
        let handle = reserve(arena, size_of::<Self>())?;
        Ok(Self {
            inner: Arc::new(ScopeAllocation {
                context,
                arena: arena.clone(),
                origin,
                _metadata: metadata,
            }),
            _handle: handle,
        })
    }
    pub(crate) fn try_alias(&self) -> Result<Self, Cause> {
        let handle = reserve(self.arena(), size_of::<Self>())?;
        Ok(Self {
            inner: self.inner.clone(),
            _handle: handle,
        })
    }
    pub(crate) fn context(&self) -> &Arc<PreparedSourceContext> {
        &self.inner.context
    }
    pub(crate) fn arena(&self) -> &ActorArena {
        &self.inner.arena
    }
    pub(crate) fn binding(&self, revision: u64) -> Binding {
        self.context().binding(revision)
    }
    pub(crate) fn retained_document_binding(&self) -> Option<Binding> {
        match &self.inner.origin {
            ScopeOrigin::Document { binding, .. } => Some(*binding),
            _ => None,
        }
    }
    fn same_scope(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
    fn validates(&self, model: &PreparedSourceModel, expected: Binding) -> bool {
        Arc::ptr_eq(self.context(), model.context())
            && self.binding(expected.revision) == expected
            && Arc::ptr_eq(
                model.context().input().original(),
                &model.model().original_handle(),
            )
    }
}
struct SharedSourceAllocation {
    model: Box<PreparedSourceModel>,
    scope: SourceScope,
    source_heap_bytes: usize,
    _metadata: Lease,
}
pub(crate) struct SharedSource {
    inner: Arc<SharedSourceAllocation>,
    _handle: Lease,
}
pub(crate) struct SourceLease {
    inner: Arc<SharedSourceAllocation>,
    _handle: Lease,
}
impl SharedSource {
    pub(crate) fn try_lease(&self) -> Result<SourceLease, Cause> {
        let handle = reserve(self.inner.scope.arena(), size_of::<SourceLease>())?;
        Ok(SourceLease {
            inner: self.inner.clone(),
            _handle: handle,
        })
    }
    pub(crate) fn model(&self) -> &SafeTextModel {
        self.inner.model.model()
    }
    pub(crate) fn model_box(&self) -> &PreparedSourceModel {
        &self.inner.model
    }
    pub(crate) fn context(&self) -> &Arc<PreparedSourceContext> {
        self.inner.scope.context()
    }
    pub(crate) fn source_heap_bytes(&self) -> usize {
        self.inner.source_heap_bytes
    }
    pub(crate) fn allocation(&self) -> SourceAllocationReceipt {
        self.inner.model.allocation()
    }
}
impl SourceLease {
    pub(crate) fn model(&self) -> &SafeTextModel {
        self.inner.model.model()
    }
    pub(crate) fn context(&self) -> &Arc<PreparedSourceContext> {
        self.inner.scope.context()
    }
    pub(crate) fn arena(&self) -> &ActorArena {
        self.inner.scope.arena()
    }
    pub(crate) fn binding(&self, revision: u64) -> Binding {
        self.context().binding(revision)
    }
    pub(crate) fn source_heap_bytes(&self) -> usize {
        self.inner.source_heap_bytes
    }
    pub(crate) fn try_alias(&self) -> Result<Self, Cause> {
        let handle = reserve(self.arena(), size_of::<Self>())?;
        Ok(Self {
            inner: self.inner.clone(),
            _handle: handle,
        })
    }
}
pub(crate) struct SharePending {
    cause: Cause,
    model: Box<PreparedSourceModel>,
    scope: SourceScope,
    expected: Binding,
}
impl SharePending {
    pub(crate) fn cause(&self) -> Cause {
        self.cause
    }
    pub(crate) fn model(&self) -> &PreparedSourceModel {
        &self.model
    }
    // The error transfers the same Box/Scope without an allocation after quota failure.
    #[allow(clippy::result_large_err)]
    pub(crate) fn retry(self, cancel: &AtomicBool) -> Result<SharedSource, Self> {
        adopt(self.model, self.scope, self.expected, cancel)
    }
}
// Keep the bounded128-byte rollback value inline; another Box would need a fallible quota reservation.
#[allow(clippy::result_large_err)]
fn adopt(
    model: Box<PreparedSourceModel>,
    scope: SourceScope,
    expected: Binding,
    cancel: &AtomicBool,
) -> Result<SharedSource, SharePending> {
    let reservations = (|| {
        if cancel.load(Ordering::Acquire) {
            return Err(Cause::Cancelled);
        }
        if !scope.validates(&model, expected) {
            return Err(Cause::Stale);
        }
        let source_heap_bytes = model
            .allocation()
            .known_heap_bytes
            .checked_add(model.output_box_bytes())
            .ok_or(Cause::Limit("Source retained allocation"))?;
        let metadata = reserve(
            scope.arena(),
            size_of::<SharedSourceAllocation>()
                .checked_sub(size_of::<SourceScope>())
                .and_then(|n| n.checked_add(2 * size_of::<usize>()))
                .ok_or(Cause::Limit("Source wrapper arithmetic"))?,
        )?;
        let handle = reserve(scope.arena(), size_of::<SharedSource>())?;
        if cancel.load(Ordering::Acquire) {
            return Err(Cause::Cancelled);
        }
        Ok((source_heap_bytes, metadata, handle))
    })();
    match reservations {
        Ok((source_heap_bytes, metadata, handle)) => Ok(SharedSource {
            inner: Arc::new(SharedSourceAllocation {
                model,
                scope,
                source_heap_bytes,
                _metadata: metadata,
            }),
            _handle: handle,
        }),
        Err(cause) => Err(SharePending {
            cause,
            model,
            scope,
            expected,
        }),
    }
}
pub(crate) enum Adoption {
    Waiting(SourceScope),
    Ready(SharedSource),
    Retained(SharePending),
}
/// Only this public-to-the-crate adoption path extracts a Model from the physically joined authority.
pub(crate) fn adopt_joined_source<DI: Send + Sync + 'static, DO: Send + 'static>(
    authority: &mut PreparationAuthority<DI, DO>,
    scope: SourceScope,
    expected: Binding,
    exit_intent: bool,
    cancel: &AtomicBool,
) -> Result<Adoption, Cause> {
    adopt_joined_source_with(authority, scope, expected, exit_intent, cancel, || {})
}
fn adopt_joined_source_with<DI: Send + Sync + 'static, DO: Send + 'static>(
    authority: &mut PreparationAuthority<DI, DO>,
    scope: SourceScope,
    expected: Binding,
    exit_intent: bool,
    cancel: &AtomicBool,
    after_join: impl FnOnce(),
) -> Result<Adoption, Cause> {
    if !authority
        .actor_arena()
        .is_some_and(|a| a.same(scope.arena()))
    {
        return Err(Cause::Stale);
    }
    match authority.take_validated(expected, exit_intent)? {
        None => Ok(Adoption::Waiting(scope)),
        Some(PreparedCompletion::Source(
            Outcome::Ready(SourcePrepared::ScopedModel(model, original_scope)),
            receipt,
        )) if receipt.joined => {
            after_join();
            if !scope.same_scope(&original_scope) {
                return Ok(Adoption::Retained(SharePending {
                    cause: Cause::Stale,
                    model,
                    scope: original_scope,
                    expected,
                }));
            }
            drop(scope);
            Ok(match adopt(model, original_scope, expected, cancel) {
                Ok(shared) => Adoption::Ready(shared),
                Err(pending) => Adoption::Retained(pending),
            })
        }
        Some(PreparedCompletion::Source(Outcome::Cancelled, _)) => Err(Cause::Cancelled),
        Some(PreparedCompletion::Source(Outcome::Limit(s), _)) => Err(Cause::Limit(s)),
        Some(PreparedCompletion::Source(Outcome::Failed(e), _)) => Err(e),
        _ => Err(Cause::Stale),
    }
}
#[cfg(test)]
#[path = "source_shared_tests.rs"]
mod tests;
