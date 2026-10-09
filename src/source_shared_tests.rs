use super::*;
use crate::{
    document_budget::TRANSPORT_RESERVE,
    document_layout::Geometry,
    preparation_owner::{global_busy, global_pending, retry_pending},
    source_preparation::{SourcePreparationInput, start_hash, start_scoped_source},
    source_view_preparation::{SourceViewportInput, SourceViewportPrepared, start_source_viewport},
};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};
fn finish<DI: Send + Sync + 'static, DO: Send + 'static>(a: &PreparationAuthority<DI, DO>) {
    let until = Instant::now() + Duration::from_secs(5);
    while !a.is_finished() && Instant::now() < until {
        thread::yield_now();
    }
    assert!(a.is_finished());
}
fn hash(text: &str, arena: &ActorArena) -> (SourceScope, PreparationAuthority<(), ()>) {
    let input = SourcePreparationInput::new(
        Arc::new(text.to_owned()),
        crate::input::SourceOrigin::Literal,
        7,
        11,
    )
    .unwrap();
    let binding = input.binding(1);
    let mut a = PreparationAuthority::in_arena(arena).unwrap();
    start_hash(input, 1, &mut a).unwrap();
    finish(&a);
    let Some(PreparedCompletion::Source(Outcome::Ready(SourcePrepared::Hash(c)), r)) =
        a.take_validated(binding, false).unwrap()
    else {
        panic!("actualHash")
    };
    assert!(r.joined);
    let scope = SourceScope::from_hash(c, arena).unwrap();
    (scope, a)
}
fn model<DI: Send + Sync + 'static, DO: Send + 'static>(
    scope: SourceScope,
    a: &mut PreparationAuthority<DI, DO>,
) -> SharedSource {
    let expected = scope.binding(2);
    start_scoped_source(scope.try_alias().unwrap(), 2, a).unwrap();
    finish(a);
    match adopt_joined_source(a, scope, expected, false, &AtomicBool::new(false)).unwrap() {
        Adoption::Ready(shared) => shared,
        _ => panic!("actualshared"),
    }
}
fn geom(width: u16, height: u16, left: usize) -> Geometry {
    Geometry {
        x: 3,
        y: 2,
        width,
        height,
        left,
    }
}
fn page<DI: Send + Sync + 'static, DO: Send + 'static>(
    shared: &SharedSource,
    a: &mut PreparationAuthority<DI, DO>,
    g: Geometry,
    selection: Option<std::ops::Range<usize>>,
) -> Box<SourceViewportPrepared> {
    let input = SourceViewportInput::new(shared, g, 0, selection, 3, 4, 5, 6).unwrap();
    let binding = input.key().binding();
    start_source_viewport(input, a).unwrap();
    finish(a);
    let Some(PreparedCompletion::Source(Outcome::Ready(SourcePrepared::Viewport(page)), receipt)) =
        a.take_validated(binding, false).unwrap()
    else {
        panic!("realSourceViewport")
    };
    assert!(receipt.joined);
    assert_eq!(page.key().binding(), binding);
    page
}
#[test]
fn common01_real_send_sync_and_predocument_lifetime() {
    fn send_sync<T: Send + Sync + 'static>() {}
    type Kernel = crate::preparation_owner::OwnedPreparation<
        crate::source_preparation::SourcePreparationTaskInput,
        SourcePrepared,
    >;
    let run: fn(
        &crate::source_preparation::SourcePreparationTaskInput,
        &AtomicBool,
    ) -> Outcome<SourcePrepared> = |_, _| Outcome::Cancelled;
    eprintln!(
        "SOURCE kernel finalReceipt={} stopArc={} ownerValue={} viewportCapture={} spawn={} rollbackInline={}",
        Kernel::known_final_receipt_bytes(),
        Kernel::known_stop_arc_bytes().unwrap(),
        Kernel::known_owner_value_bytes(),
        Kernel::known_worker_capture_bytes(&run),
        Kernel::known_spawn_bytes(&run).unwrap(),
        size_of::<SharePending>()
    );
    send_sync::<crate::presentation::SafeTextModel>();
    send_sync::<SourceScope>();
    send_sync::<SharedSource>();
    send_sync::<SourceLease>();
    send_sync::<crate::source_preparation::SourcePreparationTaskInput>();
    send_sync::<SourceViewportInput>();
    send_sync::<SourceViewportPrepared>();
    eprintln!(
        "SOURCE layouts Scope={} ScopeAllocationArc={} SharedAllocationArc={} SharedHandle={} LeaseHandle={} TaskInputArc={} Parts={} Completion={} Authority={} SourceBox={} ViewportInput={} ViewportOutput={}",
        size_of::<SourceScope>(),
        size_of::<ScopeAllocation>() + 2 * size_of::<usize>(),
        size_of::<SharedSourceAllocation>() + 2 * size_of::<usize>(),
        size_of::<SharedSource>(),
        size_of::<SourceLease>(),
        size_of::<crate::source_preparation::SourcePreparationTaskInput>() + 2 * size_of::<usize>(),
        crate::source_preparation::SourcePreparationOwner::known_parts_bytes(),
        crate::source_preparation::SourcePreparationOwner::known_completion_bytes(),
        PreparationAuthority::<(), ()>::known_authority_bytes().unwrap(),
        size_of::<PreparedSourceModel>(),
        size_of::<SourceViewportInput>(),
        size_of::<SourceViewportPrepared>()
    );
    let arena = ActorArena::new();
    let (scope, mut a) = hash("BODY\n", &arena);
    let weak = Arc::downgrade(scope.context());
    let shared = model(scope, &mut a);
    let pointer = shared.model() as *const SafeTextModel;
    let lease = shared.try_lease().unwrap();
    assert!(Arc::ptr_eq(
        shared.context().input().original(),
        &shared.model().original_handle()
    ));
    assert_eq!(
        shared.source_heap_bytes(),
        shared.allocation().known_heap_bytes + shared.model_box().output_box_bytes()
    );
    drop(shared);
    assert!(weak.upgrade().is_some());
    assert_eq!(pointer, lease.model() as *const SafeTextModel);
    drop(lease);
    drop(a);
    assert!(weak.upgrade().is_none());
    assert_eq!(arena.used(), TRANSPORT_RESERVE);
    assert!(!global_busy());
}
#[test]
fn common02_controls_ranges_hscroll_and_geometry_origin() {
    let text = include_str!("../tests/fixtures/source-common/source-controls.txt");
    let arena = ActorArena::new();
    let (scope, mut a) = hash(text, &arena);
    let shared = model(scope, &mut a);
    let pointer = shared.model() as *const SafeTextModel;
    let p = page(&shared, &mut a, geom(20, 2, 0), Some(1..2));
    assert_eq!(p.symbols(), "A␛      界e\u{301}TAIL");
    assert_eq!(p.selection().unwrap(), &(1..2, 1..4, 1));
    let wide = p.glyphs().iter().find(|g| g.width == 2).unwrap();
    assert_eq!(
        (
            wide.column,
            wide.source_range().start,
            wide.source_range().end
        ),
        (8, 3, 6)
    );
    let p = page(&shared, &mut a, geom(2, 1, 9), None);
    assert_eq!(p.symbols(), " e\u{301}");
    assert_eq!(p.glyphs()[0].width, 1);
    assert_eq!(p.glyphs()[1].visible_range().len(), 3);
    assert_eq!(pointer, p.model() as *const SafeTextModel);
}
#[test]
fn common03_dense_80_120_and_same_model() {
    assert_eq!(size_of::<crate::source_view_preparation::SourceGlyph>(), 24);
    let arena = ActorArena::new();
    let (scope, mut a) = hash(
        include_str!("../tests/fixtures/source-common/source-dense.txt"),
        &arena,
    );
    let shared = model(scope, &mut a);
    for (w, h) in [(80, 24), (120, 40)] {
        let p = page(&shared, &mut a, geom(w, h, 0), None);
        assert_eq!(p.glyphs().len(), usize::from(w) * usize::from(h));
        assert_eq!(p.symbols().len(), p.glyphs().len());
        assert!(p.work() <= crate::source_view_preparation::SOURCE_COMMON_WORK);
        assert!(p.frame_bytes() <= 256 * 1024);
        assert!(Arc::ptr_eq(p.context(), shared.context()));
        eprintln!(
            "SOURCE viewport{w}x{h} work={} frame={} SourceHeap={} arenaUsed={} peak={}",
            p.work(),
            p.frame_bytes(),
            shared.source_heap_bytes(),
            arena.used(),
            arena.peak()
        );
    }
}
#[test]
fn common04_adoption_quota_rollback_samebox_and_retry() {
    let arena = ActorArena::new();
    let (scope, mut a) = hash("RETRY\n", &arena);
    let expected = scope.binding(2);
    start_scoped_source(scope.try_alias().unwrap(), 2, &mut a).unwrap();
    finish(&a);
    let mut block = None;
    let Adoption::Retained(pending) = adopt_joined_source_with(
        &mut a,
        scope,
        expected,
        false,
        &AtomicBool::new(false),
        || {
            block = Some(
                arena
                    .reserve(crate::document_budget::ARENA_LIMIT - arena.used())
                    .unwrap(),
            );
        },
    )
    .unwrap() else {
        panic!("retainedSameBox")
    };
    let pointer = pending.model() as *const PreparedSourceModel;
    let modelpointer = pending.model().model() as *const SafeTextModel;
    assert!(matches!(pending.cause(), Cause::Limit(_)));
    let pending = match pending.retry(&AtomicBool::new(false)) {
        Err(p) => p,
        Ok(_) => panic!("quota unexpectedly available"),
    };
    assert_eq!(pointer, pending.model() as *const PreparedSourceModel);
    drop(block);
    let shared = match pending.retry(&AtomicBool::new(false)) {
        Ok(s) => s,
        Err(_) => panic!("explicitretry"),
    };
    assert_eq!(pointer, shared.model_box() as *const PreparedSourceModel);
    assert_eq!(modelpointer, shared.model() as *const SafeTextModel);
}
#[test]
fn common05_range_stale_cancel_exit_and_current_preserved() {
    let arena = ActorArena::new();
    let (scope, mut a) = hash("CURRENT\n", &arena);
    let shared = model(scope, &mut a);
    let ptr = shared.model() as *const SafeTextModel;
    assert!(
        SourceViewportInput::new(&shared, geom(80, 24, 0), 0, Some(2..99), 3, 0, 0, 0).is_err()
    );
    let input = SourceViewportInput::new(&shared, geom(80, 24, 0), 0, None, 3, 0, 0, 0).unwrap();
    let mut binding = input.key().binding();
    binding.revision += 1;
    start_source_viewport(input, &mut a).unwrap();
    finish(&a);
    assert_eq!(a.take_validated(binding, false).err(), Some(Cause::Stale));
    let input = SourceViewportInput::new(&shared, geom(80, 24, 0), 0, None, 4, 0, 0, 0).unwrap();
    let binding = input.key().binding();
    start_source_viewport(input, &mut a).unwrap();
    finish(&a);
    assert_eq!(
        a.take_validated(binding, true).err(),
        Some(Cause::Cancelled)
    );
    assert_eq!(ptr, shared.model() as *const SafeTextModel);
    assert_eq!(shared.model().original(), "CURRENT\n");
}
#[test]
fn common06_document_scope_real_binding_and_lastdrop() {
    let arena = ActorArena::new();
    let stop = AtomicBool::new(false);
    let ctl = crate::document_obsidian::Control {
        cancel: &stop,
        deadline: Instant::now() + Duration::from_secs(10),
    };
    let source = Arc::new(
        crate::document_source::DocumentSource::new(
            Arc::new(include_str!("../tests/fixtures/source-common/document-scope.md").to_owned()),
            crate::input::SourceOrigin::Literal,
            7,
            11,
            &stop,
        )
        .unwrap(),
    );
    let candidate = crate::document_parser::project(
        source,
        crate::document_model::Profile::Extended,
        arena.clone(),
        &ctl,
    )
    .unwrap();
    let depth = crate::document_validate::validate(&candidate, &ctl).unwrap();
    let validated = crate::document_validate::quarantine(candidate, depth, &ctl).unwrap();
    let document = Arc::new(Document::from_validated(validated));
    let weakdoc = Arc::downgrade(&document);
    let mut docauthority =
        crate::document_view::DocumentPreparationAuthority::in_arena(&arena).unwrap();
    let input =
        DocumentView::initial_input_in(document.clone(), geom(80, 24, 0), &docauthority).unwrap();
    let binding = input.key().binding;
    crate::document_view::start_document(input, &mut docauthority).unwrap();
    finish(&docauthority);
    let Some(PreparedCompletion::Document(Outcome::Ready(state), _)) =
        docauthority.take_validated(binding, false).unwrap()
    else {
        panic!("actualdocument")
    };
    let view = DocumentView::from_prepared(state).unwrap();
    let weakmetadata = Arc::downgrade(view.inspect_source_metadata());
    let context = view.inspect_source_context().clone();
    let sourcebinding = view.render_binding();
    assert!(SourceScope::from_view(&view, &ActorArena::new()).is_err());
    let scope = SourceScope::from_view(&view, &arena).unwrap();
    assert_eq!(scope.retained_document_binding(), Some(sourcebinding));
    assert!(Arc::ptr_eq(scope.context(), &context));
    let shared = model(scope, &mut docauthority);
    let p = page(&shared, &mut docauthority, geom(80, 24, 0), None);
    drop(context);
    drop(view);
    drop(document);
    drop(shared);
    assert!(weakmetadata.upgrade().is_some());
    assert!(weakdoc.upgrade().is_some());
    assert_eq!(p.symbols(), "# ORIGINALBODY **BOLD**");
    drop(p);
    drop(docauthority);
    assert!(weakmetadata.upgrade().is_none());
    assert!(weakdoc.upgrade().is_none());
    assert_eq!(arena.used(), TRANSPORT_RESERVE);
}
#[test]
fn common07_real_viewport_pending_retains_scope_and_blocks_hash() {
    // Barrier surrounds the real viewport factory after output preparation; no fake result/model.
    let arena = ActorArena::new();
    let (scope, mut a) = hash("PENDING\n", &arena);
    let shared = model(scope, &mut a);
    let weak = Arc::downgrade(shared.context());
    let input = SourceViewportInput::new(&shared, geom(80, 24, 0), 0, None, 3, 0, 0, 0).unwrap();
    let binding = input.key().binding();
    let entered = Arc::new(std::sync::Barrier::new(2));
    let release = Arc::new(AtomicBool::new(false));
    let x = entered.clone();
    let z = release.clone();
    crate::source_view_preparation::start_source_viewport_controlled(input, &mut a, move || {
        x.wait();
        while !z.load(Ordering::Acquire) {
            thread::yield_now();
        }
    })
    .unwrap();
    entered.wait();
    assert!(!a.poll().unwrap());
    assert!(a.take_validated(binding, false).unwrap().is_none());
    assert_eq!(a.cleanup(Instant::now()), Err(Cause::Pending));
    drop(a);
    drop(shared);
    assert!(global_pending());
    assert!(weak.upgrade().is_some());
    let next = SourcePreparationInput::new(
        Arc::new("NEXT".into()),
        crate::input::SourceOrigin::Literal,
        1,
        1,
    )
    .unwrap();
    let mut a = PreparationAuthority::<(), ()>::default();
    assert_eq!(start_hash(next, 0, &mut a).err(), Some(Cause::Pending));
    release.store(true, Ordering::Release);
    let until = Instant::now() + Duration::from_secs(5);
    while global_pending() && Instant::now() < until {
        retry_pending().unwrap();
        thread::yield_now();
    }
    assert!(!global_busy());
    assert!(!global_pending());
    assert!(weak.upgrade().is_none());
    assert_eq!(arena.used(), TRANSPORT_RESERVE);
}

