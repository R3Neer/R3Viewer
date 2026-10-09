//! Model-only fixtures and oracles. No inspector/search candidate used to derive expected values.
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
pub(super) fn cluster65(decoded_last: bool) -> Arc<Document> {
    let raw = format!("a{}", "\u{0301}".repeat(64));
    let mut c = candidate(&raw);
    assert_eq!(c.domains.len(), 1);
    assert_eq!(c.runs.len(), 1);
    let owner = c.domains[0].owner();
    let root_item = c.items[0];
    c.items.truncate(0);
    c.items.push(root_item).unwrap();
    c.runs.truncate(0);
    c.texts.truncate(0);
    let mut dw = *c.domains[0].words();
    dw[3] = 0;
    dw[4] = 65;
    c.domains[0] = Domain::from_words(dw);
    let mut bw = *c.blocks[owner as usize].words();
    bw[2] = 1;
    bw[3] = 65;
    c.blocks[owner as usize] = Block::from_words(bw);
    for i in 0..65u32 {
        let start = if i == 0 { 0 } else { 1 + (i - 1) * 2 };
        let end = if i == 0 { 1 } else { start + 2 };
        let oid = c.origins.len() as u32;
        c.origins
            .push(Origin::from_words([
                u32::from(decoded_last && i == 64),
                start,
                end,
                0,
            ]))
            .unwrap();
        let text = if decoded_last && i == 64 {
            c.decoded.extend_from_slice("\u{0301}".as_bytes()).unwrap();
            TextRef::from_words([1, 0, 2, oid])
        } else {
            TextRef::from_words([0, start, end, oid])
        };
        c.texts.push(text).unwrap();
        c.runs
            .push(Run::from_words([
                i, 0, i, 0, 0, i, oid, NONE, NONE, 0, 0, 0, 0, 0, 0, 0,
            ]))
            .unwrap();
        c.items.push(Item::from_words([owner, i, 1, i])).unwrap();
    }
    checked(c)
}
pub(super) fn shared_titles() -> Arc<Document> {
    let mut c = candidate("![fox](one \"same\") ![fox](two \"same\")");
    let id = c.resources[0].title_decoded();
    let mut words = *c.resources[1].words();
    words[5] = id;
    c.resources[1] = Resource::from_words(words);
    checked(c)
}
pub(super) fn sources() -> Vec<(&'static str, Arc<Document>)> {
    vec![
        ("styled", parsed("a*\u{0301}*")),
        ("links", parsed("[a](one)[\u{0301}](two)")),
        ("entity", parsed("A&amp;B")),
        ("controls", parsed("\u{001b}\u{001b}\u{009b}")),
        ("multi", parsed("ab")),
        ("shared", shared_titles()),
        ("exact65", cluster65(false)),
        ("mixed65", cluster65(true)),
        (
            "links65",
            parsed(
                &(0..65)
                    .map(|i| format!("[{}](t{i})", if i == 0 { "a" } else { "\u{0301}" }))
                    .collect::<String>(),
            ),
        ),
        ("role", parsed("---\n")),
        ("table", parsed("| a | b |\n| :- | -: |\n| c | d |\n")),
        (
            "resource",
            parsed("![fox](one \"title\")\n\n> [!note] title\n> text\n"),
        ),
    ]
}
#[test]
fn inspector_model_oracle_before_runtime() {
    use std::fmt::Write;
    let mut out = String::from("MODEL ONLY: real parser/quarantine before inspector runtime\n");
    for (name, d) in sources() {
        writeln!(
            out,
            "{name} source={:?} hash={:02x?}",
            d.source().text(),
            d.source().hash()
        )
        .unwrap();
        for b in d.blocks() {
            writeln!(out, "block {:?}", b.words()).unwrap();
        }
        for x in d.domains() {
            writeln!(out, "domain {:?}", x.words()).unwrap();
        }
        for x in d.items() {
            writeln!(out, "item {:?}", x.words()).unwrap();
        }
        for x in d.runs() {
            writeln!(out, "run {:?}", x.words()).unwrap();
        }
        for x in d.text_refs() {
            writeln!(out, "ref {:?}", x.words()).unwrap();
        }
        for x in d.origins() {
            writeln!(out, "origin {:?}", x.words()).unwrap();
        }
        for x in d.links() {
            writeln!(out, "link {:?}", x.words()).unwrap();
        }
        for x in d.resources() {
            writeln!(out, "resource {:?}", x.words()).unwrap();
        }
    }
    assert_eq!(
        out,
        include_str!("../tests/fixtures/document-b3-inspector/MODEL-ORACLE.txt")
    );
}

pub(super) fn empty_title() -> Arc<Document> {
    let mut c = candidate("![fox](one \"\")");
    assert_eq!(c.resources[0].title_decoded(), NONE);
    let origin = c.origins.len() as u32;
    c.origins.push(Origin::from_words([1, 11, 13, 0])).unwrap();
    let id = c.texts.len() as u32;
    c.texts
        .push(TextRef::from_words([1, 0, 0, origin]))
        .unwrap();
    let mut words = *c.resources[0].words();
    words[5] = id;
    c.resources[0] = Resource::from_words(words);
    checked(c)
}
#[test]
fn inspector_empty_title_model_oracle_before_runtime() {
    let d = empty_title();
    use std::fmt::Write;
    let mut records = String::from(
        "MODEL ONLY empty Title: Candidate/quarantine before corrected empty inspector runtime\n",
    );
    writeln!(
        records,
        "source={:?} hash={:02x?}",
        d.source().text(),
        d.source().hash()
    )
    .unwrap();
    for r in d.resources() {
        writeln!(records, "resource {:?}", r.words()).unwrap();
    }
    for r in d.text_refs() {
        writeln!(records, "ref {:?}", r.words()).unwrap();
    }
    for r in d.origins() {
        writeln!(records, "origin {:?}", r.words()).unwrap();
    }
    assert_eq!(d.text(d.resources()[0].title_decoded()).unwrap(), "");
    assert_eq!(
        records,
        include_str!("../tests/fixtures/document-b3-inspector/MODEL-EMPTY-TITLE.txt")
    );
}

#[test]
fn provider_r2_model_only_before_runtime() {
    use std::fmt::Write;
    let d = parsed("![bee](pin \"hold\") ![fox](one \"same\") ![fox](two \"same\")");
    let mut out = String::from("MODEL ONLY providers R2 before scoped admission runtime\n");
    writeln!(
        out,
        "source={:?} hash={:02x?}",
        d.source().text(),
        d.source().hash()
    )
    .unwrap();
    assert_eq!(d.resources().len(), 3);
    for (index, r) in d.resources().iter().enumerate() {
        assert_eq!(r.id() as usize, index);
        assert_eq!(
            d.text(r.title_decoded()).unwrap(),
            if index == 0 { "hold" } else { "same" }
        );
        writeln!(out, "resource {:?}", r.words()).unwrap();
    }
    for r in d.text_refs() {
        writeln!(out, "ref {:?}", r.words()).unwrap();
    }
    for r in d.origins() {
        writeln!(out, "origin {:?}", r.words()).unwrap();
    }
    for r in d.runs() {
        writeln!(out, "run {:?}", r.words()).unwrap();
    }
    assert_eq!(
        out,
        include_str!("../tests/fixtures/document-b3-inspector/MODEL-PROVIDERS-R2.txt")
    );
}
