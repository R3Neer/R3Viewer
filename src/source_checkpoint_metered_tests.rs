use super::*;
#[test]
fn checkpoint01_common_probes_same_work_partial_c1_and_legacy_outputs() {
    let text = SafeTextModel::new(
        include_str!("../tests/fixtures/source-common/checkpoint-selection.txt").to_owned(),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(text.checkpoints.len(), 4);
    let cancel = AtomicBool::new(false);
    let mut work = Work::frame(&cancel);
    work.admit(9).unwrap();
    let before = work.used();
    let mut cursor = text.provenance_cursor_with(4201, &mut work).unwrap();
    // upper_bound with 4 entries probes indices 2 then 1; selected record is entry 1.
    let expected = 3 * std::mem::size_of::<usize>()
        + 2 * (std::mem::size_of::<(usize, usize)>() + 1)
        + std::mem::size_of::<(usize, usize)>()
        + std::mem::size_of::<ProvenanceCursor<'_>>();
    assert_eq!(work.used() - before, expected);
    assert_eq!(
        cursor.source_range_with(4201..4202, &mut work).unwrap(),
        4200..4202
    );
    assert_eq!(
        cursor.source_range_with(4202..4208, &mut work).unwrap(),
        4200..4202
    );
    assert!(work.used() > before + expected);
    for raw in [0..0, 4200..4202, 8403..8407] {
        let visible = text
            .visible_range_metered_with(raw.clone(), &mut work)
            .unwrap();
        assert_eq!(visible, text.visible_range(raw));
        assert_eq!(
            text.source_range_metered_with(visible.clone(), &mut work)
                .unwrap(),
            text.source_range(visible)
        );
    }
    assert_eq!(
        text.visible_range_metered_with(4200..4202, &mut work)
            .unwrap(),
        4200..4208
    );
    let mut end = text
        .provenance_cursor_with(text.visible().len() - 1, &mut work)
        .unwrap();
    assert_eq!(
        end.source_range_with(text.visible().len() - 1..text.visible().len(), &mut work)
            .unwrap(),
        text.original().len() - 1..text.original().len()
    );
    eprintln!(
        "SC-W01 lookup={} cursor={} sameWork={}",
        expected,
        std::mem::size_of::<ProvenanceCursor<'_>>(),
        work.used()
    );
}
#[test]
fn checkpoint02_cancel_and_remaining_minus_one_no_partial_no_reset() {
    let text = SafeTextModel::new(
        include_str!("../tests/fixtures/source-common/checkpoint-selection.txt").to_owned(),
        &AtomicBool::new(false),
    )
    .unwrap();
    let cancelled = AtomicBool::new(true);
    let mut work = Work::frame(&cancelled);
    assert!(matches!(
        text.provenance_cursor_with(4201, &mut work),
        Err(PrepareError::Cancelled)
    ));
    assert_eq!(work.used(), 0);
    let cancel = AtomicBool::new(false);
    let exact = 3 * std::mem::size_of::<usize>()
        + 2 * (std::mem::size_of::<(usize, usize)>() + 1)
        + std::mem::size_of::<(usize, usize)>()
        + std::mem::size_of::<ProvenanceCursor<'_>>();
    let mut work = Work::permit(&cancel, exact - 1);
    work.admit(0).unwrap();
    assert!(matches!(
        text.provenance_cursor_with(4201, &mut work),
        Err(PrepareError::Limit(_))
    ));
    assert_eq!(
        work.used(),
        exact - std::mem::size_of::<ProvenanceCursor<'_>>()
    );
    assert!(work.used() > 0);
    let mut accepted = Work::permit(&cancel, exact);
    assert!(text.provenance_cursor_with(4201, &mut accepted).is_ok());
    assert_eq!(accepted.used(), exact);
    let mut short = Work::permit(
        &cancel,
        3 * std::mem::size_of::<usize>() + std::mem::size_of::<(usize, usize)>(),
    );
    assert!(matches!(
        text.visible_range_metered_with(4200..4202, &mut short),
        Err(PrepareError::Limit(_))
    ));
    assert_eq!(short.used(), 40);
}

#[test]
fn checkpoint03_index_openings_zero_permit_cancel_and_exact_boundary() {
    let text = SafeTextModel::new(
        include_str!("../tests/fixtures/source-common/checkpoint-selection.txt").to_owned(),
        &AtomicBool::new(false),
    )
    .unwrap();
    let cancel = AtomicBool::new(false);
    let mut zero = Work::permit(&cancel, 0);
    assert!(matches!(
        text.viewport_cursor_metered(0, 8398, &mut zero),
        Err(PrepareError::Limit(_))
    ));
    assert_eq!(zero.used(), 0);
    assert!(matches!(
        text.cell_for_offset_metered_with(4201, &mut zero),
        Err(PrepareError::Limit(_))
    ));
    let mut observed = Work::frame(&cancel);
    let point = text
        .viewport_cursor_metered(0, 8398, &mut observed)
        .unwrap();
    let cost = observed.used();
    assert!(cost > 0);
    let legacy = text
        .viewport_cursor(0, 8398, &mut Work::preparation(&cancel))
        .unwrap();
    assert_eq!(
        (point.offset(), point.line(), point.column()),
        (legacy.offset(), legacy.line(), legacy.column())
    );
    let mut short = Work::permit(&cancel, cost - 1);
    assert!(matches!(
        text.viewport_cursor_metered(0, 8398, &mut short),
        Err(PrepareError::Limit(_))
    ));
    assert!(short.used() > 0);
    let mut exact = Work::permit(&cancel, cost);
    assert!(text.viewport_cursor_metered(0, 8398, &mut exact).is_ok());
    assert_eq!(exact.used(), cost);
    let stop = AtomicBool::new(true);
    let mut cancelled = Work::frame(&stop);
    assert!(matches!(
        text.viewport_cursor_metered(0, 8398, &mut cancelled),
        Err(PrepareError::Cancelled)
    ));
    assert_eq!(cancelled.used(), 0);
    eprintln!(
        "SC-W01 Index opening exact={} Point={}",
        cost,
        std::mem::size_of::<text_engine::Point>()
    );
}
