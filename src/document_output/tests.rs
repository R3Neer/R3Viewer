use super::*;
use clap::Parser;
fn executable() -> PathBuf {
    std::env::var_os("R3VIEWER_DOCUMENT_TEST_EXE")
        .expect("explicit private real worker required")
        .into()
}
pub(crate) fn config() -> PresentationConfig {
    let path =
        std::env::var_os("R3VIEWER_DOCUMENT_TEST_CONFIG").expect("explicit owned config required");
    let cli = crate::cli::Cli::parse_from(["viewer".into(), "--config".into(), path]);
    PresentationConfig::load(&cli, false, Arc::new(AtomicBool::new(false))).unwrap()
}
fn run(
    source: &str,
    exe: PathBuf,
    origin: SourceOrigin,
    limit: usize,
) -> Result<PreparedOutput, Error> {
    prepare(
        exe,
        Arc::new(source.to_owned()),
        origin,
        80,
        &config(),
        Arc::new(AtomicBool::new(false)),
        limit,
    )
}
#[test]
fn co01_real_worker_heading_list_complete() {
    let p = run(
        "# T\n\n- A\n",
        executable(),
        SourceOrigin::Literal,
        OUTPUT_LIMIT,
    )
    .unwrap();
    assert_eq!(p.mode, Mode::Document);
    assert!(p.content.contains("# T\n"));
    assert!(p.content.contains("- A\n"));
    assert!(p.diagnostic.is_none());
}
#[test]
fn co02_explicit_markdown_json_and_empty_not_data() {
    let p = run(
        "{\"a\":1}\n",
        executable(),
        SourceOrigin::Nushell,
        OUTPUT_LIMIT,
    )
    .unwrap();
    assert_eq!(p.mode, Mode::Document);
    assert_eq!(p.content, "{\"a\":1}\n");
    let p = run("", executable(), SourceOrigin::StandardInput, OUTPUT_LIMIT).unwrap();
    assert_eq!(p.mode, Mode::Document);
    assert!(p.content.is_empty());
}
#[test]
fn co03_original_origin_never_reopens_and_arc_is_preserved() {
    let original = Arc::new("# Original\n".to_owned());
    let pointer = Arc::as_ptr(&original);
    let missing = std::env::var_os("R3VIEWER_DOCUMENT_TEST_CONFIG")
        .map(PathBuf::from)
        .unwrap()
        .with_extension("absent.md");
    assert!(!missing.exists());
    let p = prepare(
        executable(),
        original.clone(),
        SourceOrigin::LocalFile(missing),
        80,
        &config(),
        Arc::new(AtomicBool::new(false)),
        OUTPUT_LIMIT,
    )
    .unwrap();
    assert_eq!(p.mode, Mode::Document);
    assert_eq!(p.content, "# Original\n");
    assert_eq!(Arc::as_ptr(&original), pointer);
    assert_eq!(Arc::strong_count(&original), 1);
    assert_eq!(&**original, "# Original\n");
}
#[test]
fn co05_real_controls_and_decoded_entity_safe_complete() {
    let p = run(
        "A\u{1b}B &amp; Z\n",
        executable(),
        SourceOrigin::Literal,
        OUTPUT_LIMIT,
    )
    .unwrap();
    assert_eq!(p.content, "A␛B & Z\n");
    assert!(!p.content.contains('\u{1b}'));
}
#[test]
fn co06_missing_worker_closed_full_source_and_static_diagnostic() {
    let missing = executable().with_file_name("own-worker-does-not-exist.exe");
    assert!(!missing.exists());
    let p = run(
        "# T\nA\u{1b}B\n",
        missing,
        SourceOrigin::Literal,
        OUTPUT_LIMIT,
    )
    .unwrap();
    assert_eq!(p.mode, Mode::Source);
    assert_eq!(p.content, "# T\nA␛B\n");
    assert_eq!(
        p.diagnostic,
        Some("no se pudo preparar Documento; se muestra Fuente completa: fallo del worker")
    );
    assert!(!DocumentController::new().pending_cleanup());
}
#[test]
fn co07_exact_output_quota_and_minus_one_no_partial() {
    let p = run("hello\n", executable(), SourceOrigin::Literal, 6).unwrap();
    assert_eq!(p.content, "hello\n");
    let e = run("hello\n", executable(), SourceOrigin::Literal, 5).unwrap_err();
    assert_eq!(e.kind, ErrorKind::Limit);
    assert!(!e.cleanup_pending);
    assert_eq!(e.exit_code(), 1);
}
#[test]
fn co08_early_cancel_no_prepared_content() {
    let e = prepare(
        executable(),
        Arc::new("# T\n".into()),
        SourceOrigin::Literal,
        80,
        &config(),
        Arc::new(AtomicBool::new(true)),
        OUTPUT_LIMIT,
    )
    .unwrap_err();
    assert_eq!(e.kind, ErrorKind::Cancelled);
    assert_eq!(e.exit_code(), 130);
    assert!(!e.cleanup_pending);
}
#[test]
fn co13_real_wire_record_fixture_preserves_typed_input() {
    use r3viewer_wire::{Frame, Origin, Value};
    let value = Value::Record(vec![("a".into(), Value::Int(7))]);
    let mut wire = Vec::new();
    r3viewer_wire::write_magic(&mut wire).unwrap();
    r3viewer_wire::write_frame(
        &mut wire,
        &Frame::Start {
            version: r3viewer_wire::VERSION,
            cwd: "metadata-only/notas.md".into(),
            origin: Origin::SingleValue,
        },
    )
    .unwrap();
    r3viewer_wire::write_frame(&mut wire, &Frame::Value(value.clone())).unwrap();
    r3viewer_wire::write_frame(&mut wire, &Frame::End).unwrap();
    let mut decoder = r3viewer_wire::Reader::new(
        std::io::Cursor::new(&wire),
        Arc::new(AtomicBool::new(false)),
    );
    decoder.read_start().unwrap();
    let decoded = decoder.finish().unwrap();
    assert_eq!(decoded.payload, r3viewer_wire::Payload::SingleValue(value));
    let cfg = std::env::var_os("R3VIEWER_DOCUMENT_TEST_CONFIG").unwrap();
    let directory = PathBuf::from(cfg).parent().unwrap().to_path_buf();
    std::fs::write(directory.join("CO13-record.wire"), wire).unwrap();
}
