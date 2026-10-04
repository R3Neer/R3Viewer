use std::io::{self, Read};
use std::sync::{
    Arc,
    atomic::AtomicBool,
    mpsc::{self, Receiver, SyncSender},
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use nu_protocol::{ByteStreamType, PipelineData, Signals};
use r3viewer_wire::{self as wire, Frame, Origin};

use crate::mapper::{Mapper, check_cancel};

const POLL_INTERVAL: Duration = Duration::from_millis(25);
const QUEUE_FRAMES: usize = 2;

enum Message {
    Frame(Vec<u8>),
    End(Vec<u8>),
    Failure(io::Error),
}

fn wire_error(error: wire::WireError) -> io::Error {
    match error {
        wire::WireError::Io(error) => error,
        wire::WireError::Cancelled => io::Error::new(io::ErrorKind::Interrupted, error),
        error => io::Error::new(io::ErrorKind::InvalidData, error),
    }
}
fn encode(frame: &Frame, cancel: &AtomicBool) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    wire::write_frame_cancelable(&mut bytes, frame, cancel).map_err(wire_error)?;
    Ok(bytes)
}
fn send(tx: &SyncSender<Message>, frame: Frame, cancel: &AtomicBool) -> io::Result<()> {
    check_cancel(cancel)?;
    let bytes = encode(&frame, cancel)?;
    check_cancel(cancel)?;
    tx.send(Message::Frame(bytes))
        .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "R3Viewer consumer closed"))
}
fn origin(input: &PipelineData) -> Origin {
    match input {
        PipelineData::Empty => Origin::Absent,
        PipelineData::Value(_, _) => Origin::SingleValue,
        PipelineData::ListStream(_, _) => Origin::ListStream,
        PipelineData::ByteStream(stream, _) => Origin::ByteStream(match stream.type_() {
            ByteStreamType::String => wire::ByteStreamKind::String,
            ByteStreamType::Binary => wire::ByteStreamKind::Binary,
            ByteStreamType::Unknown => wire::ByteStreamKind::Unknown,
        }),
    }
}
fn initial(cwd: String, origin: Origin, cancel: &AtomicBool) -> io::Result<Vec<u8>> {
    check_cancel(cancel)?;
    let mut bytes = Vec::new();
    wire::write_magic(&mut bytes).map_err(wire_error)?;
    wire::write_frame_cancelable(
        &mut bytes,
        &Frame::Start {
            version: wire::VERSION,
            cwd,
            origin,
        },
        cancel,
    )
    .map_err(wire_error)?;
    Ok(bytes)
}

fn produce(input: PipelineData, tx: &SyncSender<Message>, cancel: &AtomicBool) -> io::Result<()> {
    let mut mapper = Mapper::new(cancel);
    check_cancel(cancel)?;
    match input {
        PipelineData::Empty => {}
        PipelineData::Value(value, _) => {
            send(tx, Frame::Value(mapper.value(&value)?), cancel)?;
        }
        PipelineData::ListStream(stream, _) => {
            let mut stream = stream.into_iter();
            loop {
                check_cancel(cancel)?;
                let Some(value) = stream.next() else {
                    break;
                };
                check_cancel(cancel)?;
                send(tx, Frame::Value(mapper.value(&value)?), cancel)?;
            }
        }
        PipelineData::ByteStream(stream, _) => {
            if let Some(mut reader) = stream.reader() {
                let mut buffer = vec![0; wire::BYTE_CHUNK_BYTES];
                loop {
                    check_cancel(cancel)?;
                    let count = match reader.read(&mut buffer) {
                        Ok(count) => count,
                        Err(error) if error.kind() == io::ErrorKind::Interrupted => {
                            check_cancel(cancel)?;
                            continue;
                        }
                        Err(error) => return Err(error),
                    };
                    check_cancel(cancel)?;
                    if count == 0 {
                        break;
                    }
                    // UTF-8 is validated by the receiver after concatenation;
                    // multibyte characters are permitted to cross chunks.
                    send(tx, Frame::Bytes(buffer[..count].to_vec()), cancel)?;
                }
            }
        }
    }
    // SDK list/byte wrappers may turn cancellation into apparent EOF. The
    // invocation's own latched flag must be checked before allowing End.
    check_cancel(cancel)
}

