//! B2-RESOURCE-OWNER01 real parser/quarantine regression on baseline3db.
use crate::{
    document_budget::ActorArena, document_model::*, document_obsidian::Control,
    document_source::DocumentSource, input::SourceOrigin,
};
use std::{
    sync::{Arc, atomic::AtomicBool},
    time::{Duration, Instant},
};
fn candidate(raw: &str) -> Candidate {
    let cancel = AtomicBool::new(false);
    let source = Arc::new(
        DocumentSource::new(
            Arc::new(raw.to_owned()),
            SourceOrigin::Literal,
            101,
            202,
            &cancel,
        )
        .unwrap(),
    );
    crate::document_parser::project(
        source,
        Profile::Extended,
        ActorArena::new(),
        &Control {
            cancel: &cancel,
            deadline: Instant::now() + Duration::from_secs(10),
        },
    )
    .unwrap()
}
fn checked(c: Candidate) -> Arc<Document> {
    let cancel = AtomicBool::new(false);
    let depth = c.max_depth;
    Arc::new(Document::from_validated(
        crate::document_validate::quarantine(
            c,
            depth,
            &Control {
                cancel: &cancel,
                deadline: Instant::now() + Duration::from_secs(10),
            },
        )
        .unwrap(),
    ))
}
pub(super) fn parsed(raw: &str) -> Arc<Document> {
    checked(candidate(raw))
}

#[test]
fn legacy_three_images_alt_owner_outside_blocks_is_safe() {
    let d = parsed(include_str!(
        "../tests/fixtures/document-b2-resource-owner/titles.md"
    ));
    assert_eq!(d.resources().len(), 3);
    assert_eq!(d.blocks().len(), 2);
    let last = &d.resources()[2];
    let alt = &d.domains()[last.alt_domain() as usize];
    assert_eq!(alt.owner(), last.id());
    assert!(alt.owner() as usize >= d.blocks().len());
    use crate::{
        document_layout::Geometry,
        document_view::{DocumentPreparationAuthority, DocumentView, start_document},
        preparation_owner::{Cause, Outcome, PreparedCompletion},
    };
    let input = DocumentView::initial_input(
        d,
        Geometry {
            x: 0,
            y: 0,
            width: 80,
            height: 24,
            left: 0,
        },
    )
    .unwrap();
    let binding = input.key().binding;
    let mut authority = DocumentPreparationAuthority::default();
    start_document(input, &mut authority).unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !authority.is_finished() && Instant::now() < deadline {
        std::thread::yield_now()
    }
    let completion = authority.take_validated(binding, false);
    assert!(
        !matches!(completion, Err(Cause::Panic)),
        "ResourceAlt owner is ResourceID, not BlockID"
    );
    assert!(
        matches!(completion,Ok(Some(PreparedCompletion::Document(Outcome::Ready(_),receipt)))if receipt.joined)
    );
}

#[test]
fn alt_resource_id_inside_blocks_does_not_inherit_code_hscroll() {
    let raw = include_str!("../tests/fixtures/document-b2-resource-owner/code-owner.md");
    let d = parsed(raw);
    assert_eq!(d.resources().len(), 3);
    assert_eq!(d.blocks()[2].kind(), 10);
    let last = &d.resources()[2];
    assert_eq!(d.domains()[last.alt_domain() as usize].owner(), 2);
    use crate::{
        document_layout::Geometry,
        document_presentation_anchor::PresentationAnchor,
        document_text::DocumentWork,
        document_view::{DocumentPreparationAuthority, DocumentView, start_document},
        preparation_owner::{Outcome, PreparedCompletion},
    };
    let input = DocumentView::initial_input(
        d,
        Geometry {
            x: 0,
            y: 0,
            width: 80,
            height: 24,
            left: 200,
        },
    )
    .unwrap();
    let binding = input.key().binding;
    let mut authority = DocumentPreparationAuthority::default();
    start_document(input, &mut authority).unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !authority.is_finished() && Instant::now() < deadline {
        std::thread::yield_now()
    }
    let Some(PreparedCompletion::Document(Outcome::Ready(p), receipt)) =
        authority.take_validated(binding, false).unwrap()
    else {
        panic!("actual viewport")
    };
    assert!(receipt.joined);
    let v = DocumentView::from_prepared(p).unwrap();
    let t = v.layout().text().unwrap();
    let cancel = AtomicBool::new(false);
    let mut w = DocumentWork::preparation(&cancel);
    let mut found = false;
    for at in 0..t.glyphs().len() {
        if let Some(PresentationAnchor::ResourceAlt(anchor)) = t.cell_anchor(at, &mut w).unwrap()
            && anchor.resource() == 2
        {
            found = true;
        }
    }
    assert!(
        found,
        "paragraph ResourceAlt remains visible with Code block at same numerical ResourceID and horizontal offset200"
    );
}

fn variant_sources() -> [(&'static str, &'static str, u32); 4] {
    [
        (
            "Image",
            include_str!("../tests/fixtures/document-b2-resource-owner/images.md"),
            0,
        ),
        (
            "WikiEmbed",
            include_str!("../tests/fixtures/document-b2-resource-owner/wiki.md"),
            4,
        ),
        (
            "MathInline",
            include_str!("../tests/fixtures/document-b2-resource-owner/math.md"),
            2,
        ),
        (
            "Mermaid",
            include_str!("../tests/fixtures/document-b2-resource-owner/mermaid.md"),
            1,
        ),
    ]
}
#[test]
fn resource_variant_model_oracle() {
    for (name, raw, kind) in variant_sources() {
        let d = parsed(raw);
        assert_eq!(d.resources().len(), 3, "{name}");
        assert!(d.resources().iter().all(|r| r.kind() == kind));
        println!(
            "MODEL {name} blocks={} resources={}",
            d.blocks().len(),
            d.resources().len()
        );
        for r in d.resources() {
            println!(
                "RESOURCE id={} kind={} ownerRun={} alt={:?}",
                r.id(),
                r.kind(),
                r.owner_run(),
                d.domains().get(r.alt_domain() as usize).map(|a| (
                    a.id(),
                    a.kind(),
                    a.owner(),
                    a.run_count()
                ))
            );
        }
    }
}
#[test]
fn resource_variants_use_actual_parent_block_without_panic() {
    use crate::{
        document_layout::Geometry,
        document_view::{DocumentPreparationAuthority, DocumentView, start_document},
        preparation_owner::{Outcome, PreparedCompletion},
    };
    for (name, raw, _) in variant_sources() {
        let d = parsed(raw);
        let input = DocumentView::initial_input(
            d,
            Geometry {
                x: 0,
                y: 0,
                width: 80,
                height: 24,
                left: 0,
            },
        )
        .unwrap();
        let binding = input.key().binding;
        let mut authority = DocumentPreparationAuthority::default();
        start_document(input, &mut authority).unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while !authority.is_finished() && Instant::now() < deadline {
            std::thread::yield_now()
        }
        let completion = authority.take_validated(binding, false);
        assert!(
            matches!(completion,Ok(Some(PreparedCompletion::Document(Outcome::Ready(_),receipt)))if receipt.joined),
            "variant {name} must complete bounded real viewport without Panic"
        );
    }
}
