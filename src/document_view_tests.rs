use super::*;
use crate::{
    document_budget::ActorArena,
    document_model::Profile,
    preparation_owner::{PreparedCompletion, global_busy, retry_pending},
};
use std::{
    sync::{Mutex, mpsc},
    thread,
    time::{Duration, Instant},
};
static SERIAL: Mutex<()> = Mutex::new(());
struct ReleaseOwnWorker(Arc<AtomicBool>);
impl Drop for ReleaseOwnWorker {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Release);
    }
}
fn geometry(height: u16) -> Geometry {
    Geometry {
        x: 0,
        y: 0,
        width: 40,
        height,
        left: 0,
    }
}
fn parsed(source: &str) -> Arc<Document> {
    let cancel = AtomicBool::new(false);
    let ctl = crate::document_obsidian::Control {
        cancel: &cancel,
        deadline: Instant::now() + Duration::from_secs(10),
    };
    let source = Arc::new(
        crate::document_source::DocumentSource::new(
            Arc::new(source.to_owned()),
            crate::input::SourceOrigin::Literal,
            7,
            9,
            &cancel,
        )
        .unwrap(),
    );
    let c = crate::document_parser::project(source, Profile::Extended, ActorArena::new(), &ctl)
        .unwrap();
    let depth = crate::document_validate::validate(&c, &ctl).unwrap();
    let p = crate::document_validate::quarantine(c, depth, &ctl).unwrap();
    Arc::new(Document::from_validated(p))
}
fn finish(a: &mut DocumentPreparationAuthority, b: Binding) -> PreparedState {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !a.is_finished() && Instant::now() < deadline {
        thread::yield_now();
    }
    assert!(a.is_finished(), "own preparation deadline");
    let Some(PreparedCompletion::Document(Outcome::Ready(p), r)) =
        a.take_validated(b, false).unwrap()
    else {
        panic!("real Document candidate")
    };
    assert!(r.joined);
    p
}
fn view(source: &str, g: Geometry) -> (DocumentView, DocumentPreparationAuthority) {
    let input = DocumentView::initial_input(parsed(source), g).unwrap();
    let b = input.key.binding;
    let mut a = DocumentPreparationAuthority::default();
    start_document(input, &mut a).unwrap();
    let p = finish(&mut a, b);
    (DocumentView::from_prepared(p).unwrap(), a)
}
fn perform(v: &mut DocumentView, a: &mut DocumentPreparationAuthority, op: Operation, g: Geometry) {
    let i = v.request(op, g).unwrap();
    let b = i.key.binding;
    start_document(i, a).unwrap();
    let p = finish(a, b);
    if let PreparedState::PositionPending(x) = &p {
        eprintln!(
            "OWN pending operation={op:?} inputOperation={:?} anchor={:?} cause={:?}",
            x.input.key.operation, x.input.anchor, x.cause
        );
    }
    v.commit(p)
        .unwrap_or_else(|e| panic!("OWN commit operation={op:?} error={e:?}"));
}
fn symbols(v: &DocumentView) -> &str {
    v.layout().text().expect("text window").symbols()
}
#[test]
fn b101_root_multirun_and_atomic_links() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (v, _) = view("A**B**\n\n[a](one.md)[\u{301}](two.md)\n", geometry(10));
    assert_eq!(symbols(&v), "ABa\u{301}");
    let glyphs = v.layout().text().unwrap().glyphs();
    assert_eq!(glyphs.len(), 3);
    assert_eq!(glyphs.get(1).unwrap().styles & 2, 2);
    assert_eq!(glyphs.get(2).unwrap().end - glyphs.get(2).unwrap().start, 3);
    assert_eq!(glyphs.get(2).unwrap().width, 1);
    assert_eq!(glyphs.get(2).unwrap().links, 2);
    let first = glyphs.get(2).unwrap().first.unwrap();
    let r = &v.document.runs()[first.run as usize];
    let tr = &v.document.text_refs()[r.text() as usize];
    assert_eq!(tr.start() as usize + first.local, 9);
    assert!(v.budget().checkpoint_capacity() <= 4096);
    assert!(v.budget().used() <= 1048576);
    eprintln!(
        "B101 consumerUsed={} consumerPeak={} checkpoints={} windowBytes={} frameWork={} preparationWork={}",
        v.budget.used(),
        v.budget.peak(),
        v.budget.checkpoint_capacity(),
        v.layout.bytes(),
        v.layout.text().unwrap().work,
        v.layout.text().unwrap().preparation_work
    );
    assert!(!global_busy());
}
#[test]
fn b102_widthzero_and_resource_barrier_reference() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    for source in ["\u{301}  \nAFTER\n", "a![x](i.png)\u{301}\n"] {
        let (v, _) = view(source, geometry(10));
        assert!(matches!(
            v.layout(),
            LayoutWindow::Reference {
                reason: ReferenceReason::WidthZero,
                ..
            }
        ));
        assert_eq!(v.document.source().text(), source);
    }
}
#[test]
fn b103_root_oversized_and_recovery_end() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let source = format!("BEFORE\n\na{}\n\nAFTER\n", "\u{301}".repeat(9000));
    let (mut v, mut a) = view(&source, geometry(10));
    assert!(matches!(
        v.layout(),
        LayoutWindow::Reference {
            reason: ReferenceReason::Symbol,
            ..
        }
    ));
    let g = v.geometry();
    perform(&mut v, &mut a, Operation::End, g);
    assert_eq!(symbols(&v), "AFTER");
    assert_eq!(v.navigator().history_len(), 0);
    assert_eq!(v.document.source().text(), source);
}
#[test]
fn b104_source_controls_and_newline_rows() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (v, _) = view("A\u{1b}  \nB\n", geometry(10));
    assert_eq!(symbols(&v), "A␛B");
    let text = v.layout().text().unwrap();
    assert_eq!(text.glyphs().get(0).unwrap().row, 0);
    assert_eq!(text.glyphs().last().unwrap().row, 1);
    assert_eq!(v.document.source().text(), "A\u{1b}  \nB\n");
}
#[test]
fn b105_flow_handle_retains_budget_and_alias_is_fallible() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (v, _) = view("BODY\n", geometry(2));
    let budget = v.budget.clone();
    let doc = Arc::downgrade(&v.document);
    let before = budget.used();
    let h = v.navigator.share_flow().unwrap();
    assert!(budget.used() > before);
    let second = h.try_alias().unwrap();
    let after = budget.used();
    drop(v);
    assert!(doc.upgrade().is_some());
    assert!(h.same(&second));
    let remaining = 1048576 - budget.used();
    let block = budget.reserve(remaining).unwrap();
    let occupied = budget.used();
    assert!(h.try_alias().is_err());
    assert_eq!(budget.used(), occupied);
    drop(block);
    drop(second);
    drop(h);
    assert!(budget.used() < after);
    drop(budget);
    assert!(doc.upgrade().is_none());
}
#[test]
fn b106_root_fold_resize_and_stale_locations() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (mut v, mut a) = view("> [!note]- T\n> BODY\n\nEND\n", geometry(10));
    assert_eq!(semantic_symbols(&v), "TEND");
    let callout = v
        .document
        .blocks()
        .iter()
        .find(|b| b.kind() == 4)
        .unwrap()
        .id();
    let g = v.geometry();
    let old = v.navigator.current().unwrap().location().clone();
    perform(
        &mut v,
        &mut a,
        Operation::Fold {
            block: callout,
            expanded: true,
        },
        g,
    );
    assert_eq!(semantic_symbols(&v), "TBODYEND");
    assert!(v.navigator.index().check_location(&old).is_err());
    perform(
        &mut v,
        &mut a,
        Operation::Resize,
        Geometry { width: 20, ..g },
    );
    assert_eq!(semantic_symbols(&v), "TBODYEND");
    assert_eq!(v.navigator.history_len(), 0);
}
#[test]
fn b107_fold_preserves_history_raw_and_back_rejects_stale_without_pop() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (mut v, mut a) = view("> [!note]+ T\n> BODY\n\nEND\n", geometry(10));
    let callout = v
        .document
        .blocks()
        .iter()
        .find(|b| b.kind() == 4)
        .unwrap()
        .id();
    let body = v
        .document
        .runs()
        .iter()
        .find(|r| r.text() != NONE && v.document.text(r.text()).ok() == Some("BODY"))
        .unwrap()
        .id();
    let stop = AtomicBool::new(false);
    let target = v.navigator.selected_run(body, &stop).unwrap();
    let mut work = DocumentWork::frame(&stop);
    v.navigator.jump_selected(target, &mut work).unwrap();
    assert_eq!(v.navigator.history_len(), 1);
    let back = v.navigator.raw_back();
    let g = v.geometry();
    perform(
        &mut v,
        &mut a,
        Operation::Fold {
            block: callout,
            expanded: false,
        },
        g,
    );
    assert_eq!(semantic_symbols(&v), "TEND");
    assert_eq!(v.navigator.history_len(), 1);
    assert_eq!(v.navigator.raw_back(), back);
    let current = RawAnchor::state(v.navigator.current().unwrap());
    assert!(v.navigator.back().is_err());
    assert_eq!(v.navigator.history_len(), 1);
    assert_eq!(RawAnchor::state(v.navigator.current().unwrap()), current);
}
#[test]
fn b108_window_line_navigation_and_no_history() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (mut v, mut a) = view("A  \nB\n\nC\n", geometry(1));
    assert_eq!(symbols(&v), "A");
    let g = v.geometry();
    perform(&mut v, &mut a, Operation::Line { delta: 1 }, g);
    assert_eq!(symbols(&v), "B");
    perform(&mut v, &mut a, Operation::Line { delta: 1 }, g);
    assert_eq!(symbols(&v), "C");
    perform(&mut v, &mut a, Operation::Line { delta: -1 }, g);
    assert_eq!(symbols(&v), "B");
    for _ in 0..400 {
        perform(&mut v, &mut a, Operation::Home, g);
    }
    assert_eq!(symbols(&v), "A");
    assert_eq!(v.navigator.history_len(), 0);
}
#[test]
fn b109_stale_and_cancel_keep_current_frame() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (mut v, mut a) = view("CURRENT\n\nTAIL\n", geometry(1));
    let g = v.geometry();
    let old = v.request(Operation::End, g).unwrap();
    let binding = old.key.binding;
    start_document(old, &mut a).unwrap();
    let p = finish(&mut a, binding);
    let newer = v.request(Operation::Home, g).unwrap();
    assert_eq!(v.commit(p).err(), Some(Cause::Stale));
    assert_eq!(symbols(&v), "CURRENT");
    start_document(newer.clone(), &mut a).unwrap();
    a.stop();
    let end = Instant::now() + Duration::from_secs(3);
    while !a.is_finished() && Instant::now() < end {
        thread::yield_now();
    }
    assert_eq!(
        a.take_validated(newer.key.binding, true).err(),
        Some(Cause::Cancelled)
    );
    assert_eq!(symbols(&v), "CURRENT");
    assert!(!global_busy());
}
#[test]
fn b110_actual_oldnew_window_and_geometry_limit() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (mut v, mut a) = view("BODY\n", geometry(4));
    let before = v.budget.used();
    let g = v.geometry();
    let input = v
        .request(Operation::Resize, Geometry { width: 30, ..g })
        .unwrap();
    let b = input.key.binding;
    start_document(input, &mut a).unwrap();
    let p = finish(&mut a, b);
    assert!(v.budget.used() > before);
    eprintln!(
        "B110 oldCurrent={} currentPlusCandidate={} peak={} checkpoints={} currentWindowBytes={}",
        before,
        v.budget.used(),
        v.budget.peak(),
        v.budget.checkpoint_capacity(),
        v.layout.bytes()
    );
    assert!(v.budget.peak() <= 1048576);
    v.commit(p).unwrap();
    perform(
        &mut v,
        &mut a,
        Operation::Resize,
        Geometry {
            width: u16::MAX,
            height: u16::MAX,
            ..g
        },
    );
    assert!(matches!(
        v.layout(),
        LayoutWindow::Reference {
            reason: ReferenceReason::Window,
            ..
        }
    ));
    assert!(v.budget.used() <= 1048576);
    assert_eq!(v.document.source().text(), "BODY\n");
}
#[test]
fn b111_typed_document_pending_retains_real_input() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let input = DocumentView::initial_input(parsed("REAL\n"), geometry(2)).unwrap();
    let weak = Arc::downgrade(&input);
    let context = input.clone();
    let binding = input.key.binding;
    let (ready_tx, ready_rx) = mpsc::sync_channel(1);
    let release = Arc::new(AtomicBool::new(false));
    let _release_on_unwind = ReleaseOwnWorker(release.clone());
    let z = release.clone();
    let mut a = DocumentPreparationAuthority::default();
    a.start_document_owner(|| {
        OwnedPreparation::spawn(input, binding, move |_, stop| {
            let p = run_prepare(context, stop);
            ready_tx.send(()).unwrap();
            let deadline = Instant::now() + Duration::from_secs(5);
            while !z.load(Ordering::Acquire) && Instant::now() < deadline {
                thread::yield_now();
            }
            p
        })
    })
    .unwrap();
    ready_rx.recv_timeout(Duration::from_secs(3)).unwrap();
    assert!(!a.poll().unwrap());
    assert!(a.take_validated(binding, false).unwrap().is_none());
    assert_eq!(a.cleanup(Instant::now()), Err(Cause::Pending));
    drop(a);
    assert!(weak.upgrade().is_some());
    let mut next = DocumentPreparationAuthority::default();
    let i = DocumentView::initial_input(parsed("NEXT\n"), geometry(1)).unwrap();
    assert_eq!(
        start_document(i.clone(), &mut next).err(),
        Some(Cause::Pending)
    );
    release.store(true, Ordering::Release);
    let deadline = Instant::now() + Duration::from_secs(3);
    while retry_pending().unwrap().is_none() {
        assert!(Instant::now() < deadline);
        thread::yield_now();
    }
    assert!(weak.upgrade().is_none());
    let b = i.key.binding;
    start_document(i, &mut next).unwrap();
    let p = finish(&mut next, b);
    let v = DocumentView::from_prepared(p).unwrap();
    assert_eq!(symbols(&v), "NEXT");
}
#[test]
fn b112_same_authority_real_document_hash_source() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (v, mut a) = view("ACTUAL\n", geometry(1));
    let i = crate::source_preparation::SourcePreparationInput::new(
        v.document.source().original_handle(),
        crate::input::SourceOrigin::Literal,
        7,
        9,
    )
    .unwrap();
    let b = i.binding(0);
    crate::source_preparation::start_hash(i, 0, &mut a).unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while !a.is_finished() && Instant::now() < deadline {
        thread::yield_now();
    }
    let Some(PreparedCompletion::Source(
        Outcome::Ready(crate::source_preparation::SourcePrepared::Hash(c)),
        _,
    )) = a.take_validated(b, false).unwrap()
    else {
        panic!("Hash after Document")
    };
    let b = c.binding(0);
    crate::source_preparation::start_source(c, 0, &mut a).unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while !a.is_finished() && Instant::now() < deadline {
        thread::yield_now();
    }
    assert!(a.is_finished());
    let Some(PreparedCompletion::Source(
        Outcome::Ready(crate::source_preparation::SourcePrepared::Model(m)),
        _,
    )) = a.take_validated(b, false).unwrap()
    else {
        panic!("Source after Document")
    };
    assert_eq!(m.model().original(), "ACTUAL\n");
    assert!(Arc::ptr_eq(
        &m.model().original_handle(),
        &v.document.source().original_handle()
    ));
}
#[test]
fn b113_terminal_wrapped_line_without_geometry_replay() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let g = Geometry {
        width: 4,
        height: 1,
        ..geometry(1)
    };
    let (mut v, mut a) = view("abcdefghij", g);
    assert_eq!(symbols(&v), "abcd");
    perform(&mut v, &mut a, Operation::End, g);
    assert!(symbols(&v).ends_with('j'));
    assert_eq!(v.document.source().text(), "abcdefghij");
    perform(&mut v, &mut a, Operation::Home, g);
    assert_eq!(symbols(&v), "abcd");
}
#[test]
fn b114_raw_anchor_rejected_before_geometry_and_quota_reference() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    for source in ["abc", "éx"] {
        let (mut v, mut a) = view(source, geometry(2));
        let current = RawAnchor::state(v.navigator.current().unwrap());
        let invalid = if source == "abc" { 4 } else { 1 };
        for g in [
            geometry(2),
            Geometry {
                width: 0,
                ..geometry(2)
            },
            Geometry {
                width: u16::MAX,
                height: u16::MAX,
                ..geometry(2)
            },
        ] {
            let input = v
                .request(
                    Operation::Seek(RawAnchor {
                        byte: invalid,
                        ..current
                    }),
                    g,
                )
                .unwrap();
            let b = input.key.binding;
            start_document(input, &mut a).unwrap();
            let p = finish(&mut a, b);
            assert_eq!(v.commit(p).err(), Some(Cause::Stale));
            assert_eq!(RawAnchor::state(v.navigator.current().unwrap()), current);
            assert_eq!(symbols(&v), source);
            assert_eq!(v.navigator.history_len(), 0);
        }
        for bad in [
            RawAnchor {
                block: 0,
                ..current
            },
            RawAnchor {
                run: NONE,
                byte: 1,
                ..current
            },
        ] {
            let input = v
                .request(
                    Operation::Seek(bad),
                    Geometry {
                        width: 0,
                        ..geometry(2)
                    },
                )
                .unwrap();
            let b = input.key.binding;
            start_document(input, &mut a).unwrap();
            let p = finish(&mut a, b);
            assert_eq!(v.commit(p).err(), Some(Cause::Stale));
            assert_eq!(RawAnchor::state(v.navigator.current().unwrap()), current);
        }
    }
}
#[test]
fn b115_break_resource_anchors_and_valid_utf8_boundaries() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (mut v, mut a) = view("éx  \n![alt](i.png)\n", geometry(4));
    let g = Geometry {
        width: 0,
        ..geometry(4)
    };
    let bad: Vec<_> = v
        .document
        .runs()
        .iter()
        .filter(|r| matches!(r.kind(), 2 | 3 | 5))
        .map(|r| RawAnchor {
            block: v.document.domains()[r.domain() as usize].owner(),
            run: r.id(),
            byte: 1,
        })
        .collect();
    assert!(bad.len() >= 2);
    let current = RawAnchor::state(v.navigator.current().unwrap());
    for anchor in bad {
        let input = v.request(Operation::Seek(anchor), g).unwrap();
        let b = input.key.binding;
        start_document(input, &mut a).unwrap();
        let p = finish(&mut a, b);
        assert_eq!(v.commit(p).err(), Some(Cause::Stale));
        assert_eq!(RawAnchor::state(v.navigator.current().unwrap()), current);
        let input = v
            .request(Operation::Seek(RawAnchor { byte: 0, ..anchor }), g)
            .unwrap();
        let b = input.key.binding;
        start_document(input, &mut a).unwrap();
        let p = finish(&mut a, b);
        v.commit(p).unwrap();
        perform(&mut v, &mut a, Operation::Home, geometry(4));
    }
    for byte in [0, 2, 3] {
        perform(
            &mut v,
            &mut a,
            Operation::Seek(RawAnchor { byte, ..current }),
            g,
        );
        assert_eq!(v.navigator.current().unwrap().raw_byte(), byte);
    }
}
#[test]
fn b116_c1_visual_progress_and_resize_without_source_offset_reinterpretation() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let g = Geometry {
        width: 4,
        height: 1,
        ..geometry(1)
    };
    let (mut v, mut a) = view("A\u{85}Z", g);
    assert_eq!(symbols(&v), "A\\u{");
    let first = v.layout.text().unwrap().next_anchor().unwrap();
    assert_eq!(first.raw.byte, 1);
    assert_eq!(first.projection_part, 3);
    perform(&mut v, &mut a, Operation::Line { delta: 1 }, g);
    assert_eq!(symbols(&v), "0085");
    assert_eq!(v.navigator.current().unwrap().raw_byte(), 1);
    assert_eq!(v.navigator.current().unwrap().projection_part(), 3);
    perform(
        &mut v,
        &mut a,
        Operation::Resize,
        Geometry { width: 3, ..g },
    );
    assert_eq!(symbols(&v), "008");
    assert_eq!(v.navigator.current().unwrap().projection_part(), 3);
    perform(&mut v, &mut a, Operation::Resize, g);
    assert_eq!(symbols(&v), "0085");
    perform(&mut v, &mut a, Operation::Line { delta: 1 }, g);
    assert_eq!(symbols(&v), "}Z");
    assert_eq!(v.navigator.current().unwrap().raw_byte(), 1);
    assert_eq!(v.navigator.current().unwrap().projection_part(), 7);
    assert_eq!(v.navigator.history_len(), 0);
    println!("B116 projected=A\\u{{0085}}Z frames=3 rawC1=1 part3_then7 history=0");
}
#[test]
fn b117_visual_affinity_invalid_before_geometry_reference() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    for source in ["A\u{85}Z", "éx", "A\u{1b}Z"] {
        let (mut v, mut a) = view(source, geometry(2));
        let before = LayoutAnchor::state(v.navigator.current().unwrap());
        let content = symbols(&v).to_owned();
        let physical = RawAnchor {
            byte: if source.starts_with('é') { 0 } else { 1 },
            ..before.raw
        };
        let part = if source.contains('\u{85}') { 8 } else { 1 };
        for g in [
            Geometry {
                width: 0,
                ..geometry(1)
            },
            Geometry {
                width: u16::MAX,
                height: u16::MAX,
                ..geometry(1)
            },
        ] {
            let input = v
                .request(
                    Operation::Continue(LayoutAnchor {
                        raw: physical,
                        projection_part: part,
                    }),
                    g,
                )
                .unwrap();
            let binding = input.key.binding;
            start_document(input, &mut a).unwrap();
            let prepared = finish(&mut a, binding);
            assert_eq!(v.commit(prepared).err(), Some(Cause::Stale));
            assert_eq!(LayoutAnchor::state(v.navigator.current().unwrap()), before);
            assert_eq!(symbols(&v), content);
            assert_eq!(v.navigator.history_len(), 0);
        }
    }
}

#[path = "document_b2_tests.rs"]
mod b2;

// Fold/history oracle reads only genuine semantic glyphs; b2 role/ref decorations are separately tested.
fn semantic_symbols(v: &DocumentView) -> String {
    let w = v.layout().text().unwrap();
    w.glyphs()
        .iter()
        .filter(|g| g.first.is_some())
        .map(|g| &w.symbols()[g.start as usize..g.end as usize])
        .collect()
}