/// Owns a bounded producer and complete serialized frames. Ping can only be
/// selected after the current frame (prefix included) has been fully emitted.
pub(crate) struct OutputReader {
    current: Vec<u8>,
    offset: usize,
    rx: Option<Receiver<Message>>,
    signals: Signals,
    cancel: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
    ended: bool,
    failure: Option<(io::ErrorKind, String)>,
    emitted: usize,
    total_limit: usize,
    wire_nodes: usize,
    wire_node_limit: usize,
}
impl OutputReader {
    pub(crate) fn new(input: PipelineData, cwd: String, signals: Signals) -> io::Result<Self> {
        let cancel = signals.interrupt_flag().ok_or_else(|| {
            io::Error::other("Nushell invocation has no owned cancellation token")
        })?;
        let initial = initial(cwd, origin(&input), &cancel)?;
        Self::spawn(initial, signals, move |tx, cancel| {
            produce(input, tx, cancel)
        })
    }

    fn spawn(
        initial: Vec<u8>,
        signals: Signals,
        producer: impl FnOnce(&SyncSender<Message>, &AtomicBool) -> io::Result<()> + Send + 'static,
    ) -> io::Result<Self> {
        let cancel = signals.interrupt_flag().ok_or_else(|| {
            io::Error::other("Nushell invocation has no owned cancellation token")
        })?;
        // The initial vector contains MAGIC + the Start frame's length prefix.
        let start_payload = initial
            .get(wire::MAGIC.len() + 4..)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing Start frame"))?;
        let wire_nodes =
            wire::messagepack_node_count(start_payload, &cancel).map_err(wire_error)?;
        if initial.len() > wire::MAX_TOTAL_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "R3Viewer total wire byte limit",
            ));
        }
        let (tx, rx) = mpsc::sync_channel(QUEUE_FRAMES);
        let worker_cancel = cancel.clone();
        let worker = thread::Builder::new()
            .name("r3viewer-encode".into())
            .spawn(move || {
                let result = producer(&tx, &worker_cancel).and_then(|()| {
                    check_cancel(&worker_cancel)?;
                    let end = encode(&Frame::End, &worker_cancel)?;
                    check_cancel(&worker_cancel)?;
                    Ok(end)
                });
                let message = match result {
                    Ok(end) => Message::End(end),
                    Err(error) => Message::Failure(error),
                };
                // Closing the receiver before joining releases a blocked send.
                let _ = tx.send(message);
            })?;
        Ok(Self {
            current: initial,
            offset: 0,
            rx: Some(rx),
            signals,
            cancel,
            worker: Some(worker),
            ended: false,
            failure: None,
            emitted: 0,
            total_limit: wire::MAX_TOTAL_BYTES,
            wire_nodes,
            wire_node_limit: wire::MAX_WIRE_NODES,
        })
    }

    fn fail(&mut self, error: io::Error) -> io::Error {
        let kind = error.kind();
        let message = error.to_string();
        self.failure = Some((kind, message.clone()));
        self.signals.trigger();
        drop(self.rx.take());
        self.current.clear();
        self.offset = 0;
        io::Error::new(kind, message)
    }

    fn read_checked(&mut self, output: &mut [u8]) -> io::Result<usize> {
        check_cancel(&self.cancel)?;
        if self.offset == self.current.len() {
            if self.ended {
                return Ok(0);
            }
            let rx = self
                .rx
                .as_ref()
                .ok_or_else(|| io::Error::other("R3Viewer producer closed"))?;
            let next = match rx.recv_timeout(POLL_INTERVAL) {
                Ok(Message::Frame(frame)) => frame,
                Ok(Message::End(frame)) => {
                    self.ended = true;
                    frame
                }
                Ok(Message::Failure(error)) => return Err(error),
                Err(mpsc::RecvTimeoutError::Timeout) => encode(&Frame::Ping, &self.cancel)?,
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "R3Viewer producer ended without End",
                    ));
                }
            };
            check_cancel(&self.cancel)?;
            // Admit each exact MessagePack object once, regardless of how
            // many Read calls fragment its prefix and payload afterward.
            let payload = next.get(4..).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "missing wire frame prefix")
            })?;
            let nodes = wire::messagepack_node_count(payload, &self.cancel).map_err(wire_error)?;
            self.wire_nodes = self
                .wire_nodes
                .checked_add(nodes)
                .filter(|nodes| *nodes <= self.wire_node_limit)
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "R3Viewer wire node limit")
                })?;
            self.current = next;
            self.offset = 0;
        }
        // Count every actual wire byte, including magic, prefixes and Pings.
        // Reject a frame before emitting any of it if it cannot fit.
        let remaining_frame = self.current.len() - self.offset;
        if remaining_frame > self.total_limit.saturating_sub(self.emitted) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "R3Viewer total wire byte limit",
            ));
        }
        let count = output.len().min(remaining_frame);
        check_cancel(&self.cancel)?;
        output[..count].copy_from_slice(&self.current[self.offset..self.offset + count]);
        self.offset += count;
        self.emitted += count;
        Ok(count)
    }
}
impl Read for OutputReader {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() {
            return Ok(0);
        }
        if let Some((kind, message)) = &self.failure {
            return Err(io::Error::new(*kind, message.clone()));
        }
        match self.read_checked(output) {
            Ok(count) => Ok(count),
            Err(error) => Err(self.fail(error)),
        }
    }
}
impl Drop for OutputReader {
    fn drop(&mut self) {
        self.signals.trigger();
        drop(self.rx.take());
        if let Some(worker) = self.worker.take() {
            // No lock is held while joining. A panic is an abnormal EOF to the
            // reader, and must not become another panic from Drop.
            let _ = worker.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn signals() -> Signals {
        Signals::new(Arc::new(AtomicBool::new(false)))
    }
    fn start(origin: Origin) -> Vec<u8> {
        initial("test cwd".into(), origin, &AtomicBool::new(false)).unwrap()
    }
    struct WorkerCount(Arc<AtomicUsize>);
    impl Drop for WorkerCount {
        fn drop(&mut self) {
            self.0.fetch_sub(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn initial_is_available_without_reading_input_and_idle_ping_is_a_real_frame() {
        let signals = signals();
        let start = start(Origin::Absent);
        let start_len = start.len();
        let mut reader = OutputReader::spawn(start, signals.clone(), |_, cancel| {
            while check_cancel(cancel).is_ok() {
                thread::sleep(Duration::from_millis(1));
            }
            check_cancel(cancel)
        })
        .unwrap();
        let mut received = vec![0; start_len];
        reader.read_exact(&mut received).unwrap();
        let mut ping = [0; 128];
        let n = reader.read(&mut ping).unwrap();
        assert_eq!(
            &ping[..n],
            encode(&Frame::Ping, &AtomicBool::new(false)).unwrap()
        );
        received.extend_from_slice(&ping[..n]);
        received.extend(encode(&Frame::End, &AtomicBool::new(false)).unwrap());
        drop(reader);
        let mut wire_reader =
            wire::Reader::new(received.as_slice(), Arc::new(AtomicBool::new(false)));
        assert_eq!(wire_reader.read_start().unwrap().origin, Origin::Absent);
        assert_eq!(wire_reader.finish().unwrap().payload, wire::Payload::Absent);
        assert!(signals.interrupted());
    }

    #[test]
    fn small_reads_never_interleave_ping_inside_a_frame() {
        let signals = signals();
        let initial = start(Origin::SingleValue);
        let mut reader = OutputReader::spawn(initial, signals, |tx, cancel| {
            send(
                tx,
                Frame::Value(wire::Value::String("é👩‍💻".repeat(100))),
                cancel,
            )
        })
        .unwrap();
        let mut bytes = Vec::new();
        let mut buffer = [0; 1];
        while reader.read(&mut buffer).unwrap() != 0 {
            bytes.push(buffer[0]);
        }
        let mut wire_reader = wire::Reader::new(bytes.as_slice(), Arc::new(AtomicBool::new(false)));
        wire_reader.read_start().unwrap();
        assert_eq!(
            wire_reader.finish().unwrap().payload,
            wire::Payload::SingleValue(wire::Value::String("é👩‍💻".repeat(100)))
        );
    }

    #[test]
    fn drop_joins_idle_worker_and_worker_blocked_on_full_queue() {
        for full_queue in [false, true] {
            let live = Arc::new(AtomicUsize::new(0));
            let observed = live.clone();
            let (ready_tx, ready_rx) = mpsc::channel();
            let reader =
                OutputReader::spawn(start(Origin::ListStream), signals(), move |tx, cancel| {
                    observed.fetch_add(1, Ordering::SeqCst);
                    let _guard = WorkerCount(observed);
                    if full_queue {
                        for _ in 0..QUEUE_FRAMES {
                            send(tx, Frame::Value(wire::Value::Int(1)), cancel)?;
                        }
                        ready_tx.send(()).unwrap();
                        send(tx, Frame::Value(wire::Value::Int(2)), cancel)
                    } else {
                        ready_tx.send(()).unwrap();
                        while check_cancel(cancel).is_ok() {
                            thread::sleep(Duration::from_millis(1));
                        }
                        check_cancel(cancel)
                    }
                })
                .unwrap();
            ready_rx.recv_timeout(Duration::from_secs(1)).unwrap();
            let (dropped_tx, dropped_rx) = mpsc::channel();
            let dropper = thread::spawn(move || {
                drop(reader);
                dropped_tx.send(()).unwrap();
            });
            dropped_rx
                .recv_timeout(Duration::from_millis(500))
                .expect("Drop must release and join its worker");
            dropper.join().unwrap();
            assert_eq!(live.load(Ordering::SeqCst), 0);
        }
    }

    #[test]
    fn cancellation_and_worker_failure_never_emit_end() {
        let token = signals();
        let initial = start(Origin::Absent);
        let start_len = initial.len();
        let mut reader = OutputReader::spawn(initial, token.clone(), |_, _| {
            Err(io::Error::other("source failure"))
        })
        .unwrap();
        let mut bytes = vec![0; start_len];
        reader.read_exact(&mut bytes).unwrap();
        let mut buffer = [0; 128];
        loop {
            match reader.read(&mut buffer) {
                Err(error) => {
                    assert_eq!(error.to_string(), "source failure");
                    break;
                }
                Ok(0) => panic!("failure must not become EOF"),
                Ok(n) => bytes.extend_from_slice(&buffer[..n]),
            }
        }
        assert!(reader.read(&mut buffer).is_err());
        drop(reader);
        let mut wire_reader = wire::Reader::new(bytes.as_slice(), Arc::new(AtomicBool::new(false)));
        wire_reader.read_start().unwrap();
        assert!(wire_reader.finish().is_err());

        let token = signals();
        let mut cancelled_reader =
            OutputReader::spawn(start(Origin::Absent), token.clone(), |_, _| Ok(())).unwrap();
        token.trigger();
        assert_eq!(
            cancelled_reader.read(&mut buffer).unwrap_err().kind(),
            io::ErrorKind::Interrupted
        );
    }

    #[test]
    fn ping_counts_toward_total_budget() {
        let initial = start(Origin::Absent);
        let start_len = initial.len();
        let mut reader = OutputReader::spawn(initial, signals(), |_, cancel| {
            while check_cancel(cancel).is_ok() {
                thread::sleep(Duration::from_millis(1));
            }
            check_cancel(cancel)
        })
        .unwrap();
        reader.total_limit =
            start_len + encode(&Frame::Ping, &AtomicBool::new(false)).unwrap().len() - 1;
        let mut first = vec![0; start_len];
        reader.read_exact(&mut first).unwrap();
        assert_eq!(
            reader.read(&mut [0; 128]).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn unknown_and_split_utf8_byte_streams_preserve_origin() {
        for kind in [
            ByteStreamType::String,
            ByteStreamType::Binary,
            ByteStreamType::Unknown,
        ] {
            let mut raw = vec![b'x'; wire::BYTE_CHUNK_BYTES - 1];
            raw.extend_from_slice("€".as_bytes());
            let stream = nu_protocol::ByteStream::read(
                io::Cursor::new(raw.clone()),
                nu_protocol::Span::unknown(),
                Signals::empty(),
                kind,
            );
            let mut reader = OutputReader::new(
                PipelineData::byte_stream(stream, None),
                "cwd".into(),
                signals(),
            )
            .unwrap();
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).unwrap();
            let mut wire_reader =
                wire::Reader::new(bytes.as_slice(), Arc::new(AtomicBool::new(false)));
            let start = wire_reader.read_start().unwrap();
            let expected_kind = match kind {
                ByteStreamType::String => wire::ByteStreamKind::String,
                ByteStreamType::Binary => wire::ByteStreamKind::Binary,
                ByteStreamType::Unknown => wire::ByteStreamKind::Unknown,
            };
            assert_eq!(start.origin, Origin::ByteStream(expected_kind));
            assert_eq!(
                wire_reader.finish().unwrap().payload,
                wire::Payload::ByteStream {
                    kind: expected_kind,
                    bytes: raw
                }
            );
        }
    }
    #[test]
    fn ping_counts_toward_wire_node_budget() {
        let initial = start(Origin::Absent);
        let start_len = initial.len();
        let mut reader = OutputReader::spawn(initial, signals(), |_, cancel| {
            while check_cancel(cancel).is_ok() {
                thread::sleep(Duration::from_millis(1));
            }
            check_cancel(cancel)
        })
        .unwrap();
        let ping = encode(&Frame::Ping, &AtomicBool::new(false)).unwrap();
        reader.wire_node_limit = reader.wire_nodes
            + wire::messagepack_node_count(&ping[4..], &AtomicBool::new(false)).unwrap()
            - 1;
        reader.read_exact(&mut vec![0; start_len]).unwrap();
        assert_eq!(
            reader.read(&mut [0; 128]).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }
}
