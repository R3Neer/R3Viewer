//! Real pulldown OffsetIter producer; parser allocations remain external to the controlled arena.
use crate::{
    document_budget::{ActorArena, ArenaVec},
    document_model::*,
    document_obsidian::{Control, WikiKind, equal, expected},
    document_source::DocumentSource,
};
use pulldown_cmark::{
    Alignment, CodeBlockKind, Event, LinkType, MetadataBlockKind, Options, Parser, Tag, TagEnd,
};
use std::{ops::Range, sync::Arc};
fn invalid() -> Error {
    Error::new(ErrorKind::InvalidContext)
}
fn id(n: usize) -> Result<u32> {
    u32::try_from(n)
        .ok()
        .filter(|&n| n != NONE)
        .ok_or_else(|| Error::new(ErrorKind::RecordLimit))
}
#[derive(Clone, Copy)]
struct State {
    block: u32,
    domain: u32,
    styles: u32,
    link: u32,
    span: u32,
}
#[derive(Clone, Copy)]
struct Frame {
    end: TagEnd,
    prior: State,
    span: u32,
    block: u32,
    resource: u32,
    nested_alt: bool,
    nested_start: usize,
    nested_end: usize,
    table_first: usize,
    first_paragraph: u32,
    first_run: usize,
    last_run: usize,
    first_span: usize,
    last_span: usize,
    first_item: usize,
    last_item: usize,
    child_item: usize,
}
impl Frame {
    const EMPTY: Self = Self {
        end: TagEnd::Paragraph,
        prior: State {
            block: 0,
            domain: NONE,
            styles: 0,
            link: NONE,
            span: NONE,
        },
        span: NONE,
        block: NONE,
        resource: NONE,
        nested_alt: false,
        nested_start: 0,
        nested_end: 0,
        table_first: 0,
        first_paragraph: NONE,
        first_run: 0,
        last_run: 0,
        first_span: 0,
        last_span: 0,
        first_item: 0,
        last_item: 0,
        child_item: 0,
    };
}
struct Builder<'a> {
    c: Candidate,
    ctl: &'a Control<'a>,
    state: State,
    stack: [Frame; 65],
    depth: usize,
    alignments: ArenaVec<u32>,
    table_first: usize,
}
impl<'a> Builder<'a> {
    fn new(
        source: Arc<DocumentSource>,
        profile: Profile,
        arena: ActorArena,
        ctl: &'a Control<'a>,
    ) -> Result<Self> {
        let mut c = Candidate::empty(source, profile, arena.clone());
        let o = Origin::from_words([0, 0, id(c.source.len())?, 0]);
        c.origins.push(o)?;
        c.blocks.push(Block::from_words([
            0, NONE, 0, 0, NONE, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]))?;
        Ok(Self {
            c,
            ctl,
            state: Frame::EMPTY.prior,
            stack: [Frame::EMPTY; 65],
            depth: 0,
            alignments: ArenaVec::new(arena),
            table_first: 0,
        })
    }
    fn guard(&self) -> Result<()> {
        self.ctl.poll()?;
        if self.c.record_count() >= RECORD_LIMIT {
            Err(Error::new(ErrorKind::RecordLimit))
        } else {
            Ok(())
        }
    }
    fn origin(&mut self, range: Range<usize>, relation: u32) -> Result<u32> {
        self.guard()?;
        self.c
            .source
            .text()
            .get(range.clone())
            .ok_or_else(invalid)?;
        let n = id(self.c.origins.len())?;
        self.c.origins.push(Origin::from_words([
            relation,
            id(range.start)?,
            id(range.end)?,
            0,
        ]))?;
        Ok(n)
    }
    fn source_text(&mut self, range: Range<usize>) -> Result<u32> {
        let origin = self.origin(range.clone(), 0)?;
        self.guard()?;
        let n = id(self.c.texts.len())?;
        self.c.texts.push(TextRef::from_words([
            0,
            id(range.start)?,
            id(range.end)?,
            origin,
        ]))?;
        Ok(n)
    }
    fn text(&mut self, value: &str, range: Range<usize>, force_decoded: bool) -> Result<u32> {
        self.guard()?;
        let raw = self
            .c
            .source
            .text()
            .get(range.clone())
            .ok_or_else(invalid)?;
        if !force_decoded && equal(value.as_bytes(), raw.as_bytes(), self.ctl)? {
            return self.source_text(range);
        }
        let origin = self.origin(range, 1)?;
        let start = id(self.c.decoded.len())?;
        for chunk in value.as_bytes().chunks(4096) {
            self.ctl.poll()?;
            self.c.decoded.extend_from_slice(chunk)?;
        }
        let end = id(self.c.decoded.len())?;
        self.guard()?;
        let n = id(self.c.texts.len())?;
        self.c
            .texts
            .push(TextRef::from_words([1, start, end, origin]))?;
        Ok(n)
    }
    fn domain(&mut self, kind: u32, owner: u32) -> Result<u32> {
        self.guard()?;
        let n = id(self.c.domains.len())?;
        self.c
            .domains
            .push(Domain::from_words([n, kind, owner, 0, 0, 0]))?;
        Ok(n)
    }
    fn item(&mut self, kind: u32, target: u32) -> Result<()> {
        self.guard()?;
        let b = self.state.block as usize;
        let ordinal = self.c.blocks[b].0[3];
        self.c
            .items
            .push(Item::from_words([b as u32, ordinal, kind, target]))?;
        self.c.blocks[b].0[3] += 1;
        Ok(())
    }
    fn block(&mut self, kind: u32, aux: [u32; 4], range: Range<usize>) -> Result<u32> {
        let origin = self.origin(range, 0)?;
        self.guard()?;
        let n = id(self.c.blocks.len())?;
        self.item(0, n)?;
        let domain = if matches!(kind, 1 | 2 | 6 | 9 | 10 | 13 | 14) {
            self.domain(0, n)?
        } else {
            NONE
        };
        self.c.blocks.push(Block::from_words([
            n,
            self.state.block,
            0,
            0,
            domain,
            kind,
            0,
            aux[0],
            aux[1],
            aux[2],
            aux[3],
            origin,
            0,
            0,
            0,
            0,
        ]))?;
        self.state = State {
            block: n,
            domain,
            styles: 0,
            link: NONE,
            span: NONE,
        };
        Ok(n)
    }
    fn nested_token(&self) -> Option<Range<usize>> {
        self.stack[..self.depth]
            .iter()
            .rev()
            .find(|f| f.nested_alt)
            .map(|f| f.nested_start..f.nested_end)
    }
    fn run_text(&mut self, value: &str, range: Range<usize>) -> Result<u32> {
        if let Some(token) = self.nested_token() {
            self.text(value, token, true)
        } else {
            self.text(value, range, false)
        }
    }
    fn run(&mut self, kind: u32, text: u32, resource: u32, range: Range<usize>) -> Result<u32> {
        self.guard()?;
        if self.state.domain == NONE {
            return Err(invalid());
        }
        let origin = if let Some(token) = self.nested_token() {
            self.origin(token, 1)?
        } else {
            self.origin(range, 0)?
        };
        let n = id(self.c.runs.len())?;
        let d = self.state.domain as usize;
        let ordinal = self.c.domains[d].0[4];
        let styles = self.state.styles | if matches!(kind, 1 | 6) { 8 } else { 0 };
        self.c.runs.push(Run::from_words([
            n,
            d as u32,
            ordinal,
            kind,
            styles,
            text,
            origin,
            self.state.link,
            resource,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ]))?;
        self.c.domains[d].0[4] += 1;
        if self.c.domains[d].kind() == 0 {
            self.item(1, n)?;
        }
        Ok(n)
    }
    fn resource(
        &mut self,
        kind: u32,
        input: u32,
        raw: u32,
        title: u32,
        range: Range<usize>,
        intent: u32,
    ) -> Result<u32> {
        self.guard()?;
        let origin = self.origin(range, 0)?;
        let n = id(self.c.resources.len())?;
        self.c.resources.push(Resource::from_words([
            n, kind, 0, input, raw, title, origin, NONE, 0, 0, 0, 0, intent, NONE, 0, 0, 0, 0, 0, 0,
        ]))?;
        Ok(n)
    }
    fn begin_span(&mut self, kind: u32, link: u32, range: Range<usize>) -> Result<u32> {
        self.guard()?;
        if self.state.domain == NONE {
            return Err(invalid());
        }
        let origin = self.origin(range, 0)?;
        let n = id(self.c.spans.len())?;
        let ordinal = self.c.domains[self.state.domain as usize].run_count();
        self.c.spans.push(Span::from_words([
            n,
            self.state.domain,
            self.state.span,
            ordinal,
            ordinal,
            kind,
            link,
            origin,
            0,
            0,
        ]))?;
        self.state.span = n;
        Ok(n)
    }
    fn cell(&self, range: Range<usize>) -> Result<Option<Range<u32>>> {
        let mut b = self.state.block;
        for _ in 0..=64 {
            self.ctl.poll()?;
            let at = self.c.blocks.get(b as usize).ok_or_else(invalid)?;
            if at.kind() == 9 {
                let o = &self.c.origins[at.origin() as usize];
                if range.start < o.start() as usize || range.end > o.end() as usize {
                    return Err(invalid());
                }
                return Ok(Some(o.start()..o.end()));
            }
            if at.parent() == NONE {
                return Ok(None);
            }
            b = at.parent();
        }
        Err(Error::new(ErrorKind::DepthLimit))
    }

