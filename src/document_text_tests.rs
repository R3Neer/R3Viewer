//! Tests generated from fixed PREOUTPUT JSON; expected fields are not candidate output.
use super::*;
use crate::{
    document_budget::ActorArena,
    document_model::{Block, Candidate, Domain, Item, Origin, Profile, Run, Span, TextRef},
    document_obsidian::Control,
    document_source::DocumentSource,
    document_validate,
    input::SourceOrigin,
};
use std::time::{Duration, Instant};
fn fixture(runs: &[(&str, u32)], decoded: bool) -> Arc<Document> {
    let cancel = AtomicBool::new(false);
    let input = if decoded {
        "&eacute;&#769;".to_owned()
    } else {
        runs.iter().map(|r| r.0).collect::<String>()
    };
    let source = Arc::new(
        DocumentSource::new(Arc::new(input), SourceOrigin::Literal, 1, 1, &cancel).unwrap(),
    );
    let mut c = Candidate::empty(source, Profile::Extended, ActorArena::new());
    let n = runs.len() as u32;
    c.origins
        .push(Origin::from_words([0, 0, c.source.len() as u32, 0]))
        .unwrap();
    c.blocks
        .push(Block::from_words([
            0, NONE, 0, 1, NONE, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]))
        .unwrap();
    c.blocks
        .push(Block::from_words([
            1, 0, 1, n, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]))
        .unwrap();
    c.items.push(Item::from_words([0, 0, 0, 1])).unwrap();
    c.domains
        .push(Domain::from_words([0, 0, 1, 0, n, 0]))
        .unwrap();
    let mut byte = 0u32;
    for (i, (s, styles)) in runs.iter().enumerate() {
        let i = i as u32;
        let end = byte + s.len() as u32;
        let origin = if decoded {
            if i == 0 { [1, 0, 8, 0] } else { [1, 8, 14, 0] }
        } else {
            [0, byte, end, 0]
        };
        c.origins.push(Origin::from_words(origin)).unwrap();
        c.texts
            .push(TextRef::from_words([u32::from(decoded), byte, end, i + 1]))
            .unwrap();
        if decoded {
            c.decoded.extend_from_slice(s.as_bytes()).unwrap();
        }
        c.runs
            .push(Run::from_words([
                i,
                0,
                i,
                u32::from(styles & 8 != 0),
                *styles,
                i,
                i + 1,
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
        c.items.push(Item::from_words([1, i, 1, i])).unwrap();
        for lane in 0..3 {
            if styles & (1 << lane) != 0 {
                let id = c.spans.len() as u32;
                c.spans
                    .push(Span::from_words([
                        id,
                        0,
                        NONE,
                        i,
                        i + 1,
                        lane,
                        NONE,
                        i + 1,
                        0,
                        0,
                    ]))
                    .unwrap();
            }
        }
        byte = end;
    }
    c.max_depth = 1;
    let ctl = Control {
        cancel: &cancel,
        deadline: Instant::now() + Duration::from_secs(10),
    };
    let depth = document_validate::validate(&c, &ctl).unwrap();
    c.max_depth = depth;
    let proof = document_validate::quarantine(c, depth, &ctl).unwrap();
    Arc::new(Document::from_validated(proof))
}
fn verify(
    doc: Arc<Document>,
    visible: &str,
    expected: &[(usize, usize, usize)],
    hint: usize,
) -> (SegmentedIndex, ConsumerBudget) {
    let cancel = AtomicBool::new(false);
    let budget = ConsumerBudget::new(&doc).unwrap();
    let text = SegmentedText::prepare(
        doc.clone(),
        0,
        0..doc.domains()[0].run_count() as usize,
        &budget,
        &cancel,
    )
    .unwrap();
    assert_eq!(text.len(), visible.len());
    let mut actual = String::new();
    let mut work = DocumentWork(Work::preparation(&cancel), None);
    text.visit_projected(0..text.len(), &mut work, |s| {
        actual.push_str(s);
        Ok(())
    })
    .unwrap();
    assert_eq!(actual, visible);
    let index = SegmentedIndex::prepare(text, &budget, &cancel).unwrap();
    let mut at = index.start();
    let mut units = Vec::new();
    for _ in expected {
        let mut work = DocumentWork::frame(&cancel);
        let unit = index.step(&mut at, hint, &mut work).unwrap().unwrap();
        units.push((unit.projected.start, unit.projected.end, unit.width));
    }
    assert_eq!(units, expected);
    assert!(
        index
            .step(&mut at, hint, &mut DocumentWork::frame(&cancel))
            .unwrap()
            .is_none()
    );
    assert!(budget.peak() <= 1048576);
    (index, budget)
}
#[test]
fn dc01_a_accent() {
    for hint in [4, 5, 7, 32, 4096] {
        let (_index, _budget) = verify(
            fixture(&[("a", 1), ("́", 2)], false),
            "á",
            &[(0, 3, 1)],
            hint,
        );
        let cancel = AtomicBool::new(false);
        let mut got = Vec::new();
        _index
            .text()
            .visit_coverage(0..3, &mut DocumentWork::frame(&cancel), |r| {
                got.push((r.run, r.local.start, r.local.end));
                Ok(())
            })
            .unwrap();
        assert_eq!(got, vec![(0, 0, 1), (1, 0, 2)]);
    }
}
#[test]
fn dc01_zwj() {
    for hint in [4, 5, 7, 32, 4096] {
        let (_index, _budget) = verify(
            fixture(&[("👩", 1), ("🏽", 2), ("‍", 4), ("💻", 8)], false),
            "👩🏽‍💻",
            &[(0, 15, 2)],
            hint,
        );
        let cancel = AtomicBool::new(false);
        let mut got = Vec::new();
        _index
            .text()
            .visit_coverage(0..15, &mut DocumentWork::frame(&cancel), |r| {
                got.push((r.run, r.local.start, r.local.end));
                Ok(())
            })
            .unwrap();
        assert_eq!(got, vec![(0, 0, 4), (1, 0, 4), (2, 0, 3), (3, 0, 4)]);
    }
}
#[test]
fn dc01_ri_mixed() {
    for hint in [4, 5, 7, 32, 4096] {
        let (_index, _budget) = verify(
            fixture(&[("x🇦", 1), ("🇧", 2), ("🇨y", 4)], false),
            "x🇦🇧🇨y",
            &[(0, 1, 1), (1, 9, 2), (9, 13, 1), (13, 14, 1)],
            hint,
        );
    }
}
#[test]
fn dc03_controls() {
    for hint in [4, 5, 7, 32, 4096] {
        let (_index, _budget) = verify(
            fixture(&[("\u{0}\u{1b}", 1), ("\u{7f}", 2), ("␀", 4)], false),
            "␀␛␡\\u{009b}␀",
            &[
                (0, 3, 1),
                (3, 6, 1),
                (6, 9, 1),
                (9, 10, 1),
                (10, 11, 1),
                (11, 12, 1),
                (12, 13, 1),
                (13, 14, 1),
                (14, 15, 1),
                (15, 16, 1),
                (16, 17, 1),
                (17, 20, 1),
            ],
            hint,
        );
    }
}
#[test]
fn dc03_control_accent() {
    for hint in [4, 5, 7, 32, 4096] {
        let (_index, _budget) = verify(
            fixture(&[("\u{0}", 1), ("́", 2)], false),
            "␀́",
            &[(0, 5, 1)],
            hint,
        );
        let cancel = AtomicBool::new(false);
        let mut got = Vec::new();
        _index
            .text()
            .visit_coverage(0..5, &mut DocumentWork::frame(&cancel), |r| {
                got.push((r.run, r.local.start, r.local.end));
                Ok(())
            })
            .unwrap();
        assert_eq!(got, vec![(0, 0, 1), (1, 0, 2)]);
    }
}
#[test]
fn dc03_crlf_tab() {
    for hint in [4, 5, 7, 32, 4096] {
        let (_index, _budget) = verify(
            fixture(&[("a\u{d}", 1), ("\u{a}\u{9}", 2), ("b", 4)], false),
            "a\u{d}\u{a}\u{9}b",
            &[(0, 1, 1), (1, 3, 0), (3, 4, 8), (4, 5, 1)],
            hint,
        );
    }
}
#[test]
fn dc03_lone_cr() {
    for hint in [4, 5, 7, 32, 4096] {
        let (_index, _budget) = verify(
            fixture(&[("\u{d}", 1), ("a", 2)], false),
            "␍a",
            &[(0, 3, 1), (3, 4, 1)],
            hint,
        );
    }
}
#[test]
fn dc04_first_widthzero() {
    for hint in [4, 5, 7, 32, 4096] {
        let (_index, _budget) = verify(
            fixture(&[("́", 1), ("\u{a}", 2), ("TAIL", 4)], false),
            "́\u{a}TAIL",
            &[
                (0, 2, 0),
                (2, 3, 0),
                (3, 4, 1),
                (4, 5, 1),
                (5, 6, 1),
                (6, 7, 1),
            ],
            hint,
        );
    }
}
#[test]
fn dc05_empty_metadata() {
    for hint in [4, 5, 7, 32, 4096] {
        let (_index, _budget) = verify(
            fixture(&[("", 1), ("a", 2), ("", 4), ("́", 8), ("", 1)], false),
            "á",
            &[(0, 3, 1)],
            hint,
        );
        let cancel = AtomicBool::new(false);
        let mut got = Vec::new();
        _index
            .text()
            .visit_coverage(0..3, &mut DocumentWork::frame(&cancel), |r| {
                got.push((r.run, r.local.start, r.local.end));
                Ok(())
            })
            .unwrap();
        assert_eq!(got, vec![(1, 0, 1), (3, 0, 2)]);
    }
}
#[test]
fn dc05_decoded_token() {
    for hint in [4, 5, 7, 32, 4096] {
        let (_index, _budget) =
            verify(fixture(&[("é", 1), ("́", 2)], true), "é́", &[(0, 4, 1)], hint);
    }
}

#[test]
fn dc02_actual_chunk_edges_and_clone_identity() {
    let ri = "🇦".repeat(2051);
    let input = format!("x{ri}y");
    let splits = [0, 13, 1029, 4101, 6149, input.len()];
    let runs: Vec<_> = splits.windows(2).map(|p| (&input[p[0]..p[1]], 0)).collect();
    let mut expected = vec![(0, 1, 1)];
    for n in 0..1025 {
        expected.push((1 + 8 * n, 9 + 8 * n, 2));
    }
    expected.push((8201, 8205, 1));
    expected.push((8205, 8206, 1));
    for hint in [4, 5, 7, 32, 4096] {
        let (index, _budget) = verify(fixture(&runs, false), &input, &expected, hint);
        let cancel = AtomicBool::new(false);
        let mut a = index.start();
        index
            .step(&mut a, hint, &mut DocumentWork::frame(&cancel))
            .unwrap();
        let mut b = a.clone();
        let ua = index
            .step(&mut a, hint, &mut DocumentWork::frame(&cancel))
            .unwrap();
        let ub = index
            .step(&mut b, hint, &mut DocumentWork::frame(&cancel))
            .unwrap();
        assert_eq!(ua, ub);
        assert_eq!(a.offset(), b.offset());
        let (foreign, _) = verify(fixture(&[("a", 0)], false), "a", &[(0, 1, 1)], 4);
        assert_eq!(
            foreign.step(&mut a, 4, &mut DocumentWork::frame(&cancel)),
            Err(Error::Invalid)
        );
    }
}
#[test]
fn dc04_exact_and_oversized_cached_resume() {
    for (s, size) in [
        (format!("é{}", "\u{301}".repeat(8191)), 16384),
        (format!("a{}", "\u{301}".repeat(8192)), 16385),
    ] {
        let mid = if size == 16384 { 8192 } else { 8191 };
        let input = format!("{s}\nTAIL");
        let (index, _) = verify(
            fixture(&[(&input[..mid], 1), (&input[mid..], 2)], false),
            &input,
            &[
                (0, size, 1),
                (size, size + 1, 0),
                (size + 1, size + 2, 1),
                (size + 2, size + 3, 1),
                (size + 3, size + 4, 1),
                (size + 4, size + 5, 1),
            ],
            4,
        );
        let cancel = AtomicBool::new(false);
        let mut point = index.start();
        let mut work = DocumentWork::frame(&cancel);
        let unit = index.step(&mut point, 4, &mut work).unwrap().unwrap();
        assert_eq!(unit.oversized, size > 16384);
        if unit.oversized {
            assert!(work.used() < 4096);
        }
    }
}
#[test]
fn dc04_32m_borrowed_and_cancelled_no_replay() {
    let s = format!("é{}\nTAIL", "\u{301}".repeat(16777215));
    let doc = fixture(&[(&s[..16777216], 1), (&s[16777216..], 2)], false);
    let cancel = AtomicBool::new(false);
    let budget = ConsumerBudget::new(&doc).unwrap();
    let before = doc.arena_used();
    drop(s);
    let stamp = Instant::now();
    let text = SegmentedText::prepare(doc.clone(), 0, 0..2, &budget, &cancel).unwrap();
    let provider_seconds = stamp.elapsed().as_secs_f64();
    let stamp = Instant::now();
    let index = SegmentedIndex::prepare(text, &budget, &cancel).unwrap();
    assert_eq!(doc.arena_used(), before);
    assert!(Arc::ptr_eq(index.text().document(), &doc));
    println!(
        "D1_GIANT {{\"provider_seconds\":{},\"index_seconds\":{},\"provider_work\":{},\"index_work\":{},\"consumer_peak\":{}}}",
        provider_seconds,
        stamp.elapsed().as_secs_f64(),
        index.text().preparation_work(),
        index.preparation_work(),
        budget.peak()
    );
    let mut p = index.start();
    let mut work = DocumentWork::frame(&cancel);
    let u = index.step(&mut p, 4, &mut work).unwrap().unwrap();
    assert_eq!(u.projected, 0..33554432);
    assert!(u.oversized);
    assert!(work.used() < 4096);
    let mut work = DocumentWork::frame(&cancel);
    assert!(index.step(&mut p, 4, &mut work).unwrap().unwrap().newline);
    let tail = index.step(&mut p, 4, &mut work).unwrap().unwrap();
    assert_eq!(tail.line, 1);
    assert_eq!(tail.width, 1);
    cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    let offset = p.offset();
    assert_eq!(
        index.step(&mut p, 4, &mut DocumentWork::frame(&cancel)),
        Err(Error::Cancelled)
    );
    assert_eq!(p.offset(), offset);
    assert!(budget.peak() < 1048576);
}
#[test]
fn dc05_empty_runs_work_cancel_and_budget_retained() {
    let mut runs = vec![("", 0); 8192];
    runs.push(("a", 0));
    let doc = fixture(&runs, false);
    let cancel = AtomicBool::new(false);
    let budget = ConsumerBudget::new(&doc).unwrap();
    let text = SegmentedText::prepare(doc.clone(), 0, 0..runs.len(), &budget, &cancel).unwrap();
    let index = SegmentedIndex::prepare(text, &budget, &cancel).unwrap();
    let mut p = index.start();
    assert_eq!(
        index.step(&mut p, 4, &mut DocumentWork::frame(&cancel)),
        Err(Error::Work)
    );
    assert_eq!(p.offset(), 0);
    let mut work = DocumentWork::bounded(&cancel, 262144).unwrap();
    work.admit(262144).unwrap();
    assert_eq!(work.admit(1), Err(Error::Work));
    let used = budget.used();
    let keep = budget.reserve(1048576 - used).unwrap();
    assert_eq!(
        budget.reserve(1).err().unwrap().kind,
        crate::document_model::ErrorKind::ArenaLimit
    );
    drop(keep);
    assert_eq!(budget.used(), used);
}
#[test]
fn dc05_actual_parser_multiple_links_and_styles() {
    let cancel = AtomicBool::new(false);
    let source = Arc::new(
        DocumentSource::new(
            Arc::new("[a](one)[\u{301}](two)".to_owned()),
            SourceOrigin::Literal,
            1,
            1,
            &cancel,
        )
        .unwrap(),
    );
    let ctl = Control {
        cancel: &cancel,
        deadline: Instant::now() + Duration::from_secs(10),
    };
    let candidate =
        crate::document_parser::project(source, Profile::Extended, ActorArena::new(), &ctl)
            .unwrap();
    let depth = candidate.max_depth;
    let doc = Arc::new(Document::from_validated(
        document_validate::quarantine(candidate, depth, &ctl).unwrap(),
    ));
    let count = doc.domains()[0].run_count() as usize;
    let budget = ConsumerBudget::new(&doc).unwrap();
    let text = SegmentedText::prepare(doc, 0, 0..count, &budget, &cancel).unwrap();
    let index = SegmentedIndex::prepare(text, &budget, &cancel).unwrap();
    let mut p = index.start();
    let u = index
        .step(&mut p, 4, &mut DocumentWork::frame(&cancel))
        .unwrap()
        .unwrap();
    assert_eq!(u.projected, 0..3);
    let mut links = std::collections::BTreeSet::new();
    index
        .text()
        .visit_coverage(u.projected, &mut DocumentWork::frame(&cancel), |c| {
            if let Some(l) = c.link {
                links.insert(l);
            }
            Ok(())
        })
        .unwrap();
    assert_eq!(links.len(), 2);
}
fn parsed(input: &str) -> Arc<Document> {
    let cancel = AtomicBool::new(false);
    let source = Arc::new(
        DocumentSource::new(
            Arc::new(input.to_owned()),
            SourceOrigin::Literal,
            7,
            9,
            &cancel,
        )
        .unwrap(),
    );
    let ctl = Control {
        cancel: &cancel,
        deadline: Instant::now() + Duration::from_secs(10),
    };
    let c = crate::document_parser::project(source, Profile::Extended, ActorArena::new(), &ctl)
        .unwrap();
    let depth = c.max_depth;
    Arc::new(Document::from_validated(
        document_validate::quarantine(c, depth, &ctl).unwrap(),
    ))
}
#[test]
fn dc01_parser_source_disjoint() {
    let (index, _) = verify(parsed("a*\u{301}*"), "a\u{301}", &[(0, 3, 1)], 4);
    let cancel = AtomicBool::new(false);
    let mut sources = Vec::new();
    let mut styles = 0;
    index
        .text()
        .visit_coverage(0..3, &mut DocumentWork::frame(&cancel), |c| {
            styles |= c.styles;
            assert_eq!(c.relation, Relation::ExactSource);
            sources.push(c.source.unwrap());
            Ok(())
        })
        .unwrap();
    assert_eq!(styles, 1);
    assert_eq!(sources, vec![0..1, 2..4]);
}
#[test]
fn dc03_parser_entities_and_control_relations() {
    let (index, _) = verify(parsed("&eacute;&#769;"), "é\u{301}", &[(0, 4, 1)], 4);
    let cancel = AtomicBool::new(false);
    index
        .text()
        .visit_coverage(0..4, &mut DocumentWork::frame(&cancel), |c| {
            assert_eq!(c.relation, Relation::EnclosingToken);
            let source = c.source.unwrap();
            assert!(source.start < source.end && source.end <= 14);
            Ok(())
        })
        .unwrap();
    let doc = fixture(&[("\0", 0), ("␀", 0)], false);
    let (index, _) = verify(doc, "␀␀", &[(0, 3, 1), (3, 6, 1)], 4);
    let mut relations = Vec::new();
    index
        .text()
        .visit_coverage(0..6, &mut DocumentWork::frame(&cancel), |c| {
            relations.push(c.relation);
            Ok(())
        })
        .unwrap();
    assert_eq!(
        relations,
        vec![Relation::ControlProjection, Relation::ExactSource]
    );
}
#[test]
fn dc03_actual_hardbreak_is_boundary() {
    let (index, _) = verify(
        parsed("a  \n\u{301}"),
        "a\n\u{301}",
        &[(0, 1, 1), (1, 2, 0), (2, 4, 0)],
        4,
    );
    let cancel = AtomicBool::new(false);
    let mut point = index.start();
    index
        .step(&mut point, 4, &mut DocumentWork::frame(&cancel))
        .unwrap();
    index
        .step(&mut point, 4, &mut DocumentWork::frame(&cancel))
        .unwrap();
    let last = index
        .step(&mut point, 4, &mut DocumentWork::frame(&cancel))
        .unwrap()
        .unwrap();
    assert_eq!(last.line, 1);
    index
        .text()
        .visit_coverage(1..2, &mut DocumentWork::frame(&cancel), |c| {
            assert_eq!(c.relation, Relation::Synthetic);
            assert!(c.source.is_none());
            assert_ne!(c.origin, NONE);
            Ok(())
        })
        .unwrap();
}
#[test]
fn dc02_prefix_scaling_counter_bound() {
    for count in [129usize, 1025, 8193] {
        let input = format!("x{}y", "🇦".repeat(count));
        let mut ends = vec![0, 13, 261, 1029];
        ends.retain(|&n| n < input.len());
        ends.push(input.len());
        let runs: Vec<_> = ends.windows(2).map(|p| (&input[p[0]..p[1]], 0)).collect();
        let mut expected = vec![(0, 1, 1)];
        for n in 0..count / 2 {
            expected.push((1 + 8 * n, 9 + 8 * n, 2));
        }
        if count % 2 == 1 {
            expected.push((1 + 4 * (count - 1), 1 + 4 * count, 1));
        }
        expected.push((1 + 4 * count, 2 + 4 * count, 1));
        let (index, budget) = verify(fixture(&runs, false), &input, &expected, 32);
        let ceiling = 384 * input.len() + 8192;
        assert!(index.preparation_work() <= ceiling);
        assert!(index.text().preparation_work() <= ceiling);
        println!(
            "D1_COUNTERS {{\"ri\":{},\"bytes\":{},\"provider_work\":{},\"index_work\":{},\"context\":{},\"consumer_peak\":{},\"checkpoint_capacity\":{}}}",
            count,
            input.len(),
            index.text().preparation_work(),
            index.preparation_work(),
            index.preparation_counters()[1],
            budget.peak(),
            budget.checkpoint_capacity()
        );
    }
}
#[test]
fn dc05_foreign_budget_oldnew_and_cancel() {
    let cancel = AtomicBool::new(false);
    let doc = fixture(&[("a", 0)], false);
    let foreign = fixture(&[("a", 0)], false);
    let budget = ConsumerBudget::new(&doc).unwrap();
    let start_used = budget.used();
    assert!(matches!(
        SegmentedText::prepare(foreign, 0, 0..1, &budget, &cancel),
        Err(Error::Invalid)
    ));
    assert_eq!(budget.used(), start_used);
    let text = SegmentedText::prepare(doc.clone(), 0, 0..1, &budget, &cancel).unwrap();
    let index = SegmentedIndex::prepare(text, &budget, &cancel).unwrap();
    let used = budget.used();
    let held = budget.reserve(1048576 - used).unwrap();
    assert!(matches!(
        SegmentedText::prepare(doc.clone(), 0, 0..1, &budget, &cancel),
        Err(Error::Budget)
    ));
    let mut point = index.start();
    assert_eq!(
        index
            .step(&mut point, 4, &mut DocumentWork::frame(&cancel))
            .unwrap()
            .unwrap()
            .width,
        1
    );
    drop(held);
    assert_eq!(budget.used(), used);
    let new = SegmentedText::prepare(doc.clone(), 0, 0..1, &budget, &cancel).unwrap();
    let new = SegmentedIndex::prepare(new, &budget, &cancel).unwrap();
    assert!(budget.used() > used);
    drop(new);
    assert_eq!(budget.used(), used);
    cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    assert!(matches!(
        SegmentedText::prepare(doc, 0, 0..1, &budget, &cancel),
        Err(Error::Cancelled)
    ));
    assert_eq!(budget.used(), used);
}
#[test]
fn dc05_global_checkpoint_exact_plus_one() {
    let doc = fixture(&[("a", 0)], false);
    let budget = ConsumerBudget::new(&doc).unwrap();
    let before = budget.used();
    let held =
        crate::document_consumer_budget::ChargedVec::<u8>::new_points(&budget, 4096, 1).unwrap();
    assert_eq!(budget.checkpoint_capacity(), 4096);
    assert!(crate::document_consumer_budget::ChargedVec::<u8>::new_points(&budget, 1, 1).is_err());
    assert_eq!(budget.checkpoint_capacity(), 4096);
    drop(held);
    assert_eq!(budget.checkpoint_capacity(), 0);
    assert_eq!(budget.used(), before);
}
pub(super) static OBSERVE: AtomicBool = AtomicBool::new(false);
pub(super) static PROGRESS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
#[test]
fn dc02_cold_resolved_precontext() {
    let input = format!("x{}y", "🇦".repeat(8193));
    let doc = fixture(&[(&input[..1029], 0), (&input[1029..], 0)], false);
    let cancel = AtomicBool::new(false);
    let budget = ConsumerBudget::new(&doc).unwrap();
    let text = SegmentedText::prepare(doc, 0, 0..2, &budget, &cancel).unwrap();
    let index = SegmentedIndex::prepare(text, &budget, &cancel).unwrap();
    let mut work = DocumentWork(Work::preparation(&cancel), None);
    let position = index.text.locate(8193, &mut work).unwrap();
    let previous = index
        .text
        .previous_character(position, &mut work)
        .unwrap()
        .1;
    let mut p = Point {
        identity: index.text.identity.clone(),
        offset: 8193,
        line: 0,
        column: 2049,
        cursor: GraphemeCursor::new(8193, index.text.len, true),
        position,
        previous,
    };
    let mut work = DocumentWork::frame(&cancel);
    let u = index.step(&mut p, 4, &mut work).unwrap().unwrap();
    assert_eq!(u.projected, 8193..8201);
    assert_eq!(u.width, 2);
    assert_eq!(u.column, 2049);
    assert!(work.counters()[1] > 0);
    println!(
        "D1_PRECONTEXT {{\"context\":{},\"work\":{}}}",
        work.counters()[1],
        work.used()
    );
}
#[test]
fn dc04_mid_prepare_cancel_owned_observer() {
    use std::sync::atomic::Ordering;
    let s = format!("é{}", "\u{301}".repeat(16777215));
    let doc = fixture(&[(&s, 0)], false);
    drop(s);
    let budget = ConsumerBudget::new(&doc).unwrap();
    let before = budget.used();
    let cancel = AtomicBool::new(false);
    struct Armed;
    impl Drop for Armed {
        fn drop(&mut self) {
            OBSERVE.store(false, Ordering::Release);
        }
    }
    PROGRESS.store(0, Ordering::Release);
    OBSERVE.store(true, Ordering::Release);
    let _armed = Armed;
    std::thread::scope(|scope| {
        let observer = scope.spawn(|| {
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                let n = PROGRESS.load(Ordering::Acquire);
                if n >= 65536 || Instant::now() >= deadline {
                    cancel.store(true, Ordering::Release);
                    return n;
                }
                std::thread::yield_now();
            }
        });
        let result = SegmentedText::prepare(doc.clone(), 0, 0..1, &budget, &cancel);
        let witness = observer.join().unwrap();
        assert!(matches!(result, Err(Error::Cancelled)));
        assert!((65536..33554432).contains(&witness));
        assert_eq!(budget.used(), before);
        assert_eq!(doc.source().len(), 33554432);
        println!(
            "D1_CANCEL {{\"progress_at_cancel\":{},\"retained_source\":{},\"charges_restored\":true,\"observer_joined\":true}}",
            witness,
            doc.source().len()
        );
    });
}

#[test]
fn dc05_midrun_header_and_index_owner() {
    let cancel = AtomicBool::new(false);
    let doc = fixture(
        &[(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            0,
        )],
        false,
    );
    let foreign = fixture(&[("a", 0)], false);
    let own = ConsumerBudget::new(&doc).unwrap();
    let foreign_budget = ConsumerBudget::new(&foreign).unwrap();
    let before = own.used();
    let foreign_before = foreign_budget.used();
    let text = SegmentedText::prepare(doc.clone(), 0, 0..1, &own, &cancel).unwrap();
    assert!(matches!(
        SegmentedIndex::prepare(text, &foreign_budget, &cancel),
        Err(Error::Invalid)
    ));
    assert_eq!(own.used(), before);
    assert_eq!(foreign_budget.used(), foreign_before);
    let text = SegmentedText::prepare(doc, 0, 0..1, &own, &cancel).unwrap();
    let index = SegmentedIndex::prepare(text, &own, &cancel).unwrap();
    let mut point = index.start();
    let mut work = DocumentWork::frame(&cancel);
    for _ in 0..100 {
        index.step(&mut point, 4, &mut work).unwrap();
    }
    let offset = point.offset();
    let mut work = DocumentWork::bounded(&cancel, 63).unwrap();
    assert_eq!(index.step(&mut point, 4, &mut work), Err(Error::Work));
    assert_eq!(point.offset(), offset);
    assert_eq!(work.counters()[4], 0);
}
