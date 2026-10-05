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
    let cli = Cli::parse();
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
    let Some(content) = text::safe_text_cancelable(&source.text, &cancelled) else {
        return ExitCode::from(130);
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