    /// Project a complete first paragraph only after all of its spans are known.
    fn callout(
        &mut self,
        quote: u32,
        paragraph: u32,
        runs: Range<usize>,
        spans: Range<usize>,
        items: Range<usize>,
        child_item: usize,
    ) -> Result<()> {
        if self.c.profile != Profile::Extended {
            return Ok(());
        }
        let q = self.c.blocks[quote as usize];
        let qo = self.c.origins[q.origin() as usize];
        let Some(marker) = marker(
            self.c.source.text(),
            qo.start() as usize..qo.end() as usize,
            self.ctl,
        )?
        else {
            return Ok(());
        };
        let domain = self.c.blocks[paragraph as usize].domain();
        let mut header_count = 0u32;
        let mut body_first = 0u32;
        let mut marker_count = 0u32;
        let mut body_count = 0u32;
        let mut title_count = 0u32;
        for run in self.c.runs[runs.clone()]
            .iter()
            .filter(|r| r.domain() == domain)
        {
            self.ctl.poll()?;
            let o = self.c.origins[run.origin() as usize];
            if matches!(run.kind(), 2 | 3)
                && o.start() as usize <= marker.line_end
                && o.end() as usize > marker.line_end
            {
                body_first = run.ordinal() + 1;
                continue;
            }
            if o.start() as usize >= marker.line_end {
                body_count += 1;
                continue;
            }
            header_count = run.ordinal() + 1;
            body_first = header_count;
            if o.end() as usize <= marker.title_start {
                marker_count += 1;
                continue;
            }
            if run.kind() == 5 {
                let res = self.c.resources[run.resource() as usize];
                if !matches!(res.kind(), 2 | 3) {
                    return Ok(());
                }
            }
            if (o.start() as usize) < marker.title_start && run.kind() != 0 {
                return Ok(());
            }
            title_count += 1;
        }
        let next_line_start = match self.c.source.text().as_bytes().get(marker.line_end) {
            Some(b'\n') => Some(marker.line_end + 1),
            Some(b'\r') => Some(
                marker.line_end
                    + if self.c.source.text().as_bytes().get(marker.line_end + 1) == Some(&b'\n') {
                        2
                    } else {
                        1
                    },
            ),
            _ => None,
        };
        let mut header_owner = false;
        let mut body_owner = false;
        // A crossing inline construct has no faithful single-domain representation; retain Quote.
        for span in self.c.spans[spans.clone()]
            .iter()
            .filter(|s| s.domain() == domain)
        {
            self.ctl.poll()?;
            let o = self.c.origins[span.origin() as usize];
            if (span.start_ordinal() < body_first && span.end_ordinal() > header_count)
                || (o.start() as usize) < marker.title_start
            {
                return Ok(());
            }
            // Empty ordinals are ambiguous when GFM consumes the header marker.
            let Some(header) = callout_span_header(span, o, &marker, next_line_start, header_count)
            else {
                return Ok(());
            };
            if header {
                span.start_ordinal()
                    .checked_sub(marker_count)
                    .ok_or_else(invalid)?;
                span.end_ordinal()
                    .checked_sub(marker_count)
                    .ok_or_else(invalid)?;
                header_owner = true;
            } else {
                span.start_ordinal()
                    .checked_sub(body_first)
                    .ok_or_else(invalid)?;
                span.end_ordinal()
                    .checked_sub(body_first)
                    .ok_or_else(invalid)?;
                body_owner = true;
            }
        }
        let title = if title_count > 0 || header_owner {
            self.domain(2, quote)?
        } else {
            NONE
        };
        let kind = self.source_text(marker.kind.clone())?;
        let marker_origin = self.origin(marker.marker.clone(), 0)?;
        self.c.blocks[quote as usize].0[5] = 4;
        self.c.blocks[quote as usize].0[7] = kind;
        self.c.blocks[quote as usize].0[8] = title;
        self.c.blocks[quote as usize].0[9] = marker.fold;
        self.c.blocks[quote as usize].0[10] = marker_origin;
        let mut title_ordinal = 0;
        let mut body_ordinal = 0;
        for index in runs.clone() {
            self.ctl.poll()?;
            let run = self.c.runs[index];
            if run.domain() != domain {
                continue;
            }
            let o = self.c.origins[run.origin() as usize];
            if matches!(run.kind(), 2 | 3)
                && o.start() as usize <= marker.line_end
                && o.end() as usize > marker.line_end
                || o.end() as usize <= marker.title_start
            {
                self.c.runs[index].0[3] = NONE;
                continue;
            }
            if o.start() as usize >= marker.line_end {
                self.c.runs[index].0[2] = body_ordinal;
                body_ordinal += 1;
                continue;
            }
            if (o.start() as usize) < marker.title_start {
                let old = self.c.texts[run.text() as usize];
                let prefix = marker.title_start - o.start() as usize;
                let cooked = self.c.text(run.text())?;
                let literal = self
                    .c
                    .source
                    .text()
                    .get(o.start() as usize..marker.title_start)
                    .ok_or_else(invalid)?;
                if cooked.get(..prefix) != Some(literal) {
                    return Err(invalid());
                }
                let origin = self.origin(
                    marker.title_start..o.end() as usize,
                    if old.kind() == 0 { 0 } else { 1 },
                )?;
                self.guard()?;
                let text = id(self.c.texts.len())?;
                self.c.texts.push(TextRef::from_words([
                    old.kind(),
                    old.start().checked_add(prefix as u32).ok_or_else(invalid)?,
                    old.end(),
                    origin,
                ]))?;
                self.c.runs[index].0[5] = text;
                self.c.runs[index].0[6] = origin;
            }
            if run.kind() == 5 {
                let res = self.c.resources[run.resource() as usize];
                self.c.runs[index].0[3] = 0;
                self.c.runs[index].0[5] = res.raw_token();
                self.c.runs[index].0[8] = NONE;
                self.c.resources[run.resource() as usize].0[1] = NONE;
            }
            self.c.runs[index].0[1] = title;
            self.c.runs[index].0[2] = title_ordinal;
            title_ordinal += 1;
        }
        if title != NONE {
            self.c.domains[title as usize].0[4] = title_ordinal;
        }
        self.c.domains[domain as usize].0[4] = body_ordinal;
        for span in self.c.spans[spans]
            .iter_mut()
            .filter(|s| s.domain() == domain)
        {
            self.ctl.poll()?;
            let o = self.c.origins[span.origin() as usize];
            if callout_span_header(span, o, &marker, next_line_start, header_count)
                .ok_or_else(invalid)?
            {
                span.0[1] = title;
                span.0[3] = span
                    .start_ordinal()
                    .checked_sub(marker_count)
                    .ok_or_else(invalid)?;
                span.0[4] = span
                    .end_ordinal()
                    .checked_sub(marker_count)
                    .ok_or_else(invalid)?;
            } else {
                span.0[3] = span
                    .start_ordinal()
                    .checked_sub(body_first)
                    .ok_or_else(invalid)?;
                span.0[4] = span
                    .end_ordinal()
                    .checked_sub(body_first)
                    .ok_or_else(invalid)?;
            }
            if span.link() != NONE {
                let link = &mut self.c.links[span.link() as usize];
                link.0[7] = span.domain();
                link.0[8] = span.start_ordinal();
                link.0[9] = span.end_ordinal() - span.start_ordinal();
            }
        }
        let mut ordinal = 0;
        for item in self.c.items[items]
            .iter_mut()
            .filter(|i| i.parent_block() == paragraph)
        {
            self.ctl.poll()?;
            let run = self
                .c
                .runs
                .get(item.target() as usize)
                .ok_or_else(invalid)?;
            if item.kind() != 1 {
                return Err(invalid());
            }
            if run.kind() == NONE || run.domain() != domain {
                item.0[2] = NONE;
            } else {
                item.0[1] = ordinal;
                ordinal += 1;
            }
        }
        self.c.blocks[paragraph as usize].0[3] = ordinal;
        if body_count == 0 && !body_owner {
            self.c.blocks[paragraph as usize].0[5] = NONE;
            self.c.domains[domain as usize].0[1] = NONE;
            self.c.items[child_item].0[2] = NONE;
        } else if body_count > 0 {
            let old = self.c.origins[self.c.blocks[paragraph as usize].origin() as usize];
            let start = self.c.runs[runs]
                .iter()
                .find(|r| r.domain() == domain && r.kind() != NONE)
                .map(|r| self.c.origins[r.origin() as usize].start())
                .ok_or_else(invalid)?;
            let o = self.origin(start as usize..old.end() as usize, 0)?;
            self.c.blocks[paragraph as usize].0[11] = o;
        }
        Ok(())
    }
    fn compact(&mut self, remap: &mut [u32]) -> Result<()> {
        // One reusable remap, no duplicate record table; IDs and ownership updated before reuse.
        let ctl = self.ctl;
        fn table<T: Copy>(
            records: &mut ArenaVec<T>,
            remap: &mut [u32],
            live: fn(&T) -> bool,
            ctl: &Control<'_>,
        ) -> Result<()> {
            let mut new = 0;
            for old in 0..records.len() {
                ctl.poll()?;
                if live(&records[old]) {
                    remap[old] = new as u32;
                    records[new] = records[old];
                    new += 1;
                } else {
                    remap[old] = NONE;
                }
            }
            records.truncate(new);
            Ok(())
        }
        table(&mut self.c.blocks, remap, |b| b.kind() != NONE, ctl)?;
        for (i, b) in self.c.blocks.iter_mut().enumerate() {
            ctl.poll()?;
            b.0[0] = i as u32;
            if b.parent() != NONE {
                b.0[1] = remap[b.parent() as usize];
            }
        }
        for d in self.c.domains.iter_mut() {
            ctl.poll()?;
            if matches!(d.kind(), 0 | 2) {
                d.0[2] = remap[d.owner() as usize];
            }
        }
        for i in self.c.items.iter_mut() {
            ctl.poll()?;
            if i.kind() != NONE {
                i.0[0] = remap[i.parent_block() as usize];
                if i.kind() == 0 {
                    i.0[3] = remap[i.target() as usize];
                }
            }
        }
        table(&mut self.c.domains, remap, |d| d.kind() != NONE, ctl)?;
        for (i, d) in self.c.domains.iter_mut().enumerate() {
            ctl.poll()?;
            d.0[0] = i as u32;
        }
        for b in self.c.blocks.iter_mut() {
            ctl.poll()?;
            if b.domain() != NONE {
                b.0[4] = remap[b.domain() as usize];
            }
            if b.kind() == 4 && b.aux1() != NONE {
                b.0[8] = remap[b.aux1() as usize];
            }
        }
        for r in self.c.runs.iter_mut() {
            ctl.poll()?;
            if r.kind() != NONE {
                r.0[1] = remap[r.domain() as usize];
            }
        }
        for s in self.c.spans.iter_mut() {
            ctl.poll()?;
            s.0[1] = *remap.get(s.domain() as usize).ok_or_else(invalid)?;
            if s.domain() == NONE {
                return Err(invalid());
            }
        }
        for l in self.c.links.iter_mut() {
            ctl.poll()?;
            l.0[7] = remap[l.label_domain() as usize];
        }
        for r in self.c.resources.iter_mut() {
            ctl.poll()?;
            if r.alt_domain() != NONE {
                r.0[7] = remap[r.alt_domain() as usize];
            }
        }
        table(&mut self.c.resources, remap, |r| r.kind() != NONE, ctl)?;
        for (i, r) in self.c.resources.iter_mut().enumerate() {
            ctl.poll()?;
            r.0[0] = i as u32;
        }
        for r in self.c.runs.iter_mut() {
            ctl.poll()?;
            if r.resource() != NONE {
                r.0[8] = remap[r.resource() as usize];
            }
        }
        for b in self.c.blocks.iter_mut() {
            ctl.poll()?;
            if b.kind() == 10 && b.aux3() != NONE {
                b.0[10] = remap[b.aux3() as usize];
            }
        }
        for d in self.c.domains.iter_mut() {
            ctl.poll()?;
            if d.kind() == 1 {
                d.0[2] = remap[d.owner() as usize];
            }
        }
        table(&mut self.c.runs, remap, |r| r.kind() != NONE, ctl)?;
        for (i, r) in self.c.runs.iter_mut().enumerate() {
            ctl.poll()?;
            r.0[0] = i as u32;
        }
        for i in self.c.items.iter_mut() {
            ctl.poll()?;
            if i.kind() == 1 {
                i.0[3] = remap[i.target() as usize];
            }
        }
        for r in self.c.resources.iter_mut() {
            ctl.poll()?;
            if r.owner_run() != NONE {
                r.0[13] = remap[r.owner_run() as usize];
            }
        }
        table(&mut self.c.items, remap, |i| i.kind() != NONE, ctl)?;
        // Deleting a header-only Paragraph changes sibling ordinals, never source order.
        for b in self.c.blocks.iter_mut() {
            ctl.poll()?;
            b.0[3] = 0;
        }
        for i in self.c.items.iter_mut() {
            ctl.poll()?;
            let b = &mut self.c.blocks[i.parent_block() as usize];
            i.0[1] = b.item_count();
            b.0[3] += 1;
        }
        Ok(())
    }
    fn finish(mut self) -> Result<Candidate> {
        if self.depth != 0 {
            return Err(invalid());
        }
        self.ctl.poll()?;
        let mut remap = ArenaVec::<u32>::new(self.c.arena.clone());
        let size = [
            self.c.blocks.len(),
            self.c.domains.len(),
            self.c.items.len(),
            self.c.runs.len(),
            self.c.spans.len(),
            self.c.resources.len(),
        ]
        .into_iter()
        .max()
        .unwrap_or(0);
        remap.reserve_exact(size)?;
        while remap.len() < size {
            let n = (size - remap.len()).min(1024);
            remap.extend_from_slice(&[0; 1024][..n])?;
            self.ctl.poll()?;
        }
        self.compact(&mut remap)?;
        // Key includes creation id, so unstable_sort preserves event order without allocation.
        self.c
            .runs
            .sort_unstable_by_key(|run| (run.domain(), run.id()));
        for (new, run) in self.c.runs.iter_mut().enumerate() {
            self.ctl.poll()?;
            remap[run.id() as usize] = new as u32;
            run.0[0] = new as u32;
        }
        for item in self.c.items.iter_mut() {
            self.ctl.poll()?;
            if item.kind() == 1 {
                item.0[3] = remap[item.target() as usize];
            }
        }
        for resource in self.c.resources.iter_mut() {
            self.ctl.poll()?;
            if resource.owner_run() != NONE {
                resource.0[13] = remap[resource.owner_run() as usize];
            }
        }
        let mut first = 0;
        for domain in self.c.domains.iter_mut() {
            self.ctl.poll()?;
            domain.0[3] = first;
            first += domain.run_count();
        }
        self.c
            .items
            .sort_unstable_by_key(|item| (item.parent_block(), item.ordinal()));
        let mut first = 0;
        for block in self.c.blocks.iter_mut() {
            self.ctl.poll()?;
            block.0[2] = first;
            first += block.item_count();
        }
        // Cache original parent depth in remap; sort keys never chase a rearranged parent table.
        for span in self.c.spans.iter() {
            self.ctl.poll()?;
            let mut at = span.parent_span();
            let mut depth = 1;
            while at != NONE {
                depth += 1;
                if depth > 64 {
                    return Err(Error::new(ErrorKind::DepthLimit));
                }
                at = self
                    .c
                    .spans
                    .get(at as usize)
                    .ok_or_else(invalid)?
                    .parent_span();
            }
            remap[span.id() as usize] = depth;
        }
        self.c.spans.sort_unstable_by_key(|span| {
            let o = &self.c.origins[span.origin() as usize];
            (
                span.domain(),
                span.start_ordinal(),
                u32::MAX - span.end_ordinal(),
                remap[span.id() as usize],
                o.start(),
                o.end(),
                span.id(),
            )
        });
        for (new, span) in self.c.spans.iter().enumerate() {
            self.ctl.poll()?;
            remap[span.id() as usize] = new as u32;
        }
        for (new, span) in self.c.spans.iter_mut().enumerate() {
            self.ctl.poll()?;
            span.0[0] = new as u32;
            if span.parent_span() != NONE {
                span.0[2] = remap[span.parent_span() as usize];
            }
        }
        drop(remap);
        drop(self.alignments);
        self.c.max_depth = crate::document_validate::validate(&self.c, self.ctl)?;
        Ok(self.c)
    }
}

