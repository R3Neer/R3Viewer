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
