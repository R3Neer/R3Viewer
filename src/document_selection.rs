//! Document interpretation follows explicit format or the actual file origin.
use crate::{
    cli::{Cli, Format},
    document_model::Profile,
    input::{InputError, SourceOrigin},
    syntax_selection::textual_input,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentSelection {
    None,
    Markdown(Profile),
}
impl DocumentSelection {
    pub fn select(
        cli: &Cli,
        origin: &SourceOrigin,
        typed: Option<&r3viewer_wire::TypedInput>,
    ) -> Result<Self, InputError> {
        if cli.format == Format::Markdown {
            if !textual_input(typed) {
                return Err(InputError::new(
                    2,
                    "--format markdown requiere texto; los valores Nu tipados conservan su inspección sin ese selector",
                ));
            }
            return Ok(Self::Markdown(Profile::Extended));
        }
        if cli.format == Format::Auto
            && textual_input(typed)
            && let SourceOrigin::LocalFile(path) = origin
            && let Some(extension) = path.extension()
            && (extension.eq_ignore_ascii_case("md") || extension.eq_ignore_ascii_case("markdown"))
        {
            return Ok(Self::Markdown(Profile::Extended));
        }
        Ok(Self::None)
    }
    pub fn is_markdown(self) -> bool {
        matches!(self, Self::Markdown(_))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        document_source::DocumentSource,
        presentation::{PrepareError, SafeTextModel},
        syntax_selection::{StructureSelection, SyntaxSelection},
    };
    use clap::Parser;
    use r3viewer_wire::{ByteStreamKind, Origin, Payload, Start, TypedInput, Value};
    use std::sync::{Arc, atomic::AtomicBool};
    fn typed(payload: Payload) -> TypedInput {
        TypedInput {
            start: Start {
                version: r3viewer_wire::VERSION,
                cwd: "foreign-cwd/notas.md".into(),
                origin: Origin::SingleValue,
            },
            payload,
        }
    }
    #[test]
    fn d3a_actual_file_extensions_and_explicit_overrides() {
        let auto = Cli::parse_from(["viewer"]);
        for name in ["a.md", "a.MD", "a.markdown", "a.MARKDOWN"] {
            assert_eq!(
                DocumentSelection::select(&auto, &SourceOrigin::LocalFile(name.into()), None)
                    .unwrap(),
                DocumentSelection::Markdown(Profile::Extended)
            );
        }
        for origin in [
            SourceOrigin::LocalFile("a.mdx".into()),
            SourceOrigin::Literal,
            SourceOrigin::StandardInput,
            SourceOrigin::Nushell,
        ] {
            assert_eq!(
                DocumentSelection::select(&auto, &origin, None).unwrap(),
                DocumentSelection::None
            );
        }
        for format in ["text", "code", "toml", "json", "yaml", "xml"] {
            let cli = Cli::parse_from(["viewer", "--format", format]);
            assert_eq!(
                DocumentSelection::select(&cli, &SourceOrigin::LocalFile("a.md".into()), None)
                    .unwrap(),
                DocumentSelection::None
            );
        }
    }
    #[test]
    fn d3a_explicit_markdown_and_source_language_are_independent() {
        let origin = SourceOrigin::LocalFile("a.json".into());
        let cli = Cli::parse_from(["viewer", "--format", "markdown", "--structure"]);
        assert!(
            DocumentSelection::select(&cli, &origin, None)
                .unwrap()
                .is_markdown()
        );
        assert_eq!(
            StructureSelection::select(&cli, &origin, None).unwrap(),
            StructureSelection::None { diagnostic: None }
        );
        assert_eq!(
            SyntaxSelection::select(&cli, &origin),
            SyntaxSelection::Language("Markdown".into())
        );
        for language in ["Rust", "own-unknown-language"] {
            let cli = Cli::parse_from(["viewer", "--format", "markdown", "--language", language]);
            assert!(
                DocumentSelection::select(&cli, &origin, None)
                    .unwrap()
                    .is_markdown()
            );
            assert_eq!(
                StructureSelection::select(&cli, &origin, None).unwrap(),
                StructureSelection::None { diagnostic: None }
            );
            if language == "Rust" {
                assert_eq!(
                    SyntaxSelection::select(&cli, &origin),
                    SyntaxSelection::Language("Rust".into())
                );
            } else {
                assert!(
                    SyntaxSelection::select(&cli, &origin)
                        .diagnostic()
                        .is_some()
                );
            }
        }
        let auto = Cli::parse_from(["viewer", "--structure"]);
        assert_eq!(
            StructureSelection::select(&auto, &SourceOrigin::LocalFile("a.md".into()), None)
                .unwrap(),
            StructureSelection::None { diagnostic: None }
        );
        let literal = Cli::parse_from(["viewer", "--value", "a.md", "--language", "markdown"]);
        assert_eq!(
            DocumentSelection::select(&literal, &SourceOrigin::Literal, None).unwrap(),
            DocumentSelection::None
        );
    }
    #[test]
    fn d3a_nushell_types_are_not_reconstructed_from_metadata() {
        let auto = Cli::parse_from(["viewer"]);
        let explicit = Cli::parse_from(["viewer", "--format", "markdown"]);
        for payload in [
            Payload::SingleValue(Value::String("notas.md".into())),
            Payload::ByteStream {
                kind: ByteStreamKind::String,
                bytes: b"# T".to_vec(),
            },
        ] {
            let t = typed(payload);
            assert_eq!(
                DocumentSelection::select(&auto, &SourceOrigin::Nushell, Some(&t)).unwrap(),
                DocumentSelection::None
            );
            assert!(
                DocumentSelection::select(&explicit, &SourceOrigin::Nushell, Some(&t))
                    .unwrap()
                    .is_markdown()
            );
        }
        for payload in [
            Payload::SingleValue(Value::Nothing),
            Payload::SingleValue(Value::Int(1)),
            Payload::SingleValue(Value::Record(vec![])),
            Payload::SingleValue(Value::List(vec![])),
            Payload::SingleValue(Value::Binary(vec![])),
            Payload::ByteStream {
                kind: ByteStreamKind::Unknown,
                bytes: b"# T".to_vec(),
            },
        ] {
            let t = typed(payload);
            assert_eq!(
                DocumentSelection::select(&explicit, &SourceOrigin::Nushell, Some(&t))
                    .unwrap_err()
                    .code,
                2
            );
            assert_eq!(
                DocumentSelection::select(&auto, &SourceOrigin::LocalFile("a.md".into()), Some(&t))
                    .unwrap(),
                DocumentSelection::None
            );
        }
    }
    #[test]
    fn d3a_source_shared_snapshot_projection_and_cancellation() {
        let original = Arc::new("A\x1bB\u{0085}C\r\n".to_owned());
        let cancel = AtomicBool::new(false);
        let source =
            DocumentSource::new(original.clone(), SourceOrigin::Literal, 17, 23, &cancel).unwrap();
        let hash = *source.hash();
        let model = SafeTextModel::from_shared(source.original_handle(), &cancel).unwrap();
        assert!(Arc::ptr_eq(&original, &source.original_handle()));
        assert!(Arc::ptr_eq(&original, &model.original_handle()));
        assert_eq!(source.text(), original.as_str());
        assert_eq!(*source.hash(), hash);
        assert_eq!(model.visible(), "A␛B\\u{0085}C\r\n");
        let stopped = AtomicBool::new(true);
        assert!(matches!(
            SafeTextModel::from_shared(original.clone(), &stopped),
            Err(PrepareError::Cancelled)
        ));
        assert_eq!(source.text(), original.as_str());
        assert_eq!(*source.hash(), hash);
        let empty = Arc::new(String::new());
        assert_eq!(
            SafeTextModel::from_shared(empty, &cancel)
                .unwrap()
                .visible(),
            ""
        );
    }
    #[test]
    fn d3a_source_line_limit_keeps_original_snapshot() {
        let original = Arc::new("\n".repeat(1_000_000));
        let cancel = AtomicBool::new(false);
        let source =
            DocumentSource::new(original.clone(), SourceOrigin::StandardInput, 7, 9, &cancel)
                .unwrap();
        let hash = *source.hash();
        assert!(matches!(
            SafeTextModel::from_shared(source.original_handle(), &cancel),
            Err(PrepareError::Limit(
                "documento excede un millón de líneas; no se muestra un prefijo"
            ))
        ));
        assert!(Arc::ptr_eq(&original, &source.original_handle()));
        assert_eq!(source.len(), 1_000_000);
        assert_eq!(*source.hash(), hash);
        assert_eq!(source.text(), original.as_str());
    }
}
