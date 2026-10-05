//! Immutable file structure. No parser, Nu or syntax transport types are stored here.
use std::{
    fmt,
    mem::size_of,
    ops::Range,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
pub const MAX_STRUCTURE_DEPTH: usize = 64;
pub const MAX_STRUCTURE_ITEMS: usize = 1_000_000;
pub const MAX_STRUCTURE_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_STRUCTURE_SOURCE: usize = 128 * 1024 * 1024;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StructureFormat {
    Toml,
    Json,
    Yaml,
    Xml,
}
impl StructureFormat {
    pub(crate) fn decode(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Toml),
            1 => Some(Self::Json),
            2 => Some(Self::Yaml),
            3 => Some(Self::Xml),
            _ => None,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntryId(pub u32);
/// UTF-8 byte coordinates in the immutable original source, never cooked offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSpan {
    pub start: u32,
    pub end: u32,
}
impl SourceSpan {
    pub fn range(self) -> Range<usize> {
        self.start as usize..self.end as usize
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextRef {
    Source(SourceSpan),
    Cooked { start: u32, end: u32 },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StructureKind {
    Mapping,
    Sequence,
    ArrayOfTables,
    String,
    Boolean,
    Integer,
    Float,
    OffsetDateTime,
    LocalDateTime,
    LocalDate,
    LocalTime,
    Null,
    JsonNumber,
    YamlStream,
    YamlDocument,
    Alias,
    YamlTaggedScalar,
    XmlDocument,
    XmlElement,
    XmlAttribute,
    XmlNamespaceDeclaration,
    XmlText,
    XmlReference,
    XmlCData,
    XmlComment,
    XmlPi,
    XmlDeclaration,
    XmlDocType,
}
impl StructureKind {
    pub fn is_container(self) -> bool {
        matches!(
            self,
            Self::Mapping
                | Self::Sequence
                | Self::ArrayOfTables
                | Self::YamlStream
                | Self::YamlDocument
                | Self::XmlDocument
                | Self::XmlElement
        )
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarationKind {
    Header,
    ArrayOfTablesHeader,
    Assignment,
    InlineAssignment,
}
#[derive(Debug, Clone, Copy)]
pub struct DeclarationPart {
    pub span: SourceSpan,
    pub node: NodeId,
}
#[derive(Debug, Clone, Copy)]
pub struct NodeSpec {
    pub kind: StructureKind,
    pub text: Option<TextRef>,
    pub location: Option<SourceSpan>,
}
#[derive(Debug, Clone, Copy)]
pub struct EntrySpec {
    pub key: Option<NodeId>,
    pub value: NodeId,
}
#[derive(Debug, Clone, Copy)]
pub struct DeclarationSpec {
    pub kind: DeclarationKind,
    pub span: SourceSpan,
    pub key_span: SourceSpan,
    pub value_span: Option<SourceSpan>,
    pub node: NodeId,
}
#[derive(Debug)]
struct Node {
    spec: NodeSpec,
    entries: Range<usize>,
    assigned: bool,
}
#[derive(Debug)]
struct Entry {
    spec: EntrySpec,
}
#[derive(Debug)]
struct Declaration {
    spec: DeclarationSpec,
    parts: Range<usize>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelError {
    Cancelled,
    SourceLimit,
    Budget,
    Depth,
    Invalid,
}
impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Cancelled => "structure cancelled",
            Self::SourceLimit => "structure source limit exceeded",
            Self::Budget => "structure budget exceeded",
            Self::Depth => "structure depth exceeded",
            Self::Invalid => "invalid structure model",
        })
    }
}
impl std::error::Error for ModelError {}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DuplicateSummary {
    pub extra_occurrences: usize,
    pub first_key_span: Option<SourceSpan>,
}
#[derive(Debug)]
pub struct Structure {
    format: StructureFormat,
    metadata: Vec<YamlMetadata>,
    xml_metadata: Vec<XmlMetadata>,
    parents: Vec<Option<ParentLink>>,
    session_bytes: usize,
    yaml_edges: usize,
    source: Arc<String>,
    generation: u64,
    root: NodeId,
    nodes: Vec<Node>,
    entries: Vec<Entry>,
    declarations: Vec<Declaration>,
    parts: Vec<DeclarationPart>,
    cooked: Vec<u8>,
    duplicates: DuplicateSummary,
}
impl Structure {
    pub fn format(&self) -> StructureFormat {
        self.format
    }
    pub fn metadata(&self, node: NodeId) -> Option<&YamlMetadata> {
        self.metadata
            .binary_search_by_key(&node.0, |m| m.node.0)
            .ok()
            .map(|i| &self.metadata[i])
    }
    pub fn xml_metadata(&self, node: NodeId) -> Option<&XmlMetadata> {
        self.xml_metadata
            .get(node.0 as usize)
            .filter(|m| m.node == node)
    }
    pub(crate) fn xml_projected_bytes(
        nodes: usize,
        entries: usize,
        cooked: usize,
    ) -> Option<usize> {
        nodes
            .checked_mul(
                size_of::<Node>()
                    + size_of::<XmlMetadata>()
                    + size_of::<Option<ParentLink>>()
                    + size_of::<u32>()
                    + 1,
            )
            .and_then(|n| {
                entries
                    .checked_mul(size_of::<Entry>() + 1)
                    .and_then(|e| n.checked_add(e))
            })
            .and_then(|n| n.checked_add(cooked))
            .and_then(|n| {
                n.checked_add(
                    1024 * 1024
                        + 4 * crate::structure_protocol::MAX_FRAME
                        + 64 * 1024
                        + size_of::<DuplicateSummary>(),
                )
            })
    }
    pub(crate) fn raw_xml_metadata(&self) -> &[XmlMetadata] {
        &self.xml_metadata
    }
    pub fn parent(&self, node: NodeId) -> Option<ParentLink> {
        self.parents.get(node.0 as usize).copied().flatten()
    }
    pub fn text_ref(&self, text: TextRef) -> &str {
        self.text(text)
    }
    pub fn session_budget_bytes(&self) -> usize {
        self.session_bytes
    }
    pub fn alias_name(&self, node: NodeId) -> Option<&str> {
        let n = self.node(node)?;
        if n.kind() != StructureKind::Alias {
            return None;
        }
        self.source
            .get(n.source_location()?.range())?
            .strip_prefix('*')
    }
    pub(crate) fn raw_metadata(&self) -> &[YamlMetadata] {
        &self.metadata
    }
    pub(crate) fn alias_edges(&self) -> usize {
        self.metadata.iter().filter(|m| m.alias.is_some()).count()
    }
    pub fn duplicate_diagnostic(&self) -> Option<String> {
        (self.duplicates.extra_occurrences > 0).then(|| {
            if self.format == StructureFormat::Yaml {
                return format!("{} repeticiones de claves scalar-string decodificadas; se conservan todos los pares; comprobación parcial, sin validar unicidad semántica de claves complejas",self.duplicates.extra_occurrences);
            }
            format!(
                "{} repeticiones de claves; se conservan todas las entradas",
                self.duplicates.extra_occurrences
            )
        })
    }
    pub fn duplicates(&self) -> DuplicateSummary {
        self.duplicates
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn source(&self) -> &str {
        self.source.as_str()
    }
    pub fn source_handle(&self) -> Arc<String> {
        Arc::clone(&self.source)
    }
    pub fn root(&self) -> NodeRef<'_> {
        self.node(self.root).expect("validated root")
    }
    pub fn node(&self, id: NodeId) -> Option<NodeRef<'_>> {
        ((id.0 as usize) < self.nodes.len()).then_some(NodeRef {
            structure: self,
            id,
        })
    }
    pub fn declarations(&self) -> impl ExactSizeIterator<Item = DeclarationRef<'_>> {
        self.declarations.iter().map(|declaration| DeclarationRef {
            structure: self,
            declaration,
        })
    }
    pub fn persistent_bytes(&self) -> usize {
        size_of::<DuplicateSummary>()
            + self.session_bytes
            + self.metadata.capacity() * size_of::<YamlMetadata>()
            + self.xml_metadata.capacity() * size_of::<XmlMetadata>()
            + self.parents.capacity() * size_of::<Option<ParentLink>>()
            + self.nodes.capacity() * size_of::<Node>()
            + self.entries.capacity() * size_of::<Entry>()
            + self.declarations.capacity() * size_of::<Declaration>()
            + self.parts.capacity() * size_of::<DeclarationPart>()
            + self.cooked.capacity()
    }
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
    pub(crate) fn raw_nodes(&self) -> impl ExactSizeIterator<Item = (NodeSpec, Range<usize>)> {
        self.nodes
            .iter()
            .map(|node| (node.spec, node.entries.clone()))
    }
    pub(crate) fn raw_entries(&self) -> impl ExactSizeIterator<Item = EntrySpec> {
        self.entries.iter().map(|entry| entry.spec)
    }
    pub(crate) fn raw_declarations(
        &self,
    ) -> impl ExactSizeIterator<Item = (DeclarationSpec, &[DeclarationPart])> {
        self.declarations
            .iter()
            .map(|declaration| (declaration.spec, &self.parts[declaration.parts.clone()]))
    }
    pub(crate) fn raw_cooked(&self) -> &[u8] {
        &self.cooked
    }
    fn text(&self, text: TextRef) -> &str {
        match text {
            TextRef::Source(span) => &self.source[span.range()],
            TextRef::Cooked { start, end } => {
                std::str::from_utf8(&self.cooked[start as usize..end as usize])
                    .expect("validated cooked UTF-8")
            }
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct NodeRef<'a> {
    structure: &'a Structure,
    id: NodeId,
}
impl<'a> NodeRef<'a> {
    pub fn metadata(self) -> Option<&'a YamlMetadata> {
        self.structure.metadata(self.id)
    }
    pub fn xml_metadata(self) -> Option<&'a XmlMetadata> {
        self.structure.xml_metadata(self.id)
    }
    pub fn parent(self) -> Option<ParentLink> {
        self.structure.parent(self.id)
    }
    pub fn decoded(self) -> Option<&'a str> {
        self.structure.nodes[self.id.0 as usize]
            .spec
            .text
            .map(|t| self.structure.text(t))
    }
    pub fn id(self) -> NodeId {
        self.id
    }
    pub(crate) fn raw_text_ref(self) -> Option<TextRef> {
        self.structure.nodes[self.id.0 as usize].spec.text
    }
    pub fn kind(self) -> StructureKind {
        self.structure.nodes[self.id.0 as usize].spec.kind
    }
    pub fn source_location(self) -> Option<SourceSpan> {
        self.structure.nodes[self.id.0 as usize].spec.location
    }
    pub fn entries(self) -> impl ExactSizeIterator<Item = EntryRef<'a>> {
        self.structure.nodes[self.id.0 as usize]
            .entries
            .clone()
            .enumerate()
            .map(move |(ordinal, index)| EntryRef {
                structure: self.structure,
                id: EntryId(index as u32),
                parent: self.id,
                ordinal,
                entry: &self.structure.entries[index],
            })
    }
    pub fn entry(self, ordinal: usize) -> Option<EntryRef<'a>> {
        let range = &self.structure.nodes[self.id.0 as usize].entries;
        (ordinal < range.len()).then(|| {
            let index = range.start + ordinal;
            EntryRef {
                structure: self.structure,
                id: EntryId(index as u32),
                parent: self.id,
                ordinal,
                entry: &self.structure.entries[index],
            }
        })
    }
    pub fn leaf(self) -> Option<LeafRef<'a>> {
        let spec = self.structure.nodes[self.id.0 as usize].spec;
        spec.text.map(|text| LeafRef {
            kind: spec.kind,
            text: self.structure.text(text),
            location: spec.location,
        })
    }
}
#[derive(Clone, Copy, Debug)]
pub struct EntryRef<'a> {
    structure: &'a Structure,
    id: EntryId,
    parent: NodeId,
    ordinal: usize,
    entry: &'a Entry,
}
impl<'a> EntryRef<'a> {
    /// A contiguous raw member range only when its enclosing container has one.
    /// This does not fabricate a TOML declaration for dispersed/implicit tables.
    pub fn source_location(self) -> Option<SourceSpan> {
        let parent = self.structure.node(self.parent)?.source_location()?;
        let value = self.value().source_location()?;
        let key = match self.key() {
            Some(key) => key.source_location()?,
            None => value,
        };
        if key.start < parent.start
            || value.end > parent.end
            || key.start > value.end
            || (self.key().is_some() && key.end > value.start)
        {
            return None;
        }
        Some(SourceSpan {
            start: key.start,
            end: value.end,
        })
    }
    pub fn id(self) -> EntryId {
        self.id
    }
    pub fn ordinal(self) -> usize {
        self.ordinal
    }
    pub fn key(self) -> Option<NodeRef<'a>> {
        self.entry
            .spec
            .key
            .map(|id| self.structure.node(id).expect("validated key"))
    }
    pub fn value(self) -> NodeRef<'a> {
        self.structure
            .node(self.entry.spec.value)
            .expect("validated value")
    }
}
#[derive(Clone, Copy, Debug)]
pub struct LeafRef<'a> {
    kind: StructureKind,
    text: &'a str,
    location: Option<SourceSpan>,
}
impl<'a> LeafRef<'a> {
    pub fn kind(self) -> StructureKind {
        self.kind
    }
    /// Complete interpreted string/key, or the authoritative original scalar lexeme.
    pub fn text(self) -> &'a str {
        self.text
    }
    pub fn source_location(self) -> Option<SourceSpan> {
        self.location
    }
}
#[derive(Clone, Copy, Debug)]
pub struct DeclarationRef<'a> {
    structure: &'a Structure,
    declaration: &'a Declaration,
}
impl<'a> DeclarationRef<'a> {
    pub fn kind(self) -> DeclarationKind {
        self.declaration.spec.kind
    }
    pub fn source_location(self) -> SourceSpan {
        self.declaration.spec.span
    }
    pub fn key_span(self) -> SourceSpan {
        self.declaration.spec.key_span
    }
    pub fn value_span(self) -> Option<SourceSpan> {
        self.declaration.spec.value_span
    }
    pub fn node(self) -> NodeRef<'a> {
        self.structure
            .node(self.declaration.spec.node)
            .expect("validated declaration")
    }
    pub fn parts(self) -> &'a [DeclarationPart] {
        &self.structure.parts[self.declaration.parts.clone()]
    }
}
/// Incremental construction: only finish exposes a readable arena.
#[derive(Debug)]
pub struct StructureBuilder {
    model: Structure,
    scratch_bytes: usize,
}
pub(crate) fn reserve<T>(
    vector: &mut Vec<T>,
    additional: usize,
    other: usize,
) -> Result<(), ModelError> {
    let needed = vector
        .len()
        .checked_add(additional)
        .ok_or(ModelError::Budget)?;
    if needed <= vector.capacity() {
        return Ok(());
    }
    let old_bytes = vector
        .capacity()
        .checked_mul(size_of::<T>())
        .ok_or(ModelError::Budget)?;
    let max = MAX_STRUCTURE_BYTES
        .checked_sub(other)
        .and_then(|n| n.checked_sub(old_bytes))
        .ok_or(ModelError::Budget)?
        / size_of::<T>();
    if needed > max {
        return Err(ModelError::Budget);
    }
    let desired = needed
        .max(vector.capacity().saturating_mul(2).max(16))
        .min(max);
    vector
        .try_reserve_exact(desired - vector.len())
        .map_err(|_| ModelError::Budget)?;
    if vector.capacity() > max {
        return Err(ModelError::Budget);
    }
    Ok(())
}
impl StructureBuilder {
    pub fn new(source: Arc<String>, generation: u64) -> Result<Self, ModelError> {
        Self::new_for_format(source, generation, StructureFormat::Toml)
    }
    pub fn new_for_format(
        source: Arc<String>,
        generation: u64,
        format: StructureFormat,
    ) -> Result<Self, ModelError> {
        if source.len() > MAX_STRUCTURE_SOURCE {
            return Err(ModelError::SourceLimit);
        }
        Ok(Self {
            scratch_bytes: 0,
            model: Structure {
                format,
                metadata: Vec::new(),
                xml_metadata: Vec::new(),
                parents: Vec::new(),
                session_bytes: if matches!(format, StructureFormat::Yaml | StructureFormat::Xml) {
                    1024 * 1024
                } else {
                    0
                },
                yaml_edges: 0,
                source,
                generation,
                root: NodeId(0),
                nodes: Vec::new(),
                entries: Vec::new(),
                declarations: Vec::new(),
                parts: Vec::new(),
                cooked: Vec::new(),
                duplicates: DuplicateSummary::default(),
            },
        })
    }
    /// Charge live caller-owned construction buffers before any arena growth.
    pub(crate) fn set_scratch_bytes(&mut self, bytes: usize) -> Result<(), ModelError> {
        if self
            .model
            .persistent_bytes()
            .checked_add(bytes)
            .is_none_or(|total| total > MAX_STRUCTURE_BYTES)
        {
            return Err(ModelError::Budget);
        }
        self.scratch_bytes = bytes;
        Ok(())
    }
    pub(crate) fn set_location(&mut self, id: NodeId, span: SourceSpan) -> Result<(), ModelError> {
        self.model
            .nodes
            .get_mut(id.0 as usize)
            .ok_or(ModelError::Invalid)?
            .spec
            .location = Some(span);
        Ok(())
    }
    pub fn push_node(&mut self, spec: NodeSpec) -> Result<NodeId, ModelError> {
        if self.model.nodes.len()
            + if self.model.format == StructureFormat::Yaml {
                self.model.yaml_edges
            } else {
                self.model.entries.len()
            }
            >= MAX_STRUCTURE_ITEMS
        {
            return Err(ModelError::Budget);
        }
        let other = self.model.persistent_bytes() - self.model.nodes.capacity() * size_of::<Node>();
        reserve(&mut self.model.nodes, 1, other + self.scratch_bytes)?;
        let id = NodeId(self.model.nodes.len() as u32);
        self.model.nodes.push(Node {
            spec,
            entries: 0..0,
            assigned: false,
        });
        Ok(id)
    }
    /// Set this container's entries once, in their final logical order.
    pub fn set_entries(
        &mut self,
        parent: NodeId,
        entries: impl IntoIterator<Item = EntrySpec>,
    ) -> Result<(), ModelError> {
        let node = self
            .model
            .nodes
            .get(parent.0 as usize)
            .ok_or(ModelError::Invalid)?;
        if node.assigned || !node.spec.kind.is_container() {
            return Err(ModelError::Invalid);
        }
        let start = self.model.entries.len();
        for spec in entries {
            if self.model.nodes.len()
                + if self.model.format == StructureFormat::Yaml {
                    self.model.yaml_edges
                } else {
                    self.model.entries.len()
                }
                >= MAX_STRUCTURE_ITEMS
            {
                return Err(ModelError::Budget);
            }
            let other =
                self.model.persistent_bytes() - self.model.entries.capacity() * size_of::<Entry>();
            reserve(&mut self.model.entries, 1, other + self.scratch_bytes)?;
            if self.model.format == StructureFormat::Yaml {
                let edges = 1 + usize::from(spec.key.is_some());
                if self.model.nodes.len() + self.model.yaml_edges + edges > MAX_STRUCTURE_ITEMS {
                    return Err(ModelError::Budget);
                }
                self.model.yaml_edges += edges;
            }
            self.model.entries.push(Entry { spec });
        }
        self.model.nodes[parent.0 as usize].entries = start..self.model.entries.len();
        self.model.nodes[parent.0 as usize].assigned = true;
        Ok(())
    }
    pub(crate) fn reserve_cooked(&mut self, additional: usize) -> Result<(), ModelError> {
        let other = self.model.persistent_bytes() - self.model.cooked.capacity();
        reserve(
            &mut self.model.cooked,
            additional,
            other + self.scratch_bytes,
        )
    }
    /// Chunks may split UTF-8; complete text is validated by finish.
    pub fn append_cooked(&mut self, bytes: &[u8]) -> Result<Range<u32>, ModelError> {
        let other = self.model.persistent_bytes() - self.model.cooked.capacity();
        reserve(
            &mut self.model.cooked,
            bytes.len(),
            other + self.scratch_bytes,
        )?;
        let start = self.model.cooked.len() as u32;
        self.model.cooked.extend_from_slice(bytes);
        Ok(start..self.model.cooked.len() as u32)
    }
    pub fn push_declaration(
        &mut self,
        spec: DeclarationSpec,
        parts: impl IntoIterator<Item = DeclarationPart>,
    ) -> Result<(), ModelError> {
        let start = self.model.parts.len();
        for part in parts {
            let other = self.model.persistent_bytes()
                - self.model.parts.capacity() * size_of::<DeclarationPart>();
            reserve(&mut self.model.parts, 1, other + self.scratch_bytes)?;
            self.model.parts.push(part);
        }
        let other = self.model.persistent_bytes()
            - self.model.declarations.capacity() * size_of::<Declaration>();
        reserve(&mut self.model.declarations, 1, other + self.scratch_bytes)?;
        self.model.declarations.push(Declaration {
            spec,
            parts: start..self.model.parts.len(),
        });
        Ok(())
    }
    pub(crate) fn from_structure(model: Structure) -> Self {
        Self {
            model,
            scratch_bytes: 0,
        }
    }
    pub(crate) fn extend_entries_chunk(
        &mut self,
        parent: NodeId,
        ordinal: u32,
        entries: impl IntoIterator<Item = EntrySpec>,
    ) -> Result<(), ModelError> {
        let node = self
            .model
            .nodes
            .get(parent.0 as usize)
            .ok_or(ModelError::Invalid)?;
        if !node.spec.kind.is_container() {
            return Err(ModelError::Invalid);
        }
        if ordinal == 0 {
            if node.assigned {
                return Err(ModelError::Invalid);
            }
            let start = self.model.entries.len();
            self.model.nodes[parent.0 as usize].entries = start..start;
            self.model.nodes[parent.0 as usize].assigned = true;
        }
        let node = &self.model.nodes[parent.0 as usize];
        if !node.assigned
            || node.entries.len() != ordinal as usize
            || node.entries.end != self.model.entries.len()
        {
            return Err(ModelError::Invalid);
        }
        for spec in entries {
            if self.model.nodes.len()
                + if self.model.format == StructureFormat::Yaml {
                    self.model.yaml_edges
                } else {
                    self.model.entries.len()
                }
                >= MAX_STRUCTURE_ITEMS
            {
                return Err(ModelError::Budget);
            }
            let other =
                self.model.persistent_bytes() - self.model.entries.capacity() * size_of::<Entry>();
            reserve(&mut self.model.entries, 1, other + self.scratch_bytes)?;
            if self.model.format == StructureFormat::Yaml {
                let edges = 1 + usize::from(spec.key.is_some());
                if self.model.nodes.len() + self.model.yaml_edges + edges > MAX_STRUCTURE_ITEMS {
                    return Err(ModelError::Budget);
                }
                self.model.yaml_edges += edges;
            }
            self.model.entries.push(Entry { spec });
        }
        self.model.nodes[parent.0 as usize].entries.end = self.model.entries.len();
        Ok(())
    }
    pub(crate) fn order_mappings(
        &mut self,
        first: &[Option<SourceSpan>],
    ) -> Result<(), ModelError> {
        if first.len() != self.model.nodes.len() {
            return Err(ModelError::Invalid);
        }
        for i in 0..self.model.nodes.len() {
            if self.model.nodes[i].spec.kind != StructureKind::Mapping {
                continue;
            }
            let range = self.model.nodes[i].entries.clone();
            for entry in &self.model.entries[range.clone()] {
                let source = first[entry.spec.value.0 as usize].ok_or(ModelError::Invalid)?;
                if let Some(key) = entry.spec.key {
                    self.model.nodes[key.0 as usize].spec.location = Some(source);
                }
            }
            self.model.entries[range].sort_unstable_by_key(|entry| {
                first[entry.spec.value.0 as usize].map(|span| span.start)
            });
        }
        Ok(())
    }
    pub fn persistent_bytes(&self) -> usize {
        self.model.persistent_bytes()
    }
    pub fn finish(
        self,
        root: NodeId,
        syntax_depth: usize,
        key_components: usize,
        cancelled: &AtomicBool,
    ) -> Result<Structure, ModelError> {
        self.finish_with(root, syntax_depth, key_components, || {
            cancelled.load(Ordering::Acquire)
        })
    }
    pub(crate) fn finish_with(
        mut self,
        root: NodeId,
        syntax_depth: usize,
        key_components: usize,
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<Structure, ModelError> {
        if self.model.format == StructureFormat::Xml {
            return self.finish_xml(root, syntax_depth, key_components, cancelled);
        }
        if !self.model.xml_metadata.is_empty()
            || self.model.nodes.iter().any(|n| n.spec.kind.is_xml())
        {
            return Err(ModelError::Invalid);
        }
        if self.model.format == StructureFormat::Yaml {
            return self.finish_yaml(root, syntax_depth, key_components, cancelled);
        }
        if cancelled() {
            return Err(ModelError::Cancelled);
        }
        if syntax_depth > MAX_STRUCTURE_DEPTH || key_components > MAX_STRUCTURE_DEPTH {
            return Err(ModelError::Depth);
        }
        if self.model.nodes.is_empty() || (root.0 as usize) >= self.model.nodes.len() {
            return Err(ModelError::Invalid);
        }
        #[derive(Clone, Copy)]
        struct Frame {
            node: NodeId,
            next: usize,
            depth: usize,
        }
        const STACK_BYTES: usize = (MAX_STRUCTURE_DEPTH + 1) * size_of::<Frame>();
        let scratch = self
            .model
            .nodes
            .len()
            .checked_add(STACK_BYTES)
            .and_then(|n| n.checked_add(self.scratch_bytes))
            .ok_or(ModelError::Budget)?;
        if self
            .model
            .persistent_bytes()
            .checked_add(scratch)
            .is_none_or(|n| n > MAX_STRUCTURE_BYTES)
        {
            return Err(ModelError::Budget);
        }
        let mut seen = Vec::new();
        seen.try_reserve_exact(self.model.nodes.len())
            .map_err(|_| ModelError::Budget)?;
        seen.resize(self.model.nodes.len(), 0u8);
        if self.model.persistent_bytes() + seen.capacity() + STACK_BYTES + self.scratch_bytes
            > MAX_STRUCTURE_BYTES
        {
            return Err(ModelError::Budget);
        }
        let cooked = std::str::from_utf8(&self.model.cooked).map_err(|_| ModelError::Invalid)?;
        let valid_span = |span: SourceSpan| {
            let range = span.range();
            range.start <= range.end
                && range.end <= self.model.source.len()
                && self.model.source.is_char_boundary(range.start)
                && self.model.source.is_char_boundary(range.end)
        };
        for (i, node) in self.model.nodes.iter().enumerate() {
            if i % 1024 == 0 && cancelled() {
                return Err(ModelError::Cancelled);
            }
            if node.spec.location.is_some_and(|span| !valid_span(span))
                || node.spec.kind.is_container() != node.spec.text.is_none()
            {
                return Err(ModelError::Invalid);
            }
            if let Some(text) = node.spec.text {
                let valid = match text {
                    TextRef::Source(span) => valid_span(span),
                    TextRef::Cooked { start, end } => {
                        start <= end
                            && (end as usize) <= cooked.len()
                            && cooked.is_char_boundary(start as usize)
                            && cooked.is_char_boundary(end as usize)
                    }
                };
                if !valid {
                    return Err(ModelError::Invalid);
                }
                if node.spec.kind != StructureKind::String
                    && Some(text) != node.spec.location.map(TextRef::Source)
                {
                    return Err(ModelError::Invalid);
                }
            }
        }
        // Depth-bounded traversal; key side leaves do not add logical pair layers.
        let mut stack = [Frame {
            node: root,
            next: 0,
            depth: 0,
        }; MAX_STRUCTURE_DEPTH + 1];
        let mut top = 0;
        seen[root.0 as usize] = 1;
        let mut visited = 1usize;
        let mut visited_entries = 0usize;
        let mut ticks = 0;
        loop {
            ticks += 1;
            if ticks % 1024 == 0 && cancelled() {
                return Err(ModelError::Cancelled);
            }
            let frame = stack[top];
            let node = &self.model.nodes[frame.node.0 as usize];
            if frame.next >= node.entries.len() {
                if top == 0 {
                    break;
                }
                top -= 1;
                continue;
            }
            let entry = &self.model.entries[node.entries.start + frame.next];
            stack[top].next += 1;
            visited_entries += 1;
            if (node.spec.kind == StructureKind::Mapping) != entry.spec.key.is_some() {
                return Err(ModelError::Invalid);
            }
            if let Some(key) = entry.spec.key {
                let key_node = self
                    .model
                    .nodes
                    .get(key.0 as usize)
                    .ok_or(ModelError::Invalid)?;
                if key_node.spec.kind != StructureKind::String
                    || !key_node.entries.is_empty()
                    || seen[key.0 as usize] != 0
                {
                    return Err(ModelError::Invalid);
                }
                seen[key.0 as usize] = 1;
                visited += 1;
            }
            let child = self
                .model
                .nodes
                .get(entry.spec.value.0 as usize)
                .ok_or(ModelError::Invalid)?;
            if seen[entry.spec.value.0 as usize] != 0 {
                return Err(ModelError::Invalid);
            }
            if node.spec.kind == StructureKind::ArrayOfTables
                && child.spec.kind != StructureKind::Mapping
            {
                return Err(ModelError::Invalid);
            }
            let depth = frame.depth + 1;
            if depth > MAX_STRUCTURE_DEPTH {
                return Err(ModelError::Depth);
            }
            seen[entry.spec.value.0 as usize] = 1;
            visited += 1;
            top += 1;
            stack[top] = Frame {
                node: entry.spec.value,
                next: 0,
                depth,
            };
        }
        if visited != self.model.nodes.len() || visited_entries != self.model.entries.len() {
            return Err(ModelError::Invalid);
        }
        let mut previous = 0;
        let mut visited_parts = 0usize;
        for (i, declaration) in self.model.declarations.iter().enumerate() {
            if i % 1024 == 0 && cancelled() {
                return Err(ModelError::Cancelled);
            }
            let spec = declaration.spec;
            if !valid_span(spec.span)
                || !valid_span(spec.key_span)
                || spec.span.start < previous
                || spec.key_span.start < spec.span.start
                || spec.key_span.end > spec.span.end
                || (spec.node.0 as usize) >= self.model.nodes.len()
            {
                return Err(ModelError::Invalid);
            }
            previous = spec.span.start;
            let header = matches!(
                spec.kind,
                DeclarationKind::Header | DeclarationKind::ArrayOfTablesHeader
            );
            if header != spec.value_span.is_none() {
                return Err(ModelError::Invalid);
            }
            if header && self.model.nodes[spec.node.0 as usize].spec.kind != StructureKind::Mapping
            {
                return Err(ModelError::Invalid);
            }
            if let Some(value) = spec.value_span
                && (!valid_span(value)
                    || value.start < spec.key_span.end
                    || value.end != spec.span.end
                    || self.model.nodes[spec.node.0 as usize].spec.location != Some(value))
            {
                return Err(ModelError::Invalid);
            }
            if declaration.parts.is_empty() {
                return Err(ModelError::Invalid);
            }
            if declaration.parts.len() > MAX_STRUCTURE_DEPTH {
                return Err(ModelError::Depth);
            }
            visited_parts += declaration.parts.len();
            let mut end = spec.key_span.start;
            for part in &self.model.parts[declaration.parts.clone()] {
                if !valid_span(part.span)
                    || part.span.start < end
                    || part.span.end > spec.key_span.end
                    || (part.node.0 as usize) >= self.model.nodes.len()
                {
                    return Err(ModelError::Invalid);
                }
                end = part.span.end;
            }
        }
        if cancelled() {
            return Err(ModelError::Cancelled);
        }
        if visited_parts != self.model.parts.len() {
            return Err(ModelError::Invalid);
        }
        let other =
            self.model.persistent_bytes() + self.scratch_bytes + seen.capacity() + STACK_BYTES;
        let mut ids = Vec::<usize>::new();
        let mut duplicates = DuplicateSummary::default();
        for (n, node) in self.model.nodes.iter().enumerate() {
            if n % 1024 == 0 && cancelled() {
                return Err(ModelError::Cancelled);
            }
            if node.spec.kind != StructureKind::Mapping || node.entries.len() < 2 {
                continue;
            }
            ids.clear();
            reserve(&mut ids, node.entries.len(), other)?;
            for (i, id) in node.entries.clone().enumerate() {
                if i % 1024 == 0 && cancelled() {
                    return Err(ModelError::Cancelled);
                }
                ids.push(id);
            }
            let key_text = |entry: usize| -> &str {
                let key = self.model.entries[entry].spec.key.expect("validated key");
                self.model.node(key).unwrap().leaf().unwrap().text()
            };
            heap_sort(&mut ids, |a, b| {
                Ok(compare_text(key_text(a), key_text(b), &mut cancelled)?.then(a.cmp(&b)))
            })?;
            for pair in ids.windows(2) {
                if compare_text(key_text(pair[0]), key_text(pair[1]), &mut cancelled)?
                    == std::cmp::Ordering::Equal
                {
                    duplicates.extra_occurrences += 1;
                    let key = self.model.entries[pair[1]].spec.key.unwrap();
                    let span = self.model.nodes[key.0 as usize]
                        .spec
                        .location
                        .ok_or(ModelError::Invalid)?;
                    if duplicates
                        .first_key_span
                        .is_none_or(|old| span.start < old.start)
                    {
                        duplicates.first_key_span = Some(span);
                    }
                }
            }
        }
        if cancelled() {
            return Err(ModelError::Cancelled);
        }
        self.model.duplicates = duplicates;
        self.model.root = root;
        Ok(self.model)
    }
}
/// Byte-lexicographic comparison with a cancellation checkpoint per bounded block.
fn compare_text(
    a: &str,
    b: &str,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<std::cmp::Ordering, ModelError> {
    use std::cmp::Ordering;
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let mut at = 0;
    let common = a.len().min(b.len());
    while at < common {
        if cancelled() {
            return Err(ModelError::Cancelled);
        }
        let end = (at + 4096).min(common);
        let order = a[at..end].cmp(&b[at..end]);
        if order != Ordering::Equal {
            return Ok(order);
        }
        at = end;
    }
    if cancelled() {
        return Err(ModelError::Cancelled);
    }
    Ok(a.len().cmp(&b.len()))
}
/// Fallible in-place heapsort of IDs; no sorting allocation or cloned key text.
fn heap_sort(
    ids: &mut [usize],
    mut compare: impl FnMut(usize, usize) -> Result<std::cmp::Ordering, ModelError>,
) -> Result<(), ModelError> {
    fn sift(
        ids: &mut [usize],
        mut root: usize,
        end: usize,
        compare: &mut impl FnMut(usize, usize) -> Result<std::cmp::Ordering, ModelError>,
    ) -> Result<(), ModelError> {
        loop {
            let left = root * 2 + 1;
            if left >= end {
                return Ok(());
            }
            let mut child = left;
            if left + 1 < end && compare(ids[left], ids[left + 1])?.is_lt() {
                child += 1;
            }
            if !compare(ids[root], ids[child])?.is_lt() {
                return Ok(());
            }
            ids.swap(root, child);
            root = child;
        }
    }
    for root in (0..ids.len() / 2).rev() {
        sift(ids, root, ids.len(), &mut compare)?;
    }
    for end in (1..ids.len()).rev() {
        ids.swap(0, end);
        sift(ids, 0, end, &mut compare)?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decoded_duplicates_keep_every_ordinal_and_raw_key_span() {
        let source = Arc::new(r#"{"a":0,"\u0061":1,"a":2}"#.to_owned());
        let mut b = StructureBuilder::new(source.clone(), 1).unwrap();
        let root = b
            .push_node(NodeSpec {
                kind: StructureKind::Mapping,
                text: None,
                location: Some(SourceSpan {
                    start: 0,
                    end: source.len() as u32,
                }),
            })
            .unwrap();
        let cooked = b.append_cooked(b"a").unwrap();
        let mut at = 0;
        let mut entries = Vec::new();
        let mut locations = Vec::new();
        for (ordinal, raw_key) in [r#""a""#, r#""\u0061""#, r#""a""#].into_iter().enumerate() {
            let start = at + source[at..].find(raw_key).unwrap();
            let span = SourceSpan {
                start: start as u32,
                end: (start + raw_key.len()) as u32,
            };
            let key = b
                .push_node(NodeSpec {
                    kind: StructureKind::String,
                    text: Some(TextRef::Cooked {
                        start: cooked.start,
                        end: cooked.end,
                    }),
                    location: Some(span),
                })
                .unwrap();
            let value_at = start + raw_key.len() + 1;
            let value_span = SourceSpan {
                start: value_at as u32,
                end: value_at as u32 + 1,
            };
            let value = b
                .push_node(NodeSpec {
                    kind: StructureKind::JsonNumber,
                    text: Some(TextRef::Source(value_span)),
                    location: Some(value_span),
                })
                .unwrap();
            assert_eq!(source[value_span.range()], ordinal.to_string());
            entries.push(EntrySpec {
                key: Some(key),
                value,
            });
            locations.push(span);
            at = value_at + 1;
        }
        b.set_entries(root, entries).unwrap();
        let model = b.finish(root, 1, 0, &AtomicBool::new(false)).unwrap();
        assert_eq!(
            model.duplicates(),
            DuplicateSummary {
                extra_occurrences: 2,
                first_key_span: Some(locations[1])
            }
        );
        assert_eq!(
            model
                .root()
                .entries()
                .map(|e| e.value().leaf().unwrap().text())
                .collect::<Vec<_>>(),
            ["0", "1", "2"]
        );
    }
    #[test]
    fn long_common_key_prefix_comparison_checks_cancellation_between_blocks() {
        let a = "a".repeat(32_768);
        let mut checks = 0;
        assert_eq!(
            compare_text(&a, &a, &mut || {
                checks += 1;
                checks >= 3
            })
            .unwrap_err(),
            ModelError::Cancelled
        );
        assert_eq!(checks, 3);
        let mut ids = [3, 1, 2, 0];
        heap_sort(&mut ids, |a, b| Ok(a.cmp(&b))).unwrap();
        assert_eq!(ids, [0, 1, 2, 3]);
        let mut ids = [3, 1, 2, 0];
        assert_eq!(
            heap_sort(&mut ids, |_, _| Err(ModelError::Cancelled)).unwrap_err(),
            ModelError::Cancelled
        );
    }

    #[test]
    fn live_scratch_prevents_arena_growth_before_allocation() {
        let mut builder = StructureBuilder::new(Arc::new(String::new()), 1).unwrap();
        builder
            .set_scratch_bytes(MAX_STRUCTURE_BYTES - builder.persistent_bytes())
            .unwrap();
        let leaf = NodeSpec {
            kind: StructureKind::String,
            text: Some(TextRef::Source(SourceSpan { start: 0, end: 0 })),
            location: None,
        };
        assert_eq!(builder.push_node(leaf).unwrap_err(), ModelError::Budget);
        assert_eq!(builder.model.nodes.capacity(), 0);
        assert_eq!(builder.append_cooked(b"x").unwrap_err(), ModelError::Budget);
        assert_eq!(builder.model.cooked.capacity(), 0);
        builder.set_scratch_bytes(0).unwrap();
        builder.push_node(leaf).unwrap();
        let before = builder.persistent_bytes();
        builder
            .set_scratch_bytes(MAX_STRUCTURE_BYTES - before)
            .unwrap();
        assert_eq!(builder.append_cooked(b"x").unwrap_err(), ModelError::Budget);
        assert_eq!(builder.persistent_bytes(), before);
    }
    #[test]
    fn scratch_vector_reservation_rejects_before_capacity_changes() {
        let mut vector = Vec::<u32>::new();
        assert_eq!(
            reserve(&mut vector, 1, MAX_STRUCTURE_BYTES - 3).unwrap_err(),
            ModelError::Budget
        );
        assert_eq!(vector.capacity(), 0);
        reserve(&mut vector, 1, MAX_STRUCTURE_BYTES - 4).unwrap();
        assert_eq!(vector.capacity(), 1);
        vector.push(0);
        assert_eq!(
            reserve(&mut vector, 1, MAX_STRUCTURE_BYTES - 4).unwrap_err(),
            ModelError::Budget
        );
        assert_eq!(vector.capacity(), 1);
    }

    fn span(start: u32, end: u32) -> SourceSpan {
        SourceSpan { start, end }
    }
    #[test]
    fn interpreted_unicode_and_origin_are_separate() {
        let source = Arc::new("a=\"\\u754c\"".to_owned());
        let mut builder = StructureBuilder::new(source.clone(), 7).unwrap();
        let root = builder
            .push_node(NodeSpec {
                kind: StructureKind::Mapping,
                text: None,
                location: None,
            })
            .unwrap();
        let key = builder
            .push_node(NodeSpec {
                kind: StructureKind::String,
                text: Some(TextRef::Source(span(0, 1))),
                location: Some(span(0, 1)),
            })
            .unwrap();
        let bytes = "界".as_bytes();
        builder.append_cooked(&bytes[..1]).unwrap();
        builder.append_cooked(&bytes[1..]).unwrap();
        let leaf = builder
            .push_node(NodeSpec {
                kind: StructureKind::String,
                text: Some(TextRef::Cooked { start: 0, end: 3 }),
                location: Some(span(2, 10)),
            })
            .unwrap();
        builder
            .set_entries(
                root,
                [EntrySpec {
                    key: Some(key),
                    value: leaf,
                }],
            )
            .unwrap();
        builder
            .push_declaration(
                DeclarationSpec {
                    kind: DeclarationKind::Assignment,
                    span: span(0, 10),
                    key_span: span(0, 1),
                    value_span: Some(span(2, 10)),
                    node: leaf,
                },
                [DeclarationPart {
                    span: span(0, 1),
                    node: leaf,
                }],
            )
            .unwrap();
        let model = builder.finish(root, 0, 1, &AtomicBool::new(false)).unwrap();
        assert!(Arc::ptr_eq(&source, &model.source_handle()));
        assert_eq!(
            model
                .root()
                .entry(0)
                .unwrap()
                .value()
                .leaf()
                .unwrap()
                .text(),
            "界"
        );
        assert_eq!(
            model.root().entry(0).unwrap().value().source_location(),
            Some(span(2, 10))
        );
        assert_eq!(model.generation(), 7);
    }
    #[test]
    fn invalid_boundary_and_cycle_reject_whole_model() {
        let mut builder = StructureBuilder::new(Arc::new("界".to_owned()), 1).unwrap();
        let root = builder
            .push_node(NodeSpec {
                kind: StructureKind::String,
                text: Some(TextRef::Source(span(0, 1))),
                location: None,
            })
            .unwrap();
        assert_eq!(
            builder
                .finish(root, 0, 0, &AtomicBool::new(false))
                .unwrap_err(),
            ModelError::Invalid
        );
        let mut builder = StructureBuilder::new(Arc::new(String::new()), 1).unwrap();
        let root = builder
            .push_node(NodeSpec {
                kind: StructureKind::Sequence,
                text: None,
                location: None,
            })
            .unwrap();
        builder
            .set_entries(
                root,
                [EntrySpec {
                    key: None,
                    value: root,
                }],
            )
            .unwrap();
        assert_eq!(
            builder
                .finish(root, 1, 0, &AtomicBool::new(false))
                .unwrap_err(),
            ModelError::Invalid
        );
    }
    #[test]
    fn cancellation_and_logical_depth_are_independent_of_wire() {
        for depth in [64, 65] {
            let mut builder = StructureBuilder::new(Arc::new("0".to_owned()), 1).unwrap();
            let root = builder
                .push_node(NodeSpec {
                    kind: StructureKind::Sequence,
                    text: None,
                    location: None,
                })
                .unwrap();
            let mut previous = root;
            for i in 1..=depth {
                let child = builder
                    .push_node(NodeSpec {
                        kind: if i == depth {
                            StructureKind::Integer
                        } else {
                            StructureKind::Sequence
                        },
                        text: if i == depth {
                            Some(TextRef::Source(span(0, 1)))
                        } else {
                            None
                        },
                        location: if i == depth { Some(span(0, 1)) } else { None },
                    })
                    .unwrap();
                builder
                    .set_entries(
                        previous,
                        [EntrySpec {
                            key: None,
                            value: child,
                        }],
                    )
                    .unwrap();
                previous = child;
            }
            assert_eq!(
                builder.finish(root, 0, 0, &AtomicBool::new(false)).is_ok(),
                depth == 64
            );
        }
        let builder = StructureBuilder::new(Arc::new(String::new()), 1).unwrap();
        assert_eq!(
            builder
                .finish(NodeId(0), 0, 0, &AtomicBool::new(true))
                .unwrap_err(),
            ModelError::Cancelled
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParentPart {
    Key,
    Value,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParentLink {
    pub owner: NodeId,
    pub ordinal: u32,
    pub part: ParentPart,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YamlStyle {
    Plain,
    SingleQuoted,
    DoubleQuoted,
    Literal,
    Folded,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AliasTarget {
    pub anchor: u32,
    pub target: NodeId,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct YamlMetadata {
    pub node: NodeId,
    pub document: u32,
    pub style: Option<YamlStyle>,
    pub anchor: Option<u32>,
    pub tag: Option<TextRef>,
    pub alias: Option<AliasTarget>,
    pub explicit_start: bool,
    pub synthetic_root: bool,
}
#[path = "yaml_model.rs"]
mod yaml_model;

/// Names retain lexical identity; namespace is the normalized URI, never dereferenced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XmlName {
    pub qname: SourceSpan,
    pub local: SourceSpan,
    pub prefix: Option<SourceSpan>,
    pub namespace: Option<TextRef>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum XmlReferenceKind {
    Predefined,
    Decimal,
    Hexadecimal,
}
/// A fixed-size row per XML node. Incompatible fields are forbidden by finish_xml.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XmlMetadata {
    pub node: NodeId,
    pub name: Option<XmlName>,
    pub attribute_count: u32,
    pub empty_form: bool,
    pub open_span: Option<SourceSpan>,
    pub close_span: Option<SourceSpan>,
    pub quoted_value: Option<SourceSpan>,
    pub value_content: Option<SourceSpan>,
    pub binding_prefix: Option<SourceSpan>,
    pub reference_kind: Option<XmlReferenceKind>,
    pub target: Option<SourceSpan>,
    pub root_element: Option<NodeId>,
    pub has_doctype: bool,
}
impl XmlMetadata {
    pub(crate) fn new(node: NodeId) -> Self {
        Self {
            node,
            name: None,
            attribute_count: 0,
            empty_form: false,
            open_span: None,
            close_span: None,
            quoted_value: None,
            value_content: None,
            binding_prefix: None,
            reference_kind: None,
            target: None,
            root_element: None,
            has_doctype: false,
        }
    }
}
#[path = "xml_model.rs"]
mod xml_validation;

#[cfg(test)]
mod peak_admission_tests {
    use super::*;
    #[test]
    fn realloc_peak_rejection_preserves_content_len_and_capacity() {
        let mut v = Vec::<u32>::new();
        v.try_reserve_exact(16).unwrap();
        v.extend(0..16);
        let before = (v.clone(), v.len(), v.capacity());
        let old = v.capacity() * size_of::<u32>();
        // There is room for the final 17-element buffer, but not old+new simultaneously.
        let other = MAX_STRUCTURE_BYTES - (old + 4);
        assert_eq!(reserve(&mut v, 1, other).unwrap_err(), ModelError::Budget);
        assert_eq!(v, before.0);
        assert_eq!(v.len(), before.1);
        assert_eq!(v.capacity(), before.2);
    }
}
