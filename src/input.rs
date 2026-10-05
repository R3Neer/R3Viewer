use crate::cli::Cli;
use std::{
    fs::File,
    io::{self, Read},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};

pub const INPUT_LIMIT: usize = 128 * 1024 * 1024;

#[derive(Debug)]
pub struct InputError {
    pub code: u8,
    pub message: String,
}
impl InputError {
    pub fn new(code: u8, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

pub struct TextSource {
    pub title: String,
    pub text: String,
    pub typed: Option<r3viewer_wire::TypedInput>,
}

fn read_bounded(mut reader: impl Read, limit: usize) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 16384];
    loop {
        let count = reader.read(&mut chunk)?;
        if count == 0 {
            return Ok(bytes);
        }
        if count > limit.saturating_sub(bytes.len()) {
            return Err(io::Error::other(format!(
                "entrada excede el límite de {limit} bytes; no se muestra un prefijo incompleto"
            )));
        }
        bytes.extend_from_slice(&chunk[..count]);
    }
}

// El hilo bloqueado no posee terminal ni estado persistente; cancelar libera main,
// y al salir el proceso se eliminan sus handles. No esperamos EOF para cancelar.
fn read_cancelable(
    reader: impl Read + Send + 'static,
    cancelled: &AtomicBool,
) -> Result<Vec<u8>, InputError> {
    read_operation_cancelable(move || read_bounded(reader, INPUT_LIMIT), cancelled)
}

// Opening a FIFO, remote filesystem or device may itself block. Keep open and
// read under the same cancellation boundary, not just reads after File::open.
fn read_operation_cancelable(
    operation: impl FnOnce() -> io::Result<Vec<u8>> + Send + 'static,
    cancelled: &AtomicBool,
) -> Result<Vec<u8>, InputError> {
    let (sender, receiver) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let _ = sender.send(operation());
    });
    loop {
        if cancelled.load(Ordering::Relaxed) {
            return Err(InputError::new(130, "lectura cancelada"));
        }
        match receiver.recv_timeout(Duration::from_millis(25)) {
            Ok(result) => return result.map_err(|err| InputError::new(1, err.to_string())),
            Err(mpsc::RecvTimeoutError::Timeout) => (),
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(InputError::new(1, "falló el trabajador de lectura"));
            }
        }
    }
}

pub fn load(
    cli: &Cli,
    stdin_is_terminal: bool,
    cancelled: Arc<AtomicBool>,
) -> Result<TextSource, InputError> {
    if cli.nu_wire {
        return crate::nu_input::load(cli, cancelled);
    }
    if cli.source.is_none() && stdin_is_terminal {
        return Err(InputError::new(
            2,
            "falta entrada: r3viewer archivo.txt o r3viewer --value texto",
        ));
    }
    if cli.path && cli.source.is_none() {
        return Err(InputError::new(
            2,
            "--path desde stdin textual requiere el puente tipado Nu, todavía pendiente; pasa la ruta como argumento",
        ));
    }
    let piped = if stdin_is_terminal {
        Vec::new()
    } else if cli.source.is_some() {
        // Un byte basta para rechazar dos orígenes; no consumir el pipe completo.
        read_cancelable(io::stdin().take(1), &cancelled)?
    } else {
        read_cancelable(io::stdin(), &cancelled)?
    };
    if cli.source.is_some() && !piped.is_empty() {
        return Err(InputError::new(
            2,
            "elige un único origen: argumento o contenido de stdin",
        ));
    }
    let (title, bytes) = match &cli.source {
        Some(source) if cli.value => {
            if source.len() > INPUT_LIMIT {
                return Err(InputError::new(
                    1,
                    "contenido literal excede el límite de 128 MiB",
                ));
            }
            ("Contenido literal".to_owned(), source.as_bytes().to_vec())
        }
        Some(source) => {
            if !cli.path && (source.starts_with("https://") || source.starts_with("http://")) {
                return Err(InputError::new(
                    1,
                    "lectura de URL pendiente de E08; S01 no inicia tráfico de red",
                ));
            }
            let path = source.clone();
            let bytes = read_operation_cancelable(
                move || {
                    let file = File::open(&path).map_err(|err| {
                        io::Error::new(err.kind(), format!("no se puede abrir {path:?}: {err}"))
                    })?;
                    read_bounded(file, INPUT_LIMIT)
                },
                &cancelled,
            )?;
            (source.clone(), bytes)
        }
        None => ("Entrada estándar".to_owned(), piped),
    };
    let text = String::from_utf8(bytes).map_err(|err| {
        InputError::new(
            1,
            format!(
                "entrada no es UTF-8 válido (byte {}); visor binario pendiente",
                err.utf8_error().valid_up_to()
            ),
        )
    })?;
    Ok(TextSource {
        title,
        text,
        typed: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn limit_accepts_exact_and_rejects_overflow_without_prefix() {
        assert_eq!(read_bounded(&b"abcd"[..], 4).unwrap(), b"abcd");
        assert!(read_bounded(&b"abcde"[..], 4).is_err());
    }
    #[test]
    fn cancellation_does_not_wait_for_a_blocked_reader() {
        struct Blocked;
        impl Read for Blocked {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                thread::sleep(Duration::from_secs(1));
                Ok(0)
            }
        }
        let cancelled = AtomicBool::new(true);
        let started = std::time::Instant::now();
        assert_eq!(read_cancelable(Blocked, &cancelled).unwrap_err().code, 130);
        assert!(started.elapsed() < Duration::from_millis(250));
    }
}
