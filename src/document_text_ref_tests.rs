use super::*;
use crate::{document_budget::ActorArena, document_model::Profile};
use std::{
    sync::atomic::Ordering,
    time::{Duration, Instant},
};
fn document(source: &str) -> Arc<Document> {
    let cancel = AtomicBool::new(false);
    let ctl = crate::document_obsidian::Control {
        cancel: &cancel,
        deadline: Instant::now() + Duration::from_secs(10),
    };
    let source = Arc::new(
        crate::document_source::DocumentSource::new(
            Arc::new(source.to_owned()),
            crate::input::SourceOrigin::Literal,
            1,
            2,
            &cancel,
        )
        .unwrap(),
    );
    let candidate =
        crate::document_parser::project(source, Profile::Extended, ActorArena::new(), &ctl)
            .unwrap();
    let depth = crate::document_validate::validate(&candidate, &ctl).unwrap();
    Arc::new(Document::from_validated(
        crate::document_validate::quarantine(candidate, depth, &ctl).unwrap(),
    ))
}
fn profile() -> IndexProfile {
    IndexProfile {
        projection_points: 2,
        unicode_points: 2,
        exceptions: 2,
    }
}
#[test]
fn rf01_real_title_unicode() {
    let doc = document("![ALT](x \"a\u{301} 👩‍💻 🇪🇸\")\n");
    let cancel = AtomicBool::new(false);
    let budget = ConsumerBudget::new(&doc).unwrap();
    let r = &doc.resources()[0];
    let index =
        TextRefIndex::prepare(doc.clone(), r.title_decoded(), &budget, &cancel, profile()).unwrap();
    assert_eq!(index.id(), r.title_decoded());
    let mut at = index.start();
    let mut work = DocumentWork::frame(&cancel);
    let mut glyphs = Vec::new();
    let mut widths = Vec::new();
    while let Some(u) = index.step(&mut at, 4, &mut work).unwrap() {
        let mut symbol = String::new();
        index
            .visit_projected(u.projected.clone(), &mut work, |s| {
                symbol.push_str(s);
                Ok(())
            })
            .unwrap();
        glyphs.push(symbol);
        widths.push(u.width);
    }
    assert_eq!(glyphs, ["a\u{301}", " ", "👩‍💻", " ", "🇪🇸"]);
    assert_eq!(widths, [1, 1, 2, 1, 2]);
    assert!(matches!(
        index.index.raw_anchor_with_work(&at, &mut work),
        Err(Error::Invalid)
    ));
    assert!(matches!(
        index
            .index
            .text
            .visit_coverage(0..index.len(), &mut work, |_| Ok(())),
        Err(Error::Invalid)
    ));
}
#[test]
fn rf02_control_and_enclosing() {
    let doc = document("![ALT](x \"A\u{1b}\u{85}&amp;Z\")\n");
    let cancel = AtomicBool::new(false);
    let budget = ConsumerBudget::new(&doc).unwrap();
    let id = doc.resources()[0].title_decoded();
    let index = TextRefIndex::prepare(doc.clone(), id, &budget, &cancel, profile()).unwrap();
    let mut work = DocumentWork::frame(&cancel);
    let mut out = String::new();
    index
        .visit_projected(0..index.len(), &mut work, |s| {
            out.push_str(s);
            Ok(())
        })
        .unwrap();
    assert_eq!(out, "A␛\\u{0085}&Z");
    let mut n = 0;
    index
        .visit_coverage(0..index.len(), &mut work, |c| {
            assert_eq!(c.text_ref, id);
            assert_eq!(c.relation, Relation::EnclosingToken);
            assert!(c.local.start < c.local.end);
            let origin = &doc.origins()[c.origin as usize];
            assert_eq!(c.source, origin.start()..origin.end());
            if c.local == (1..2) || c.local == (2..4) {
                assert!(c.control_projected);
            }
            n += 1;
            Ok(())
        })
        .unwrap();
    assert_eq!(
        n, 5,
        "five original semantic scalars; C1 projection is eight output scalars"
    );
}
#[test]
fn rf03_identity_cancel_retained() {
    let doc = document("![a](x \"title\")\n");
    let other = document("other");
    let cancel = AtomicBool::new(false);
    let budget = ConsumerBudget::new(&doc).unwrap();
    let foreign = ConsumerBudget::new(&other).unwrap();
    let id = doc.resources()[0].title_decoded();
    let before = budget.used();
    assert!(matches!(
        TextRefIndex::prepare(doc.clone(), id, &foreign, &cancel, profile()),
        Err(Error::Invalid)
    ));
    assert!(matches!(
        TextRefIndex::prepare(doc.clone(), u32::MAX, &budget, &cancel, profile()),
        Err(Error::Invalid)
    ));
    let index = TextRefIndex::prepare(doc.clone(), id, &budget, &cancel, profile()).unwrap();
    assert!(budget.used() > before);
    drop(index);
    assert_eq!(budget.used(), before);
    cancel.store(true, Ordering::Release);
    assert!(matches!(
        TextRefIndex::prepare(doc, id, &budget, &cancel, profile()),
        Err(Error::Cancelled)
    ));
    assert_eq!(budget.used(), before);
}
#[test]
fn rf04_whole_giant_tail() {
    let title = format!("a{} AFTER", "\u{301}".repeat(9000));
    let doc = document(&format!("![a](x \"{title}\")\n"));
    let cancel = AtomicBool::new(false);
    let budget = ConsumerBudget::new(&doc).unwrap();
    let index = TextRefIndex::prepare(
        doc.clone(),
        doc.resources()[0].title_decoded(),
        &budget,
        &cancel,
        profile(),
    )
    .unwrap();
    let mut at = index.start();
    let mut work = DocumentWork::frame(&cancel);
    let first = index.step(&mut at, 4096, &mut work).unwrap().unwrap();
    assert!(first.oversized);
    assert_eq!(first.projected, 0..18001);
    let mut rest = String::new();
    while let Some(u) = index.step(&mut at, 4096, &mut work).unwrap() {
        index
            .visit_projected(u.projected, &mut work, |s| {
                rest.push_str(s);
                Ok(())
            })
            .unwrap();
    }
    assert_eq!(rest, " AFTER");
}

