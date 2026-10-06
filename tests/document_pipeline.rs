//! Source-only assertions fixed before the first integrated worker output.
use r3viewer::{
    document_budget::ActorArena,
    document_controller::{ControllerPoll, DocumentController, IdentityLatch},
    document_model::{Document, ErrorKind, Profile},
    document_source::DocumentSource,
    input::SourceOrigin,
};
use std::{
    sync::{Arc, atomic::AtomicBool},
    thread,
    time::{Duration, Instant},
};
fn run(text: &str) -> Document {
    let cancel = Arc::new(AtomicBool::new(false));
    let source = Arc::new(
        DocumentSource::new(
            Arc::new(text.to_owned()),
            SourceOrigin::Literal,
            7,
            11,
            &cancel,
        )
        .unwrap(),
    );
    let identity = Arc::new(IdentityLatch::new(7, 11));
    let mut controller = DocumentController::new();
    controller
        .prepare(
            std::path::PathBuf::from(env!("CARGO_BIN_EXE_r3viewer")),
            source,
            Profile::Extended,
            ActorArena::new(),
            cancel,
            identity,
        )
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        match controller.poll() {
            ControllerPoll::Ready(doc) => {
                assert!(!controller.pending_cleanup());
                println!(
                    "DOCUMENT_RECEIPT counts={:?} depth={} arena_used={} arena_peak={} source_len={} source_capacity={}",
                    doc.counts(),
                    doc.max_depth(),
                    doc.arena_used(),
                    doc.arena_peak(),
                    doc.source().len(),
                    doc.source().capacity()
                );
                for link in doc.links() {
                    println!(
                        "LINK_RECEIPT kind={} target={:?} raw={:?}",
                        link.kind(),
                        doc.text(link.target_decoded()).unwrap(),
                        doc.text(link.raw_token()).unwrap()
                    );
                }
                for resource in doc.resources() {
                    println!(
                        "RESOURCE_RECEIPT kind={} input={:?} dimensions={}x{}",
                        resource.kind(),
                        doc.text(resource.input_decoded()).unwrap(),
                        resource.width_value(),
                        resource.height_value()
                    );
                }
                return doc;
            }
            ControllerPoll::Failed(error) => {
                panic!("real document worker rejected source fixture: {error:?}")
            }
            ControllerPoll::Pending => {}
            _ => panic!("unexpected controller state"),
        };
        assert!(
            Instant::now() < deadline,
            "owned task did not complete; Drop requests cancellation, supervisor retains authority"
        );
        thread::sleep(Duration::from_millis(2));
    }
}
#[test]
fn actual_worker_table_adapter_to_host_document() {
    let text = "| A |\n|---|\n| [[Nota\\|**Alias**]] ![[dir\\Doc.pdf#page=7\\|100x145]] |\n";
    let doc = run(text);
    assert_eq!(doc.source().text(), text);
    let l = &doc.links()[0];
    assert_eq!(doc.text(l.target_decoded()).unwrap(), "Nota");
    assert_eq!(doc.text(l.raw_token()).unwrap(), "[[Nota\\|**Alias**]]");
    let image = &doc.resources()[0];
    assert_eq!(image.kind(), 4);
    assert_eq!(
        doc.text(image.input_decoded()).unwrap(),
        "dir\\Doc.pdf#page=7"
    );
    assert_eq!((image.width_value(), image.height_value()), (100, 145));
    assert!(doc.spans().iter().any(|s| s.kind() == 1));
}
#[test]
fn actual_worker_complex_roles_and_provenance() {
    let text = "# Heading\n\n- [x] Done\n\n> [!custom]- **Title** $x$\n> Body\n\n![before ![**inner**](inner.png) after](outer.png)\n\n$$x^2$$\n\n```mermaid\ngraph TD; A-->B\n```\n";
    let doc = run(text);
    assert_eq!(doc.source().text(), text);
    let callout = doc.blocks().iter().find(|b| b.kind() == 4).unwrap();
    assert_eq!(doc.text(callout.aux0()).unwrap(), "custom");
    assert_eq!(callout.aux2(), 2);
    let mut title = String::new();
    for r in doc.runs().iter().filter(|r| r.domain() == callout.aux1()) {
        if r.text() != u32::MAX {
            title.push_str(doc.text(r.text()).unwrap());
        }
    }
    assert_eq!(title, "Title $x$");
    assert!(doc.resources().iter().any(|r| r.kind() == 3));
    assert!(
        doc.resources()
            .iter()
            .any(|r| r.kind() == 1 && r.owner_run() == u32::MAX)
    );
    assert_eq!(doc.resources().iter().filter(|r| r.kind() == 0).count(), 1);
    let inner = doc
        .runs()
        .iter()
        .find(|r| r.text() != u32::MAX && doc.text(r.text()).unwrap() == "inner")
        .unwrap();
    let o = &doc.origins()[inner.origin() as usize];
    assert_eq!(o.relation(), 1);
    assert_eq!(
        &text[o.start() as usize..o.end() as usize],
        "![**inner**](inner.png)"
    );
}
#[test]
fn actual_worker_does_not_evaluate_code_or_frontmatter() {
    let text = "---\na: b\n---\n\n```nu\nerror make {msg: 'THIS MUST NEVER EXECUTE'}\n```\n\n<script>[[NoEscape\\|Alias]]</script>\n";
    let doc = run(text);
    assert_eq!(doc.source().text(), text);
    assert!(doc.blocks().iter().any(|b| b.kind() == 14));
    assert!(doc.blocks().iter().any(|b| b.kind() == 13));
    assert!(doc.links().is_empty());
    assert!(doc.resources().is_empty());
}
#[test]
fn cancelled_prepare_is_failed_and_releases_ownership() {
    let cancel = Arc::new(AtomicBool::new(true));
    let source = Arc::new(
        DocumentSource::new(
            Arc::new("x".to_owned()),
            SourceOrigin::Literal,
            7,
            11,
            &AtomicBool::new(false),
        )
        .unwrap(),
    );
    let identity = Arc::new(IdentityLatch::new(7, 11));
    let mut controller = DocumentController::new();
    controller
        .prepare(
            std::path::PathBuf::from(env!("CARGO_BIN_EXE_r3viewer")),
            source,
            Profile::Extended,
            ActorArena::new(),
            cancel,
            identity,
        )
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match controller.poll() {
            ControllerPoll::Failed(error) => {
                assert_eq!(error.kind, ErrorKind::Cancelled);
                assert!(!controller.pending_cleanup());
                break;
            }
            ControllerPoll::Pending => {}
            _ => panic!("unexpected cancellation result"),
        };
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(2));
    }
}

