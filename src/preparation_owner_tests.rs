use super::*;
use crate::input::SourceOrigin;
use crate::source_preparation::*;
use std::sync::Barrier;
static SERIAL: Mutex<()> = Mutex::new(());
fn b() -> Binding {
    Binding {
        input: 1,
        generation: 2,
        request: 3,
        revision: 4,
        digest: None,
    }
}
fn wait<I: Send + Sync + 'static, O: Send + 'static>(o: &mut OwnedPreparation<I, O>) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while !o.is_finished() && Instant::now() < deadline {
        thread::yield_now();
    }
    assert!(o.is_finished(), "own task deadline");
}

fn wait_authority<DI: Send + Sync + 'static, DO: Send + 'static>(o: &PreparationAuthority<DI, DO>) {
    let end = Instant::now() + Duration::from_secs(3);
    while !o.is_finished() && Instant::now() < end {
        thread::yield_now();
    }
    assert!(o.is_finished(), "own authority deadline");
}
fn hash_owner(input: Arc<SourcePreparationInput>, rev: u64) -> PreparationAuthority<(), ()> {
    let mut a = PreparationAuthority::default();
    start_hash(input, rev, &mut a).unwrap();
    a
}
fn model_owner(context: Arc<PreparedSourceContext>, rev: u64) -> PreparationAuthority<(), ()> {
    let mut a = PreparationAuthority::default();
    start_source(context, rev, &mut a).unwrap();
    a
}
fn resolved(input: Arc<SourcePreparationInput>) -> Arc<PreparedSourceContext> {
    let binding = input.binding(0);
    let mut a = hash_owner(input, 0);
    wait_authority(&a);
    let Some(PreparedCompletion::Source(Outcome::Ready(SourcePrepared::Hash(c)), _)) =
        a.take_validated(binding, false).unwrap()
    else {
        panic!("resolved real hash")
    };
    c
}
#[test]
fn b001_real_hash_source() {
    let _lock = SERIAL.lock().unwrap();
    assert!(!global_busy());
    let original = Arc::new("A\u{1b}e\u{301}\r\nZ".to_owned());
    let input =
        SourcePreparationInput::new(original.clone(), SourceOrigin::Literal, 7, 11).unwrap();
    let binding = input.binding(3);
    let mut hash = hash_owner(input.clone(), 3);
    wait_authority(&hash);
    let Some(PreparedCompletion::Source(Outcome::Ready(SourcePrepared::Hash(h)), receipt)) =
        hash.take_validated(binding, false).unwrap()
    else {
        panic!("real hash")
    };
    assert!(receipt.joined);
    assert_eq!(
        h.source().hash(),
        &[
            99, 185, 210, 184, 22, 41, 234, 15, 108, 49, 134, 33, 10, 226, 109, 108, 148, 15, 129,
            125, 234, 124, 125, 210, 95, 113, 152, 243, 233, 24, 48, 173
        ]
    );
    assert!(Arc::ptr_eq(&h.source().original_handle(), &original));
    assert_eq!(h.source().origin(), &SourceOrigin::Literal);
    assert_eq!(h.source().generation(), 7);
    assert_eq!(h.source().request(), 11);
    let source_binding = h.binding(3);
    assert_eq!(source_binding.input, binding.input);
    let mut model = model_owner(h.clone(), 3);
    wait_authority(&model);
    let Some(PreparedCompletion::Source(Outcome::Ready(SourcePrepared::Model(m)), receipt)) =
        model.take_validated(source_binding, false).unwrap()
    else {
        panic!("real model")
    };
    assert!(receipt.joined);
    assert!(receipt.authority_box_bytes > 0);
    println!(
        "AUTH box={} value={} parts={} completion={} closure={} inputArc={} contextArc={} sourceArc={} originInput={} originSource={}",
        receipt.authority_box_bytes,
        receipt.authority_value_bytes,
        receipt.parts_bytes,
        receipt.completion_bytes,
        receipt.closure_bytes,
        h.input_layout_bytes(),
        h.context_layout_bytes(),
        h.source_layout_bytes(),
        h.input().origin_capacity(),
        h.origin_capacity()
    );
    assert!(Arc::ptr_eq(m.context(), &h));
    assert!(Arc::ptr_eq(&m.model().original_handle(), &original));
    assert_eq!(m.model().original(), original.as_str());
    assert_eq!(m.model().visible(), "A␛e\u{301}\r\nZ");
    let r = m.allocation();
    assert!(r.visible_capacity >= m.model().visible().len());
    assert!(
        r.known_heap_bytes
            >= r.visible_capacity
                + r.original_capacity
                + r.checkpoint_bytes
                + r.line_bytes
                + r.index_bytes
    );
    println!(
        "B001 original={} visible={} checkpoints={} lines={} index={} growth={} known={}",
        r.original_capacity,
        r.visible_capacity,
        r.checkpoint_bytes,
        r.line_bytes,
        r.index_bytes,
        r.index_growth_peak,
        r.known_heap_bytes
    );
    assert!(!global_busy());
}
#[test]
fn b002_spawn_panic() {
    let _lock = SERIAL.lock().unwrap();
    let input = Arc::new(0u8);
    let err = OwnedPreparation::spawn_with(
        input.clone(),
        b(),
        |_, _| Outcome::Ready(1u8),
        |_| Err(std::io::Error::other("injected spawn")),
        None,
    )
    .err();
    assert_eq!(err, Some(Cause::Spawn));
    assert!(!global_busy());
    assert_eq!(Arc::strong_count(&input), 1);
    let mut o =
        OwnedPreparation::<u8, u8>::spawn(input, b(), |_, _| panic!("injected own worker panic"))
            .unwrap();
    wait(&mut o);
    assert_eq!(o.take_validated(b(), false).err(), Some(Cause::Panic));
    assert!(!global_busy());
}
#[test]
fn b003_ready_before_join() {
    let _lock = SERIAL.lock().unwrap();
    let ready = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let a = ready.clone();
    let z = release.clone();
    let mut o = OwnedPreparation::spawn_with(
        Arc::new(0u8),
        b(),
        |_, _| Outcome::Ready(2u8),
        move |f| {
            thread::spawn(move || {
                let receipt = f();
                a.wait();
                z.wait();
                receipt
            })
            .pipe(Ok)
        },
        None,
    )
    .unwrap();
    ready.wait();
    assert!(!o.poll().unwrap());
    assert!(o.take_validated(b(), false).unwrap().is_none());
    release.wait();
    wait(&mut o);
    assert!(matches!(
        o.take_validated(b(), false).unwrap(),
        Some((Outcome::Ready(2), _))
    ));
    assert!(!global_busy());
}
trait Pipe: Sized {
    fn pipe<T>(self, f: impl FnOnce(Self) -> T) -> T {
        f(self)
    }
}
impl<T> Pipe for T {}
#[test]
fn b004_full_and_disconnect() {
    let _lock = SERIAL.lock().unwrap();
    let mut full = OwnedPreparation::spawn_with(
        Arc::new(0u8),
        b(),
        |_, _| Outcome::Ready(3u8),
        |f| thread::Builder::new().spawn(f),
        Some(Outcome::Cancelled),
    )
    .unwrap();
    wait(&mut full);
    assert_eq!(full.take_validated(b(), false).err(), Some(Cause::SendFull));
    assert!(!global_busy());
    let input = Arc::new(0u8);
    let stop = Arc::new(AtomicBool::new(false));
    let (s, r) = mpsc::sync_channel(1);
    drop(r);
    let final_receipt = execute(input, stop, b(), s, |_, _| Outcome::Ready(4u8));
    assert_eq!(final_receipt.cause, Some(Cause::Disconnected));
    assert!(matches!(
        final_receipt.retained.unwrap().outcome,
        Outcome::Ready(4)
    ));
}
#[test]
fn b005_global_pending_invocation() {
    let _lock = SERIAL.lock().unwrap();
    let ready = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let a = ready.clone();
    let z = release.clone();
    let original = Arc::new("NEXT".to_owned());
    let input =
        SourcePreparationInput::new(original.clone(), SourceOrigin::StandardInput, 2, 3).unwrap();
    let context = resolved(input.clone());
    let mut other = PreparationAuthority::<(), ()>::default();
    let held = OwnedPreparation::spawn(input.clone(), input.binding(0), move |_, _| {
        a.wait();
        z.wait();
        Outcome::Ready(9u8)
    })
    .unwrap();
    ready.wait();
    assert_eq!(held.cleanup(Instant::now()), Err(Cause::Pending));
    assert!(global_pending() && global_busy());
    assert_eq!(
        start_source(context.clone(), 1, &mut other).err(),
        Some(Cause::Pending)
    );
    assert_eq!(
        start_hash(input.clone(), 1, &mut other).err(),
        Some(Cause::Pending)
    );
    assert!(retry_pending().unwrap().is_none());
    assert!(Arc::strong_count(&input) > 1);
    release.wait();
    let deadline = Instant::now() + Duration::from_secs(3);
    let receipt = loop {
        if let Some(r) = retry_pending().unwrap() {
            break r;
        }
        assert!(Instant::now() < deadline);
        thread::yield_now();
    };
    assert!(receipt.joined);
    assert_eq!(receipt.cause, Some(Cause::Cancelled));
    assert!(!global_pending() && !global_busy());
    assert_eq!(Arc::strong_count(&input), 2);
    let source_binding = context.binding(1);
    let mut next = model_owner(context, 1);
    wait_authority(&next);
    assert!(matches!(
        next.take_validated(source_binding, false).unwrap(),
        Some(PreparedCompletion::Source(
            Outcome::Ready(SourcePrepared::Model(_)),
            _
        ))
    ));
}
#[test]
fn b006_stale_stop_late() {
    let _lock = SERIAL.lock().unwrap();
    let mut stale =
        OwnedPreparation::spawn(Arc::new(0u8), b(), |_, _| Outcome::Ready(1u8)).unwrap();
    wait(&mut stale);
    let mut changed = b();
    changed.revision += 1;
    assert_eq!(
        stale.take_validated(changed, false).err(),
        Some(Cause::Stale)
    );
    let mut stopped =
        OwnedPreparation::spawn(Arc::new(0u8), b(), |_, _| Outcome::Ready(1u8)).unwrap();
    stopped.stop();
    wait(&mut stopped);
    assert_eq!(
        stopped.take_validated(b(), true).err(),
        Some(Cause::Cancelled)
    );
    assert!(!global_busy());
}
#[test]
fn b007_real_source_cancel() {
    let _lock = SERIAL.lock().unwrap();
    let original = Arc::new("INPUT".to_owned());
    let input = SourcePreparationInput::new(original.clone(), SourceOrigin::Nushell, 1, 2).unwrap();
    let stop = AtomicBool::new(true);
    assert!(matches!(
        crate::source_preparation::prepare_source(resolved(input.clone()), &stop),
        Outcome::Cancelled
    ));
    assert_eq!(original.as_str(), "INPUT");
}
#[test]
fn b008_drop_running_retained() {
    let _lock = SERIAL.lock().unwrap();
    let ready = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let a = ready.clone();
    let z = release.clone();
    let task = OwnedPreparation::spawn(Arc::new(7u8), b(), move |_, _| {
        a.wait();
        z.wait();
        Outcome::Ready(1u8)
    })
    .unwrap();
    ready.wait();
    drop(task);
    assert!(global_pending());
    assert_eq!(
        OwnedPreparation::spawn(Arc::new(8u8), b(), |_, _| Outcome::Ready(1u8)).err(),
        Some(Cause::Pending)
    );
    release.wait();
    let deadline = Instant::now() + Duration::from_secs(3);
    while retry_pending().unwrap().is_none() {
        assert!(Instant::now() < deadline);
        thread::yield_now();
    }
    assert!(!global_busy());
}

