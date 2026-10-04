use std::{
    io::{self, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};

#[derive(Debug)]
pub enum OutputError {
    Cancelled,
    Io(io::Error),
}

fn write_cancelable_with(
    writer: impl Write + Send + 'static,
    content: String,
    cancelled: &AtomicBool,
) -> Result<(), OutputError> {
    if cancelled.load(Ordering::Relaxed) {
        return Err(OutputError::Cancelled);
    }
    let (sender, receiver) = mpsc::sync_channel(1);
    // Solo salida sin raw-mode: si el consumidor bloquea, main puede terminar
    // y el SO cierra el trabajador. No posee estado de sesión que restaurar.
    thread::spawn(move || {
        let mut writer = writer;
        let result = writer
            .write_all(content.as_bytes())
            .and_then(|()| writer.flush());
        let _ = sender.send(result);
    });
    loop {
        if cancelled.load(Ordering::Relaxed) {
            return Err(OutputError::Cancelled);
        }
        match receiver.recv_timeout(Duration::from_millis(25)) {
            Ok(result) => {
                if cancelled.load(Ordering::Relaxed) {
                    return Err(OutputError::Cancelled);
                }
                return result.map_err(OutputError::Io);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => (),
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(OutputError::Io(io::Error::other(
                    "falló el trabajador de salida",
                )));
            }
        }
    }
}

pub fn write_cancelable(content: String, cancelled: &AtomicBool) -> Result<(), OutputError> {
    write_cancelable_with(io::stdout(), content, cancelled)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    #[test]
    fn cancellation_releases_main_even_when_consumer_does_not_read() {
        struct Blocked;
        impl Write for Blocked {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                thread::sleep(Duration::from_secs(1));
                Ok(bytes.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let cancelled = Arc::new(AtomicBool::new(false));
        let signal = cancelled.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(30));
            signal.store(true, Ordering::Relaxed);
        });
        let started = std::time::Instant::now();
        assert!(matches!(
            write_cancelable_with(Blocked, "data".into(), &cancelled),
            Err(OutputError::Cancelled)
        ));
        assert!(started.elapsed() < Duration::from_millis(250));
    }
}