const UNC_PREOUTPUT_CASES: &[(&str, &str, &str, &str, &str, usize, usize)] = &[
    (
        "unc-2-link-table-alias",
        "| 名 |\r\n|---|\r\n| [[\\\\server\\share\\Nota\\|**Álias**]] |\r\n",
        "\\server\\share\\Nota\\",
        "\\\\server\\share\\Nota",
        "link",
        18,
        53,
    ),
    (
        "unc-2-link-table-no-alias",
        "| 名 |\r\n|---|\r\n| [[\\\\server\\share\\Nota]] |\r\n",
        "\\server\\share\\Nota",
        "\\\\server\\share\\Nota",
        "link",
        18,
        41,
    ),
    (
        "unc-2-link-paragraph-alias",
        "[[\\\\server\\share\\Nota\\|**Álias**]]\r\n",
        "\\server\\share\\Nota\\",
        "\\\\server\\share\\Nota\\",
        "link",
        0,
        35,
    ),
    (
        "unc-2-link-paragraph-no-alias",
        "[[\\\\server\\share\\Nota]]\r\n",
        "\\server\\share\\Nota",
        "\\\\server\\share\\Nota",
        "link",
        0,
        23,
    ),
    (
        "unc-2-embed-table-alias",
        "| 名 |\r\n|---|\r\n| ![[\\\\server\\share\\Nota\\|100x145]] |\r\n",
        "\\server\\share\\Nota\\",
        "\\\\server\\share\\Nota",
        "embed",
        18,
        51,
    ),
    (
        "unc-2-embed-table-no-alias",
        "| 名 |\r\n|---|\r\n| ![[\\\\server\\share\\Nota]] |\r\n",
        "\\server\\share\\Nota",
        "\\\\server\\share\\Nota",
        "embed",
        18,
        42,
    ),
    (
        "unc-2-embed-paragraph-alias",
        "![[\\\\server\\share\\Nota\\|100x145]]\r\n",
        "\\server\\share\\Nota\\",
        "\\\\server\\share\\Nota\\",
        "embed",
        0,
        33,
    ),
    (
        "unc-2-embed-paragraph-no-alias",
        "![[\\\\server\\share\\Nota]]\r\n",
        "\\server\\share\\Nota",
        "\\\\server\\share\\Nota",
        "embed",
        0,
        24,
    ),
    (
        "unc-3-link-table-alias",
        "| 名 |\r\n|---|\r\n| [[\\\\\\server\\share\\Nota\\|**Álias**]] |\r\n",
        "\\\\server\\share\\Nota\\",
        "\\\\\\server\\share\\Nota",
        "link",
        18,
        54,
    ),
    (
        "unc-3-link-table-no-alias",
        "| 名 |\r\n|---|\r\n| [[\\\\\\server\\share\\Nota]] |\r\n",
        "\\\\server\\share\\Nota",
        "\\\\\\server\\share\\Nota",
        "link",
        18,
        42,
    ),
    (
        "unc-3-link-paragraph-alias",
        "[[\\\\\\server\\share\\Nota\\|**Álias**]]\r\n",
        "\\\\server\\share\\Nota\\",
        "\\\\\\server\\share\\Nota\\",
        "link",
        0,
        36,
    ),
    (
        "unc-3-link-paragraph-no-alias",
        "[[\\\\\\server\\share\\Nota]]\r\n",
        "\\\\server\\share\\Nota",
        "\\\\\\server\\share\\Nota",
        "link",
        0,
        24,
    ),
    (
        "unc-3-embed-table-alias",
        "| 名 |\r\n|---|\r\n| ![[\\\\\\server\\share\\Nota\\|100x145]] |\r\n",
        "\\\\server\\share\\Nota\\",
        "\\\\\\server\\share\\Nota",
        "embed",
        18,
        52,
    ),
    (
        "unc-3-embed-table-no-alias",
        "| 名 |\r\n|---|\r\n| ![[\\\\\\server\\share\\Nota]] |\r\n",
        "\\\\server\\share\\Nota",
        "\\\\\\server\\share\\Nota",
        "embed",
        18,
        43,
    ),
    (
        "unc-3-embed-paragraph-alias",
        "![[\\\\\\server\\share\\Nota\\|100x145]]\r\n",
        "\\\\server\\share\\Nota\\",
        "\\\\\\server\\share\\Nota\\",
        "embed",
        0,
        34,
    ),
    (
        "unc-3-embed-paragraph-no-alias",
        "![[\\\\\\server\\share\\Nota]]\r\n",
        "\\\\server\\share\\Nota",
        "\\\\\\server\\share\\Nota",
        "embed",
        0,
        25,
    ),
    (
        "unc-4-link-table-alias",
        "| 名 |\r\n|---|\r\n| [[\\\\\\\\server\\share\\Nota\\|**Álias**]] |\r\n",
        "\\\\\\server\\share\\Nota\\",
        "\\\\\\\\server\\share\\Nota",
        "link",
        18,
        55,
    ),
    (
        "unc-4-link-table-no-alias",
        "| 名 |\r\n|---|\r\n| [[\\\\\\\\server\\share\\Nota]] |\r\n",
        "\\\\\\server\\share\\Nota",
        "\\\\\\\\server\\share\\Nota",
        "link",
        18,
        43,
    ),
    (
        "unc-4-link-paragraph-alias",
        "[[\\\\\\\\server\\share\\Nota\\|**Álias**]]\r\n",
        "\\\\\\server\\share\\Nota\\",
        "\\\\\\\\server\\share\\Nota\\",
        "link",
        0,
        37,
    ),
    (
        "unc-4-link-paragraph-no-alias",
        "[[\\\\\\\\server\\share\\Nota]]\r\n",
        "\\\\\\server\\share\\Nota",
        "\\\\\\\\server\\share\\Nota",
        "link",
        0,
        25,
    ),
    (
        "unc-4-embed-table-alias",
        "| 名 |\r\n|---|\r\n| ![[\\\\\\\\server\\share\\Nota\\|100x145]] |\r\n",
        "\\\\\\server\\share\\Nota\\",
        "\\\\\\\\server\\share\\Nota",
        "embed",
        18,
        53,
    ),
    (
        "unc-4-embed-table-no-alias",
        "| 名 |\r\n|---|\r\n| ![[\\\\\\\\server\\share\\Nota]] |\r\n",
        "\\\\\\server\\share\\Nota",
        "\\\\\\\\server\\share\\Nota",
        "embed",
        18,
        44,
    ),
    (
        "unc-4-embed-paragraph-alias",
        "![[\\\\\\\\server\\share\\Nota\\|100x145]]\r\n",
        "\\\\\\server\\share\\Nota\\",
        "\\\\\\\\server\\share\\Nota\\",
        "embed",
        0,
        35,
    ),
    (
        "unc-4-embed-paragraph-no-alias",
        "![[\\\\\\\\server\\share\\Nota]]\r\n",
        "\\\\\\server\\share\\Nota",
        "\\\\\\\\server\\share\\Nota",
        "embed",
        0,
        26,
    ),
];