#[test]
fn common08_c1_partial_provenance_packed_roundtrip() {
    let arena = ActorArena::new();
    let (scope, mut a) = hash("X\u{009b}Y\r\n😀e\u{301}", &arena);
    let shared = model(scope, &mut a);
    let p = page(&shared, &mut a, geom(30, 2, 0), None);
    assert_eq!(p.symbols(), "X\\u{009b}Y😀e\u{301}");
    for g in &p.glyphs()[1..9] {
        assert_eq!(g.source_range(), 1..3);
        assert_eq!(g.visible_range().len(), 1);
        assert_eq!(g.symbol_range().len(), 1);
    }
    assert_eq!(size_of::<crate::source_view_preparation::SourceGlyph>(), 24);
}
#[test]
fn common09_zero_extreme_symbol_and_foreign_scope() {
    let arena = ActorArena::new();
    let (scope, mut a) = hash("CURRENT\n", &arena);
    let shared = model(scope, &mut a);
    let p = page(&shared, &mut a, geom(0, 40, 0), None);
    assert!(p.glyphs().is_empty());
    assert!(p.symbols().is_empty());
    let input = SourceViewportInput::new(&shared, geom(u16::MAX, u16::MAX, 0), 0, None, 3, 0, 0, 0)
        .unwrap();
    let b = input.key().binding();
    start_source_viewport(input, &mut a).unwrap();
    finish(&a);
    assert!(matches!(
        a.take_validated(b, false).unwrap(),
        Some(PreparedCompletion::Source(Outcome::Limit("SourceFrame"), _))
    ));
    assert_eq!(shared.model().original(), "CURRENT\n");
    let (scope, mut a) = hash(&format!("a{}", "\u{301}".repeat(8192)), &arena);
    let large = model(scope, &mut a);
    let input = SourceViewportInput::new(&large, geom(80, 24, 0), 0, None, 3, 0, 0, 0).unwrap();
    let b = input.key().binding();
    start_source_viewport(input, &mut a).unwrap();
    finish(&a);
    assert!(matches!(
        a.take_validated(b, false).unwrap(),
        Some(PreparedCompletion::Source(
            Outcome::Limit("SourceSymbol16KiB"),
            _
        ))
    ));
    let (scope, mut a) = hash("A\n", &arena);
    let (scope_other, _) = hash("B\n", &arena);
    let expected = scope.binding(2);
    start_scoped_source(scope, 2, &mut a).unwrap();
    finish(&a);
    let Adoption::Retained(pending) = adopt_joined_source(
        &mut a,
        scope_other,
        expected,
        false,
        &AtomicBool::new(false),
    )
    .unwrap() else {
        panic!("foreigncontext")
    };
    assert_eq!(pending.cause(), Cause::Stale);
    assert_eq!(pending.model().model().original(), "A\n");
}
#[test]
fn common10_cancel_adopt_retains_box_for_explicit_retry() {
    let arena = ActorArena::new();
    let (scope, mut a) = hash("CANCEL\n", &arena);
    let expected = scope.binding(2);
    start_scoped_source(scope.try_alias().unwrap(), 2, &mut a).unwrap();
    finish(&a);
    let cancel = AtomicBool::new(true);
    let Adoption::Retained(pending) =
        adopt_joined_source(&mut a, scope, expected, false, &cancel).unwrap()
    else {
        panic!("cancelpending")
    };
    assert_eq!(pending.cause(), Cause::Cancelled);
    let ptr = pending.model() as *const PreparedSourceModel;
    let pending = match pending.retry(&cancel) {
        Err(p) => p,
        Ok(_) => panic!("cancelretry"),
    };
    assert_eq!(ptr, pending.model() as *const PreparedSourceModel);
    cancel.store(false, Ordering::Release);
    let shared = match pending.retry(&cancel) {
        Ok(s) => s,
        Err(_) => panic!("uncancelretry"),
    };
    assert_eq!(ptr, shared.model_box() as *const PreparedSourceModel);
}

