use super::*;
use crate::{
    document_inspector_fixtures as fx,
    document_search::{SearchOutcome, SearchRequest},
    search::Direction,
};
use std::sync::Mutex;
static VIEW_SERIAL: Mutex<()> = Mutex::new(());
fn stop() -> AtomicBool {
    AtomicBool::new(false)
}
fn shared(view: &DocumentView, domain: SearchDomain, q: &str) -> SharedHit {
    let request = SearchRequest::from_view(view, domain, q, 303, &stop()).unwrap();
    let hit = match request.find(Direction::Forward, None, &stop()) {
        SearchOutcome::DocumentFound(h) => h,
        _ => panic!("ready actual scoped hit"),
    };
    SharedHit::try_share(hit, &mut DocumentWork::preparation(&stop()))
        .ok()
        .unwrap()
}
fn inspect(d: &Arc<Document>, domain: SearchDomain, q: &str) -> (Inspection, ConsumerBudget) {
    let view = actual_view(d.clone());
    let budget = view.budget().clone();
    let h = shared(&view, domain, q);
    let i = Inspection::from_hit(&h, &view, &mut DocumentWork::preparation(&stop())).unwrap();
    drop(h);
    (i, budget)
}
fn whole(i: &Inspection) -> WholeAction {
    let mut s = i
        .summary_start(&mut DocumentWork::preparation(&stop()))
        .unwrap();
    for _ in 0..8 {
        if let Some(a) = s.complete() {
            return a.clone();
        }
        s = i
            .summary_advance(&s, &mut DocumentWork::preparation(&stop()))
            .unwrap();
    }
    panic!("bounded small fixture")
}
#[test]
fn in13_14_actual_hit_inspects_anchor_unit() {
    let d = fx::parsed("ab");
    let view = actual_view(d.clone());
    let h = shared(&view, SearchDomain::MainBody, "ab");
    assert_eq!(h.hit().highlight(), Some(0..2));
    let i = Inspection::from_hit(&h, &view, &mut DocumentWork::preparation(&stop())).unwrap();
    assert_eq!(i.projected_unit(), Some(0..1));
    assert_eq!(whole(&i), WholeAction::Exact(0..1));
    assert_eq!(h.hit().highlight(), Some(0..2));
    let (i, _) = inspect(
        &fx::parsed("\u{001b}\u{001b}"),
        SearchDomain::MainBody,
        "␛␛",
    );
    assert_eq!(i.projected_unit(), Some(0..3));
    assert_eq!(whole(&i), WholeAction::ControlScalar(0..1));
}
#[test]
fn ir02_real_65_component_prefix_is_not_whole_exact() {
    let d = fx::cluster65(true);
    let raw = d.source().text();
    let (i, _) = inspect(&d, SearchDomain::MainBody, raw);
    let page = i
        .page(None, &mut DocumentWork::preparation(&stop()))
        .unwrap();
    assert_eq!(page.values().len(), 64);
    assert!(page.next().is_some());
    assert!(
        page.values()
            .iter()
            .all(|c| matches!(c.action(), ComponentAction::Exact(_)))
    );
    let s = i
        .summary_start(&mut DocumentWork::preparation(&stop()))
        .unwrap();
    let first = i
        .summary_advance(&s, &mut DocumentWork::preparation(&stop()))
        .unwrap();
    assert!(first.complete().is_none());
    assert!(matches!(
        i.summary_advance(&first, &mut DocumentWork::bounded(&stop(), 0).unwrap()),
        Err(Error::Work)
    ));
    assert!(first.complete().is_none());
    assert_eq!(page.values().len(), 64);
    let second = i
        .summary_advance(&first, &mut DocumentWork::preparation(&stop()))
        .unwrap();
    assert_eq!(second.complete(), Some(&WholeAction::Disjoint));
    let suffix = i
        .page(page.next(), &mut DocumentWork::preparation(&stop()))
        .unwrap();
    assert_eq!(suffix.values().len(), 1);
    assert!(
        matches!(suffix.values()[0].action(),ComponentAction::Enclosing{range,..}if *range==(127..129))
    );
    let (exact, _) = inspect(&fx::cluster65(false), SearchDomain::MainBody, raw);
    assert_eq!(whole(&exact), WholeAction::Exact(0..129));
}
#[test]
fn links_multirun_and_explicit_chooser() {
    let (i, _) = inspect(
        &fx::parsed("[a](one)[\u{0301}](two)"),
        SearchDomain::MainBody,
        "a\u{0301}",
    );
    assert_eq!(whole(&i), WholeAction::Disjoint);
    let p = i
        .choices(None, &mut DocumentWork::preparation(&stop()))
        .unwrap();
    assert_eq!(p.links(), &[0, 1]);
    let c = i
        .choose(&p, 1, &mut DocumentWork::preparation(&stop()))
        .unwrap();
    let selected = i
        .confirm(&p, &c, &mut DocumentWork::preparation(&stop()))
        .unwrap();
    assert_eq!(selected.id(), 1);
    assert_eq!(
        i.sample_link(
            &selected,
            LinkField::Target,
            &mut DocumentWork::preparation(&stop())
        )
        .unwrap()
        .unwrap()
        .text(),
        "two"
    );
    let p2 = i
        .choices(None, &mut DocumentWork::preparation(&stop()))
        .unwrap();
    assert!(matches!(
        i.confirm(&p2, &c, &mut DocumentWork::preparation(&stop())),
        Err(Error::Invalid)
    ));
}
#[test]
fn entity_and_resource_real_owner() {
    let (i, _) = inspect(&fx::parsed("A&amp;B"), SearchDomain::MainBody, "&");
    assert!(matches!(whole(&i),WholeAction::Enclosing{range,..}if range==(1..6)));
    let d = fx::shared_titles();
    let v = actual_view(d.clone());
    let h = shared(&v, SearchDomain::ResourceTitle, "same");
    let i = Inspection::from_hit(&h, &v, &mut DocumentWork::preparation(&stop())).unwrap();
    assert_eq!(i.resource_id(), Some(0));
    assert_eq!(h.hit().resource_id(), Some(0));
    assert_eq!(
        i.sample_resource(
            ResourceField::Input,
            &mut DocumentWork::preparation(&stop())
        )
        .unwrap()
        .unwrap()
        .text(),
        "one"
    );
}
#[test]
fn shared_rollback_lease_lifetime_and_charge() {
    let d = fx::parsed("ab");
    let b = ConsumerBudget::new(&d).unwrap();
    let baseline = b.used();
    let r =
        SearchRequest::prepare(d.clone(), &b, SearchDomain::MainBody, "ab", 303, &stop()).unwrap();
    let h = match r.find(Direction::Forward, None, &stop()) {
        SearchOutcome::DocumentFound(h) => h,
        _ => panic!(),
    };
    let held = b.used();
    let pending = match SharedHit::try_share(h, &mut DocumentWork::bounded(&stop(), 0).unwrap()) {
        Err(p) => p,
        Ok(_) => panic!(),
    };
    assert_eq!(b.used(), held);
    assert_eq!(pending.hit.highlight(), Some(0..2));
    let h = SharedHit::try_share(pending.hit, &mut DocumentWork::preparation(&stop()))
        .ok()
        .unwrap();
    let lease = h.lease(&mut DocumentWork::preparation(&stop())).unwrap();
    drop(h);
    assert_eq!(
        lease
            .initial_unit(&mut DocumentWork::preparation(&stop()))
            .unwrap(),
        0..1
    );
    drop(lease);
    drop(r);
    assert_eq!(b.used(), baseline);
}
fn actual_view(d: Arc<Document>) -> DocumentView {
    use crate::preparation_owner::{Outcome, PreparationAuthority, PreparedCompletion};
    use preparation::{
        self, InspectionLibraryAuthority, InspectionLibraryInput, InspectionLibraryOutput,
    };
    let _serial = VIEW_SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut authority: InspectionLibraryAuthority =
        PreparationAuthority::in_arena(d.actor_arena()).unwrap();
    let geometry = crate::document_layout::Geometry {
        x: 0,
        y: 0,
        width: 80,
        height: 24,
        left: 0,
    };
    let descriptor =
        DocumentView::initial_shared_inspectable_input(d, geometry, &authority).unwrap();
    let input =
        InspectionLibraryInput::viewport(descriptor, &mut DocumentWork::preparation(&stop()))
            .unwrap();
    let binding = input.binding();
    if let Err(pending) = preparation::start(input, &mut authority) {
        panic!("start {:?}", pending.cause)
    }
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !authority.is_finished() && std::time::Instant::now() < deadline {
        std::thread::yield_now()
    }
    let Some(PreparedCompletion::Document(
        Outcome::Ready(InspectionLibraryOutput::Viewport(p)),
        receipt,
    )) = authority.take_validated(binding, false).unwrap()
    else {
        panic!("real scoped viewport")
    };
    assert!(receipt.joined);
    DocumentView::from_prepared(p).unwrap()
}

