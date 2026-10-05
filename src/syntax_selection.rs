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
            if let SourceOrigin::LocalFile(path) = origin {
                path.extension().and_then(|e| e.to_str())
            } else {
                None
            }
        });
        if let Some(token) = token {
            if let Some(name) = resolve(token) {
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
        let set = syntect::parsing::SyntaxSet::load_defaults_newlines();
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
}
