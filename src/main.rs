use clap::Parser;
use r3viewer::{cli::Cli, input, output, session, text};
use std::{
    io::{self, IsTerminal},
    process::ExitCode,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

fn main() -> ExitCode {
    let document_args: Vec<_> = std::env::args_os().skip(1).take(5).collect();
    if document_args
        .first()
        .is_some_and(|arg| arg == "--document-worker")
    {
        if document_args.len() != 4 {
            return ExitCode::from(75);
        }
        let parsed = (
            document_args[1]
                .to_str()
                .and_then(|s| s.parse::<u64>().ok()),
            document_args[2]
                .to_str()
                .and_then(|s| s.parse::<u64>().ok()),
            document_args[3]
                .to_str()
                .and_then(|s| s.parse::<usize>().ok()),
        );
        return match parsed {
            (Some(generation), Some(request), Some(handle)) => {
                if r3viewer::document_worker::run_private_stdio(generation, request, handle).is_ok()
                {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::from(75)
                }
            }
            _ => ExitCode::from(75),
        };
    }

    let mut private_args = std::env::args_os().skip(1);
    let private = private_args.next();
    if private_args.next().is_none() {
        let worker = match private.as_deref() {
            Some(arg) if arg == "--syntax-worker" => {
                Some(r3viewer::syntax_worker::run_private_stdio())
            }
            Some(arg) if arg == "--structure-worker" => {
                Some(r3viewer::structure_worker::run_private_stdio())
            }
            _ => None,
        };
        if let Some(result) = worker {
            return if result.is_ok() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(75)
            };
        }
    }
    let cli = Cli::parse();
    if let Err(error) = cli.validate() {
        eprintln!("r3viewer: {}", text::safe_text(&error.message));
        return ExitCode::from(error.code);
    }
    let cancelled = Arc::new(AtomicBool::new(false));
    let signal_flag = cancelled.clone();
    if let Err(err) = ctrlc::set_handler(move || {
        signal_flag.store(true, Ordering::Relaxed);
    }) {
        eprintln!("r3viewer: no se puede preparar cancelación: {err}");
        return ExitCode::from(1);
    }
    let presentation = match r3viewer::config::PresentationConfig::load(
        &cli,
        io::stdout().is_terminal(),
        cancelled.clone(),
    ) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("r3viewer: {}", text::safe_text(&err.message));
            return ExitCode::from(err.code);
        }
    };
    let mut source = match input::load(&cli, io::stdin().is_terminal(), cancelled.clone()) {
        Ok(source) => source,
        Err(err) => {
            eprintln!("r3viewer: {}", text::safe_text(&err.message));
            return ExitCode::from(err.code);
        }
    };
    if cancelled.load(Ordering::Relaxed) {
        return ExitCode::from(130);
    }
    let document_selection = match r3viewer::document_selection::DocumentSelection::select(
        &cli,
        &source.origin,
        source.typed.as_ref(),
    ) {
        Ok(selection) => selection,
        Err(error) => {
            eprintln!("r3viewer: {}", text::safe_text(&error.message));
            return ExitCode::from(error.code);
        }
    };
    let stdout_tty = io::stdout().is_terminal();
    let interactive = !cli.inline && stdout_tty && session::available();
    // Complete noninteractive Markdown; interactive Markdown currently shows Source.
    if document_selection.is_markdown() && !interactive {
        let executable = match std::env::current_exe() {
            Ok(path) => path,
            Err(_) => {
                eprintln!("r3viewer: no se puede localizar el worker de Documento");
                return ExitCode::from(1);
            }
        };
        let original = Arc::new(std::mem::take(&mut source.text));
        let columns = if stdout_tty {
            crossterm::terminal::size()
                .map(|(width, _)| width)
                .unwrap_or(80)
        } else {
            80
        };
        let prepared = match r3viewer::document_output::prepare(
            executable,
            original,
            source.origin,
            columns,
            &presentation,
            cancelled.clone(),
            r3viewer::document_output::OUTPUT_LIMIT,
        ) {
            Ok(prepared) => prepared,
            Err(error) => {
                if error.exit_code() != 130 {
                    eprintln!("r3viewer: {}", error.diagnostic());
                }
                return ExitCode::from(error.exit_code());
            }
        };
        if let Some(diagnostic) = prepared.diagnostic {
            eprintln!("r3viewer: {diagnostic}");
        }
        return match output::write_cancelable(prepared.content, &cancelled) {
            Ok(()) => ExitCode::SUCCESS,
            Err(output::OutputError::Cancelled) => ExitCode::from(130),
            Err(output::OutputError::Io(error)) if error.kind() == io::ErrorKind::BrokenPipe => {
                ExitCode::SUCCESS
            }
            Err(output::OutputError::Io(error)) => {
                eprintln!("r3viewer: salida fallida: {error}");
                ExitCode::from(1)
            }
        };
    }
    if document_selection.is_markdown() && interactive {
        eprintln!(
            "r3viewer: Markdown interactivo pendiente; se muestra Fuente. Usa --inline/-i para la vista Markdown."
        );
    }
    let syntax = r3viewer::syntax_selection::SyntaxSelection::select(&cli, &source.origin);
    let structure = match r3viewer::syntax_selection::StructureSelection::select(
        &cli,
        &source.origin,
        source.typed.as_ref(),
    ) {
        Ok(selection) => selection,
        Err(error) => {
            eprintln!("r3viewer: {}", text::safe_text(&error.message));
            return ExitCode::from(error.code);
        }
    };
    if interactive {
        let model = match r3viewer::presentation::SafeTextModel::new(source.text, &cancelled) {
            Ok(model) => Arc::new(model),
            Err(r3viewer::presentation::PrepareError::Cancelled) => return ExitCode::from(130),
            Err(r3viewer::presentation::PrepareError::Limit(message)) => {
                eprintln!("r3viewer: {message}");
                return ExitCode::from(1);
            }
        };
        let Some(title) = text::safe_text_cancelable(&source.title, &cancelled) else {
            return ExitCode::from(130);
        };
        match r3viewer::viewer_session::run(
            model,
            source.typed.map(Arc::new),
            &title,
            &presentation,
            &syntax,
            &structure,
            cli.structure,
            cancelled.clone(),
        ) {
            Ok(true) => return ExitCode::from(130),
            Ok(false) => return ExitCode::SUCCESS,
            Err(err) => {
                eprintln!(
                    "r3viewer: sesión fallida: {}",
                    text::safe_text(&err.to_string())
                );
                return ExitCode::from(1);
            }
        }
    }
    if let Some(diagnostic) = structure.diagnostic() {
        eprintln!("r3viewer: {}", text::safe_text(diagnostic));
    }
    let structured = if cli.structure && structure.format().is_some() {
        let original = Arc::new(std::mem::take(&mut source.text));
        match r3viewer::structure_output::prepare(
            original.clone(),
            structure.format().unwrap(),
            cancelled.clone(),
        ) {
            Ok(r3viewer::structure_output::PreparedStructure::Rendered {
                content,
                diagnostic,
            }) => {
                if let Some(diagnostic) = diagnostic {
                    eprintln!("r3viewer: {}", text::safe_text(&diagnostic));
                }
                // This labelled human representation has its own ordinal lines;
                // source line numbers and source grammar do not describe it.
                source.typed = None;
                Some(content)
            }
            Ok(r3viewer::structure_output::PreparedStructure::Fallback(diagnostic)) => {
                eprintln!("r3viewer: {}", text::safe_text(&diagnostic));
                source.text = Arc::try_unwrap(original).unwrap_or_else(|shared| (*shared).clone());
                None
            }
            Err(r3viewer::structure_output::OutputError::Cancelled) => return ExitCode::from(130),
            Err(_) => {
                eprintln!(
                    "r3viewer: no se puede preparar estructura; se conserva la fuente completa"
                );
                source.text = Arc::try_unwrap(original).unwrap_or_else(|shared| (*shared).clone());
                None
            }
        }
    } else {
        None
    };
    let content = if let Some(content) = structured {
        content
    } else if io::stdout().is_terminal() {
        let model = match r3viewer::presentation::SafeTextModel::new(source.text, &cancelled) {
            Ok(model) => model,
            Err(r3viewer::presentation::PrepareError::Cancelled) => return ExitCode::from(130),
            Err(r3viewer::presentation::PrepareError::Limit(message)) => {
                eprintln!("r3viewer: {message}");
                return ExitCode::from(1);
            }
        };
        // Container projections and non-string descriptors are never code.
        let literal_string = r3viewer::syntax_selection::textual_input(source.typed.as_ref());
        let plain = r3viewer::syntax_selection::SyntaxSelection::plain();
        let selected = if literal_string { &syntax } else { &plain };
        match r3viewer::inline::render(&model, selected, &presentation, cancelled.clone()) {
            Ok(output) => {
                if let Some(diagnostic) = output.diagnostic {
                    eprintln!("r3viewer: {}", text::safe_text(&diagnostic));
                }
                output.content
            }
            Err(r3viewer::presentation::PrepareError::Cancelled) => return ExitCode::from(130),
            Err(r3viewer::presentation::PrepareError::Limit(message)) => {
                eprintln!("r3viewer: {message}");
                return ExitCode::from(1);
            }
        }
    } else {
        if let Some(diagnostic) = syntax.diagnostic() {
            eprintln!("r3viewer: {}", text::safe_text(diagnostic));
        }
        let Some(content) = text::safe_text_cancelable(&source.text, &cancelled) else {
            return ExitCode::from(130);
        };
        content
    };
    match output::write_cancelable(content, &cancelled) {
        Ok(()) => ExitCode::SUCCESS,
        Err(output::OutputError::Cancelled) => ExitCode::from(130),
        Err(output::OutputError::Io(err)) if err.kind() == io::ErrorKind::BrokenPipe => {
            ExitCode::SUCCESS
        }
        Err(output::OutputError::Io(err)) => {
            eprintln!("r3viewer: salida fallida: {err}");
            ExitCode::from(1)
        }
    }
}
