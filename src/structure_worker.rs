//! Shared one-shot source-only structure worker transport.
use crate::{
    structure_model::*,
    structure_protocol::{self as ipc, Counts, Frame, Message, WireNode},
};
use std::{
    io,
    sync::{Arc, atomic::AtomicBool},
};
fn output(
    model: &Structure,
    format: StructureFormat,
    syntax: usize,
    key_max: usize,
    generation: u64,
    request: u64,
    out: &mut impl io::Write,
) -> io::Result<()> {
    let send =
        |out: &mut _, message| ipc::write_frame(out, &Frame::new(generation, request, message));
    let parts = model
        .raw_declarations()
        .map(|(_, parts)| parts.len())
        .sum::<usize>();
    send(
        out,
        Message::Begin(Counts {
            format,
            nodes: model.node_count() as u32,
            entries: model.entry_count() as u32,
            declarations: model.declarations().len() as u32,
            parts: parts as u32,
            cooked: model.raw_cooked().len() as u32,
            root: model.root().id(),
            syntax_depth: syntax as u32,
            key_components: key_max as u32,
        }),
    )?;
    let mut batch = Vec::new();
    let mut offset = 0;
    for (spec, entries) in model.raw_nodes() {
        batch.push(WireNode {
            spec,
            entries: entries.len() as u32,
        });
        if batch.len() == ipc::BATCH {
            let len = batch.len() as u32;
            send(
                out,
                Message::Nodes {
                    offset,
                    nodes: std::mem::take(&mut batch),
                },
            )?;
            offset += len;
        }
    }
    if !batch.is_empty() {
        send(
            out,
            Message::Nodes {
                offset,
                nodes: batch,
            },
        )?;
    }
    for (index, (_, range)) in model.raw_nodes().enumerate() {
        let mut at = range.start;
        while at < range.end {
            let count = (range.end - at).min(ipc::BATCH);
            let entries = model.raw_entries().skip(at).take(count).collect();
            send(
                out,
                Message::Entries {
                    parent: NodeId(index as u32),
                    entries,
                },
            )?;
            at += count;
        }
    }
    for (offset, bytes) in model.raw_cooked().chunks(ipc::CHUNK).enumerate() {
        send(
            out,
            Message::Cooked {
                offset: (offset * ipc::CHUNK) as u32,
                bytes: bytes.to_vec(),
            },
        )?;
    }
    for (spec, parts) in model.raw_declarations() {
        send(
            out,
            Message::Declaration {
                spec,
                parts: parts.to_vec(),
            },
        )?;
    }
    send(out, Message::End)
}
/// Dispatch before CLI/config/terminal initialization; private source-only process.
pub fn run_private_stdio() -> io::Result<()> {
    let (send, receive) = std::sync::mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("structure-lease".into())
        .spawn(move || {
            let stdin = io::stdin();
            let mut input = stdin.lock();
            let mut source_ended = false;
            loop {
                let frame = match ipc::read_frame(&mut input) {
                    Ok(Some(f)) => f,
                    _ => std::process::exit(85),
                };
                if source_ended {
                    std::process::exit(86);
                }
                source_ended = matches!(frame.message, Message::SourceEnd);
                if send.try_send(frame).is_err() {
                    std::process::exit(86);
                }
            }
        })?;
    let stdout = io::stdout();
    let mut out = stdout.lock();
    ipc::write_frame(&mut out, &Frame::new(0, 0, Message::Boot))?;
    let init = receive
        .recv()
        .map_err(|_| io::Error::other("missing Init"))?;
    let (size, format) = match init.message {
        Message::Init { format, bytes } if bytes as usize <= MAX_STRUCTURE_SOURCE => {
            (bytes as usize, format)
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid structure Init",
            ));
        }
    };
    let generation = init.generation;
    let request = init.request;
    let mut source = Vec::new();
    source
        .try_reserve_exact(size)
        .map_err(|_| io::Error::other("source allocation"))?;
    ipc::write_frame(
        &mut out,
        &Frame::new(generation, request, Message::Ack { offset: 0 }),
    )?;
    loop {
        let f = receive
            .recv()
            .map_err(|_| io::Error::other("source disconnected"))?;
        if f.generation != generation || f.request != request {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "source identity",
            ));
        }
        match f.message {
            Message::Source { offset, bytes }
                if offset as usize == source.len()
                    && !bytes.is_empty()
                    && source.len() + bytes.len() <= size =>
            {
                source.extend_from_slice(&bytes);
                ipc::write_frame(
                    &mut out,
                    &Frame::new(
                        generation,
                        request,
                        Message::Ack {
                            offset: source.len() as u32,
                        },
                    ),
                )?;
            }
            Message::SourceEnd if source.len() == size => break,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "source sequence",
                ));
            }
        }
    }
    let source = Arc::new(
        String::from_utf8(source)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "source UTF-8"))?,
    );
    let cancel = AtomicBool::new(false);
    let result = match format {
        StructureFormat::Toml => crate::toml_worker::project(source, generation, &cancel),
        StructureFormat::Json => crate::json_worker::project(source, generation, &cancel),
    };
    match result {
        Ok((model, syntax, key_max)) => output(
            &model, format, syntax, key_max, generation, request, &mut out,
        ),
        Err(reason) => ipc::write_frame(
            &mut out,
            &Frame::new(generation, request, Message::Failure(reason as u8)),
        ),
    }
}
