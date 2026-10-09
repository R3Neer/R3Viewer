use super::*;
use std::{
    thread,
    time::{Duration, Instant},
};
fn geometry() -> Geometry {
    Geometry {
        x: 0,
        y: 0,
        width: 80,
        height: 24,
        left: 0,
    }
}
fn parsed(arena: &crate::document_budget::ActorArena) -> Arc<Document> {
    let stop = AtomicBool::new(false);
    let ctl = crate::document_obsidian::Control {
        cancel: &stop,
        deadline: Instant::now() + Duration::from_secs(10),
    };
    let source = Arc::new(
        crate::document_source::DocumentSource::new(
            Arc::new("BODY\n".to_owned()),
            crate::input::SourceOrigin::Literal,
            7,
            11,
            &stop,
        )
        .unwrap(),
    );
    let c = crate::document_parser::project(
        source,
        crate::document_model::Profile::Extended,
        arena.clone(),
        &ctl,
    )
    .unwrap();
    let depth = crate::document_validate::validate(&c, &ctl).unwrap();
    let validated = crate::document_validate::quarantine(c, depth, &ctl).unwrap();
    Arc::new(Document::from_validated(validated))
}
fn finish(a: &mut DocumentPreparationAuthority, b: Binding) -> PreparedState {
    let until = Instant::now() + Duration::from_secs(5);
    while !a.is_finished() && Instant::now() < until {
        thread::yield_now();
    }
    assert!(a.is_finished());
    let Some(crate::preparation_owner::PreparedCompletion::Document(Outcome::Ready(p), r)) =
        a.take_validated(b, false).unwrap()
    else {
        panic!("actualDoc")
    };
    assert_eq!(
        r.closure_bytes,
        OwnedPreparation::<DocumentPreparationInput, PreparedState>::known_capture_layout::<
            DocumentRunner,
        >()
    );
    assert_eq!(
        r.authority_box_bytes + r.authority_value_bytes,
        DocumentPreparationAuthority::known_authority_bytes().unwrap()
    );
    p
}
#[test]
fn metadata01_compiled_delta24_and_one_actor_authority() {
    type Owner = OwnedPreparation<DocumentPreparationInput, PreparedState>;
    eprintln!(
        "DOC layouts input={} inputArc={} runner={} parts={} completion={} finalReceipt={} stopArc={} ownerValue={} sharedDescriptor={}",
        size_of::<DocumentPreparationInput>(),
        size_of::<DocumentPreparationInput>() + 2 * size_of::<usize>(),
        size_of::<DocumentRunner>(),
        Owner::known_parts_bytes(),
        Owner::known_completion_bytes(),
        Owner::known_final_receipt_bytes(),
        Owner::known_stop_arc_bytes().unwrap(),
        Owner::known_owner_value_bytes(),
        size_of::<SharedDocumentDescriptor>()
    );
    let prior = size_of::<DocumentPreparationInput>()
        + 2 * size_of::<usize>()
        + Owner::known_parts_bytes()
        + Owner::known_completion_bytes()
        + size_of::<Binding>()
        + 4 * size_of::<usize>()
        + 64;
    let default = input_metadata_bytes(false, KernelFunding::Legacy).unwrap();
    let scoped = input_metadata_bytes(false, KernelFunding::Actor).unwrap();
    assert_eq!(default - prior, 24);
    assert_eq!(default - scoped, 48);
    assert_eq!(
        input_metadata_bytes(false, KernelFunding::Shared).unwrap(),
        size_of::<DocumentPreparationInput>() + 2 * size_of::<usize>()
    );
    eprintln!(
        "DOC metadata default={default} actor={scoped} prior={prior} capture={} finalReceiptExcluded={} authority={}",
        Owner::known_capture_layout::<DocumentRunner>(),
        Owner::known_final_receipt_bytes(),
        DocumentPreparationAuthority::known_authority_bytes().unwrap()
    );
}
#[test]
fn metadata02_default_and_actor_nearquota_atomic() {
    for scoped in [false, true] {
        let arena = crate::document_budget::ActorArena::new();
        let document = parsed(&arena);
        let mut a = if scoped {
            DocumentPreparationAuthority::in_arena(&arena).unwrap()
        } else {
            DocumentPreparationAuthority::default()
        };
        let input = if scoped {
            DocumentView::initial_input_in(document, geometry(), &a)
        } else {
            DocumentView::initial_input(document, geometry())
        }
        .unwrap();
        let b = input.key().binding;
        start_document(input, &mut a).unwrap();
        let p = finish(&mut a, b);
        let mut v = DocumentView::from_prepared(p).unwrap();
        let prior_binding = v.render_binding();
        let prior = v.layout().text().unwrap().symbols().to_owned();
        let before = v.budget.used();
        let input = if scoped {
            v.request_in(Operation::Resize, geometry(), &a)
        } else {
            v.request(Operation::Resize, geometry())
        }
        .unwrap();
        let bytes = v.budget.used() - before;
        drop(input);
        assert_eq!(v.budget.used(), before);
        let block = v.budget.reserve(1048576 - before - (bytes - 1)).unwrap();
        let used = v.budget.used();
        assert!(
            if scoped {
                v.request_in(Operation::Resize, geometry(), &a)
            } else {
                v.request(Operation::Resize, geometry())
            }
            .is_err()
        );
        assert_eq!(v.budget.used(), used);
        assert_eq!(v.render_binding(), prior_binding);
        assert_eq!(v.layout().text().unwrap().symbols(), prior);
        drop(block);
        let block = v.budget.reserve(1048576 - before - bytes).unwrap();
        let input = if scoped {
            v.request_in(Operation::Resize, geometry(), &a)
        } else {
            v.request(Operation::Resize, geometry())
        }
        .unwrap();
        assert_eq!(v.budget.used(), 1048576);
        drop(input);
        drop(block);
        assert_eq!(v.budget.used(), before);
    }
}
#[test]
fn metadata03_closed_shared_descriptor_cannot_spawn_standalone() {
    let arena = crate::document_budget::ActorArena::new();
    let document = parsed(&arena);
    let mut a = DocumentPreparationAuthority::in_arena(&arena).unwrap();
    let shared = DocumentView::initial_shared_input(document, geometry(), &a).unwrap();
    assert_eq!(
        shared.metadata_bytes(),
        input_metadata_bytes(false, KernelFunding::Shared).unwrap()
    );
    assert_eq!(
        start_document(shared.input.clone(), &mut a).err(),
        Some(Cause::Stale)
    );
    assert!(a.is_finished());
    let budget = shared.budget().clone();
    let before = budget.used();
    let job = budget
        .reserve(
            OwnedPreparation::<DocumentPreparationInput, PreparedState>::known_job_bytes(
                &(|_: &DocumentPreparationInput, _: &AtomicBool| Outcome::Cancelled),
            )
            .unwrap(),
        )
        .unwrap();
    assert!(budget.used() > before);
    let Outcome::Ready(p) = prepare_shared_document(&shared, &AtomicBool::new(false)) else {
        panic!("actual shared preparation")
    };
    let v = DocumentView::from_prepared(p).unwrap();
    assert_eq!(v.layout().text().unwrap().symbols(), "BODY");
    drop(job);
}

