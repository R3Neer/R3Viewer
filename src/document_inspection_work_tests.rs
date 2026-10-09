//! CGR1-W01: prospective compiled-body Work limits on real scoped owners.
use super::*;
use crate::document_provider_gate_tests as gate;

fn shared(view: &DocumentView) -> SharedHit {
    let stop = AtomicBool::new(false);
    let request = SearchRequest::from_view(view, SearchDomain::MainBody, "ab", 303, &stop).unwrap();
    let SearchOutcome::DocumentFound(hit) = request.find(Direction::Forward, None, &stop) else {
        panic!("actual scoped hit")
    };
    SharedHit::try_share(hit, &mut DocumentWork::preparation(&stop))
        .ok()
        .unwrap()
}
fn seed_limits<F>(view: &DocumentView, mut make: F)
where
    F: FnMut(&mut DocumentWork<'_>) -> Result<InspectionPreparationInput>,
{
    let stop = AtomicBool::new(false);
    let budget = view.budget();
    let baseline = budget.used();
    let gate = view.inspect_provider_gate().unwrap();
    let occupied = gate.occupied();
    let binding = view.render_binding();
    let expected = size_of::<Scope>() + size_of::<InspectionPreparationInput>();
    let consumer =
        size_of::<Scope>() + 4 * size_of::<usize>() + size_of::<InspectionPreparationInput>();
    for minus in [1usize, 0] {
        let mut work = DocumentWork::bounded(&stop, expected - minus).unwrap();
        let result = make(&mut work);
        if minus == 1 {
            assert!(
                matches!(result, Err(Error::Work)),
                "complete seed body exact-1 must reject"
            );
            assert_eq!(budget.used(), baseline);
        } else {
            assert!(result.is_ok());
            assert_eq!(work.used(), expected);
            assert_eq!(budget.used(), baseline + consumer);
        }
        drop(result);
        assert_eq!(budget.used(), baseline);
        assert_eq!(gate.occupied(), occupied);
        assert_eq!(view.render_binding(), binding);
    }
    let cancelled = AtomicBool::new(true);
    let mut work = DocumentWork::bounded(&cancelled, expected).unwrap();
    assert!(matches!(make(&mut work), Err(Error::Cancelled)));
    assert_eq!(work.used(), 0);
    assert_eq!(budget.used(), baseline);
    assert_eq!(gate.occupied(), occupied);
}
#[test]
fn cgr1_w01_hit_seed_work_exact_minus_one_rollback() {
    let (view, _authority) = gate::initial("ab");
    let hit = shared(&view);
    seed_limits(&view, |work| {
        InspectionPreparationInput::hit(&view, &hit, work)
    });
    assert_eq!(hit.hit().highlight(), Some(0..2));
    assert_eq!(hit.hit().query_revision(), 303);
}
#[test]
fn cgr1_w01_view_seed_work_exact_minus_one_rollback() {
    let (view, _authority) = gate::initial("ab");
    seed_limits(&view, |work| {
        InspectionPreparationInput::view(&view, None, work)
    });
}
#[test]
fn cgr1_w01_cursor_seed_work_exact_minus_one_rollback() {
    let (view, _authority) = gate::initial("ab");
    let state = view.navigator().current().unwrap();
    seed_limits(&view, |work| {
        InspectionPreparationInput::cursor(&view, state, work)
    });
}
fn proof_limits(view: &DocumentView, seed: &InspectionPreparationInput) {
    let stop = AtomicBool::new(false);
    let baseline = view.budget().used();
    let occupied = view.inspect_provider_gate().unwrap().occupied();
    let expected = size_of::<InspectionProof>();
    for minus in [1usize, 0] {
        let mut work = DocumentWork::bounded(&stop, expected - minus).unwrap();
        let proof = seed.proof(&mut work);
        if minus == 1 {
            assert!(
                matches!(proof, Err(Error::Work)),
                "complete proof body exact-1 must reject"
            );
            assert_eq!(view.budget().used(), baseline);
        } else {
            assert!(proof.is_ok());
            assert_eq!(work.used(), expected);
            assert_eq!(view.budget().used(), baseline + expected);
        }
        drop(proof);
        assert_eq!(view.budget().used(), baseline);
        assert_eq!(view.inspect_provider_gate().unwrap().occupied(), occupied);
    }
    let cancelled = AtomicBool::new(true);
    assert!(matches!(
        seed.proof(&mut DocumentWork::bounded(&cancelled, expected).unwrap()),
        Err(Error::Cancelled)
    ));
    assert_eq!(view.budget().used(), baseline);
}
#[test]
fn cgr1_w01_hit_proof_work_exact_minus_one_rollback() {
    let (view, _authority) = gate::initial("ab");
    let hit = shared(&view);
    let stop = AtomicBool::new(false);
    let seed = InspectionPreparationInput::hit(&view, &hit, &mut DocumentWork::preparation(&stop))
        .unwrap();
    proof_limits(&view, &seed);
    assert_eq!(hit.hit().highlight(), Some(0..2));
}
#[test]
fn cgr1_w01_anchor_proof_work_exact_minus_one_rollback() {
    let (view, _authority) = gate::initial("ab");
    let stop = AtomicBool::new(false);
    let seed = InspectionPreparationInput::view(&view, None, &mut DocumentWork::preparation(&stop))
        .unwrap();
    proof_limits(&view, &seed);
}
#[test]
fn cgr1_w01_seed_consumer_exact_minus_one_physical_rollback() {
    let (view, _authority) = gate::initial("ab");
    let hit = shared(&view);
    let stop = AtomicBool::new(false);
    let budget = view.budget();
    let baseline = budget.used();
    let need =
        size_of::<Scope>() + 4 * size_of::<usize>() + size_of::<InspectionPreparationInput>();
    for kind in 0..3 {
        for minus in [1usize, 0] {
            let blocker: ChargedVec<u8> = ChargedVec::new(
                budget,
                crate::document_consumer_budget::CONSUMER_LIMIT - baseline - need + minus,
            )
            .unwrap();
            let mut work = DocumentWork::preparation(&stop);
            let seed = match kind {
                0 => InspectionPreparationInput::hit(&view, &hit, &mut work),
                1 => InspectionPreparationInput::view(&view, None, &mut work),
                _ => InspectionPreparationInput::cursor(
                    &view,
                    view.navigator().current().unwrap(),
                    &mut work,
                ),
            };
            if minus == 1 {
                assert!(matches!(seed, Err(Error::Budget)));
            } else {
                assert!(seed.is_ok());
                assert_eq!(
                    budget.used(),
                    crate::document_consumer_budget::CONSUMER_LIMIT
                );
            }
            drop(seed);
            drop(blocker);
            assert_eq!(budget.used(), baseline);
            assert_eq!(hit.hit().highlight(), Some(0..2));
        }
    }
}