#[test]
fn pinned_upstream_initial_body_node_loss_matches_all_24_source_predictions() {
    for &(id, text, expected_original, _, kind, start, end) in UNC_PREOUTPUT_CASES {
        let mut observed = 0;
        for (event, range) in pulldown_cmark::Parser::new_ext(
            text,
            pulldown_cmark::Options::from_bits(35998).unwrap(),
        )
        .into_offset_iter()
        {
            let actual = match event {
                pulldown_cmark::Event::Start(pulldown_cmark::Tag::Link {
                    link_type: pulldown_cmark::LinkType::WikiLink { .. },
                    dest_url,
                    ..
                }) if kind == "link" => Some(dest_url),
                pulldown_cmark::Event::Start(pulldown_cmark::Tag::Image {
                    link_type: pulldown_cmark::LinkType::WikiLink { .. },
                    dest_url,
                    ..
                }) if kind == "embed" => Some(dest_url),
                _ => None,
            };
            if let Some(destination) = actual {
                assert_eq!(
                    destination.as_ref(),
                    expected_original,
                    "{id}: original pinned dependency, no product adapter"
                );
                assert_eq!(range, start..end, "{id}: raw whole-token range");
                observed += 1;
            }
        }
        assert_eq!(observed, 1, "{id}: no synthetic wiki");
    }
}
#[test]
fn actual_worker_all_24_unc_source_authority_cases() {
    for &(id, text, _, target, kind, start, end) in UNC_PREOUTPUT_CASES {
        let doc = run(text);
        assert_eq!(doc.source().text(), text, "{id}");
        let (target_id, raw_id) = if kind == "link" {
            assert_eq!(doc.links().len(), 1, "{id}");
            let link = &doc.links()[0];
            (link.target_decoded(), link.raw_token())
        } else {
            assert_eq!(doc.resources().len(), 1, "{id}");
            let res = &doc.resources()[0];
            assert_eq!(res.kind(), 4, "{id}");
            (res.input_decoded(), res.raw_token())
        };
        assert_eq!(doc.text(target_id).unwrap(), target, "{id}");
        assert_eq!(doc.text(raw_id).unwrap(), &text[start..end], "{id}");
        let t = &doc.text_refs()[target_id as usize];
        assert_eq!(t.kind(), 1, "{id}");
        let o = &doc.origins()[t.origin() as usize];
        assert_eq!(
            (o.relation(), o.start() as usize, o.end() as usize),
            (1, start, end),
            "{id}"
        );
        if kind == "link" && id.ends_with("alias") && !id.ends_with("no-alias") {
            assert!(
                doc.spans().iter().any(|span| span.kind() == 1),
                "{id}: Strong alias untouched"
            );
        }
        if kind == "embed" {
            let res = &doc.resources()[0];
            let dimensions = if id.ends_with("no-alias") {
                (0, 0)
            } else {
                (100, 145)
            };
            assert_eq!((res.width_value(), res.height_value()), dimensions, "{id}");
        }
    }
}
#[test]
fn independent_original_unc_repro_keeps_two_initial_backslashes() {
    let text = include_str!("fixtures/markdown/wiki-unc-crlf.md");
    let doc = run(text);
    let link = &doc.links()[0];
    assert_eq!(
        doc.text(link.target_decoded()).unwrap(),
        "\\\\server\\share\\Nota"
    );
    assert_eq!(doc.text(link.raw_token()).unwrap(), &text[18..53]);
    assert_eq!(doc.source().text(), text);
    assert!(doc.spans().iter().any(|span| span.kind() == 1));
}