#[test]
fn common11_document_foreign_adoption_retains_original_metadata_after_join() {
    let arena = ActorArena::new();
    let (foreign, foreign_authority) = hash("FOREIGN\n", &arena);
    drop(foreign_authority);
    let stop = AtomicBool::new(false);
    let ctl = crate::document_obsidian::Control {
        cancel: &stop,
        deadline: Instant::now() + Duration::from_secs(10),
    };
    let source = Arc::new(
        crate::document_source::DocumentSource::new(
            Arc::new(include_str!("../tests/fixtures/source-common/document-scope.md").to_owned()),
            crate::input::SourceOrigin::Literal,
            7,
            11,
            &stop,
        )
        .unwrap(),
    );
    let candidate = crate::document_parser::project(
        source,
        crate::document_model::Profile::Extended,
        arena.clone(),
        &ctl,
    )
    .unwrap();
    let depth = crate::document_validate::validate(&candidate, &ctl).unwrap();
    let validated = crate::document_validate::quarantine(candidate, depth, &ctl).unwrap();
    let document = Arc::new(Document::from_validated(validated));
    let weakdoc = Arc::downgrade(&document);
    let mut authority =
        crate::document_view::DocumentPreparationAuthority::in_arena(&arena).unwrap();
    let input =
        DocumentView::initial_input_in(document.clone(), geom(80, 24, 0), &authority).unwrap();
    let binding = input.key().binding;
    crate::document_view::start_document(input, &mut authority).unwrap();
    finish(&authority);
    let Some(PreparedCompletion::Document(Outcome::Ready(state), _)) =
        authority.take_validated(binding, false).unwrap()
    else {
        panic!("realview")
    };
    let view = DocumentView::from_prepared(state).unwrap();
    let weakmetadata = Arc::downgrade(view.inspect_source_metadata());
    let weakcontext = Arc::downgrade(view.inspect_source_context());
    let scope = SourceScope::from_view(&view, &arena).unwrap();
    let expected = scope.binding(2);
    start_scoped_source(scope, 2, &mut authority).unwrap();
    finish(&authority);
    drop(view);
    drop(document);
    assert!(weakmetadata.upgrade().is_some());
    let Adoption::Retained(pending) = adopt_joined_source(
        &mut authority,
        foreign,
        expected,
        false,
        &AtomicBool::new(false),
    )
    .unwrap() else {
        panic!("foreign retainsoriginal")
    };
    assert_eq!(pending.cause(), Cause::Stale);
    assert!(weakmetadata.upgrade().is_some());
    assert!(weakdoc.upgrade().is_some());
    assert!(Arc::ptr_eq(
        pending.model().context(),
        &weakcontext.upgrade().unwrap()
    ));
    let ptr = pending.model() as *const PreparedSourceModel;
    let shared = match pending.retry(&AtomicBool::new(false)) {
        Ok(s) => s,
        Err(_) => panic!("originalscoperetry"),
    };
    assert_eq!(ptr, shared.model_box() as *const PreparedSourceModel);
    assert!(weakmetadata.upgrade().is_some());
    drop(shared);
    drop(authority);
    assert!(weakmetadata.upgrade().is_none());
    assert!(weakcontext.upgrade().is_none());
    assert!(weakdoc.upgrade().is_none());
    assert_eq!(arena.used(), TRANSPORT_RESERVE);
}