#[test]
fn b009_real_source_line_limit() {
    let _lock = SERIAL.lock().unwrap();
    let original = Arc::new("\n".repeat(1_000_000));
    let input = SourcePreparationInput::new(original.clone(), SourceOrigin::Literal, 2, 3).unwrap();
    let context = resolved(input);
    let binding = context.binding(0);
    let mut owner = model_owner(context, 0);
    wait_authority(&owner);
    assert!(matches!(
        owner.take_validated(binding, false).unwrap(),
        Some(PreparedCompletion::Source(Outcome::Limit(_), _))
    ));
    assert_eq!(original.len(), 1_000_000);
    assert!(!global_busy());
}
#[test]
fn b010_late_final_receipt_real_model() {
    let _lock = SERIAL.lock().unwrap();
    let input =
        SourcePreparationInput::new(Arc::new("REAL".into()), SourceOrigin::Literal, 2, 3).unwrap();
    let context = resolved(input.clone());
    let binding = context.binding(0);
    let mut o = OwnedPreparation::spawn_with(
        input,
        binding,
        move |_, stop| crate::source_preparation::prepare_source(context, stop),
        |f| {
            thread::Builder::new().spawn(move || {
                let mut r = f();
                r.cause = Some(Cause::Limit("late final receipt"));
                r
            })
        },
        None,
    )
    .unwrap();
    wait(&mut o);
    assert_eq!(
        o.take_validated(binding, false).err(),
        Some(Cause::Limit("late final receipt"))
    );
    assert!(!global_busy());
}
#[test]
fn b011_actual_oldnew_same_original() {
    let _lock = SERIAL.lock().unwrap();
    let original = Arc::new("A\u{1b}B".to_owned());
    let input = SourcePreparationInput::new(original.clone(), SourceOrigin::Literal, 2, 3).unwrap();
    let context = resolved(input);
    let binding = context.binding(0);
    let mut a = model_owner(context.clone(), 0);
    wait_authority(&a);
    let Some(PreparedCompletion::Source(Outcome::Ready(SourcePrepared::Model(old)), _)) =
        a.take_validated(binding, false).unwrap()
    else {
        panic!("old realSource")
    };
    let mut b = model_owner(context, 0);
    wait_authority(&b);
    let Some(PreparedCompletion::Source(Outcome::Ready(SourcePrepared::Model(new)), _)) =
        b.take_validated(binding, false).unwrap()
    else {
        panic!("new realSource")
    };
    assert!(Arc::ptr_eq(
        &old.model().original_handle(),
        &new.model().original_handle()
    ));
    let x = old.allocation();
    let y = new.allocation();
    let combined = x
        .known_heap_bytes
        .checked_add(y.known_heap_bytes)
        .unwrap()
        .checked_add(old.output_box_bytes() + new.output_box_bytes())
        .unwrap()
        .checked_sub(x.original_capacity + x.original_arc_bytes)
        .unwrap();
    assert!(combined > x.known_heap_bytes);
    println!(
        "B011 oldHeap={} newHeap={} sharedOriginal={} combinedHeap={}",
        x.known_heap_bytes,
        y.known_heap_bytes,
        x.original_capacity + x.original_arc_bytes,
        combined
    );
}

