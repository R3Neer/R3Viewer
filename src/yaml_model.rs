//! YAML-specific whole-arena validation; legacy validation remains unchanged.
use super::*;
impl StructureBuilder {
    pub(crate) fn mark_document_synthetic(&mut self, node: NodeId) -> Result<(), ModelError> {
        let m = self
            .model
            .metadata
            .iter_mut()
            .rev()
            .find(|m| m.node == node)
            .ok_or(ModelError::Invalid)?;
        m.synthetic_root = true;
        Ok(())
    }

    pub(crate) fn push_metadata(&mut self, metadata: YamlMetadata) -> Result<(), ModelError> {
        if self.model.format != StructureFormat::Yaml
            || self
                .model
                .metadata
                .last()
                .is_some_and(|m| m.node.0 >= metadata.node.0)
            || self
                .model
                .nodes
                .get(metadata.node.0 as usize)
                .is_none_or(|n| n.spec.kind == StructureKind::YamlStream)
        {
            return Err(ModelError::Invalid);
        }
        if metadata.alias.is_some() {
            if self.model.nodes.len() + self.model.yaml_edges >= MAX_STRUCTURE_ITEMS {
                return Err(ModelError::Budget);
            }
            self.model.yaml_edges += 1;
        }
        let other = self.model.persistent_bytes()
            - self.model.metadata.capacity() * size_of::<YamlMetadata>();
        reserve(&mut self.model.metadata, 1, other + self.scratch_bytes)?;
        self.model.metadata.push(metadata);
        Ok(())
    }
    pub(crate) fn finish_yaml(
        mut self,
        root: NodeId,
        syntax: usize,
        keys: usize,
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<Structure, ModelError> {
        if cancelled() {
            return Err(ModelError::Cancelled);
        }
        if syntax > MAX_STRUCTURE_DEPTH || keys != 0 {
            return Err(ModelError::Depth);
        }
        let model = &mut self.model;
        if model.nodes.is_empty()
            || model
                .nodes
                .get(root.0 as usize)
                .is_none_or(|n| n.spec.kind != StructureKind::YamlStream)
            || !model.declarations.is_empty()
            || !model.parts.is_empty()
        {
            return Err(ModelError::Invalid);
        }
        #[derive(Clone, Copy)]
        struct Frame {
            node: NodeId,
            next: usize,
            depth: usize,
            document: u32,
        }
        let stack_bytes = size_of::<[Frame; 67]>();
        let mut seen = Vec::<u8>::new();
        let mut seen_entries = Vec::<u8>::new();
        let other = model.persistent_bytes() + self.scratch_bytes + stack_bytes;
        reserve(&mut model.parents, model.nodes.len(), other)?;
        model.parents.resize(model.nodes.len(), None);
        reserve(
            &mut seen,
            model.nodes.len(),
            model.persistent_bytes() + self.scratch_bytes + stack_bytes,
        )?;
        seen.resize(model.nodes.len(), 0);
        reserve(
            &mut seen_entries,
            model.entries.len(),
            model.persistent_bytes() + self.scratch_bytes + stack_bytes + seen.capacity(),
        )?;
        seen_entries.resize(model.entries.len(), 0);
        let valid_span = |s: SourceSpan| {
            s.start <= s.end
                && (s.end as usize) <= model.source.len()
                && model.source.is_char_boundary(s.start as usize)
                && model.source.is_char_boundary(s.end as usize)
        };
        let cooked = std::str::from_utf8(&model.cooked).map_err(|_| ModelError::Invalid)?;
        let valid_text = |t: TextRef| match t {
            TextRef::Source(s) => valid_span(s),
            TextRef::Cooked { start, end } => {
                start <= end
                    && end as usize <= cooked.len()
                    && cooked.is_char_boundary(start as usize)
                    && cooked.is_char_boundary(end as usize)
            }
        };
        for (i, n) in model.nodes.iter().enumerate() {
            if i % 1024 == 0 && cancelled() {
                return Err(ModelError::Cancelled);
            }
            let no_text = n.spec.kind.is_container() || n.spec.kind == StructureKind::Alias;
            if no_text != n.spec.text.is_none()
                || n.entries.end > model.entries.len()
                || n.entries.start > n.entries.end
                || (!n.spec.kind.is_container() && !n.entries.is_empty())
                || n.spec.location.is_some_and(|s| !valid_span(s))
                || n.spec.text.is_some_and(|t| !valid_text(t))
            {
                return Err(ModelError::Invalid);
            }
            if matches!(
                n.spec.kind,
                StructureKind::ArrayOfTables
                    | StructureKind::JsonNumber
                    | StructureKind::OffsetDateTime
                    | StructureKind::LocalDateTime
                    | StructureKind::LocalDate
                    | StructureKind::LocalTime
            ) {
                return Err(ModelError::Invalid);
            }
            if matches!(
                n.spec.kind,
                StructureKind::YamlStream | StructureKind::YamlDocument
            ) && n.spec.location.is_some()
            {
                return Err(ModelError::Invalid);
            }
            if let Some(s) = n.spec.location
                && n.spec.kind.is_container()
            {
                let bytes = &model.source.as_bytes()[s.range()];
                let (a, b) = if n.spec.kind == StructureKind::Mapping {
                    (b'{', b'}')
                } else {
                    (b'[', b']')
                };
                if bytes.first() != Some(&a) || bytes.last() != Some(&b) {
                    return Err(ModelError::Invalid);
                }
            }
            if n.spec.kind != StructureKind::YamlStream
                && model.metadata(NodeId(i as u32)).is_none()
            {
                return Err(ModelError::Invalid);
            }
        }
        let mut last_anchor = None;
        for (i, m) in model.metadata.iter().enumerate() {
            if i % 1024 == 0 && cancelled() {
                return Err(ModelError::Cancelled);
            }
            let n = model.node(m.node).ok_or(ModelError::Invalid)?;
            if i > 0 && model.metadata[i - 1].node.0 >= m.node.0
                || m.tag.is_some_and(|t| !valid_text(t))
                || (n.kind() == StructureKind::Alias) != m.alias.is_some()
                || m.anchor == Some(0)
                || (n.kind() == StructureKind::YamlDocument
                    && (m.anchor.is_some() || m.tag.is_some() || m.alias.is_some()))
                || ((m.explicit_start || m.synthetic_root)
                    && n.kind() != StructureKind::YamlDocument)
            {
                return Err(ModelError::Invalid);
            }
            if let Some(anchor) = m.anchor {
                if last_anchor.is_some_and(|(doc, id)| doc == m.document && id >= anchor) {
                    return Err(ModelError::Invalid);
                }
                last_anchor = Some((m.document, anchor));
            }
            if let Some(alias) = m.alias {
                let target = model.metadata(alias.target).ok_or(ModelError::Invalid)?;
                if alias.target.0 >= m.node.0
                    || target.document != m.document
                    || target.anchor != Some(alias.anchor)
                    || m.style.is_some()
                    || m.tag.is_some()
                    || m.anchor.is_some()
                {
                    return Err(ModelError::Invalid);
                }
                if let Some(s) = n.source_location()
                    && !model.source[s.range()].starts_with('*')
                {
                    return Err(ModelError::Invalid);
                }
            } else if let Some(decoded) = n.decoded() {
                let style = m.style.ok_or(ModelError::Invalid)?;
                let tag = m.tag.map(|t| model.text(t));
                let expected = crate::yaml_worker::scalar_kind(decoded, style, tag, &mut cancelled)
                    .map_err(|_| ModelError::Invalid)?;
                if expected != n.kind() {
                    return Err(ModelError::Invalid);
                }
                if style == YamlStyle::Plain && decoded.is_empty() && n.source_location().is_some()
                {
                    return Err(ModelError::Invalid);
                }
            } else {
                if m.style.is_some() {
                    return Err(ModelError::Invalid);
                }
                crate::yaml_worker::validate_shape(n.kind(), m.tag.map(|t| model.text(t)))
                    .map_err(|_| ModelError::Invalid)?;
            }
        }
        let mut stack = [Frame {
            node: root,
            next: 0,
            depth: 0,
            document: u32::MAX,
        }; 67];
        let mut top = 0;
        let mut max_depth = 0;
        let mut count = 1;
        let mut edges = 0;
        let mut ticks = 0;
        seen[root.0 as usize] = 1;
        loop {
            ticks += 1;
            if ticks % 1024 == 0 && cancelled() {
                return Err(ModelError::Cancelled);
            }
            let f = stack[top];
            let n = &model.nodes[f.node.0 as usize];
            let mapping = n.spec.kind == StructureKind::Mapping;
            let steps = n.entries.len() * (if mapping { 2 } else { 1 });
            if n.spec.kind == StructureKind::YamlDocument {
                if n.entries.len() != 1 {
                    return Err(ModelError::Invalid);
                }
                if model.metadata(f.node).is_some_and(|m| m.synthetic_root) {
                    let child = model
                        .node(model.entries[n.entries.start].spec.value)
                        .ok_or(ModelError::Invalid)?;
                    if child.decoded() != Some("") || child.source_location().is_some() {
                        return Err(ModelError::Invalid);
                    }
                }
            }
            if f.next == steps {
                if top == 0 {
                    break;
                }
                top -= 1;
                continue;
            }
            let ordinal = f.next / (if mapping { 2 } else { 1 });
            let part = if mapping && f.next % 2 == 0 {
                ParentPart::Key
            } else {
                ParentPart::Value
            };
            let idx = n.entries.start + ordinal;
            if part == ParentPart::Value {
                if seen_entries[idx] != 0 {
                    return Err(ModelError::Invalid);
                }
                seen_entries[idx] = 1;
            }
            let e = model.entries[idx].spec;
            if mapping != e.key.is_some() {
                return Err(ModelError::Invalid);
            }
            let child = if part == ParentPart::Key {
                e.key.unwrap()
            } else {
                e.value
            };
            let c = model
                .nodes
                .get(child.0 as usize)
                .ok_or(ModelError::Invalid)?;
            if seen[child.0 as usize] != 0 {
                return Err(ModelError::Invalid);
            }
            let document = if n.spec.kind == StructureKind::YamlStream {
                if c.spec.kind != StructureKind::YamlDocument {
                    return Err(ModelError::Invalid);
                }
                ordinal as u32
            } else {
                f.document
            };
            if c.spec.kind == StructureKind::YamlStream
                || (c.spec.kind == StructureKind::YamlDocument
                    && n.spec.kind != StructureKind::YamlStream)
                || model.metadata(child).is_none_or(|m| m.document != document)
            {
                return Err(ModelError::Invalid);
            }
            if let (Some(a), Some(b)) = (n.spec.location, c.spec.location)
                && (b.start <= a.start || b.end >= a.end)
            {
                return Err(ModelError::Invalid);
            }
            let depth = f.depth
                + usize::from(matches!(
                    c.spec.kind,
                    StructureKind::Mapping | StructureKind::Sequence
                ));
            max_depth = max_depth.max(depth);
            if depth > 64 || top + 1 >= 67 {
                return Err(ModelError::Depth);
            }
            model.parents[child.0 as usize] = Some(ParentLink {
                owner: f.node,
                ordinal: ordinal as u32,
                part,
            });
            seen[child.0 as usize] = 1;
            count += 1;
            edges += 1;
            stack[top].next += 1;
            top += 1;
            stack[top] = Frame {
                node: child,
                next: 0,
                depth,
                document,
            };
        }
        let aliases = model.alias_edges();
        if count != model.nodes.len()
            || seen_entries.contains(&0)
            || count + edges + aliases > MAX_STRUCTURE_ITEMS
            || max_depth != syntax
        {
            return Err(ModelError::Invalid);
        }
        let other = model.persistent_bytes()
            + self.scratch_bytes
            + stack_bytes
            + seen.capacity()
            + seen_entries.capacity();
        let mut ids = Vec::new();
        let mut duplicates = DuplicateSummary::default();
        for n in &model.nodes {
            if cancelled() {
                return Err(ModelError::Cancelled);
            }
            if n.spec.kind != StructureKind::Mapping {
                continue;
            }
            ids.clear();
            reserve(&mut ids, n.entries.len(), other)?;
            for (i, id) in n.entries.clone().enumerate() {
                if i % 1024 == 0 && cancelled() {
                    return Err(ModelError::Cancelled);
                }
                let k = model.entries[id].spec.key.unwrap();
                if model.nodes[k.0 as usize].spec.kind == StructureKind::String {
                    ids.push(id);
                }
            }
            let text = |id: usize| {
                model
                    .node(model.entries[id].spec.key.unwrap())
                    .unwrap()
                    .decoded()
                    .unwrap()
            };
            heap_sort(&mut ids, |a, b| {
                Ok(compare_text(text(a), text(b), &mut cancelled)?.then(a.cmp(&b)))
            })?;
            for pair in ids.windows(2) {
                if compare_text(text(pair[0]), text(pair[1]), &mut cancelled)?.is_eq() {
                    duplicates.extra_occurrences += 1;
                    let span = model.nodes[model.entries[pair[1]].spec.key.unwrap().0 as usize]
                        .spec
                        .location;
                    if let Some(s) = span
                        && duplicates
                            .first_key_span
                            .is_none_or(|old| s.start < old.start)
                    {
                        duplicates.first_key_span = span;
                    }
                }
            }
        }
        if cancelled() {
            return Err(ModelError::Cancelled);
        }
        model.duplicates = duplicates;
        model.root = root;
        Ok(self.model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn candidate() -> (Structure, usize) {
        let (s, d, _) = crate::yaml_worker::project(
            Arc::new("{a: &x [*x], b: 2}".into()),
            8,
            &AtomicBool::new(false),
        )
        .unwrap();
        (s, d)
    }
    #[test]
    fn receiver_derives_owners_and_rejects_alias_scope_target_and_reused_children() {
        for alteration in 0..6 {
            let (mut s, depth) = candidate();
            let alias = s.metadata.iter().position(|m| m.alias.is_some()).unwrap();
            match alteration {
                0 => s.metadata[alias].document = 17,
                1 => s.metadata[alias].alias.as_mut().unwrap().target = s.metadata[alias].node,
                2 => s.metadata[alias].alias.as_mut().unwrap().anchor = 999,
                3 => s.metadata[alias].style = Some(YamlStyle::Plain),
                4 => {
                    let owner = s
                        .root()
                        .entry(0)
                        .unwrap()
                        .value()
                        .entry(0)
                        .unwrap()
                        .value()
                        .id();
                    let range = s.nodes[owner.0 as usize].entries.clone();
                    s.entries[range.start + 1].spec.value = s.entries[range.start].spec.value;
                }
                _ => {
                    let m = s.metadata[alias];
                    s.metadata.insert(alias, m);
                }
            }
            assert!(
                StructureBuilder::from_structure(s)
                    .finish(NodeId(0), depth, 0, &AtomicBool::new(false))
                    .is_err(),
                "alteration {alteration}"
            );
        }
    }
    #[test]
    fn metadata_growth_rejected_before_capacity_and_yaml_budget_reserves_session() {
        let mut b =
            StructureBuilder::new_for_format(Arc::new("x".into()), 1, StructureFormat::Yaml)
                .unwrap();
        let n = b
            .push_node(NodeSpec {
                kind: StructureKind::String,
                text: Some(TextRef::Source(SourceSpan { start: 0, end: 1 })),
                location: Some(SourceSpan { start: 0, end: 1 }),
            })
            .unwrap();
        b.set_scratch_bytes(MAX_STRUCTURE_BYTES - b.persistent_bytes())
            .unwrap();
        assert_eq!(
            b.push_metadata(YamlMetadata {
                node: n,
                document: 0,
                style: Some(YamlStyle::Plain),
                anchor: None,
                tag: None,
                alias: None,
                explicit_start: false,
                synthetic_root: false
            })
            .unwrap_err(),
            ModelError::Budget
        );
        assert_eq!(b.model.metadata.capacity(), 0);
        assert_eq!(b.model.session_budget_bytes(), 1024 * 1024);
    }
    #[test]
    fn receiver_rejects_forged_flow_span_and_reported_depth() {
        let (mut s, d) = candidate();
        let root = s
            .root()
            .entry(0)
            .unwrap()
            .value()
            .entry(0)
            .unwrap()
            .value()
            .id();
        s.nodes[root.0 as usize].spec.location = Some(SourceSpan { start: 1, end: 3 });
        assert_eq!(
            StructureBuilder::from_structure(s)
                .finish(NodeId(0), d, 0, &AtomicBool::new(false))
                .unwrap_err(),
            ModelError::Invalid
        );
        let (s, d) = candidate();
        assert_eq!(
            StructureBuilder::from_structure(s)
                .finish(NodeId(0), d + 1, 0, &AtomicBool::new(false))
                .unwrap_err(),
            ModelError::Invalid
        );
    }
}
