use super::*;
use crate::{
    document_budget::ActorArena, document_model::Profile, document_obsidian::Control,
    document_source::DocumentSource, input::SourceOrigin,
};
use sha2::{Digest, Sha256};
use std::{sync::atomic::AtomicBool, time::Duration};
fn document(text: &str) -> Arc<Document> {
    let cancel = AtomicBool::new(false);
    let ctl = Control {
        cancel: &cancel,
        deadline: Instant::now() + Duration::from_secs(10),
    };
    let source = Arc::new(
        DocumentSource::new(
            Arc::new(text.into()),
            SourceOrigin::Literal,
            11,
            19,
            &cancel,
        )
        .unwrap(),
    );
    let c = crate::document_parser::project(source, Profile::Extended, ActorArena::new(), &ctl)
        .unwrap();
    let depth = crate::document_validate::validate(&c, &ctl).unwrap();
    Arc::new(Document::from_validated(
        crate::document_validate::quarantine(c, depth, &ctl).unwrap(),
    ))
}
fn context() -> OperationContext {
    OperationContext::document(
        41,
        Instant::now() + Duration::from_secs(20),
        Arc::new(AtomicBool::new(false)),
    )
}
fn blocks(doc: &Document) -> impl Iterator<Item = u32> + '_ {
    doc.blocks()
        .iter()
        .filter(|b| b.kind() == 10)
        .map(|b| b.id())
}
fn ready(
    doc: &Arc<Document>,
    budget: &ConsumerBudget,
    id: u32,
    ctx: &OperationContext,
) -> Arc<CodeSnapshot> {
    match CodeSnapshot::construct(doc, budget, id, ctx, true).unwrap() {
        CodeConstruction::Ready(v) => v,
        CodeConstruction::Plain { reason } => panic!("unexpectedPlain {reason:?}"),
    }
}
#[test]
fn real_duplicate_fences_have_distinct_keys_and_same_payload_hash() {
    let doc = document(include_str!(
        "../tests/fixtures/markdown-code/C01-duplicate.md"
    ));
    let budget = ConsumerBudget::new(&doc).unwrap();
    let base = budget.used();
    let ctx = context();
    let ids = blocks(&doc).collect::<Vec<_>>();
    assert_eq!(ids.len(), 2);
    let a = ready(&doc, &budget, ids[0], &ctx);
    let b = ready(&doc, &budget, ids[1], &ctx);
    assert_eq!(a.input().text(), "let x = 1;\n");
    assert_eq!(a.key().language(), "Rust");
    assert_eq!(a.key().payload_hash(), b.key().payload_hash());
    assert!(!a.key().same(b.key()));
    assert_eq!(
        a.offset_for(b.key(), a.segments()[0].run, 0, &ctx),
        Err(FallbackReason::InvalidIpc)
    );
    let input = a.input().clone();
    drop(a);
    drop(b);
    assert!(budget.used() > base);
    drop(input);
    assert_eq!(budget.used(), base);
}
#[test]
fn real_code_text_preserves_unicode_controls_alias_and_quote_deindent() {
    for (fixture, payload, language) in [
        (
            include_str!("../tests/fixtures/markdown-code/C02-unicode-control.md"),
            "é\t$a\x1b[31m\n👩🏽‍💻\n",
            "Nushell",
        ),
        (
            include_str!("../tests/fixtures/markdown-code/C03-explicit-alias.md"),
            "$x = 1\n",
            "PowerShell",
        ),
        (
            include_str!("../tests/fixtures/markdown-code/C07-quoted-fence.md"),
            "let x = 1;\n",
            "Rust",
        ),
    ] {
        let doc = document(fixture);
        let budget = ConsumerBudget::new(&doc).unwrap();
        let ctx = context();
        let id = blocks(&doc).next().unwrap();
        let s = ready(&doc, &budget, id, &ctx);
        assert_eq!(s.input().text(), payload);
        assert_eq!(s.key().language(), language);
        assert_eq!(
            *s.key().payload_hash(),
            <[u8; 32]>::from(Sha256::digest(payload.as_bytes()))
        );
        for part in s.segments() {
            let row = &doc.runs()[part.run as usize];
            let text = doc.text(row.text()).unwrap();
            assert_eq!(
                &s.input().text()[part.payload_start as usize..part.payload_end as usize],
                text
            );
            for local in 0..=text.len() {
                let result = s.offset_for(s.key(), part.run, local, &ctx);
                if local < text.len() && text.is_char_boundary(local) {
                    assert_eq!(result, Ok(part.payload_start + local as u32));
                } else {
                    assert_eq!(result, Err(FallbackReason::InvalidIpc));
                }
            }
        }
    }
}
#[test]
fn ineligible_fences_stay_plain_without_retained_allocations() {
    for fixture in [
        include_str!("../tests/fixtures/markdown-code/C04-unknown.md"),
        include_str!("../tests/fixtures/markdown-code/C05-empty.md"),
        include_str!("../tests/fixtures/markdown-code/C06-indented.md"),
        include_str!("../tests/fixtures/markdown-code/C08-no-token.md"),
    ] {
        let doc = document(fixture);
        let budget = ConsumerBudget::new(&doc).unwrap();
        let used = budget.used();
        let ctx = context();
        let id = blocks(&doc).next().unwrap();
        assert!(matches!(
            CodeSnapshot::construct(&doc, &budget, id, &ctx, true).unwrap(),
            CodeConstruction::Plain { .. }
        ));
        assert_eq!(budget.used(), used);
    }
    let doc = document(include_str!(
        "../tests/fixtures/markdown-code/C01-duplicate.md"
    ));
    let budget = ConsumerBudget::new(&doc).unwrap();
    let used = budget.used();
    assert!(matches!(
        CodeSnapshot::construct(
            &doc,
            &budget,
            blocks(&doc).next().unwrap(),
            &context(),
            false
        )
        .unwrap(),
        CodeConstruction::Plain { .. }
    ));
    assert_eq!(budget.used(), used);
}
#[test]
fn crlf_constructor_uses_semantic_refs_without_second_normalization() {
    let doc = document(include_str!("../tests/fixtures/markdown-code/C09-crlf.md"));
    let budget = ConsumerBudget::new(&doc).unwrap();
    let ctx = context();
    let s = ready(&doc, &budget, blocks(&doc).next().unwrap(), &ctx);
    let joined = s
        .segments()
        .iter()
        .map(|part| doc.text(doc.runs()[part.run as usize].text()).unwrap())
        .collect::<String>();
    assert_eq!(s.input().text(), joined);
    assert_eq!(
        *s.key().payload_hash(),
        <[u8; 32]>::from(Sha256::digest(joined.as_bytes()))
    );
}
#[test]
fn exact_budget_and_one_byte_less_are_transactional() {
    let doc = document(include_str!(
        "../tests/fixtures/markdown-code/C01-duplicate.md"
    ));
    let budget = ConsumerBudget::new(&doc).unwrap();
    let base = budget.used();
    let ctx = context();
    let id = blocks(&doc).next().unwrap();
    let s = ready(&doc, &budget, id, &ctx);
    let needed = budget.used() - base;
    drop(s);
    assert_eq!(budget.used(), base);
    for free in [needed, needed - 1] {
        let held = budget
            .reserve(crate::document_consumer_budget::CONSUMER_LIMIT - base - free)
            .unwrap();
        let before = budget.used();
        let result = CodeSnapshot::construct(&doc, &budget, id, &ctx, true).unwrap();
        assert_eq!(matches!(result, CodeConstruction::Ready(_)), free == needed);
        drop(result);
        assert_eq!(budget.used(), before);
        drop(held);
        assert_eq!(budget.used(), base);
    }
}
#[test]
fn foreign_budget_domain_context_and_longline_reject_without_partial_snapshot() {
    let doc = document(include_str!(
        "../tests/fixtures/markdown-code/C01-duplicate.md"
    ));
    let foreign = document(include_str!(
        "../tests/fixtures/markdown-code/C01-duplicate.md"
    ));
    let budget = ConsumerBudget::new(&doc).unwrap();
    let wrong = ConsumerBudget::new(&foreign).unwrap();
    let ctx = context();
    let id = blocks(&doc).next().unwrap();
    assert!(matches!(
        CodeSnapshot::construct(&doc, &wrong, id, &ctx, true),
        Err(FallbackReason::InvalidIpc)
    ));
    assert!(matches!(
        CodeSnapshot::construct(&doc, &budget, id, &OperationContext::LegacySource, true),
        Err(FallbackReason::InvalidIpc)
    ));
    let expired = OperationContext::document(42, Instant::now(), Arc::new(AtomicBool::new(false)));
    assert!(matches!(
        CodeSnapshot::construct(&doc, &budget, id, &expired, true),
        Err(FallbackReason::RequestTimeout)
    ));
    let s = ready(&doc, &budget, id, &ctx);
    assert_eq!(
        s.offset_for(s.key(), s.segments()[0].run, 0, &context()),
        Err(FallbackReason::InvalidIpc)
    );
    drop(s);
    let text = format!(
        "```rs\n{}\n```\n",
        "a".repeat(crate::syntax_worker::MAX_LINE)
    );
    let giant = document(&text);
    let b = ConsumerBudget::new(&giant).unwrap();
    let baseline = b.used();
    assert!(matches!(
        CodeSnapshot::construct(&giant, &b, blocks(&giant).next().unwrap(), &context(), true)
            .unwrap(),
        CodeConstruction::Plain {
            reason: Some(FallbackReason::LongLine)
        }
    ));
    assert_eq!(b.used(), baseline);
}