#[test]
fn b012_cleanup_preserves_worker_panic() {
    let _lock = SERIAL.lock().unwrap();
    let mut owner = OwnedPreparation::<u8, u8>::spawn(Arc::new(0), b(), |_, _| {
        panic!("injected panic before cleanup")
    })
    .unwrap();
    wait(&mut owner);
    let receipt = owner.cleanup(Instant::now()).unwrap();
    assert!(receipt.joined);
    assert_eq!(receipt.cause, Some(Cause::Cancelled));
    assert_eq!(receipt.worker_cause, Some(Cause::Panic));
    assert!(!global_busy());
}

#[test]
fn hs02_digest_and_revision_rejected() {
    let _lock = SERIAL.lock().unwrap();
    let input = SourcePreparationInput::new(
        Arc::new("BIND".to_owned()),
        SourceOrigin::StandardInput,
        31,
        32,
    )
    .unwrap();
    let c = resolved(input.clone());
    for revision in [false, true] {
        let mut a = model_owner(c.clone(), 4);
        wait_authority(&a);
        let mut b = c.binding(4);
        if revision {
            b.revision += 1
        } else {
            b.digest.as_mut().unwrap()[0] ^= 1
        };
        assert_eq!(a.take_validated(b, false).err(), Some(Cause::Stale));
        assert!(!global_busy());
    }
    assert_eq!(c.input().binding(4).input, input.binding(4).input);
}
#[test]
fn hs03_hash_cancel_cannot_resolve() {
    let _lock = SERIAL.lock().unwrap();
    let input =
        SourcePreparationInput::new(Arc::new("CANCEL".to_owned()), SourceOrigin::Literal, 1, 2)
            .unwrap();
    let binding = input.binding(0);
    let mut a = hash_owner(input, 0);
    a.stop();
    wait_authority(&a);
    assert_eq!(
        a.take_validated(binding, false).err(),
        Some(Cause::Cancelled)
    );
    assert!(!global_busy());
}
#[test]
fn hs04_real_document_context() {
    let _lock = SERIAL.lock().unwrap();
    use crate::document_budget::ActorArena;
    use crate::document_model::{Document, Profile};
    use crate::document_obsidian::Control;
    let cancel = AtomicBool::new(false);
    let ctl = Control {
        cancel: &cancel,
        deadline: Instant::now() + Duration::from_secs(3),
    };
    let original = Arc::new("# ACTUAL\nbody\n".to_owned());
    let source = Arc::new(
        crate::document_source::DocumentSource::new(
            original.clone(),
            SourceOrigin::StandardInput,
            51,
            52,
            &cancel,
        )
        .unwrap(),
    );
    let candidate =
        crate::document_parser::project(source.clone(), Profile::Extended, ActorArena::new(), &ctl)
            .unwrap();
    let depth = crate::document_validate::validate(&candidate, &ctl).unwrap();
    let proof = crate::document_validate::quarantine(candidate, depth, &ctl).unwrap();
    let doc = Arc::new(Document::from_validated(proof));
    let c = PreparedSourceContext::from_document(doc.clone()).unwrap();
    assert!(Arc::ptr_eq(c.source(), &source));
    assert!(Arc::ptr_eq(c.input().original(), &original));
    assert_eq!(Arc::strong_count(&doc), 2);
    let binding = c.binding(6);
    let mut a = model_owner(c.clone(), 6);
    wait_authority(&a);
    let Some(PreparedCompletion::Source(Outcome::Ready(SourcePrepared::Model(m)), _)) =
        a.take_validated(binding, false).unwrap()
    else {
        panic!("document Source")
    };
    drop(doc);
    drop(c);
    assert_eq!(m.model().original(), original.as_str());
    assert!(Arc::ptr_eq(m.context().source(), &source));
}
#[test]
fn hs05_source_failure_keeps_context() {
    let _lock = SERIAL.lock().unwrap();
    let original = Arc::new("\n".repeat(1_000_000));
    let input = SourcePreparationInput::new(original, SourceOrigin::Literal, 61, 62).unwrap();
    let c = resolved(input);
    let weak = Arc::downgrade(&c);
    let binding = c.binding(0);
    let mut a = model_owner(c.clone(), 0);
    drop(c);
    assert!(weak.upgrade().is_some());
    wait_authority(&a);
    assert!(matches!(
        a.take_validated(binding, false).unwrap(),
        Some(PreparedCompletion::Source(Outcome::Limit(_), _))
    ));
    assert!(weak.upgrade().is_none());
    assert!(!global_busy());
}
#[test]
fn auth01_typed_alternation() {
    let _lock = SERIAL.lock().unwrap();
    let mut a = PreparationAuthority::<u8, u8>::default();
    let input =
        SourcePreparationInput::new(Arc::new("ALT".to_owned()), SourceOrigin::Literal, 71, 72)
            .unwrap();
    let binding = input.binding(0);
    start_hash(input, 0, &mut a).unwrap();
    wait_authority(&a);
    let Some(PreparedCompletion::Source(Outcome::Ready(SourcePrepared::Hash(c)), _)) =
        a.take_validated(binding, false).unwrap()
    else {
        panic!("hash")
    };
    a.start_document_owner(|| {
        OwnedPreparation::spawn(Arc::new(3u8), b(), |_, _| Outcome::Ready(4u8))
    })
    .unwrap();
    wait_authority(&a);
    assert!(
        matches!(a.take_validated(b(),false).unwrap(),Some(PreparedCompletion::Document(Outcome::Ready(4),r))if r.joined)
    );
    let binding = c.binding(0);
    start_source(c, 0, &mut a).unwrap();
    wait_authority(&a);
    assert!(matches!(
        a.take_validated(binding, false).unwrap(),
        Some(PreparedCompletion::Source(
            Outcome::Ready(SourcePrepared::Model(_)),
            _
        ))
    ));
    assert!(!global_busy()); // u8 is only a kernel control, not a fabricated production Document.
}
#[test]
fn auth02_typed_pending_retains_enum() {
    let _lock = SERIAL.lock().unwrap();
    let ready = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let x = ready.clone();
    let z = release.clone();
    let input = Arc::new(91u8);
    let weak = Arc::downgrade(&input);
    let mut a = PreparationAuthority::<u8, u8>::default();
    a.start_document_owner(|| {
        OwnedPreparation::spawn(input, b(), move |_, _| {
            x.wait();
            z.wait();
            Outcome::Ready(8u8)
        })
    })
    .unwrap();
    ready.wait();
    assert_eq!(a.cleanup(Instant::now()), Err(Cause::Pending));
    assert!(a.retry_cleanup().unwrap().is_none());
    assert_eq!(
        a.start_document_owner(|| OwnedPreparation::spawn(Arc::new(0u8), b(), |_, _| {
            Outcome::Ready(0u8)
        }))
        .err(),
        Some(Cause::Pending)
    );
    drop(a);
    assert!(global_pending());
    assert!(weak.upgrade().is_some());
    let mut next = PreparationAuthority::<(), ()>::default();
    let input =
        SourcePreparationInput::new(Arc::new("NEXT".to_owned()), SourceOrigin::Literal, 1, 2)
            .unwrap();
    assert_eq!(
        start_hash(input.clone(), 0, &mut next).err(),
        Some(Cause::Pending)
    );
    release.wait();
    let end = Instant::now() + Duration::from_secs(3);
    while retry_pending().unwrap().is_none() {
        assert!(Instant::now() < end);
        thread::yield_now();
    }
    assert!(weak.upgrade().is_none());
    assert!(!global_busy());
    let binding = input.binding(0);
    start_hash(input, 0, &mut next).unwrap();
    wait_authority(&next);
    assert!(next.take_validated(binding, false).unwrap().is_some());
}