#[test]
fn source_metadata_survives_view_and_hit_drop() {
    let d = fx::parsed("ab");
    let v = actual_view(d.clone());
    let budget = v.budget().clone();
    let binding = v.render_binding();
    let metadata = Arc::downgrade(v.inspect_source_metadata());
    let context = Arc::downgrade(v.inspect_source_context());
    let h = shared(&v, SearchDomain::MainBody, "ab");
    let i = Inspection::from_hit(&h, &v, &mut DocumentWork::preparation(&stop())).unwrap();
    assert_ne!(h.hit().query_revision(), binding.revision);
    assert_eq!(h.hit().query_revision(), 303);
    let p = i
        .page(None, &mut DocumentWork::preparation(&stop()))
        .unwrap();
    let a = i
        .source_component(&p, 0, &mut DocumentWork::preparation(&stop()))
        .unwrap();
    assert_eq!(a.binding(), binding);
    drop(v);
    drop(h);
    assert!(metadata.upgrade().is_some());
    assert!(context.upgrade().is_some());
    assert_eq!(a.binding(), binding);
    drop(i);
    assert!(metadata.upgrade().is_some());
    drop(p);
    assert!(metadata.upgrade().is_some());
    drop(a);
    assert!(metadata.upgrade().is_none());
    assert!(context.upgrade().is_none());
    assert!(
        budget.used() < 1000,
        "only ConsumerBudget itself retained, used={}",
        budget.used()
    );
}
#[test]
fn no_query_real_view_anchor_variants_and_logical_cursor() {
    let v = actual_view(fx::parsed(
        "# A\n\n![fox](one \"title\")\n\n---\n\n> [!note] title\n> text\n",
    ));
    let window = v.layout().text().unwrap();
    let mut seen = [false; 4];
    for cell in 0..window.glyphs().len() {
        let Some(anchor) = window
            .cell_anchor(cell, &mut DocumentWork::preparation(&stop()))
            .unwrap()
        else {
            continue;
        };
        let category = match anchor {
            PresentationAnchor::Domain(_) => 0,
            PresentationAnchor::Ref(_) => 1,
            PresentationAnchor::Role(_) => 2,
            PresentationAnchor::ResourceAlt(_) => 3,
            PresentationAnchor::Table(_) => continue,
        };
        let i = Inspection::from_view(
            &v,
            Some(cell),
            None,
            &stop(),
            &mut DocumentWork::preparation(&stop()),
        )
        .unwrap();
        let p = i
            .page(None, &mut DocumentWork::preparation(&stop()))
            .unwrap();
        assert!(!p.values().is_empty());
        let _ = whole(&i);
        seen[category] = true;
    }
    assert_eq!(seen, [true; 4]);
    let d = v.inspect_document();
    let run = d
        .runs()
        .iter()
        .find(|r| r.kind() == 0 && d.domains()[r.domain() as usize].kind() == 0)
        .unwrap()
        .id();
    let selected = State::prepared_visual(v.navigator().index(), run, 0, 0, &stop()).unwrap();
    let i = Inspection::from_cursor(
        &v,
        &selected,
        None,
        &stop(),
        &mut DocumentWork::preparation(&stop()),
    )
    .unwrap();
    assert!(i.projected_unit().is_some());
    let role = State::prepared_role(v.navigator().index(), 1, &stop()).unwrap();
    let i = Inspection::from_cursor(
        &v,
        &role,
        None,
        &stop(),
        &mut DocumentWork::preparation(&stop()),
    )
    .unwrap();
    assert_eq!(whole(&i), WholeAction::NoLiteral);
    let other = actual_view(d.clone());
    assert!(matches!(
        Inspection::from_cursor(
            &other,
            &selected,
            None,
            &stop(),
            &mut DocumentWork::preparation(&stop())
        ),
        Err(Error::Invalid)
    ));
}
#[test]
fn no_query_actual_table_resume_selected_cell() {
    let v = actual_view(fx::parsed("| a | b |\n| :- | -: |\n| c | d |\n"));
    let w = v.layout().text().unwrap();
    let anchor = w.presentation_row(0).unwrap();
    assert!(matches!(anchor, PresentationAnchor::Table(_)));
    let i = Inspection::from_view(
        &v,
        None,
        None,
        &stop(),
        &mut DocumentWork::preparation(&stop()),
    )
    .unwrap();
    assert!(matches!(i.anchor(), Some(PresentationAnchor::Table(_))));
    assert_eq!(whole(&i), WholeAction::Exact(2..3));
}
#[test]
fn incremental_cursor_matches_d1_and_preserves_failure() {
    let d = fx::cluster65(true);
    let v = actual_view(d.clone());
    let h = shared(&v, SearchDomain::MainBody, d.source().text());
    let lease = h.lease(&mut DocumentWork::preparation(&stop())).unwrap();
    let ProviderRef::Runs(index) = lease.provider().unwrap() else {
        panic!()
    };
    let mut expected = Vec::new();
    index
        .text()
        .visit_coverage(0..129, &mut DocumentWork::preparation(&stop()), |c| {
            expected.push(c);
            Ok(())
        })
        .unwrap();
    let mut cursor = CoverageCursor::runs(
        index.text(),
        0..129,
        &mut DocumentWork::preparation(&stop()),
    )
    .unwrap();
    let mut actual = Vec::new();
    loop {
        assert!(matches!(
            cursor.next_runs(
                index.text(),
                &mut DocumentWork::bounded(&stop(), 0).unwrap()
            ),
            Err(Error::Work)
        ));
        match cursor
            .next_runs(index.text(), &mut DocumentWork::preparation(&stop()))
            .unwrap()
        {
            CoverageStep::Component(CoverageComponent::Run(c)) => actual.push(c),
            CoverageStep::End(proof) => {
                assert!(proof.is_for(&cursor));
                break;
            }
            _ => panic!(),
        }
    }
    assert_eq!(actual, expected);
    assert_eq!(actual.len(), 65);
    let cancel = AtomicBool::new(true);
    assert!(matches!(
        cursor.try_fork(&mut DocumentWork::preparation(&cancel)),
        Err(Error::Cancelled)
    ));
    let whole = index.text();
    let mut short =
        CoverageCursor::runs(whole, 127..129, &mut DocumentWork::preparation(&stop())).unwrap();
    loop {
        if let CoverageStep::End(proof) = short
            .next_runs(whole, &mut DocumentWork::preparation(&stop()))
            .unwrap()
        {
            assert!(!proof.is_for(&cursor));
            break;
        }
    }
}
#[test]
fn link65_pages_and_long_projected_samples() {
    let raw = (0..65)
        .map(|i| format!("[{}](t{i})", if i == 0 { "a" } else { "\u{0301}" }))
        .collect::<String>();
    let (i, _) = inspect(
        &fx::parsed(&raw),
        SearchDomain::MainBody,
        &format!("a{}", "\u{0301}".repeat(64)),
    );
    let p = i
        .choices(None, &mut DocumentWork::preparation(&stop()))
        .unwrap();
    assert_eq!(p.links(), &(0..64).collect::<Vec<_>>());
    let next = i
        .choices(p.next(), &mut DocumentWork::preparation(&stop()))
        .unwrap();
    assert_eq!(next.links(), &[64]);
    assert!(matches!(
        i.choices(p.next(), &mut DocumentWork::bounded(&stop(), 0).unwrap()),
        Err(Error::Work)
    ));
    assert_eq!(p.links().len(), 64);
    let title = format!("{}\u{001b}\u{009b}", "x".repeat(4100));
    let source = format!("![fox](one \"{title}\")");
    let (i, _) = inspect(&fx::parsed(&source), SearchDomain::ResourceTitle, "x");
    let first = i
        .sample_resource(
            ResourceField::Title,
            &mut DocumentWork::preparation(&stop()),
        )
        .unwrap()
        .unwrap();
    assert_eq!(first.text().len(), 4096);
    let second = i
        .sample_next(
            first.next().unwrap(),
            &mut DocumentWork::preparation(&stop()),
        )
        .unwrap();
    assert_eq!(second.text(), "xxxx␛\\u{009b}");
    assert!(second.next().is_none());
}
#[test]
fn actual_scalar_vs_merged_controls_and_second_cursor() {
    let d = fx::parsed("\u{001b}\u{001b}");
    let v = actual_view(d.clone());
    let h = shared(&v, SearchDomain::MainBody, "␛␛");
    let i = Inspection::from_hit(&h, &v, &mut DocumentWork::preparation(&stop())).unwrap();
    let mut merged = None;
    h.hit()
        .visit_coverage(&mut DocumentWork::preparation(&stop()), |c| {
            if let crate::document_search::HitCoverage::Run(c) = c {
                merged = Some(c)
            }
            Ok(())
        })
        .unwrap();
    let c = merged.unwrap();
    assert_eq!(c.source, Some(0..2));
    assert_eq!(
        ComponentAction::from_run(&i.scope, &c, &mut DocumentWork::preparation(&stop())).unwrap(),
        ComponentAction::ControlProjection {
            range: 0..2,
            single_scalar: false
        }
    );
    let s = State::prepared_visual(v.navigator().index(), 0, 1, 0, &stop()).unwrap();
    let second = Inspection::from_cursor(
        &v,
        &s,
        None,
        &stop(),
        &mut DocumentWork::preparation(&stop()),
    )
    .unwrap();
    assert_eq!(second.projected_unit(), Some(3..6));
    assert_eq!(whole(&second), WholeAction::ControlScalar(1..2));
}
#[test]
fn share_admission_archeader_handle_and_lease_quota_preserve_hit() {
    use crate::document_consumer_budget::CONSUMER_LIMIT;
    for remaining in [0usize, 48] {
        let d = fx::parsed("ab");
        let b = ConsumerBudget::new(&d).unwrap();
        let r = SearchRequest::prepare(d.clone(), &b, SearchDomain::MainBody, "ab", 303, &stop())
            .unwrap();
        let h = match r.find(Direction::Forward, None, &stop()) {
            SearchOutcome::DocumentFound(h) => h,
            _ => panic!(),
        };
        let initial = b.used();
        let filler = b.reserve(CONSUMER_LIMIT - initial - remaining).unwrap();
        let before = b.used();
        let pending = SharedHit::try_share(h, &mut DocumentWork::preparation(&stop()))
            .err()
            .expect("actual quota failure");
        assert_eq!(pending.cause, Error::Budget);
        assert_eq!(b.used(), before);
        assert_eq!(
            pending.hit.anchor(),
            crate::document_search::DocumentAnchor::Run {
                run: 0,
                raw_byte: 0,
                projection_part: 0
            }
        );
        drop(filler);
        let h = SharedHit::try_share(pending.hit, &mut DocumentWork::preparation(&stop()))
            .ok()
            .unwrap();
        let filler = b.reserve(CONSUMER_LIMIT - b.used()).unwrap();
        let before = b.used();
        assert!(matches!(
            h.lease(&mut DocumentWork::preparation(&stop())),
            Err(Error::Budget)
        ));
        assert_eq!(b.used(), before);
        assert_eq!(h.hit().matched(), 0..2);
        drop(filler);
    }
}
#[test]
fn resource_shared_ref_ids_second_owner_and_stale_page() {
    let d = fx::shared_titles();
    let v = actual_view(d.clone());
    let r =
        SearchRequest::from_view(&v, SearchDomain::ResourceTitle, "same", 303, &stop()).unwrap();
    let r = crate::document_search::shared_query::SharedQuery::new(
        r,
        &mut DocumentWork::preparation(&stop()),
    )
    .unwrap();
    let first = match r.find(Direction::Forward, None, &stop()) {
        SearchOutcome::DocumentFound(h) => h,
        _ => panic!(),
    };
    let first_anchor = first.anchor();
    let cursor = r
        .cursor(first.cursor(), &mut DocumentWork::preparation(&stop()))
        .unwrap();
    drop(first);
    let second = match r.find(Direction::Forward, Some(&cursor), &stop()) {
        SearchOutcome::DocumentFound(h) => h,
        _ => panic!(),
    };
    assert_eq!(first_anchor, second.anchor());
    let h = SharedHit::try_share(second, &mut DocumentWork::preparation(&stop()))
        .ok()
        .unwrap();
    let i = Inspection::from_hit(&h, &v, &mut DocumentWork::preparation(&stop())).unwrap();
    assert_eq!(i.resource_id(), Some(1));
    assert!(matches!(
        i.ref_owner(),
        Some(RefOwner::Resource {
            resource: 1,
            field: RefField::Title,
            ..
        })
    ));
    assert_eq!(
        i.sample_resource(
            ResourceField::Input,
            &mut DocumentWork::preparation(&stop())
        )
        .unwrap()
        .unwrap()
        .text(),
        "two"
    );
    let other = Inspection::from_hit(&h, &v, &mut DocumentWork::preparation(&stop())).unwrap();
    let p = i
        .page(None, &mut DocumentWork::preparation(&stop()))
        .unwrap();
    assert!(matches!(
        other.source_component(&p, 0, &mut DocumentWork::preparation(&stop())),
        Err(Error::Invalid)
    ));
    let foreign = actual_view(d.clone());
    assert!(matches!(
        Inspection::from_hit(&h, &foreign, &mut DocumentWork::preparation(&stop())),
        Err(Error::Invalid)
    ));
}
#[test]
fn decoded_identity_styles_origins_and_completed_disjoint_cursor() {
    let d = fx::parsed("a*\u{0301}*");
    let (i, _) = inspect(&d, SearchDomain::MainBody, "a\u{0301}");
    let p = i
        .page(None, &mut DocumentWork::preparation(&stop()))
        .unwrap();
    assert_eq!(p.values().len(), 2);
    for index in 0..2 {
        let (run, text) = i
            .origin_records(&p, index, &mut DocumentWork::preparation(&stop()))
            .unwrap();
        assert!(run.is_some());
        assert!(text.is_some());
    }
    let mut summary = i
        .summary_start(&mut DocumentWork::preparation(&stop()))
        .unwrap();
    summary = i
        .summary_advance(&summary, &mut DocumentWork::preparation(&stop()))
        .unwrap();
    assert_eq!(summary.complete(), Some(&WholeAction::Disjoint));
    let cursor = summary
        .disjoint_cursor(&mut DocumentWork::preparation(&stop()))
        .unwrap();
    let replay = i
        .page(Some(&cursor), &mut DocumentWork::preparation(&stop()))
        .unwrap();
    assert_eq!(replay.values().len(), 2);
    let mixed = fx::cluster65(true);
    let (i, _) = inspect(&mixed, SearchDomain::MainBody, mixed.source().text());
    let a = i
        .page(None, &mut DocumentWork::preparation(&stop()))
        .unwrap();
    let b = i
        .page(a.next(), &mut DocumentWork::preparation(&stop()))
        .unwrap();
    assert!(matches!(
        b.values()[0].action(),
        ComponentAction::Enclosing { .. }
    ));
}
#[test]
fn terminal_containers_charge_entire_layout_and_last_drop() {
    let (i, b) = inspect(&fx::parsed("a"), SearchDomain::MainBody, "a");
    let base = b.used();
    let p = i
        .page(None, &mut DocumentWork::preparation(&stop()))
        .unwrap();
    assert_eq!(p.values().len(), 1);
    assert!(p.next().is_none());
    assert_eq!(
        b.used() - base,
        size_of::<InspectionPage>() + p.values.capacity() * size_of::<Association>()
    );
    drop(p);
    assert_eq!(b.used(), base);
    let mut end = i
        .cursor(
            CursorKind::Associations,
            &mut DocumentWork::preparation(&stop()),
        )
        .unwrap();
    assert!(matches!(
        end.next(&mut DocumentWork::preparation(&stop())).unwrap(),
        Next::Component(_)
    ));
    assert!(matches!(
        end.next(&mut DocumentWork::preparation(&stop())).unwrap(),
        Next::End
    ));
    let before = b.used();
    let empty = i
        .page(Some(&end), &mut DocumentWork::preparation(&stop()))
        .unwrap();
    assert!(empty.values().is_empty());
    assert!(empty.next().is_none());
    assert_eq!(
        b.used() - before,
        size_of::<InspectionPage>() + empty.values.capacity() * size_of::<Association>()
    );
    drop(empty);
    assert_eq!(b.used(), before);
    drop(end);
    assert_eq!(b.used(), base);
    let links = i
        .choices(None, &mut DocumentWork::preparation(&stop()))
        .unwrap();
    assert!(links.links().is_empty());
    assert!(links.next().is_none());
    assert_eq!(
        b.used() - base,
        size_of::<LinkPage>() + links.links.capacity() * size_of::<u32>() + 2 * size_of::<usize>()
    );
    drop(links);
    assert_eq!(b.used(), base);
    let (i, b) = inspect(
        &fx::parsed("![fox](one \"title\")"),
        SearchDomain::ResourceTitle,
        "t",
    );
    let base = b.used();
    let sample = i
        .sample_resource(
            ResourceField::Input,
            &mut DocumentWork::preparation(&stop()),
        )
        .unwrap()
        .unwrap();
    assert_eq!(sample.text(), "one");
    assert!(sample.next().is_none());
    assert_eq!(
        b.used() - base,
        size_of::<FieldSample>() + sample.bytes.capacity()
    );
    drop(sample);
    assert_eq!(b.used(), base);
}
#[test]
fn no_query_resource_cursor_inert_descriptor_and_absent_vs_empty() {
    for (d, title) in [
        (fx::parsed("![fox](one)"), None),
        (fx::parsed("![fox](one \"\")"), None),
        (fx::empty_title(), Some("")),
    ] {
        let v = actual_view(d.clone());
        let r = d.resources()[0];
        let state =
            State::prepared_visual(v.navigator().index(), r.owner_run(), 0, 0, &stop()).unwrap();
        let i = Inspection::from_cursor(
            &v,
            &state,
            None,
            &stop(),
            &mut DocumentWork::preparation(&stop()),
        )
        .unwrap();
        assert_eq!(i.resource_id(), Some(0));
        assert_eq!(whole(&i), WholeAction::NoLiteral);
        let sample = i
            .sample_resource(
                ResourceField::Title,
                &mut DocumentWork::preparation(&stop()),
            )
            .unwrap();
        assert_eq!(sample.as_ref().map(|s| s.text()), title);
        assert!(
            i.enclosing_resource(&mut DocumentWork::preparation(&stop()))
                .unwrap()
                .is_some()
        );
        assert_eq!(i.source().origin(), &crate::input::SourceOrigin::Literal);
    }
}
#[test]
fn cursor_capability_and_whole_source_are_closed() {
    let (i, _) = inspect(&fx::parsed("[a](one)"), SearchDomain::MainBody, "a");
    let wrong = i
        .cursor(CursorKind::Links, &mut DocumentWork::preparation(&stop()))
        .unwrap();
    assert!(matches!(
        i.page(Some(&wrong), &mut DocumentWork::preparation(&stop())),
        Err(Error::Invalid)
    ));
    let mut summary = i
        .summary_start(&mut DocumentWork::preparation(&stop()))
        .unwrap();
    assert!(matches!(
        i.whole_source(&summary, &mut DocumentWork::preparation(&stop())),
        Err(Error::Work)
    ));
    summary = i
        .summary_advance(&summary, &mut DocumentWork::preparation(&stop()))
        .unwrap();
    let action = i
        .whole_source(&summary, &mut DocumentWork::preparation(&stop()))
        .unwrap()
        .unwrap();
    assert_eq!(action.relation(), &ComponentAction::Exact(1..2));
}