pub(crate) struct Marker {
    pub(crate) kind: Range<usize>,
    pub(crate) marker: Range<usize>,
    pub(crate) fold: u32,
    title_start: usize,
    line_end: usize,
}
fn callout_span_header(
    span: &Span,
    origin: Origin,
    marker: &Marker,
    next_line_start: Option<usize>,
    header_count: u32,
) -> Option<bool> {
    if span.start_ordinal() < span.end_ordinal() {
        return Some(span.end_ordinal() <= header_count);
    }
    if origin.start() as usize >= marker.title_start && origin.end() as usize <= marker.line_end {
        Some(true)
    } else if next_line_start.is_some_and(|start| origin.start() as usize >= start) {
        Some(false)
    } else {
        None
    }
}
pub(crate) fn marker(
    source: &str,
    range: Range<usize>,
    ctl: &Control<'_>,
) -> Result<Option<Marker>> {
    ctl.poll()?;
    let base = range.start;
    let bytes = source.get(range).ok_or_else(invalid)?.as_bytes();
    let mut at = 0;
    for _ in 0..3 {
        if bytes.get(at) == Some(&b' ') {
            at += 1;
        }
    }
    if bytes.get(at) != Some(&b'>') {
        return Ok(None);
    }
    at += 1;
    if matches!(bytes.get(at), Some(b' ' | b'\t')) {
        at += 1;
    }
    if bytes.get(at..at + 2) != Some(b"[!") {
        return Ok(None);
    }
    let start = at;
    at += 2;
    let kind_start = at;
    while let Some(&c) = bytes.get(at) {
        if at.is_multiple_of(1024) {
            ctl.poll()?;
        }
        if c == b']' {
            break;
        }
        if !c.is_ascii_alphanumeric() && c != b'-' && c != b'_' {
            return Ok(None);
        }
        at += 1;
    }
    if at == kind_start || bytes.get(at) != Some(&b']') {
        return Ok(None);
    }
    let kind = kind_start..at;
    at += 1;
    let fold = match bytes.get(at) {
        Some(b'+') => {
            at += 1;
            1
        }
        Some(b'-') => {
            at += 1;
            2
        }
        _ => 0,
    };
    let token = start..at;
    while matches!(bytes.get(at), Some(b' ' | b'\t')) {
        if at.is_multiple_of(1024) {
            ctl.poll()?;
        }
        at += 1;
    }
    let title_start = at;
    let mut line_end = at;
    for (i, chunk) in bytes[at..].chunks(1024).enumerate() {
        ctl.poll()?;
        if let Some(n) = chunk.iter().position(|&c| c == b'\n' || c == b'\r') {
            line_end = at + i * 1024 + n;
            break;
        }
        line_end = at + i * 1024 + chunk.len();
    }
    Ok(Some(Marker {
        kind: base + kind.start..base + kind.end,
        marker: base + token.start..base + token.end,
        fold,
        title_start: base + title_start,
        line_end: base + line_end,
    }))
}
fn link_kind(kind: LinkType) -> (u32, u32) {
    match kind {
        LinkType::Inline => (0, 0),
        LinkType::Reference => (1, 0),
        LinkType::ReferenceUnknown => (2, 0),
        LinkType::Collapsed => (3, 0),
        LinkType::CollapsedUnknown => (4, 0),
        LinkType::Shortcut => (5, 0),
        LinkType::ShortcutUnknown => (6, 0),
        LinkType::Autolink => (7, 0),
        LinkType::Email => (8, 0),
        LinkType::WikiLink { has_pothole } => (9, u32::from(has_pothole)),
    }
}
pub(crate) fn project(
    source: Arc<DocumentSource>,
    profile: Profile,
    arena: ActorArena,
    ctl: &Control<'_>,
) -> Result<Candidate> {
    ctl.poll()?;
    let options = Options::from_bits(profile.option_bits()).ok_or_else(invalid)?;
    let owner = source.clone();
    let mut iter = Parser::new_ext(owner.text(), options).into_offset_iter();
    let mut b = Builder::new(source, profile, arena, ctl)?;
    while let Some((event, range)) = iter.next() {
        ctl.poll()?;
        b.c.source.text().get(range.clone()).ok_or_else(invalid)?;
        let prior_inline = b.state;
        let automatic = b.state.domain == NONE
            && matches!(&event, Event::DisplayMath(_) | Event::InlineMath(_));
        if automatic {
            b.block(1, [0; 4], range.clone())?;
        }
        match event {
            Event::Start(tag) => {
                if b.depth == 64 {
                    return Err(Error::new(ErrorKind::DepthLimit));
                }
                let mut frame = Frame {
                    end: tag.to_end(),
                    prior: b.state,
                    span: NONE,
                    block: NONE,
                    resource: NONE,
                    nested_alt: false,
                    nested_start: 0,
                    nested_end: 0,
                    table_first: b.table_first,
                    first_paragraph: NONE,
                    first_run: 0,
                    last_run: 0,
                    first_span: 0,
                    last_span: 0,
                    first_item: 0,
                    last_item: 0,
                    child_item: 0,
                };
                match tag {
                    Tag::Paragraph => {
                        if b.c.blocks[b.state.block as usize].kind() == 6 {
                            let list = b.c.blocks[b.state.block as usize].parent();
                            b.c.blocks[list as usize].0[9] = 0;
                        }
                        let parent = b.state.block;
                        let first = b.c.blocks[parent as usize].item_count() == 0;
                        frame.block = b.block(1, [0; 4], range.clone())?;
                        if first && b.c.blocks[parent as usize].kind() == 3 {
                            for active in b.stack[..b.depth].iter_mut().rev() {
                                if active.block == parent {
                                    active.first_paragraph = frame.block;
                                    active.first_run = b.c.runs.len();
                                    active.first_span = b.c.spans.len();
                                    active.first_item = b.c.items.len();
                                    active.child_item = b.c.items.len() - 1;
                                    break;
                                }
                            }
                        }
                    }
                    Tag::Heading {
                        level,
                        id,
                        classes,
                        attrs,
                    } => {
                        if id.is_some() || !classes.is_empty() || !attrs.is_empty() {
                            return Err(invalid());
                        }
                        frame.block = b.block(2, [level as u32, NONE, 0, 0], range.clone())?;
                    }
                    Tag::BlockQuote(_) => {
                        frame.block = b.block(3, [0; 4], range.clone())?;
                    }
                    Tag::List(start) => {
                        let first = start
                            .map(u32::try_from)
                            .transpose()
                            .map_err(|_| invalid())?
                            .unwrap_or(0);
                        if first == NONE {
                            return Err(invalid());
                        }
                        frame.block =
                            b.block(5, [u32::from(start.is_some()), first, 1, 0], range.clone())?;
                    }
                    Tag::Item => {
                        frame.block = b.block(6, [0; 4], range.clone())?;
                    }
                    Tag::Table(align) => {
                        b.table_first = b.alignments.len();
                        for a in align.iter() {
                            ctl.poll()?;
                            b.alignments.push(match a {
                                Alignment::None => 0,
                                Alignment::Left => 1,
                                Alignment::Center => 2,
                                Alignment::Right => 3,
                            })?;
                        }
                        frame.block = b.block(7, [id(align.len())?, 0, 0, 0], range.clone())?;
                    }
                    Tag::TableHead => {
                        frame.block = b.block(8, [1, 0, 0, 0], range.clone())?;
                    }
                    Tag::TableRow => {
                        frame.block = b.block(8, [0; 4], range.clone())?;
                    }
                    Tag::TableCell => {
                        let ordinal = b.c.blocks[b.state.block as usize].item_count() as usize;
                        let a = *b
                            .alignments
                            .get(b.table_first + ordinal)
                            .ok_or_else(invalid)?;
                        frame.block = b.block(9, [a, 0, 0, 0], range.clone())?;
                    }
                    Tag::CodeBlock(kind) => {
                        let (style, info, language) = match &kind {
                            CodeBlockKind::Indented => (0, NONE, NONE),
                            CodeBlockKind::Fenced(info) => {
                                let info_id = b.text(info.as_ref(), range.clone(), true)?;
                                let lang = info.split_whitespace().next().unwrap_or("");
                                let lang_id = if lang.is_empty() {
                                    NONE
                                } else {
                                    b.text(lang, range.clone(), true)?
                                };
                                (1, info_id, lang_id)
                            }
                        };
                        frame.block = b.block(10, [style, info, language, NONE], range.clone())?;
                        if let CodeBlockKind::Fenced(info) = kind
                            && info.split_whitespace().next() == Some("mermaid")
                        {
                            let raw = b.source_text(range.clone())?;
                            let empty = b.text("", range.clone(), true)?;
                            let res = b.resource(1, empty, raw, NONE, range.clone(), 1)?;
                            b.c.blocks[frame.block as usize].0[10] = res;
                            frame.resource = res;
                        }
                    }
                    Tag::HtmlBlock => {
                        frame.block = b.block(13, [0; 4], range.clone())?;
                    }
                    Tag::FootnoteDefinition(label) => {
                        let label = b.text(label.as_ref(), range.clone(), true)?;
                        frame.block = b.block(12, [label, 0, 0, 0], range.clone())?;
                    }
                    Tag::MetadataBlock(MetadataBlockKind::YamlStyle) => {
                        frame.block = b.block(14, [0; 4], range.clone())?;
                    }
                    Tag::Emphasis | Tag::Strong | Tag::Strikethrough => {
                        let (kind, bit) = match tag {
                            Tag::Emphasis => (0, 1),
                            Tag::Strong => (1, 2),
                            _ => (2, 4),
                        };
                        frame.span = b.begin_span(kind, NONE, range.clone())?;
                        b.state.styles |= bit;
                    }
                    Tag::Link {
                        link_type,
                        dest_url,
                        title,
                        id: label,
                    } => {
                        let (kind, flags) = link_kind(link_type);
                        let cell = b.cell(range.clone())?;
                        let mut target = dest_url.as_ref();
                        let mut adapted = false;
                        if kind == 9 {
                            let result = expected(
                                owner.text(),
                                id(range.start)?..id(range.end)?,
                                profile,
                                WikiKind::Link {
                                    pothole: flags == 1,
                                },
                                cell,
                                Some(target),
                                ctl,
                            )?;
                            target = result.value;
                            adapted = result.adapted || result.first_body_loss;
                        }
                        let target_range = if matches!(kind, 1 | 3 | 5) {
                            iter.reference_definitions()
                                .get(label.as_ref())
                                .map(|def| def.span.clone())
                                .ok_or_else(invalid)?
                        } else {
                            range.clone()
                        };
                        let target_id = b.text(
                            target,
                            target_range.clone(),
                            adapted || target_range != range,
                        )?;
                        let target_origin = if target_range != range {
                            b.c.texts[target_id as usize].origin()
                        } else {
                            NONE
                        };
                        let title_id = if title.is_empty() {
                            NONE
                        } else {
                            b.text(title.as_ref(), target_range.clone(), true)?
                        };
                        let label_id = if label.is_empty() {
                            NONE
                        } else {
                            b.text(label.as_ref(), range.clone(), true)?
                        };
                        let raw = b.source_text(range.clone())?;
                        let origin = b.origin(range.clone(), 0)?;
                        let link = id(b.c.links.len())?;
                        b.guard()?;
                        let start = b.c.domains[b.state.domain as usize].run_count();
                        b.c.links.push(Link::from_words([
                            link,
                            kind,
                            flags,
                            target_id,
                            raw,
                            title_id,
                            label_id,
                            b.state.domain,
                            start,
                            0,
                            origin,
                            target_origin,
                            0,
                            0,
                            0,
                            0,
                        ]))?;
                        frame.span = b.begin_span(3, link, range.clone())?;
                        b.state.link = link;
                    }
                    Tag::Image {
                        link_type,
                        dest_url,
                        title,
                        ..
                    } => {
                        if b.c.domains[b.state.domain as usize].kind() != 0 {
                            frame.nested_alt = true;
                            frame.nested_start = range.start;
                            frame.nested_end = range.end;
                        } else {
                            let wiki = matches!(link_type, LinkType::WikiLink { .. });
                            let mut target = dest_url.as_ref();
                            let mut adapted = false;
                            if wiki {
                                let result = expected(
                                    owner.text(),
                                    id(range.start)?..id(range.end)?,
                                    profile,
                                    WikiKind::Embed,
                                    b.cell(range.clone())?,
                                    Some(target),
                                    ctl,
                                )?;
                                target = result.value;
                                adapted = result.adapted || result.first_body_loss;
                            }
                            let target = b.text(target, range.clone(), adapted)?;
                            let raw = b.source_text(range.clone())?;
                            let title = if title.is_empty() {
                                NONE
                            } else {
                                b.text(title.as_ref(), range.clone(), true)?
                            };
                            let res = b.resource(
                                if wiki { 4 } else { 0 },
                                target,
                                raw,
                                title,
                                range.clone(),
                                0,
                            )?;
                            let run = b.run(5, NONE, res, range.clone())?;
                            b.c.resources[res as usize].0[13] = run;
                            let alt = b.domain(1, res)?;
                            b.c.resources[res as usize].0[7] = alt;
                            frame.resource = res;
                            b.state.domain = alt;
                            b.state.styles = 0;
                            b.state.link = NONE;
                            b.state.span = NONE;
                        }
                    }
                    _ => return Err(invalid()),
                }
                b.stack[b.depth] = frame;
                b.depth += 1;
            }
            Event::End(end) => {
                if b.depth == 0 {
                    return Err(invalid());
                }
                b.depth -= 1;
                let frame = b.stack[b.depth];
                if frame.end != end {
                    return Err(invalid());
                }
                if frame.span != NONE {
                    let ordinal = b.c.domains[b.state.domain as usize].run_count();
                    b.c.spans[frame.span as usize].0[4] = ordinal;
                    let link = b.c.spans[frame.span as usize].link();
                    if link != NONE {
                        let start = b.c.links[link as usize].label_start();
                        b.c.links[link as usize].0[9] = ordinal - start;
                    }
                }
                if frame.resource != NONE {
                    let res = frame.resource as usize;
                    if b.c.resources[res].kind() == 1 {
                        let domain = b.c.blocks[frame.block as usize].domain();
                        let start = b.c.decoded.len();
                        for run in b.c.runs.iter().filter(|run| run.domain() == domain) {
                            let text = b.c.texts[run.text() as usize];
                            match text.kind() {
                                0 => {
                                    for chunk in owner.text().as_bytes()
                                        [text.start() as usize..text.end() as usize]
                                        .chunks(4096)
                                    {
                                        ctl.poll()?;
                                        b.c.decoded.extend_from_slice(chunk)?;
                                    }
                                }
                                1 => {
                                    let range = text.start() as usize..text.end() as usize;
                                    b.c.decoded.reserve_exact(b.c.decoded.len() + range.len())?;
                                    for at in range {
                                        if at.is_multiple_of(1024) {
                                            ctl.poll()?;
                                        }
                                        let byte = b.c.decoded[at];
                                        b.c.decoded.push(byte)?;
                                    }
                                }
                                _ => return Err(invalid()),
                            }
                        }
                        let origin = b.c.resources[res].origin();
                        let end = id(b.c.decoded.len())?;
                        let text = id(b.c.texts.len())?;
                        b.guard()?;
                        b.c.texts
                            .push(TextRef::from_words([1, id(start)?, end, origin]))?;
                        b.c.resources[res].0[3] = text;
                    } else if b.c.resources[res].kind() == 4 {
                        let t = &b.c.texts[b.c.resources[res].raw_token() as usize];
                        let token = owner
                            .text()
                            .get(t.start() as usize..t.end() as usize)
                            .ok_or_else(invalid)?;
                        let (w, h) = crate::document_obsidian::image_dimensions(token, ctl)?;
                        b.c.resources[res].0[8] = u32::from(w != 0);
                        b.c.resources[res].0[9] = w;
                        b.c.resources[res].0[10] = u32::from(h != 0);
                        b.c.resources[res].0[11] = h;
                    }
                }

                if frame.end == TagEnd::Paragraph {
                    for active in b.stack[..b.depth].iter_mut().rev() {
                        if active.first_paragraph == frame.block {
                            active.last_run = b.c.runs.len();
                            active.last_span = b.c.spans.len();
                            active.last_item = b.c.items.len();
                            break;
                        }
                    }
                }
                if frame.block != NONE
                    && b.c.blocks[frame.block as usize].kind() == 3
                    && frame.first_paragraph != NONE
                {
                    b.callout(
                        frame.block,
                        frame.first_paragraph,
                        frame.first_run..frame.last_run,
                        frame.first_span..frame.last_span,
                        frame.first_item..frame.last_item,
                        frame.child_item,
                    )?;
                }
                b.state = frame.prior;
                b.table_first = frame.table_first;
            }
            Event::Text(value) | Event::Html(value) => {
                let kind = match b.c.blocks[b.state.block as usize].kind() {
                    10 | 14 => 6,
                    13 => 7,
                    _ => 0,
                };
                let text = b.run_text(value.as_ref(), range.clone())?;
                b.run(kind, text, NONE, range)?;
            }
            Event::InlineHtml(_) => {
                let text = b.source_text(range.clone())?;
                b.run(0, text, NONE, range)?;
            }
            Event::Code(value) => {
                let text = b.run_text(value.as_ref(), range.clone())?;
                b.run(1, text, NONE, range)?;
            }
            Event::InlineMath(ref value) | Event::DisplayMath(ref value) => {
                let block = matches!(event, Event::DisplayMath(_));
                if b.c.domains[b.state.domain as usize].kind() != 0 {
                    let text = b.source_text(range.clone())?;
                    b.run(0, text, NONE, range)?;
                } else {
                    let input = b.text(value.as_ref(), range.clone(), false)?;
                    let raw = b.source_text(range.clone())?;
                    let res = b.resource(
                        if block { 3 } else { 2 },
                        input,
                        raw,
                        NONE,
                        range.clone(),
                        u32::from(block),
                    )?;
                    let run = b.run(5, NONE, res, range)?;
                    b.c.resources[res as usize].0[13] = run;
                }
            }
            Event::SoftBreak => {
                b.run(2, NONE, NONE, range)?;
            }
            Event::HardBreak => {
                b.run(3, NONE, NONE, range)?;
            }
            Event::FootnoteReference(value) => {
                let text = b.run_text(value.as_ref(), range.clone())?;
                b.run(4, text, NONE, range)?;
            }
            Event::TaskListMarker(checked) => {
                let mut block = b.state.block;
                loop {
                    let here = b.c.blocks.get(block as usize).ok_or_else(invalid)?;
                    if here.kind() == 6 {
                        b.c.blocks[block as usize].0[7] = if checked { 2 } else { 1 };
                        break;
                    }
                    if here.parent() == NONE {
                        return Err(invalid());
                    }
                    block = here.parent();
                }
            }
            Event::Rule => {
                let prior = b.state;
                b.block(11, [0; 4], range)?;
                b.state = prior;
            }
        }
        if automatic {
            b.state = prior_inline;
        }
    }
    b.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::SourceOrigin;
    use std::{
        sync::atomic::AtomicBool,
        time::{Duration, Instant},
    };
    fn parsed(s: &str) -> Candidate {
        let cancelled = AtomicBool::new(false);
        let ctl = Control {
            cancel: &cancelled,
            deadline: Instant::now() + Duration::from_secs(10),
        };
        let source = Arc::new(
            DocumentSource::new(
                Arc::new(s.to_owned()),
                SourceOrigin::Literal,
                1,
                1,
                &cancelled,
            )
            .unwrap(),
        );
        project(source, Profile::Extended, ActorArena::new(), &ctl).unwrap()
    }
    #[test]
    fn actual_records_preserve_semantic_types_and_raw_source() {
        for (source, kind) in [
            ("# Heading\n", 2),
            ("text\n", 1),
            ("> quote\n", 3),
            ("- [x] done\n", 5),
            ("| A | B |\n|:-|--:|\n| c | d |\n", 7),
            ("```rust\nlet x=1;\n```\n", 10),
            ("---\na: b\n---\n", 14),
            ("<div>x</div>\n", 13),
        ] {
            let c = parsed(source);
            assert_eq!(c.source.text(), source);
            assert!(c.blocks.iter().any(|b| b.kind() == kind));
        }
    }
    #[test]
    fn real_table_wiki_adapts_only_unique_pipe_escape() {
        let source = "| A |\n|---|\n| [[Nota\\|**Alias**]] |\n";
        let c = parsed(source);
        let link = c.links.iter().find(|l| l.kind() == 9).unwrap();
        assert_eq!(c.text(link.target_decoded()).unwrap(), "Nota");
        assert_eq!(c.text(link.raw_token()).unwrap(), "[[Nota\\|**Alias**]]");
        assert!(c.spans.iter().any(|s| s.kind() == 1));
        let target = &c.texts[link.target_decoded() as usize];
        let o = &c.origins[target.origin() as usize];
        assert_eq!(o.relation(), 1);
        assert_eq!(o.start(), c.texts[link.raw_token() as usize].start());
        let c = parsed("[[Nota\\|Alias]]\n");
        assert_eq!(c.text(c.links[0].target_decoded()).unwrap(), "Nota\\");
    }
    #[test]
    fn resources_keep_separate_alt_and_principal_payload() {
        let c = parsed("![**cat**](cat.png) $x^2$\n\n```mermaid\ngraph TD; A-->B\n```\n");
        assert!(
            c.resources
                .iter()
                .any(|r| r.kind() == 0 && r.alt_domain() != NONE)
        );
        assert!(c.resources.iter().any(|r| r.kind() == 2));
        let mermaid = c.resources.iter().find(|r| r.kind() == 1).unwrap();
        assert_eq!(
            c.text(mermaid.input_decoded()).unwrap(),
            "graph TD; A-->B\n"
        );
        assert_eq!(mermaid.owner_run(), NONE);
        assert!(
            c.blocks
                .iter()
                .any(|b| b.kind() == 10 && b.aux3() == mermaid.id())
        );
    }
    #[test]
    fn reference_target_origin_points_to_definition_not_usage() {
        let c = parsed("[label][r]\n\n[r]: dest \"Title\"\n");
        let l = &c.links[0];
        assert_eq!(c.text(l.target_decoded()).unwrap(), "dest");
        assert_ne!(l.target_origin(), NONE);
        let o = &c.origins[l.target_origin() as usize];
        assert!(o.start() > c.texts[l.raw_token() as usize].end());
    }
    #[test]
    fn reject_mutant_ownership_and_link_label_without_publishing() {
        let mut c = parsed("[x](dest)\n");
        let cancelled = AtomicBool::new(false);
        let ctl = Control {
            cancel: &cancelled,
            deadline: Instant::now() + Duration::from_secs(10),
        };
        c.links[0].0[9] += 1;
        assert_eq!(
            crate::document_validate::validate(&c, &ctl)
                .unwrap_err()
                .kind,
            ErrorKind::InvalidContext
        );
    }
}

