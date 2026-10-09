//! Real HASH/SOURCE factories on the same typed authority. No preparation on the UI thread.
#![allow(dead_code)] // Private b0 library; routing/pump are deliberately not adopted here.
use crate::{
    document_model::Document,
    document_source::DocumentSource,
    input::SourceOrigin,
    preparation_owner::{Binding, Cause, Outcome, OwnedPreparation, PreparationAuthority},
    presentation::{PrepareError, SafeTextModel, SourceAllocationReceipt},
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
static NEXT_INPUT: AtomicU64 = AtomicU64::new(1);
pub(crate) struct SourcePreparationInput {
    original: Arc<String>,
    origin: SourceOrigin,
    generation: u64,
    request: u64,
    input_identity: u64,
    document: Option<Arc<Document>>,
}
impl SourcePreparationInput {
    pub(crate) fn new(
        original: Arc<String>,
        origin: SourceOrigin,
        generation: u64,
        request: u64,
    ) -> Result<Arc<Self>, Cause> {
        if original.len() > 134217728 || original.capacity() > 134217728 {
            return Err(Cause::Limit("Source original128MiB"));
        }
        let input_identity = NEXT_INPUT
            .try_update(Ordering::AcqRel, Ordering::Acquire, |v| v.checked_add(1))
            .map_err(|_| Cause::Limit("input identity"))?;
        Ok(Arc::new(Self {
            original,
            origin,
            generation,
            request,
            input_identity,
            document: None,
        }))
    }
    pub(crate) fn binding(&self, revision: u64) -> Binding {
        Binding {
            input: self.input_identity,
            generation: self.generation,
            request: self.request,
            revision,
            digest: None,
        }
    }
    pub(crate) fn original(&self) -> &Arc<String> {
        &self.original
    }
    pub(crate) fn origin(&self) -> &SourceOrigin {
        &self.origin
    }
    pub(crate) fn origin_capacity(&self) -> usize {
        match &self.origin {
            SourceOrigin::LocalFile(v) => v.capacity(),
            _ => 0,
        }
    }
}
/// Constructor and fields are private: a digest cannot be supplied by the caller.
pub(crate) struct PreparedSourceContext {
    input: Arc<SourcePreparationInput>,
    source: Arc<DocumentSource>,
}
impl PreparedSourceContext {
    fn checked(
        input: Arc<SourcePreparationInput>,
        source: Arc<DocumentSource>,
    ) -> Result<Arc<Self>, Cause> {
        if !Arc::ptr_eq(&input.original, &source.original_handle())
            || input.origin != *source.origin()
            || input.generation != source.generation()
            || input.request != source.request()
        {
            return Err(Cause::Stale);
        }
        Ok(Arc::new(Self { input, source }))
    }
    pub(crate) fn from_document(document: Arc<Document>) -> Result<Arc<Self>, Cause> {
        let source = document.source_handle();
        let mut input = SourcePreparationInput::new(
            source.original_handle(),
            source.origin().clone(),
            source.generation(),
            source.request(),
        )?;
        Arc::get_mut(&mut input).expect("new input unique").document = Some(document);
        Self::checked(input, source)
    }
    pub(crate) fn source(&self) -> &Arc<DocumentSource> {
        &self.source
    }
    pub(crate) fn is_document_context(&self) -> bool {
        self.input.document.is_some()
    }
    pub(crate) fn input(&self) -> &Arc<SourcePreparationInput> {
        &self.input
    }
    pub(crate) fn digest(&self) -> &[u8; 32] {
        self.source.hash()
    }
    pub(crate) fn binding(&self, revision: u64) -> Binding {
        Binding {
            digest: Some(*self.digest()),
            ..self.input.binding(revision)
        }
    }
    pub(crate) fn input_layout_bytes(&self) -> usize {
        std::mem::size_of::<SourcePreparationInput>() + 2 * std::mem::size_of::<usize>()
    }
    pub(crate) fn context_layout_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>()
    }
    pub(crate) fn source_layout_bytes(&self) -> usize {
        std::mem::size_of::<DocumentSource>() + 2 * std::mem::size_of::<usize>()
    }
    pub(crate) fn origin_capacity(&self) -> usize {
        match self.source.origin() {
            SourceOrigin::LocalFile(v) => v.capacity(),
            _ => 0,
        }
    }
}
pub(crate) struct PreparedSourceModel {
    model: SafeTextModel,
    context: Arc<PreparedSourceContext>,
    allocation: SourceAllocationReceipt,
}
impl PreparedSourceModel {
    pub(crate) fn model(&self) -> &SafeTextModel {
        &self.model
    }
    pub(crate) fn context(&self) -> &Arc<PreparedSourceContext> {
        &self.context
    }
    pub(crate) fn output_box_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
    pub(crate) fn allocation(&self) -> SourceAllocationReceipt {
        self.allocation
    }
}
pub(crate) enum SourcePrepared {
    Hash(Arc<PreparedSourceContext>),
    Model(Box<PreparedSourceModel>),
    ScopedModel(Box<PreparedSourceModel>, crate::source_shared::SourceScope),
    Viewport(Box<crate::source_view_preparation::SourceViewportPrepared>),
}
pub(crate) enum SourcePreparationTaskInput {
    Build {
        input: Arc<SourcePreparationInput>,
        scope: Option<crate::source_shared::SourceScope>,
        metadata: Option<crate::document_budget::Lease>,
    },
    Viewport(crate::source_view_preparation::SourceViewportInput),
}
pub(crate) type SourcePreparationOwner =
    OwnedPreparation<SourcePreparationTaskInput, SourcePrepared>;