#[test]
fn common12_metered_checkpoints_selection_c1_and_viewport_near_eof() {
    let arena = ActorArena::new();
    let (scope, mut authority) = hash(
        include_str!("../tests/fixtures/source-common/checkpoint-selection.txt"),
        &arena,
    );
    let shared = model(scope, &mut authority);
    let input = SourceViewportInput::new(
        &shared,
        geom(16, 2, 8398),
        0,
        Some(4200..4202),
        12,
        13,
        14,
        15,
    )
    .unwrap();
    let binding = input.key().binding();
    start_source_viewport(input, &mut authority).unwrap();
    finish(&authority);
    let Some(PreparedCompletion::Source(Outcome::Ready(SourcePrepared::Viewport(output)), receipt)) =
        authority.take_validated(binding, false).unwrap()
    else {
        panic!("realCommon");
    };
    assert!(receipt.joined);
    let selected = output.selection().unwrap();
    assert_eq!(selected.0, 4200..4202);
    assert_eq!(selected.1, 4200..4208);
    assert_eq!(output.symbols(), "bbbbbbbbbb");
    assert!(output.work() <= crate::source_view_preparation::SOURCE_COMMON_WORK);
    assert!(output.frame_bytes() <= 262144);
    assert!(std::ptr::eq(output.model(), shared.model()));
    let tail = SourceViewportInput::new(&shared, geom(16, 1, 0), 1, None, 16, 13, 17, 18).unwrap();
    let binding = tail.key().binding();
    start_source_viewport(tail, &mut authority).unwrap();
    finish(&authority);
    let Some(PreparedCompletion::Source(Outcome::Ready(SourcePrepared::Viewport(tail)), _)) =
        authority.take_validated(binding, false).unwrap()
    else {
        panic!("realTail");
    };
    assert_eq!(tail.symbols(), "TAIL");
}