#[test]
fn validated_controlled_multirun_preserves_empty_segments_and_origins() {
    use crate::document_model::{Item, Origin, Run, TextRef};
    let cancel = AtomicBool::new(false);
    let ctl = Control {
        cancel: &cancel,
        deadline: Instant::now() + Duration::from_secs(10),
    };
    let source = Arc::new(
        DocumentSource::new(
            Arc::new("```rs\néA\t\x1b\n```\n".into()),
            SourceOrigin::Literal,
            11,
            19,
            &cancel,
        )
        .unwrap(),
    );
    let mut c = crate::document_parser::project(source, Profile::Extended, ActorArena::new(), &ctl)
        .unwrap();
    let block = c.blocks.iter().find(|b| b.kind() == 10).unwrap().id();
    let domain = c.blocks[block as usize].domain();
    assert_eq!(c.runs.len(), 1);
    let template = *c.runs[0].words();
    let first = c.blocks[block as usize].first_item();
    c.runs.truncate(0);
    c.blocks[block as usize].0[3] = 5;
    c.domains[domain as usize].0[4] = 5;
    for (ordinal, (start, end)) in [(6u32, 6u32), (6, 8), (8, 8), (8, 10), (10, 11)]
        .into_iter()
        .enumerate()
    {
        let origin = c.origins.len() as u32;
        c.origins
            .push(Origin::from_words([0, start, end, 0]))
            .unwrap();
        let text = c.texts.len() as u32;
        c.texts
            .push(TextRef::from_words([0, start, end, origin]))
            .unwrap();
        let mut row = template;
        row[0] = ordinal as u32;
        row[2] = ordinal as u32;
        row[5] = text;
        row[6] = origin;
        c.runs.push(Run::from_words(row)).unwrap();
        if ordinal == 0 {
            c.items[first as usize].0 = [block, 0, 1, 0];
        } else {
            c.items
                .push(Item::from_words([block, ordinal as u32, 1, ordinal as u32]))
                .unwrap();
        }
    }
    let depth = crate::document_validate::validate(&c, &ctl).unwrap();
    let doc = Arc::new(Document::from_validated(
        crate::document_validate::quarantine(c, depth, &ctl).unwrap(),
    ));
    let budget = ConsumerBudget::new(&doc).unwrap();
    let ctx = context();
    let s = ready(&doc, &budget, block, &ctx);
    assert_eq!(s.input().text(), "éA\t\x1b");
    assert_eq!(
        s.segments()
            .iter()
            .map(|x| (x.payload_start, x.payload_end))
            .collect::<Vec<_>>(),
        vec![(0, 0), (0, 2), (2, 2), (2, 4), (4, 5)]
    );
    for (run, local, expected) in [(1, 0, 0), (3, 1, 3), (4, 0, 4)] {
        assert_eq!(s.offset_for(s.key(), run, local, &ctx), Ok(expected));
    }
    for (run, local) in [(0, 0), (1, 1), (2, 0), (3, 2)] {
        assert_eq!(
            s.offset_for(s.key(), run, local, &ctx),
            Err(FallbackReason::InvalidIpc)
        );
    }
}

