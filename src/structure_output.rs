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
    strict_growth: bool,
}
impl<'a> Builder<'a> {
    fn new(cancel: &'a AtomicBool, limit: usize) -> Self {
        Self {
            text: String::new(),
            cancel,
            limit,
            strict_growth: false,
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
            // YAML admits old and replacement capacities together before realloc.
            if self.strict_growth
                && self
                    .text
                    .capacity()
                    .checked_add(target)
                    .is_none_or(|peak| peak > self.limit)
            {
                return Err(OutputError::Limit);
            }
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
    if structure.format() == crate::structure_model::StructureFormat::Yaml {
        let available = crate::structure_model::MAX_STRUCTURE_BYTES
            .checked_sub(structure.persistent_bytes())
            .ok_or(OutputError::Limit)?;
        render_yaml(structure, cancel, OUTPUT_LIMIT.min(available))
    } else if structure.format() == crate::structure_model::StructureFormat::Xml {
        let available = crate::structure_model::MAX_STRUCTURE_BYTES
            .checked_sub(structure.persistent_bytes())
            .ok_or(OutputError::Limit)?;
        render_xml(structure, cancel, OUTPUT_LIMIT.min(available))
    } else {
        render_with_limit(structure, cancel, OUTPUT_LIMIT)
    }
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

/// Fixed work stack, ordered pair keys are nodes, aliases stay descriptors.
fn render_yaml(
    structure: &crate::structure_model::Structure,
    cancel: &AtomicBool,
    limit: usize,
) -> Result<String, OutputError> {
    use crate::structure_model::{NodeRef, StructureKind};
    #[derive(Clone, Copy)]
    enum Work<'a> {
        Node(NodeRef<'a>, usize, bool),
        Children(NodeRef<'a>, usize, usize, bool),
        Value(NodeRef<'a>, usize),
        PairEnd(usize),
    }
    let mut stack = [None; 256];
    let mut count = 1;
    stack[0] = Some(Work::Node(structure.root(), 0, true));
    let mut output = Builder::new(cancel, limit);
    output.strict_growth = true;
    while count > 0 {
        count -= 1;
        let work = stack[count].take().ok_or(OutputError::InvalidModel)?;
        if cancel.load(Ordering::Relaxed) {
            return Err(OutputError::Cancelled);
        }
        let mut push = |work| -> Result<(), OutputError> {
            if count == stack.len() {
                return Err(OutputError::InvalidModel);
            }
            stack[count] = Some(work);
            count += 1;
            Ok(())
        };
        match work {
            Work::Node(node, depth, root) => {
                output.append(crate::data::file_kind_name(node.kind()))?;
                if let Some(meta) = node.metadata() {
                    output.append(" metadata{")?;
                    output.append(&format!("document={}", meta.document))?;
                    if let Some(anchor) = meta.anchor {
                        output.append(&format!(", anchorID={anchor}"))?
                    }
                    if let Some(tag) = meta.tag {
                        output.append(", tagURI=")?;
                        output.quoted(structure.text_ref(tag))?
                    }
                    if let Some(style) = meta.style {
                        output.append(&format!(", style={style:?}"))?
                    }
                    if node.kind() == StructureKind::YamlDocument {
                        output.append(if meta.explicit_start {
                            ", start=explicit"
                        } else {
                            ", start=implicit"
                        })?;
                        if meta.synthetic_root {
                            output.append(", token=absent")?
                        }
                    }
                    output.append("}")?;
                }
                if let Some(alias) = node.metadata().and_then(|m| m.alias) {
                    output.append(&format!(
                        "(anchorID={}, targetNode={}, name=",
                        alias.anchor, alias.target.0
                    ))?;
                    if let Some(name) = structure.alias_name(node.id()) {
                        output.quoted(name)?
                    } else {
                        output.append("unavailable")?
                    }
                    output.append(")")?;
                    if !root {
                        output.append(",\n")?
                    }
                } else if let Some(decoded) = node.decoded() {
                    output.append("(decoded=")?;
                    output.quoted(decoded)?;
                    output.append(", raw=")?;
                    if let Some(span) = node.source_location() {
                        output.quoted(&structure.source()[span.range()])?
                    } else {
                        output.append("absent")?
                    }
                    output.append(")")?;
                    if !root {
                        output.append(",\n")?
                    }
                } else {
                    output.append("[")?;
                    if node.entries().len() == 0 {
                        output.append("]")?;
                        if !root {
                            output.append(",\n")?
                        }
                    } else {
                        output.append("\n")?;
                        push(Work::Children(node, 0, depth, root))?
                    }
                }
            }
            Work::Children(node, next, depth, root) => {
                if let Some(entry) = node.entry(next) {
                    output.indent(depth + 1)?;
                    push(Work::Children(node, next + 1, depth, root))?;
                    if let Some(key) = entry.key() {
                        output.append(&format!("pair[{}] {{\n", entry.ordinal()))?;
                        output.indent(depth + 2)?;
                        output.append("key = ")?;
                        push(Work::PairEnd(depth + 1))?;
                        push(Work::Value(entry.value(), depth + 2))?;
                        push(Work::Node(key, depth + 2, false))?;
                    } else {
                        output.append(&format!("[{}] = ", entry.ordinal()))?;
                        push(Work::Node(entry.value(), depth + 1, false))?
                    }
                } else {
                    output.indent(depth)?;
                    output.append("]")?;
                    if !root {
                        output.append(",\n")?
                    }
                }
            }
            Work::Value(node, depth) => {
                output.indent(depth)?;
                output.append("value = ")?;
                push(Work::Node(node, depth, false))?
            }
            Work::PairEnd(depth) => {
                output.indent(depth)?;
                output.append("},\n")?
            }
        }
    }
    output.append("\n")?;
    Ok(output.text)
}
#[cfg(test)]
mod yaml_output_tests {
    use super::*;
    #[test]
    fn stream_documents_complex_key_and_alias_descriptor_are_complete_without_expansion() {
        let raw = "---\n? [a, b]\n: &x [*x]\n---\n\"\"\n---\n";
        let cancel = AtomicBool::new(false);
        let structure =
            crate::yaml_worker::project(std::sync::Arc::new(raw.to_owned()), 1, &cancel)
                .unwrap()
                .0;
        let text = render(&structure, &cancel).unwrap();
        assert!(text.starts_with("yaml_stream["));
        assert_eq!(text.matches("yaml_document metadata").count(), 3);
        assert!(text.contains("key = array"));
        assert!(text.contains("targetNode="));
        assert!(text.contains("decoded=\"\", raw=\"\\\"\\\"\""));
        assert!(text.contains("token=absent"));
        assert!(text.len() < 2000);
        assert_eq!(
            render_yaml(&structure, &cancel, 30),
            Err(OutputError::Limit)
        );
        assert_eq!(
            render(&structure, &AtomicBool::new(true)),
            Err(OutputError::Cancelled)
        );
    }
    #[test]
    fn yaml_output_preadmits_old_and_new_capacities_before_realloc() {
        let cancel = AtomicBool::new(false);
        let mut output = Builder::new(&cancel, 10_000);
        output.strict_growth = true;
        output.append(&"x".repeat(3000)).unwrap();
        let capacity = output.text.capacity();
        assert_eq!(output.append(&"y".repeat(2000)), Err(OutputError::Limit));
        assert_eq!(output.text.len(), 3000);
        assert_eq!(output.text.capacity(), capacity);
    }
}

fn render_xml(
    structure: &crate::structure_model::Structure,
    cancel: &AtomicBool,
    limit: usize,
) -> Result<String, OutputError> {
    let mut output = Builder::new(cancel, limit);
    output.strict_growth = true;
    for id in crate::xml_view::Visits::new(structure, cancel) {
        let id = id.map_err(|e| {
            if e == crate::xml_view::Error::Cancelled {
                OutputError::Cancelled
            } else {
                OutputError::InvalidModel
            }
        })?;
        let node = structure.node(id).ok_or(OutputError::InvalidModel)?;
        let mut depth = 0;
        let mut at = id;
        while let Some(parent) = structure.parent(at) {
            depth += 1;
            if depth > 66 {
                return Err(OutputError::InvalidModel);
            }
            at = parent.owner;
        }
        output.indent(depth)?;
        output.append(crate::data::file_kind_name(node.kind()))?;
        for domain in crate::xml_view::SEARCH_DOMAINS {
            if let Some(value) = crate::xml_view::unit(structure, id, domain) {
                output.append(" ")?;
                output.append(match domain {
                    crate::xml_view::Domain::QName => "QName=",
                    crate::xml_view::Domain::LocalName => "LocalName=",
                    crate::xml_view::Domain::NamespaceUri => "NamespaceUri=",
                    crate::xml_view::Domain::ValueDecoded => "ValueDecoded=",
                    _ => "Metadata=",
                })?;
                output.quoted(value)?;
            }
        }
        if let Some(meta) = node.xml_metadata() {
            if meta.empty_form {
                output.append(" empty=true")?;
            }
            if meta.has_doctype {
                output.append(" DOCTYPE=inerte(sin-recursos-externos)")?;
            }
        }
        output.append("\n")?;
    }
    Ok(output.text)
}

#[cfg(test)]
mod xml_output_tests {
    use super::*;
    #[test]
    fn xml_inspection_is_complete_safe_ordered_and_locally_bounded() {
        let cancel = AtomicBool::new(false);
        let s = crate::xml_worker::project(
            std::sync::Arc::new("<r b='true' a='12'>\u{009b}&amp;<!--x--></r>".into()),
            1,
            &cancel,
        )
        .unwrap()
        .0;
        let output = render(&s, &cancel).unwrap();
        assert!(output.find("QName=\"b\"").unwrap() < output.find("QName=\"a\"").unwrap());
        assert!(output.contains("ValueDecoded=\"true\""));
        assert!(output.contains("ValueDecoded=\"12\""));
        assert!(!output.contains('\u{009b}'));
        assert!(output.contains("\\u{009b}"));
        assert_eq!(render_xml(&s, &cancel, 32), Err(OutputError::Limit));
        cancel.store(true, Ordering::Relaxed);
        assert_eq!(render(&s, &cancel), Err(OutputError::Cancelled));
    }
}