pub(crate) fn start_hash<DI: Send + Sync + 'static, DO: Send + 'static>(
    input: Arc<SourcePreparationInput>,
    revision: u64,
    authority: &mut PreparationAuthority<DI, DO>,
) -> Result<(), Cause> {
    let binding = input.binding(revision);
    let context = input.clone();
    authority.start_source_owner(move || {
        OwnedPreparation::spawn(
            Arc::new(SourcePreparationTaskInput::Build {
                input,
                scope: None,
                metadata: None,
            }),
            binding,
            move |i, stop| {
                let SourcePreparationTaskInput::Build { input: i, .. } = i else {
                    return Outcome::Failed(Cause::Stale);
                };
                match DocumentSource::new(
                    i.original.clone(),
                    i.origin.clone(),
                    i.generation,
                    i.request,
                    stop,
                ) {
                    Ok(s) => match PreparedSourceContext::checked(context, Arc::new(s)) {
                        Ok(c) => Outcome::Ready(SourcePrepared::Hash(c)),
                        Err(e) => Outcome::Failed(e),
                    },
                    Err(e) => match e.kind {
                        crate::document_model::ErrorKind::Cancelled => Outcome::Cancelled,
                        _ => Outcome::Failed(Cause::Limit("DocumentSource")),
                    },
                }
            },
        )
    })
}
pub(crate) fn start_source<DI: Send + Sync + 'static, DO: Send + 'static>(
    context: Arc<PreparedSourceContext>,
    revision: u64,
    authority: &mut PreparationAuthority<DI, DO>,
) -> Result<(), Cause> {
    let binding = context.binding(revision);
    let input = context.input.clone();
    authority.start_source_owner(move || {
        OwnedPreparation::spawn(
            Arc::new(SourcePreparationTaskInput::Build {
                input,
                scope: None,
                metadata: None,
            }),
            binding,
            move |_, stop| prepare_source(context, stop),
        )
    })
}
pub(crate) fn prepare_source(
    context: Arc<PreparedSourceContext>,
    stop: &AtomicBool,
) -> Outcome<SourcePrepared> {
    match SafeTextModel::from_shared(context.input.original.clone(), stop) {
        Ok(model) => match model.allocation_receipt() {
            Some(allocation) => {
                Outcome::Ready(SourcePrepared::Model(Box::new(PreparedSourceModel {
                    model,
                    context,
                    allocation,
                })))
            }
            None => Outcome::Limit("Source capacity arithmetic"),
        },
        Err(PrepareError::Cancelled) => Outcome::Cancelled,
        Err(PrepareError::Limit(v)) => Outcome::Limit(v),
    }
}

/// Common Source build retains the charged original scope under the same kernel, including Pending.
pub(crate) fn start_scoped_source<DI: Send + Sync + 'static, DO: Send + 'static>(
    scope: crate::source_shared::SourceScope,
    revision: u64,
    authority: &mut PreparationAuthority<DI, DO>,
) -> Result<(), Cause> {
    if !authority
        .actor_arena()
        .is_some_and(|a| a.same(scope.arena()))
    {
        return Err(Cause::Stale);
    }
    let context = scope.context().clone();
    let binding = scope.binding(revision);
    let input = context.input().clone();
    let run = move |i: &SourcePreparationTaskInput, stop: &AtomicBool| {
        let SourcePreparationTaskInput::Build {
            scope: Some(retained),
            ..
        } = i
        else {
            return Outcome::Failed(Cause::Stale);
        };
        let retained = match retained.try_alias() {
            Ok(s) => s,
            Err(e) => return Outcome::Failed(e),
        };
        match prepare_source(context, stop) {
            Outcome::Ready(SourcePrepared::Model(model)) => {
                Outcome::Ready(SourcePrepared::ScopedModel(model, retained))
            }
            Outcome::Cancelled => Outcome::Cancelled,
            Outcome::Limit(s) => Outcome::Limit(s),
            Outcome::Failed(e) => Outcome::Failed(e),
            _ => Outcome::Failed(Cause::Stale),
        }
    };
    let bytes = SourcePreparationOwner::known_spawn_bytes(&run)
        .and_then(|n| n.checked_sub(std::mem::size_of::<crate::source_shared::SourceScope>()))
        .ok_or(Cause::Limit("Source Build arithmetic"))?;
    let metadata = scope
        .arena()
        .reserve(bytes)
        .map_err(|_| Cause::Limit("Source Build metadata"))?;
    authority.start_source_owner(move || {
        OwnedPreparation::spawn(
            Arc::new(SourcePreparationTaskInput::Build {
                input,
                scope: Some(scope),
                metadata: Some(metadata),
            }),
            binding,
            run,
        )
    })
}