#[test]
fn actual_worker_empty_callout_spans_have_source_defined_owners() {
    for text in [
        "> [!note] [](x)\n> body\n",
        "> [!note] [](x)\n",
        "> [!note] **[](x)**\n> body\n",
        "> [!note] [](x)\n> [](y)\n",
        "> [!note] [](á)\r\n> [](名)",
    ] {
        let doc = run(text);
        assert_eq!(doc.source().text(), text);
        let callout = doc.blocks().iter().find(|b| b.kind() == 4).unwrap();
        let title = callout.aux1();
        assert_ne!(title, u32::MAX);
        assert_eq!(
            (
                doc.domains()[title as usize].kind(),
                doc.domains()[title as usize].owner(),
                doc.domains()[title as usize].run_count()
            ),
            (2, callout.id(), 0)
        );
        for link in doc.links() {
            let raw = doc.text(link.raw_token()).unwrap();
            assert_eq!(
                doc.text(link.target_decoded()).unwrap(),
                &raw[3..raw.len() - 1]
            );
            let start = text.find(raw).unwrap();
            let r = &doc.text_refs()[link.raw_token() as usize];
            let o = &doc.origins()[r.origin() as usize];
            assert_eq!(
                (o.start() as usize, o.end() as usize),
                (start, start + raw.len())
            );
            if text.contains("**[](x)**") {
                let strong = doc
                    .spans()
                    .iter()
                    .find(|s| s.kind() == 1 && s.domain() == title)
                    .unwrap();
                assert_eq!((strong.start_ordinal(), strong.end_ordinal()), (0, 0));
                let inner = doc.spans().iter().find(|s| s.link() == link.id()).unwrap();
                assert_eq!(inner.parent_span(), strong.id());
                let origin = &doc.origins()[strong.origin() as usize];
                assert_eq!(
                    &text[origin.start() as usize..origin.end() as usize],
                    "**[](x)**"
                );
            }
            let header = start < text.find('\n').unwrap();
            let d = &doc.domains()[link.label_domain() as usize];
            assert_eq!(d.run_count(), 0);
            assert_eq!((link.label_start(), link.label_count()), (0, 0));
            if header {
                assert_eq!(link.label_domain(), title);
            } else {
                assert_ne!(link.label_domain(), title);
                assert_eq!(d.kind(), 0);
                let body = &doc.blocks()[d.owner() as usize];
                assert_eq!(body.kind(), 1);
                assert_eq!(body.parent(), callout.id());
            }
            let span = doc.spans().iter().find(|s| s.link() == link.id()).unwrap();
            assert_eq!(
                (span.domain(), span.start_ordinal(), span.end_ordinal()),
                (link.label_domain(), 0, 0)
            );
            let origin = &doc.origins()[span.origin() as usize];
            assert_eq!(
                (origin.start() as usize, origin.end() as usize),
                (start, start + raw.len())
            );
        }
        assert_eq!(
            doc.links().len(),
            if text.contains("[](y)") || text.contains("[](名)") {
                2
            } else {
                1
            }
        );
    }
}
#[test]
fn actual_worker_callout_empty_and_quote_controls_preserve_source() {
    for text in [
        "> [!note]\n",
        "> [!note] **Title**\n> body\n",
        "> [!note] $x$\n> body\n",
        "> [!note] ****\n> body\n",
        "> [!note] **a\n> b\n",
        "> [](x)\n> body\n",
        "> [!note] [a\n> b](x)\n",
        "> [!note] **a\n> b**\n",
    ] {
        let doc = run(text);
        assert_eq!(doc.source().text(), text);
        if text == "> [!note]\n" {
            // Pinned upstream emits BlockQuote(Some(Note)) without a Paragraph.
            // Existing producer keeps that inert Quote; this correction adds no recognition.
            assert!(!doc.blocks().iter().any(|b| b.kind() == 4));
            assert!(doc.blocks().iter().any(|b| b.kind() == 3));
            assert!(doc.domains().is_empty());
        }
        if text == "> [](x)\n> body\n" || text.contains("[a\n") || text.contains("b**") {
            assert!(!doc.blocks().iter().any(|b| b.kind() == 4));
            assert!(doc.blocks().iter().any(|b| b.kind() == 3));
        }
        if text.contains("$x$") {
            let c = doc.blocks().iter().find(|b| b.kind() == 4).unwrap();
            assert!(
                doc.runs()
                    .iter()
                    .filter(|r| r.domain() == c.aux1())
                    .any(|r| r.kind() == 0 && doc.text(r.text()) == Ok("$x$"))
            );
        }
    }
}
#[test]
fn original_parser_empty_link_ranges_precede_candidate_outputs() {
    for text in [
        "> [!note] [](x)\n> [](y)\n",
        "> [!note] **[](x)**\n> body\n",
        "> [!note] [](á)\r\n> [](名)",
    ] {
        let mut count = 0;
        for (event, range) in pulldown_cmark::Parser::new_ext(
            text,
            pulldown_cmark::Options::from_bits(35998).unwrap(),
        )
        .into_offset_iter()
        {
            if let pulldown_cmark::Event::Start(pulldown_cmark::Tag::Link { dest_url, .. }) = event
            {
                let raw = &text[range.clone()];
                assert!(raw.starts_with("[]("));
                assert_eq!(&raw[3..raw.len() - 1], dest_url.as_ref());
                println!("ORIGINAL_EMPTY_LINK {range:?} {raw:?}");
                count += 1;
            }
        }
        assert_eq!(count, if text.contains("body") { 1 } else { 2 });
    }
}

