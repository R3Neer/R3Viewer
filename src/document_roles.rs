//! Borrowed semantic roles shared by viewport and complete inline emission.
//! No geometry, IO, String flattening, Source mapping fabrication or graphical success.
#![allow(dead_code)] // b2 consumers are being connected in this private copy.
use crate::{
    document_model::{Document, NONE, Origin, Resource},
    document_text::{DocumentWork, Error, Result},
};
use std::{mem::size_of, ops::Range};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TaskState {
    Absent,
    Unchecked,
    Checked,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Alignment {
    None,
    Left,
    Center,
    Right,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BlockRole {
    Root,
    Paragraph,
    Heading {
        level: u32,
    },
    Quote,
    Callout {
        kind_text: u32,
        title_domain: Option<u32>,
        initially_closed: bool,
    },
    List {
        ordered: bool,
        start: u32,
        tight: bool,
    },
    Item {
        task: TaskState,
    },
    Table {
        columns: u32,
    },
    Row {
        header: bool,
    },
    Cell {
        alignment: Alignment,
    },
    Code {
        fenced: bool,
        info: Option<u32>,
        language: Option<u32>,
        mermaid: Option<u32>,
    },
    Rule,
    Footnote {
        label: u32,
    },
    HtmlInert,
    Frontmatter,
}
fn optional(id: u32) -> Option<u32> {
    (id != NONE).then_some(id)
}
pub(crate) fn block_role(
    document: &Document,
    block: u32,
    work: &mut DocumentWork<'_>,
) -> Result<BlockRole> {
    work.admit_structure(size_of::<crate::document_model::Block>())?;
    let b = document
        .blocks()
        .get(block as usize)
        .ok_or(Error::Invalid)?;
    Ok(match b.kind() {
        0 => BlockRole::Root,
        1 => BlockRole::Paragraph,
        2 => BlockRole::Heading { level: b.aux0() },
        3 => BlockRole::Quote,
        4 => BlockRole::Callout {
            kind_text: b.aux0(),
            title_domain: optional(b.aux1()),
            initially_closed: b.aux2() == 2,
        },
        5 => BlockRole::List {
            ordered: b.aux0() == 1,
            start: b.aux1(),
            tight: b.aux2() == 1,
        },
        6 => BlockRole::Item {
            task: match b.aux0() {
                0 => TaskState::Absent,
                1 => TaskState::Unchecked,
                2 => TaskState::Checked,
                _ => return Err(Error::Invalid),
            },
        },
        7 => BlockRole::Table { columns: b.aux0() },
        8 => BlockRole::Row {
            header: b.aux0() == 1,
        },
        9 => BlockRole::Cell {
            alignment: match b.aux0() {
                0 => Alignment::None,
                1 => Alignment::Left,
                2 => Alignment::Center,
                3 => Alignment::Right,
                _ => return Err(Error::Invalid),
            },
        },
        10 => BlockRole::Code {
            fenced: b.aux0() == 1,
            info: optional(b.aux1()),
            language: optional(b.aux2()),
            mermaid: optional(b.aux3()),
        },
        11 => BlockRole::Rule,
        12 => BlockRole::Footnote { label: b.aux0() },
        13 => BlockRole::HtmlInert,
        14 => BlockRole::Frontmatter,
        _ => return Err(Error::Invalid),
    })
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ResourceKind {
    Image,
    Mermaid,
    MathInline,
    MathBlock,
    WikiEmbed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ResourceState {
    PendingRequired,
}
pub(crate) struct ResourceDescriptor<'a> {
    document: &'a Document,
    record: &'a Resource,
    kind: ResourceKind,
}
impl ResourceDescriptor<'_> {
    pub(crate) fn id(&self) -> u32 {
        self.record.id()
    }
    pub(crate) fn kind(&self) -> ResourceKind {
        self.kind
    }
    pub(crate) fn state(&self) -> ResourceState {
        ResourceState::PendingRequired
    }
    pub(crate) fn block_intent(&self) -> bool {
        self.record.intent() == 1
    }
    pub(crate) fn alt_domain(&self) -> Option<u32> {
        optional(self.record.alt_domain())
    }
    pub(crate) fn input_ref(&self) -> u32 {
        self.record.input_decoded()
    }
    pub(crate) fn raw_ref(&self) -> u32 {
        self.record.raw_token()
    }
    pub(crate) fn title_ref(&self) -> Option<u32> {
        optional(self.record.title_decoded())
    }
    pub(crate) fn dimension_intent(&self) -> (Option<u32>, Option<u32>) {
        (
            (self.record.width_kind() == 1).then_some(self.record.width_value()),
            (self.record.height_kind() == 1).then_some(self.record.height_value()),
        )
    }
    pub(crate) fn origin(&self, work: &mut DocumentWork<'_>) -> Result<&Origin> {
        work.admit_structure(size_of::<Origin>())?;
        self.document
            .origins()
            .get(self.record.origin() as usize)
            .ok_or(Error::Invalid)
    }
}
pub(crate) fn resource_descriptor<'a>(
    document: &'a Document,
    id: u32,
    work: &mut DocumentWork<'_>,
) -> Result<ResourceDescriptor<'a>> {
    work.admit_structure(size_of::<Resource>())?;
    let record = document
        .resources()
        .get(id as usize)
        .ok_or(Error::Invalid)?;
    let kind = match record.kind() {
        0 => ResourceKind::Image,
        1 => ResourceKind::Mermaid,
        2 => ResourceKind::MathInline,
        3 => ResourceKind::MathBlock,
        4 => ResourceKind::WikiEmbed,
        _ => return Err(Error::Invalid),
    };
    Ok(ResourceDescriptor {
        document,
        record,
        kind,
    })
}
/// Semantic payload only: callers MUST run the borrowed Unicode/control provider before output.
/// This never constructs a Domain/Run or claims a decoded scalar's exact Source byte.
pub(crate) struct TextRefView<'a> {
    document: &'a Document,
    id: u32,
    text: &'a str,
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TextRefRelation {
    Exact(Range<usize>),
    Enclosing(Range<usize>),
}
impl TextRefView<'_> {
    pub(crate) fn id(&self) -> u32 {
        self.id
    }
    pub(crate) fn semantic_text(&self) -> &str {
        self.text
    }
    pub(crate) fn relation(
        &self,
        local: Range<usize>,
        work: &mut DocumentWork<'_>,
    ) -> Result<TextRefRelation> {
        work.admit_structure(
            size_of::<crate::document_model::TextRef>() + size_of::<Origin>() + 2,
        )?;
        if local.start > local.end
            || local.end > self.text.len()
            || !self.text.is_char_boundary(local.start)
            || !self.text.is_char_boundary(local.end)
        {
            return Err(Error::Invalid);
        }
        let t = &self.document.text_refs()[self.id as usize];
        let o = &self.document.origins()[t.origin() as usize];
        if t.kind() == 0 {
            return Ok(TextRefRelation::Exact(
                (t.start() as usize)
                    .checked_add(local.start)
                    .ok_or(Error::Invalid)?
                    ..(t.start() as usize)
                        .checked_add(local.end)
                        .ok_or(Error::Invalid)?,
            ));
        }
        Ok(TextRefRelation::Enclosing(
            o.start() as usize..o.end() as usize,
        ))
    }
}
pub(crate) fn text_ref_view<'a>(
    document: &'a Document,
    id: u32,
    work: &mut DocumentWork<'_>,
) -> Result<TextRefView<'a>> {
    work.admit_structure(size_of::<crate::document_model::TextRef>())?;
    let text = document.text(id).map_err(|_| Error::Invalid)?;
    Ok(TextRefView { document, id, text })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::{Arc, atomic::AtomicBool},
        time::{Duration, Instant},
    };
    #[test]
    fn rl01_real_roles_and_resources() {
        let cancel = AtomicBool::new(false);
        let control = crate::document_obsidian::Control {
            cancel: &cancel,
            deadline: Instant::now() + Duration::from_secs(5),
        };
        let source=Arc::new(crate::document_source::DocumentSource::new(Arc::new("# H\n\n> [!note] TITLE\n> body\n\n- [x] checked\n\n| A | B |\n|:--|--:|\n| c | d |\n\n```rust\nlet a=1;\n```\n\n![ALT](x \"TITLE\")\n".to_owned()),crate::input::SourceOrigin::Literal,1,1,&cancel).unwrap());
        let c = crate::document_parser::project(
            source,
            crate::document_model::Profile::Extended,
            crate::document_budget::ActorArena::new(),
            &control,
        )
        .unwrap();
        let depth = crate::document_validate::validate(&c, &control).unwrap();
        let d = crate::document_model::Document::from_validated(
            crate::document_validate::quarantine(c, depth, &control).unwrap(),
        );
        let mut work = DocumentWork::frame(&cancel);
        let mut kinds = [false; 6];
        for b in d.blocks() {
            match block_role(&d, b.id(), &mut work).unwrap() {
                BlockRole::Heading { level: 1 } => kinds[0] = true,
                BlockRole::Callout {
                    title_domain: Some(_),
                    ..
                } => kinds[1] = true,
                BlockRole::Item {
                    task: TaskState::Checked,
                } => kinds[2] = true,
                BlockRole::Table { columns: 2 } => kinds[3] = true,
                BlockRole::Cell {
                    alignment: Alignment::Right,
                } => kinds[4] = true,
                BlockRole::Code {
                    fenced: true,
                    language: Some(_),
                    ..
                } => kinds[5] = true,
                _ => {}
            }
        }
        assert_eq!(kinds, [true; 6]);
        let r = resource_descriptor(&d, 0, &mut work).unwrap();
        assert_eq!(r.kind(), ResourceKind::Image);
        assert_eq!(r.state(), ResourceState::PendingRequired);
        assert!(r.alt_domain().is_some());
        let title = text_ref_view(&d, r.title_ref().unwrap(), &mut work).unwrap();
        assert_eq!(title.semantic_text(), "TITLE");
        assert!(matches!(
            title.relation(0..5, &mut work).unwrap(),
            TextRefRelation::Enclosing(_)
        ));
        let input = text_ref_view(&d, r.input_ref(), &mut work).unwrap();
        assert_eq!(input.semantic_text(), "x");
        assert!(work.used() > 0);
    }
    #[test]
    fn b2_backend_capacity_preflight() {
        use ratatui::{
            buffer::{Buffer, Cell},
            layout::Rect,
        };
        for (w, h) in [(80, 24), (120, 40)] {
            let current = Buffer::empty(Rect::new(0, 0, w, h));
            let candidate = Buffer::empty(Rect::new(0, 0, w, h));
            let bytes = (current.content.capacity() + candidate.content.capacity())
                * std::mem::size_of::<Cell>();
            println!(
                "B2_BACKEND_PREFLIGHT width={w} height={h} sizeofCell={} currentCapacity={} candidateCapacity={} twoGridRequested={bytes} frameworkLimit=262144 admitted=false measurementOnly=true",
                std::mem::size_of::<Cell>(),
                current.content.capacity(),
                candidate.content.capacity()
            );
            assert_eq!(current.content.len(), usize::from(w) * usize::from(h));
        }
    }
}
