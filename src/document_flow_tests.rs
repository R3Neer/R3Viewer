//! PREOUTPUT D2A01..12; do not derive expected values from Flow outputs.
use super::*;
use crate::{
    document_budget::ActorArena,
    document_consumer_profile::ConsumerProfile,
    document_model::{Block, Candidate, Domain, Item, Origin, Profile, Run, TextRef},
    document_navigation::Navigator,
    document_obsidian::Control,
    document_source::DocumentSource,
    document_text::{IndexProfile, SegmentedIndex, SegmentedText},
    input::SourceOrigin,
};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
const EXPECTED: &str = include_str!("../tests/fixtures/document-d2a/EXPECTED-D2A.json");
fn parsed_in(text: &str, arena: ActorArena) -> Arc<Document> {
    parsed_owned(text.to_owned(), arena)
}
fn parsed_owned(text: String, arena: ActorArena) -> Arc<Document> {
    let cancel = AtomicBool::new(false);
    let source = Arc::new(
        DocumentSource::new(Arc::new(text), SourceOrigin::Literal, 1, 1, &cancel).unwrap(),
    );
    let ctl = Control {
        cancel: &cancel,
        deadline: Instant::now() + Duration::from_secs(10),
    };
    let candidate =
        crate::document_parser::project(source, Profile::Extended, arena, &ctl).unwrap();
    let depth = candidate.max_depth;
    Arc::new(Document::from_validated(
        crate::document_validate::quarantine(candidate, depth, &ctl).unwrap(),
    ))
}
fn parsed(text: &str) -> Arc<Document> {
    parsed_in(text, ActorArena::new())
}
fn fixed_graph() -> Arc<Document> {
    let cancel = AtomicBool::new(false);
    let source = Arc::new(
        DocumentSource::new(
            Arc::new("FIRST MIDDLE TAIL".to_owned()),
            SourceOrigin::Literal,
            1,
            1,
            &cancel,
        )
        .unwrap(),
    );
    let mut c = Candidate::empty(source, Profile::Extended, ActorArena::new());
    c.origins.push(Origin::from_words([0, 0, 17, 0])).unwrap();
    for (id, parent, first, count, domain, kind) in [
        (0, NONE, 0, 3, NONE, 0),
        (1, 0, 3, 1, 2, 1),
        (2, 0, 4, 1, NONE, 3),
        (3, 2, 5, 1, 1, 1),
        (4, 0, 6, 1, 0, 1),
    ] {
        c.blocks
            .push(Block::from_words([
                id, parent, first, count, domain, kind, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]))
            .unwrap();
    }
    for words in [
        [0, 0, 0, 4],
        [0, 1, 0, 2],
        [0, 2, 0, 1],
        [1, 0, 1, 2],
        [2, 0, 0, 3],
        [3, 0, 1, 1],
        [4, 0, 1, 0],
    ] {
        c.items.push(Item::from_words(words)).unwrap();
    }
    for (id, owner, start, end) in [(0, 4, 0, 5), (1, 3, 6, 12), (2, 1, 13, 17)] {
        c.domains
            .push(Domain::from_words([id, 0, owner, id, 1, 0]))
            .unwrap();
        c.origins
            .push(Origin::from_words([0, start, end, 0]))
            .unwrap();
        c.texts
            .push(TextRef::from_words([0, start, end, id + 1]))
            .unwrap();
        c.runs
            .push(Run::from_words([
                id,
                id,
                0,
                0,
                0,
                id,
                id + 1,
                NONE,
                NONE,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ]))
            .unwrap();
    }
    let ctl = Control {
        cancel: &cancel,
        deadline: Instant::now() + Duration::from_secs(10),
    };
    c.max_depth = crate::document_validate::validate(&c, &ctl).unwrap();
    let depth = c.max_depth;
    Arc::new(Document::from_validated(
        crate::document_validate::quarantine(c, depth, &ctl).unwrap(),
    ))
}
fn events(index: &FlowIndex) -> Vec<Event> {
    let cancel = AtomicBool::new(false);
    let mut w = DocumentWork::preparation(&cancel);
    let mut cur = index.start().unwrap();
    let mut e = vec![];
    while let Some(v) = index.next_event(&mut cur, &mut w).unwrap() {
        e.push(v);
    }
    e
}
fn fixture_source(name: &str) -> String {
    match name {
        "two-paragraphs" => {
            include_str!("../tests/fixtures/document-d2a/sources/two-paragraphs.md")
        }
        "styles" => include_str!("../tests/fixtures/document-d2a/sources/styles.md"),
        "resource" => include_str!("../tests/fixtures/document-d2a/sources/resource.md"),
        "roles" => include_str!("../tests/fixtures/document-d2a/sources/roles.md"),
        _ => panic!("unknown fixed fixture"),
    }
    .to_owned()
}
#[test]
fn d2a01_empty() {
    assert!(EXPECTED.contains("D2A01") && EXPECTED.contains("D2A12"));
    let d = parsed("");
    let b = ConsumerBudget::new(&d).unwrap();
    let i = FlowIndex::prepare(d, &b, 2, &AtomicBool::new(false)).unwrap();
    assert_eq!(events(&i), vec![Event::Enter(0), Event::Leave(0)]);
    assert!(i.first().is_none() && i.terminal().is_none());
    assert_eq!(i.event_count(), 2);
}
#[test]
fn d2a02_flat_exact() {
    let d = parsed(&fixture_source("two-paragraphs"));
    let b = ConsumerBudget::new(&d).unwrap();
    let i = FlowIndex::prepare(d.clone(), &b, 3, &AtomicBool::new(false)).unwrap();
    assert_eq!(
        events(&i),
        vec![
            Event::Enter(0),
            Event::Enter(1),
            Event::Run { block: 1, run: 0 },
            Event::Leave(1),
            Event::Enter(2),
            Event::Run { block: 2, run: 1 },
            Event::Leave(2),
            Event::Leave(0)
        ]
    );
    assert_eq!(i.first().unwrap().run, 0);
    assert_eq!(i.terminal().unwrap().run, 1);
    assert_eq!(i.terminal().unwrap().raw_byte, 1);
    assert_eq!(d.source().text(), "A\n\nB\n");
}
#[test]
fn d2a03_styles_endpoint() {
    let d = parsed(&fixture_source("styles"));
    let b = ConsumerBudget::new(&d).unwrap();
    let i = FlowIndex::prepare(d.clone(), &b, 4, &AtomicBool::new(false)).unwrap();
    let r: Vec<_> = events(&i)
        .into_iter()
        .filter_map(|e| {
            if let Event::Run { run, .. } = e {
                Some(d.text(d.runs()[run as usize].text()).unwrap().to_owned())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(r, ["a", "\u{301}", "TAIL"]);
    assert_eq!(i.terminal().unwrap().raw_byte, 4);
    let p = IndexProfile {
        projection_points: 2,
        unicode_points: 2,
        exceptions: 0,
    };
    let text =
        SegmentedText::prepare_with_profile(d, 0, 0..2, &b, &AtomicBool::new(false), p).unwrap();
    let ix = SegmentedIndex::prepare_with_profile(text, &b, &AtomicBool::new(false), p).unwrap();
    let cancel = AtomicBool::new(false);
    let mut point = ix.start();
    assert_eq!(
        ix.step(&mut point, 4, &mut DocumentWork::frame(&cancel))
            .unwrap()
            .unwrap()
            .projected,
        0..3
    );
}
#[test]
fn d2a04_resource_barrier() {
    let d = parsed(&fixture_source("resource"));
    let b = ConsumerBudget::new(&d).unwrap();
    let i = FlowIndex::prepare(d.clone(), &b, 4, &AtomicBool::new(false)).unwrap();
    let kinds: Vec<_> = events(&i)
        .into_iter()
        .filter_map(|e| {
            if let Event::Run { run, .. } = e {
                Some(d.runs()[run as usize].kind())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(kinds, [0, 5, 0, 0]);
    assert_eq!(
        d.text(d.runs()[i.terminal().unwrap().run as usize].text())
            .unwrap(),
        "TAIL"
    );
    assert_eq!(d.resources().len(), 1);
    assert_eq!(
        d.text(d.resources()[0].input_decoded()).unwrap(),
        "image.png"
    );
}
#[test]
fn d2a05_roles() {
    let d = parsed(&fixture_source("roles"));
    let b = ConsumerBudget::new(&d).unwrap();
    let i = FlowIndex::prepare(d.clone(), &b, 3, &AtomicBool::new(false)).unwrap();
    let text: Vec<_> = events(&i)
        .into_iter()
        .filter_map(|e| {
            if let Event::Run { run, .. } = e {
                Some(d.text(d.runs()[run as usize].text()).unwrap().to_owned())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(text, ["H", "Q", "TAIL"]);
    assert_eq!(d.blocks()[1].kind(), 2);
    assert_eq!(d.blocks()[2].kind(), 3);
}
#[test]
fn d2a06_permuted_graph() {
    let d = fixed_graph();
    let b = ConsumerBudget::new(&d).unwrap();
    let i = FlowIndex::prepare(d, &b, 3, &AtomicBool::new(false)).unwrap();
    assert_eq!(
        events(&i),
        vec![
            Event::Enter(0),
            Event::Enter(4),
            Event::Run { block: 4, run: 0 },
            Event::Leave(4),
            Event::Enter(2),
            Event::Enter(3),
            Event::Run { block: 3, run: 1 },
            Event::Leave(3),
            Event::Leave(2),
            Event::Enter(1),
            Event::Run { block: 1, run: 2 },
            Event::Leave(1),
            Event::Leave(0)
        ]
    );
    assert_eq!(
        i.locate_run(2, &AtomicBool::new(false)).unwrap().ordinal(),
        10
    );
    assert_eq!(i.terminal().unwrap().block, 1);
}
#[test]
fn d2a08_slots_and_profiles() {
    let d = fixed_graph();
    let b = ConsumerBudget::new(&d).unwrap();
    for slots in [2, 3, 5] {
        let i = FlowIndex::prepare(d.clone(), &b, slots, &AtomicBool::new(false)).unwrap();
        assert_eq!(i.event_count(), 13);
        let cancel = AtomicBool::new(false);
        for n in 0..13 {
            assert_eq!(
                i.event_at(&i.location(n).unwrap(), &mut DocumentWork::frame(&cancel))
                    .unwrap(),
                events(&i)[n as usize]
            );
        }
    }
    assert_eq!(b.checkpoint_capacity(), 0);
    let profile = ConsumerProfile::plan(&b, 4096, IndexProfile::DEFAULT, 2).unwrap();
    assert!(profile.checkpoint_capacity <= 4096 && profile.requested_bytes <= 1048576 - b.used());
    assert_eq!(
        IndexProfile {
            projection_points: 1,
            unicode_points: 2,
            exceptions: 0
        }
        .checked()
        .err(),
        Some(Error::Budget)
    );
}
#[test]
fn d2a09_work_preserves_cursor() {
    let d = parsed(&"p\n\n".repeat(8193));
    let b = ConsumerBudget::new(&d).unwrap();
    let i = FlowIndex::prepare(d, &b, 2, &AtomicBool::new(false)).unwrap();
    let cancel = AtomicBool::new(false);
    let mut cur = i.start().unwrap();
    assert_eq!(
        i.next_event(&mut cur, &mut DocumentWork::bounded(&cancel, 31).unwrap()),
        Err(Error::Work)
    );
    assert_eq!(cur.ordinal(), 0);
    let target = i.location(16000).unwrap();
    assert_eq!(
        i.event_at(&target, &mut DocumentWork::frame(&cancel)),
        Err(Error::Work)
    );
    let other = FlowIndex::prepare(
        fixed_graph(),
        &ConsumerBudget::new(&fixed_graph()).unwrap(),
        2,
        &cancel,
    );
    assert_eq!(other.err(), Some(Error::Invalid));
}
#[test]
fn d2a10_oldnew_caps() {
    let d = fixed_graph();
    let b = ConsumerBudget::new(&d).unwrap();
    let initial = b.used();
    let old = FlowIndex::prepare(d.clone(), &b, 3, &AtomicBool::new(false)).unwrap();
    let afterold = b.used();
    let new = FlowIndex::prepare(d, &b, 5, &AtomicBool::new(false)).unwrap();
    assert!(b.used() > afterold && b.checkpoint_capacity() > 0);
    drop(new);
    assert_eq!(b.used(), afterold);
    let fill = b.reserve(1048576 - b.used()).unwrap();
    assert_eq!(
        b.reserve(1).err().unwrap().kind,
        crate::document_model::ErrorKind::ArenaLimit
    );
    drop(fill);
    drop(old);
    assert_eq!(b.used(), initial);
    let exact = b.reserve_points(0, 4096).unwrap();
    assert!(b.reserve_points(0, 1).is_err());
    drop(exact);
    assert_eq!(b.checkpoint_capacity(), 0);
}
#[test]
fn d2a11_history_stale_cancel() {
    let d = fixed_graph();
    let b = ConsumerBudget::new(&d).unwrap();
    let cancel = AtomicBool::new(false);
    let mut nav = Navigator::prepare(d.clone(), 3, &b, &cancel).unwrap();
    for n in 0..128 {
        let selected = nav
            .selected_run(if n % 2 == 0 { 1 } else { 0 }, &cancel)
            .unwrap();
        nav.jump_selected(selected, &mut DocumentWork::frame(&cancel))
            .unwrap();
    }
    assert_eq!(nav.history_len(), 128);
    let before = nav.current().unwrap().location().ordinal();
    let selected = nav.selected_run(1, &cancel).unwrap();
    assert_eq!(
        nav.jump_selected(selected, &mut DocumentWork::frame(&cancel)),
        Err(Error::Budget)
    );
    assert_eq!(nav.current().unwrap().location().ordinal(), before);
    nav.back().unwrap();
    assert_eq!(nav.history_len(), 127);
    let foreign = FlowIndex::prepare(d.clone(), &b, 2, &cancel).unwrap();
    assert_eq!(
        nav.seek_ordinal(
            foreign.location(0).unwrap(),
            &mut DocumentWork::frame(&cancel)
        ),
        Err(Error::Invalid)
    );
    cancel.store(true, Ordering::Release);
    let bytes = b.used();
    assert_eq!(
        FlowIndex::prepare(d, &b, 2, &cancel).err(),
        Some(Error::Cancelled)
    );
    assert_eq!(b.used(), bytes);
}
#[test]
fn d2a12_shared_actor() {
    let arena = ActorArena::new();
    let old = parsed_in("OLD", arena.clone());
    let before = arena.used();
    let oldbudget = ConsumerBudget::new(&old).unwrap();
    assert_eq!(arena.used() - before, 1048576);
    let new = parsed_in("NEW", arena.clone());
    let before_new = arena.used();
    let newbudget = ConsumerBudget::new(&new).unwrap();
    assert_eq!(arena.used() - before_new, 1048576);
    drop(old);
    assert!(arena.used() >= before_new + 1048576);
    drop(oldbudget);
    drop(new);
    drop(newbudget);
    assert_eq!(arena.used(), crate::document_budget::TRANSPORT_RESERVE);
}

#[test]
fn d2a07_giant_first_terminal_no_root() {
    let mut source = String::with_capacity(33630000);
    source.push('a');
    for _ in 0..16777216 {
        source.push('\u{301}');
    }
    assert_eq!(source.len(), 33554433);
    for i in 1..8193 {
        source.push_str("\n\n");
        if i == 8192 {
            source.push_str("TAIL");
        } else {
            use std::fmt::Write;
            write!(source, "p{i}").unwrap();
        }
    }
    let source_bytes = source.len();
    let start = Instant::now();
    let d = parsed_owned(source, ActorArena::new());
    let b = ConsumerBudget::new(&d).unwrap();
    let cancel = AtomicBool::new(false);
    let i = FlowIndex::prepare(d.clone(), &b, 8, &cancel).unwrap();
    let last = i.terminal().unwrap();
    assert_eq!(d.text(d.runs()[last.run as usize].text()).unwrap(), "TAIL");
    let mut stats = QueryStats::default();
    for _ in 0..10 {
        assert_eq!(
            i.event_at_with_stats(
                &last.location,
                &mut DocumentWork::frame(&cancel),
                &mut stats
            )
            .unwrap(),
            Event::Run {
                block: last.block,
                run: last.run
            }
        );
    }
    assert_eq!(stats.root_enters, 0);
    assert_eq!(stats.checkpoint_reads, 0);
    assert_eq!(stats.record_reads, 10);
    let first = i.first().unwrap();
    assert_eq!(
        d.text(d.runs()[first.run as usize].text()).unwrap().len(),
        33554433
    );
    let mut home_stats = QueryStats::default();
    i.event_at_with_stats(
        &first.location,
        &mut DocumentWork::frame(&cancel),
        &mut home_stats,
    )
    .unwrap();
    assert_eq!(home_stats.root_enters, 0);
    println!(
        "D2A07 sourceBytes={source_bytes} events={} preparationWork={} consumerPeak={} rootEnd={} rootHome={} seconds={}",
        i.event_count(),
        i.preparation_work(),
        b.peak(),
        stats.root_enters,
        home_stats.root_enters,
        start.elapsed().as_secs_f64()
    );
}

#[test]
fn d2a09_known_anchor_pending() {
    let d = fixed_graph();
    let b = ConsumerBudget::new(&d).unwrap();
    let cancel = AtomicBool::new(false);
    let mut nav = Navigator::prepare(d, 2, &b, &cancel).unwrap();
    let selected = nav.selected_run(1, &cancel).unwrap();
    assert_eq!(
        (selected.block(), selected.run(), selected.raw_byte()),
        (3, 1, 0)
    );
    let ordinal = selected.location().ordinal();
    nav.seek_selected(selected, &mut DocumentWork::bounded(&cancel, 0).unwrap())
        .unwrap();
    let current = nav.current().unwrap();
    assert_eq!(
        (
            current.block(),
            current.run(),
            current.raw_byte(),
            current.location().ordinal()
        ),
        (3, 1, 0, ordinal)
    );
    assert!(current.position_pending());
}

#[test]
fn d2a11_ordinary_not_history() {
    let d = fixed_graph();
    let b = ConsumerBudget::new(&d).unwrap();
    let cancel = AtomicBool::new(false);
    let mut nav = Navigator::prepare(d, 3, &b, &cancel).unwrap();
    for _ in 0..400 {
        nav.end().unwrap();
        nav.home().unwrap();
    }
    assert_eq!(nav.history_len(), 0);
    let same = nav.selected_run(0, &cancel).unwrap();
    for _ in 0..200 {
        nav.jump_selected(same.clone(), &mut DocumentWork::frame(&cancel))
            .unwrap();
    }
    assert_eq!(nav.history_len(), 0);
}
#[test]
fn d2a11_fold_actual_visibility() {
    let d = parsed("> [!note]- T\n> BODY\n\nTAIL\n");
    let b = ConsumerBudget::new(&d).unwrap();
    let cancel = AtomicBool::new(false);
    let callout = d.blocks().iter().find(|v| v.kind() == 4).unwrap();
    assert_eq!(callout.aux2(), 2);
    let block = callout.id();
    let old = FlowIndex::prepare(d.clone(), &b, 3, &cancel).unwrap();
    assert_eq!(old.event_count(), 7);
    let old_location = old.terminal().unwrap().location.clone();
    let after_old = b.used();
    let new = FlowIndex::prepare_with_folds(
        d.clone(),
        &b,
        3,
        &[FoldOverride {
            block,
            expanded: true,
        }],
        &cancel,
    )
    .unwrap();
    assert_eq!(new.event_count(), 10);
    assert!(b.used() > after_old);
    let payload = |i: &FlowIndex| {
        events(i)
            .into_iter()
            .filter_map(|e| {
                if let Event::Run { run, .. } = e {
                    Some(d.text(d.runs()[run as usize].text()).unwrap().to_owned())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(payload(&old), ["TAIL"]);
    assert_eq!(payload(&new), ["BODY", "TAIL"]);
    assert_eq!(
        new.event_at(&old_location, &mut DocumentWork::frame(&cancel)),
        Err(Error::Invalid)
    );
    assert!(d.source().text().contains("BODY"));
    assert_eq!(
        FlowIndex::prepare_with_folds(
            d,
            &b,
            2,
            &[
                FoldOverride {
                    block,
                    expanded: true
                },
                FoldOverride {
                    block,
                    expanded: false
                }
            ],
            &cancel
        )
        .err(),
        Some(Error::Invalid)
    );
    drop(new);
    assert_eq!(b.used(), after_old);
}

#[test]
fn d2a10_cursor_owned_capacity() {
    let d = fixed_graph();
    let b = ConsumerBudget::new(&d).unwrap();
    let cancel = AtomicBool::new(false);
    let initial = b.used();
    let i = FlowIndex::prepare(d.clone(), &b, 2, &cancel).unwrap();
    let before = b.used();
    let points = b.checkpoint_capacity();
    let cur = i.start().unwrap();
    assert!(b.used() > before);
    assert_eq!(b.checkpoint_capacity(), points + 1);
    let fill = b.reserve_points(0, 4096 - b.checkpoint_capacity()).unwrap();
    assert_eq!(i.start().err(), Some(Error::Budget));
    assert_eq!(b.checkpoint_capacity(), 4096);
    drop(fill);
    drop(i);
    drop(d);
    assert!(b.used() > initial);
    drop(cur);
    assert_eq!(b.checkpoint_capacity(), 0);
    assert!(b.used() < before);
}