#[cfg(test)]
mod callout_tests {
    use super::*;
    use crate::input::SourceOrigin;
    use std::{
        sync::atomic::AtomicBool,
        time::{Duration, Instant},
    };
    #[test]
    fn source_defined_custom_title_fold_and_body_are_separate_domains() {
        let cancel = AtomicBool::new(false);
        let ctl = Control {
            cancel: &cancel,
            deadline: Instant::now() + Duration::from_secs(10),
        };
        let original = "> [!custom]- **Title** $x$\n> Body\n";
        let source = Arc::new(
            DocumentSource::new(
                Arc::new(original.to_owned()),
                SourceOrigin::Literal,
                1,
                1,
                &cancel,
            )
            .unwrap(),
        );
        let c = project(source, Profile::Extended, ActorArena::new(), &ctl).unwrap();
        let callout = c.blocks.iter().find(|b| b.kind() == 4).unwrap();
        assert_eq!(c.text(callout.aux0()).unwrap(), "custom");
        assert_eq!(callout.aux2(), 2);
        let title = callout.aux1();
        let mut parts = String::new();
        for run in c.runs.iter().filter(|r| r.domain() == title) {
            if run.text() != NONE {
                parts.push_str(c.text(run.text()).unwrap());
            }
        }
        assert_eq!(parts, "Title $x$");
        assert!(c.resources.is_empty());
        let body = c
            .blocks
            .iter()
            .find(|b| b.parent() == callout.id() && b.kind() == 1)
            .unwrap();
        let mut parts = String::new();
        for run in c.runs.iter().filter(|r| r.domain() == body.domain()) {
            if run.text() != NONE {
                parts.push_str(c.text(run.text()).unwrap());
            }
        }
        assert_eq!(parts, "Body");
    }
}

