//! Library dispatcher on the existing kernel: one financed authority, no pump/session.
use super::*;
use crate::{
    document_flow::FlowHandle,
    document_search::{
        SearchOutcome, SearchRequest,
        shared_query::{QueryCursor, SharedQuery},
    },
    document_view::{CacheRetirement, PreparedState, SharedDocumentDescriptor},
    preparation_owner::{Binding, Cause, Outcome, OwnedPreparation, PreparationAuthority},
    search::Direction,
};
use std::{mem::size_of, sync::Mutex};
enum SeedKind {
    Hit(HitLease),
    Anchor {
        anchor: PresentationAnchor,
        slot: Option<Arc<SequenceSlot>>,
    },
    Cursor {
        state: State,
        flow: FlowHandle,
        slot: Option<Arc<SequenceSlot>>,
    },
}
pub(crate) struct InspectionPreparationInput {
    scope: Arc<Scope>,
    kind: SeedKind,
    _charge: Charge,
}
impl InspectionPreparationInput {
    fn scope(view: &DocumentView, work: &mut DocumentWork<'_>) -> Result<Arc<Scope>> {
        Scope::new(
            view.inspect_document().clone(),
            view.inspect_source_context().clone(),
            view.budget(),
            view.inspect_source_metadata().clone(),
            view.render_binding(),
            view.inspect_provider_gate()
                .cloned()
                .ok_or(Error::ScopePending)?,
            work,
        )
    }
    pub(crate) fn hit(
        view: &DocumentView,
        hit: &SharedHit,
        work: &mut DocumentWork<'_>,
    ) -> Result<Self> {
        let scope = Self::scope(view, work)?;
        if !Arc::ptr_eq(hit.hit().document(), &scope.document)
            || !hit
                .hit()
                .provider_gate()
                .is_some_and(|g| g.same(&scope.gate))
        {
            return Err(Error::ScopePending);
        }
        // HitLease admits its own body; admit the remaining seed before construction.
        work.admit_structure(size_of::<Self>() - size_of::<HitLease>())?;
        let charge = scope
            .budget
            .reserve(size_of::<Self>() - size_of::<HitLease>())?;
        let lease = hit.lease(work)?;
        work.admit_structure(0)?;
        Ok(Self {
            scope,
            kind: SeedKind::Hit(lease),
            _charge: charge,
        })
    }
    pub(crate) fn view(
        view: &DocumentView,
        cell: Option<usize>,
        work: &mut DocumentWork<'_>,
    ) -> Result<Self> {
        let scope = Self::scope(view, work)?;
        work.admit_structure(size_of::<Self>())?;
        let charge = scope.budget.reserve(size_of::<Self>())?;
        let anchor = if let Some(cell) = cell {
            view.layout()
                .text()
                .ok_or(Error::Invalid)?
                .cell_anchor(cell, work)?
                .ok_or(Error::Invalid)?
        } else {
            view.presentation_anchor().cloned().ok_or(Error::Invalid)?
        };
        let slot = view.layout().text().and_then(|t| t.slot_alias());
        work.admit_structure(0)?;
        Ok(Self {
            scope,
            kind: SeedKind::Anchor { anchor, slot },
            _charge: charge,
        })
    }
    pub(crate) fn cursor(
        view: &DocumentView,
        state: &State,
        work: &mut DocumentWork<'_>,
    ) -> Result<Self> {
        let scope = Self::scope(view, work)?;
        view.navigator().index().check_location(state.location())?;
        // FlowHandle has a Consumer charge, but no separate Work admission.
        work.admit_structure(size_of::<Self>())?;
        let charge = scope
            .budget
            .reserve(size_of::<Self>() - size_of::<FlowHandle>())?;
        let flow = view.navigator().share_flow()?;
        work.admit_structure(0)?;
        Ok(Self {
            scope,
            kind: SeedKind::Cursor {
                state: state.clone(),
                flow,
                slot: view.layout().text().and_then(|t| t.slot_alias()),
            },
            _charge: charge,
        })
    }
    fn proof(&self, work: &mut DocumentWork<'_>) -> Result<InspectionProof> {
        // A hit alias admits its own body; every other proof field still needs Work.
        let alias_body = if matches!(&self.kind, SeedKind::Hit(_)) {
            size_of::<HitLease>()
        } else {
            0
        };
        work.admit_structure(size_of::<InspectionProof>() - alias_body)?;
        let hit = match &self.kind {
            SeedKind::Hit(h) => Some(h.try_alias(work)?),
            _ => None,
        };
        let charge = self.scope.budget.reserve(
            size_of::<InspectionProof>()
                - if hit.is_some() {
                    size_of::<HitLease>()
                } else {
                    0
                },
        )?;
        work.admit_structure(0)?;
        Ok(InspectionProof {
            scope: self.scope.clone(),
            hit,
            _charge: charge,
        })
    }
    fn prepare(self, cancel: &AtomicBool, work: &mut DocumentWork<'_>) -> Result<Inspection> {
        work.admit_structure(0)?;
        match self.kind {
            SeedKind::Hit(lease) => Inspection::from_hit_scope(lease, self.scope, work),
            SeedKind::Anchor { anchor, slot } => {
                Inspection::from_anchor_scope(self.scope, anchor, slot, cancel, work)
            }
            SeedKind::Cursor { state, flow, slot } => {
                if state.position_pending() {
                    return Err(Error::Work);
                }
                if state.run() == NONE {
                    let actual = State::prepared_role(flow.index(), state.block(), cancel)?;
                    roles::block_role(&self.scope.document, actual.block(), work)?;
                    let charge = self.scope.budget.reserve(size_of::<Inspection>())?;
                    Ok(Inspection {
                        scope: self.scope,
                        provider: ProviderOwner::Role,
                        range: None,
                        anchor: None,
                        ref_owner: None,
                        resource: None,
                        block: actual.block(),
                        _charge: charge,
                    })
                } else {
                    let actual = State::prepared_visual(
                        flow.index(),
                        state.run(),
                        state.raw_byte(),
                        state.projection_part(),
                        cancel,
                    )?;
                    let anchor = PresentationAnchor::Domain(crate::document_view::LayoutAnchor {
                        raw: crate::document_view::RawAnchor {
                            block: actual.block(),
                            run: actual.run(),
                            byte: actual.raw_byte(),
                        },
                        projection_part: actual.projection_part(),
                    });
                    Inspection::from_anchor_scope(self.scope, anchor, slot, cancel, work)
                }
            }
        }
    }
}
pub(crate) struct InspectionProof {
    scope: Arc<Scope>,
    hit: Option<HitLease>,
    _charge: Charge,
}
impl InspectionProof {
    fn check(&self, view: &DocumentView, current_hit: Option<&SharedHit>) -> Result<()> {
        if self.scope.binding != view.render_binding()
            || !Arc::ptr_eq(&self.scope.document, view.inspect_document())
            || !Arc::ptr_eq(&self.scope.context, view.inspect_source_context())
            || !view
                .inspect_provider_gate()
                .is_some_and(|g| g.same(&self.scope.gate))
        {
            return Err(Error::Invalid);
        }
        if let Some(hit) = &self.hit
            && !current_hit.is_some_and(|h| hit.same_shared(h))
        {
            return Err(Error::Invalid);
        }
        Ok(())
    }
}
pub(crate) struct PreparedInspection {
    proof: InspectionProof,
    result: Result<Inspection>,
    _metadata: Charge,
}
impl PreparedInspection {
    pub(crate) fn check_view(&self, view: &DocumentView, hit: Option<&SharedHit>) -> Result<()> {
        self.proof.check(view, hit)
    }
    pub(crate) fn result(&self) -> std::result::Result<&Inspection, Error> {
        self.result.as_ref().map_err(|e| *e)
    }
    pub(crate) fn into_inspection(
        self,
        view: &DocumentView,
        hit: Option<&SharedHit>,
    ) -> Result<Inspection> {
        self.proof.check(view, hit)?;
        self.result
    }
}
enum QuerySeed {
    Raw {
        domain: SearchDomain,
        query: ChargedVec<u8>,
        revision: u64,
    },
    Ready {
        query: SharedQuery,
        after: QueryCursor,
    },
}
struct SearchSeed {
    scope: Arc<Scope>,
    query: QuerySeed,
    direction: Direction,
    _metadata: Charge,
}
enum InputKind {
    Viewport(SharedDocumentDescriptor),
    Search(Box<SearchSeed>),
    Inspect(Mutex<Option<Box<InspectionPreparationInput>>>),
    RetireAndInspect {
        retired: Mutex<Option<Box<CacheRetirement>>>,
        seed: Mutex<Option<Box<InspectionPreparationInput>>>,
    },
}
struct FundedJob {
    _charge: Charge,
    arena: crate::document_budget::ActorArena,
}
pub(crate) struct InspectionLibraryInput {
    kind: InputKind,
    binding: Binding,
    document: Arc<Document>,
    budget: ConsumerBudget,
    _metadata: Charge,
    job: Mutex<Option<FundedJob>>,
}
pub(crate) enum InspectionLibraryOutput {
    Viewport(PreparedState),
    Search(Box<SearchCompletion>),
    Inspector(Box<PreparedInspection>),
}
pub(crate) struct SearchCompletion {
    scope: Arc<Scope>,
    query: SharedQuery,
    outcome: SearchOutcome,
    _metadata: Charge,
}
impl SearchCompletion {
    pub(crate) fn outcome(&self) -> &SearchOutcome {
        &self.outcome
    }
    #[allow(clippy::result_large_err)] // Preserve the complete charged candidate on stale view.
    pub(crate) fn into_outcome(
        self,
        view: &DocumentView,
    ) -> std::result::Result<SearchOutcome, (Error, Self)> {
        if let Err(error) = self.check_view(view) {
            return Err((error, self));
        }
        Ok(self.outcome)
    }

