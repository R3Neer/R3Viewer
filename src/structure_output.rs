//! Complete, bounded human representation. This is not TOML serialization.
use std::sync::atomic::{AtomicBool, Ordering};

pub const OUTPUT_LIMIT: usize = 256 * 1024 * 1024;
#[derive(Debug, PartialEq, Eq)]
pub enum OutputError {
    Cancelled,
    Limit,
    InvalidModel,
}
struct Builder<'a> {
    text: String,
    cancel: &'a AtomicBool,
    limit: usize,
}
impl<'a> Builder<'a> {
    fn new(cancel: &'a AtomicBool, limit: usize) -> Self {
        Self {
            text: String::new(),
            cancel,
            limit,
        }
    }
    fn append(&mut self, text: &str) -> Result<(), OutputError> {
        if self.cancel.load(Ordering::Relaxed) {
            return Err(OutputError::Cancelled);
        }
        let len = self
            .text
            .len()
            .checked_add(text.len())
            .ok_or(OutputError::Limit)?;
        if len > self.limit {
            return Err(OutputError::Limit);
        }
        if len > self.text.capacity() {
            let target = self
                .text
                .capacity()
                .saturating_mul(2)
                .max(len)
                .max(4096)
                .min(self.limit);
            self.text
                .try_reserve_exact(target - self.text.len())
                .map_err(|_| OutputError::Limit)?;
        }
        if self.text.capacity() > self.limit {
            return Err(OutputError::Limit);
        }
        self.text.push_str(text);
        Ok(())
    }
    fn indent(&mut self, depth: usize) -> Result<(), OutputError> {
        for _ in 0..depth {
            self.append("  ")?;
        }
        Ok(())
    }
    fn quoted(&mut self, value: &str) -> Result<(), OutputError> {
        self.append("\"")?;
        let mut chunk_start = 0;
        for (byte, ch) in value.char_indices() {
            let escape = match ch {
                '"' => Some("\\\""),
                '\\' => Some("\\\\"),
                '\n' => Some("\\n"),
                '\r' => Some("\\r"),
                '\t' => Some("\\t"),
                _ => None,
            };
            if escape.is_some() || ch.is_control() {
                self.append(&value[chunk_start..byte])?;
                if let Some(escape) = escape {
                    self.append(escape)?;
                } else {
                    self.append(&format!("\\u{{{:04x}}}", ch as u32))?;
                }
                chunk_start = byte + ch.len_utf8();
            } else if byte.saturating_sub(chunk_start) >= 65536 {
                self.append(&value[chunk_start..byte])?;
                chunk_start = byte;
            }
        }
        self.append(&value[chunk_start..])?;
        self.append("\"")
    }
}
pub enum PreparedStructure {
    Rendered {
        content: String,
        diagnostic: Option<String>,
    },
    Fallback(String),
}
/// Inline/redirection has no syntax child: build and validate the whole arena,
/// reap the controller, then prepare bounded output before any document write.
pub fn prepare(
    source: std::sync::Arc<String>,
    format: crate::structure_model::StructureFormat,
    cancel: std::sync::Arc<AtomicBool>,
) -> Result<PreparedStructure, OutputError> {
    use crate::{
        structure::{Mode, StructureClient, StructureEvent, StructureFailure},
        worker_process::ReapTicket,
    };
    use std::time::{Duration, Instant};
    if cancel.load(Ordering::Relaxed) {
        return Err(OutputError::Cancelled);
    }
    let mut client = match StructureClient::new(cancel.clone(), format) {
        Ok(client) => client,
        Err(_) => {
            return Ok(PreparedStructure::Fallback(
                StructureFailure::WorkerExited.description().into(),
            ));
        }
    };
    client.set_document(source.clone(), 1);
    let start = Instant::now();
    let result = if !client.request(1, 1, Mode::Inline, ReapTicket::reaped()) {
        Err(StructureFailure::InvalidIpc)
    } else {
        loop {
            if cancel.load(Ordering::Relaxed) {
                client.cancel_request(1, 1);
                break Err(StructureFailure::WorkerExited);
            }
            if let Some(event) = client.poll() {
                match event {
                    StructureEvent::Ready {
                        generation: 1,
                        request_id: 1,
                        structure,
                    } if structure.generation() == 1
                        && std::sync::Arc::ptr_eq(&source, &structure.source_handle()) =>
                    {
                        break Ok(structure);
                    }
                    StructureEvent::Unavailable {
                        generation: 1,
                        request_id: 1,
                        reason,
                    } => break Err(reason),
                    _ => (),
                }
            }
            if start.elapsed() >= Duration::from_secs(5) {
                client.cancel_request(1, 1);
                break Err(StructureFailure::Timeout);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    };
    let _ = client.shutdown().join();
    if cancel.load(Ordering::Relaxed) {
        return Err(OutputError::Cancelled);
    }
    match result {
        Ok(structure) => match render(&structure, &cancel) {
            Ok(text) => Ok(PreparedStructure::Rendered {
                content: text,
                diagnostic: structure.duplicate_diagnostic(),
            }),
            Err(OutputError::Cancelled) => Err(OutputError::Cancelled),
            Err(_) => Ok(PreparedStructure::Fallback(
                "la representación estructurada excede sus límites; se conserva la fuente completa"
                    .into(),
            )),
        },
        Err(reason) => Ok(PreparedStructure::Fallback(reason.description().into())),
    }
}
/// Prepare the entire representation transactionally before the caller writes.
pub fn render(
    structure: &crate::structure_model::Structure,
    cancel: &AtomicBool,
) -> Result<String, OutputError> {
    render_with_limit(structure, cancel, OUTPUT_LIMIT)
}
fn render_with_limit(
    structure: &crate::structure_model::Structure,
    cancel: &AtomicBool,
    limit: usize,
) -> Result<String, OutputError> {
    use crate::structure_model::{NodeRef, StructureKind};
    enum Work<'a> {
        Node {
            node: NodeRef<'a>,
            depth: usize,
            root: bool,
        },
        Children {
            node: NodeRef<'a>,
            next: usize,
            depth: usize,
            root: bool,
        },
    }
    let mut output = Builder::new(cancel, limit);
    let mut stack = vec![Work::Node {
        node: structure.root(),
        depth: 0,
        root: true,
    }];
    while let Some(work) = stack.pop() {
        if cancel.load(Ordering::Relaxed) {
            return Err(OutputError::Cancelled);
        }
        match work {
            Work::Node { node, depth, root } => {
                if depth > 64 {
                    return Err(OutputError::InvalidModel);
                }
                output.append(crate::data::file_kind_name(node.kind()))?;
                if let Some(leaf) = node.leaf() {
                    output.append(if leaf.kind() == StructureKind::String {
                        "("
                    } else {
                        "(raw="
                    })?;
                    output.quoted(leaf.text())?;
                    output.append(")")?;
                    if !root {
                        output.append(",\n")?;
                    }
                } else {
                    let mapping = node.kind() == StructureKind::Mapping;
                    output.append(if mapping { "{" } else { "[" })?;
                    if node.entries().len() == 0 {
                        output.append(if mapping { "}" } else { "]" })?;
                        if !root {
                            output.append(",\n")?;
                        }
                    } else {
                        output.append("\n")?;
                        stack.push(Work::Children {
                            node,
                            next: 0,
                            depth,
                            root,
                        });
                    }
                }
            }
            Work::Children {
                node,
                next,
                depth,
                root,
            } => {
                if let Some(entry) = node.entry(next) {
                    output.indent(depth + 1)?;
                    if let Some(key) = entry.key() {
                        output.append("key(")?;
                        output.quoted(key.leaf().ok_or(OutputError::InvalidModel)?.text())?;
                        output.append(") = ")?;
                    } else {
                        output.append(&format!("[{}] = ", entry.ordinal()))?;
                    }
                    stack.push(Work::Children {
                        node,
                        next: next + 1,
                        depth,
                        root,
                    });
                    stack.push(Work::Node {
                        node: entry.value(),
                        depth: depth + 1,
                        root: false,
                    });
                } else {
                    output.indent(depth)?;
                    output.append(if node.kind() == StructureKind::Mapping {
                        "}"
                    } else {
                        "]"
                    })?;
                    if !root {
                        output.append(",\n")?;
                    }
                }
            }
        }
    }
    output.append("\n")?;
    Ok(output.text)
}
#[cfg(test)]
mod output_tests {
    use super::*;
    #[test]
    fn quoted_strings_distinguish_controls_literals_delimiters_and_empty() {
        let cancel = AtomicBool::new(false);
        let mut output = Builder::new(&cancel, 4096);
        output
            .quoted("\x1b␛\\u{001b}\"\n\u{009b}界e\u{301}")
            .unwrap();
        assert_eq!(
            output.text,
            "\"\\u{001b}␛\\\\u{001b}\\\"\\n\\u{009b}界e\u{301}\""
        );
        assert!(!output.text.chars().any(char::is_control));
        output.quoted("").unwrap();
        assert!(output.text.ends_with("\"\""));
    }
    #[test]
    fn output_limit_and_cancellation_never_return_an_incomplete_success() {
        let cancel = AtomicBool::new(false);
        let mut output = Builder::new(&cancel, 8);
        assert_eq!(output.quoted("1234567"), Err(OutputError::Limit));
        cancel.store(true, Ordering::Relaxed);
        assert_eq!(output.append("x"), Err(OutputError::Cancelled));
    }
}

#[cfg(test)]
mod representation_tests {
    use super::*;
    #[test]
    fn complete_typed_output_preserves_order_lexeme_cooked_controls_and_empty_containers() {
        let structure = crate::data::file_fixture::sample();
        let cancel = AtomicBool::new(false);
        let output = render(&structure, &cancel).unwrap();
        assert!(output.contains("key(\"a.b\") = string(\"界\\u{001b}\")"));
        assert!(output.contains("integer(raw=\"0xF\")"));
        assert!(output.contains("string(\"\")"));
        assert!(output.contains("key(\"empty\") = table{}"));
        assert!(output.find("key(\"a.b\")").unwrap() < output.find("key(\"items\")").unwrap());
        assert!(!output.contains("nothing"));
        assert!(!output.contains('\x1b'));
        assert_eq!(
            render_with_limit(&structure, &cancel, 30),
            Err(OutputError::Limit)
        );
        cancel.store(true, Ordering::Relaxed);
        assert_eq!(render(&structure, &cancel), Err(OutputError::Cancelled));
    }
}
