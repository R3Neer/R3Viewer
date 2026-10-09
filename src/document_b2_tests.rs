use super::*;
use crate::document_presentation_anchor::{PaintRole, PresentationAnchor};
fn g(w: u16, h: u16) -> Geometry {
    Geometry {
        x: 0,
        y: 0,
        width: w,
        height: h,
        left: 0,
    }
}
fn rows(v: &DocumentView) -> Vec<String> {
    let w = v.layout().text().expect("b2 text window");
    let mut cells =
        vec![vec![String::from(" "); v.geometry.width as usize]; v.geometry.height as usize];
    for glyph in w.glyphs().iter() {
        cells[glyph.row as usize][glyph.x as usize] =
            w.symbols()[glyph.start as usize..glyph.end as usize].to_owned();
        for dx in 1..glyph.width {
            cells[glyph.row as usize][(glyph.x + dx) as usize] = String::new();
        }
    }
    cells
        .into_iter()
        .map(|r| r.concat().trim_end().to_owned())
        .collect()
}
fn fixture(name: &str) -> String {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/document-b2")
            .join(name),
    )
    .unwrap()
}
#[test]
fn b2_heading_quote_tasks_roles() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (v, _) = view(&fixture("heading.md"), g(80, 24));
    let r = rows(&v);
    assert_eq!(r[0], "# a\u{301}");
    assert!(r.iter().any(|s| s == "###### H6"));
    let w = v.layout.text().unwrap();
    let content = w.glyphs().iter().find(|x| x.first.is_some()).unwrap();
    assert_eq!(content.styles & 2, 2);
    assert_eq!(w.paint(content).unwrap(), PaintRole::Heading);
    assert!(matches!(
        w.cell_anchor(0, &mut DocumentWork::frame(&AtomicBool::new(false)))
            .unwrap(),
        Some(PresentationAnchor::Role(_))
    ));
    let (mut v, mut a) = view(&fixture("quote.md"), g(8, 4));
    assert_eq!(&rows(&v)[..2], &["> abcdef", "> ghijk"]);
    perform(&mut v, &mut a, Operation::Line { delta: 1 }, g(8, 4));
    assert_eq!(rows(&v)[0], "> ghijk");
    let (v, _) = view(&fixture("tasks.md"), g(80, 24));
    assert_eq!(&rows(&v)[..3], &["3. one", "4. [x] two", "5. [ ] three"]);
}
#[test]
fn b2_items_order_and_callout_fold() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (v, _) = view(&fixture("order.md"), g(80, 24));
    let s = rows(&v).join("\n");
    let mut prior = 0;
    for needle in ["BEFORE", "INNER", "AFTER", "TAIL"] {
        let at = s.find(needle).unwrap();
        assert!(at >= prior);
        prior = at;
    }
    let (mut v, mut a) = view(&fixture("callout.md"), g(80, 24));
    let s = rows(&v).join("\n");
    for needle in [
        "[callout note +] Closed",
        "[callout tip -] Open",
        "| BODY",
        "TAIL",
    ] {
        assert!(s.contains(needle), "{needle} absent in {s:?}");
    }
    assert!(!s.contains("HIDDEN"));
    let block = v
        .document
        .blocks()
        .iter()
        .find(|b| b.kind() == 4)
        .unwrap()
        .id();
    perform(
        &mut v,
        &mut a,
        Operation::Fold {
            block,
            expanded: true,
        },
        g(80, 24),
    );
    assert!(rows(&v).join("\n").contains("HIDDEN"));
}
#[test]
fn b2_table_vertical_columns_and_alignment() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (v, _) = view(&fixture("table.md"), g(18, 24));
    assert_eq!(
        &rows(&v)[..11],
        &[
            "column 1: A",
            "column 2: B",
            "column 3: C",
            "",
            "column 1: a1",
            "column 2: b1",
            "column 3: c1",
            "",
            "column 1: a2",
            "column 2: b2",
            "column 3: c2"
        ]
    );
    let (v, _) = view(&fixture("table.md"), g(80, 24));
    let w = v.layout.text().unwrap();
    for (text, x, row) in [
        ("A", 0, 0),
        ("B", 40, 0),
        ("C", 79, 0),
        ("a", 0, 2),
        ("b", 39, 2),
        ("c", 78, 2),
    ] {
        assert!(
            w.glyphs().iter().any(|c| c.x == x
                && c.row == row
                && &w.symbols()[c.start as usize..c.end as usize] == text),
            "{text}@{x},{row}"
        );
    }
}
#[test]
fn b2_code_inert_footnote_frontmatter() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    for (name, needles) in [
        (
            "code.md",
            vec!["[code: rust]", "let x = 1;", "  TAB", "TAIL"],
        ),
        (
            "inert.md",
            vec!["<div onclick=\"EXEC\">literal</div>", "TAIL"],
        ),
        ("frontmatter.md", vec!["a: 1", "TAIL"]),
        ("footnote.md", vec!["---", "[^n]: foot", "TAIL"]),
    ] {
        let (v, _) = view(&fixture(name), g(80, 24));
        let r = rows(&v).join("\n");
        for needle in needles {
            assert!(r.contains(needle), "{name}: absent {needle} in {r:?}");
        }
    }
}
#[test]
fn b2_resource_lanes_and_safe_controls() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (v, _) = view(&fixture("image.md"), g(80, 24));
    let r = rows(&v).join("\n");
    assert!(
        r.contains("[Image pending: image.png; alt: ALT; title: TITLE]"),
        "{r:?}"
    );
    assert!(r.contains("TAIL"));
    let (v, _) = view(&fixture("controls.md"), g(80, 24));
    let r = rows(&v).join("\n");
    assert!(!r.contains('\u{1b}') && !r.contains('\u{7}'));
    assert!(r.contains('␛'));
    assert!(r.contains('␇'));
    let (v, _) = view(&fixture("resources.md"), g(80, 24));
    let r = rows(&v).join("\n");
    for needle in [
        "[Math pending: x+1]",
        "[Math pending: x^2]",
        "[Mermaid pending:",
        "graph TD; A-->B;",
        "[Wiki embed pending: Missing]",
        "TAIL",
    ] {
        assert!(r.contains(needle), "absent {needle} in {r:?}");
    }
}
#[test]
fn b2_adaptive_classes_and_dense() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (v, _) = view(&fixture("class-growth.md"), g(80, 24));
    assert_eq!(symbols(&v), vec!["x"; 220].join(" "));
    assert!(v.layout.bytes() <= 131072);
    let source = "X".repeat(120) + "  \n";
    let source = source.repeat(40);
    for (width, height, glyphs) in [(80, 24, 1440), (120, 40, 4800)] {
        let (v, _) = view(&source, g(width, height));
        let w = v.layout.text().expect("dense stays Text");
        assert_eq!(w.glyphs().len(), glyphs);
        assert_eq!(w.rows().len(), height as usize);
        assert!(w.bytes() <= 131072);
        assert!(w.work + w.frame_receipt().render_capacity() <= 8388608);
        eprintln!(
            "B2 DENSE {width}x{height} window={} layout={} held={} consumer={} peak={}",
            w.bytes(),
            w.work,
            w.frame_receipt().render_capacity(),
            v.budget.used(),
            v.budget.peak()
        );
    }
}