#[test]
fn cancellation_during_bounded_copy_releases_partial_allocation() {
    let text = format!(
        "```rs\n{}```\n",
        format!("{}\n", "a".repeat(1000)).repeat(32)
    );
    let doc = document(&text);
    let budget = ConsumerBudget::new(&doc).unwrap();
    let base = budget.used();
    let ctx = context();
    RetainedSyntaxInput::own_copy_cancel_after(Some(1));
    let result = CodeSnapshot::construct(&doc, &budget, blocks(&doc).next().unwrap(), &ctx, true);
    let copied = RetainedSyntaxInput::own_copy_bytes();
    RetainedSyntaxInput::own_copy_cancel_after(None);
    assert_eq!(copied, 4096);
    assert!(matches!(result, Err(FallbackReason::Cancelled)));
    assert_eq!(budget.used(), base);
}

#[test]
fn exactly_two_record_passes_and_empty_has_no_retained_input() {
    let doc = document(include_str!(
        "../tests/fixtures/markdown-code/C02-unicode-control.md"
    ));
    let budget = ConsumerBudget::new(&doc).unwrap();
    let ctx = context();
    let id = blocks(&doc).next().unwrap();
    let count = doc.domains()[doc.blocks()[id as usize].domain() as usize].run_count() as usize;
    RetainedSyntaxInput::own_domain_stats(true);
    let s = ready(&doc, &budget, id, &ctx);
    assert_eq!(
        RetainedSyntaxInput::own_domain_stats(true),
        [count, count, 1]
    );
    drop(s);
    let empty = document(include_str!("../tests/fixtures/markdown-code/C05-empty.md"));
    let b = ConsumerBudget::new(&empty).unwrap();
    let base = b.used();
    RetainedSyntaxInput::own_domain_stats(true);
    assert!(matches!(
        CodeSnapshot::construct(&empty, &b, blocks(&empty).next().unwrap(), &context(), true)
            .unwrap(),
        CodeConstruction::Plain { reason: None }
    ));
    let stats = RetainedSyntaxInput::own_domain_stats(true);
    assert_eq!(stats[1], 0);
    assert_eq!(stats[2], 0);
    assert_eq!(b.used(), base);
}