    pub(crate) fn continuation(
        &self,
        direction: Direction,
        work: &mut DocumentWork<'_>,
    ) -> Result<Arc<InspectionLibraryInput>> {
        let cursor = self
            .query
            .cursor(self.outcome.cursor().ok_or(Error::Invalid)?, work)?;
        let query = self.query.alias(work)?;
        let metadata = self.scope.budget.reserve(
            size_of::<SearchSeed>() - size_of::<SharedQuery>() - size_of::<QueryCursor>(),
        )?;
        work.admit_structure(0)?;
        let seed = SearchSeed {
            scope: self.scope.clone(),
            query: QuerySeed::Ready {
                query,
                after: cursor,
            },
            direction,
            _metadata: metadata,
        };
        work.admit_structure(size_of::<SearchSeed>())?;
        InspectionLibraryInput::wrap(
            InputKind::Search(Box::new(seed)),
            self.scope.binding,
            &self.scope.budget,
            work,
        )
    }
    pub(crate) fn check_view(&self, view: &DocumentView) -> Result<()> {
        if self.scope.binding == view.render_binding()
            && Arc::ptr_eq(&self.scope.document, view.inspect_document())
        {
            Ok(())
        } else {
            Err(Error::Invalid)
        }
    }
}
pub(crate) type InspectionLibraryAuthority =
    PreparationAuthority<InspectionLibraryInput, InspectionLibraryOutput>;
