//! Semantic Markdown search. Preparation-only, no worker, history or visible Flow.
//! A single Items walker includes hidden bodies. D1 supplies safe projected chunks;
//! original Source bytes use a distinct result. No flattened document or hit index.
use crate::{
    document_consumer_budget::{Charge, ChargedVec, ConsumerBudget},
    document_model::{Document, NONE},
    document_source::DocumentSource,
    document_text::{
        Coverage, DocumentWork, Error, IndexProfile, Point, RefCoverage, Result, SegmentedIndex,
        SegmentedText, TextRefIndex,
    },
    search::{Direction, QUERY_LIMIT},
    text_engine::Identity,
};
use std::{
    mem::size_of,
    ops::Range,
    sync::{Arc, atomic::AtomicBool},
};

const PROFILE: IndexProfile = IndexProfile {
    projection_points: 2,
    unicode_points: 2,
    exceptions: 512,
};
const CHUNK: usize = 4096;
pub(crate) const HEADING_PAGE: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SearchDomain {
    MainBody,
    CalloutTitle,
    ResourceAlt,
    ResourceTitle,
    SourceOriginal,
}

/// Never accepts a string/buffer painted by a renderer as provenance.
pub(crate) struct SearchRequest {
    document: Arc<Document>,
    budget: ConsumerBudget,
    domain: SearchDomain,
    revision: u64,
    identity: Identity,
    query: ChargedVec<u8>,
    prefix: ChargedVec<usize>,
    _charge: Charge,
}
/// A sealed cursor, tied to this exact query, source and document snapshot.
/// Starts (rather than ends) order continuations, preserving overlapping matches.
#[derive(Debug)]
pub(crate) struct SearchCursor {
    identity: Identity,
    domain: SearchDomain,
    sequence: u64,
    start: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DocumentAnchor {
    Run {
        run: u32,
        raw_byte: usize,
        projection_part: u8,
    },
    TextRef {
        text_ref: u32,
        raw_byte: usize,
    },
}
pub(crate) enum HitCoverage {
    Run(Coverage),
    TextRef(RefCoverage),
}

pub(crate) struct DocumentHit {
    document: Arc<Document>,
    revision: u64,
    domain: SearchDomain,
    sequence: Sequence,
    cursor: SearchCursor,
    matched: Range<usize>,
    highlight: Option<Range<usize>>,
    anchor: DocumentAnchor,
    provider: Option<HitProvider>,
    _charge: Charge,
}
impl DocumentHit {
    pub(crate) fn document(&self) -> &Arc<Document> {
        &self.document
    }
    pub(crate) fn query_revision(&self) -> u64 {
        self.revision
    }
    pub(crate) fn domain(&self) -> SearchDomain {
        self.domain
    }
    pub(crate) fn block(&self) -> u32 {
        self.sequence.block
    }
    /// Owner comes from the actual validated Resource record, never TextRef or ordinal inference.
    pub(crate) fn resource_id(&self) -> Option<u32> {
        self.sequence.resource_id
    }
    pub(crate) fn domain_id(&self) -> Option<u32> {
        match self.sequence.payload {
            Payload::Runs { domain, .. } => Some(domain),
            _ => None,
        }
    }
    pub(crate) fn cursor(&self) -> &SearchCursor {
        &self.cursor
    }
    pub(crate) fn matched(&self) -> Range<usize> {
        self.matched.clone()
    }
    pub(crate) fn highlight(&self) -> Option<Range<usize>> {
        self.highlight.clone()
    }
    pub(crate) fn anchor(&self) -> DocumentAnchor {
        self.anchor
    }
    /// Visits the entire glyph coverage. Pending positioning must be retried
    /// explicitly; no fake coverage or Source bounding range is manufactured.
    pub(crate) fn visit_coverage(
        &self,
        work: &mut DocumentWork<'_>,
        mut visit: impl FnMut(HitCoverage) -> Result<()>,
    ) -> Result<()> {
        let range = self.highlight.clone().ok_or(Error::Invalid)?;
        match self.provider.as_ref().ok_or(Error::Invalid)? {
            HitProvider::Runs(index) => index
                .text()
                .visit_coverage(range, work, |c| visit(HitCoverage::Run(c))),
            HitProvider::TextRef(index) => {
                index.visit_coverage(range, work, |c| visit(HitCoverage::TextRef(c)))
            }
        }
    }
}
/// Absolute original UTF8 bytes, with no Block/Run/Domain or inverse mapping.
pub(crate) struct SourceHit {
    source: Arc<DocumentSource>,
    revision: u64,
    cursor: SearchCursor,
    range: Range<usize>,
    _charge: Charge,
}
impl SourceHit {
    pub(crate) fn source(&self) -> &Arc<DocumentSource> {
        &self.source
    }
    pub(crate) fn query_revision(&self) -> u64 {
        self.revision
    }
    pub(crate) fn range(&self) -> Range<usize> {
        self.range.clone()
    }
    pub(crate) fn cursor(&self) -> &SearchCursor {
        &self.cursor
    }
}
pub(crate) enum SearchOutcome {
    DocumentFound(DocumentHit),
    DocumentFoundPending(DocumentHit, Error),
    SourceMatched(SourceHit),
    NoMatch,
    Cancelled,
    Failed(Error),
}
impl SearchOutcome {
    pub(crate) fn cursor(&self) -> Option<&SearchCursor> {
        match self {
            Self::DocumentFound(hit) | Self::DocumentFoundPending(hit, _) => Some(hit.cursor()),
            Self::SourceMatched(hit) => Some(hit.cursor()),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Payload {
    Runs { domain: u32, first: u32, count: u32 },
    TextRef(u32),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Sequence {
    resource_id: Option<u32>,
    ordinal: u64,
    block: u32,
    payload: Payload,
}
#[derive(Clone, Copy, Default)]
struct Frame {
    block: u32,
    next: u32,
    entered: bool,
}
struct Walker {
    stack: [Frame; 65],
    depth: usize,
}
enum SemanticEvent {
    Enter(u32),
    Run { block: u32, run: u32 },
    Leave,
}
impl Walker {
    fn new() -> Self {
        Self {
            stack: [Frame::default(); 65],
            depth: 1,
        }
    }
    fn next(
        &mut self,
        document: &Document,
        work: &mut DocumentWork<'_>,
    ) -> Result<Option<SemanticEvent>> {
        loop {
            work.admit_structure(0)?;
            if self.depth == 0 {
                return Ok(None);
            }
            work.admit_structure(size_of::<crate::document_model::Block>() + size_of::<Frame>())?;
            let at = self.depth - 1;
            let frame = self.stack[at];
            let block = document
                .blocks()
                .get(frame.block as usize)
                .ok_or(Error::Invalid)?;
            if !frame.entered {
                self.stack[at].entered = true;
                return Ok(Some(SemanticEvent::Enter(frame.block)));
            }
            if frame.next == block.item_count() {
                self.depth -= 1;
                return Ok(Some(SemanticEvent::Leave));
            }
            let id = block
                .first_item()
                .checked_add(frame.next)
                .ok_or(Error::Invalid)?;
            work.admit_structure(size_of::<crate::document_model::Item>())?;
            let item = document.items().get(id as usize).ok_or(Error::Invalid)?;
            self.stack[at].next = frame.next.checked_add(1).ok_or(Error::Invalid)?;
            match item.kind() {
                0 => {
                    if self.depth == self.stack.len() {
                        return Err(Error::Invalid);
                    }
                    self.stack[self.depth] = Frame {
                        block: item.target(),
                        ..Frame::default()
                    };
                    self.depth += 1;
                }
                1 => {
                    return Ok(Some(SemanticEvent::Run {
                        block: frame.block,
                        run: item.target(),
                    }));
                }
                _ => return Err(Error::Invalid),
            }
        }
    }
}
/// Grouping walks Items, never the numerical domain/block table order.
struct Sequences {
    walker: Walker,
    pending: Option<SemanticEvent>,
    next_ordinal: u64,
}
impl Sequences {
    fn new() -> Self {
        Self {
            walker: Walker::new(),
            pending: None,
            next_ordinal: 0,
        }
    }
    fn wrap(&mut self, block: u32, payload: Payload, resource_id: Option<u32>) -> Result<Sequence> {
        let ordinal = self.next_ordinal;
        self.next_ordinal = ordinal.checked_add(1).ok_or(Error::Invalid)?;
        Ok(Sequence {
            resource_id,
            ordinal,
            block,
            payload,
        })
    }
    fn domain_payload(
        document: &Document,
        domain: u32,
        work: &mut DocumentWork<'_>,
    ) -> Result<Payload> {
        work.admit_structure(size_of::<crate::document_model::Domain>())?;
        let d = document
            .domains()
            .get(domain as usize)
            .ok_or(Error::Invalid)?;
        Ok(Payload::Runs {
            domain,
            first: 0,
            count: d.run_count(),
        })
    }
    fn resource(
        &mut self,
        document: &Document,
        block: u32,
        id: u32,
        domain: SearchDomain,
        work: &mut DocumentWork<'_>,
    ) -> Result<Option<Sequence>> {
        work.admit_structure(size_of::<crate::document_model::Resource>())?;
        let resource = document
            .resources()
            .get(id as usize)
            .ok_or(Error::Invalid)?;
        let payload = match domain {
            SearchDomain::ResourceAlt if resource.alt_domain() != NONE => {
                Self::domain_payload(document, resource.alt_domain(), work)?
            }
            SearchDomain::ResourceTitle if resource.title_decoded() != NONE => {
                Payload::TextRef(resource.title_decoded())
            }
            _ => return Ok(None),
        };
        self.wrap(block, payload, Some(id)).map(Some)
    }
    fn next(
        &mut self,
        document: &Document,
        domain: SearchDomain,
        work: &mut DocumentWork<'_>,
    ) -> Result<Option<Sequence>> {
        loop {
            let event = if let Some(event) = self.pending.take() {
                Some(event)
            } else {
                self.walker.next(document, work)?
            };
            match event {
                None => return Ok(None),
                Some(SemanticEvent::Enter(block)) => {
                    work.admit_structure(size_of::<crate::document_model::Block>())?;
                    let b = &document.blocks()[block as usize];
                    if domain == SearchDomain::CalloutTitle && b.kind() == 4 && b.aux1() != NONE {
                        let payload = Self::domain_payload(document, b.aux1(), work)?;
                        return self.wrap(block, payload, None).map(Some);
                    }
                    // Mermaid descriptors can belong to a code block, rather than a Run.
                    if matches!(
                        domain,
                        SearchDomain::ResourceAlt | SearchDomain::ResourceTitle
                    ) && b.kind() == 10
                        && b.aux3() != NONE
                        && let Some(sequence) =
                            self.resource(document, block, b.aux3(), domain, work)?
                    {
                        return Ok(Some(sequence));
                    }
                }
                Some(SemanticEvent::Run { block, run }) => {
                    work.admit_structure(size_of::<crate::document_model::Run>())?;
                    let r = document.runs().get(run as usize).ok_or(Error::Invalid)?;
                    if r.kind() == 5 {
                        if let Some(sequence) =
                            self.resource(document, block, r.resource(), domain, work)?
                        {
                            return Ok(Some(sequence));
                        }
                        continue;
                    }
                    if domain != SearchDomain::MainBody {
                        continue;
                    }
                    let first = r.ordinal();
                    let mut count = 1u32;
                    loop {
                        let next = self.walker.next(document, work)?;
                        let compatible = match next {
                            Some(SemanticEvent::Run {
                                block: next_block,
                                run: next_run,
                            }) => {
                                work.admit_structure(size_of::<crate::document_model::Run>())?;
                                let next_record = document
                                    .runs()
                                    .get(next_run as usize)
                                    .ok_or(Error::Invalid)?;
                                next_block == block
                                    && next_record.domain() == r.domain()
                                    && next_record.kind() != 5
                                    && next_record.ordinal()
                                        == first.checked_add(count).ok_or(Error::Invalid)?
                            }
                            _ => false,
                        };
                        if compatible {
                            count = count.checked_add(1).ok_or(Error::Invalid)?;
                        } else {
                            self.pending = next;
                            break;
                        }
                    }
                    return self
                        .wrap(
                            block,
                            Payload::Runs {
                                domain: r.domain(),
                                first,
                                count,
                            },
                            None,
                        )
                        .map(Some);
                }
                Some(SemanticEvent::Leave) => {}
            }
        }
    }
}
struct Kmp<'a> {
    query: &'a [u8],
    prefix: &'a [usize],
    matched: usize,
    offset: usize,
}
impl<'a> Kmp<'a> {
    fn new(query: &'a [u8], prefix: &'a [usize]) -> Self {
        Self {
            query,
            prefix,
            matched: 0,
            offset: 0,
        }
    }
    fn feed(&mut self, bytes: &[u8], mut found: impl FnMut(Range<usize>)) -> Result<()> {
        if bytes.len() > CHUNK {
            return Err(Error::Invalid);
        }
        for &byte in bytes {
            while self.matched != 0 && byte != self.query[self.matched] {
                self.matched = self.prefix[self.matched - 1];
            }
            if byte == self.query[self.matched] {
                self.matched += 1;
            }
            self.offset = self.offset.checked_add(1).ok_or(Error::Invalid)?;
            if self.matched == self.query.len() {
                found(self.offset - self.query.len()..self.offset);
                self.matched = self.prefix[self.matched - 1];
            }
        }
        Ok(())
    }
}
enum ProjectionProvider {
    Runs(SegmentedText),
    TextRef(TextRefIndex),
}
enum HitProvider {
    Runs(SegmentedIndex),
    TextRef(TextRefIndex),
}
struct Match {
    sequence: Sequence,
    range: Range<usize>,
    primary: bool,
    provider: ProjectionProvider,
}
impl SearchRequest {
    pub(crate) fn prepare(
        document: Arc<Document>,
        budget: &ConsumerBudget,
        domain: SearchDomain,
        query: &str,
        revision: u64,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        let mut work = DocumentWork::preparation(cancel);
        work.admit_structure(0)?;
        if !budget.belongs_to(&document) || query.len() > QUERY_LIMIT {
            return Err(Error::Invalid);
        }
        let charge = budget.reserve(size_of::<Self>() + 2 * size_of::<usize>())?;
        let mut bytes = ChargedVec::new(budget, query.len())?;
        let mut prefix = ChargedVec::new(budget, query.len())?;
        for (at, byte) in query.bytes().enumerate() {
            work.admit_structure(3)?;
            bytes.push(byte)?;
            let mut matched = if at == 0 {
                0
            } else {
                prefix.as_slice()[at - 1]
            };
            while matched > 0 && byte != query.as_bytes()[matched] {
                work.admit_structure(1)?;
                matched = prefix.as_slice()[matched - 1];
            }
            if at != 0 && byte == query.as_bytes()[matched] {
                matched += 1;
            }
            prefix.push(matched)?;
        }
        Ok(Self {
            document,
            budget: budget.clone(),
            domain,
            revision,
            identity: Identity::new(),
            query: bytes,
            prefix,
            _charge: charge,
        })
    }
    fn cursor(&self, sequence: u64, start: usize) -> SearchCursor {
        SearchCursor {
            identity: self.identity.clone(),
            domain: self.domain,
            sequence,
            start,
        }
    }
    fn eligible(
        direction: Direction,
        after: Option<&SearchCursor>,
        sequence: u64,
        start: usize,
    ) -> bool {
        after.is_none_or(|after| match direction {
            Direction::Forward => (sequence, start) > (after.sequence, after.start),
            Direction::Backward => (sequence, start) < (after.sequence, after.start),
        })
    }
    fn better(
        direction: Direction,
        primary: bool,
        coordinate: (u64, usize),
        old_primary: bool,
        old_coordinate: (u64, usize),
    ) -> bool {
        if primary != old_primary {
            return primary;
        }
        match direction {
            Direction::Forward => coordinate < old_coordinate,
            Direction::Backward => coordinate > old_coordinate,
        }
    }
    /// Owner/background entry point. All provider preparations use their real
    /// preparation Work (cancelable); no UI frame or latency bound is implied.
    pub(crate) fn find(
        &self,
        direction: Direction,
        after: Option<&SearchCursor>,
        cancel: &AtomicBool,
    ) -> SearchOutcome {
        let mut work = DocumentWork::preparation(cancel);
        self.find_with_work(direction, after, cancel, &mut work)
    }
    /// An injected Work can reject traversal/matching/coverage; provider preparation
    /// still uses D1's own preparation Work. This is not a total CPU quota API.
    pub(crate) fn find_with_work(
        &self,
        direction: Direction,
        after: Option<&SearchCursor>,
        cancel: &AtomicBool,
        work: &mut DocumentWork<'_>,
    ) -> SearchOutcome {
        match self.find_inner(direction, after, cancel, work) {
            Ok(outcome) => outcome,
            Err(Error::Cancelled) => SearchOutcome::Cancelled,
            Err(error) => SearchOutcome::Failed(error),
        }
    }
    fn find_inner(
        &self,
        direction: Direction,
        after: Option<&SearchCursor>,
        cancel: &AtomicBool,
        work: &mut DocumentWork<'_>,
    ) -> Result<SearchOutcome> {
        work.admit_structure(0)?;
        if after.is_some_and(|cursor| {
            !cursor.identity.same(&self.identity) || cursor.domain != self.domain
        }) {
            return Err(Error::Invalid);
        }
        if self.query.as_slice().is_empty() {
            return Ok(SearchOutcome::NoMatch);
        }
        let _scratch = self
            .budget
            .reserve(size_of::<Sequences>() + size_of::<Kmp<'_>>() + size_of::<Match>())?;
        let result_charge = self.budget.reserve(
            size_of::<DocumentHit>().max(size_of::<SourceHit>()) + 2 * size_of::<usize>(),
        )?;
        if self.domain == SearchDomain::SourceOriginal {
            return self.find_source(direction, after, work, result_charge);
        }
        let mut sequences = Sequences::new();
        let mut best: Option<Match> = None;
        while let Some(sequence) = sequences.next(&self.document, self.domain, work)? {
            let provider = self.provider(sequence, cancel)?;
            let mut matcher = Kmp::new(self.query.as_slice(), self.prefix.as_slice());
            let mut selected: Option<(Range<usize>, bool)> = None;
            provider.visit(work, |chunk| {
                matcher.feed(chunk.as_bytes(), |range| {
                    let primary = Self::eligible(direction, after, sequence.ordinal, range.start);
                    if selected.as_ref().is_none_or(|(old, old_primary)| {
                        Self::better(
                            direction,
                            primary,
                            (sequence.ordinal, range.start),
                            *old_primary,
                            (sequence.ordinal, old.start),
                        )
                    }) {
                        selected = Some((range, primary));
                    }
                })
            })?;
            if let Some((range, primary)) = selected
                && best.as_ref().is_none_or(|old| {
                    Self::better(
                        direction,
                        primary,
                        (sequence.ordinal, range.start),
                        old.primary,
                        (old.sequence.ordinal, old.range.start),
                    )
                })
            {
                // New provider is fully charged before the previous candidate is released.
                best = Some(Match {
                    sequence,
                    range,
                    primary,
                    provider,
                });
                if primary && matches!(direction, Direction::Forward) {
                    break;
                }
            }
        }
        work.admit_structure(0)?;
        match best {
            None => Ok(SearchOutcome::NoMatch),
            Some(found) => self.document_hit(found, cancel, work, result_charge),
        }
    }
    /// Explicit positioning retry; callers keep current/pending state until success.
    pub(crate) fn retry_hit(&self, hit: &DocumentHit, cancel: &AtomicBool) -> SearchOutcome {
        let mut work = DocumentWork::preparation(cancel);
        let prepared = (|| {
            work.admit_structure(0)?;
            if !Arc::ptr_eq(&self.document, &hit.document)
                || !hit.cursor.identity.same(&self.identity)
                || self.domain != hit.domain
                || self.revision != hit.revision
            {
                return Err(Error::Invalid);
            }
            let _scratch = self.budget.reserve(size_of::<Match>())?;
            let result_charge = self
                .budget
                .reserve(size_of::<DocumentHit>() + 2 * size_of::<usize>())?;
            let provider = self.provider(hit.sequence, cancel)?;
            self.document_hit(
                Match {
                    sequence: hit.sequence,
                    range: hit.matched.clone(),
                    primary: true,
                    provider,
                },
                cancel,
                &mut work,
                result_charge,
            )
        })();
        match prepared {
            Ok(outcome) => outcome,
            Err(Error::Cancelled) => SearchOutcome::Cancelled,
            Err(error) => SearchOutcome::Failed(error),
        }
    }

    fn provider(&self, sequence: Sequence, cancel: &AtomicBool) -> Result<ProjectionProvider> {
        Ok(match sequence.payload {
            Payload::Runs {
                domain,
                first,
                count,
            } => ProjectionProvider::Runs(SegmentedText::prepare_with_profile(
                self.document.clone(),
                domain,
                first as usize..first.checked_add(count).ok_or(Error::Invalid)? as usize,
                &self.budget,
                cancel,
                PROFILE,
            )?),
            Payload::TextRef(id) => ProjectionProvider::TextRef(TextRefIndex::prepare(
                self.document.clone(),
                id,
                &self.budget,
                cancel,
                PROFILE,
            )?),
        })
    }
    fn document_hit(
        &self,
        found: Match,
        cancel: &AtomicBool,
        work: &mut DocumentWork<'_>,
        charge: Charge,
    ) -> Result<SearchOutcome> {
        let mut anchor = None;
        let mut coverage_work = DocumentWork::preparation(cancel);
        match &found.provider {
            ProjectionProvider::Runs(text) => {
                text.visit_coverage(found.range.clone(), &mut coverage_work, |coverage| {
                    if anchor.is_none() {
                        anchor = Some(DocumentAnchor::Run {
                            run: coverage.run,
                            raw_byte: coverage.local.start,
                            projection_part: 0,
                        });
                    }
                    Ok(())
                })?
            }
            ProjectionProvider::TextRef(index) => {
                index.visit_coverage(found.range.clone(), &mut coverage_work, |coverage| {
                    if anchor.is_none() {
                        anchor = Some(DocumentAnchor::TextRef {
                            text_ref: coverage.text_ref,
                            raw_byte: coverage.local.start,
                        });
                    }
                    Ok(())
                })?
            }
        }
        let mut hit = DocumentHit {
            document: self.document.clone(),
            revision: self.revision,
            domain: self.domain,
            cursor: self.cursor(found.sequence.ordinal, found.range.start),
            sequence: found.sequence,
            matched: found.range.clone(),
            highlight: None,
            anchor: anchor.ok_or(Error::Invalid)?,
            provider: None,
            _charge: charge,
        };
        if let Err(cause) = work.admit_structure(coverage_work.used()) {
            if cause == Error::Cancelled {
                return Err(cause);
            }
            return Ok(SearchOutcome::DocumentFoundPending(hit, cause));
        }

        let prepared = match found.provider {
            ProjectionProvider::Runs(text) => {
                SegmentedIndex::prepare_with_profile(text, &self.budget, cancel, PROFILE)
                    .map(HitProvider::Runs)
            }
            ProjectionProvider::TextRef(index) => Ok(HitProvider::TextRef(index)),
        };
        match prepared {
            Err(Error::Cancelled) => Err(Error::Cancelled),
            Err(cause) => Ok(SearchOutcome::DocumentFoundPending(hit, cause)),
            Ok(provider) => {
                // Retain provider on Work/position failure, allowing a typed pending
                // result without losing the already accredited semantic match.
                match provider.highlight(found.range, work) {
                    Ok((range, exact_anchor)) => {
                        hit.highlight = Some(range);
                        if let Some(anchor) = exact_anchor {
                            hit.anchor = anchor;
                        }
                        hit.provider = Some(provider);
                        work.admit_structure(0)?;
                        Ok(SearchOutcome::DocumentFound(hit))
                    }
                    Err(Error::Cancelled) => Err(Error::Cancelled),
                    Err(cause) => {
                        hit.provider = Some(provider);
                        Ok(SearchOutcome::DocumentFoundPending(hit, cause))
                    }
                }
            }
        }
    }
    fn find_source(
        &self,
        direction: Direction,
        after: Option<&SearchCursor>,
        work: &mut DocumentWork<'_>,
        charge: Charge,
    ) -> Result<SearchOutcome> {
        let text = self.document.source().text();
        let mut matcher = Kmp::new(self.query.as_slice(), self.prefix.as_slice());
        let mut best: Option<(Range<usize>, bool)> = None;
        // UTF8 query matching implies UTF8 boundaries; chunks may cut a scalar,
        // because this raw-byte matcher never exposes these chunks as &str.
        for chunk in text.as_bytes().chunks(CHUNK) {
            work.admit_structure(chunk.len().checked_mul(3).ok_or(Error::Work)?)?;
            matcher.feed(chunk, |range| {
                let primary = Self::eligible(direction, after, 0, range.start);
                if best.as_ref().is_none_or(|(old, old_primary)| {
                    Self::better(
                        direction,
                        primary,
                        (0, range.start),
                        *old_primary,
                        (0, old.start),
                    )
                }) {
                    best = Some((range, primary));
                }
            })?;
        }
        work.admit_structure(0)?;
        let Some((range, _)) = best else {
            return Ok(SearchOutcome::NoMatch);
        };
        if text.get(range.clone()).is_none() {
            return Err(Error::Invalid);
        }
        let hit = SourceHit {
            source: self.document.source_handle(),
            revision: self.revision,
            cursor: self.cursor(0, range.start),
            range,
            _charge: charge,
        };
        Ok(SearchOutcome::SourceMatched(hit))
    }
}
impl ProjectionProvider {
    fn visit(
        &self,
        work: &mut DocumentWork<'_>,
        mut visit: impl FnMut(&str) -> Result<()>,
    ) -> Result<()> {
        let len = match self {
            Self::Runs(text) => text.len(),
            Self::TextRef(index) => index.len(),
        };
        // KMP has linear byte work, including its failure transitions.
        work.admit_structure(len.checked_mul(3).ok_or(Error::Work)?)?;
        match self {
            Self::Runs(text) => text.visit_projected(0..len, work, &mut visit),
            Self::TextRef(index) => index.visit_projected(0..len, work, &mut visit),
        }
    }
}
impl HitProvider {
    fn highlight(
        &self,
        matched: Range<usize>,
        work: &mut DocumentWork<'_>,
    ) -> Result<(Range<usize>, Option<DocumentAnchor>)> {
        let mut point: Point = match self {
            Self::Runs(index) => index.seek_before_with_work(matched.start, work)?,
            Self::TextRef(index) => index.start(),
        };
        let mut highlight: Option<Range<usize>> = None;
        let mut anchor = None;
        loop {
            let before = point.clone();
            let unit = match self {
                Self::Runs(index) => index.step(&mut point, 4, work)?,
                Self::TextRef(index) => index.step(&mut point, 4, work)?,
            }
            .ok_or(Error::Invalid)?;
            if unit.projected.end > matched.start && unit.projected.start < matched.end {
                if highlight.is_none() {
                    highlight = Some(unit.projected.clone());
                    if let Self::TextRef(index) = self {
                        index.visit_coverage(unit.projected.clone(), work, |c| {
                            if anchor.is_none() {
                                anchor = Some(DocumentAnchor::TextRef {
                                    text_ref: c.text_ref,
                                    raw_byte: c.local.start,
                                });
                            }
                            Ok(())
                        })?;
                    }
                    if let Self::Runs(index) = self {
                        let (run, raw_byte, projection_part) =
                            index.visual_anchor_with_work(&before, work)?;
                        anchor = Some(DocumentAnchor::Run {
                            run,
                            raw_byte,
                            projection_part,
                        });
                    }
                } else if let Some(range) = &mut highlight {
                    range.end = unit.projected.end;
                }
            }
            if unit.projected.end >= matched.end {
                break;
            }
        }
        Ok((highlight.ok_or(Error::Invalid)?, anchor))
    }
}

/// Sealed logical target, not a visible ordinal or invented slug.
pub(crate) struct HeadingTarget {
    document: Arc<Document>,
    block: u32,
    domain: u32,
}
impl HeadingTarget {
    pub(crate) fn document(&self) -> &Arc<Document> {
        &self.document
    }
    pub(crate) fn block(&self) -> u32 {
        self.block
    }
    pub(crate) fn domain(&self) -> u32 {
        self.domain
    }
    pub(crate) fn generation(&self) -> u64 {
        self.document.source().generation()
    }
    pub(crate) fn request(&self) -> u64 {
        self.document.source().request()
    }
    pub(crate) fn visit_title(
        &self,
        budget: &ConsumerBudget,
        cancel: &AtomicBool,
        work: &mut DocumentWork<'_>,
        visit: impl FnMut(&str) -> Result<()>,
    ) -> Result<()> {
        let payload = Sequences::domain_payload(&self.document, self.domain, work)?;
        let Payload::Runs { count, .. } = payload else {
            return Err(Error::Invalid);
        };
        let text = SegmentedText::prepare_with_profile(
            self.document.clone(),
            self.domain,
            0..count as usize,
            budget,
            cancel,
            PROFILE,
        )?;
        text.visit_projected(0..text.len(), work, visit)
    }
}
pub(crate) struct HeadingCursor {
    document: Arc<Document>,
    walker: Walker,
    _charge: Charge,
}
pub(crate) struct HeadingPage {
    targets: ChargedVec<HeadingTarget>,
    next: Option<HeadingCursor>,
    _charge: Charge,
}
impl HeadingPage {
    pub(crate) fn targets(&self) -> &[HeadingTarget] {
        self.targets.as_slice()
    }
    /// Consuming an accepted page is optional. A continuation can instead be
    /// borrowed below, preparing current+candidate pages before replacement.
    pub(crate) fn next(&self) -> Option<&HeadingCursor> {
        self.next.as_ref()
    }
}
impl HeadingCursor {
    pub(crate) fn start(document: Arc<Document>, budget: &ConsumerBudget) -> Result<Self> {
        if !budget.belongs_to(&document) {
            return Err(Error::Invalid);
        }
        let charge = budget.reserve(size_of::<Self>())?;
        Ok(Self {
            document,
            walker: Walker::new(),
            _charge: charge,
        })
    }
    /// Clone only fixed cursor state under charge. No global headings index.
    pub(crate) fn page(
        &self,
        budget: &ConsumerBudget,
        work: &mut DocumentWork<'_>,
    ) -> Result<HeadingPage> {
        if !budget.belongs_to(&self.document) {
            return Err(Error::Invalid);
        }
        work.admit_structure(size_of::<Walker>())?;
        let cursor_charge = budget.reserve(size_of::<Self>())?;
        let mut cursor = Self {
            document: self.document.clone(),
            walker: Walker {
                stack: self.walker.stack,
                depth: self.walker.depth,
            },
            _charge: cursor_charge,
        };
        let charge = budget.reserve(size_of::<HeadingPage>())?;
        let mut targets = ChargedVec::new(budget, HEADING_PAGE)?;
        while targets.as_slice().len() < HEADING_PAGE {
            let Some(event) = cursor.walker.next(&self.document, work)? else {
                return Ok(HeadingPage {
                    targets,
                    next: None,
                    _charge: charge,
                });
            };
            if let SemanticEvent::Enter(block) = event {
                work.admit_structure(size_of::<crate::document_model::Block>())?;
                let record = &self.document.blocks()[block as usize];
                if record.kind() == 2 {
                    targets.push(HeadingTarget {
                        document: self.document.clone(),
                        block,
                        domain: record.domain(),
                    })?;
                }
            }
        }
        work.admit_structure(0)?;
        Ok(HeadingPage {
            targets,
            next: Some(cursor),
            _charge: charge,
        })
    }
}

#[cfg(test)]
#[path = "document_search_tests.rs"]
mod tests;
