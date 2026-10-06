//! Select a built-in grammar from actual origin, never inferred pipeline metadata.
use crate::{
    cli::{Cli, Format},
    input::SourceOrigin,
    language_catalog::LANGUAGES,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyntaxSelection {
    Plain { diagnostic: Option<String> },
    Language(String),
}
impl SyntaxSelection {
    pub fn select(cli: &Cli, origin: &SourceOrigin) -> Self {
        if cli.format == Format::Text {
            return Self::plain();
        }
        let token = cli.language.as_deref().map(str::trim).or_else(|| {
            if cli.format == Format::Toml {
                return Some("toml");
            }
            if cli.format == Format::Json {
                return Some("json");
            }
            if cli.format == Format::Xml {
                return Some("xml");
            }
            if cli.format == Format::Yaml {
                return Some("yaml");
            }
            if let SourceOrigin::LocalFile(path) = origin {
                path.extension().and_then(|e| e.to_str())
            } else {
                None
            }
        });
        if let Some(token) = token {
            let selected = if cli.language.is_some() {
                select_explicit_token(token)
            } else {
                resolve(token)
            };
            if let Some(name) = selected {
                return if name == "Plain Text" {
                    Self::plain()
                } else {
                    Self::Language(name.into())
                };
            }
            if cli.language.is_some() {
                return Self::Plain {
                    diagnostic: Some(format!(
                        "lenguaje integrado desconocido: {token:?}; se conserva texto plano"
                    )),
                };
            }
        }
        if cli.format == Format::Code {
            Self::Plain {
                diagnostic: Some(
                    "no se puede deducir el lenguaje; indica --language para resaltar código"
                        .into(),
                ),
            }
        } else {
            Self::plain()
        }
    }
    pub fn plain() -> Self {
        Self::Plain { diagnostic: None }
    }
    pub fn diagnostic(&self) -> Option<&str> {
        match self {
            Self::Plain { diagnostic } => diagnostic.as_deref(),
            Self::Language(_) => None,
        }
    }
}
/// Interpreter selection is independent of the highlighting language.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StructureSelection {
    None { diagnostic: Option<String> },
    Toml,
    Json,
    Yaml,
    Xml,
}
impl StructureSelection {
    pub fn select(
        cli: &Cli,
        origin: &SourceOrigin,
        typed: Option<&r3viewer_wire::TypedInput>,
    ) -> Result<Self, crate::input::InputError> {
        if matches!(
            cli.format,
            Format::Toml | Format::Json | Format::Yaml | Format::Xml
        ) {
            if !textual_input(typed) {
                return Err(crate::input::InputError::new(
                    2,
                    "--format toml/json/yaml/xml requiere texto; los valores Nu tipados conservan su inspección sin ese selector",
                ));
            }
            return Ok(match cli.format {
                Format::Json => Self::Json,
                Format::Yaml => Self::Yaml,
                Format::Xml => Self::Xml,
                _ => Self::Toml,
            });
        }
        if cli.format != Format::Text
            && textual_input(typed)
            && let SourceOrigin::LocalFile(path) = origin
            && let Some(ext) = path.extension()
        {
            if ext.eq_ignore_ascii_case("toml") {
                return Ok(Self::Toml);
            }
            if ext.eq_ignore_ascii_case("json") {
                return Ok(Self::Json);
            }
            if ext.eq_ignore_ascii_case("xml") {
                return Ok(Self::Xml);
            }
            if ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml") {
                return Ok(Self::Yaml);
            }
        }
        Ok(Self::None {
            diagnostic: if cli.structure && typed.is_none() {
                Some("no hay intérprete estructural para esta fuente; indica --format toml, json, yaml o xml para interpretar datos".into())
            } else {
                None
            },
        })
    }
    pub fn format(&self) -> Option<crate::structure_model::StructureFormat> {
        use crate::structure_model::StructureFormat;
        match self {
            Self::Toml => Some(StructureFormat::Toml),
            Self::Json => Some(StructureFormat::Json),
            Self::Yaml => Some(StructureFormat::Yaml),
            Self::Xml => Some(StructureFormat::Xml),
            Self::None { .. } => None,
        }
    }
    pub fn is_toml(&self) -> bool {
        matches!(self, Self::Toml)
    }
    pub fn diagnostic(&self) -> Option<&str> {
        match self {
            Self::None { diagnostic } => diagnostic.as_deref(),
            Self::Toml | Self::Json | Self::Yaml | Self::Xml => None,
        }
    }
}
/// Explicit built-in language token only: no origin or interpreter inference.
pub(crate) fn select_explicit_token(token: &str) -> Option<&'static str> {
    let token = token.trim();
    if token.eq_ignore_ascii_case("pwsh") {
        Some("PowerShell")
    } else {
        resolve(token)
    }
}
// Match the local worker: exact extension, then optional leading dot removed.
// Extension lookup is reverse; the custom name lookup uses forward order.
fn resolve(token: &str) -> Option<&'static str> {
    let extension = token.trim_start_matches('.');
    LANGUAGES
        .iter()
        .rev()
        .find(|(_, extensions)| extensions.iter().any(|e| e.eq_ignore_ascii_case(token)))
        .or_else(|| {
            LANGUAGES.iter().rev().find(|(_, extensions)| {
                extensions.iter().any(|e| e.eq_ignore_ascii_case(extension))
            })
        })
        .or_else(|| {
            LANGUAGES
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(token))
        })
        .map(|(name, _)| *name)
}
/// Presentation diagnostic; worker protocol reasons remain independent of UI language.
pub fn fallback_diagnostic(reason: crate::syntax::FallbackReason) -> String {
    use crate::syntax::FallbackReason::*;
    let detail = match reason {
        SourceBudget => "fuente demasiado grande para el resaltado",
        LongLine => "línea mayor de 64 KiB",
        UnknownLanguage => "lenguaje integrado desconocido",
        StartupTimeout => "se agotó el tiempo de preparación",
        LineTimeout => "se agotó el tiempo de una línea",
        RequestTimeout => "se agotó el tiempo de resaltado",
        SpanBudget => "demasiados intervalos de estilo",
        InvalidIpc => "respuesta inválida del motor",
        WorkerExited => "el motor terminó inesperadamente",
        Cancelled => "resaltado cancelado",
        UnsupportedPlatform => "aislamiento del motor pendiente de validar en esta plataforma",
        ContainmentFailed => "no se pudo aislar el motor",
        GrammarError => "falló la gramática integrada",
        OperationBudget => "gramática demasiado compleja",
    };
    format!("{detail}; se conserva texto plano")
}