#[test]
fn b2_table_resume_keeps_logical_positions_on_resize() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (mut v, mut a) = view(&fixture("table-resize.md"), g(30, 24));
    perform(&mut v, &mut a, Operation::Line { delta: 3 }, g(30, 24));
    let Some(PresentationAnchor::Table(t)) = v.presentation_anchor() else {
        panic!("TableResume")
    };
    let before = t.cells().to_vec();
    assert_eq!(before.len(), 3);
    assert!(
        matches!(before[0].progress,crate::document_presentation_anchor::CellProgress::At(x)if x.raw.byte==8)
    );
    assert!(matches!(
        before[1].progress,
        crate::document_presentation_anchor::CellProgress::Done
    ));
    assert!(
        matches!(before[2].progress,crate::document_presentation_anchor::CellProgress::At(x)if x.raw.byte==8)
    );
    perform(&mut v, &mut a, Operation::Resize, g(66, 24));
    let Some(PresentationAnchor::Table(t)) = v.presentation_anchor() else {
        panic!("resized TableResume")
    };
    assert_eq!(t.cells(), before.as_slice());
    let w = v.layout.text().unwrap();
    // The frozen oracle is logical text. Read only emitted glyphs of each actual ownerCell;
    // width20 intentionally wraps the 22-byte third-cell suffix over two visual rows.
    for (cell, expected) in [
        (before[0].cell, "ijklmnopqrstuvwxyz"),
        (before[2].cell, "8901234567890123456789"),
    ] {
        let stream: String = w
            .glyphs()
            .iter()
            .filter(|c| c.first.is_some_and(|f| f.block == cell))
            .map(|c| &w.symbols()[c.start as usize..c.end as usize])
            .collect();
        assert_eq!(stream, expected);
    }
    assert!(
        !w.glyphs()
            .iter()
            .any(|c| c.first.is_some_and(|f| f.block == before[1].cell))
    );
    let s = rows(&v).join("\n");
    for needle in ["ROW2A", "ROW2B", "ROW2C", "TAIL"] {
        assert!(s.contains(needle), "lost {needle} in {s:?}");
    }
}
#[test]
fn b2_long_block_refs_continue_to_tail() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (mut v, mut a) = view(&fixture("role-ref-resume.md"), g(12, 5));
    let mut ref_text = String::new();
    let mut body = String::new();
    let mut pages = 0;
    loop {
        let w = v.layout.text().expect("refs stay drawable Text");
        for c in w.glyphs().iter() {
            let s = &w.symbols()[c.start as usize..c.end as usize];
            match c.kind {
                crate::document_layout::CellKind::RefText(_) => ref_text.push_str(s),
                _ if c.first.is_some() => body.push_str(s),
                _ => {}
            }
        }
        pages += 1;
        eprintln!(
            "REF page={pages} refs={ref_text:?} body={body:?} rows={:?}",
            rows(&v)
        );
        if body.contains("TAIL") {
            break;
        }
        assert!(pages < 20, "continuation progresses");
        assert!(
            w.presentation_next().is_some(),
            "typed next required before TAIL"
        );
        perform(&mut v, &mut a, Operation::Line { delta: 5 }, g(12, 5));
    }
    for needle in [
        "averylongcalloutkind",
        "verylonglanguage extra-long-info",
        "averylongfootnotelabel",
    ] {
        assert!(ref_text.contains(needle), "lost {needle} in {ref_text:?}");
    }
    for needle in ["Title", "BODY", "CODE", "FOOT", "TAIL"] {
        assert!(body.contains(needle), "lost {needle} in {body:?}");
    }
}
#[test]
fn b2_horizontal_is_logical_and_cluster_safe() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (mut v, mut a) = view("```\nab界cd\tZ\n```\n", g(8, 4));
    let code = v.document.blocks().iter().find(|b| b.kind() == 10).unwrap();
    let d = &v.document.domains()[code.domain() as usize];
    let anchor = RawAnchor {
        block: code.id(),
        run: d.first_run(),
        byte: 0,
    };
    perform(&mut v, &mut a, Operation::Seek(anchor), g(8, 4));
    let mut geometry = g(8, 4);
    geometry.x = 7;
    geometry.left = 3;
    perform(&mut v, &mut a, Operation::Horizontal { left: 3 }, geometry);
    let w = v.layout.text().unwrap();
    assert_eq!(w.symbols(), "cd  Z");
    for (text, x) in [("c", 1), ("d", 2), ("Z", 5)] {
        assert!(
            w.glyphs()
                .iter()
                .any(|c| c.x == x && &w.symbols()[c.start as usize..c.end as usize] == text)
        );
    }
    assert!(!w.symbols().contains('界'));
}