#[test]
fn common13_diagnostic512k_full_cost_not_production_admission() {
    let arena = ActorArena::new();
    let (scope, mut authority) = hash(
        include_str!("../tests/fixtures/source-common/source-dense.txt"),
        &arena,
    );
    let shared = model(scope, &mut authority);
    let input =
        SourceViewportInput::new(&shared, geom(120, 40, 0), 0, None, 20, 21, 22, 23).unwrap();
    let binding = input.key().binding();
    crate::source_view_preparation::start_source_viewport_diagnostic(input, &mut authority, 524288)
        .unwrap();
    finish(&authority);
    let Some(PreparedCompletion::Source(Outcome::Ready(SourcePrepared::Viewport(output)), receipt)) =
        authority.take_validated(binding, false).unwrap()
    else {
        panic!("diagnosticrealViewport");
    };
    assert!(receipt.joined);
    assert_eq!(output.glyphs().len(), 4800);
    assert_eq!(output.symbols(), "x".repeat(4800));
    assert!(std::ptr::eq(output.model(), shared.model()));
    assert!(output.work() > 262144 && output.work() <= 524288);
    assert_eq!(output.frame_bytes(), 123712);
    eprintln!(
        "SC-W01 DIAGNOSTIC Source120x40 exactWork={} frame={} oldproduction256KFAILpreserved",
        output.work(),
        output.frame_bytes()
    );
}

