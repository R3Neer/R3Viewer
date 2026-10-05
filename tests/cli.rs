use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);

fn invoke(args: &[&str], input: Option<&[u8]>) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_r3viewer"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(bytes) = input {
        child.stdin.as_mut().unwrap().write_all(bytes).unwrap();
    }
    drop(child.stdin.take());
    child.wait_with_output().unwrap()
}
struct Fixture(PathBuf);
impl Fixture {
    fn new(bytes: &[u8]) -> Self {
        let path = std::env::temp_dir().join(format!(
            "r3viewer-s01-{}-{} con espacios.txt",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::write(&path, bytes).unwrap();
        Self(path)
    }
    fn path(&self) -> &str {
        self.0.to_str().unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

#[test]
fn redirected_file_keeps_unicode_crlf_and_trailing_blank_lines() {
    let source = "á界🙂\t alineado\r\n\r\nfin\n\n";
    let file = Fixture::new(source.as_bytes());
    let normal = invoke(&[file.path()], None);
    let inline = invoke(&["-i", file.path()], None);
    assert!(normal.status.success(), "{:?}", normal.stderr);
    assert_eq!(normal.stdout, source.as_bytes());
    assert_eq!(normal.stdout, inline.stdout);
    assert!(normal.stderr.is_empty());
}
#[test]
fn pipeline_and_value_are_literal_never_reopen_a_path_or_url() {
    let file = Fixture::new(b"secret source");
    for literal in [file.path(), "https://example.invalid/private", ""] {
        for args in [&[][..], &["-v"][..]] {
            let result = invoke(args, Some(literal.as_bytes()));
            assert!(result.status.success());
            assert_eq!(result.stdout, literal.as_bytes());
        }
        let result = invoke(&["--value", literal], None);
        assert!(result.status.success());
        assert_eq!(result.stdout, literal.as_bytes());
    }
}
#[test]
fn control_bytes_cannot_escape_redirected_output() {
    let result = invoke(&["-v", "\x1b]52;c;payload\x07\u{009b}31m\rreplace"], None);
    let output = String::from_utf8(result.stdout).unwrap();
    assert!(result.status.success());
    assert!(!output.chars().any(char::is_control));
    assert!(output.contains("␛]52;c;payload␇"));
}
#[test]
fn conflicting_sources_and_flags_are_cli_errors() {
    for args in [
        &["-pv", "literal"][..],
        &["-v", "arg"][..],
        &["-p"][..],
        &["--format", "html"][..],
        &["one", "two"][..],
    ] {
        let result = invoke(args, Some(b"pipeline"));
        assert_eq!(
            result.status.code(),
            Some(2),
            "args {args:?}: {:?}",
            result.stderr
        );
        assert!(result.stdout.is_empty());
        assert!(!result.stderr.is_empty());
    }
}
#[test]
fn io_and_encoding_failures_have_no_content_output() {
    let invalid = Fixture::new(&[0xff, 0xfe]);
    for source in [
        invalid.path(),
        "missing-r3viewer-test-unique-file",
        "https://example.invalid/",
    ] {
        let result = invoke(&[source], None);
        assert_eq!(result.status.code(), Some(1));
        assert!(result.stdout.is_empty());
        assert!(!result.stderr.is_empty());
    }
}
#[test]
fn path_flag_does_not_authorize_url_fetch() {
    let result = invoke(&["--path", "https://example.invalid/"], None);
    assert_eq!(result.status.code(), Some(1));
    let message = String::from_utf8(result.stderr).unwrap();
    assert!(message.contains("no se puede abrir"));
}

#[test]
fn invalid_configuration_is_reported_before_opening_input() {
    let config = Fixture::new(b"colur = 'always'\n");
    let result = invoke(
        &[
            "--config",
            config.path(),
            "absent-document-that-must-not-open.txt",
        ],
        None,
    );
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stdout.is_empty());
    let stderr = String::from_utf8(result.stderr).unwrap();
    assert!(stderr.contains("configuración"));
    assert!(!stderr.contains("no se puede abrir \"absent-document"));
    let result = invoke(&["--config", config.path(), "-v", "text"], None);
    assert_eq!(result.status.code(), Some(2));
}
#[test]
fn redirected_colour_and_theme_never_emit_terminal_styles() {
    let config = Fixture::new(
        b"colour='always'\ntheme='light'\nline_numbers=true\n[colours]\nvalue='#123456'\n",
    );
    let original = "// comment\r\nlet x = \"🦊\";\n";
    let result = invoke(
        &[
            "-v",
            original,
            "--config",
            config.path(),
            "--colour",
            "always",
            "--theme",
            "dark",
            "--line-numbers",
        ],
        None,
    );
    assert!(result.status.success(), "{:?}", result.stderr);
    assert_eq!(result.stdout, original.as_bytes());
    assert!(result.stderr.is_empty());
}
#[test]
fn configuration_limit_and_missing_explicit_file_are_cli_errors() {
    let config = Fixture::new(&vec![b' '; 65537]);
    assert_eq!(
        invoke(&["--config", config.path(), "-v", "x"], None)
            .status
            .code(),
        Some(2)
    );
    assert_eq!(
        invoke(
            &["--config", "absent-explicit-config.toml", "-v", "x"],
            None
        )
        .status
        .code(),
        Some(2)
    );
}

#[test]
fn syntax_options_validate_before_any_configuration_or_document_read() {
    for options in [
        vec!["--format", "text", "--language", "rs"],
        vec!["--language", "   "],
    ] {
        let mut args = options;
        args.extend([
            "--config",
            "r3viewer-config-that-does-not-exist.toml",
            "nonexistent-source.rs",
        ]);
        let result = invoke(&args, None);
        assert_eq!(result.status.code(), Some(2));
        let error = String::from_utf8(result.stderr).unwrap();
        assert!(error.contains("--language"));
        assert!(!error.contains("no se puede abrir"));
        assert!(result.stdout.is_empty());
    }
}
#[test]
fn redirected_code_retains_complete_safe_source_despite_colour_and_numbers() {
    let source = "/* é界 */\r\nlet s = \"👩‍💻e\u{301}\";\n\n";
    let file = Fixture::new(source.as_bytes());
    let result = invoke(
        &[
            "-i",
            "--format",
            "code",
            "--language",
            "rs",
            "--colour",
            "always",
            "-n",
            file.path(),
        ],
        None,
    );
    assert!(result.status.success(), "{:?}", result.stderr);
    assert_eq!(result.stdout, source.as_bytes());
    assert!(result.stderr.is_empty());
    let unknown = invoke(
        &["--language", "r3viewer-no-such-grammar", "--value", source],
        None,
    );
    assert!(unknown.status.success());
    assert_eq!(unknown.stdout, source.as_bytes());
    assert!(
        String::from_utf8(unknown.stderr)
            .unwrap()
            .contains("desconocido")
    );
}

#[test]
fn structure_text_conflict_fails_before_configuration_and_document_io() {
    let result = invoke(
        &[
            "--format",
            "text",
            "--structure",
            "--config",
            "missing-d1-config.toml",
            "missing-d1-document.toml",
        ],
        None,
    );
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stdout.is_empty());
    let message = String::from_utf8(result.stderr).unwrap();
    assert!(message.contains("--structure"));
    assert!(!message.contains("configuración"));
    assert!(!message.contains("no se puede abrir"));
}

fn nu_value_wire(value: r3viewer_wire::Value) -> Vec<u8> {
    use r3viewer_wire::{Frame, Origin, VERSION, write_frame, write_magic};
    let mut bytes = Vec::new();
    write_magic(&mut bytes).unwrap();
    write_frame(
        &mut bytes,
        &Frame::Start {
            version: VERSION,
            cwd: "metadata.toml".into(),
            origin: Origin::SingleValue,
        },
    )
    .unwrap();
    write_frame(&mut bytes, &Frame::Value(value)).unwrap();
    write_frame(&mut bytes, &Frame::End).unwrap();
    bytes
}
#[test]
fn structure_flag_keeps_typed_nu_without_reinterpretation() {
    use r3viewer_wire::Value;
    let wire = nu_value_wire(Value::Record(vec![
        ("first".into(), Value::Nothing),
        ("second".into(), Value::String("".into())),
        (
            "range".into(),
            Value::IntRange {
                start: 1,
                step: 2,
                end: r3viewer_wire::Bound::Unbounded,
            },
        ),
    ]));
    let existing = invoke(&["--nu-wire"], Some(&wire));
    let structured = invoke(&["--nu-wire", "--structure"], Some(&wire));
    assert!(existing.status.success());
    assert!(structured.status.success());
    assert_eq!(existing.stdout, structured.stdout);
    assert!(structured.stderr.is_empty());
}
#[test]
fn explicit_toml_rejects_nontextual_nu_after_classification() {
    use r3viewer_wire::Value;
    for value in [
        Value::Int(7),
        Value::Record(vec![]),
        Value::Binary(b"x=1".to_vec()),
    ] {
        let wire = nu_value_wire(value);
        let result = invoke(&["--nu-wire", "--format", "toml"], Some(&wire));
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
        assert!(
            String::from_utf8(result.stderr)
                .unwrap()
                .contains("requiere texto")
        );
    }
}

#[cfg(windows)]
#[test]
fn explicit_toml_structure_outputs_complete_typed_data_and_preserves_default_source() {
    let source = "# source comment\r\n\"a.b\"=\"\\u754c\\e\"\r\nempty={}\r\narray=[]\r\nhex=0xF_F\r\nfloat=-nan\r\noffset=1979-05-27T07:32:00-07:00\r\nlocal=1979-05-27T07:32:00\r\ndate=1979-05-27\r\ntime=07:32:00\r\n";
    let default = invoke(&["--format", "toml", "--value", source], None);
    assert!(default.status.success(), "{:?}", default.stderr);
    assert_eq!(default.stdout, source.as_bytes());
    assert!(default.stderr.is_empty());
    for args in [
        vec!["--structure", "--format", "toml", "--value", source],
        vec![
            "-i",
            "--structure",
            "--format",
            "toml",
            "--colour",
            "always",
            "-n",
            "--value",
            source,
        ],
    ] {
        let output = invoke(&args, None);
        assert!(output.status.success(), "{:?}", output.stderr);
        assert!(output.stderr.is_empty(), "{:?}", output.stderr);
        let human = String::from_utf8(output.stdout).unwrap();
        for item in [
            "key(\"a.b\") = string(\"界\\u{001b}\")",
            "key(\"empty\") = table{}",
            "key(\"array\") = array[]",
            "integer(raw=\"0xF_F\")",
            "float(raw=\"-nan\")",
            "offset_datetime(raw=\"1979-05-27T07:32:00-07:00\")",
            "local_datetime(raw=\"1979-05-27T07:32:00\")",
            "local_date(raw=\"1979-05-27\")",
            "local_time(raw=\"07:32:00\")",
        ] {
            assert!(human.contains(item), "missing {item:?} in {human:?}");
        }
        assert!(!human.contains('\x1b'));
        assert!(!human.contains("# source comment"));
        assert!(human.find("key(\"a.b\")").unwrap() < human.find("key(\"hex\")").unwrap());
    }
}
#[cfg(windows)]
#[test]
fn structural_failure_returns_complete_source_and_language_alone_never_interprets() {
    let malformed = "# keep\r\na=[1,\r\n";
    let failed = invoke(
        &["--format", "toml", "--structure", "--value", malformed],
        None,
    );
    assert!(failed.status.success());
    assert_eq!(failed.stdout, malformed.as_bytes());
    assert!(
        String::from_utf8(failed.stderr)
            .unwrap()
            .contains("TOML no válido")
    );
    let valid = "x=1\r\n";
    let language = invoke(
        &["--structure", "--language", "toml", "--value", valid],
        None,
    );
    assert!(language.status.success());
    assert_eq!(language.stdout, valid.as_bytes());
    assert!(
        String::from_utf8(language.stderr)
            .unwrap()
            .contains("no hay intérprete")
    );
    let file = Fixture::new(valid.as_bytes());
    let toml_path = file.0.with_extension("toml");
    fs::rename(&file.0, &toml_path).unwrap();
    let file = Fixture(toml_path);
    let inferred = invoke(&["--structure", "--language", "py", file.path()], None);
    assert!(inferred.status.success(), "{:?}", inferred.stderr);
    assert!(
        String::from_utf8(inferred.stdout)
            .unwrap()
            .contains("integer(raw=\"1\")")
    );
    let text = invoke(&["--format", "text", file.path()], None);
    assert!(text.status.success());
    assert_eq!(text.stdout, valid.as_bytes());
}
#[cfg(windows)]
#[test]
fn explicit_toml_interprets_string_wire_without_using_nu_metadata() {
    let wire = nu_value_wire(r3viewer_wire::Value::String("x=1".into()));
    let automatic = invoke(&["--nu-wire", "--structure"], Some(&wire));
    assert!(automatic.status.success());
    assert_eq!(automatic.stdout, b"x=1");
    assert!(automatic.stderr.is_empty());
    let explicit = invoke(
        &["--nu-wire", "--structure", "--format", "toml"],
        Some(&wire),
    );
    assert!(explicit.status.success(), "{:?}", explicit.stderr);
    assert!(
        String::from_utf8(explicit.stdout)
            .unwrap()
            .contains("integer(raw=\"1\")")
    );
}

#[cfg(windows)]
#[test]
fn explicit_toml_string_byte_stream_accepts_text_but_unknown_never_becomes_text() {
    use r3viewer_wire::{ByteStreamKind, Frame, Origin, VERSION, write_frame, write_magic};
    for kind in [ByteStreamKind::String, ByteStreamKind::Unknown] {
        let mut wire = Vec::new();
        write_magic(&mut wire).unwrap();
        write_frame(
            &mut wire,
            &Frame::Start {
                version: VERSION,
                cwd: "metadata.toml".into(),
                origin: Origin::ByteStream(kind),
            },
        )
        .unwrap();
        write_frame(&mut wire, &Frame::Bytes(b"x=".to_vec())).unwrap();
        write_frame(&mut wire, &Frame::Bytes(b"1".to_vec())).unwrap();
        write_frame(&mut wire, &Frame::End).unwrap();
        let result = invoke(
            &["--nu-wire", "--format", "toml", "--structure"],
            Some(&wire),
        );
        if kind == ByteStreamKind::String {
            assert!(result.status.success(), "{:?}", result.stderr);
            assert!(
                String::from_utf8(result.stdout)
                    .unwrap()
                    .contains("integer(raw=\"1\")")
            );
            assert!(result.stderr.is_empty());
        } else {
            assert_eq!(result.status.code(), Some(2));
            assert!(result.stdout.is_empty());
            assert!(
                String::from_utf8(result.stderr)
                    .unwrap()
                    .contains("requiere texto")
            );
        }
    }
}
