use super::*;
use crate::{
    document_budget::ActorArena, document_model::Profile, document_obsidian::Control,
    input::SourceOrigin,
};
use std::{
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

const ENTITY: &str = include_str!("../tests/fixtures/document-b3-search/entity.md");
const IMAGE: &str = include_str!("../tests/fixtures/document-b3-search/image.md");
const NESTED: &str = include_str!("../tests/fixtures/document-b3-search/nested.md");
const ORACLE: &str = include_str!("../tests/fixtures/document-b3-search/PREOUTPUT.txt");

fn parsed(raw: &str) -> Arc<Document> {
    let cancel = AtomicBool::new(false);
    let source = Arc::new(
        DocumentSource::new(
            Arc::new(raw.to_owned()),
            SourceOrigin::Literal,
            71,
            93,
            &cancel,
        )
        .unwrap(),
    );
    let control = Control {
        cancel: &cancel,
        deadline: Instant::now() + Duration::from_secs(10),
    };
    let candidate =
        crate::document_parser::project(source, Profile::Extended, ActorArena::new(), &control)
            .unwrap();
    let depth = candidate.max_depth;
    Arc::new(Document::from_validated(
        crate::document_validate::quarantine(candidate, depth, &control).unwrap(),
    ))
}
fn request(
    document: &Arc<Document>,
    budget: &ConsumerBudget,
    domain: SearchDomain,
    query: &str,
) -> SearchRequest {
    SearchRequest::prepare(
        document.clone(),
        budget,
        domain,
        query,
        11,
        &AtomicBool::new(false),
    )
    .unwrap()
}
fn found(
    request: &SearchRequest,
    direction: Direction,
    after: Option<&SearchCursor>,
) -> DocumentHit {
    match request.find(direction, after, &AtomicBool::new(false)) {
        SearchOutcome::DocumentFound(hit) => hit,
        SearchOutcome::DocumentFoundPending(_, error) => panic!("unexpected pending: {error:?}"),
        SearchOutcome::Failed(error) => panic!("search failed: {error:?}"),
        _ => panic!("expected document match"),
    }
}
fn source_found(
    request: &SearchRequest,
    direction: Direction,
    after: Option<&SearchCursor>,
) -> SourceHit {
    match request.find(direction, after, &AtomicBool::new(false)) {
        SearchOutcome::SourceMatched(hit) => hit,
        _ => panic!("expected original Source match"),
    }
}
fn no_match(document: &Arc<Document>, budget: &ConsumerBudget, domain: SearchDomain, query: &str) {
    assert!(matches!(
        request(document, budget, domain, query).find(
            Direction::Forward,
            None,
            &AtomicBool::new(false)
        ),
        SearchOutcome::NoMatch
    ));
}

/// Run this model-only oracle before matcher tests. It does not call SearchRequest.
/// The persisted records come from the real parser/validator, never from matcher outputs.
#[test]
fn b3_model_oracle_before_search() {
    assert!(ORACLE.contains("before matcher outputs"));
    let image = parsed(IMAGE);
    assert_eq!(&IMAGE[31..37], "secret");
    assert_eq!(image.resources().len(), 1);
    let resource = image.resources()[0];
    assert_eq!(image.text(resource.title_decoded()).unwrap(), "caption");
    let alt = image.domains()[resource.alt_domain() as usize];
    assert_eq!(alt.kind(), 1);
    assert_eq!(
        image
            .text(image.runs()[alt.first_run() as usize].text())
            .unwrap(),
        "fox"
    );
    let entity = parsed(ENTITY);
    assert_eq!(ENTITY.get(1..6), Some("&amp;"));
    let nested = parsed(NESTED);
    assert_eq!(
        nested
            .blocks()
            .iter()
            .filter(|b| b.kind() == 4 && b.aux2() == 2)
            .count(),
        2
    );
    assert_eq!(nested.blocks().iter().filter(|b| b.kind() == 2).count(), 1);
    let mut records = String::from("MODEL ONLY: real parser/validator before matcher runtime\n");
    use std::fmt::Write;
    for (name, document) in [("entity", entity), ("image", image), ("nested", nested)] {
        writeln!(
            records,
            "{name} sourceSHA={:02x?}",
            document.source().hash()
        )
        .unwrap();
        for block in document.blocks() {
            writeln!(records, "block {:?}", block.words()).unwrap();
        }
        for domain in document.domains() {
            writeln!(records, "domain {:?}", domain.words()).unwrap();
        }
        for item in document.items() {
            writeln!(records, "item {:?}", item.words()).unwrap();
        }
        for run in document.runs() {
            writeln!(records, "run {:?}", run.words()).unwrap();
        }
        for origin in document.origins() {
            writeln!(records, "origin {:?}", origin.words()).unwrap();
        }
        for resource in document.resources() {
            writeln!(records, "resource {:?}", resource.words()).unwrap();
        }
    }
    assert_eq!(
        records,
        include_str!("../tests/fixtures/document-b3-search/MODEL-ORACLE.txt")
    );
}
#[test]
fn b3_entity_and_original_provenance() {
    let document = parsed(ENTITY);
    let budget = ConsumerBudget::new(&document).unwrap();
    let hit = found(
        &request(&document, &budget, SearchDomain::MainBody, "A&B"),
        Direction::Forward,
        None,
    );
    assert_eq!(hit.matched(), 0..3);
    let mut coverage = Vec::new();
    hit.visit_coverage(
        &mut DocumentWork::preparation(&AtomicBool::new(false)),
        |c| {
            if let HitCoverage::Run(c) = c {
                coverage.push(c);
            }
            Ok(())
        },
    )
    .unwrap();
    let amp = coverage
        .iter()
        .find(|c| c.relation == crate::document_text::Relation::EnclosingToken)
        .unwrap();
    assert_eq!(amp.source, Some(1..6));
    no_match(&document, &budget, SearchDomain::SourceOriginal, "A&B");
    let source = source_found(
        &request(&document, &budget, SearchDomain::SourceOriginal, "&amp;"),
        Direction::Forward,
        None,
    );
    assert_eq!(source.range(), 1..6);
    assert_eq!(source.source().generation(), 71);
    assert_eq!(source.source().request(), 93);
    assert_eq!(source.query_revision(), 11);
    assert!(Arc::ptr_eq(
        &source.source().original_handle(),
        &document.source().original_handle()
    ));
}
#[test]
fn b3_separate_domains_resource_and_source_without_run() {
    let document = parsed(IMAGE);
    let budget = ConsumerBudget::new(&document).unwrap();
    for query in ["fox", "secret", "caption", "https"] {
        no_match(&document, &budget, SearchDomain::MainBody, query);
    }
    let alt = found(
        &request(&document, &budget, SearchDomain::ResourceAlt, "fox"),
        Direction::Forward,
        None,
    );
    assert_eq!(alt.matched(), 0..3);
    assert!(alt.domain_id().is_some());
    assert!(matches!(alt.anchor(), DocumentAnchor::Run { .. }));
    let title = found(
        &request(&document, &budget, SearchDomain::ResourceTitle, "caption"),
        Direction::Forward,
        None,
    );
    assert_eq!(title.matched(), 0..7);
    assert!(title.domain_id().is_none());
    assert!(matches!(title.anchor(), DocumentAnchor::TextRef { .. }));
    title
        .visit_coverage(
            &mut DocumentWork::preparation(&AtomicBool::new(false)),
            |c| {
                assert!(matches!(c, HitCoverage::TextRef(_)));
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(
        source_found(
            &request(&document, &budget, SearchDomain::SourceOriginal, "secret"),
            Direction::Forward,
            None
        )
        .range(),
        31..37
    );
}
#[test]
fn b3_overlap_both_directions_one_wrap_and_query_identity() {
    for domain in [SearchDomain::MainBody, SearchDomain::SourceOriginal] {
        let document = parsed("aaa");
        let budget = ConsumerBudget::new(&document).unwrap();
        let query = request(&document, &budget, domain, "aa");
        let mut current: Option<SearchOutcome> = None;
        for expected in [0, 1, 0, 1] {
            let result = query.find(
                Direction::Forward,
                current.as_ref().and_then(SearchOutcome::cursor),
                &AtomicBool::new(false),
            );
            let range = match &result {
                SearchOutcome::DocumentFound(hit) => hit.matched(),
                SearchOutcome::SourceMatched(hit) => hit.range(),
                _ => panic!("missing overlapping forward match"),
            };
            assert_eq!(range, expected..expected + 2);
            current = Some(result);
        }
        current = None;
        for expected in [1, 0, 1, 0] {
            let result = query.find(
                Direction::Backward,
                current.as_ref().and_then(SearchOutcome::cursor),
                &AtomicBool::new(false),
            );
            let range = match &result {
                SearchOutcome::DocumentFound(hit) => hit.matched(),
                SearchOutcome::SourceMatched(hit) => hit.range(),
                _ => panic!("missing overlapping backward match"),
            };
            assert_eq!(range, expected..expected + 2);
            current = Some(result);
        }
        let same_spelling_new_query = request(&document, &budget, domain, "aa");
        assert!(matches!(
            same_spelling_new_query.find(
                Direction::Forward,
                current.as_ref().and_then(SearchOutcome::cursor),
                &AtomicBool::new(false)
            ),
            SearchOutcome::Failed(Error::Invalid)
        ));
    }
}
#[test]
fn b3_source_markup_and_scalar_boundaries() {
    let document = parsed("**fox**");
    let budget = ConsumerBudget::new(&document).unwrap();
    no_match(&document, &budget, SearchDomain::MainBody, "**");
    let query = request(&document, &budget, SearchDomain::SourceOriginal, "**");
    let first = source_found(&query, Direction::Forward, None);
    assert_eq!(first.range(), 0..2);
    assert_eq!(
        source_found(&query, Direction::Forward, Some(first.cursor())).range(),
        5..7
    );
    let document = parsed(&format!("{}éé", "x".repeat(4095)));
    let budget = ConsumerBudget::new(&document).unwrap();
    let query = request(&document, &budget, SearchDomain::SourceOriginal, "é");
    let first = source_found(&query, Direction::Forward, None);
    assert_eq!(first.range(), 4095..4097);
    assert_eq!(
        source_found(&query, Direction::Forward, Some(first.cursor())).range(),
        4097..4099
    );
}
#[test]
fn b3_multirun_glyph_and_chunk_crossing() {
    let document = parsed("a*\u{301}*");
    let budget = ConsumerBudget::new(&document).unwrap();
    assert_eq!(document.runs().len(), 2);
    assert_ne!(document.runs()[0].styles(), document.runs()[1].styles());
    let hit = found(
        &request(&document, &budget, SearchDomain::MainBody, "\u{301}"),
        Direction::Forward,
        None,
    );
    assert_eq!(hit.matched(), 1..3);
    assert_eq!(hit.highlight(), Some(0..3));
    let mut runs = Vec::new();
    hit.visit_coverage(
        &mut DocumentWork::preparation(&AtomicBool::new(false)),
        |c| {
            if let HitCoverage::Run(c) = c {
                runs.push(c.run);
            }
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(runs, [0, 1]);
    assert_eq!(
        hit.anchor(),
        DocumentAnchor::Run {
            run: 0,
            raw_byte: 0,
            projection_part: 0
        }
    );
    let document = parsed(&format!("{}b", "a".repeat(4096)));
    let budget = ConsumerBudget::new(&document).unwrap();
    assert_eq!(
        found(
            &request(&document, &budget, SearchDomain::MainBody, "ab"),
            Direction::Forward,
            None
        )
        .matched(),
        4095..4097
    );
}
#[test]
fn b3_semantic_barriers_and_provider_breaks() {
    let document = parsed("left\nright\n\nnext ![fox](target) end");
    let budget = ConsumerBudget::new(&document).unwrap();
    assert_eq!(
        found(
            &request(&document, &budget, SearchDomain::MainBody, "left right"),
            Direction::Forward,
            None
        )
        .matched(),
        0..10
    );
    no_match(&document, &budget, SearchDomain::MainBody, "rightnext");
    no_match(&document, &budget, SearchDomain::MainBody, "next  end");
    no_match(&document, &budget, SearchDomain::MainBody, "next fox end");
    let document = parsed("left  \nright");
    let budget = ConsumerBudget::new(&document).unwrap();
    assert_eq!(
        found(
            &request(&document, &budget, SearchDomain::MainBody, "left\nright"),
            Direction::Forward,
            None
        )
        .matched(),
        0..10
    );
}
#[test]
fn b3_controls_keep_source_and_projection_distinct() {
    let raw = "A\u{1b}B";
    let document = parsed(raw);
    let budget = ConsumerBudget::new(&document).unwrap();
    let control = found(
        &request(&document, &budget, SearchDomain::MainBody, "␛"),
        Direction::Forward,
        None,
    );
    let mut saw = false;
    control
        .visit_coverage(
            &mut DocumentWork::preparation(&AtomicBool::new(false)),
            |c| {
                if let HitCoverage::Run(c) = c {
                    saw |= c.relation == crate::document_text::Relation::ControlProjection
                        && c.source == Some(1..2);
                }
                Ok(())
            },
        )
        .unwrap();
    assert!(saw);
    no_match(&document, &budget, SearchDomain::SourceOriginal, "␛");
    assert_eq!(
        source_found(
            &request(&document, &budget, SearchDomain::SourceOriginal, "\u{1b}"),
            Direction::Forward,
            None
        )
        .range(),
        1..2
    );
}
#[test]
fn b3_hidden_body_title_and_heading_logical_targets() {
    let document = parsed(NESTED);
    let budget = ConsumerBudget::new(&document).unwrap();
    found(
        &request(&document, &budget, SearchDomain::MainBody, "HIT"),
        Direction::Forward,
        None,
    );
    no_match(&document, &budget, SearchDomain::MainBody, "OUT");
    found(
        &request(&document, &budget, SearchDomain::CalloutTitle, "OUT"),
        Direction::Forward,
        None,
    );
    found(
        &request(&document, &budget, SearchDomain::CalloutTitle, "IN"),
        Direction::Forward,
        None,
    );
    let start = HeadingCursor::start(document.clone(), &budget).unwrap();
    let page = start
        .page(
            &budget,
            &mut DocumentWork::preparation(&AtomicBool::new(false)),
        )
        .unwrap();
    assert_eq!(page.targets().len(), 1);
    let target = &page.targets()[0];
    assert_eq!(document.blocks()[target.block() as usize].kind(), 2);
    assert_eq!(target.generation(), 71);
    assert_eq!(target.request(), 93);
    let mut title = String::new();
    target
        .visit_title(
            &budget,
            &AtomicBool::new(false),
            &mut DocumentWork::preparation(&AtomicBool::new(false)),
            |chunk| {
                title.push_str(chunk);
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(title, "Destino");
    assert_eq!(
        document
            .blocks()
            .iter()
            .filter(|b| b.kind() == 4 && b.aux2() == 2)
            .count(),
        2
    );
}
#[test]
fn b3_heading_pages_actual_candidate_quota_and_cancel() {
    let document = parsed(&"# Same\n\n".repeat(65));
    let budget = ConsumerBudget::new(&document).unwrap();
    let base = budget.used();
    let start = HeadingCursor::start(document.clone(), &budget).unwrap();
    let first = start
        .page(
            &budget,
            &mut DocumentWork::preparation(&AtomicBool::new(false)),
        )
        .unwrap();
    assert_eq!(first.targets().len(), 64);
    let second = first
        .next()
        .unwrap()
        .page(
            &budget,
            &mut DocumentWork::preparation(&AtomicBool::new(false)),
        )
        .unwrap();
    assert_eq!(second.targets().len(), 1);
    assert!(second.next().is_none());
    let mut ids = std::collections::HashSet::new();
    for target in first.targets().iter().chain(second.targets()) {
        assert!(ids.insert(target.block()));
    }
    assert_eq!(ids.len(), 65);
    let retained = budget.used();
    let fill = budget
        .reserve(crate::document_consumer_budget::CONSUMER_LIMIT - retained)
        .unwrap();
    assert!(matches!(
        first.next().unwrap().page(
            &budget,
            &mut DocumentWork::preparation(&AtomicBool::new(false))
        ),
        Err(Error::Budget)
    ));
    assert_eq!(first.targets().len(), 64);
    drop(fill);
    assert_eq!(budget.used(), retained);
    assert!(matches!(
        start.page(
            &budget,
            &mut DocumentWork::preparation(&AtomicBool::new(true))
        ),
        Err(Error::Cancelled)
    ));
    assert!(matches!(
        start.page(
            &budget,
            &mut DocumentWork::bounded(&AtomicBool::new(false), 0).unwrap()
        ),
        Err(Error::Work)
    ));
    drop(second);
    drop(first);
    drop(start);
    assert_eq!(budget.used(), base);
}
#[test]
fn b3_query_limit_work_cancel_and_quota_keep_current() {
    let document = parsed("aaa");
    let budget = ConsumerBudget::new(&document).unwrap();
    let base = budget.used();
    let query = request(&document, &budget, SearchDomain::MainBody, "aa");
    let current = found(&query, Direction::Forward, None);
    let retained = budget.used();
    let fill = budget
        .reserve(crate::document_consumer_budget::CONSUMER_LIMIT - retained)
        .unwrap();
    assert!(matches!(
        query.find(
            Direction::Forward,
            Some(current.cursor()),
            &AtomicBool::new(false)
        ),
        SearchOutcome::Failed(Error::Budget)
    ));
    assert_eq!(current.matched(), 0..2);
    drop(fill);
    assert_eq!(budget.used(), retained);
    assert!(matches!(
        query.find(Direction::Forward, None, &AtomicBool::new(true)),
        SearchOutcome::Cancelled
    ));
    let cancel = AtomicBool::new(false);
    let mut work = DocumentWork::bounded(&cancel, 0).unwrap();
    assert!(matches!(
        query.find_with_work(Direction::Forward, None, &cancel, &mut work),
        SearchOutcome::Failed(Error::Work)
    ));
    let exact = "é".repeat(2048);
    assert!(
        SearchRequest::prepare(
            document.clone(),
            &budget,
            SearchDomain::SourceOriginal,
            &exact,
            12,
            &cancel
        )
        .is_ok()
    );
    let over = format!("{exact}a");
    assert!(matches!(
        SearchRequest::prepare(
            document.clone(),
            &budget,
            SearchDomain::MainBody,
            &over,
            13,
            &cancel
        ),
        Err(Error::Invalid)
    ));
    no_match(&document, &budget, SearchDomain::MainBody, "");
    cancel.store(true, Ordering::Release);
    assert!(matches!(
        SearchRequest::prepare(
            document.clone(),
            &budget,
            SearchDomain::MainBody,
            "",
            13,
            &cancel
        ),
        Err(Error::Cancelled)
    ));
    drop(current);
    drop(query);
    assert_eq!(budget.used(), base);
}
#[test]
fn b3_multiple_semantic_sequences_direction_and_wrap() {
    let document = parsed("aaa\n\nxxaa\n\nAA\n\naa");
    let budget = ConsumerBudget::new(&document).unwrap();
    let query = request(&document, &budget, SearchDomain::MainBody, "aa");
    let mut current: Option<DocumentHit> = None;
    let mut forward = Vec::new();
    for _ in 0..5 {
        let hit = found(
            &query,
            Direction::Forward,
            current.as_ref().map(DocumentHit::cursor),
        );
        forward.push((hit.block(), hit.matched()));
        current = Some(hit);
    }
    assert_eq!(forward[0], forward[4]);
    assert_eq!(forward[0].1, 0..2);
    assert_eq!(forward[1].1, 1..3);
    assert_eq!(forward[2].1, 2..4);
    assert_eq!(forward[3].1, 0..2);
    current = None;
    for expected in forward[..4].iter().rev() {
        let hit = found(
            &query,
            Direction::Backward,
            current.as_ref().map(DocumentHit::cursor),
        );
        assert_eq!((hit.block(), hit.matched()), *expected);
        current = Some(hit);
    }
}

#[test]
fn b3_found_pending_position_explicit_retry_and_stale() {
    let document = parsed("aaa");
    let budget = ConsumerBudget::new(&document).unwrap();
    let query = request(&document, &budget, SearchDomain::MainBody, "aa");
    let cancel = AtomicBool::new(false);
    let sequence = Sequences::new()
        .next(
            &document,
            SearchDomain::MainBody,
            &mut DocumentWork::preparation(&cancel),
        )
        .unwrap()
        .unwrap();
    let provider = query.provider(sequence, &cancel).unwrap();
    // The semantic match is already accredited; exhaust only positioning Work.
    let pending = match query
        .document_hit(
            Match {
                sequence,
                range: 0..2,
                primary: true,
                provider,
            },
            &cancel,
            &mut DocumentWork::bounded(&cancel, 0).unwrap(),
            budget
                .reserve(size_of::<DocumentHit>() + 2 * size_of::<usize>())
                .unwrap(),
        )
        .unwrap()
    {
        SearchOutcome::DocumentFoundPending(hit, Error::Work) => hit,
        _ => panic!("position failure lost semantic FoundPending"),
    };
    assert_eq!(pending.matched(), 0..2);
    assert_eq!(pending.highlight(), None);
    assert_eq!(
        pending.anchor(),
        DocumentAnchor::Run {
            run: 0,
            raw_byte: 0,
            projection_part: 0
        }
    );
    let ready = match query.retry_hit(&pending, &cancel) {
        SearchOutcome::DocumentFound(hit) => hit,
        _ => panic!("explicit retry did not prepare positioning"),
    };
    assert_eq!(ready.matched(), 0..2);
    assert_eq!(ready.highlight(), Some(0..2));
    assert!(matches!(
        request(&document, &budget, SearchDomain::MainBody, "aa").retry_hit(&pending, &cancel),
        SearchOutcome::Failed(Error::Invalid)
    ));
    cancel.store(true, Ordering::Release);
    assert!(matches!(
        query.retry_hit(&pending, &cancel),
        SearchOutcome::Cancelled
    ));
    assert_eq!(pending.matched(), 0..2);
}
#[test]
fn b3_shared_title_ref_retains_resource_owner_ready_pending_retry() {
    const SHARED: &str =
        include_str!("../tests/fixtures/document-b3-search/shared-resource-title.md");
    const OWNER_ORACLE: &str =
        include_str!("../tests/fixtures/document-b3-search/PREOUTPUT-OWNER01-R2.txt");
    assert!(OWNER_ORACLE.contains("before candidate runtime"));
    let cancel = AtomicBool::new(false);
    let source = Arc::new(
        DocumentSource::new(
            Arc::new(SHARED.to_owned()),
            SourceOrigin::Literal,
            71,
            93,
            &cancel,
        )
        .unwrap(),
    );
    let control = Control {
        cancel: &cancel,
        deadline: Instant::now() + Duration::from_secs(10),
    };
    let mut candidate =
        crate::document_parser::project(source, Profile::Extended, ActorArena::new(), &control)
            .unwrap();
    assert_eq!(candidate.resources.len(), 2);
    let shared_ref = candidate.resources[0].title_decoded();
    let mut words = *candidate.resources[1].words();
    words[5] = shared_ref;
    candidate.resources[1] = crate::document_model::Resource::from_words(words);
    let depth = candidate.max_depth;
    let document = Arc::new(Document::from_validated(
        crate::document_validate::quarantine(candidate, depth, &control).unwrap(),
    ));
    assert_eq!(
        document.resources()[0].title_decoded(),
        document.resources()[1].title_decoded()
    );
    let budget = ConsumerBudget::new(&document).unwrap();
    let query = request(&document, &budget, SearchDomain::ResourceTitle, "same");
    let first = found(&query, Direction::Forward, None);
    let second = found(&query, Direction::Forward, Some(first.cursor()));
    assert_eq!(first.resource_id(), Some(0));
    assert_eq!(second.resource_id(), Some(1));
    assert_eq!(first.domain(), SearchDomain::ResourceTitle);
    assert_eq!(second.domain(), SearchDomain::ResourceTitle);
    assert_eq!(first.anchor(), second.anchor());
    assert_eq!(first.matched(), 0..4);
    assert_eq!(second.matched(), 0..4);
    assert_eq!(
        found(&query, Direction::Forward, Some(second.cursor())).resource_id(),
        Some(0)
    );
    let back = found(&query, Direction::Backward, None);
    assert_eq!(back.resource_id(), Some(1));
    assert_eq!(
        found(&query, Direction::Backward, Some(back.cursor())).resource_id(),
        Some(0)
    );

    let alt_query = request(&document, &budget, SearchDomain::ResourceAlt, "fox");
    let alt0 = found(&alt_query, Direction::Forward, None);
    let alt1 = found(&alt_query, Direction::Forward, Some(alt0.cursor()));
    assert_eq!(alt0.resource_id(), Some(0));
    assert_eq!(alt1.resource_id(), Some(1));
    assert_eq!(alt1.domain(), SearchDomain::ResourceAlt);
    no_match(&document, &budget, SearchDomain::MainBody, "same");

    let mut sequences = Sequences::new();
    sequences
        .next(
            &document,
            SearchDomain::ResourceTitle,
            &mut DocumentWork::preparation(&cancel),
        )
        .unwrap()
        .unwrap();
    let sequence = sequences
        .next(
            &document,
            SearchDomain::ResourceTitle,
            &mut DocumentWork::preparation(&cancel),
        )
        .unwrap()
        .unwrap();
    let provider = query.provider(sequence, &cancel).unwrap();
    let pending = match query
        .document_hit(
            Match {
                sequence,
                range: 0..4,
                primary: true,
                provider,
            },
            &cancel,
            &mut DocumentWork::bounded(&cancel, 0).unwrap(),
            budget
                .reserve(size_of::<DocumentHit>() + 2 * size_of::<usize>())
                .unwrap(),
        )
        .unwrap()
    {
        SearchOutcome::DocumentFoundPending(hit, Error::Work) => hit,
        _ => panic!("position failure lost title hit"),
    };
    assert_eq!(pending.resource_id(), Some(1));
    assert_eq!(pending.domain(), SearchDomain::ResourceTitle);
    assert_eq!(pending.anchor(), second.anchor());
    assert_eq!(pending.matched(), 0..4);
    let ready = match query.retry_hit(&pending, &cancel) {
        SearchOutcome::DocumentFound(hit) => hit,
        _ => panic!("owner title retry failed"),
    };
    assert_eq!(ready.resource_id(), Some(1));
    assert_eq!(ready.anchor(), second.anchor());
    assert_eq!(ready.matched(), 0..4);
    assert!(matches!(
        request(&document, &budget, SearchDomain::ResourceTitle, "same")
            .retry_hit(&pending, &cancel),
        SearchOutcome::Failed(Error::Invalid)
    ));
    assert_eq!(pending.resource_id(), Some(1));

    let body_doc = parsed("text");
    let body_budget = ConsumerBudget::new(&body_doc).unwrap();
    assert_eq!(
        found(
            &request(&body_doc, &body_budget, SearchDomain::MainBody, "text"),
            Direction::Forward,
            None
        )
        .resource_id(),
        None
    );
}
