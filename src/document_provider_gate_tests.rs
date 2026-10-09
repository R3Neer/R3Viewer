//! Prospective P08/P09/P12 controls on real parser, viewport, providers and one kernel.
use crate::{
    document_inspector::preparation::{
        self, InspectionLibraryAuthority, InspectionLibraryInput, InspectionLibraryOutput,
        InspectionPreparationInput,
    },
    document_inspector_fixtures as fx,
    document_provider_gate::RefOwner,
    document_search::{SearchDomain, SearchOutcome, inspect_hit::SharedHit},
    document_text::{DocumentWork, Error, IndexProfile},
    document_view::DocumentView,
    preparation_owner::{Outcome, PreparationAuthority, PreparedCompletion},
    search::Direction,
};
use std::sync::{Arc, atomic::AtomicBool};
fn stop() -> AtomicBool {
    AtomicBool::new(false)
}
fn work(stop: &AtomicBool) -> DocumentWork<'_> {
    DocumentWork::preparation(stop)
}
pub(crate) fn run(
    input: Arc<InspectionLibraryInput>,
    authority: &mut InspectionLibraryAuthority,
) -> InspectionLibraryOutput {
    let binding = input.binding();
    if let Err(pending) = preparation::start(input, authority) {
        panic!("start {:?}", pending.cause)
    }
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !authority.is_finished() && std::time::Instant::now() < deadline {
        std::thread::yield_now()
    }
    let Some(PreparedCompletion::Document(outcome, receipt)) =
        authority.take_validated(binding, false).unwrap()
    else {
        panic!("joined document result")
    };
    assert!(receipt.joined);
    match outcome {
        Outcome::Ready(output) => output,
        Outcome::Failed(cause) => panic!("failed {cause:?}"),
        Outcome::Limit(cause) => panic!("limit {cause}"),
        Outcome::Cancelled => panic!("cancelled"),
    }
}
pub(crate) fn initial(raw: &str) -> (DocumentView, InspectionLibraryAuthority) {
    initial_document(fx::parsed(raw))
}
pub(crate) fn initial_document(
    document: Arc<crate::document_model::Document>,
) -> (DocumentView, InspectionLibraryAuthority) {
    let mut authority = PreparationAuthority::in_arena(document.actor_arena()).unwrap();
    let geometry = crate::document_layout::Geometry {
        x: 0,
        y: 0,
        width: 80,
        height: 24,
        left: 0,
    };
    let descriptor =
        DocumentView::initial_shared_inspectable_input(document, geometry, &authority).unwrap();
    let input = InspectionLibraryInput::viewport(descriptor, &mut work(&stop())).unwrap();
    let InspectionLibraryOutput::Viewport(prepared) = run(input, &mut authority) else {
        panic!("viewport")
    };
    (DocumentView::from_prepared(prepared).unwrap(), authority)
}
const TITLES: &str = "![bee](pin \"hold\") ![fox](one \"same\") ![fox](two \"same\")";
#[test]
fn p09_factory_busy_precedes_d1_and_weak_charge_lasts_until_purge() {
    let (view, _authority) = initial(TITLES);
    let gate = view.inspect_provider_gate().unwrap();
    let budget = view.budget();
    let (live, entries) = gate.occupied();
    assert_eq!(live, 1);
    assert!((1..=2).contains(&entries));
    let base = budget.used();
    let expected_weak_only =
        base + gate.permit_allocation_bytes() - (entries - live) * gate.permit_allocation_bytes();
    let profile = IndexProfile {
        projection_points: 2,
        unicode_points: 2,
        exceptions: 512,
    };
    let title = view.inspect_document().resources()[0].title_decoded();
    let pinned = RefOwner::prepare(
        view.inspect_document().clone(),
        title,
        budget,
        &stop(),
        profile,
        Some(gate),
    )
    .unwrap();
    assert_eq!(gate.occupied(), (2, 2));
    let held = budget.used();
    // Invalid RefID/profile would fail D1. Occupied gate must reject before either is read.
    let invalid = IndexProfile {
        projection_points: 0,
        unicode_points: 0,
        exceptions: 0,
    };
    assert!(matches!(
        RefOwner::prepare(
            view.inspect_document().clone(),
            u32::MAX,
            budget,
            &stop(),
            invalid,
            Some(gate)
        ),
        Err(Error::ProvidersBusy)
    ));
    assert_eq!(budget.used(), held);
    assert_eq!(gate.stats().2, 2);
    drop(pinned);
    assert_eq!(gate.occupied(), (1, 2));
    assert_eq!(budget.used(), expected_weak_only);
    let cancelled = AtomicBool::new(true);
    assert!(matches!(
        RefOwner::prepare(
            view.inspect_document().clone(),
            title,
            budget,
            &cancelled,
            profile,
            Some(gate)
        ),
        Err(Error::Cancelled)
    ));
    assert_eq!(gate.occupied(), (1, 2));
    let next = RefOwner::prepare(
        view.inspect_document().clone(),
        title,
        budget,
        &stop(),
        profile,
        Some(gate),
    )
    .unwrap();
    assert_eq!(gate.occupied(), (2, 2));
    drop(next);
    assert_eq!(budget.used(), expected_weak_only);
}
#[test]
fn p08_p12_pinned_ref_best_next_and_one_authority() {
    let (mut view, mut authority) = initial(TITLES);
    let gate = view.inspect_provider_gate().unwrap().clone();
    let pin = InspectionLibraryInput::search(
        &view,
        SearchDomain::ResourceTitle,
        "hold",
        91,
        Direction::Forward,
        &mut work(&stop()),
    )
    .unwrap();
    let InspectionLibraryOutput::Search(pin) = run(pin, &mut authority) else {
        panic!("pin result")
    };
    let hit = match pin.into_outcome(&view) {
        Ok(SearchOutcome::DocumentFound(hit)) => hit,
        _ => panic!("pin hit"),
    };
    assert_eq!(hit.resource_id(), Some(0));
    let pinned = SharedHit::try_share(hit, &mut work(&stop())).ok().unwrap();
    assert_eq!(gate.occupied().0, 2);
    let seed = InspectionPreparationInput::hit(&view, &pinned, &mut work(&stop())).unwrap();
    let retired = view.retire_cached_provider(&mut work(&stop())).unwrap();
    let task = InspectionLibraryInput::retire_and_inspect(retired, seed, &mut work(&stop()))
        .ok()
        .unwrap();
    let InspectionLibraryOutput::Inspector(retained) = run(task, &mut authority) else {
        panic!("inspection")
    };
    retained.check_view(&view, Some(&pinned)).unwrap();
    assert!(retained.result().is_ok());
    assert_eq!(gate.occupied().0, 1);
    let search = InspectionLibraryInput::search(
        &view,
        SearchDomain::ResourceTitle,
        "same",
        303,
        Direction::Backward,
        &mut work(&stop()),
    )
    .unwrap();
    let InspectionLibraryOutput::Search(found) = run(search, &mut authority) else {
        panic!("search")
    };
    let SearchOutcome::DocumentFound(hit) = found.outcome() else {
        panic!("ready backward")
    };
    assert_eq!(hit.resource_id(), Some(2));
    assert_eq!(hit.matched(), 0..4);
    assert_eq!(gate.occupied().0, 2);
    let next = found
        .continuation(Direction::Backward, &mut work(&stop()))
        .unwrap();
    drop(found);
    assert_eq!(gate.occupied().0, 1);
    let InspectionLibraryOutput::Search(found) = run(next, &mut authority) else {
        panic!("next")
    };
    let SearchOutcome::DocumentFound(hit) = found.outcome() else {
        panic!("ready next")
    };
    assert_eq!(hit.resource_id(), Some(1));
    let wrap = found
        .continuation(Direction::Backward, &mut work(&stop()))
        .unwrap();
    drop(found);
    let InspectionLibraryOutput::Search(found) = run(wrap, &mut authority) else {
        panic!("wrap")
    };
    let SearchOutcome::DocumentFound(hit) = found.outcome() else {
        panic!("ready wrap")
    };
    assert_eq!(hit.resource_id(), Some(2));
    assert_eq!(gate.stats().2, 2);
    assert_eq!(gate.stats().1, 0);
    drop(found);
    drop(retained);
    drop(pinned);
    assert_eq!(gate.occupied().0, 0);
}

