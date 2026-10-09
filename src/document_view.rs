//! Private b1 library: real Flow/text preparation, typed final commit, no session/pump/renderer.
#![allow(dead_code)]
use crate::document_presentation_anchor::PresentationAnchor;
use crate::{
    document_consumer_budget::{Charge, ChargedVec, ConsumerBudget},
    document_consumer_profile::ConsumerProfile,
    document_flow::{FlowHandle, FlowIndex, FoldOverride, SharedFlow},
    document_layout::{self, Geometry, LayoutWindow, ReferenceReason, SequenceSlot, WindowBuilder},
    document_model::{Document, NONE},
    document_navigation::{NavigationStorage, Navigator, State},
    document_text::{DocumentWork, Error, IndexProfile},
    preparation_owner::{Binding, Cause, Outcome, OwnedPreparation, PreparationAuthority},
    source_preparation::{PreparedSourceContext, SourcePreparationInput},
};
use std::{
    mem::size_of,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};
static NEXT_REVISION: AtomicU64 = AtomicU64::new(1);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RawAnchor {
    pub block: u32,
    pub run: u32,
    pub byte: usize,
}
impl RawAnchor {
    fn state(s: &State) -> Self {
        Self {
            block: s.block(),
            run: s.run(),
            byte: s.raw_byte(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LayoutAnchor {
    pub raw: RawAnchor,
    pub projection_part: u8,
}
impl LayoutAnchor {
    fn state(s: &State) -> Self {
        Self {
            raw: RawAnchor::state(s),
            projection_part: s.projection_part(),
        }
    }
    fn physical(raw: RawAnchor) -> Self {
        Self {
            raw,
            projection_part: 0,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Operation {
    Initial,
    Home,
    End,
    Resize,
    Presentation,
    Horizontal { left: usize },
    Seek(RawAnchor),
    Continue(LayoutAnchor),
    Fold { block: u32, expanded: bool },
    Line { delta: i32 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DocumentKey {
    pub binding: Binding,
    pub geometry: Geometry,
    pub fold_revision: u64,
    pub operation: Operation,
}
struct ViewStorage {
    _reference_current: ChargedVec<u8>,
    _reference_candidate: ChargedVec<u8>,
    _scratch: ChargedVec<u8>,
    _roles: ChargedVec<u32>,
    _fixed: Charge,
}
impl ViewStorage {
    fn new(budget: &ConsumerBudget) -> Result<Arc<Self>, Error> {
        let fixed = budget.reserve(
            size_of::<Self>()
                .checked_add(2 * size_of::<usize>())
                .and_then(|n| n.checked_add(size_of::<DocumentView>()))
                .ok_or(Error::Budget)?,
        )?;
        Ok(Arc::new(Self {
            _reference_current: ChargedVec::new(budget, 4096)?,
            _reference_candidate: ChargedVec::new(budget, 4096)?,
            _scratch: ChargedVec::new(budget, 8192)?,
            _roles: ChargedVec::new(budget, 65)?,
            _fixed: fixed,
        }))
    }
}
pub(crate) struct DocumentPreparationInput {
    document: Arc<Document>,
    provider_gate: Option<Arc<crate::document_provider_gate::ProviderGate>>,
    budget: ConsumerBudget,
    source: Arc<PreparedSourceContext>,
    storage: Arc<ViewStorage>,
    key: DocumentKey,
    old_flow: Option<FlowHandle>,
    old_slot: Option<Arc<SequenceSlot>>,
    anchor: Option<LayoutAnchor>,
    presentation: Option<PresentationAnchor>,
    old_window_bytes: usize,
    _metadata: Charge,
    _source_metadata: Arc<Charge>,
    funding: KernelFunding,
}
impl DocumentPreparationInput {
    pub(crate) fn metadata_bytes(&self) -> usize {
        self._metadata.bytes()
    }
    pub(crate) fn key(&self) -> DocumentKey {
        self.key
    }
    pub(crate) fn budget(&self) -> &ConsumerBudget {
        &self.budget
    }
    pub(crate) fn document(&self) -> &Arc<Document> {
        &self.document
    }
}
pub(crate) struct PreparedViewport {
    input: Arc<DocumentPreparationInput>,
    new_flow: Option<SharedFlow>,
    selection: Option<State>,
    navigation: Option<NavigationStorage>,
    layout: LayoutWindow,
    _metadata: Charge,
}
pub(crate) struct PreparedPending {
    input: Arc<DocumentPreparationInput>,
    cause: Error,
    _metadata: Charge,
}
pub(crate) enum PreparedState {
    Viewport(Box<PreparedViewport>),
    PositionPending(Box<PreparedPending>),
}
pub(crate) type DocumentPreparationAuthority =
    PreparationAuthority<DocumentPreparationInput, PreparedState>;
pub(crate) struct DocumentView {
    document: Arc<Document>,
    provider_gate: Option<Arc<crate::document_provider_gate::ProviderGate>>,
    budget: ConsumerBudget,
    source: Arc<PreparedSourceContext>,
    storage: Arc<ViewStorage>,
    _source_metadata: Arc<Charge>,
    navigator: Navigator,
    layout: LayoutWindow,
    geometry: Geometry,
    fold_revision: u64,
    presentation: Option<PresentationAnchor>,
    expected: Option<DocumentKey>,
    render_binding: Binding,
}
fn map_cause(e: Error) -> Cause {
    match e {
        Error::Cancelled => Cause::Cancelled,
        Error::Invalid => Cause::Stale,
        Error::Work => Cause::Limit("DocumentWork"),
        Error::Budget => Cause::Limit("ConsumerBudget"),
        Error::ProvidersBusy => Cause::Limit("Provider slots"),
        Error::ScopePending => Cause::Limit("Inspectable scope"),
        Error::UnsupportedRun(_) => Cause::Limit("DocumentRun"),
    }
}
fn revision() -> Result<u64, Cause> {
    NEXT_REVISION
        .try_update(Ordering::AcqRel, Ordering::Acquire, |n| n.checked_add(1))
        .map_err(|_| Cause::Limit("Document revision"))
}
struct DocumentRunner {
    context: Arc<DocumentPreparationInput>,
}
impl DocumentRunner {
    fn execute(self, stop: &AtomicBool) -> Outcome<PreparedState> {
        run_prepare(self.context, stop)
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum KernelFunding {
    Legacy,
    Actor,
    Shared,
}
fn input_metadata_bytes(has_flow: bool, funding: KernelFunding) -> Result<usize, Error> {
    let input = size_of::<DocumentPreparationInput>()
        .checked_sub(if has_flow { size_of::<FlowHandle>() } else { 0 })
        .and_then(|n| n.checked_add(2 * size_of::<usize>()))
        .ok_or(Error::Budget)?;
    if matches!(funding, KernelFunding::Shared) {
        return Ok(input);
    }
    type Owner = OwnedPreparation<DocumentPreparationInput, PreparedState>;
    // Preserve the historical scope of known OwnerReceipt fields. The returned thread packet/FinalReceipt
    // is not an additional guarantee in this legacy path; common Source charges its known Rust value separately.
    let mut kernel = Owner::known_parts_bytes()
        .checked_add(Owner::known_completion_bytes())
        .and_then(|n| n.checked_add(Owner::known_capture_layout::<DocumentRunner>()))
        .and_then(|n| n.checked_add(Owner::known_stop_arc_bytes()?))
        .and_then(|n| n.checked_add(Owner::known_owner_value_bytes()))
        .ok_or(Error::Budget)?;
    if matches!(funding, KernelFunding::Legacy) {
        kernel = kernel
            .checked_add(
                DocumentPreparationAuthority::known_authority_bytes().ok_or(Error::Budget)?,
            )
            .ok_or(Error::Budget)?;
    }
    input.checked_add(kernel).ok_or(Error::Budget)
}
fn input_charge(
    budget: &ConsumerBudget,
    has_flow: bool,
    funding: KernelFunding,
) -> Result<Charge, Error> {
    budget
        .reserve(input_metadata_bytes(has_flow, funding)?)
        .map_err(Into::into)
}
pub(crate) struct SharedDocumentDescriptor {
    input: Arc<DocumentPreparationInput>,
}
impl SharedDocumentDescriptor {
    pub(crate) fn key(&self) -> DocumentKey {
        self.input.key()
    }
    pub(crate) fn budget(&self) -> &ConsumerBudget {
        self.input.budget()
    }
    pub(crate) fn document(&self) -> &Arc<Document> {
        self.input.document()
    }
    pub(crate) fn metadata_bytes(&self) -> usize {
        self.input.metadata_bytes()
    }
}
/// Does not spawn. The scoped sum factory must fund its concrete I/O job before calling this in its worker.
pub(crate) fn prepare_shared_document(
    input: &SharedDocumentDescriptor,
    stop: &AtomicBool,
) -> Outcome<PreparedState> {
    run_prepare(input.input.clone(), stop)
}

impl DocumentView {
    pub(crate) fn initial_input(
        document: Arc<Document>,
        geometry: Geometry,
    ) -> Result<Arc<DocumentPreparationInput>, Cause> {
        Self::initial_with(document, geometry, KernelFunding::Legacy)
    }
    pub(crate) fn initial_input_in<DI: Send + Sync + 'static, DO: Send + 'static>(
        document: Arc<Document>,
        geometry: Geometry,
        authority: &PreparationAuthority<DI, DO>,
    ) -> Result<Arc<DocumentPreparationInput>, Cause> {
        if !authority
            .actor_arena()
            .is_some_and(|a| a.same(document.actor_arena()))
        {
            return Err(Cause::Stale);
        }
        Self::initial_with(document, geometry, KernelFunding::Actor)
    }
    pub(crate) fn initial_shared_input<DI: Send + Sync + 'static, DO: Send + 'static>(
        document: Arc<Document>,
        geometry: Geometry,
        authority: &PreparationAuthority<DI, DO>,
    ) -> Result<SharedDocumentDescriptor, Cause> {
        if !authority
            .actor_arena()
            .is_some_and(|a| a.same(document.actor_arena()))
        {
            return Err(Cause::Stale);
        }
        Ok(SharedDocumentDescriptor {
            input: Self::initial_with(document, geometry, KernelFunding::Shared)?,
        })
    }
    fn initial_with(
        document: Arc<Document>,
        geometry: Geometry,
        funding: KernelFunding,
    ) -> Result<Arc<DocumentPreparationInput>, Cause> {
        let budget = ConsumerBudget::new(&document).map_err(|_| Cause::Limit("ConsumerBudget"))?;
        let storage = ViewStorage::new(&budget).map_err(map_cause)?;
        let source_meta = size_of::<SourcePreparationInput>()
            .checked_add(size_of::<PreparedSourceContext>())
            .and_then(|n| n.checked_add(6 * size_of::<usize>()))
            .ok_or(Cause::Limit("Source context"))?;
        let path = match document.source().origin() {
            crate::input::SourceOrigin::LocalFile(p) => p.capacity(),
            _ => 0,
        };
        let source_metadata = Arc::new(
            budget
                .reserve(
                    source_meta
                        .checked_add(path)
                        .ok_or(Cause::Limit("Source context"))?,
                )
                .map_err(|_| Cause::Limit("Source context"))?,
        );
        let source = PreparedSourceContext::from_document(document.clone())?;
        let actual = source.input().origin_capacity();
        if actual > path {
            return Err(Cause::Limit("Source context actual capacity"));
        }
        let binding = source.binding(revision()?);
        let metadata = input_charge(&budget, false, funding).map_err(map_cause)?;
        Ok(Arc::new(DocumentPreparationInput {
            document,
            provider_gate: None,
            budget,
            source,
            storage,
            key: DocumentKey {
                binding,
                geometry,
                fold_revision: 0,
                operation: Operation::Initial,
            },
            old_flow: None,
            old_slot: None,
            anchor: None,
            presentation: None,
            old_window_bytes: 0,
            _metadata: metadata,
            _source_metadata: source_metadata,
            funding,
        }))
    }
    pub(crate) fn from_prepared(state: PreparedState) -> Result<Self, Cause> {
        let PreparedState::Viewport(v) = state else {
            return Err(Cause::Limit("Initial position pending"));
        };
        let PreparedViewport {
            input,
            new_flow,
            selection,
            navigation,
            layout,
            _metadata,
        } = *v;
        if input.key.operation != Operation::Initial {
            return Err(Cause::Stale);
        }
        let flow = new_flow.ok_or(Cause::Missing)?;
        let navigation = navigation.ok_or(Cause::Missing)?;
        let navigator = Navigator::from_storage(flow, selection, navigation);
        let presentation = input
            .presentation
            .clone()
            .or_else(|| layout.text().and_then(|w| w.presentation_row(0).cloned()));
        Ok(Self {
            document: input.document.clone(),
            provider_gate: input.provider_gate.clone(),
            budget: input.budget.clone(),
            source: input.source.clone(),
            storage: input.storage.clone(),
            _source_metadata: input._source_metadata.clone(),
            navigator,
            layout,
            geometry: input.key.geometry,
            fold_revision: input.key.fold_revision,
            presentation,
            expected: None,
            render_binding: input.key.binding,
        })
    }
    pub(crate) fn request(
        &mut self,
        operation: Operation,
        geometry: Geometry,
    ) -> Result<Arc<DocumentPreparationInput>, Cause> {
        self.request_with(operation, geometry, KernelFunding::Legacy)
    }
    pub(crate) fn request_in<DI: Send + Sync + 'static, DO: Send + 'static>(
        &mut self,
        operation: Operation,
        geometry: Geometry,
        authority: &PreparationAuthority<DI, DO>,
    ) -> Result<Arc<DocumentPreparationInput>, Cause> {
        if !authority
            .actor_arena()
            .is_some_and(|a| a.same(self.document.actor_arena()))
        {
            return Err(Cause::Stale);
        }
        self.request_with(operation, geometry, KernelFunding::Actor)
    }
    pub(crate) fn request_shared<DI: Send + Sync + 'static, DO: Send + 'static>(
        &mut self,
        operation: Operation,
        geometry: Geometry,
        authority: &PreparationAuthority<DI, DO>,
    ) -> Result<SharedDocumentDescriptor, Cause> {
        if !authority
            .actor_arena()
            .is_some_and(|a| a.same(self.document.actor_arena()))
        {
            return Err(Cause::Stale);
        }
        Ok(SharedDocumentDescriptor {
            input: self.request_with(operation, geometry, KernelFunding::Shared)?,
        })
    }
    fn request_with(
        &mut self,
        operation: Operation,
        geometry: Geometry,
        funding: KernelFunding,
    ) -> Result<Arc<DocumentPreparationInput>, Cause> {
        if operation == Operation::Initial {
            return Err(Cause::Stale);
        }
        let flow = self.navigator.share_flow().map_err(map_cause)?;
        let metadata = input_charge(&self.budget, true, funding).map_err(map_cause)?;
        let fold_revision = if matches!(operation, Operation::Fold { .. }) {
            self.fold_revision
                .checked_add(1)
                .ok_or(Cause::Limit("Fold revision"))?
        } else {
            self.fold_revision
        };
        let key = DocumentKey {
            binding: self.source.binding(revision()?),
            geometry,
            fold_revision,
            operation,
        };
        let mut anchor = self.navigator.current().map(LayoutAnchor::state);
        let mut operation = operation;
        let mut presentation = self.presentation.clone();
        if matches!(
            operation,
            Operation::Seek(_) | Operation::Continue(_) | Operation::Home | Operation::End
        ) {
            presentation = None;
        }
        if let Operation::Line { delta } = operation
            && delta > 0
            && geometry == self.geometry
            && let Some(w) = self.layout.text()
        {
            if let Some(a) = w
                .presentation_row(delta as usize)
                .or_else(|| w.presentation_next())
            {
                presentation = Some(a.clone());
                anchor = Some(a.layout_selection());
                operation = Operation::Presentation;
            } else if let Some(a) = w.row_anchor(delta as usize).or_else(|| w.next_anchor()) {
                anchor = Some(a);
                presentation = Some(PresentationAnchor::Domain(a));
                operation = Operation::Continue(a);
            }
        }
        if matches!(operation, Operation::Resize | Operation::Horizontal { .. })
            && let Some(a) = presentation.as_ref()
        {
            anchor = Some(a.layout_selection());
        }
        let key = DocumentKey { operation, ..key };
        let old_slot = self.layout.text().and_then(|w| w.slot_alias());
        let input = Arc::new(DocumentPreparationInput {
            document: self.document.clone(),
            provider_gate: self.provider_gate.clone(),
            budget: self.budget.clone(),
            source: self.source.clone(),
            storage: self.storage.clone(),
            key,
            old_flow: Some(flow),
            old_slot,
            anchor,
            presentation,
            old_window_bytes: self.layout.bytes(),
            _metadata: metadata,
            _source_metadata: self._source_metadata.clone(),
            funding,
        });
        self.expected = Some(key);
        Ok(input)
    }
    pub(crate) fn commit(&mut self, state: PreparedState) -> Result<(), Cause> {
        match state {
            PreparedState::PositionPending(p) => {
                self.check(&p.input)?;
                Err(map_cause(p.cause)) // preserve current/frame/history; not NoMatch or destructive Back.
            }
            PreparedState::Viewport(v) => {
                self.check(&v.input)?;
                let PreparedViewport {
                    input,
                    new_flow,
                    selection,
                    navigation,
                    layout,
                    _metadata,
                } = *v;
                if navigation.is_some() {
                    return Err(Cause::Stale);
                }
                self.navigator
                    .commit_prepared(new_flow, selection)
                    .map_err(map_cause)?;
                self.render_binding = input.key.binding;
                self.presentation = layout
                    .text()
                    .and_then(|w| w.presentation_row(0).cloned())
                    .or_else(|| input.presentation.clone());
                self.layout = layout;
                self.geometry = input.key.geometry;
                self.fold_revision = input.key.fold_revision;
                self.expected = None;
                Ok(())
            }
        }
    }
    fn check(&self, input: &DocumentPreparationInput) -> Result<(), Cause> {
        if self.expected != Some(input.key)
            || !Arc::ptr_eq(&self.document, &input.document)
            || !Arc::ptr_eq(&self.source, &input.source)
            || !input
                .old_flow
                .as_ref()
                .is_some_and(|f| self.navigator.same_flow(f))
        {
            return Err(Cause::Stale);
        }
        Ok(())
    }
    pub(crate) fn presentation_anchor(&self) -> Option<&PresentationAnchor> {
        self.presentation.as_ref()
    }
    pub(crate) fn render_binding(&self) -> Binding {
        self.render_binding
    }
    pub(crate) fn inspect_document(&self) -> &Arc<Document> {
        &self.document
    }
    pub(crate) fn inspect_source_context(&self) -> &Arc<PreparedSourceContext> {
        &self.source
    }
    pub(crate) fn inspect_source_metadata(&self) -> &Arc<Charge> {
        &self._source_metadata
    }
    pub(crate) fn layout(&self) -> &LayoutWindow {
        &self.layout
    }
    pub(crate) fn navigator(&self) -> &Navigator {
        &self.navigator
    }
    pub(crate) fn budget(&self) -> &ConsumerBudget {
        &self.budget
    }
    pub(crate) fn geometry(&self) -> Geometry {
        self.geometry
    }
}
pub(crate) fn start_document(
    input: Arc<DocumentPreparationInput>,
    authority: &mut DocumentPreparationAuthority,
) -> Result<(), Cause> {
    let funding = match authority.actor_arena() {
        Some(a) if a.same(input.document.actor_arena()) => KernelFunding::Actor,
        Some(_) => return Err(Cause::Stale),
        None => KernelFunding::Legacy,
    };
    if input.funding != funding
        || input.metadata_bytes()
            != input_metadata_bytes(input.old_flow.is_some(), funding).map_err(map_cause)?
    {
        return Err(Cause::Stale);
    }
    let binding = input.key.binding;
    let runner = DocumentRunner {
        context: input.clone(),
    };
    authority.start_document_owner(move || {
        OwnedPreparation::spawn(input, binding, move |_, stop| runner.execute(stop))
    })
}
fn remap(flow: &FlowHandle, layout: LayoutAnchor, cancel: &AtomicBool) -> Result<State, Error> {
    let anchor = layout.raw;
    if anchor.run == NONE {
        State::prepared_role(flow.index(), anchor.block, cancel)
    } else {
        State::prepared_visual(
            flow.index(),
            anchor.run,
            anchor.byte,
            layout.projection_part,
            cancel,
        )
    }
}
pub(crate) fn validate_raw_anchor(
    document: &Document,
    anchor: RawAnchor,
    work: &mut DocumentWork<'_>,
) -> Result<(), Error> {
    work.admit_structure(size_of::<crate::document_model::Block>())?;
    document
        .blocks()
        .get(anchor.block as usize)
        .ok_or(Error::Invalid)?;
    if anchor.run == NONE {
        return if anchor.byte == 0 {
            Ok(())
        } else {
            Err(Error::Invalid)
        };
    }
    work.admit_structure(
        size_of::<crate::document_model::Run>() + size_of::<crate::document_model::Domain>(),
    )?;
    let run = document
        .runs()
        .get(anchor.run as usize)
        .ok_or(Error::Invalid)?;
    let domain = document
        .domains()
        .get(run.domain() as usize)
        .ok_or(Error::Invalid)?;
    if domain.owner() != anchor.block || !matches!(domain.kind(), 0..=2) {
        return Err(Error::Invalid);
    }
    if matches!(run.kind(), 2 | 3 | 5) {
        return if anchor.byte == 0 {
            Ok(())
        } else {
            Err(Error::Invalid)
        };
    }
    work.admit_structure(size_of::<crate::document_model::TextRef>() + 1)?;
    let text = document.text(run.text()).map_err(|_| Error::Invalid)?;
    if anchor.byte > text.len() || !text.is_char_boundary(anchor.byte) {
        return Err(Error::Invalid);
    }
    Ok(())
}
fn candidate_folds(
    input: &DocumentPreparationInput,
    block: u32,
    expanded: bool,
    cancel: &AtomicBool,
) -> Result<ChargedVec<FoldOverride>, Error> {
    let old = input
        .old_flow
        .as_ref()
        .ok_or(Error::Invalid)?
        .index()
        .overrides();
    let mut folds = ChargedVec::new(
        &input.budget,
        old.len().checked_add(1).ok_or(Error::Budget)?,
    )?;
    let mut inserted = false;
    let mut w = DocumentWork::preparation(cancel);
    for &f in old {
        w.admit_structure(size_of::<FoldOverride>())?;
        if !inserted && block < f.block {
            folds.push(FoldOverride { block, expanded })?;
            inserted = true;
        }
        if f.block == block {
            if !inserted {
                folds.push(FoldOverride { block, expanded })?;
                inserted = true;
            }
        } else {
            folds.push(f)?;
        }
    }
    if !inserted {
        w.admit_structure(size_of::<FoldOverride>())?;
        folds.push(FoldOverride { block, expanded })?;
    }
    Ok(folds)
}
fn run_prepare(
    input: Arc<DocumentPreparationInput>,
    cancel: &AtomicBool,
) -> Outcome<PreparedState> {
    let result = prepare(input.clone(), cancel);
    match result {
        Ok(v) => Outcome::Ready(PreparedState::Viewport(Box::new(v))),
        Err(Error::Cancelled) => Outcome::Cancelled,
        Err(e) => match input.budget.reserve(size_of::<PreparedPending>()) {
            Ok(metadata) => {
                Outcome::Ready(PreparedState::PositionPending(Box::new(PreparedPending {
                    input,
                    cause: e,
                    _metadata: metadata,
                })))
            }
            Err(_) => Outcome::Limit("ConsumerBudget pending receipt"),
        },
    }
}
fn prepare(
    input: Arc<DocumentPreparationInput>,
    cancel: &AtomicBool,
) -> Result<PreparedViewport, Error> {
    let mut prep = DocumentWork::preparation(cancel);
    prep.admit_structure(0)?;
    let metadata = input.budget.reserve(size_of::<PreparedViewport>())?;
    let navigation = if input.key.operation == Operation::Initial {
        Some(NavigationStorage::new(&input.budget)?)
    } else {
        None
    };
    // Logical input validity precedes geometry/quota Reference and any window allocation.
    // A closed Source/Document does not prove a caller-supplied raw Run/local anchor.
    let requested = match input.key.operation {
        Operation::Seek(raw) => Some(LayoutAnchor::physical(raw)),
        Operation::Continue(layout) => Some(layout),
        Operation::Presentation => Some(
            input
                .presentation
                .as_ref()
                .ok_or(Error::Invalid)?
                .layout_selection(),
        ),
        _ => input.anchor,
    };
    if let Some(layout) = requested {
        validate_raw_anchor(&input.document, layout.raw, &mut prep)?;
        if layout.raw.run == NONE && layout.projection_part != 0 {
            return Err(Error::Invalid);
        }
        crate::document_text::validate_visual_part(
            &input.document,
            layout.raw.run,
            layout.raw.byte,
            layout.projection_part,
            &mut prep,
        )?;
    }
    if let Operation::Horizontal { left } = input.key.operation {
        if input.key.geometry.left != left {
            return Err(Error::Invalid);
        }
        let block = requested.ok_or(Error::Invalid)?.raw.block;
        if !matches!(
            crate::document_roles::block_role(&input.document, block, &mut prep)?,
            crate::document_roles::BlockRole::Code { .. }
                | crate::document_roles::BlockRole::Frontmatter
        ) {
            return Err(Error::Invalid);
        }
    }
    if let Some(anchor) = input.presentation.as_ref() {
        match anchor {
            PresentationAnchor::Ref(a) => {
                crate::document_presentation_anchor::RefAnchor::checked(
                    &input.document,
                    a.owner(),
                    a.text_ref(),
                    a.local(),
                    a.part(),
                    &mut prep,
                )?;
            }
            PresentationAnchor::ResourceAlt(a) => {
                crate::document_presentation_anchor::ResourceAltAnchor::checked(
                    &input.document,
                    a.block(),
                    a.run(),
                    a.resource(),
                    a.domain(),
                    a.position(),
                    &mut prep,
                )?;
            }
            PresentationAnchor::Table(t) => {
                for p in t.cells() {
                    prep.admit_structure(size_of::<
                        crate::document_presentation_anchor::TableCellProgress,
                    >())?;
                    if let crate::document_presentation_anchor::CellProgress::At(a) = p.progress {
                        validate_raw_anchor(&input.document, a.raw, &mut prep)?;
                        crate::document_text::validate_visual_part(
                            &input.document,
                            a.raw.run,
                            a.raw.byte,
                            a.projection_part,
                            &mut prep,
                        )?;
                    }
                }
            }
            _ => {}
        }
    }
    if let Some(PresentationAnchor::Role(a)) = input.presentation.as_ref() {
        let flow = input.old_flow.as_ref().ok_or(Error::Invalid)?;
        let location = flow.index().location(a.ordinal())?;
        let event = flow.index().event_at(&location, &mut prep)?;
        if !matches!(event,crate::document_flow::Event::Enter(block)if block==a.block()&&a.run()==NONE)
            && !matches!(event,crate::document_flow::Event::Run{block,run}if block==a.block()&&run==a.run())
        {
            return Err(Error::Invalid);
        }
    }
    // Window capacity is mandatory and admitted before adaptive sparse-index density.
    let builder = WindowBuilder::new(&input.budget, input.key.geometry, input.old_window_bytes)
        .map(|b| b.with_provider_gate(input.provider_gate.clone()));
    let profile = ConsumerProfile::plan(
        &input.budget,
        512,
        IndexProfile {
            projection_points: 256,
            unicode_points: 256,
            exceptions: 8,
        },
        2,
    )?;
    let new_flow = match input.key.operation {
        Operation::Initial => Some(SharedFlow::new(FlowIndex::prepare_visible(
            input.document.clone(),
            &input.budget,
            profile.flow_slots,
            &[],
            cancel,
        )?)?),
        Operation::Fold { block, expanded } => {
            prep.admit_structure(size_of::<crate::document_model::Block>())?;
            if input
                .document
                .blocks()
                .get(block as usize)
                .is_none_or(|b| b.kind() != 4)
            {
                return Err(Error::Invalid);
            }
            let overrides = candidate_folds(&input, block, expanded, cancel)?;
            Some(SharedFlow::new(FlowIndex::prepare_visible(
                input.document.clone(),
                &input.budget,
                profile.flow_slots,
                overrides.as_slice(),
                cancel,
            )?)?)
        }
        _ => None,
    };
    let flow = if let Some(f) = &new_flow {
        f.alias()?
    } else {
        input.old_flow.as_ref().ok_or(Error::Invalid)?.try_alias()?
    };
    let mut selection = match input.key.operation {
        Operation::Initial | Operation::Home => flow.index().first().map(State::from_target),
        Operation::End => flow.index().terminal().map(State::from_target),
        Operation::Seek(raw) => Some(remap(&flow, LayoutAnchor::physical(raw), cancel)?),
        Operation::Continue(layout) => Some(remap(&flow, layout, cancel)?),
        Operation::Presentation => Some(remap(
            &flow,
            input
                .presentation
                .as_ref()
                .ok_or(Error::Invalid)?
                .layout_selection(),
            cancel,
        )?),
        Operation::Fold { block, .. } => match input.anchor {
            Some(raw) => match remap(&flow, raw, cancel) {
                Ok(s) => Some(s),
                Err(Error::Cancelled) => return Err(Error::Cancelled),
                Err(_) => Some(State::prepared_role(flow.index(), block, cancel)?),
            },
            None => flow.index().first().map(State::from_target),
        },
        Operation::Resize | Operation::Horizontal { .. } | Operation::Line { .. } => {
            input.anchor.map(|r| remap(&flow, r, cancel)).transpose()?
        }
    };
    let mut shifted_slot = None;
    if let Operation::Line { delta } = input.key.operation
        && let Some(selected) = selection.as_ref()
    {
        let (s, slot) = document_layout::shift_line_scoped(
            input.document.clone(),
            &flow,
            selected,
            delta,
            input.old_slot.as_ref(),
            &input.budget,
            cancel,
            profile,
            input.provider_gate.as_ref(),
        )?;
        selection = Some(s);
        shifted_slot = slot;
    }
    let layout = match (builder, selection.as_ref()) {
        (_, None) => {
            LayoutWindow::reference(ReferenceReason::Geometry, input.key.geometry, cancel)?
        }
        (Err(Error::Cancelled), _) => return Err(Error::Cancelled),
        (Err(_), _) => LayoutWindow::reference(
            if input.key.geometry.width == 0 || input.key.geometry.height == 0 {
                ReferenceReason::Geometry
            } else {
                ReferenceReason::Window
            },
            input.key.geometry,
            cancel,
        )?,
        (Ok(mut builder), Some(selected)) => {
            builder.retain_candidate(shifted_slot.take());
            let end_anchor = input.key.operation == Operation::End
                || matches!(input.key.operation,Operation::Line{delta} if delta<0)
                    && selected.run() != u32::MAX
                    && input
                        .document
                        .runs()
                        .get(selected.run() as usize)
                        .is_some_and(|r| {
                            input
                                .document
                                .text(r.text())
                                .is_ok_and(|t| selected.raw_byte() == t.len())
                        });
            document_layout::build_window(
                builder,
                input.document.clone(),
                &flow,
                selected,
                input.old_slot.as_ref(),
                &input.budget,
                cancel,
                profile,
                end_anchor,
                input.presentation.as_ref(),
            )?
        }
    };
    prep.admit_structure(0)?;
    Ok(PreparedViewport {
        input,
        new_flow,
        selection,
        navigation,
        layout,
        _metadata: metadata,
    })
}
#[cfg(test)]
#[path = "document_view_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "document_kernel_metadata_tests.rs"]
mod kernel_metadata_tests;

impl DocumentView {
    pub(crate) fn inspect_provider_gate(
        &self,
    ) -> Option<&Arc<crate::document_provider_gate::ProviderGate>> {
        self.provider_gate.as_ref()
    }
}

pub(crate) struct CacheRetirement {
    pub(crate) gate: Arc<crate::document_provider_gate::ProviderGate>,
    pub(crate) slot: Option<Arc<SequenceSlot>>,
    _charge: Charge,
}
impl DocumentView {
    pub(crate) fn retire_cached_provider(
        &mut self,
        work: &mut DocumentWork<'_>,
    ) -> Result<CacheRetirement, Error> {
        let gate = self.provider_gate.as_ref().ok_or(Error::ScopePending)?;
        work.admit_structure(size_of::<CacheRetirement>())?;
        let charge = self.budget.reserve(size_of::<CacheRetirement>())?;
        work.admit_structure(0)?;
        let slot = match &mut self.layout {
            LayoutWindow::Text(text) => text.take_provider_cache(),
            LayoutWindow::Reference { .. } => None,
        };
        Ok(CacheRetirement {
            gate: gate.clone(),
            slot,
            _charge: charge,
        })
    }
}

impl DocumentView {
    pub(crate) fn initial_shared_inspectable_input<
        DI: Send + Sync + 'static,
        DO: Send + 'static,
    >(
        document: Arc<Document>,
        geometry: Geometry,
        authority: &PreparationAuthority<DI, DO>,
    ) -> Result<SharedDocumentDescriptor, Cause> {
        let mut descriptor = Self::initial_shared_input(document, geometry, authority)?;
        let input = Arc::get_mut(&mut descriptor.input).ok_or(Cause::Missing)?;
        let stop = AtomicBool::new(false);
        let gate = crate::document_provider_gate::ProviderGate::from_owner(
            input.document.clone(),
            &input.budget,
            input.source.clone(),
            input._source_metadata.clone(),
            &mut DocumentWork::preparation(&stop),
        )
        .map_err(map_cause)?;
        input.provider_gate = Some(gate);
        Ok(descriptor)
    }
}
