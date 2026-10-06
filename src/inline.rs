//! Prepare complete TTY inline output before the caller writes any bytes.
use crate::text_engine::{self, Work};
use crate::{
    config::{PresentationConfig, Role},
    presentation::{PrepareError, SafeTextModel},
    syntax::{FallbackReason, RoleSpan, SyntaxLimits, SyntaxRole, highlight_inline},
    syntax_selection::{SyntaxSelection, fallback_diagnostic},
};
use ratatui::style::Color;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use unicode_segmentation::GraphemeCursor;

const MAX_OUTPUT: usize = 256 * 1024 * 1024;
const COPY_CHUNK: usize = 64 * 1024;
const OUTPUT_BUDGET: &str = "salida inline excede256MiB; no se muestra un prefijo";

#[derive(Debug)]
pub struct InlineOutput {
    pub content: String,
    pub diagnostic: Option<String>,
}
/// No terminal mode changes or writes occur here. Failed syntax degrades the
/// entire snapshot; cancellation and output limits return no partial output.
pub fn render(
    model: &SafeTextModel,
    selection: &SyntaxSelection,
    config: &PresentationConfig,
    cancelled: Arc<AtomicBool>,
) -> Result<InlineOutput, PrepareError> {
    check_cancelled(&cancelled)?;
    let highlighted = match selection {
        SyntaxSelection::Language(language) if config.coloured => Some(highlight_inline(
            model.original_handle(),
            language.clone(),
            SyntaxLimits::default(),
            Arc::clone(&cancelled),
        )),
        _ => None,
    };
    prepare(
        model,
        selection,
        config,
        &cancelled,
        highlighted,
        MAX_OUTPUT,
    )
}
fn check_cancelled(cancelled: &AtomicBool) -> Result<(), PrepareError> {
    if cancelled.load(Ordering::Relaxed) {
        Err(PrepareError::Cancelled)
    } else {
        Ok(())
    }
}
struct BoundedOutput<'a> {
    content: String,
    limit: usize,
    cancelled: &'a AtomicBool,
}
impl BoundedOutput<'_> {
    fn push(&mut self, mut text: &str) -> Result<(), PrepareError> {
        // Bound copying a single long plain line as well as SGR/gutters.
        while !text.is_empty() {
            check_cancelled(self.cancelled)?;
            let mut end = text.len().min(COPY_CHUNK);
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            let needed = self
                .content
                .len()
                .checked_add(end)
                .filter(|size| *size <= self.limit)
                .ok_or(PrepareError::Limit(OUTPUT_BUDGET))?;
            if needed > self.content.capacity() {
                let capacity = self
                    .content
                    .capacity()
                    .saturating_mul(2)
                    .max(needed)
                    .min(self.limit);
                self.content
                    .try_reserve_exact(capacity - self.content.len())
                    .map_err(|_| {
                        PrepareError::Limit("no se puede reservar la salida inline completa")
                    })?;
            }
            self.content.push_str(&text[..end]);
            text = &text[end..];
        }
        Ok(())
    }
}
fn colour(config: &PresentationConfig, role: SyntaxRole) -> Color {
    let role = match role {
        SyntaxRole::Value => Role::Value,
        SyntaxRole::Secondary => Role::Secondary,
        SyntaxRole::Accent => Role::Accent,
        SyntaxRole::Success => Role::Success,
        SyntaxRole::Process => Role::Process,
        SyntaxRole::Heading => Role::Heading,
    };
    // PresentationConfig resolves palette entries to RGB or terminal default.
    match config.style(role).fg {
        Some(colour @ Color::Rgb(..)) => colour,
        _ => Color::Reset,
    }
}
fn set_colour(
    out: &mut BoundedOutput<'_>,
    current: &mut Color,
    next: Color,
) -> Result<(), PrepareError> {
    if *current != next {
        match next {
            Color::Rgb(r, g, b) => out.push(&format!("\x1b[38;2;{r};{g};{b}m"))?,
            _ => out.push("\x1b[0m")?,
        }
        *current = next;
    }
    Ok(())
}
fn prepare(
    model: &SafeTextModel,
    selection: &SyntaxSelection,
    config: &PresentationConfig,
    cancelled: &AtomicBool,
    highlighted: Option<Result<Vec<RoleSpan>, FallbackReason>>,
    limit: usize,
) -> Result<InlineOutput, PrepareError> {
    check_cancelled(cancelled)?;
    let (spans, mut diagnostic) = match highlighted {
        Some(Ok(spans)) => (Some(spans), None),
        Some(Err(FallbackReason::Cancelled)) => return Err(PrepareError::Cancelled),
        Some(Err(reason)) => (None, Some(fallback_diagnostic(reason))),
        None => (None, selection.diagnostic().map(str::to_owned)),
    };
    let content = match assemble(model, config, cancelled, spans.as_deref(), limit) {
        Err(PrepareError::Limit(reason)) if reason == OUTPUT_BUDGET && spans.is_some() => {
            // The failed styled buffer was dropped before this whole-document rebuild.
            diagnostic = Some("salida de estilos excede256MiB; se conserva texto plano".into());
            assemble(model, config, cancelled, None, limit)?
        }
        other => other?,
    };
    check_cancelled(cancelled)?;
    Ok(InlineOutput {
        content,
        diagnostic,
    })
}
fn assemble(
    model: &SafeTextModel,
    config: &PresentationConfig,
    cancelled: &AtomicBool,
    spans: Option<&[RoleSpan]>,
    limit: usize,
) -> Result<String, PrepareError> {
    let mut out = BoundedOutput {
        content: String::new(),
        limit,
        cancelled,
    };
    if spans.is_none() && !config.line_numbers {
        out.push(model.visible())?;
    } else {
        let visible = model.visible();
        // The model keeps a final cursor row after LF; inline never emits it.
        let rows = if visible.is_empty() {
            0
        } else {
            model.line_count() - usize::from(visible.ends_with('\n'))
        };
        let digits = rows.to_string().len();
        let mut current = Color::Reset;
        let mut provenance = model.provenance_cursor(0);
        let mut span_index = 0;
        for row in 0..rows {
            check_cancelled(cancelled)?;
            // Indexed line starts avoid an uninterruptible search through a long plain line.
            let start = model.line_range(row).unwrap().start;
            let end = model
                .line_range(row + 1)
                .map_or(visible.len(), |range| range.start);
            let line = &visible[start..end];
            if config.line_numbers {
                set_colour(&mut out, &mut current, Color::Reset)?;
                out.push(&format!("{:>digits$} ", row + 1))?;
            }
            if let Some(spans) = spans {
                let mut cursor = GraphemeCursor::new(0, line.len(), true);
                let mut work = Work::preparation(cancelled);
                let mut byte = 0;
                while let Some(end) = text_engine::next(line, &mut cursor, &mut work)
                    .map_err(crate::presentation::engine_error)?
                {
                    let grapheme = &line[byte..end];
                    check_cancelled(cancelled)?;
                    let source = provenance.source_range_with(
                        start + byte..start + byte + grapheme.len(),
                        &mut work,
                    )?;
                    while spans
                        .get(span_index)
                        .is_some_and(|span| span.end as usize <= source.start)
                    {
                        span_index += 1;
                    }
                    let role = spans
                        .get(span_index)
                        .filter(|span| {
                            span.start as usize <= source.start && source.start < span.end as usize
                        })
                        .map_or(SyntaxRole::Value, |span| span.role);
                    set_colour(&mut out, &mut current, colour(config, role))?;
                    out.push(grapheme)?;
                    byte = end;
                }
            } else {
                out.push(line)?;
            }
        }
        set_colour(&mut out, &mut current, Color::Reset)?;
    }
    check_cancelled(cancelled)?;
    Ok(out.content)
}