#[test]
fn p00_legacy_three_resources_does_not_index_alt_owner_as_block() {
    let document = fx::parsed(TITLES);
    assert_eq!(document.resources().len(), 3);
    assert_eq!(document.blocks().len(), 2);
    let geometry = crate::document_layout::Geometry {
        x: 0,
        y: 0,
        width: 80,
        height: 24,
        left: 0,
    };
    let input = DocumentView::initial_input(document, geometry).unwrap();
    let binding = input.key().binding;
    let mut authority = crate::document_view::DocumentPreparationAuthority::default();
    crate::document_view::start_document(input, &mut authority).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !authority.is_finished() && std::time::Instant::now() < deadline {
        std::thread::yield_now()
    }
    let completion = authority.take_validated(binding, false);
    assert!(
        !matches!(completion, Err(crate::preparation_owner::Cause::Panic)),
        "ResourceAlt owner is ResourceID, not BlockID"
    );
    assert!(
        matches!(completion,Ok(Some(PreparedCompletion::Document(Outcome::Ready(_),receipt)))if receipt.joined)
    );
}

fn pinned(view: &DocumentView, authority: &mut InspectionLibraryAuthority) -> SharedHit {
    let input = InspectionLibraryInput::search(
        view,
        SearchDomain::ResourceTitle,
        "hold",
        91,
        Direction::Forward,
        &mut work(&stop()),
    )
    .unwrap();
    let InspectionLibraryOutput::Search(found) = run(input, authority) else {
        panic!("search")
    };
    let hit = match found.into_outcome(view) {
        Ok(SearchOutcome::DocumentFound(hit)) => hit,
        _ => panic!("hit"),
    };
    SharedHit::try_share(hit, &mut work(&stop())).ok().unwrap()
}
pub(crate) fn resource_cell(view: &DocumentView, id: usize) -> usize {
    use crate::document_presentation_anchor::{PresentationAnchor, RefOwner};
    let text = view.layout().text().unwrap();
    (0..text.glyphs().len()).find(|&at| matches!(
        text.cell_anchor(at, &mut work(&stop())).unwrap(),
        Some(PresentationAnchor::Ref(a)) if matches!(a.owner(), RefOwner::Resource { resource, .. } if resource as usize == id)
    )).unwrap()
}
fn anchor_fingerprint(anchor: crate::document_presentation_anchor::PresentationAnchor) -> String {
    use crate::document_presentation_anchor::PresentationAnchor;
    match anchor {
        PresentationAnchor::Domain(a) => format!("Domain:{a:?}"),
        PresentationAnchor::Role(a) => format!("Role:{a:?}"),
        PresentationAnchor::Ref(a) => format!("Ref:{a:?}"),
        PresentationAnchor::ResourceAlt(a) => format!("ResourceAlt:{a:?}"),
        PresentationAnchor::Table(a) => {
            format!("Table:{}:{}:{:?}", a.row(), a.selected(), a.cells())
        }
    }
}
fn frame(
    view: &DocumentView,
) -> (
    String,
    Vec<String>,
    Vec<Option<String>>,
    usize,
    crate::preparation_owner::Binding,
) {
    let text = view.layout().text().unwrap();
    let mut anchors = Vec::new();
    let mut glyphs = Vec::new();
    for at in 0..text.glyphs().len() {
        glyphs.push(format!("{:?}", text.glyphs().get(at).unwrap()));
        anchors.push(
            text.cell_anchor(at, &mut work(&stop()))
                .unwrap()
                .map(anchor_fingerprint),
        );
    }
    (
        text.symbols().to_owned(),
        glyphs,
        anchors,
        text.work,
        view.render_binding(),
    )
}
#[test]
fn p01_p02_old_page_pending_then_closed_handoff_admits_candidate() {
    use crate::document_inspector::Inspection;
    let (mut view, mut authority) = initial(TITLES);
    let gate = view.inspect_provider_gate().unwrap().clone();
    let hit = pinned(&view, &mut authority);
    let old = Inspection::from_hit(&hit, &view, &mut work(&stop())).unwrap();
    let page = old.page(None, &mut work(&stop())).unwrap();
    let action = old.source_component(&page, 0, &mut work(&stop())).unwrap();
    let before_binding = action.binding();
    let old_summary = old.summary_start(&mut work(&stop())).unwrap();
    let state = resource_cell(&view, 1);
    let before = view.budget().used();
    assert!(matches!(
        Inspection::from_view(&view, Some(state), None, &stop(), &mut work(&stop())),
        Err(Error::ProvidersBusy)
    ));
    assert_eq!(view.budget().used(), before);
    assert_eq!(gate.occupied().0, 2);
    assert_eq!(
        old.source_component(&page, 0, &mut work(&stop()))
            .unwrap()
            .binding(),
        before_binding
    );
    assert!(
        old.summary_advance(&old_summary, &mut work(&stop()))
            .is_ok()
    );
    let material = frame(&view);
    let seed = InspectionPreparationInput::view(&view, Some(state), &mut work(&stop())).unwrap();
    let retired = view.retire_cached_provider(&mut work(&stop())).unwrap();
    assert_eq!(gate.occupied().0, 2);
    assert_eq!(frame(&view), material);
    let input = InspectionLibraryInput::retire_and_inspect(retired, seed, &mut work(&stop()))
        .ok()
        .unwrap();
    let InspectionLibraryOutput::Inspector(prepared) = run(input, &mut authority) else {
        panic!("candidate")
    };
    prepared.check_view(&view, None).unwrap();
    assert!(prepared.result().is_ok());
    assert_eq!(gate.occupied().0, 2);
    assert_eq!(frame(&view), material);
    assert_eq!(action.binding(), view.render_binding());
    assert_ne!(hit.hit().query_revision(), action.binding().revision);
    assert!(old.source_component(&page, 0, &mut work(&stop())).is_ok());
}
#[test]
fn p04_p11_input_old_slot_alias_survives_handoff_and_pending() {
    use crate::{document_inspector::Inspection, document_view::Operation};
    let (mut view, mut authority) = initial(TITLES);
    let gate = view.inspect_provider_gate().unwrap().clone();
    let hit = pinned(&view, &mut authority);
    let old = Inspection::from_hit(&hit, &view, &mut work(&stop())).unwrap();
    let page = old.page(None, &mut work(&stop())).unwrap();
    let alias = view
        .request_shared(Operation::Resize, view.geometry(), &authority)
        .unwrap();
    let state = resource_cell(&view, 1);
    let seed = InspectionPreparationInput::view(&view, Some(state), &mut work(&stop())).unwrap();
    let material = frame(&view);
    let retired = view.retire_cached_provider(&mut work(&stop())).unwrap();
    let input = InspectionLibraryInput::retire_and_inspect(retired, seed, &mut work(&stop()))
        .ok()
        .unwrap();
    let InspectionLibraryOutput::Inspector(pending) = run(input, &mut authority) else {
        panic!("pending")
    };
    assert!(matches!(pending.result(), Err(Error::ProvidersBusy)));
    assert_eq!(gate.occupied().0, 2);
    assert_eq!(frame(&view), material);
    assert!(old.source_component(&page, 0, &mut work(&stop())).is_ok());
    drop(alias);
    assert_eq!(gate.occupied().0, 1);
    let input = InspectionLibraryInput::inspect(
        InspectionPreparationInput::view(&view, Some(state), &mut work(&stop())).unwrap(),
        &mut work(&stop()),
    )
    .unwrap();
    let InspectionLibraryOutput::Inspector(candidate) = run(input, &mut authority) else {
        panic!("candidate")
    };
    assert!(candidate.result().is_ok());
    assert_eq!(gate.occupied().0, 2);
    // CacheNone cannot bypass admission during explicit navigation reprepare.
    let input = view
        .request_shared(Operation::Home, view.geometry(), &authority)
        .unwrap();
    let input = InspectionLibraryInput::viewport(input, &mut work(&stop())).unwrap();
    let InspectionLibraryOutput::Viewport(state) = run(input, &mut authority) else {
        panic!("viewport")
    };
    assert!(view.commit(state).is_err());
    assert_eq!(frame(&view), material);
    assert_eq!(gate.occupied().0, 2);
}
#[test]
fn p03_p05_aliases_zero_slots_and_two_inspectors_block_third() {
    use crate::document_inspector::Inspection;
    let (mut view, mut authority) = initial(TITLES);
    let gate = view.inspect_provider_gate().unwrap().clone();
    let text = view.layout().text().unwrap();
    let last = view.inspect_document().resources()[2].title_decoded();
    let selected=(0..text.glyphs().len()).find(|&at|matches!(text.cell_anchor(at,&mut work(&stop())).unwrap(),Some(crate::document_presentation_anchor::PresentationAnchor::Ref(a))if a.text_ref()==last)).unwrap();
    let old =
        Inspection::from_view(&view, Some(selected), None, &stop(), &mut work(&stop())).unwrap();
    let page = old.page(None, &mut work(&stop())).unwrap();
    assert_eq!(gate.occupied().0, 1);
    let summary = old.summary_start(&mut work(&stop())).unwrap();
    let state = resource_cell(&view, 0);
    let other =
        Inspection::from_view(&view, Some(state), None, &stop(), &mut work(&stop())).unwrap();
    assert_eq!(gate.occupied().0, 2);
    let seed =
        InspectionPreparationInput::view(&view, Some(resource_cell(&view, 1)), &mut work(&stop()))
            .unwrap();
    let retired = view.retire_cached_provider(&mut work(&stop())).unwrap();
    let input = InspectionLibraryInput::retire_and_inspect(retired, seed, &mut work(&stop()))
        .ok()
        .unwrap();
    let InspectionLibraryOutput::Inspector(pending) = run(input, &mut authority) else {
        panic!("pending")
    };
    assert!(matches!(pending.result(), Err(Error::ProvidersBusy)));
    drop(old);
    assert_eq!(gate.occupied().0, 2);
    assert!(!page.values().is_empty());
    drop(page);
    assert_eq!(gate.occupied().0, 2);
    drop(summary);
    assert_eq!(gate.occupied().0, 1);
    let input = InspectionLibraryInput::inspect(
        InspectionPreparationInput::view(&view, Some(resource_cell(&view, 1)), &mut work(&stop()))
            .unwrap(),
        &mut work(&stop()),
    )
    .unwrap();
    let InspectionLibraryOutput::Inspector(candidate) = run(input, &mut authority) else {
        panic!("candidate")
    };
    assert!(candidate.result().is_ok());
    assert_eq!(gate.occupied().0, 2);
    drop(candidate);
    drop(other);
    assert_eq!(gate.occupied().0, 0);
}
#[test]
fn p06_p12_actual_authority_and_foreign_owner_rejected_with_rollback() {
    use crate::preparation_owner::Cause;
    let (view, _authority) = initial(TITLES);
    let input = InspectionLibraryInput::search(
        &view,
        SearchDomain::MainBody,
        "a",
        303,
        Direction::Forward,
        &mut work(&stop()),
    )
    .unwrap();
    let before = view.budget().used();
    let mut wrong = InspectionLibraryAuthority::default();
    let pending = preparation::start(input.clone(), &mut wrong).err().unwrap();
    assert_eq!(pending.cause, Cause::Stale);
    assert_eq!(view.budget().used(), before);
    drop(pending);
    let foreign = fx::parsed(TITLES);
    let mut wrong = PreparationAuthority::in_arena(foreign.actor_arena()).unwrap();
    let pending = preparation::start(input.clone(), &mut wrong).err().unwrap();
    assert_eq!(pending.cause, Cause::Stale);
    assert_eq!(view.budget().used(), before);
    drop(pending);
    let gate = view.inspect_provider_gate().unwrap();
    let foreign_budget = crate::document_consumer_budget::ConsumerBudget::new(&foreign).unwrap();
    let result = RefOwner::prepare(
        foreign,
        0,
        &foreign_budget,
        &stop(),
        IndexProfile {
            projection_points: 2,
            unicode_points: 2,
            exceptions: 512,
        },
        Some(gate),
    );
    assert!(matches!(result, Err(Error::Invalid)));
    assert_eq!(gate.occupied().0, 1);
}