pub(crate) struct StartPending {
    pub(crate) cause: Cause,
    pub(crate) input: Arc<InspectionLibraryInput>,
}
impl InspectionLibraryInput {
    fn wrap(
        kind: InputKind,
        binding: Binding,
        budget: &ConsumerBudget,
        work: &mut DocumentWork<'_>,
    ) -> Result<Arc<Self>> {
        work.admit_structure(size_of::<Self>())?;
        let metadata = budget.reserve(size_of::<Self>() + 2 * size_of::<usize>())?;
        work.admit_structure(0)?;
        let document = match &kind {
            InputKind::Viewport(input) => input.document().clone(),
            InputKind::Search(seed) => seed.scope.document.clone(),
            InputKind::Inspect(seed) | InputKind::RetireAndInspect { seed, .. } => seed
                .lock()
                .map_err(|_| Error::Invalid)?
                .as_ref()
                .ok_or(Error::Invalid)?
                .scope
                .document
                .clone(),
        };
        Ok(Arc::new(Self {
            kind,
            binding,
            document,
            budget: budget.clone(),
            _metadata: metadata,
            job: Mutex::new(None),
        }))
    }
    pub(crate) fn viewport(
        input: SharedDocumentDescriptor,
        work: &mut DocumentWork<'_>,
    ) -> Result<Arc<Self>> {
        let binding = input.key().binding;
        let budget = input.budget().clone();
        Self::wrap(InputKind::Viewport(input), binding, &budget, work)
    }
    pub(crate) fn inspect(
        seed: InspectionPreparationInput,
        work: &mut DocumentWork<'_>,
    ) -> Result<Arc<Self>> {
        let binding = seed.scope.binding;
        let budget = seed.scope.budget.clone();
        work.admit_structure(size_of::<InspectionPreparationInput>())?;
        Self::wrap(
            InputKind::Inspect(Mutex::new(Some(Box::new(seed)))),
            binding,
            &budget,
            work,
        )
    }
    #[allow(clippy::result_large_err)] // Return funded original owners without allocating during rollback.
    pub(crate) fn retire_and_inspect(
        retired: CacheRetirement,
        mut seed: InspectionPreparationInput,
        work: &mut DocumentWork<'_>,
    ) -> std::result::Result<Arc<Self>, (Error, CacheRetirement, InspectionPreparationInput)> {
        let checked = (|| {
            work.admit_structure(
                size_of::<CacheRetirement>() + size_of::<InspectionPreparationInput>(),
            )?;
            if !retired.gate.same(&seed.scope.gate)
                || retired
                    .slot
                    .as_ref()
                    .is_some_and(|s| !s.provider_gate().is_some_and(|g| g.same(&retired.gate)))
            {
                return Err(Error::Invalid);
            }
            Ok(seed
                .scope
                .budget
                .reserve(size_of::<Self>() + 2 * size_of::<usize>())?)
        })();
        match checked {
            Err(error) => Err((error, retired, seed)),
            Ok(metadata) => {
                if let SeedKind::Anchor { slot, .. } | SeedKind::Cursor { slot, .. } =
                    &mut seed.kind
                    && slot
                        .as_ref()
                        .zip(retired.slot.as_ref())
                        .is_some_and(|(a, b)| Arc::ptr_eq(a, b))
                {
                    drop(slot.take());
                }
                let binding = seed.scope.binding;
                let budget = seed.scope.budget.clone();
                let document = seed.scope.document.clone();
                Ok(Arc::new(Self {
                    kind: InputKind::RetireAndInspect {
                        retired: Mutex::new(Some(Box::new(retired))),
                        seed: Mutex::new(Some(Box::new(seed))),
                    },
                    binding,
                    document,
                    budget,
                    _metadata: metadata,
                    job: Mutex::new(None),
                }))
            }
        }
    }
    pub(crate) fn binding(&self) -> Binding {
        self.binding
    }
    pub(crate) fn search(
        view: &DocumentView,
        domain: SearchDomain,
        query: &str,
        revision: u64,
        direction: Direction,
        work: &mut DocumentWork<'_>,
    ) -> Result<Arc<Self>> {
        if query.len() > crate::search::QUERY_LIMIT {
            return Err(Error::Invalid);
        }
        let scope = InspectionPreparationInput::scope(view, work)?;
        let metadata = scope.budget.reserve(size_of::<SearchSeed>())?;
        let mut bytes = ChargedVec::new(&scope.budget, query.len())?;
        for &byte in query.as_bytes() {
            work.admit_structure(1)?;
            bytes.push(byte)?;
        }
        let budget = scope.budget.clone();
        let binding = scope.binding;
        work.admit_structure(size_of::<SearchSeed>())?;
        Self::wrap(
            InputKind::Search(Box::new(SearchSeed {
                scope,
                query: QuerySeed::Raw {
                    domain,
                    query: bytes,
                    revision,
                },
                direction,
                _metadata: metadata,
            })),
            binding,
            &budget,
            work,
        )
    }
}
pub(crate) fn authority_for_view(
    view: &DocumentView,
) -> std::result::Result<InspectionLibraryAuthority, Cause> {
    PreparationAuthority::in_arena(view.inspect_document().actor_arena())
}
#[allow(clippy::result_large_err)] // Return the charged original owner without another allocation on failure.
pub(crate) fn start(
    input: Arc<InspectionLibraryInput>,
    authority: &mut InspectionLibraryAuthority,
) -> std::result::Result<(), StartPending> {
    type Run = fn(&InspectionLibraryInput, &AtomicBool) -> Outcome<InspectionLibraryOutput>;
    let run: Run = dispatch;
    let mut installed = false;
    let prepared = (|| {
        // Provenance comes from the ACTUAL financed authority at execution admission.
        let arena = authority.actor_arena().ok_or(Cause::Stale)?;
        if !arena.same(input.document.actor_arena()) {
            return Err(Cause::Stale);
        }
        let mut job = input.job.lock().map_err(|_| Cause::Poisoned)?;
        if job.is_some() {
            return Err(Cause::Busy);
        }
        let bytes =
            OwnedPreparation::<InspectionLibraryInput, InspectionLibraryOutput>::known_spawn_bytes(
                &run,
            )
            .ok_or(Cause::Limit("inspection job layout"))?;
        let body = size_of::<InspectionLibraryInput>() + 2 * size_of::<usize>();
        let charge = input
            .budget
            .reserve(
                bytes
                    .checked_sub(body)
                    .ok_or(Cause::Limit("inspection job body"))?,
            )
            .map_err(|_| Cause::Limit("inspection job quota"))?;
        *job = Some(FundedJob {
            _charge: charge,
            arena: arena.clone(),
        });
        installed = true;
        drop(job);
        let owner_input = input.clone();
        let binding = input.binding;
        authority.start_document_owner(move || OwnedPreparation::spawn(owner_input, binding, run))
    })();
    match prepared {
        Ok(()) => Ok(()),
        Err(cause) => {
            if installed && let Ok(mut job) = input.job.lock() {
                drop(job.take());
            }
            Err(StartPending { cause, input })
        }
    }
}
fn prepare_inspection(
    seed: InspectionPreparationInput,
    stop: &AtomicBool,
) -> Outcome<InspectionLibraryOutput> {
    let mut work = DocumentWork::preparation(stop);
    let proof = match seed.proof(&mut work) {
        Ok(proof) => proof,
        Err(Error::Cancelled) => return Outcome::Cancelled,
        Err(_) => return Outcome::Limit("inspection proof"),
    };
    let budget = seed.scope.budget.clone();
    let result = seed.prepare(stop, &mut work);
    if matches!(result, Err(Error::Cancelled)) {
        return Outcome::Cancelled;
    }
    let covered = if result.is_ok() {
        size_of::<Inspection>()
    } else {
        0
    };
    let bytes = size_of::<PreparedInspection>() - covered - size_of::<InspectionProof>();
    match work.admit_structure(size_of::<PreparedInspection>()) {
        Ok(()) => {}
        Err(Error::Cancelled) => return Outcome::Cancelled,
        Err(_) => return Outcome::Limit("inspection output work"),
    }
    let metadata = match budget.reserve(bytes) {
        Ok(c) => c,
        Err(_) => return Outcome::Limit("inspection output metadata"),
    };
    Outcome::Ready(InspectionLibraryOutput::Inspector(Box::new(
        PreparedInspection {
            proof,
            result,
            _metadata: metadata,
        },
    )))
}
fn dispatch(input: &InspectionLibraryInput, stop: &AtomicBool) -> Outcome<InspectionLibraryOutput> {
    // Only start can mint FundedJob; it retains the exact arena of its physical authority lease.
    match input.job.lock() {
        Ok(job)
            if job
                .as_ref()
                .is_some_and(|proof| proof.arena.same(input.document.actor_arena())) => {}
        Ok(_) => return Outcome::Failed(Cause::Stale),
        Err(_) => return Outcome::Failed(Cause::Poisoned),
    }
    match &input.kind {
        InputKind::Viewport(document) => {
            match crate::document_view::prepare_shared_document(document, stop) {
                Outcome::Ready(v) => Outcome::Ready(InspectionLibraryOutput::Viewport(v)),
                Outcome::Cancelled => Outcome::Cancelled,
                Outcome::Limit(x) => Outcome::Limit(x),
                Outcome::Failed(e) => Outcome::Failed(e),
            }
        }
        InputKind::Inspect(seed) => match seed
            .lock()
            .map_err(|_| Cause::Poisoned)
            .and_then(|mut s| s.take().ok_or(Cause::Missing))
        {
            Ok(seed) => prepare_inspection(*seed, stop),
            Err(e) => Outcome::Failed(e),
        },
        InputKind::RetireAndInspect { retired, seed } => {
            match retired.lock() {
                Ok(mut slot) => drop(slot.take()),
                Err(_) => return Outcome::Failed(Cause::Poisoned),
            }
            match seed
                .lock()
                .map_err(|_| Cause::Poisoned)
                .and_then(|mut s| s.take().ok_or(Cause::Missing))
            {
                Ok(seed) => prepare_inspection(*seed, stop),
                Err(e) => Outcome::Failed(e),
            }
        }
        InputKind::Search(seed) => {
            let (query, outcome) = match &seed.query {
                QuerySeed::Raw {
                    domain,
                    query,
                    revision,
                } => {
                    let query = match std::str::from_utf8(query.as_slice()) {
                        Ok(q) => q,
                        Err(_) => return Outcome::Failed(Cause::Stale),
                    };
                    let request = match SearchRequest::prepare_with_gate(
                        seed.scope.document.clone(),
                        &seed.scope.budget,
                        *domain,
                        query,
                        *revision,
                        stop,
                        &seed.scope.gate,
                    ) {
                        Ok(r) => r,
                        Err(Error::Cancelled) => return Outcome::Cancelled,
                        Err(_) => return Outcome::Limit("scoped search query"),
                    };
                    let query =
                        match SharedQuery::new(request, &mut DocumentWork::preparation(stop)) {
                            Ok(q) => q,
                            Err(Error::Cancelled) => return Outcome::Cancelled,
                            Err(_) => return Outcome::Limit("scoped query owner"),
                        };
                    let outcome = query.find(seed.direction, None, stop);
                    (query, outcome)
                }
                QuerySeed::Ready { query, after } => {
                    let query = match query.alias(&mut DocumentWork::preparation(stop)) {
                        Ok(q) => q,
                        Err(Error::Cancelled) => return Outcome::Cancelled,
                        Err(_) => return Outcome::Limit("scoped query alias"),
                    };
                    let outcome = query.find(seed.direction, Some(after), stop);
                    (query, outcome)
                }
            };
            if matches!(outcome, SearchOutcome::Cancelled) {
                return Outcome::Cancelled;
            }
            let body = outcome.covered_body();
            let metadata = match input
                .budget
                .reserve(size_of::<SearchCompletion>() - body - size_of::<SharedQuery>())
            {
                Ok(c) => c,
                Err(_) => return Outcome::Limit("search output metadata"),
            };
            let mut output_work = DocumentWork::preparation(stop);
            match output_work.admit_structure(size_of::<SearchCompletion>()) {
                Ok(()) => {}
                Err(Error::Cancelled) => return Outcome::Cancelled,
                Err(_) => return Outcome::Limit("search output work"),
            }
            Outcome::Ready(InspectionLibraryOutput::Search(Box::new(
                SearchCompletion {
                    scope: seed.scope.clone(),
                    query,
                    outcome,
                    _metadata: metadata,
                },
            )))
        }
    }
}