#[test]
fn auth03_typed_source_pending() {
    let _lock = SERIAL.lock().unwrap();
    let input = SourcePreparationInput::new(
        Arc::new("REAL-PENDING".to_owned()),
        SourceOrigin::Literal,
        1,
        2,
    )
    .unwrap();
    let context = resolved(input);
    let weak = Arc::downgrade(&context);
    let binding = context.binding(0);
    let ready = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let x = ready.clone();
    let z = release.clone();
    let mut a = PreparationAuthority::<(), ()>::default();
    let i = context.input().clone();
    a.start_source_owner(|| {
        OwnedPreparation::spawn_with(
            i,
            binding,
            move |_, stop| crate::source_preparation::prepare_source(context, stop),
            move |f| {
                thread::Builder::new().spawn(move || {
                    let r = f();
                    x.wait();
                    z.wait();
                    r
                })
            },
            None,
        )
    })
    .unwrap();
    ready.wait();
    assert!(!a.poll().unwrap());
    assert!(a.take_validated(binding, false).unwrap().is_none());
    assert_eq!(a.cleanup(Instant::now()), Err(Cause::Pending));
    drop(a);
    assert!(weak.upgrade().is_some());
    assert!(global_pending());
    let mut next = PreparationAuthority::<(), ()>::default();
    let input =
        SourcePreparationInput::new(Arc::new("AFTER".to_owned()), SourceOrigin::Literal, 3, 4)
            .unwrap();
    assert_eq!(
        start_hash(input.clone(), 0, &mut next).err(),
        Some(Cause::Pending)
    );
    release.wait();
    let deadline = Instant::now() + Duration::from_secs(3);
    let r = loop {
        if let Some(r) = retry_pending().unwrap() {
            break r;
        }
        assert!(Instant::now() < deadline);
        thread::yield_now();
    };
    assert_eq!(r.cause, Some(Cause::Cancelled));
    assert!(r.joined);
    assert!(weak.upgrade().is_none());
    assert!(!global_busy());
    let binding = input.binding(0);
    start_hash(input, 0, &mut next).unwrap();
    wait_authority(&next);
    assert!(next.take_validated(binding, false).unwrap().is_some());
}