#[cfg(test)]
mod nested_tests {
    use super::*;
    use crate::input::SourceOrigin;
    use std::{
        sync::atomic::AtomicBool,
        time::{Duration, Instant},
    };
    #[test]
    fn nested_image_alt_has_no_resource_and_encloses_the_complete_nested_token() {
        let cancel = AtomicBool::new(false);
        let ctl = Control {
            cancel: &cancel,
            deadline: Instant::now() + Duration::from_secs(10),
        };
        let original = "![before ![**inner**](inner.png) after](outer.png)\n";
        let source = Arc::new(
            DocumentSource::new(
                Arc::new(original.to_owned()),
                SourceOrigin::Literal,
                1,
                1,
                &cancel,
            )
            .unwrap(),
        );
        let c = project(source, Profile::Extended, ActorArena::new(), &ctl).unwrap();
        assert_eq!(c.resources.len(), 1);
        let inner = c
            .runs
            .iter()
            .find(|r| r.text() != NONE && c.text(r.text()).unwrap() == "inner")
            .unwrap();
        assert_eq!(inner.styles(), 2);
        let origin = c.origins[inner.origin() as usize];
        assert_eq!(origin.relation(), 1);
        assert_eq!(
            &original[origin.start() as usize..origin.end() as usize],
            "![**inner**](inner.png)"
        );
        let text = c.texts[inner.text() as usize];
        assert_eq!(text.kind(), 1);
        assert_eq!(c.origins[text.origin() as usize].relation(), 1);
    }
}