#[test]
fn primary_marker_only_has_no_paragraph() {
    let text = "> [!note]\n";
    let events: Vec<_> =
        pulldown_cmark::Parser::new_ext(text, pulldown_cmark::Options::from_bits(35998).unwrap())
            .into_offset_iter()
            .collect();
    println!("PRIMARY_MARKER_ONLY {events:?}");
    assert!(!events.iter().any(|(e, _)| matches!(
        e,
        pulldown_cmark::Event::Start(pulldown_cmark::Tag::Paragraph)
    )));
}

#[test]
fn actual_worker_body_only_callout_has_no_title_domain() {
    for text in [
        "> [!note]\n> [](y)\n",
        "> [!NOTE]\n> [](y)\n",
        "> [!note]\r\n> [](y)\r\n",
        "> [!NOTE]\n> [](y)",
    ] {
        let doc = run(text);
        assert_eq!(doc.source().text(), text);
        let callout = doc.blocks().iter().find(|b| b.kind() == 4).unwrap();
        assert_eq!(callout.aux1(), u32::MAX);
        let link = doc.links()[0];
        assert_eq!(doc.text(link.target_decoded()).unwrap(), "y");
        let domain = doc.domains()[link.label_domain() as usize];
        assert_eq!((domain.kind(), domain.run_count()), (0, 0));
        let block = doc.blocks()[domain.owner() as usize];
        assert_eq!((block.kind(), block.parent()), (1, callout.id()));
        let span = doc.spans().iter().find(|s| s.link() == link.id()).unwrap();
        assert_eq!(
            (span.domain(), span.start_ordinal(), span.end_ordinal()),
            (domain.id(), 0, 0)
        );
        let origin = doc.origins()[span.origin() as usize];
        let start = text.find("[](y)").unwrap();
        assert_eq!(
            (origin.start() as usize, origin.end() as usize),
            (start, start + 5)
        );
    }
}