#[test]
fn common14_full_operation_exact_minus_one_preserves_real_current_and_charges() {
    let arena = ActorArena::new();
    let (scope, mut authority) = hash(
        include_str!("../tests/fixtures/source-common/source-dense.txt"),
        &arena,
    );
    let shared = model(scope, &mut authority);
    let current = page(&shared, &mut authority, geom(80, 24, 0), None);
    let symbols = current.symbols().to_owned();
    let key = current.key();
    let used = arena.used();
    for (limit, accept) in [(300397, false), (300398, true)] {
        let input =
            SourceViewportInput::new(&shared, geom(120, 40, 0), 0, None, 30, 31, 32, 33).unwrap();
        let binding = input.key().binding();
        crate::source_view_preparation::start_source_viewport_diagnostic(
            input,
            &mut authority,
            limit,
        )
        .unwrap();
        finish(&authority);
        let Some(PreparedCompletion::Source(result, receipt)) =
            authority.take_validated(binding, false).unwrap()
        else {
            panic!("actualCommonfactory");
        };
        assert!(receipt.joined);
        match result {
            Outcome::Ready(SourcePrepared::Viewport(output)) if accept => {
                assert_eq!(output.work(), 300398);
                assert_eq!(output.glyphs().len(), 4800);
                assert_eq!(output.symbols(), "x".repeat(4800));
                assert!(std::ptr::eq(output.model(), shared.model()));
            }
            Outcome::Limit("SourceWork") if !accept => {}
            _ => panic!("typed boundary mismatch"),
        }
        assert_eq!(current.key(), key);
        assert_eq!(current.symbols(), symbols);
        assert_eq!(arena.used(), used);
    }
}