#[cfg(test)]
#[test]
fn dispatcher_compiled_layout_inventory() {
    macro_rules! item {($($t:ty),*)=>{$(eprintln!("LAYOUT {} {}",stringify!($t),size_of::<$t>());)*};}
    item!(
        SeedKind,
        InspectionPreparationInput,
        InspectionProof,
        PreparedInspection,
        QuerySeed,
        SearchSeed,
        InputKind,
        FundedJob,
        InspectionLibraryInput,
        InspectionLibraryOutput,
        SearchCompletion,
        CacheRetirement,
        SharedDocumentDescriptor
    );
    type Owner = OwnedPreparation<InspectionLibraryInput, InspectionLibraryOutput>;
    type Run = fn(&InspectionLibraryInput, &AtomicBool) -> Outcome<InspectionLibraryOutput>;
    let run: Run = dispatch;
    eprintln!(
        "KERNEL AuthorityValue {} Parts {} Completion {} FinalReceipt {} Capture {} StopArc {} Job {} Spawn {}",
        size_of::<InspectionLibraryAuthority>(),
        Owner::known_parts_bytes(),
        Owner::known_completion_bytes(),
        Owner::known_final_receipt_bytes(),
        Owner::known_worker_capture_bytes(&run),
        Owner::known_stop_arc_bytes().unwrap(),
        Owner::known_job_bytes(&run).unwrap(),
        Owner::known_spawn_bytes(&run).unwrap()
    );
}