#[test]
fn b2_ref_and_alt_small_windows_keep_each_lane() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let source = "![abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ](i.png \"012345678901234567890123456789\")\n\nTAIL\n";
    let (mut v, mut a) = view(source, g(8, 1));
    let mut alt = String::new();
    let mut title = String::new();
    let mut input = String::new();
    let mut body = String::new();
    let mut pages = 0;
    loop {
        let w = v.layout.text().expect("safe small resource pane");
        let cancel = AtomicBool::new(false);
        let mut work = DocumentWork::frame(&cancel);
        for (i, c) in w.glyphs().iter().enumerate() {
            let text = &w.symbols()[c.start as usize..c.end as usize];
            match w.cell_anchor(i, &mut work).unwrap() {
                Some(PresentationAnchor::ResourceAlt(_)) => alt.push_str(text),
                Some(PresentationAnchor::Ref(r)) => match r.field() {
                    crate::document_presentation_anchor::RefField::Input => input.push_str(text),
                    crate::document_presentation_anchor::RefField::Title => title.push_str(text),
                    _ => {}
                },
                Some(PresentationAnchor::Domain(_)) => body.push_str(text),
                _ => {}
            }
        }
        pages += 1;
        if body.contains("TAIL") {
            break;
        }
        assert!(
            pages < 30,
            "resource progress: input={input:?} alt={alt:?} title={title:?} body={body:?}"
        );
        assert!(w.presentation_next().is_some(), "typed resource next");
        perform(&mut v, &mut a, Operation::Line { delta: 1 }, g(8, 1));
    }
    assert_eq!(input, "i.png");
    assert_eq!(alt, "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ");
    assert_eq!(title, "012345678901234567890123456789");
    assert_eq!(body, "TAIL");
}
#[test]
fn b2_role_ref_pagination_crosses_provider_not_only_block() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (mut v, mut a) = view(&fixture("role-ref-resume.md"), g(12, 2));
    let mut refs = String::new();
    let mut body = String::new();
    let mut count = 0;
    loop {
        let w = v.layout.text().unwrap();
        for c in w.glyphs().iter() {
            let s = &w.symbols()[c.start as usize..c.end as usize];
            if matches!(c.kind, crate::document_layout::CellKind::RefText(_)) {
                refs.push_str(s);
            } else if c.first.is_some() {
                body.push_str(s);
            }
        }
        count += 1;
        if body.contains("TAIL") {
            break;
        }
        assert!(count < 30, "Ref progress");
        assert!(w.presentation_next().is_some());
        perform(&mut v, &mut a, Operation::Line { delta: 2 }, g(12, 2));
    }
    assert_eq!(
        refs,
        "averylongcalloutkindverylonglanguage extra-long-infoaverylongfootnotelabel"
    );
    assert_eq!(body, "TitleBODYCODEFOOTTAIL");
}
#[test]
fn b2_short_table_windows_complete_all_cells_rows_and_tail() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let source = fixture("table-resize.md");
    for width in [18, 30] {
        let (mut v, mut a) = view(&source, g(width, 2));
        let mut owners = std::collections::BTreeMap::<u32, String>::new();
        let mut count = 0;
        loop {
            let w = v.layout.text().unwrap();
            for c in w.glyphs().iter() {
                if let Some(f) = c.first {
                    owners
                        .entry(f.block)
                        .or_default()
                        .push_str(&w.symbols()[c.start as usize..c.end as usize]);
                }
            }
            count += 1;
            if owners.values().any(|v| v == "TAIL") {
                break;
            }
            assert!(count < 40, "table progress {width}: {owners:?}");
            assert!(w.presentation_next().is_some(), "table next {width}");
            perform(&mut v, &mut a, Operation::Line { delta: 2 }, g(width, 2));
        }
        for s in [
            "A",
            "B",
            "C",
            "abcdefghijklmnopqrstuvwxyz",
            "xy",
            "012345678901234567890123456789",
            "ROW2A",
            "ROW2B",
            "ROW2C",
            "TAIL",
        ] {
            assert_eq!(
                owners.values().filter(|v| v.as_str() == s).count(),
                1,
                "each owner emits complete text once {s} at {width}: {owners:?}"
            );
        }
    }
}

#[test]
fn b2_separator_line_preserves_role_anchor() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (mut v, mut a) = view(&fixture("table.md"), g(80, 24));
    perform(&mut v, &mut a, Operation::Line { delta: 1 }, g(80, 24));
    let r = rows(&v);
    assert!(r[0].chars().all(|c| c == '-' || c == '+'));
    assert_eq!(r[0].len(), 80);
    for needle in ["a1", "b1", "c1"] {
        assert!(r[1].contains(needle), "{r:?}");
    }
    assert!(!r[0].contains('A'));
    let w = v.layout.text().unwrap();
    assert!(matches!(
        w.presentation_row(0),
        Some(PresentationAnchor::Role(_))
    ));
    assert!(
        w.glyphs()
            .iter()
            .filter(|c| c.row == 0)
            .all(|c| c.first.is_none())
    );
}