#[test]
fn metadata04_actor_input_rejects_default_and_foreign_before_spawn() {
    let arena = crate::document_budget::ActorArena::new();
    let document = parsed(&arena);
    let captured = DocumentPreparationAuthority::in_arena(&arena).unwrap();
    let input = DocumentView::initial_input_in(document, geometry(), &captured).unwrap();
    let pointer = Arc::as_ptr(&input);
    let context = Arc::as_ptr(&input.source);
    let binding = input.key().binding;
    let used = input.budget().used();
    drop(captured);
    let mut default = DocumentPreparationAuthority::default();
    assert_eq!(
        start_document(input.clone(), &mut default).err(),
        Some(Cause::Stale)
    );
    assert!(default.is_finished());
    let other = crate::document_budget::ActorArena::new();
    let mut foreign = DocumentPreparationAuthority::in_arena(&other).unwrap();
    assert_eq!(
        start_document(input.clone(), &mut foreign).err(),
        Some(Cause::Stale)
    );
    assert!(foreign.is_finished());
    assert_eq!(Arc::as_ptr(&input), pointer);
    assert_eq!(Arc::as_ptr(&input.source), context);
    assert_eq!(input.key().binding, binding);
    assert_eq!(input.budget().used(), used);
    let mut actual = DocumentPreparationAuthority::in_arena(&arena).unwrap();
    start_document(input.clone(), &mut actual).unwrap();
    let view = DocumentView::from_prepared(finish(&mut actual, binding)).unwrap();
    assert_eq!(view.layout().text().unwrap().symbols(), "BODY");
}