#[cfg(test)]
mod tests {
    #[test]
    fn giant_cluster_inline_preserves_every_byte_and_first_scalar_role() {
        let cancel = AtomicBool::new(false);
        let text = format!("a{}END", "\u{301}".repeat(16_000_000));
        let model = model(&text);
        let config = config(true, false);
        let spans = [RoleSpan {
            start: 0,
            end: 1,
            role: SyntaxRole::Accent,
        }];
        let output = assemble(&model, &config, &cancel, Some(&spans), 256 * 1024 * 1024).unwrap();
        assert_eq!(no_sgr(&output), text);
        assert!(output.starts_with("\x1b[38;2;1;2;3m"));
    }

    use super::*;
    use crate::cli::Cli;
    use clap::Parser;
    use std::sync::atomic::AtomicU64;
    fn config(coloured: bool, numbers: bool) -> PresentationConfig {
        static ID: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "r3viewer-inline-{}-{}.toml",
            std::process::id(),
            ID.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, "[colours]\nvalue='default'\naccent='#010203'\nsecondary='#040506'\nsuccess='#070809'\nprocess='#0a0b0c'\nheading='#0d0e0f'\n").unwrap();
        let cli = Cli::parse_from([
            "viewer",
            "--config",
            path.to_str().unwrap(),
            "--colour",
            if coloured { "always" } else { "never" },
            if numbers {
                "--line-numbers"
            } else {
                "--no-line-numbers"
            },
        ]);
        let config =
            PresentationConfig::load(&cli, true, Arc::new(AtomicBool::new(false))).unwrap();
        std::fs::remove_file(path).unwrap();
        config
    }
    fn model(source: &str) -> SafeTextModel {
        SafeTextModel::new(source.into(), &AtomicBool::new(false)).unwrap()
    }
    fn no_sgr(text: &str) -> String {
        let mut chars = text.chars();
        let mut plain = String::new();
        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                assert_eq!(chars.next(), Some('['));
                for ch in chars.by_ref() {
                    if ch == 'm' {
                        break;
                    }
                }
            } else {
                plain.push(ch);
            }
        }
        plain
    }
    #[test]
    fn plain_preserves_controls_crlf_tabs_and_real_numbered_lines() {
        let config = config(false, true);
        let language = SyntaxSelection::Language("Rust".into());
        for (source, expected) in [
            ("", ""),
            ("\n", "1 \n"),
            ("a\r\n\t界\n", "1 a\r\n2 \t界\n"),
            ("a\n\nlast", "1 a\n2 \n3 last"),
            ("\x1b\u{009b}\t", "1 ␛\\u{009b}\t"),
        ] {
            let text = model(source);
            let output =
                render(&text, &language, &config, Arc::new(AtomicBool::new(false))).unwrap();
            assert_eq!(output.content, expected);
            assert!(output.diagnostic.is_none());
            assert_eq!(text.original(), source);
        }
        let text = model("x\n".repeat(10).as_str());
        let output = render(
            &text,
            &SyntaxSelection::plain(),
            &config,
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
        assert!(output.content.starts_with(" 1 x\n"));
        assert!(output.content.ends_with("10 x\n"));
    }
    #[test]
    fn roles_use_first_original_scalar_of_whole_graphemes_and_placeholders() {
        let config = config(true, false);
        let text = model("e\u{301}👩‍💻\u{009b}␛\x1b\r\n界");
        let spans = text
            .original()
            .char_indices()
            .map(|(start, ch)| RoleSpan {
                start: start as u32,
                end: (start + ch.len_utf8()) as u32,
                role: match ch {
                    'e' => SyntaxRole::Accent,
                    '👩' => SyntaxRole::Success,
                    '\u{009b}' => SyntaxRole::Secondary,
                    '\x1b' => SyntaxRole::Heading,
                    '\u{301}' | '💻' | '\u{200d}' | '界' => SyntaxRole::Process,
                    _ => SyntaxRole::Value,
                },
            })
            .collect();
        let output = prepare(
            &text,
            &SyntaxSelection::Language("Rust".into()),
            &config,
            &AtomicBool::new(false),
            Some(Ok(spans)),
            MAX_OUTPUT,
        )
        .unwrap();
        assert_eq!(no_sgr(&output.content), text.visible());
        assert!(output.content.contains("\x1b[38;2;1;2;3me\u{301}"));
        assert!(output.content.contains("\x1b[38;2;7;8;9m👩‍💻"));
        assert!(output.content.contains("\x1b[38;2;4;5;6m\\u{009b}"));
        assert!(output.content.contains("\x1b[0m␛\x1b[38;2;13;14;15m␛"));
        assert!(output.content.contains("\r\n"));
        assert!(output.content.ends_with("界\x1b[0m"));
    }
    #[test]
    fn later_scalar_role_does_not_colour_an_unstyled_cluster() {
        let config = config(true, false);
        let text = model("e\u{301}👩‍💻x");
        let source = text.original();
        let spans = vec![
            RoleSpan {
                start: 1,
                end: 3,
                role: SyntaxRole::Accent,
            },
            RoleSpan {
                start: source.find('💻').unwrap() as u32,
                end: source.find('x').unwrap() as u32,
                role: SyntaxRole::Process,
            },
            RoleSpan {
                start: source.find('x').unwrap() as u32,
                end: source.len() as u32,
                role: SyntaxRole::Success,
            },
        ];
        let output = prepare(
            &text,
            &SyntaxSelection::Language("Rust".into()),
            &config,
            &AtomicBool::new(false),
            Some(Ok(spans)),
            MAX_OUTPUT,
        )
        .unwrap();
        assert_eq!(output.content, "e\u{301}👩‍💻\x1b[38;2;7;8;9mx\x1b[0m");
    }
    #[test]
    fn failure_and_style_expansion_degrade_the_whole_document_atomically() {
        let config = config(true, true);
        let text = model("abc\r\n界\tlast");
        let output = prepare(
            &text,
            &SyntaxSelection::Language("Rust".into()),
            &config,
            &AtomicBool::new(false),
            Some(Err(FallbackReason::SpanBudget)),
            MAX_OUTPUT,
        )
        .unwrap();
        assert_eq!(output.content, "1 abc\r\n2 界\tlast");
        assert!(!output.content.contains('\x1b'));
        assert!(output.diagnostic.unwrap().contains("texto plano"));
        let text = model("abc");
        let styles = || {
            vec![
                RoleSpan {
                    start: 0,
                    end: 1,
                    role: SyntaxRole::Accent,
                },
                RoleSpan {
                    start: 1,
                    end: 2,
                    role: SyntaxRole::Success,
                },
                RoleSpan {
                    start: 2,
                    end: 3,
                    role: SyntaxRole::Process,
                },
            ]
        };
        let output = prepare(
            &text,
            &SyntaxSelection::Language("Rust".into()),
            &config,
            &AtomicBool::new(false),
            Some(Ok(styles())),
            5,
        )
        .unwrap();
        assert_eq!(output.content, "1 abc");
        assert!(output.diagnostic.unwrap().contains("salida de estilos"));
        assert!(matches!(
            prepare(
                &text,
                &SyntaxSelection::Language("Rust".into()),
                &config,
                &AtomicBool::new(false),
                Some(Ok(styles())),
                4
            ),
            Err(PrepareError::Limit(_))
        ));
        assert!(matches!(
            prepare(
                &text,
                &SyntaxSelection::plain(),
                &config,
                &AtomicBool::new(true),
                None,
                MAX_OUTPUT
            ),
            Err(PrepareError::Cancelled)
        ));
        assert!(matches!(
            prepare(
                &text,
                &SyntaxSelection::plain(),
                &config,
                &AtomicBool::new(false),
                Some(Err(FallbackReason::Cancelled)),
                MAX_OUTPUT
            ),
            Err(PrepareError::Cancelled)
        ));
        assert_eq!(text.original(), "abc");
    }
    #[test]
    fn plain_diagnostic_does_not_enable_ansi_or_syntax() {
        let config = config(true, false);
        let text = model("let x = 42;\n");
        let selection = SyntaxSelection::Plain {
            diagnostic: Some("idioma no disponible".into()),
        };
        let output = render(&text, &selection, &config, Arc::new(AtomicBool::new(false))).unwrap();
        assert_eq!(output.content, text.visible());
        assert_eq!(output.diagnostic.as_deref(), Some("idioma no disponible"));
    }
}
