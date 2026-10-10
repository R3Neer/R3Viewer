//! Incremental physical-owner regressions; real worker required and process globals serialized.
use super::*;
fn live_retained() -> (
    SyntaxClient,
    Arc<RetainedSyntaxInput>,
    crate::document_consumer_budget::ConsumerBudget,
    OperationContext,
) {
    assert!(std::env::var_os("R3VIEWER_SYNTAX_TEST_EXE").is_some());
    let document = retained::fixture_document();
    let budget = crate::document_consumer_budget::ConsumerBudget::new(&document).unwrap();
    let context = OperationContext::document(
        301,
        Instant::now() + Duration::from_secs(8),
        Arc::new(AtomicBool::new(false)),
    );
    let input =
        RetainedSyntaxInput::copy_document("let value=1;\n", &document, &budget, &context).unwrap();
    let mut client =
        SyntaxClient::new(SyntaxLimits::default(), Arc::new(AtomicBool::new(false))).unwrap();
    client
        .set_retained_document(
            input.clone(),
            "Rust".into(),
            1,
            SyntaxLimits::default(),
            context.clone(),
        )
        .unwrap();
    let request = client.request_retained(1, None, &context);
    let until = Instant::now() + Duration::from_secs(4);
    loop {
        if let Some(RetainedSyntaxEvent::Styles {
            generation: 1,
            request: actual,
            ..
        }) = client.poll_retained()
        {
            assert_eq!(actual, request);
            break;
        }
        assert!(Instant::now() < until, "real Styles");
        thread::sleep(Duration::from_millis(2));
    }
    (client, input, budget, context)
}
fn pending(client: &SyntaxClient) {
    SyntaxClient::own_close_pipe_fault(true);
    client.pause_document(1);
    let until = Instant::now() + Duration::from_secs(4);
    while client.cleanup_status() != CleanupStatus::Pending && Instant::now() < until {
        thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(client.cleanup_status(), CleanupStatus::Pending);
}
fn reap(client: &SyntaxClient) {
    SyntaxClient::own_close_pipe_fault(false);
    client.retry_cleanup();
    let until = Instant::now() + Duration::from_secs(4);
    while PENDING_PRESENT.load(Ordering::Acquire) && Instant::now() < until {
        thread::sleep(Duration::from_millis(2));
    }
    assert!(!PENDING_PRESENT.load(Ordering::Acquire));
}
#[test]
#[cfg(windows)]
#[ignore = "requires explicit previously built native worker path"]
fn r2_live_owner_foreign_retry_rejected_original_same_thread_recovers() {
    let (mut a, input, budget, context) = live_retained();
    let thread = a.own_controller_thread();
    pending(&a);
    let before = budget.used();
    let weak = Arc::downgrade(&input);
    let b = SyntaxClient::new(SyntaxLimits::default(), Arc::new(AtomicBool::new(false))).unwrap();
    let memory = b.memory_usage();
    let generation = b.current_generation();
    b.retry_cleanup();
    let until = Instant::now() + Duration::from_secs(1);
    while b.shared.lock().retry_cleanup && Instant::now() < until {
        thread::sleep(Duration::from_millis(2));
    }
    let untouched = {
        let slot = pending_slot().lock().unwrap();
        let owner = slot.as_ref().unwrap();
        Arc::ptr_eq(&owner.owner_shared, &a.shared) && Arc::ptr_eq(&owner.input, &input)
    };
    let rejected =
        PENDING_PRESENT.load(Ordering::Acquire) && a.cleanup_status() == CleanupStatus::Pending;
    let charges = budget.used() == before;
    let foreign_unchanged = b.current_generation() == generation
        && format!("{:?}", b.memory_usage()) == format!("{:?}", memory);
    reap(&a);
    let physical = matches!(a.cleanup_status(), CleanupStatus::Reaped { failed: true });
    let same_thread = a.own_controller_thread() == thread;
    // New descriptor, complete new context. The failed request is never replayed.
    let fresh = OperationContext::document(
        302,
        Instant::now() + Duration::from_secs(8),
        Arc::new(AtomicBool::new(false)),
    );
    a.set_retained_document(
        input.clone(),
        "Rust".into(),
        2,
        SyntaxLimits::default(),
        fresh.clone(),
    )
    .unwrap();
    let request = a.request_retained(2, None, &fresh);
    let until = Instant::now() + Duration::from_secs(4);
    let mut ready = false;
    while Instant::now() < until {
        if let Some(RetainedSyntaxEvent::Styles {
            generation: 2,
            request: actual,
            ..
        }) = a.poll_retained()
        {
            ready = actual == request;
            break;
        }
        thread::sleep(Duration::from_millis(2));
    }
    a.shutdown().join().unwrap();
    b.shutdown().join().unwrap();
    assert!(
        untouched && rejected && charges && foreign_unchanged && physical && same_thread && ready
    );
    assert!(weak.upgrade().is_some());
    drop(input);
    drop(context);
    assert!(weak.upgrade().is_none());
    eprintln!(
        "SA14 actualPendingA foreignBnoSpawn untouchedOwner/cargos sameThreadA retry newContext2 PASS"
    );
}
#[test]
#[cfg(windows)]
#[ignore = "requires explicit previously built native worker path"]
fn r2_stopped_owner_backstop_reaps_original_shared_without_resetting_foreign() {
    let (a, input, budget, _context) = live_retained();
    let original = a.shared.clone();
    let weak = Arc::downgrade(&input);
    pending(&a);
    a.shutdown().join().unwrap();
    assert!(original.lock().stopping);
    let b = SyntaxClient::new(SyntaxLimits::default(), Arc::new(AtomicBool::new(false))).unwrap();
    let memory = b.memory_usage();
    let generation = b.current_generation();
    reap(&b);
    let original_reaped = matches!(
        original.lock().cleanup,
        CleanupStatus::Reaped { failed: true }
    );
    let foreign_unchanged = format!("{:?}", b.memory_usage()) == format!("{:?}", memory)
        && b.current_generation() == generation;
    b.shutdown().join().unwrap();
    assert!(original_reaped && foreign_unchanged);
    assert!(weak.upgrade().is_some());
    assert!(budget.used() > 0);
    drop(original);
    drop(input);
    assert!(weak.upgrade().is_none());
    eprintln!(
        "SA15 actualShutdownA ownerArc retained orphanReapB updatesOriginalOnly aliasesCharged PASS"
    );
}

#[test]
#[cfg(windows)]
#[ignore = "requires explicit previously built native worker path"]
fn r2_connection_owner_metadata_exact_and_minus_one_admit_before_spawn() {
    assert!(std::env::var_os("R3VIEWER_SYNTAX_TEST_EXE").is_some());
    let mut saw_exact = false;
    let mut saw_minus = false;
    for minus in [1usize, 0] {
        let client = super::tests::dormant_client();
        let document = retained::fixture_document();
        let budget = crate::document_consumer_budget::ConsumerBudget::new(&document).unwrap();
        let context = OperationContext::document(
            303,
            Instant::now() + Duration::from_secs(8),
            Arc::new(AtomicBool::new(false)),
        );
        let input =
            RetainedSyntaxInput::copy_document("let value=1;\n", &document, &budget, &context)
                .unwrap();
        client
            .set_retained_document(input, "Rust".into(), 1, SyntaxLimits::default(), context)
            .unwrap();
        let Some(Update::Set(doc)) = client.shared.lock().update.take() else {
            panic!("admitted descriptor");
        };
        let need = size_of::<Connection>()
            + size_of::<Shared>()
            + 2 * size_of::<usize>()
            + size_of::<Outgoing>()
            + 4096;
        let held = budget
            .reserve(
                crate::document_consumer_budget::CONSUMER_LIMIT - budget.used() - (need - minus),
            )
            .unwrap();
        let before = budget.used();
        match Connection::spawn(&doc, &client.shared) {
            Err((FallbackReason::SourceBudget, None)) if minus == 1 => {
                saw_minus = budget.used() == before && !SYNTAX_ACTIVE.load(Ordering::Acquire);
            }
            Ok(mut owner) if minus == 0 => {
                saw_exact = budget.used() == crate::document_consumer_budget::CONSUMER_LIMIT
                    && Arc::ptr_eq(&owner.owner_shared, &client.shared);
                assert!(owner.close(), "exact metadata child closes physically");
            }
            Err((reason, mut owner)) => {
                if let Some(owner) = owner.as_mut() {
                    owner.close();
                }
                panic!("unexpected metadata result {reason:?}");
            }
            Ok(mut owner) => {
                owner.close();
                panic!("minus one spawned");
            }
        }
        drop(held);
        drop(doc);
        client.shutdown().join().unwrap();
    }
    assert!(saw_exact && saw_minus);
    eprintln!(
        "SA16 compiledConnection={} Shared={} exactConsumer/-1 beforeChild PASS",
        size_of::<Connection>(),
        size_of::<Shared>()
    );
}