#[cfg(test)]
mod profile_tests {
    use super::*;
    use crate::input::SourceOrigin;
    use std::{
        sync::atomic::AtomicBool,
        time::{Duration, Instant},
    };
    #[test]
    fn core_rejects_extended_task_and_footnote_mutants() {
        let cancel = AtomicBool::new(false);
        let ctl = Control {
            cancel: &cancel,
            deadline: Instant::now() + Duration::from_secs(10),
        };
        for (raw, task) in [("- task\n", true), ("label\n", false)] {
            let source = Arc::new(
                DocumentSource::new(
                    Arc::new(raw.to_owned()),
                    SourceOrigin::Literal,
                    1,
                    1,
                    &cancel,
                )
                .unwrap(),
            );
            let mut candidate = project(source, Profile::Core, ActorArena::new(), &ctl).unwrap();
            if task {
                let item = candidate.blocks.iter_mut().find(|b| b.kind() == 6).unwrap();
                assert_eq!(item.aux0(), 0);
                item.0[7] = 1;
            } else {
                let run = candidate.runs.iter_mut().find(|r| r.kind() == 0).unwrap();
                run.0[3] = 4;
            }
            assert_eq!(
                crate::document_validate::validate(&candidate, &ctl)
                    .unwrap_err()
                    .kind,
                ErrorKind::InvalidContext
            );
        }
    }
    #[test]
    fn hostile_declared_pixel_dimensions_cannot_disagree_with_raw_alias() {
        let cancel = AtomicBool::new(false);
        let ctl = Control {
            cancel: &cancel,
            deadline: Instant::now() + Duration::from_secs(10),
        };
        let source = Arc::new(
            DocumentSource::new(
                Arc::new("![[image|100x145]]".to_owned()),
                SourceOrigin::Literal,
                1,
                1,
                &cancel,
            )
            .unwrap(),
        );
        let mut candidate = project(source, Profile::Extended, ActorArena::new(), &ctl).unwrap();
        candidate.resources[0].0[9] = 99;
        assert_eq!(
            crate::document_validate::validate(&candidate, &ctl)
                .unwrap_err()
                .kind,
            ErrorKind::InvalidContext
        );
    }
}