#[test]
fn p10_capture_cancel_quota_and_retirement_rollback_preserve_real_owners() {
    use crate::{document_consumer_budget::CONSUMER_LIMIT, document_inspector::Inspection};
    let (mut view, mut authority) = initial(TITLES);
    let gate = view.inspect_provider_gate().unwrap().clone();
    let hit = pinned(&view, &mut authority);
    let old = Inspection::from_hit(&hit, &view, &mut work(&stop())).unwrap();
    let page = old.page(None, &mut work(&stop())).unwrap();
    let action = old.source_component(&page, 0, &mut work(&stop())).unwrap();
    let material = frame(&view);
    let before = view.budget().used();
    assert!(matches!(
        view.retire_cached_provider(&mut DocumentWork::bounded(&stop(), 0).unwrap()),
        Err(Error::Work)
    ));
    assert_eq!(view.budget().used(), before);
    let cancelled = AtomicBool::new(true);
    assert!(matches!(
        view.retire_cached_provider(&mut work(&cancelled)),
        Err(Error::Cancelled)
    ));
    assert_eq!(frame(&view), material);
    assert_eq!(gate.occupied().0, 2);
    assert!(matches!(
        InspectionPreparationInput::view(
            &view,
            Some(resource_cell(&view, 1)),
            &mut work(&cancelled)
        ),
        Err(Error::Cancelled)
    ));
    assert_eq!(view.budget().used(), before);
    let seed =
        InspectionPreparationInput::view(&view, Some(resource_cell(&view, 1)), &mut work(&stop()))
            .unwrap();
    let retired = view.retire_cached_provider(&mut work(&stop())).unwrap();
    let held = view.budget().used();
    let required = std::mem::size_of::<crate::document_view::CacheRetirement>()
        + std::mem::size_of::<InspectionPreparationInput>();
    let (cause, retired, seed) = InspectionLibraryInput::retire_and_inspect(
        retired,
        seed,
        &mut DocumentWork::bounded(&stop(), required - 1).unwrap(),
    )
    .err()
    .unwrap();
    assert_eq!(cause, Error::Work);
    assert_eq!(view.budget().used(), held);
    assert_eq!(gate.occupied().0, 2);
    assert_eq!(frame(&view), material);
    let full = view
        .budget()
        .reserve(CONSUMER_LIMIT - view.budget().used())
        .unwrap();
    let (error, retired, seed) =
        InspectionLibraryInput::retire_and_inspect(retired, seed, &mut work(&stop()))
            .err()
            .unwrap();
    assert_eq!(error, Error::Budget);
    assert_eq!(gate.occupied().0, 2);
    assert_eq!(frame(&view), material);
    assert_eq!(action.binding(), view.render_binding());
    drop(full);
    let input = InspectionLibraryInput::retire_and_inspect(retired, seed, &mut work(&stop()))
        .ok()
        .unwrap();
    let InspectionLibraryOutput::Inspector(prepared) = run(input, &mut authority) else {
        panic!("inspection")
    };
    assert!(prepared.result().is_ok());
    assert!(old.source_component(&page, 0, &mut work(&stop())).is_ok());
    assert_eq!(gate.stats().2, 2);
}
#[test]
fn p10_p12_actual_compiled_job_quota_rollback_then_same_input_join() {
    use crate::{
        document_consumer_budget::CONSUMER_LIMIT,
        preparation_owner::{Cause, OwnedPreparation},
    };
    let (view, mut authority) = initial(TITLES);
    let input = InspectionLibraryInput::search(
        &view,
        SearchDomain::MainBody,
        "missing",
        303,
        Direction::Forward,
        &mut work(&stop()),
    )
    .unwrap();
    type Run = fn(&InspectionLibraryInput, &AtomicBool) -> Outcome<InspectionLibraryOutput>;
    let run: Run = |_, _| Outcome::Cancelled;
    let required =
        OwnedPreparation::<InspectionLibraryInput, InspectionLibraryOutput>::known_spawn_bytes(
            &run,
        )
        .unwrap()
            - std::mem::size_of::<InspectionLibraryInput>()
            - 2 * std::mem::size_of::<usize>();
    let before = view.budget().used();
    let block = view
        .budget()
        .reserve(CONSUMER_LIMIT - before - required + 1)
        .unwrap();
    let blocked = view.budget().used();
    let pending = preparation::start(input.clone(), &mut authority)
        .err()
        .unwrap();
    assert_eq!(pending.cause, Cause::Limit("inspection job quota"));
    assert!(Arc::ptr_eq(&pending.input, &input));
    assert_eq!(view.budget().used(), blocked);
    drop(pending);
    drop(block);
    assert_eq!(view.budget().used(), before);
    let output = run_input(input.clone(), &mut authority);
    assert!(matches!(output, InspectionLibraryOutput::Search(_)));
    drop(output);
    drop(input);
    assert!(view.budget().used() < before);
}
fn run_input(
    input: Arc<InspectionLibraryInput>,
    authority: &mut InspectionLibraryAuthority,
) -> InspectionLibraryOutput {
    run(input, authority)
}
#[test]
fn p10_p12_cancelled_and_stale_join_keep_old_page_and_full_binding() {
    use crate::{document_inspector::Inspection, preparation_owner::Cause};
    let (view, mut authority) = initial(TITLES);
    let hit = pinned(&view, &mut authority);
    let old = Inspection::from_hit(&hit, &view, &mut work(&stop())).unwrap();
    let page = old.page(None, &mut work(&stop())).unwrap();
    let action = old.source_component(&page, 0, &mut work(&stop())).unwrap();
    for cancelled in [true, false] {
        let seed = InspectionPreparationInput::hit(&view, &hit, &mut work(&stop())).unwrap();
        let input = InspectionLibraryInput::inspect(seed, &mut work(&stop())).unwrap();
        let binding = input.binding();
        if let Err(pending) = preparation::start(input, &mut authority) {
            panic!("start {:?}", pending.cause)
        }
        if cancelled {
            authority.stop();
        }
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while !authority.is_finished() && std::time::Instant::now() < deadline {
            std::thread::yield_now()
        }
        let mut expected = binding;
        if !cancelled {
            expected.revision += 1;
        }
        let result = authority.take_validated(expected, false);
        assert!(
            matches!(result,Err(cause)if cause==if cancelled{Cause::Cancelled}else{Cause::Stale})
        );
        assert_eq!(action.binding(), view.render_binding());
        assert!(old.source_component(&page, 0, &mut work(&stop())).is_ok());
        assert_eq!(view.inspect_provider_gate().unwrap().occupied().0, 2);
    }
}
#[test]
fn p11_nonterminal_page_alias_retains_actual_provider_after_inspection_drop() {
    use crate::document_inspector::Inspection;
    let (mut view, mut authority) = initial_document(fx::cluster65(true));
    let gate = view.inspect_provider_gate().unwrap().clone();
    let old = Inspection::from_view(&view, Some(0), None, &stop(), &mut work(&stop())).unwrap();
    let page = old.page(None, &mut work(&stop())).unwrap();
    assert_eq!(page.values().len(), 64);
    assert!(page.next().is_some());
    assert_eq!(gate.occupied().0, 1);
    let seed = InspectionPreparationInput::view(&view, Some(0), &mut work(&stop())).unwrap();
    let retired = view.retire_cached_provider(&mut work(&stop())).unwrap();
    let input = InspectionLibraryInput::retire_and_inspect(retired, seed, &mut work(&stop()))
        .ok()
        .unwrap();
    let prepared = run(input, &mut authority);
    drop(prepared);
    assert_eq!(gate.occupied().0, 1);
    drop(old);
    assert_eq!(gate.occupied().0, 1);
    assert_eq!(page.values().len(), 64);
    drop(page);
    assert_eq!(gate.occupied().0, 0);
}
#[test]
fn p07_p13_legacy_direct_d1_and_scoped_heading_factory() {
    use crate::{
        document_consumer_budget::ConsumerBudget, document_search::HeadingCursor,
        document_text::TextRefIndex,
    };
    let document = fx::parsed(TITLES);
    let budget = ConsumerBudget::new(&document).unwrap();
    let profile = IndexProfile {
        projection_points: 2,
        unicode_points: 2,
        exceptions: 512,
    };
    let id = document.resources()[0].title_decoded();
    let baseline = budget.used();
    let mut providers = Vec::new();
    for _ in 0..4 {
        providers
            .push(TextRefIndex::prepare(document.clone(), id, &budget, &stop(), profile).unwrap());
    }
    assert_eq!(providers.len(), 4);
    drop(providers);
    assert_eq!(budget.used(), baseline);
    let (view, mut authority) = initial(&format!("## heading\n\n{TITLES}"));
    let hit = pinned(&view, &mut authority);
    let gate = view.inspect_provider_gate().unwrap();
    assert_eq!(gate.occupied().0, 2);
    let cursor = HeadingCursor::start(view.inspect_document().clone(), view.budget()).unwrap();
    let page = cursor.page(view.budget(), &mut work(&stop())).unwrap();
    assert_eq!(page.targets().len(), 1);
    let before = view.budget().used();
    assert!(matches!(
        page.targets()[0].visit_title_in_view(&view, &stop(), &mut work(&stop()), |_| Ok(())),
        Err(Error::ProvidersBusy)
    ));
    assert_eq!(view.budget().used(), before);
    assert_eq!(gate.stats().2, 2);
    drop(hit);
    let mut actual = String::new();
    page.targets()[0]
        .visit_title_in_view(&view, &stop(), &mut work(&stop()), |chunk| {
            actual.push_str(chunk);
            Ok(())
        })
        .unwrap();
    assert_eq!(actual, "heading");
    assert_eq!(gate.occupied().0, 1);
}
#[test]
fn p10_scoped_found_pending_owner_and_retry_preserve_old_page() {
    use crate::{document_inspector::Inspection, document_search::SearchRequest};
    let (mut view, mut authority) = initial(TITLES);
    let gate = view.inspect_provider_gate().unwrap().clone();
    let pinned = pinned(&view, &mut authority);
    let old = Inspection::from_hit(&pinned, &view, &mut work(&stop())).unwrap();
    let page = old.page(None, &mut work(&stop())).unwrap();
    let action = old.source_component(&page, 0, &mut work(&stop())).unwrap();
    let seed = InspectionPreparationInput::hit(&view, &pinned, &mut work(&stop())).unwrap();
    let retired = view.retire_cached_provider(&mut work(&stop())).unwrap();
    let input = InspectionLibraryInput::retire_and_inspect(retired, seed, &mut work(&stop()))
        .ok()
        .unwrap();
    let retained = run(input, &mut authority);
    assert_eq!(gate.occupied().0, 1);
    let request =
        SearchRequest::from_view(&view, SearchDomain::ResourceTitle, "same", 303, &stop()).unwrap();
    let mut pending = None;
    for limit in 0..4096 {
        let outcome = request.find_with_work(
            Direction::Forward,
            None,
            &stop(),
            &mut DocumentWork::bounded(&stop(), limit).unwrap(),
        );
        if let SearchOutcome::DocumentFoundPending(hit, Error::Work) = outcome {
            pending = Some(hit);
            break;
        }
    }
    let pending = pending.expect("bounded Work reaches accredited semantic pending");
    assert_eq!(pending.resource_id(), Some(1));
    assert_eq!(pending.domain(), SearchDomain::ResourceTitle);
    assert_eq!(pending.query_revision(), 303);
    assert_eq!(pending.matched(), 0..4);
    assert_eq!(pending.highlight(), None);
    assert_eq!(gate.occupied().0, 1);
    let anchor = pending.anchor();
    let SearchOutcome::DocumentFound(ready) = request.retry_hit(&pending, &stop()) else {
        panic!("retry")
    };
    assert_eq!(ready.resource_id(), Some(1));
    assert_eq!(ready.anchor(), anchor);
    assert_eq!(ready.matched(), 0..4);
    assert_eq!(ready.highlight(), Some(0..4));
    assert_eq!(gate.occupied().0, 2);
    assert_eq!(gate.stats().2, 2);
    assert_eq!(action.binding(), view.render_binding());
    assert!(old.source_component(&page, 0, &mut work(&stop())).is_ok());
    drop(retained);
}
