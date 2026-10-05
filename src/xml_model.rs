//! XML arena invariants. Source semantics are independently replayed after transport.
use super::*;
impl StructureBuilder {
    pub(crate) fn push_xml_metadata(&mut self, m: XmlMetadata) -> Result<(), ModelError> {
        if self.model.format != StructureFormat::Xml
            || m.node.0 as usize != self.model.xml_metadata.len()
            || m.node.0 as usize >= self.model.nodes.len()
        {
            return Err(ModelError::Invalid);
        }
        let other = self.model.persistent_bytes()
            - self.model.xml_metadata.capacity() * size_of::<XmlMetadata>();
        reserve(&mut self.model.xml_metadata, 1, other + self.scratch_bytes)?;
        self.model.xml_metadata.push(m);
        Ok(())
    }
    pub(crate) fn replace_xml_metadata(&mut self, m: XmlMetadata) -> Result<(), ModelError> {
        let slot = self
            .model
            .xml_metadata
            .get_mut(m.node.0 as usize)
            .ok_or(ModelError::Invalid)?;
        *slot = m;
        Ok(())
    }
    pub(crate) fn xml_text(&self, t: TextRef) -> &str {
        self.model.text_ref(t)
    }
    pub(crate) fn xml_metadata_mut(&mut self, id: NodeId) -> Result<&mut XmlMetadata, ModelError> {
        self.model
            .xml_metadata
            .get_mut(id.0 as usize)
            .ok_or(ModelError::Invalid)
    }
    pub(crate) fn finish_xml(
        mut self,
        root: NodeId,
        syntax: usize,
        keys: usize,
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<Structure, ModelError> {
        if cancelled() {
            return Err(ModelError::Cancelled);
        }
        if syntax > MAX_STRUCTURE_DEPTH {
            return Err(ModelError::Depth);
        }
        let m = &mut self.model;
        if root != NodeId(0)
            || keys != 0
            || m.nodes.is_empty()
            || m.xml_metadata.len() != m.nodes.len()
            || !m.metadata.is_empty()
            || !m.declarations.is_empty()
            || !m.parts.is_empty()
            || m.nodes[0].spec.kind != StructureKind::XmlDocument
        {
            return Err(ModelError::Invalid);
        }
        let valid_span = |s: SourceSpan| {
            s.start <= s.end
                && s.end as usize <= m.source.len()
                && m.source.is_char_boundary(s.start as usize)
                && m.source.is_char_boundary(s.end as usize)
        };
        let cooked = std::str::from_utf8(&m.cooked).map_err(|_| ModelError::Invalid)?;
        let valid_text = |t: TextRef| match t {
            TextRef::Source(s) => valid_span(s),
            TextRef::Cooked { start, end } => {
                start <= end
                    && end as usize <= cooked.len()
                    && cooked.is_char_boundary(start as usize)
                    && cooked.is_char_boundary(end as usize)
            }
        };
        for (i, n) in m.nodes.iter().enumerate() {
            if i % 1024 == 0 && cancelled() {
                return Err(ModelError::Cancelled);
            }
            if !n.spec.kind.is_xml()
                || m.xml_metadata[i].node != NodeId(i as u32)
                || n.entries.start > n.entries.end
                || n.entries.end > m.entries.len()
                || (!n.spec.kind.is_container() && !n.entries.is_empty())
                || n.spec.kind.is_container() != n.spec.text.is_none()
                || n.spec.location.is_some_and(|s| !valid_span(s))
                || n.spec.text.is_some_and(|t| !valid_text(t))
                || (i == 0) != n.spec.location.is_none()
            {
                return Err(ModelError::Invalid);
            }
            let x = &m.xml_metadata[i];
            if [
                x.open_span,
                x.close_span,
                x.quoted_value,
                x.value_content,
                x.binding_prefix,
                x.target,
            ]
            .into_iter()
            .flatten()
            .any(|s| !valid_span(s))
                || x.name.is_some_and(|n| {
                    !valid_span(n.qname)
                        || !valid_span(n.local)
                        || n.prefix.is_some_and(|s| !valid_span(s))
                        || n.namespace.is_some_and(|t| !valid_text(t))
                })
            {
                return Err(ModelError::Invalid);
            }
        }
        let mut seen = Vec::<u8>::new();
        let mut edges = Vec::<u8>::new();
        let stack_bytes = size_of::<[(NodeId, usize, usize); 66]>();
        let parent_other = m.persistent_bytes() + self.scratch_bytes + stack_bytes;
        reserve(&mut m.parents, m.nodes.len(), parent_other)?;
        m.parents.resize(m.nodes.len(), None);
        reserve(
            &mut seen,
            m.nodes.len(),
            m.persistent_bytes() + self.scratch_bytes + stack_bytes,
        )?;
        seen.resize(m.nodes.len(), 0);
        reserve(
            &mut edges,
            m.entries.len(),
            m.persistent_bytes() + self.scratch_bytes + stack_bytes + seen.capacity(),
        )?;
        edges.resize(m.entries.len(), 0);
        let mut stack = [(NodeId(0), 0usize, 0usize); 66];
        let mut len = 1;
        seen[0] = 1;
        let mut depth = 0;
        while len > 0 {
            if cancelled() {
                return Err(ModelError::Cancelled);
            }
            let (owner, at, d) = stack[len - 1];
            let n = &m.nodes[owner.0 as usize];
            if at == n.entries.len() {
                len -= 1;
                continue;
            }
            stack[len - 1].1 += 1;
            let eid = n.entries.start + at;
            if edges[eid] != 0 {
                return Err(ModelError::Invalid);
            }
            edges[eid] = 1;
            let e = m.entries[eid].spec;
            if e.key.is_some()
                || e.value.0 as usize >= m.nodes.len()
                || e.value == root
                || seen[e.value.0 as usize] != 0
            {
                return Err(ModelError::Invalid);
            }
            seen[e.value.0 as usize] = 1;
            m.parents[e.value.0 as usize] = Some(ParentLink {
                owner,
                ordinal: at as u32,
                part: ParentPart::Value,
            });
            let child = &m.nodes[e.value.0 as usize];
            let cd = d + usize::from(child.spec.kind == StructureKind::XmlElement);
            depth = depth.max(cd);
            if cd > MAX_STRUCTURE_DEPTH {
                return Err(ModelError::Depth);
            }
            if len == stack.len() {
                return Err(ModelError::Depth);
            }
            stack[len] = (e.value, 0, cd);
            len += 1;
        }
        if seen.contains(&0) || edges.contains(&0) || depth != syntax {
            return Err(ModelError::Invalid);
        }
        m.root = root;
        drop(seen);
        drop(edges);
        crate::xml_worker::validate_source(m, &mut cancelled)?;
        Ok(self.model)
    }
}
impl StructureKind {
    pub fn is_xml(self) -> bool {
        (self as u8) >= Self::XmlDocument as u8
    }
}