#[cfg(test)]
mod context_tests {
    use super::*;
    use crate::input::SourceOrigin;
    use std::{
        sync::atomic::AtomicBool,
        time::{Duration, Instant},
    };
    fn parsed(text: &str) -> Candidate {
        let cancel = AtomicBool::new(false);
        let ctl = Control {
            cancel: &cancel,
            deadline: Instant::now() + Duration::from_secs(10),
        };
        let source = Arc::new(
            DocumentSource::new(
                Arc::new(text.to_owned()),
                SourceOrigin::Literal,
                1,
                1,
                &cancel,
            )
            .unwrap(),
        );
        project(source, Profile::Extended, ActorArena::new(), &ctl).unwrap()
    }
    fn visible(c: &Candidate) -> String {
        let mut text = String::new();
        for run in c.runs.iter() {
            if run.text() != NONE {
                text.push_str(c.text(run.text()).unwrap());
            }
        }
        text
    }
    #[test]
    fn known_marker_and_header_only_keep_roles() {
        let c = parsed("> [!NOTE]\n> Body\n");
        let callout = c.blocks.iter().find(|b| b.kind() == 4).unwrap();
        assert_eq!(c.text(callout.aux0()).unwrap(), "NOTE");
        assert_eq!(visible(&c), "Body");
        let c = parsed("> [!custom]+ Title\n");
        let callout = c.blocks.iter().find(|b| b.kind() == 4).unwrap();
        assert_eq!(callout.item_count(), 0);
        assert_eq!(callout.aux2(), 1);
        assert_eq!(visible(&c), "Title");
    }
    #[test]
    fn crossing_or_invalid_marker_remains_literal_quote() {
        for text in ["> [!custom] **Title\n> Body**\n", "> [!bad?] literal\n"] {
            let c = parsed(text);
            assert!(c.blocks.iter().any(|b| b.kind() == 3));
            assert!(!c.blocks.iter().any(|b| b.kind() == 4));
            assert!(visible(&c).contains(if text.contains("bad?") {
                "[!bad?]"
            } else {
                "[!custom]"
            }));
        }
    }
    #[test]
    fn callout_semantics_are_proved_against_original_marker() {
        for mutation in 0..3 {
            let mut c = parsed("> [!custom]- Title\n> Body\n");
            let callout = c.blocks.iter().position(|b| b.kind() == 4).unwrap();
            match mutation {
                0 => c.blocks[callout].0[9] = 1,
                1 => {
                    c.blocks[callout].0[7] =
                        c.runs.iter().find(|r| r.text() != NONE).unwrap().text()
                }
                _ => {
                    let o = c.blocks[callout].aux3() as usize;
                    c.origins[o].0[1] += 1;
                }
            }
            let cancel = AtomicBool::new(false);
            let ctl = Control {
                cancel: &cancel,
                deadline: Instant::now() + Duration::from_secs(10),
            };
            assert_eq!(
                crate::document_validate::validate(&c, &ctl)
                    .unwrap_err()
                    .kind,
                ErrorKind::InvalidContext
            );
        }
    }
}