#[test]
fn empty_natural_fence_plain_has_no_peak_even_when_consumer_full() {
    let doc = document(include_str!("../tests/fixtures/markdown-code/C05-empty.md"));
    let budget = ConsumerBudget::new(&doc).unwrap();
    let id = blocks(&doc).next().unwrap();
    for full in [false, true] {
        let held = if full {
            Some(
                budget
                    .reserve(crate::document_consumer_budget::CONSUMER_LIMIT - budget.used())
                    .unwrap(),
            )
        } else {
            None
        };
        let before = budget.used();
        let peak = budget.peak();
        RetainedSyntaxInput::own_domain_stats(true);
        assert!(matches!(
            CodeSnapshot::construct(&doc, &budget, id, &context(), true).unwrap(),
            CodeConstruction::Plain { reason: None }
        ));
        assert_eq!(budget.used(), before);
        assert_eq!(budget.peak(), peak);
        assert_eq!(RetainedSyntaxInput::own_domain_stats(true), [0; 3]);
        drop(held);
    }
}

#[test]
fn predicates_descriptor_identity_and_no_new_payload() {
    let doc = document("```rust\nA\n```\n\n```rust\nA\n```\n");
    let foreign = document("```rust\nA\n```\n\n```rust\nA\n```\n");
    let budget = ConsumerBudget::new(&doc).unwrap();
    let ids = blocks(&doc).collect::<Vec<_>>();
    let snapshot = ready(&doc, &budget, ids[0], &context());
    let used = budget.used();
    let peak = budget.peak();
    for _ in 0..20 {
        assert!(snapshot.key().matches_document_block(&doc, ids[0], "Rust"));
        assert!(!snapshot.key().matches_document_block(&doc, ids[1], "Rust"));
        assert!(
            !snapshot
                .key()
                .matches_document_block(&foreign, ids[0], "Rust")
        );
        assert!(
            !snapshot
                .key()
                .matches_document_block(&doc, ids[0], "Nushell")
        );
        assert!(
            !snapshot
                .key()
                .matches_document_block(&doc, u32::MAX, "Rust")
        );
    }
    assert_eq!((budget.used(), budget.peak()), (used, peak));
    let alias = budget.clone();
    let other = ConsumerBudget::new(&doc).unwrap();
    assert!(budget.same(&alias));
    assert!(!budget.same(&other));
}
#[test]
fn predicates_coverage_lines_empty_eof_and_foreign_input() {
    use crate::{document_code_cache::SemanticCoverage, syntax::Viewport};
    let doc = document("```rust\nA\n\né\n```\n");
    let budget = ConsumerBudget::new(&doc).unwrap();
    let id = blocks(&doc).next().unwrap();
    let snapshot = ready(&doc, &budget, id, &context());
    assert_eq!(snapshot.input().text(), "A\n\né\n");
    assert_eq!(snapshot.input().line_count(), Some(4));
    let used = budget.used();
    let peak = budget.peak();
    let all = SemanticCoverage::from_request(&snapshot, None).unwrap();
    let view = |first_line, line_count| {
        Some(Viewport {
            first_line,
            line_count,
        })
    };
    assert!(all.contains_request(&snapshot, view(1, 1)).unwrap());
    assert!(all.contains_request(&snapshot, view(3, 1)).unwrap());
    assert!(all.contains_request(&snapshot, view(u32::MAX, 1)).unwrap());
    let middle = SemanticCoverage::from_request(&snapshot, view(1, 1)).unwrap();
    assert!(middle.contains_request(&snapshot, view(1, 1)).unwrap());
    assert!(!middle.contains_request(&snapshot, view(1, 2)).unwrap());
    assert!(!middle.contains_request(&snapshot, view(3, 1)).unwrap());
    assert!(!middle.contains_request(&snapshot, None).unwrap());
    assert!(SemanticCoverage::from_request(&snapshot, view(0, 0)).is_err());
    assert!(SemanticCoverage::from_request(&snapshot, view(0, 257)).is_err());
    let second = ready(&doc, &budget, id, &context());
    assert!(!all.contains_request(&second, None).unwrap());
    drop(second);
    assert_eq!(budget.used(), used);
    // Queries themselves reserve/copy nothing; constructing second snapshot raised peak separately.
    let final_peak = budget.peak();
    for _ in 0..20 {
        assert!(all.contains_request(&snapshot, view(0, 1)).unwrap());
    }
    assert_eq!(budget.peak(), final_peak);
    assert!(final_peak >= peak);
}
#[test]
fn predicates_semantic_mapping_does_not_authorize_expired_operation() {
    let doc = document("```rust\né\n```\n");
    let budget = ConsumerBudget::new(&doc).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    let ctx = OperationContext::document(71, deadline, Arc::new(AtomicBool::new(false)));
    let snapshot = ready(&doc, &budget, blocks(&doc).next().unwrap(), &ctx);
    let run = snapshot.segments()[0].run;
    std::thread::sleep(
        deadline.saturating_duration_since(Instant::now()) + Duration::from_millis(1),
    );
    assert!(Instant::now() >= deadline);
    assert_eq!(
        snapshot.offset_for(snapshot.key(), run, 0, &ctx),
        Err(FallbackReason::RequestTimeout)
    );
    assert_eq!(snapshot.semantic_offset_for(snapshot.key(), run, 0), Ok(0));
    assert_eq!(
        snapshot.semantic_offset_for(snapshot.key(), run, 1),
        Err(FallbackReason::InvalidIpc)
    );
    assert_eq!(
        snapshot.offset_for(snapshot.key(), run, 0, &context()),
        Err(FallbackReason::InvalidIpc)
    );
}
#[test]
fn predicates_cache_capacity_charge_empty_and_owner_lifetime() {
    use crate::document_code_cache::SemanticCodeCache;
    let doc = document("```rust\nA\n```\n");
    let budget = ConsumerBudget::new(&doc).unwrap();
    let base = budget.used();
    assert!(SemanticCodeCache::new(&budget, 257).is_err());
    assert_eq!(budget.used(), base);
    let mut cache = SemanticCodeCache::new(&budget, 2).unwrap();
    assert!(budget.used() > base);
    assert!(
        cache
            .lookup(&doc, blocks(&doc).next().unwrap(), "Rust", None)
            .is_none()
    );
    cache.clear();
    assert!(budget.used() > base);
    let needed = budget.used() - base;
    drop(cache);
    assert_eq!(budget.used(), base);
    let exact_fill = budget
        .reserve(crate::document_consumer_budget::CONSUMER_LIMIT - base - needed)
        .unwrap();
    let exact_cache = SemanticCodeCache::new(&budget, 2).unwrap();
    assert_eq!(
        budget.used(),
        crate::document_consumer_budget::CONSUMER_LIMIT
    );
    drop(exact_cache);
    drop(exact_fill);
    let plus_fill = budget
        .reserve(crate::document_consumer_budget::CONSUMER_LIMIT - base - needed + 1)
        .unwrap();
    let before_reject = budget.used();
    assert!(SemanticCodeCache::new(&budget, 2).is_err());
    assert_eq!(budget.used(), before_reject);
    drop(plus_fill);
    let fill = budget
        .reserve(crate::document_consumer_budget::CONSUMER_LIMIT - base)
        .unwrap();
    assert!(SemanticCodeCache::new(&budget, 1).is_err());
    assert_eq!(
        budget.used(),
        crate::document_consumer_budget::CONSUMER_LIMIT
    );
    drop(fill);
    assert_eq!(budget.used(), base);
}
#[test]
fn predicates_local_stop_during_copy_does_not_mutate_global_context() {
    use std::sync::atomic::Ordering;
    let doc = document(&format!("```rust\n{}\n```\n", "a".repeat(9000)));
    let budget = ConsumerBudget::new(&doc).unwrap();
    let base = budget.used();
    let global = Arc::new(AtomicBool::new(false));
    let ctx =
        OperationContext::document(72, Instant::now() + Duration::from_secs(20), global.clone());
    let local = AtomicBool::new(false);
    RetainedSyntaxInput::own_local_stop_after_copy(Some(4096));
    let result = CodeSnapshot::construct_with_stop(
        &doc,
        &budget,
        blocks(&doc).next().unwrap(),
        &ctx,
        true,
        &local,
    );
    RetainedSyntaxInput::own_local_stop_after_copy(None);
    assert!(matches!(result, Err(FallbackReason::Cancelled)));
    assert!(local.load(Ordering::Acquire));
    assert!(!global.load(Ordering::Acquire));
    assert_eq!(budget.used(), base);
    assert!(ctx.check(Instant::now()).is_ok());
    let fresh_local = AtomicBool::new(false);
    assert!(matches!(
        CodeSnapshot::construct_with_stop(
            &doc,
            &budget,
            blocks(&doc).next().unwrap(),
            &ctx,
            true,
            &fresh_local
        ),
        Ok(CodeConstruction::Ready(_))
    ));
}
#[test]
fn predicates_local_stop_preserves_global_timeout_and_cancel() {
    let doc = document("```rust\nA\n```\n");
    let budget = ConsumerBudget::new(&doc).unwrap();
    let id = blocks(&doc).next().unwrap();
    let local = AtomicBool::new(true);
    let expired = OperationContext::document(73, Instant::now(), Arc::new(AtomicBool::new(false)));
    assert!(matches!(
        CodeSnapshot::construct_with_stop(&doc, &budget, id, &expired, true, &local),
        Err(FallbackReason::RequestTimeout)
    ));
    let cancelled = OperationContext::document(
        74,
        Instant::now() + Duration::from_secs(20),
        Arc::new(AtomicBool::new(true)),
    );
    assert!(matches!(
        CodeSnapshot::construct_with_stop(&doc, &budget, id, &cancelled, true, &local),
        Err(FallbackReason::Cancelled)
    ));
    assert!(matches!(
        CodeSnapshot::construct_with_stop(&doc, &budget, id, &context(), true, &local),
        Err(FallbackReason::Cancelled)
    ));
}