/// Only Nu strings and explicitly typed string streams are textual snapshots.
pub fn textual_input(input: Option<&r3viewer_wire::TypedInput>) -> bool {
    use r3viewer_wire::{ByteStreamKind, Payload, Value};
    input.is_none_or(|input| {
        matches!(
            input.payload,
            Payload::SingleValue(Value::String(_))
                | Payload::ByteStream {
                    kind: ByteStreamKind::String,
                    ..
                }
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_json_literal_selects_json_but_language_does_not_reclassify() {
        let cli = Cli::parse_from(["viewer", "--format", "json", "--value", "null"]);
        let origin = SourceOrigin::Literal;
        assert_eq!(
            StructureSelection::select(&cli, &origin, None).unwrap(),
            StructureSelection::Json
        );
        assert_eq!(
            SyntaxSelection::select(&cli, &origin),
            SyntaxSelection::Language("JSON".into())
        );
        let cli = Cli::parse_from(["viewer", "--language", "toml", "x.json"]);
        assert_eq!(
            StructureSelection::select(&cli, &SourceOrigin::LocalFile("x.json".into()), None)
                .unwrap(),
            StructureSelection::Json
        );
        assert_eq!(
            SyntaxSelection::select(&cli, &SourceOrigin::LocalFile("x.json".into())),
            SyntaxSelection::Language("TOML".into())
        );
        assert_eq!(
            StructureSelection::select(&cli, &origin, None)
                .unwrap()
                .format(),
            None
        );
    }

    use clap::Parser;
    #[test]
    fn only_actual_file_origin_infers_and_explicit_language_precedes_extension() {
        let auto = Cli::parse_from(["viewer"]);
        assert_eq!(
            SyntaxSelection::select(&auto, &SourceOrigin::LocalFile("x.rs".into())),
            SyntaxSelection::Language("Rust".into())
        );
        for origin in [
            SourceOrigin::Literal,
            SourceOrigin::StandardInput,
            SourceOrigin::Nushell,
        ] {
            assert_eq!(
                SyntaxSelection::select(&auto, &origin),
                SyntaxSelection::plain()
            );
        }
        let explicit = Cli::parse_from(["viewer", "--language", ".PY"]);
        assert_eq!(
            SyntaxSelection::select(&explicit, &SourceOrigin::LocalFile("x.rs".into())),
            SyntaxSelection::Language("Python".into())
        );
        assert_eq!(resolve("h"), Some("Objective-C"));
        assert_eq!(resolve("plain text"), Some("Plain Text"));
    }
    #[cfg(windows)]
    #[test]
    fn generated_catalog_matches_the_packaged_worker_grammar_set() {
        let set = crate::syntax_worker::bundled_syntaxes().unwrap();
        assert_eq!(set.syntaxes().len(), LANGUAGES.len());
        for syntax in set.syntaxes() {
            for token in syntax
                .file_extensions
                .iter()
                .map(String::as_str)
                .chain(std::iter::once(syntax.name.as_str()))
            {
                let expected = set
                    .find_syntax_by_extension(token)
                    .or_else(|| {
                        set.find_syntax_by_extension(
                            &token.trim_start_matches('.').to_ascii_lowercase(),
                        )
                    })
                    .or_else(|| {
                        set.syntaxes()
                            .iter()
                            .find(|s| s.name.eq_ignore_ascii_case(token))
                    })
                    .unwrap();
                assert_eq!(resolve(token), Some(expected.name.as_str()), "{token}");
            }
        }
    }
    #[test]
    fn toml_interpreter_requires_real_file_or_explicit_format_not_language_or_metadata() {
        let auto = Cli::parse_from(["viewer", "--structure"]);
        assert_eq!(
            StructureSelection::select(&auto, &SourceOrigin::LocalFile("actual.TOML".into()), None)
                .unwrap(),
            StructureSelection::Toml
        );
        for origin in [
            SourceOrigin::Literal,
            SourceOrigin::StandardInput,
            SourceOrigin::Nushell,
        ] {
            assert!(
                !StructureSelection::select(&auto, &origin, None)
                    .unwrap()
                    .is_toml()
            );
        }
        let language = Cli::parse_from(["viewer", "--language", "toml"]);
        assert!(
            !StructureSelection::select(&language, &SourceOrigin::Literal, None)
                .unwrap()
                .is_toml()
        );
        let override_language = Cli::parse_from(["viewer", "--language", "py"]);
        assert!(
            StructureSelection::select(
                &override_language,
                &SourceOrigin::LocalFile("actual.toml".into()),
                None
            )
            .unwrap()
            .is_toml()
        );
        let explicit = Cli::parse_from(["viewer", "--format", "toml", "--value", "x=1"]);
        assert!(
            StructureSelection::select(&explicit, &SourceOrigin::Literal, None)
                .unwrap()
                .is_toml()
        );
        let conflict = Cli::parse_from(["viewer", "--structure", "--format", "text"]);
        assert_eq!(conflict.validate().unwrap_err().code, 2);
    }
    #[test]
    fn explicit_toml_accepts_only_textual_nu_but_structure_is_idempotent_on_typed_data() {
        use r3viewer_wire::{ByteStreamKind, Origin, Payload, Start, TypedInput, VERSION, Value};
        let explicit = Cli::parse_from(["viewer", "--format", "toml"]);
        let structure = Cli::parse_from(["viewer", "--structure"]);
        for payload in [
            Payload::SingleValue(Value::String("x=1".into())),
            Payload::ByteStream {
                kind: ByteStreamKind::String,
                bytes: b"x=1".to_vec(),
            },
        ] {
            let input = TypedInput {
                start: Start {
                    version: VERSION,
                    cwd: "metadata.toml".into(),
                    origin: Origin::SingleValue,
                },
                payload,
            };
            assert!(
                StructureSelection::select(&explicit, &SourceOrigin::Nushell, Some(&input))
                    .unwrap()
                    .is_toml()
            );
            assert!(
                !StructureSelection::select(&structure, &SourceOrigin::Nushell, Some(&input))
                    .unwrap()
                    .is_toml()
            );
        }
        for payload in [
            Payload::SingleValue(Value::Int(1)),
            Payload::SingleValue(Value::Record(vec![])),
            Payload::ListStream(vec![]),
            Payload::ByteStream {
                kind: ByteStreamKind::Unknown,
                bytes: b"x=1".to_vec(),
            },
            Payload::SingleValue(Value::Binary(b"x=1".to_vec())),
        ] {
            let input = TypedInput {
                start: Start {
                    version: VERSION,
                    cwd: "metadata.toml".into(),
                    origin: Origin::SingleValue,
                },
                payload,
            };
            assert_eq!(
                StructureSelection::select(&explicit, &SourceOrigin::Nushell, Some(&input))
                    .unwrap_err()
                    .code,
                2
            );
            let selected =
                StructureSelection::select(&structure, &SourceOrigin::Nushell, Some(&input))
                    .unwrap();
            assert!(!selected.is_toml());
            assert!(selected.diagnostic().is_none());
        }
    }
    #[test]
    fn yaml_selector_separates_file_origin_language_and_typed_payload() {
        use r3viewer_wire::{ByteStreamKind, Origin, Payload, Start, TypedInput, VERSION, Value};
        let auto = Cli::parse_from(["viewer", "--structure"]);
        for path in ["actual.yaml", "actual.YML"] {
            assert_eq!(
                StructureSelection::select(&auto, &SourceOrigin::LocalFile(path.into()), None)
                    .unwrap(),
                StructureSelection::Yaml
            );
        }
        let explicit = Cli::parse_from(["viewer", "--format", "yaml"]);
        let language = Cli::parse_from(["viewer", "--language", "yaml"]);
        for origin in [
            SourceOrigin::Literal,
            SourceOrigin::StandardInput,
            SourceOrigin::Nushell,
        ] {
            assert_eq!(
                StructureSelection::select(&explicit, &origin, None).unwrap(),
                StructureSelection::Yaml
            );
            assert_eq!(
                StructureSelection::select(&language, &origin, None)
                    .unwrap()
                    .format(),
                None
            );
            assert_eq!(
                StructureSelection::select(&auto, &origin, None)
                    .unwrap()
                    .format(),
                None
            );
        }
        assert_eq!(
            SyntaxSelection::select(&explicit, &SourceOrigin::Literal),
            SyntaxSelection::Language("YAML".into())
        );
        let override_language = Cli::parse_from(["viewer", "--language", "rs"]);
        let origin = SourceOrigin::LocalFile("actual.yaml".into());
        assert_eq!(
            StructureSelection::select(&override_language, &origin, None).unwrap(),
            StructureSelection::Yaml
        );
        assert_eq!(
            SyntaxSelection::select(&override_language, &origin),
            SyntaxSelection::Language("Rust".into())
        );
        let text = Cli::parse_from(["viewer", "--format", "text"]);
        assert_eq!(
            StructureSelection::select(&text, &origin, None)
                .unwrap()
                .format(),
            None
        );
        for payload in [
            Payload::SingleValue(Value::String("x: 1".into())),
            Payload::ByteStream {
                kind: ByteStreamKind::String,
                bytes: b"x: 1".to_vec(),
            },
        ] {
            let input = TypedInput {
                start: Start {
                    version: VERSION,
                    cwd: "metadata.yaml".into(),
                    origin: Origin::SingleValue,
                },
                payload,
            };
            assert_eq!(
                StructureSelection::select(&explicit, &SourceOrigin::Nushell, Some(&input))
                    .unwrap(),
                StructureSelection::Yaml
            );
            assert_eq!(
                StructureSelection::select(&auto, &SourceOrigin::Nushell, Some(&input))
                    .unwrap()
                    .format(),
                None
            );
        }
        for payload in [
            Payload::SingleValue(Value::Nothing),
            Payload::SingleValue(Value::Int(1)),
            Payload::SingleValue(Value::Record(vec![])),
            Payload::ListStream(vec![]),
            Payload::ByteStream {
                kind: ByteStreamKind::Unknown,
                bytes: b"x: 1".to_vec(),
            },
            Payload::SingleValue(Value::Binary(b"x: 1".to_vec())),
        ] {
            let input = TypedInput {
                start: Start {
                    version: VERSION,
                    cwd: "metadata.yaml".into(),
                    origin: Origin::SingleValue,
                },
                payload,
            };
            assert_eq!(
                StructureSelection::select(&explicit, &SourceOrigin::Nushell, Some(&input))
                    .unwrap_err()
                    .code,
                2
            );
            assert_eq!(
                StructureSelection::select(&auto, &SourceOrigin::Nushell, Some(&input))
                    .unwrap()
                    .format(),
                None
            );
        }
    }

    #[test]
    fn explicit_failures_are_diagnostic_not_document_errors() {
        let code = Cli::parse_from(["viewer", "--format", "code"]);
        assert!(
            SyntaxSelection::select(&code, &SourceOrigin::Literal)
                .diagnostic()
                .is_some()
        );
        let bad = Cli::parse_from(["viewer", "--language", "missing-language"]);
        assert!(
            SyntaxSelection::select(&bad, &SourceOrigin::Literal)
                .diagnostic()
                .is_some()
        );
        for args in [
            vec!["viewer", "--format", "text", "--language", "rs"],
            vec!["viewer", "--language", "  "],
        ] {
            assert_eq!(Cli::parse_from(args).validate().unwrap_err().code, 2);
        }
    }
    #[test]
    fn xml_selector_separates_file_origin_language_and_typed_payload() {
        use r3viewer_wire::{ByteStreamKind, Origin, Payload, Start, TypedInput, VERSION, Value};
        let auto = Cli::parse_from(["viewer", "--structure"]);
        for path in ["actual.xml", "actual.XML"] {
            assert_eq!(
                StructureSelection::select(&auto, &SourceOrigin::LocalFile(path.into()), None)
                    .unwrap(),
                StructureSelection::Xml
            );
        }
        let explicit = Cli::parse_from(["viewer", "--format", "xml"]);
        let language = Cli::parse_from(["viewer", "--language", "xml"]);
        for origin in [
            SourceOrigin::Literal,
            SourceOrigin::StandardInput,
            SourceOrigin::Nushell,
        ] {
            assert_eq!(
                StructureSelection::select(&explicit, &origin, None).unwrap(),
                StructureSelection::Xml
            );
            assert_eq!(
                StructureSelection::select(&language, &origin, None)
                    .unwrap()
                    .format(),
                None
            );
            assert_eq!(
                StructureSelection::select(&auto, &origin, None)
                    .unwrap()
                    .format(),
                None
            );
        }
        assert_eq!(
            SyntaxSelection::select(&explicit, &SourceOrigin::Literal),
            SyntaxSelection::Language("XML".into())
        );
        let override_language = Cli::parse_from(["viewer", "--language", "rs"]);
        let origin = SourceOrigin::LocalFile("actual.xml".into());
        assert_eq!(
            StructureSelection::select(&override_language, &origin, None).unwrap(),
            StructureSelection::Xml
        );
        assert_eq!(
            SyntaxSelection::select(&override_language, &origin),
            SyntaxSelection::Language("Rust".into())
        );
        let text = Cli::parse_from(["viewer", "--format", "text"]);
        assert_eq!(
            StructureSelection::select(&text, &origin, None)
                .unwrap()
                .format(),
            None
        );
        for payload in [
            Payload::SingleValue(Value::String("x: 1".into())),
            Payload::ByteStream {
                kind: ByteStreamKind::String,
                bytes: b"x: 1".to_vec(),
            },
        ] {
            let input = TypedInput {
                start: Start {
                    version: VERSION,
                    cwd: "metadata.xml".into(),
                    origin: Origin::SingleValue,
                },
                payload,
            };
            assert_eq!(
                StructureSelection::select(&explicit, &SourceOrigin::Nushell, Some(&input))
                    .unwrap(),
                StructureSelection::Xml
            );
            assert_eq!(
                StructureSelection::select(&auto, &SourceOrigin::Nushell, Some(&input))
                    .unwrap()
                    .format(),
                None
            );
        }
        for payload in [
            Payload::SingleValue(Value::Nothing),
            Payload::SingleValue(Value::Int(1)),
            Payload::SingleValue(Value::Record(vec![])),
            Payload::ListStream(vec![]),
            Payload::ByteStream {
                kind: ByteStreamKind::Unknown,
                bytes: b"x: 1".to_vec(),
            },
            Payload::SingleValue(Value::Binary(b"x: 1".to_vec())),
        ] {
            let input = TypedInput {
                start: Start {
                    version: VERSION,
                    cwd: "metadata.xml".into(),
                    origin: Origin::SingleValue,
                },
                payload,
            };
            assert_eq!(
                StructureSelection::select(&explicit, &SourceOrigin::Nushell, Some(&input))
                    .unwrap_err()
                    .code,
                2
            );
            assert_eq!(
                StructureSelection::select(&auto, &SourceOrigin::Nushell, Some(&input))
                    .unwrap()
                    .format(),
                None
            );
        }
    }
    #[test]
    fn shell_languages_alias_and_origin_are_separate() {
        use clap::Parser;
        for (token, expected) in [
            ("nu", "Nushell"),
            ("NUSHELL", "Nushell"),
            ("ps1", "PowerShell"),
            ("PSM1", "PowerShell"),
            ("psd1", "PowerShell"),
            ("PowerShell", "PowerShell"),
            ("pwsh", "PowerShell"),
            ("PWSH", "PowerShell"),
        ] {
            let cli = Cli::try_parse_from(["viewer", "--language", token]).unwrap();
            assert_eq!(
                SyntaxSelection::select(&cli, &SourceOrigin::Literal),
                SyntaxSelection::Language(expected.into())
            );
        }
        let cli = Cli::try_parse_from(["viewer"]).unwrap();
        for (extension, expected) in [
            ("nu", "Nushell"),
            ("NU", "Nushell"),
            ("ps1", "PowerShell"),
            ("psm1", "PowerShell"),
            ("PSD1", "PowerShell"),
        ] {
            assert_eq!(
                SyntaxSelection::select(
                    &cli,
                    &SourceOrigin::LocalFile(format!("file.{extension}").into())
                ),
                SyntaxSelection::Language(expected.into())
            );
        }
        assert_eq!(
            SyntaxSelection::select(&cli, &SourceOrigin::LocalFile("file.pwsh".into())),
            SyntaxSelection::plain()
        );
        assert_eq!(
            SyntaxSelection::select(&cli, &SourceOrigin::Literal),
            SyntaxSelection::plain()
        );
        assert_eq!(resolve("pwsh"), None);
        assert!(
            !LANGUAGES
                .iter()
                .flat_map(|(_, extensions)| *extensions)
                .any(|extension| extension.eq_ignore_ascii_case("pwsh"))
        );
        let plain = Cli::try_parse_from(["viewer", "--format", "text"]).unwrap();
        assert_eq!(
            SyntaxSelection::select(&plain, &SourceOrigin::LocalFile("file.nu".into())),
            SyntaxSelection::plain()
        );
        let conflict =
            Cli::try_parse_from(["viewer", "--format", "text", "--language", "pwsh"]).unwrap();
        assert!(conflict.validate().is_err());
    }
}