#[cfg(test)]
mod empty_callout_tests {
    use super::*;
    use crate::input::SourceOrigin;
    use std::{
        sync::atomic::AtomicBool,
        time::{Duration, Instant},
    };
    #[test]
    fn empty_span_owner_mutants_are_rejected_not_panicked() {
        for mutation in 0..6 {
            let cancel = AtomicBool::new(false);
            let ctl = Control {
                cancel: &cancel,
                deadline: Instant::now() + Duration::from_secs(10),
            };
            let source = Arc::new(
                DocumentSource::new(
                    Arc::new("> [!note] [](x)\n> [](y)\n".to_owned()),
                    SourceOrigin::Literal,
                    1,
                    1,
                    &cancel,
                )
                .unwrap(),
            );
            let mut c = project(source, Profile::Extended, ActorArena::new(), &ctl).unwrap();
            crate::document_validate::validate(&c, &ctl).unwrap();
            let block = c.blocks.iter().position(|b| b.kind() == 4).unwrap();
            let title = c.blocks[block].aux1();
            let span = c
                .spans
                .iter()
                .position(|s| s.domain() == title && s.link() != NONE)
                .unwrap();
            let link = c.spans[span].link() as usize;
            match mutation {
                0 => c.blocks[block].0[8] = NONE,
                1 => c.domains[title as usize].0[2] = 0,
                2 => {
                    c.spans[span].0[3] = 1;
                    c.spans[span].0[4] = 1;
                }
                3 => c.links[link].0[7] = NONE,
                4 => c.links[link].0[9] = 1,
                _ => {
                    let origin = c.spans[span].origin() as usize;
                    c.origins[origin].0[2] = c.source.len() as u32 + 1;
                }
            }
            assert_eq!(
                crate::document_validate::validate(&c, &ctl)
                    .unwrap_err()
                    .kind,
                ErrorKind::InvalidContext
            );
        }
    }
}

#[cfg(test)]
mod body_only_callout_tests {
    use super::*;
    use crate::input::SourceOrigin;
    use std::{
        sync::atomic::AtomicBool,
        time::{Duration, Instant},
    };
    #[test]
    fn body_only_lower_upper_crlf_eof_keep_origin_and_body_owner() {
        for text in [
            "> [!note]\n> [](y)\n",
            "> [!NOTE]\n> [](y)\n",
            "> [!note]\r\n> [](y)\r\n",
            "> [!NOTE]\n> [](y)",
            "> [!custom]\n> [](y)\n",
            "> [!TIP]\n> [](y)\n",
        ] {
            let cancel = AtomicBool::new(false);
            let ctl = Control {
                cancel: &cancel,
                deadline: Instant::now() + Duration::from_secs(10),
            };
            let source = Arc::new(
                DocumentSource::new(
                    Arc::new(text.to_owned()),
                    SourceOrigin::Literal,
                    1,
                    1,
                    &cancel,
                )
                .unwrap(),
            );
            let c = project(source, Profile::Extended, ActorArena::new(), &ctl).unwrap();
            crate::document_validate::validate(&c, &ctl).unwrap();
            assert_eq!(c.source.text(), text);
            let callout = c.blocks.iter().find(|b| b.kind() == 4).unwrap();
            assert_eq!(callout.aux1(), NONE, "no source title: {text:?}");
            assert_eq!(c.links.len(), 1);
            let link = c.links[0];
            assert_eq!(c.text(link.target_decoded()).unwrap(), "y");
            assert_eq!(c.text(link.raw_token()).unwrap(), "[](y)");
            let domain = c.domains[link.label_domain() as usize];
            assert_eq!(
                (
                    domain.kind(),
                    domain.run_count(),
                    link.label_start(),
                    link.label_count()
                ),
                (0, 0, 0, 0)
            );
            let body = c.blocks[domain.owner() as usize];
            assert_eq!(
                (body.kind(), body.parent(), body.domain(), body.item_count()),
                (1, callout.id(), domain.id(), 0)
            );
            let span = c.spans.iter().find(|s| s.link() == link.id()).unwrap();
            assert_eq!(
                (span.domain(), span.start_ordinal(), span.end_ordinal()),
                (domain.id(), 0, 0)
            );
            let o = c.origins[span.origin() as usize];
            let start = text.find("[](y)").unwrap();
            assert_eq!((o.start() as usize, o.end() as usize), (start, start + 5));
            let raw = c.texts[link.raw_token() as usize];
            let ro = c.origins[raw.origin() as usize];
            assert_eq!((ro.start(), ro.end()), (o.start(), o.end()));
        }
    }
    #[test]
    fn primary_consumed_markers_keep_empty_link_source_ranges() {
        for text in [
            "> [!note]\n> [](y)\n",
            "> [!NOTE]\n> [](y)\n",
            "> [!note]\r\n> [](y)\r\n",
            "> [!NOTE]\n> [](y)",
        ] {
            let mut links = 0;
            for (event, range) in
                Parser::new_ext(text, Options::from_bits(35998).unwrap()).into_offset_iter()
            {
                assert!(!matches!(
                    event,
                    Event::Text(_) | Event::SoftBreak | Event::HardBreak
                ));
                if let Event::Start(Tag::Link { dest_url, .. }) = event {
                    assert_eq!(dest_url.as_ref(), "y");
                    let start = text.find("[](y)").unwrap();
                    assert_eq!(range, start..start + 5);
                    links += 1;
                    println!("ORIGINAL_BODY_ONLY {text:?} {range:?}");
                }
            }
            assert_eq!(links, 1);
        }
    }
    #[test]
    fn classifier_source_zone_and_nonempty_rules_are_distinct() {
        let m = Marker {
            kind: 0..1,
            marker: 0..9,
            fold: 0,
            title_start: 9,
            line_end: 9,
        };
        let empty = Span::from_words([0, 0, NONE, 0, 0, 3, 0, 0, 0, 0]);
        assert_eq!(
            callout_span_header(&empty, Origin::from_words([0, 12, 17, 0]), &m, Some(10), 0),
            Some(false)
        );
        assert_eq!(
            callout_span_header(&empty, Origin::from_words([0, 9, 9, 0]), &m, Some(10), 0),
            Some(true)
        );
        for range in [(9, 10), (8, 10), (8, 9)] {
            assert_eq!(
                callout_span_header(
                    &empty,
                    Origin::from_words([0, range.0, range.1, 0]),
                    &m,
                    Some(10),
                    0
                ),
                None
            );
        }
        assert_eq!(
            callout_span_header(&empty, Origin::from_words([0, 12, 17, 0]), &m, None, 0),
            None
        );
        let nonempty = Span::from_words([0, 0, NONE, 0, 1, 3, 0, 0, 0, 0]);
        assert_eq!(
            callout_span_header(
                &nonempty,
                Origin::from_words([0, 12, 17, 0]),
                &m,
                Some(10),
                1
            ),
            Some(true)
        );
    }
}
