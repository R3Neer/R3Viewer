//! The one-shot application owns its decode worker; cancellation never waits
//! for a blocked pipe or EOF after End. The persistent Nu companion releases
//! its own worker through its separately tested SDK cancellation patch.
use crate::{
    cli::Cli,
    input::{self, InputError, TextSource},
};
use r3viewer_wire::{Origin, Payload, Reader, Start, TypedInput, Value, WireError};
use std::{
    io,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

const INACTIVITY: Duration = Duration::from_secs(30);
enum Stage {
    Start(Start),
    Complete(Result<(TypedInput, String), WireError>),
}
fn error(err: WireError) -> InputError {
    InputError::new(
        if matches!(err, WireError::Cancelled) {
            130
        } else {
            1
        },
        err.to_string(),
    )
}
pub fn load(cli: &Cli, cancelled: Arc<AtomicBool>) -> Result<TextSource, InputError> {
    let (send, receive) = mpsc::sync_channel(1);
    let (proceed, decision) = mpsc::sync_channel(1);
    let progress = Arc::new(AtomicU64::new(0));
    let worker_flag = cancelled.clone();
    let worker_progress = progress.clone();
    thread::spawn(move || {
        let mut reader =
            Reader::new(io::stdin(), worker_flag.clone()).with_progress(worker_progress);
        let start = match reader.read_start() {
            Ok(start) => start,
            Err(err) => {
                let _ = send.send(Stage::Complete(Err(err)));
                return;
            }
        };
        if send.send(Stage::Start(start)).is_err() {
            return;
        }
        loop {
            if worker_flag.load(Ordering::Relaxed) {
                return;
            }
            match decision.recv_timeout(Duration::from_millis(25)) {
                Ok(()) => break,
                Err(mpsc::RecvTimeoutError::Timeout) => (),
                Err(mpsc::RecvTimeoutError::Disconnected) => return,
            }
        }
        let result = reader.finish().and_then(|input| {
            if worker_flag.load(Ordering::Relaxed) {
                return Err(WireError::Cancelled);
            }
            let text = input.typed_text()?;
            if worker_flag.load(Ordering::Relaxed) {
                return Err(WireError::Cancelled);
            }
            Ok((input, text))
        });
        let _ = send.send(Stage::Complete(result));
    });
    let mut last_progress = 0;
    let mut last_activity = Instant::now();
    let mut start_context = None;
    loop {
        if cancelled.load(Ordering::Relaxed) {
            return Err(InputError::new(130, "lectura Nu cancelada"));
        }
        let now_progress = progress.load(Ordering::Relaxed);
        if now_progress != last_progress {
            last_progress = now_progress;
            last_activity = Instant::now();
        }
        if last_activity.elapsed() >= INACTIVITY {
            cancelled.store(true, Ordering::Relaxed);
            return Err(InputError::new(
                1,
                "entrada Nu sin progreso durante 30 s; Ping no renueva el plazo",
            ));
        }
        match receive.recv_timeout(Duration::from_millis(25)) {
            Ok(Stage::Start(start)) => {
                if cli.source.is_some() && start.origin != Origin::Absent {
                    cancelled.store(true, Ordering::Relaxed);
                    return Err(InputError::new(
                        2,
                        "elige un único origen: argumento o valor de pipeline Nushell",
                    ));
                }
                if cli.source.is_none() && start.origin == Origin::Absent {
                    cancelled.store(true, Ordering::Relaxed);
                    return Err(InputError::new(
                        2,
                        "falta entrada: viewer archivo o contenido por pipeline",
                    ));
                }
                start_context = Some(start);
                proceed
                    .send(())
                    .map_err(|_| InputError::new(1, "falló el lector Nu"))?;
            }
            Ok(Stage::Complete(result)) => {
                if cancelled.load(Ordering::Relaxed) {
                    return Err(InputError::new(130, "lectura Nu cancelada"));
                }
                let (typed, text) = result.map_err(error)?;
                let start = start_context.ok_or_else(|| InputError::new(1, "falta contexto Nu"))?;
                if cli.source.is_some() {
                    let mut plain = cli.clone();
                    plain.nu_wire = false;
                    if !plain.value
                        && let Some(source) = plain.source.as_mut()
                        && !Path::new(source).is_absolute()
                        && (plain.path
                            || !(source.starts_with("https://") || source.starts_with("http://")))
                    {
                        *source = Path::new(&start.cwd)
                            .join(&*source)
                            .to_string_lossy()
                            .into_owned();
                    }
                    return input::load(&plain, true, cancelled);
                }
                if cli.path {
                    let Payload::SingleValue(Value::String(path)) = &typed.payload else {
                        return Err(InputError::new(
                            2,
                            "--path requiere una cadena única como ruta local",
                        ));
                    };
                    let mut plain = cli.clone();
                    plain.nu_wire = false;
                    plain.source = Some(if Path::new(path).is_absolute() {
                        path.clone()
                    } else {
                        Path::new(&start.cwd)
                            .join(path)
                            .to_string_lossy()
                            .into_owned()
                    });
                    return input::load(&plain, true, cancelled);
                }
                return Ok(TextSource {
                    title: "Datos Nushell".into(),
                    text,
                    typed: Some(typed),
                });
            }
            Err(mpsc::RecvTimeoutError::Timeout) => (),
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(InputError::new(1, "falló el trabajador Nu"));
            }
        }
    }
}