#[test]
fn ru_ref_wrapper_preadmission_and_real_coverage() {
    let doc = document("![ALT](x \"a\u{301} 👩‍💻 🇪🇸\")\n");
    let cancel = AtomicBool::new(false);
    let budget = ConsumerBudget::new(&doc).unwrap();
    let id = doc.resources()[0].title_decoded();
    let index = TextRefIndex::prepare(doc, id, &budget, &cancel, profile()).unwrap();
    let before = std::mem::size_of::<Position>();
    let core = std::mem::size_of::<ResolvedUnit<'_>>();
    let whole = std::mem::size_of::<ResolvedRefUnit<'_>>();
    assert!(whole > core);
    for cap in [before + core - 1, before + whole - 1, before + whole] {
        let mut p = index.start();
        let saved = p.clone();
        let mut work = DocumentWork::bounded(&cancel, cap).unwrap();
        assert!(matches!(
            index.step_resolved(&mut p, 4, &mut work),
            Err(Error::Work)
        ));
        assert_eq!(p.position, saved.position);
        assert_eq!(p.previous, saved.previous);
        assert_eq!(
            (p.offset, p.line, p.column, p.cursor.cur_cursor()),
            (
                saved.offset,
                saved.line,
                saved.column,
                saved.cursor.cur_cursor()
            )
        );
    }
    let mut p = index.start();
    let mut glyphs = Vec::new();
    while let Some(token) = index
        .step_resolved(&mut p, 4, &mut DocumentWork::frame(&cancel))
        .unwrap()
    {
        let mut s = String::new();
        token
            .visit_projected(&mut DocumentWork::frame(&cancel), |v| {
                s.push_str(v);
                Ok(())
            })
            .unwrap();
        glyphs.push(s);
        token
            .visit_coverage(&mut DocumentWork::frame(&cancel), |c| {
                assert_eq!(c.text_ref, id);
                assert_eq!(c.relation, Relation::EnclosingToken);
                Ok(())
            })
            .unwrap();
    }
    assert_eq!(glyphs, ["a\u{301}", " ", "👩‍💻", " ", "🇪🇸"]);
}

#[test]
fn ru_ref_exact_total_and_cancel_before_commit() {
    let doc = document("![ALT](x \"abc\")\n");
    let cancel = Arc::new(AtomicBool::new(false));
    let budget = ConsumerBudget::new(&doc).unwrap();
    let id = doc.resources()[0].title_decoded();
    let index = TextRefIndex::prepare(doc, id, &budget, &cancel, profile()).unwrap();
    let mut expected = index.start();
    let original = expected.clone();
    let mut work = DocumentWork::frame(&cancel);
    index
        .step_resolved(&mut expected, 4, &mut work)
        .unwrap()
        .unwrap();
    let total = work.used();
    for cap in [total - 1, total, total + 1] {
        let mut at = original.clone();
        let mut work = DocumentWork::bounded(&cancel, cap).unwrap();
        let result = index.step_resolved(&mut at, 4, &mut work);
        if cap < total {
            assert!(matches!(result, Err(Error::Work)));
            assert_eq!(at.position, original.position);
            assert_eq!(at.cursor.cur_cursor(), original.cursor.cur_cursor());
        } else {
            assert!(result.unwrap().is_some());
            assert_eq!(at.position, expected.position);
            assert_eq!(work.used(), total);
        }
    }
    let signal = cancel.clone();
    RESOLVED_PRECOMMIT
        .with(|h| *h.borrow_mut() = Some(Box::new(move || signal.store(true, Ordering::Release))));
    let mut at = original.clone();
    let mut work = DocumentWork::frame(&cancel);
    assert!(matches!(
        index.step_resolved(&mut at, 4, &mut work),
        Err(Error::Cancelled)
    ));
    assert_eq!(at.position, original.position);
    assert_eq!(at.previous, original.previous);
    assert_eq!(at.cursor.cur_cursor(), original.cursor.cur_cursor());
}
