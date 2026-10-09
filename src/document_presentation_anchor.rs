//! Logical presentation positions. Decorations never manufacture Source, Run or Domain identity.
use crate::{
    document_consumer_budget::{Charge, ChargedVec, ConsumerBudget},
    document_model::{Document, NONE},
    document_roles::{self as roles, BlockRole},
    document_text::{DocumentWork, Error, Result},
    document_view::{LayoutAnchor, RawAnchor},
};
use std::{mem::size_of, sync::Arc};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PaintRole {
    Value,
    Heading,
    Code,
    Secondary,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RefField {
    Input,
    Title,
    Kind,
    Info,
    Label,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RefOwner {
    Resource {
        block: u32,
        run: u32,
        resource: u32,
        field: RefField,
    },
    BlockRole {
        block: u32,
        field: RefField,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RefAnchor {
    owner: RefOwner,
    text_ref: u32,
    local: usize,
    part: u8,
}
impl RefAnchor {
    pub(crate) fn checked(
        document: &Document,
        owner: RefOwner,
        id: u32,
        local: usize,
        part: u8,
        w: &mut DocumentWork<'_>,
    ) -> Result<Self> {
        let expected = match owner {
            RefOwner::Resource {
                block,
                run,
                resource,
                field,
            } => {
                if run == NONE {
                    if !matches!(roles::block_role(document,block,w)?,BlockRole::Code{mermaid:Some(r),..}if r==resource)
                    {
                        return Err(Error::Invalid);
                    }
                } else {
                    w.admit_structure(size_of::<crate::document_model::Run>())?;
                    let r = document.runs().get(run as usize).ok_or(Error::Invalid)?;
                    if r.kind() != 5
                        || r.resource() != resource
                        || document
                            .domains()
                            .get(r.domain() as usize)
                            .is_none_or(|d| d.owner() != block)
                    {
                        return Err(Error::Invalid);
                    }
                }
                let d = roles::resource_descriptor(document, resource, w)?;
                match field {
                    RefField::Input => Some(d.input_ref()),
                    RefField::Title => d.title_ref(),
                    _ => None,
                }
            }
            RefOwner::BlockRole { block, field } => {
                match (roles::block_role(document, block, w)?, field) {
                    (BlockRole::Callout { kind_text, .. }, RefField::Kind) => Some(kind_text),
                    (BlockRole::Code { info, .. }, RefField::Info) => info,
                    (BlockRole::Footnote { label }, RefField::Label) => Some(label),
                    _ => None,
                }
            }
        };
        if expected != Some(id) {
            return Err(Error::Invalid);
        }
        let t = roles::text_ref_view(document, id, w)?;
        let s = t.semantic_text();
        if local > s.len() || !s.is_char_boundary(local) {
            return Err(Error::Invalid);
        }
        // Actual visual affinity is validated again by the owning TextRef provider before seeking.
        if local == s.len() && part != 0 {
            return Err(Error::Invalid);
        }
        crate::document_text::validate_ref_visual_part(document, id, local, part, w)?;
        Ok(Self {
            owner,
            text_ref: id,
            local,
            part,
        })
    }
    pub(crate) fn from_provider(owner: RefOwner, text_ref: u32, local: usize, part: u8) -> Self {
        Self {
            owner,
            text_ref,
            local,
            part,
        }
    }
    pub(crate) fn owner(&self) -> RefOwner {
        self.owner
    }
    pub(crate) fn text_ref(&self) -> u32 {
        self.text_ref
    }
    pub(crate) fn local(&self) -> usize {
        self.local
    }
    pub(crate) fn part(&self) -> u8 {
        self.part
    }
    pub(crate) fn block(&self) -> u32 {
        match self.owner {
            RefOwner::Resource { block, .. } | RefOwner::BlockRole { block, .. } => block,
        }
    }
    pub(crate) fn field(&self) -> RefField {
        match self.owner {
            RefOwner::Resource { field, .. } | RefOwner::BlockRole { field, .. } => field,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RoleAnchor {
    block: u32,
    run: u32,
    ordinal: u64,
    phase: u8,
    offset: u16,
}
impl RoleAnchor {
    pub(crate) fn run(&self) -> u32 {
        self.run
    }
    pub(crate) fn ordinal(&self) -> u64 {
        self.ordinal
    }
    pub(crate) fn block(&self) -> u32 {
        self.block
    }
    pub(crate) fn phase(&self) -> u8 {
        self.phase
    }
    pub(crate) fn offset(&self) -> u16 {
        self.offset
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CellProgress {
    At(LayoutAnchor),
    Done,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TableCellProgress {
    pub(crate) cell: u32,
    pub(crate) progress: CellProgress,
}
pub(crate) struct TableResume {
    row: u32,
    selected: u32,
    cells: ChargedVec<TableCellProgress>,
    _metadata: Charge,
}
impl TableResume {
    pub(crate) fn new(
        document: &Document,
        row: u32,
        selected: u32,
        values: &[TableCellProgress],
        budget: &ConsumerBudget,
        w: &mut DocumentWork<'_>,
    ) -> Result<Arc<Self>> {
        w.admit_structure(size_of::<crate::document_model::Block>())?;
        let r = document.blocks().get(row as usize).ok_or(Error::Invalid)?;
        if r.kind() != 8 {
            return Err(Error::Invalid);
        }
        let metadata = budget.reserve(size_of::<Self>() + 2 * size_of::<usize>())?;
        let mut cells = ChargedVec::new(budget, values.len())?;
        let mut found = false;
        for &v in values {
            w.admit_structure(
                size_of::<TableCellProgress>() + size_of::<crate::document_model::Block>(),
            )?;
            let c = document
                .blocks()
                .get(v.cell as usize)
                .ok_or(Error::Invalid)?;
            if c.kind() != 9 || c.parent() != row {
                return Err(Error::Invalid);
            }
            if let CellProgress::At(a) = v.progress {
                if a.raw.block != v.cell || a.raw.run == NONE {
                    return Err(Error::Invalid);
                }
                crate::document_view::validate_raw_anchor(document, a.raw, w)?;
                crate::document_text::validate_visual_part(
                    document,
                    a.raw.run,
                    a.raw.byte,
                    a.projection_part,
                    w,
                )?;
            }
            found |= v.cell == selected;
            cells.push(v)?;
        }
        if !found {
            return Err(Error::Invalid);
        }
        Ok(Arc::new(Self {
            row,
            selected,
            cells,
            _metadata: metadata,
        }))
    }
    pub(crate) fn requested_bytes(cells: usize) -> Result<usize> {
        (size_of::<Self>() + 2 * size_of::<usize>())
            .checked_add(
                cells
                    .checked_mul(size_of::<TableCellProgress>())
                    .ok_or(Error::Budget)?,
            )
            .ok_or(Error::Budget)
    }
    pub(crate) fn bytes(&self) -> usize {
        size_of::<Self>()
            + 2 * size_of::<usize>()
            + self.cells.capacity() * size_of::<TableCellProgress>()
    }
    pub(crate) fn row(&self) -> u32 {
        self.row
    }
    pub(crate) fn selected(&self) -> u32 {
        self.selected
    }
    pub(crate) fn cells(&self) -> &[TableCellProgress] {
        self.cells.as_slice()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ResourceAltAnchor {
    block: u32,
    run: u32,
    resource: u32,
    domain: u32,
    position: LayoutAnchor,
}
impl ResourceAltAnchor {
    pub(crate) fn checked(
        document: &Document,
        block: u32,
        run: u32,
        resource: u32,
        domain: u32,
        position: LayoutAnchor,
        w: &mut DocumentWork<'_>,
    ) -> Result<Self> {
        let d = roles::resource_descriptor(document, resource, w)?;
        if d.alt_domain() != Some(domain) {
            return Err(Error::Invalid);
        }
        w.admit_structure(
            size_of::<crate::document_model::Run>() + size_of::<crate::document_model::Domain>(),
        )?;
        let owner = document.runs().get(run as usize).ok_or(Error::Invalid)?;
        if owner.kind() != 5
            || owner.resource() != resource
            || document.domains()[owner.domain() as usize].owner() != block
        {
            return Err(Error::Invalid);
        }
        let record = document
            .runs()
            .get(position.raw.run as usize)
            .ok_or(Error::Invalid)?;
        if record.domain() != domain || position.raw.block != block {
            return Err(Error::Invalid);
        }
        let text = document.text(record.text()).map_err(|_| Error::Invalid)?;
        if position.raw.byte > text.len() || !text.is_char_boundary(position.raw.byte) {
            return Err(Error::Invalid);
        }
        crate::document_text::validate_visual_part(
            document,
            position.raw.run,
            position.raw.byte,
            position.projection_part,
            w,
        )?;
        Ok(Self {
            block,
            run,
            resource,
            domain,
            position,
        })
    }
    pub(crate) fn block(&self) -> u32 {
        self.block
    }
    pub(crate) fn run(&self) -> u32 {
        self.run
    }
    pub(crate) fn resource(&self) -> u32 {
        self.resource
    }
    pub(crate) fn domain(&self) -> u32 {
        self.domain
    }
    pub(crate) fn position(&self) -> LayoutAnchor {
        self.position
    }
    pub(crate) fn from_provider(
        block: u32,
        run: u32,
        resource: u32,
        domain: u32,
        position: LayoutAnchor,
    ) -> Self {
        Self {
            block,
            run,
            resource,
            domain,
            position,
        }
    }
}
#[derive(Clone)]
pub(crate) enum PresentationAnchor {
    Domain(LayoutAnchor),
    Role(RoleAnchor),
    Ref(RefAnchor),
    ResourceAlt(ResourceAltAnchor),
    Table(Arc<TableResume>),
}
impl PresentationAnchor {
    pub(crate) fn block(&self) -> u32 {
        match self {
            Self::Domain(a) => a.raw.block,
            Self::Role(a) => a.block(),
            Self::Ref(a) => a.block(),
            Self::ResourceAlt(a) => a.block(),
            Self::Table(a) => a.selected(),
        }
    }
    pub(crate) fn layout_selection(&self) -> LayoutAnchor {
        match self {
            Self::Domain(a) => *a,
            Self::Role(a) => LayoutAnchor {
                raw: RawAnchor {
                    block: a.block(),
                    run: a.run(),
                    byte: 0,
                },
                projection_part: 0,
            },
            Self::ResourceAlt(a) => LayoutAnchor {
                raw: RawAnchor {
                    block: a.block(),
                    run: a.run(),
                    byte: 0,
                },
                projection_part: 0,
            },
            Self::Table(t) => t
                .cells()
                .iter()
                .find_map(|v| {
                    if v.cell == t.selected() {
                        if let CellProgress::At(a) = v.progress {
                            Some(a)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .unwrap_or(LayoutAnchor {
                    raw: RawAnchor {
                        block: t.selected(),
                        run: NONE,
                        byte: 0,
                    },
                    projection_part: 0,
                }),
            Self::Ref(a) => match a.owner() {
                RefOwner::Resource { block, run, .. } => LayoutAnchor {
                    raw: RawAnchor {
                        block,
                        run,
                        byte: 0,
                    },
                    projection_part: 0,
                },
                _ => LayoutAnchor {
                    raw: RawAnchor {
                        block: a.block(),
                        run: NONE,
                        byte: 0,
                    },
                    projection_part: 0,
                },
            },
        }
    }
}
/// Compact shared origin per lane or decoration, not copied into every cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OriginMeta {
    Role {
        block: u32,
        run: u32,
        ordinal: u64,
        phase: u8,
        paint: PaintRole,
    },
    Ref {
        owner: RefOwner,
        text_ref: u32,
        paint: PaintRole,
    },
    Alt {
        block: u32,
        run: u32,
        resource: u32,
        domain: u32,
        paint: PaintRole,
    },
}

const _: () = assert!(size_of::<OriginMeta>() <= 32);

impl OriginMeta {
    pub(crate) fn role_anchor(&self, offset: u16) -> Result<RoleAnchor> {
        match *self {
            Self::Role {
                block,
                run,
                ordinal,
                phase,
                ..
            } => Ok(RoleAnchor {
                block,
                run,
                ordinal,
                phase,
                offset,
            }),
            _ => Err(Error::Invalid),
        }
    }
}