#[cfg(test)]
#[test]
fn boxed_search_completion_exact_minus_one_and_coexistence_ledger() {
    use crate::{
        document_consumer_budget::CONSUMER_LIMIT, document_provider_gate_tests as gate,
        preparation_owner::PreparedCompletion,
    };
    let (view, mut authority) = gate::initial("BODY");
    let budget = view.budget();
    let baseline = budget.used();
    let arc = 2 * size_of::<usize>();
    type Owner = OwnedPreparation<InspectionLibraryInput, InspectionLibraryOutput>;
    type Run = fn(&InspectionLibraryInput, &AtomicBool) -> Outcome<InspectionLibraryOutput>;
    let run: Run = dispatch;
    let job = Owner::known_job_bytes(&run).unwrap();
    let request = size_of::<SearchRequest>() + arc;
    let query_extra = size_of::<Charge>() + arc;
    let handle = size_of::<SharedQuery>();
    let parent = size_of::<SearchCompletion>() - handle;
    let worker = job + request + query_extra + handle + parent;
    for short in [true, false] {
        let cancel = AtomicBool::new(false);
        let input = InspectionLibraryInput::search(
            &view,
            SearchDomain::SourceOriginal,
            "",
            303,
            Direction::Forward,
            &mut DocumentWork::preparation(&cancel),
        )
        .unwrap();
        let binding = input.binding();
        let captured = budget.used();
        let scope = size_of::<Scope>() + 4 * size_of::<usize>();
        assert_eq!(
            captured - baseline,
            scope + size_of::<SearchSeed>() + size_of::<InspectionLibraryInput>() + arc
        );
        let blocker = budget
            .reserve(CONSUMER_LIMIT - captured - worker + usize::from(short))
            .unwrap();
        if let Err(pending) = start(input.clone(), &mut authority) {
            panic!("start {:?}", pending.cause)
        }
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while !authority.is_finished() && std::time::Instant::now() < deadline {
            std::thread::yield_now()
        }
        let Some(PreparedCompletion::Document(outcome, receipt)) =
            authority.take_validated(binding, false).unwrap()
        else {
            panic!("joined")
        };
        assert!(receipt.joined);
        if short {
            assert!(matches!(outcome, Outcome::Limit("search output metadata")));
            drop(blocker);
            assert_eq!(budget.used(), captured + job);
            drop(input);
            assert_eq!(budget.used(), baseline);
        } else {
            let Outcome::Ready(InspectionLibraryOutput::Search(ready)) = outcome else {
                panic!("boxed ready")
            };
            assert!(matches!(ready.outcome(), SearchOutcome::NoMatch));
            assert_eq!(ready._metadata.bytes(), parent);
            assert_eq!(ready.scope.binding, view.render_binding());
            assert_eq!(budget.used(), CONSUMER_LIMIT);
            drop(blocker);
            assert_eq!(budget.used(), captured + worker);
            drop(input);
            assert_eq!(
                budget.used(),
                baseline + scope + request + query_extra + handle + parent
            );
            drop(ready);
            assert_eq!(budget.used(), baseline);
        }
    }
}

#[cfg(test)]
#[path = "document_inspection_work_tests.rs"]
mod work_tests;
