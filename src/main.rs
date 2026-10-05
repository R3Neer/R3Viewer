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
    let mut private_args = std::env::args_os().skip(1);
    if private_args
        .next()
        .is_some_and(|arg| arg == "--syntax-worker")
        && private_args.next().is_none()
    {
        return match r3viewer::syntax_worker::run_private_stdio() {
            Ok(()) => ExitCode::SUCCESS,
            Err(_) => ExitCode::from(75),
        };
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
    let source = match input::load(&cli, io::stdin().is_terminal(), cancelled.clone()) {
        Ok(source) => source,
        Err(err) => {
            eprintln!("r3viewer: {}", text::safe_text(&err.message));
            return ExitCode::from(err.code);
        }
    };
    if cancelled.load(Ordering::Relaxed) {
        return ExitCode::from(130);
    }
    let syntax = r3viewer::syntax_selection::SyntaxSelection::select(&cli, &source.origin);
    if !cli.inline && io::stdout().is_terminal() && session::available() {
        let model = match r3viewer::presentation::SafeTextModel::new(source.text, &cancelled) {
            Ok(model) => Arc::new(model),
            Err(r3viewer::presentation::PrepareError::Cancelled) => return ExitCode::from(130),
            Err(r3viewer::presentation::PrepareError::Limit(message)) => {
                eprintln!("r3viewer: {message}");
                return ExitCode::from(1);
            }
        };
        match r3viewer::viewer_session::run(
            model,
            source.typed.map(Arc::new),
            &text::safe_text(&source.title),
            &presentation,
            &syntax,
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
    let content = if io::stdout().is_terminal() {
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
