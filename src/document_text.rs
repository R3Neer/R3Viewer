//! Borrowed projected domain text. Run/style boundaries do not reset Unicode state.
//! No source/domain leaf is flattened; only bounded 4096-byte scratch is copied.
use crate::{
    document_consumer_budget::{Charge, ChargedVec, ConsumerBudget},
    document_model::{Document, NONE},
    text_engine::{Identity, Work},
    text_projection,
};
use std::{
    ops::Range,
    sync::{Arc, atomic::AtomicBool},
};
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete};
pub const SYMBOL_BYTES: usize = 16384;
const CHUNK: usize = 4096;
const SPARSE: usize = 1536;
#[derive(Clone, Copy, Debug)]
pub struct IndexProfile {
    pub projection_points: usize,
    pub unicode_points: usize,
    pub exceptions: usize,
}
impl IndexProfile {
    pub const DEFAULT: Self = Self {
        projection_points: SPARSE,
        unicode_points: SPARSE,
        exceptions: 512,
    };
    pub fn checked(self) -> Result<Self> {
        if !(2..=SPARSE).contains(&self.projection_points)
            || !(2..=SPARSE).contains(&self.unicode_points)
            || self.exceptions > 512
        {
            Err(Error::Budget)
        } else {
            Ok(self)
        }
    }
    pub fn fixed_requested_bytes() -> usize {
        2 * CHUNK
            + 8 * std::mem::size_of::<Point>()
            + std::mem::size_of::<SegmentedText>()
            + std::mem::size_of::<SegmentedIndex>()
            + std::mem::size_of::<unicode_width_adapter::WidthState>()
            + 256
    }
    pub fn requested_bytes(self) -> Result<usize> {
        self.checked()?;
        self.projection_points
            .checked_mul(std::mem::size_of::<Checkpoint>())
            .and_then(|n| {
                self.unicode_points
                    .checked_mul(std::mem::size_of::<Point>())
                    .and_then(|v| n.checked_add(v))
            })
            .and_then(|n| {
                self.exceptions
                    .checked_mul(std::mem::size_of::<Exception>())
                    .and_then(|v| n.checked_add(v))
            })
            .ok_or(Error::Budget)
    }
    pub fn checkpoint_capacity(self) -> Result<usize> {
        self.checked()?;
        self.projection_points
            .checked_add(self.unicode_points)
            .and_then(|n| {
                self.exceptions
                    .checked_mul(2)
                    .and_then(|e| n.checked_add(e))
            })
            .ok_or(Error::Budget)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Cancelled,
    Work,
    Invalid,
    Budget,
    UnsupportedRun(u32),
}
pub type Result<T> = std::result::Result<T, Error>;
impl From<crate::text_engine::Error> for Error {
    fn from(e: crate::text_engine::Error) -> Self {
        match e {
            crate::text_engine::Error::Cancelled => Self::Cancelled,
            crate::text_engine::Error::Work => Self::Work,
            crate::text_engine::Error::Invalid => Self::Invalid,
        }
    }
}
impl From<crate::document_model::Error> for Error {
    fn from(e: crate::document_model::Error) -> Self {
        if e.kind == crate::document_model::ErrorKind::Cancelled {
            Self::Cancelled
        } else {
            Self::Budget
        }
    }
}
pub(crate) const DOCUMENT_FRAME_WORK: usize = 8 * 1024 * 1024;

pub struct DocumentWork<'a>(Work<'a>, Option<(Identity, usize)>);
impl<'a> DocumentWork<'a> {
    pub fn frame(cancel: &'a AtomicBool) -> Self {
        Self(Work::permit(cancel, DOCUMENT_FRAME_WORK), None)
    }
    pub fn bounded(cancel: &'a AtomicBool, limit: usize) -> Result<Self> {
        if limit > DOCUMENT_FRAME_WORK {
            Err(Error::Work)
        } else {
            Ok(Self(Work::permit(cancel, limit), None))
        }
    }
    pub fn used(&self) -> usize {
        self.0.used()
    }
    pub fn counters(&self) -> [usize; 5] {
        let c = &self.0.counters;
        [
            c.supplied,
            c.context,
            c.advanced,
            c.width_reserved,
            c.projection,
        ]
    }
    pub(crate) fn preparation(cancel: &'a AtomicBool) -> Self {
        Self(Work::preparation(cancel), None)
    }
    pub(crate) fn admit_structure(&mut self, n: usize) -> Result<()> {
        self.admit(n)
    }
    fn admit(&mut self, n: usize) -> Result<()> {
        self.0.admit(n).map_err(Into::into)
    }
    fn record(&mut self, identity: &Identity, run: usize) -> Result<bool> {
        self.record_size(
            identity,
            run,
            std::mem::size_of::<crate::document_model::Run>(),
        )
    }
    fn record_size(&mut self, identity: &Identity, run: usize, bytes: usize) -> Result<bool> {
        let cached = self
            .1
            .as_ref()
            .is_some_and(|(id, r)| id.same(identity) && *r == run);
        self.admit(if cached { 1 } else { bytes })?;
        if !cached {
            self.1 = Some((identity.clone(), run));
        }
        Ok(!cached)
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Raw {
    run: usize,
    byte: usize,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Position {
    raw: Raw,
    part: usize,
    offset: usize,
}
#[derive(Clone, Copy)]
struct Scalar {
    raw: Raw,
    raw_end: usize,
    ch: char,
    next: Raw,
    synthetic: bool,
}
#[derive(Clone, Copy)]
struct Checkpoint {
    pos: Position,
}
struct Chunk {
    bytes: [u8; CHUNK],
    len: usize,
    start: usize,
    start_pos: Position,
    end_pos: Position,
    prev_pos: Position,
}
impl Chunk {
    fn text(&self) -> &str {
        std::str::from_utf8(&self.bytes[..self.len])
            .expect("projection consists of complete UTF8 scalars")
    }
    fn end(&self) -> usize {
        self.start
            .checked_add(self.len)
            .expect("bounded chunk endpoint")
    }
}
#[derive(Clone, Copy)]
#[allow(dead_code)] // TextRef consumer is wired in the subsequent renderer/inline cut.
enum PayloadSelection {
    Runs,
    TextRef(u32),
}
pub struct SegmentedText {
    selection: PayloadSelection,
    document: Arc<Document>,
    runs: Range<usize>,
    identity: Identity,
    len: usize,
    preparation_work: usize,
    preparation_counters: [usize; 5],
    checkpoints: ChargedVec<Checkpoint>,
    _scratch: Charge,
}
impl SegmentedText {
    pub fn prepare(
        document: Arc<Document>,
        domain: u32,
        run_ordinals: Range<usize>,
        budget: &ConsumerBudget,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        Self::prepare_with_profile(
            document,
            domain,
            run_ordinals,
            budget,
            cancel,
            IndexProfile::DEFAULT,
        )
    }
    pub fn prepare_with_profile(
        document: Arc<Document>,
        domain: u32,
        run_ordinals: Range<usize>,
        budget: &ConsumerBudget,
        cancel: &AtomicBool,
        profile: IndexProfile,
    ) -> Result<Self> {
        let profile = profile.checked()?;
        if !budget.belongs_to(&document) {
            return Err(Error::Invalid);
        }
        let d = document
            .domains()
            .get(domain as usize)
            .ok_or(Error::Invalid)?;
        if run_ordinals.start > run_ordinals.end || run_ordinals.end > d.run_count() as usize {
            return Err(Error::Invalid);
        }
        let first = d.first_run() as usize;
        let runs = first
            .checked_add(run_ordinals.start)
            .ok_or(Error::Invalid)?
            ..first.checked_add(run_ordinals.end).ok_or(Error::Invalid)?;
        Self::prepare_selection(
            document,
            runs,
            PayloadSelection::Runs,
            budget,
            cancel,
            profile,
        )
    }
    fn prepare_selection(
        document: Arc<Document>,
        runs: Range<usize>,
        selection: PayloadSelection,
        budget: &ConsumerBudget,
        cancel: &AtomicBool,
        profile: IndexProfile,
    ) -> Result<Self> {
        if !budget.belongs_to(&document) {
            return Err(Error::Invalid);
        }
        let scratch = budget.reserve(
            2 * CHUNK
                + 8 * std::mem::size_of::<Point>()
                + std::mem::size_of::<Self>()
                + std::mem::size_of::<SegmentedIndex>()
                + std::mem::size_of::<unicode_width_adapter::WidthState>()
                + 256,
        )?;
        let mut result = Self {
            selection,
            document,
            runs: runs.clone(),
            identity: Identity::new(),
            len: 0,
            preparation_work: 0,
            preparation_counters: [0; 5],
            checkpoints: ChargedVec::new(budget, 0)?,
            _scratch: scratch,
        };
        let mut w = DocumentWork(Work::preparation(cancel), None);
        let start = Position {
            raw: Raw {
                run: runs.start,
                byte: 0,
            },
            ..Position::default()
        };
        let mut at = start;
        while let Some((_, next)) = result.character(at, &mut w)? {
            at = next;
        }
        result.len = at.offset;
        let stride = result
            .len
            .div_ceil(profile.projection_points - 1)
            .max(CHUNK);
        result.checkpoints = ChargedVec::new_points(
            budget,
            result
                .len
                .div_ceil(stride)
                .saturating_add(1)
                .min(profile.projection_points),
            1,
        )?;
        result.checkpoints.push(Checkpoint { pos: start })?;
        at = start;
        let mut threshold = stride;
        while let Some((_, next)) = result.character(at, &mut w)? {
            at = next;
            if at.offset >= threshold && at.offset < result.len {
                result.checkpoints.push(Checkpoint { pos: at })?;
                threshold = at.offset.saturating_add(stride);
            }
        }
        w.admit(0)?;
        result.preparation_work = w.used();
        result.preparation_counters = w.counters();
        Ok(result)
    }
    pub fn preparation_work(&self) -> usize {
        self.preparation_work
    }
    pub fn preparation_counters(&self) -> [usize; 5] {
        self.preparation_counters
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn document(&self) -> &Arc<Document> {
        &self.document
    }
    fn payload<'s>(&'s self, ordinal: usize, w: &mut DocumentWork<'_>) -> Result<(&'s str, bool)> {
        match self.selection {
            PayloadSelection::TextRef(id) => {
                if ordinal != 0 {
                    return Err(Error::Invalid);
                }
                w.record_size(
                    &self.identity,
                    ordinal,
                    std::mem::size_of::<crate::document_model::TextRef>(),
                )?;
                Ok((self.document.text(id).map_err(|_| Error::Invalid)?, false))
            }
            PayloadSelection::Runs => {
                let run = self.document.runs().get(ordinal).ok_or(Error::Invalid)?;
                let entering = w.record(&self.identity, ordinal)?;
                match run.kind() {
                    0 | 1 | 4 | 6 | 7 => {
                        if entering {
                            w.admit(std::mem::size_of::<crate::document_model::TextRef>())?;
                        }
                        Ok((
                            self.document.text(run.text()).map_err(|_| Error::Invalid)?,
                            false,
                        ))
                    }
                    2 => Ok((" ", true)),
                    3 => Ok(("\n", true)),
                    kind => Err(Error::UnsupportedRun(kind)),
                }
            }
        }
    }
    fn raw_scalar(&self, mut at: Raw, w: &mut DocumentWork<'_>) -> Result<Option<Scalar>> {
        loop {
            w.admit(0)?;
            if at.run >= self.runs.end {
                return Ok(None);
            }
            let (text, synthetic) = self.payload(at.run, w)?;
            if at.byte == text.len() {
                at = Raw {
                    run: at.run.checked_add(1).ok_or(Error::Invalid)?,
                    byte: 0,
                };
                continue;
            }
            let ch = text
                .get(at.byte..)
                .ok_or(Error::Invalid)?
                .chars()
                .next()
                .ok_or(Error::Invalid)?;
            w.0.project(ch.len_utf8())?;
            return Ok(Some(Scalar {
                raw: at,
                raw_end: at.byte.checked_add(ch.len_utf8()).ok_or(Error::Invalid)?,
                ch,
                next: Raw {
                    run: at.run,
                    byte: at.byte.checked_add(ch.len_utf8()).ok_or(Error::Invalid)?,
                },
                synthetic,
            }));
        }
    }
    fn projection(
        &self,
        s: Scalar,
        w: &mut DocumentWork<'_>,
        bytes: &mut [u8; 8],
    ) -> Result<usize> {
        let next = if s.ch == '\r' {
            self.raw_scalar(s.next, w)?
                .filter(|s| !s.synthetic)
                .map(|s| s.ch)
        } else {
            None
        };
        Ok(text_projection::scalar(s.ch, next, bytes).len())
    }
    fn character(
        &self,
        at: Position,
        w: &mut DocumentWork<'_>,
    ) -> Result<Option<(char, Position)>> {
        let Some(s) = self.raw_scalar(at.raw, w)? else {
            return Ok(None);
        };
        let mut bytes = [0; 8];
        let len = self.projection(s, w, &mut bytes)?;
        let part = std::str::from_utf8(&bytes[..len])
            .map_err(|_| Error::Invalid)?
            .get(at.part..)
            .ok_or(Error::Invalid)?;
        let ch = part.chars().next().ok_or(Error::Invalid)?;
        let end = at.part.checked_add(ch.len_utf8()).ok_or(Error::Invalid)?;
        #[cfg(test)]
        if tests::OBSERVE.load(std::sync::atomic::Ordering::Relaxed) {
            tests::PROGRESS.fetch_add(ch.len_utf8(), std::sync::atomic::Ordering::AcqRel);
        }
        Ok(Some((
            ch,
            Position {
                raw: if end == len { s.next } else { s.raw },
                part: if end == len { 0 } else { end },
                offset: at.offset.checked_add(ch.len_utf8()).ok_or(Error::Invalid)?,
            },
        )))
    }
    fn locate(&self, offset: usize, w: &mut DocumentWork<'_>) -> Result<Position> {
        if offset > self.len {
            return Err(Error::Invalid);
        }
        let points = self.checkpoints.as_slice();
        let i = charged_upper(points.len(), w, std::mem::size_of::<Checkpoint>(), |i| {
            points[i].pos.offset <= offset
        })?
        .saturating_sub(1);
        let mut at = points[i].pos;
        while at.offset < offset {
            let (_, next) = self.character(at, w)?.ok_or(Error::Invalid)?;
            if next.offset > offset {
                return Err(Error::Invalid);
            }
            at = next;
        }
        Ok(at)
    }
    fn chunk(&self, start: usize, end: usize, w: &mut DocumentWork<'_>) -> Result<Chunk> {
        let at = self.locate(start, w)?;
        self.chunk_at(at, end, w)
    }
    fn chunk_at(&self, mut at: Position, end: usize, w: &mut DocumentWork<'_>) -> Result<Chunk> {
        if at.offset > end || end > self.len || end - at.offset > CHUNK {
            return Err(Error::Invalid);
        }
        let mut out = Chunk {
            bytes: [0; CHUNK],
            len: 0,
            start: at.offset,
            start_pos: at,
            end_pos: at,
            prev_pos: at,
        };
        while at.offset < end {
            let (ch, next) = self.character(at, w)?.ok_or(Error::Invalid)?;
            if next.offset > end {
                break;
            }
            ch.encode_utf8(&mut out.bytes[out.len..]);
            out.len = out.len.checked_add(ch.len_utf8()).ok_or(Error::Invalid)?;
            out.prev_pos = at;
            at = next;
        }
        out.end_pos = at;
        w.admit(out.len)?;
        Ok(out)
    }
    fn context(&self, end: usize, w: &mut DocumentWork<'_>) -> Result<Chunk> {
        let wanted = end.saturating_sub(CHUNK);
        let points = self.checkpoints.as_slice();
        let i = charged_upper(points.len(), w, std::mem::size_of::<Checkpoint>(), |i| {
            points[i].pos.offset <= wanted
        })?
        .saturating_sub(1);
        let mut at = points[i].pos;
        while at.offset < wanted {
            at = self.character(at, w)?.ok_or(Error::Invalid)?.1;
        }
        self.chunk(at.offset, end, w)
    }
    fn next_boundary(
        &self,
        cursor: &mut GraphemeCursor,
        position: &mut Position,
        previous: &mut Position,
        hint: usize,
        adaptive: bool,
        w: &mut DocumentWork<'_>,
    ) -> Result<Option<usize>> {
        let mut at = *position;
        let mut prev = *previous;
        let mut forward_hint = hint;
        loop {
            let end = (at.offset.saturating_add(forward_hint.clamp(4, CHUNK - 4))).min(self.len);
            let chunk = self.chunk_at(prev, end, w)?;
            w.0.counters.supplied =
                w.0.counters
                    .supplied
                    .checked_add(chunk.len)
                    .ok_or(Error::Work)?;
            let before = cursor.cur_cursor();
            let result = cursor.next_boundary(chunk.text(), chunk.start);
            w.0.counters.advanced =
                w.0.counters
                    .advanced
                    .checked_add(cursor.cur_cursor().saturating_sub(before))
                    .ok_or(Error::Work)?;
            w.admit(0)?;
            match result {
                Ok(v) => {
                    if let Some(boundary) = v {
                        let mut pos = chunk.start_pos;
                        let mut last = pos;
                        while pos.offset < boundary {
                            last = pos;
                            pos = self.character(pos, w)?.ok_or(Error::Invalid)?.1;
                        }
                        if pos.offset != boundary {
                            return Err(Error::Invalid);
                        }
                        *position = pos;
                        *previous = last;
                    }
                    return Ok(v);
                }
                Err(GraphemeIncomplete::NextChunk) => {
                    if chunk.end() <= at.offset {
                        return Err(Error::Invalid);
                    }
                    at = chunk.end_pos;
                    prev = chunk.prev_pos;
                    if adaptive && at.offset.saturating_sub(position.offset) >= 256 {
                        forward_hint = CHUNK - 4;
                    }
                }
                Err(GraphemeIncomplete::PreContext(n)) => {
                    let context = self.context(n, w)?;
                    w.0.counters.context =
                        w.0.counters
                            .context
                            .checked_add(context.len)
                            .ok_or(Error::Work)?;
                    cursor.provide_context(context.text(), context.start);
                }
                _ => return Err(Error::Invalid),
            }
        }
    }

    pub(crate) fn locate_raw(
        &self,
        run: u32,
        byte: usize,
        w: &mut DocumentWork<'_>,
    ) -> Result<usize> {
        let target = Raw {
            run: run as usize,
            byte,
        };
        if !matches!(self.selection, PayloadSelection::Runs) {
            return Err(Error::Invalid);
        }
        if !self.runs.contains(&target.run) {
            return Err(Error::Invalid);
        }
        let record = &self.document.runs()[target.run];
        w.admit_structure(std::mem::size_of_val(record))?;
        let text = match record.kind() {
            0 | 1 | 4 | 6 | 7 => self
                .document
                .text(record.text())
                .map_err(|_| Error::Invalid)?,
            2 => " ",
            3 => "\n",
            k => return Err(Error::UnsupportedRun(k)),
        };
        if byte > text.len() || !text.is_char_boundary(byte) {
            return Err(Error::Invalid);
        }
        let points = self.checkpoints.as_slice();
        let i = charged_upper(points.len(), w, std::mem::size_of::<Checkpoint>(), |i| {
            let p = points[i].pos;
            (p.raw.run, p.raw.byte, p.part) <= (target.run, target.byte, 0)
        })?
        .saturating_sub(1);
        let mut at = points[i].pos;
        loop {
            // Cursor positions lazily retain the exhausted previous Run. Normalize its
            // boundary BEFORE consuming the next scalar, or seeking B:0 would skip B.
            while at.part == 0 && at.raw.run < target.run {
                w.admit_structure(
                    std::mem::size_of::<crate::document_model::Run>()
                        + std::mem::size_of::<crate::document_model::TextRef>(),
                )?;
                let r = &self.document.runs()[at.raw.run];
                let length = if matches!(r.kind(), 2 | 3) {
                    1
                } else {
                    self.document
                        .text(r.text())
                        .map_err(|_| Error::Invalid)?
                        .len()
                };
                if at.raw.byte != length {
                    break;
                }
                at.raw.run = at.raw.run.checked_add(1).ok_or(Error::Invalid)?;
                at.raw.byte = 0;
            }
            if (at.raw.run, at.raw.byte, at.part) >= (target.run, target.byte, 0) {
                break;
            }
            at = self.character(at, w)?.ok_or(Error::Invalid)?.1;
        }
        if (at.raw.run, at.raw.byte, at.part) == (target.run, target.byte, 0) {
            Ok(at.offset)
        } else {
            Err(Error::Invalid)
        }
    }
    pub(crate) fn locate_visual(
        &self,
        run: u32,
        byte: usize,
        part: u8,
        work: &mut DocumentWork<'_>,
    ) -> Result<usize> {
        validate_visual_part(&self.document, run, byte, part, work)?;
        self.locate_raw(run, byte, work)?
            .checked_add(usize::from(part))
            .ok_or(Error::Invalid)
    }
    pub fn visit_projected(
        &self,
        range: Range<usize>,
        work: &mut DocumentWork<'_>,
        visit: impl FnMut(&str) -> Result<()>,
    ) -> Result<()> {
        if range.start > range.end || range.end > self.len {
            return Err(Error::Invalid);
        }
        if range.is_empty() {
            return work.admit(0);
        }
        let at = self.locate(range.start, work)?;
        self.visit_projected_at(at, range.end, work, visit)
            .map(|_| ())
    }
    fn visit_projected_at(
        &self,
        mut at: Position,
        end: usize,
        work: &mut DocumentWork<'_>,
        mut visit: impl FnMut(&str) -> Result<()>,
    ) -> Result<Position> {
        if at.offset > end || end > self.len {
            return Err(Error::Invalid);
        }
        while at.offset < end {
            let chunk = self.chunk_at(at, at.offset.saturating_add(CHUNK).min(end), work)?;
            if chunk.len == 0 {
                return Err(Error::Invalid);
            }
            visit(chunk.text())?;
            at = chunk.end_pos;
        }
        work.admit(0)?;
        Ok(at)
    }
    fn previous_character(
        &self,
        at: Position,
        w: &mut DocumentWork<'_>,
    ) -> Result<(char, Position)> {
        if at.offset == 0 {
            return Err(Error::Invalid);
        }
        let mut raw = at.raw;
        let scalar = if at.part > 0 {
            self.raw_scalar(raw, w)?.ok_or(Error::Invalid)?
        } else {
            loop {
                w.admit(0)?;
                if raw.run >= self.runs.end || raw.byte == 0 {
                    if raw.run <= self.runs.start {
                        return Err(Error::Invalid);
                    }
                    raw.run -= 1;
                    raw.byte = self.payload(raw.run, w)?.0.len();
                    if raw.byte == 0 {
                        continue;
                    }
                }
                let text = self.payload(raw.run, w)?.0;
                let (byte, _) = text
                    .get(..raw.byte)
                    .ok_or(Error::Invalid)?
                    .char_indices()
                    .next_back()
                    .ok_or(Error::Invalid)?;
                raw.byte = byte;
                break self.raw_scalar(raw, w)?.ok_or(Error::Invalid)?;
            }
        };
        let mut bytes = [0; 8];
        let len = self.projection(scalar, w, &mut bytes)?;
        let end = if at.part > 0 { at.part } else { len };
        let text = std::str::from_utf8(&bytes[..end]).map_err(|_| Error::Invalid)?;
        let (part, ch) = text.char_indices().next_back().ok_or(Error::Invalid)?;
        Ok((
            ch,
            Position {
                raw: scalar.raw,
                part,
                offset: at.offset - ch.len_utf8(),
            },
        ))
    }
    fn width(
        &self,
        range: Range<usize>,
        column: usize,
        mut at: Position,
        w: &mut DocumentWork<'_>,
    ) -> Result<usize> {
        if range.len() == 1 {
            let (ch, _) = self.previous_character(at, w)?;
            if ch == '\t' {
                return Ok(8 - column % 8);
            }
        }
        let mut state = unicode_width_adapter::WidthState::new();
        while at.offset > range.start {
            let mut bytes = [0; CHUNK];
            let mut left = CHUNK;
            while at.offset > range.start {
                let (ch, previous) = self.previous_character(at, w)?;
                if ch.len_utf8() > left {
                    break;
                }
                left -= ch.len_utf8();
                ch.encode_utf8(&mut bytes[left..]);
                at = previous;
            }
            let text = std::str::from_utf8(&bytes[left..]).map_err(|_| Error::Invalid)?;
            let mut failure = None;
            if !state.push_reverse_block(text, |n| match w.admit(n) {
                Ok(()) => {
                    let Some(total) = w.0.counters.width_reserved.checked_add(n) else {
                        failure = Some(Error::Work);
                        return false;
                    };
                    w.0.counters.width_reserved = total;
                    true
                }
                Err(e) => {
                    failure = Some(e);
                    false
                }
            }) {
                return Err(failure.unwrap_or(Error::Cancelled));
            }
        }
        Ok(state.finish())
    }
    pub fn visit_coverage(
        &self,
        range: Range<usize>,
        work: &mut DocumentWork<'_>,
        visit: impl FnMut(Coverage) -> Result<()>,
    ) -> Result<()> {
        if !matches!(self.selection, PayloadSelection::Runs) {
            return Err(Error::Invalid);
        }
        if range.start > range.end || range.end > self.len {
            return Err(Error::Invalid);
        }
        let at = self.locate(range.start, work)?;
        self.visit_coverage_at(range, at, work, visit).map(|_| ())
    }
    fn visit_coverage_at(
        &self,
        range: Range<usize>,
        mut at: Position,
        work: &mut DocumentWork<'_>,
        mut visit: impl FnMut(Coverage) -> Result<()>,
    ) -> Result<Position> {
        if !matches!(self.selection, PayloadSelection::Runs)
            || at.offset != range.start
            || range.start > range.end
            || range.end > self.len
        {
            return Err(Error::Invalid);
        }
        let mut pending: Option<Coverage> = None;
        while at.offset < range.end {
            let scalar = self.raw_scalar(at.raw, work)?.ok_or(Error::Invalid)?;
            let run = &self.document.runs()[scalar.raw.run];
            let mut bytes = [0; 8];
            let len = self.projection(scalar, work, &mut bytes)?;
            let control_projected = !scalar.synthetic && len != scalar.ch.len_utf8();
            let relation = if scalar.synthetic {
                Relation::Synthetic
            } else {
                let t = &self.document.text_refs()[run.text() as usize];
                if t.kind() == 1 {
                    Relation::EnclosingToken
                } else if control_projected {
                    Relation::ControlProjection
                } else {
                    Relation::ExactSource
                }
            };
            let text_ref = if scalar.synthetic {
                None
            } else {
                Some(&self.document.text_refs()[run.text() as usize])
            };
            let text_origin = text_ref.map(|t| t.origin());
            let source = text_ref
                .map(|t| -> Result<Range<u32>> {
                    if t.kind() == 0 {
                        let start = u32::try_from(scalar.raw.byte).map_err(|_| Error::Invalid)?;
                        let end = u32::try_from(scalar.raw_end).map_err(|_| Error::Invalid)?;
                        Ok(t.start().checked_add(start).ok_or(Error::Invalid)?
                            ..t.start().checked_add(end).ok_or(Error::Invalid)?)
                    } else {
                        let o = &self.document.origins()[t.origin() as usize];
                        Ok(o.start()..o.end())
                    }
                })
                .transpose()?;
            let coverage = Coverage {
                control_projected,
                text_origin,
                source,
                run: run.id(),
                local: scalar.raw.byte..scalar.raw_end,
                origin: run.origin(),
                styles: run.styles(),
                link: if run.link() == NONE {
                    None
                } else {
                    Some(run.link())
                },
                relation,
            };
            if let Some(last) = pending.as_mut() {
                if last.run == coverage.run
                    && last.local.end == coverage.local.start
                    && last.relation == coverage.relation
                    && last.control_projected == coverage.control_projected
                    && last.origin == coverage.origin
                    && last.text_origin == coverage.text_origin
                {
                    last.local.end = coverage.local.end;
                    if (last.relation == Relation::ExactSource
                        || last.relation == Relation::ControlProjection)
                        && let (Some(a), Some(b)) = (&mut last.source, &coverage.source)
                    {
                        a.end = b.end;
                    }
                } else {
                    visit(pending.take().unwrap())?;
                    pending = Some(coverage);
                }
            } else {
                pending = Some(coverage);
            }
            // A projected scalar may span several glyphs (C1 spelling); its entire raw scalar is retained as provenance.
            let remaining = len - at.part;
            let consumed = remaining.min(range.end - at.offset);
            let endpart = at.part.checked_add(consumed).ok_or(Error::Invalid)?;
            if !std::str::from_utf8(&bytes[..len])
                .map_err(|_| Error::Invalid)?
                .is_char_boundary(endpart)
            {
                return Err(Error::Invalid);
            }
            at = Position {
                raw: if endpart == len {
                    scalar.next
                } else {
                    scalar.raw
                },
                part: if endpart == len { 0 } else { endpart },
                offset: at.offset.checked_add(consumed).ok_or(Error::Invalid)?,
            };
        }
        work.admit(0)?;
        if let Some(last) = pending {
            visit(last)?;
        }
        work.admit(0)?;
        Ok(at)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relation {
    ExactSource,
    ControlProjection,
    EnclosingToken,
    Synthetic,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Coverage {
    pub control_projected: bool,
    pub text_origin: Option<u32>,
    pub source: Option<Range<u32>>,
    pub run: u32,
    pub local: Range<usize>,
    pub origin: u32,
    pub styles: u32,
    pub link: Option<u32>,
    pub relation: Relation,
}
#[derive(Clone)]
pub struct Point {
    identity: Identity,
    offset: usize,
    line: usize,
    column: usize,
    cursor: GraphemeCursor,
    position: Position,
    previous: Position,
}
impl Point {
    pub fn offset(&self) -> usize {
        self.offset
    }
    pub fn line(&self) -> usize {
        self.line
    }
    pub fn column(&self) -> usize {
        self.column
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unit {
    pub projected: Range<usize>,
    pub line: usize,
    pub column: usize,
    pub width: usize,
    pub newline: bool,
    pub oversized: bool,
}
struct Exception {
    before: usize,
    unit: Unit,
    after: Point,
}
#[cfg(test)]
thread_local! {
    static RESOLVED_PRECOMMIT: std::cell::RefCell<Option<Box<dyn FnOnce()>>> =
        const { std::cell::RefCell::new(None) };
}
struct PreparedStep {
    next: Point,
    unit: Unit,
}
pub(crate) struct ResolvedUnit<'i> {
    index: &'i SegmentedIndex,
    start: Position,
    end: Position,
    unit: Unit,
}
impl ResolvedUnit<'_> {
    pub(crate) fn unit(&self) -> &Unit {
        &self.unit
    }
    pub(crate) fn visit_projected(
        &self,
        work: &mut DocumentWork<'_>,
        visit: impl FnMut(&str) -> Result<()>,
    ) -> Result<()> {
        let end =
            self.index
                .text
                .visit_projected_at(self.start, self.unit.projected.end, work, visit)?;
        if end != self.end {
            return Err(Error::Invalid);
        }
        work.admit(0)
    }
    pub(crate) fn visit_coverage(
        &self,
        work: &mut DocumentWork<'_>,
        visit: impl FnMut(Coverage) -> Result<()>,
    ) -> Result<()> {
        let end = self.index.text.visit_coverage_at(
            self.unit.projected.clone(),
            self.start,
            work,
            visit,
        )?;
        if end != self.end {
            return Err(Error::Invalid);
        }
        work.admit(0)
    }
}

pub struct SegmentedIndex {
    text: SegmentedText,
    points: ChargedVec<Point>,
    exceptions: ChargedVec<Exception>,
    max_width: usize,
    lines: usize,
    preparation_work: usize,
    preparation_counters: [usize; 5],
}
impl SegmentedIndex {
    pub(crate) fn raw_anchor_with_work(
        &self,
        p: &Point,
        w: &mut DocumentWork<'_>,
    ) -> Result<(u32, usize)> {
        if !matches!(self.text.selection, PayloadSelection::Runs) {
            return Err(Error::Invalid);
        }
        if !p.identity.same(&self.text.identity) {
            return Err(Error::Invalid);
        }
        if let Some(s) = self.text.raw_scalar(p.position.raw, w)? {
            return Ok((
                u32::try_from(s.raw.run).map_err(|_| Error::Invalid)?,
                s.raw.byte,
            ));
        }
        let run = self.text.runs.end.checked_sub(1).ok_or(Error::Invalid)?;
        w.admit_structure(std::mem::size_of::<crate::document_model::Run>())?;
        let r = &self.text.document.runs()[run];
        let byte = if matches!(r.kind(), 2 | 3) {
            0
        } else {
            w.admit_structure(std::mem::size_of::<crate::document_model::TextRef>())?;
            self.text
                .document
                .text(r.text())
                .map_err(|_| Error::Invalid)?
                .len()
        };
        Ok((u32::try_from(run).map_err(|_| Error::Invalid)?, byte))
    }
    pub(crate) fn visual_anchor_with_work(
        &self,
        point: &Point,
        work: &mut DocumentWork<'_>,
    ) -> Result<(u32, usize, u8)> {
        let (run, byte) = self.raw_anchor_with_work(point, work)?;
        let part = u8::try_from(point.position.part).map_err(|_| Error::Invalid)?;
        validate_visual_part(&self.text.document, run, byte, part, work)?;
        Ok((run, byte, part))
    }
    pub fn prepare(
        text: SegmentedText,
        budget: &ConsumerBudget,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        Self::prepare_with_profile(text, budget, cancel, IndexProfile::DEFAULT)
    }
    pub fn prepare_with_profile(
        text: SegmentedText,
        budget: &ConsumerBudget,
        cancel: &AtomicBool,
        profile: IndexProfile,
    ) -> Result<Self> {
        let profile = profile.checked()?;
        if !budget.belongs_to(&text.document) {
            return Err(Error::Invalid);
        }
        let stride = text.len.div_ceil(profile.unicode_points - 1).max(CHUNK);
        let capacity = text
            .len
            .div_ceil(stride)
            .saturating_add(1)
            .min(profile.unicode_points);
        let exception_capacity = (text.len / (SYMBOL_BYTES + 1)).min(profile.exceptions);
        let mut result = Self {
            text,
            points: ChargedVec::new_points(budget, capacity, 1)?,
            exceptions: ChargedVec::new_points(budget, exception_capacity, 2)?,
            max_width: 0,
            lines: 1,
            preparation_work: 0,
            preparation_counters: [0; 5],
        };
        let mut point = result.start();
        result.points.push(point.clone())?;
        let stride = result
            .text
            .len
            .div_ceil(profile.unicode_points - 1)
            .max(CHUNK);
        let mut threshold = stride;
        let mut work = DocumentWork(Work::preparation(cancel), None);
        while let Some(unit) = result.compute(&mut point, 4, true, &mut work)? {
            result.max_width = result
                .max_width
                .max(unit.column.checked_add(unit.width).ok_or(Error::Invalid)?);
            if unit.oversized {
                result.exceptions.push(Exception {
                    before: unit.projected.start,
                    unit: unit.clone(),
                    after: point.clone(),
                })?;
            }
            if point.offset >= threshold && point.offset < result.text.len {
                result.points.push(point.clone())?;
                threshold = point.offset.saturating_add(stride);
            }
        }
        result.lines = point.line.checked_add(1).ok_or(Error::Invalid)?;
        work.admit(0)?;
        result.preparation_work = work.used();
        result.preparation_counters = work.counters();
        Ok(result)
    }
    pub fn preparation_work(&self) -> usize {
        self.preparation_work
    }
    pub fn preparation_counters(&self) -> [usize; 5] {
        self.preparation_counters
    }
    pub fn text(&self) -> &SegmentedText {
        &self.text
    }
    pub fn max_width(&self) -> usize {
        self.max_width
    }
    pub fn line_count(&self) -> usize {
        self.lines
    }
    pub fn start(&self) -> Point {
        Point {
            identity: self.text.identity.clone(),
            offset: 0,
            line: 0,
            column: 0,
            cursor: GraphemeCursor::new(0, self.text.len, true),
            position: Position {
                raw: Raw {
                    run: self.text.runs.start,
                    byte: 0,
                },
                ..Position::default()
            },
            previous: Position {
                raw: Raw {
                    run: self.text.runs.start,
                    byte: 0,
                },
                ..Position::default()
            },
        }
    }
    fn prepare_compute(
        &self,
        point: &Point,
        hint: usize,
        adaptive: bool,
        w: &mut DocumentWork<'_>,
    ) -> Result<Option<PreparedStep>> {
        if !point.identity.same(&self.text.identity) {
            return Err(Error::Invalid);
        }
        if point.offset == self.text.len {
            w.admit(0)?;
            return Ok(None);
        }
        let mut next = point.clone();
        let Some(end) = self.text.next_boundary(
            &mut next.cursor,
            &mut next.position,
            &mut next.previous,
            hint,
            adaptive,
            w,
        )?
        else {
            return Err(Error::Invalid);
        };
        let range = point.offset..end;
        let initial = self.text.chunk_at(
            point.position,
            range.start.saturating_add(4).min(range.end),
            w,
        )?;
        let newline = matches!(initial.text(), "\n" | "\r\n") && range.len() <= 2;
        let width = if newline {
            0
        } else {
            self.text
                .width(range.clone(), point.column, next.position, w)?
        };
        let unit = Unit {
            projected: range.clone(),
            line: point.line,
            column: point.column,
            width,
            newline,
            oversized: range.len() > SYMBOL_BYTES,
        };
        next.offset = end;
        if newline {
            next.line = next.line.checked_add(1).ok_or(Error::Invalid)?;
            next.column = 0;
        } else {
            next.column = next.column.checked_add(width).ok_or(Error::Invalid)?;
        }
        Ok(Some(PreparedStep { next, unit }))
    }
    fn compute(
        &self,
        point: &mut Point,
        hint: usize,
        adaptive: bool,
        w: &mut DocumentWork<'_>,
    ) -> Result<Option<Unit>> {
        let prepared = self.prepare_compute(point, hint, adaptive, w)?;
        w.admit(0)?;
        Ok(prepared.map(|p| {
            *point = p.next;
            p.unit
        }))
    }
    fn prepare_step(
        &self,
        point: &Point,
        hint: usize,
        w: &mut DocumentWork<'_>,
    ) -> Result<Option<PreparedStep>> {
        if !point.identity.same(&self.text.identity) {
            return Err(Error::Invalid);
        }
        let exceptions = self.exceptions.as_slice();
        let i = charged_upper(exceptions.len(), w, std::mem::size_of::<Exception>(), |i| {
            exceptions[i].before < point.offset
        })?;
        if let Some(e) = exceptions.get(i).filter(|e| e.before == point.offset) {
            w.admit(std::mem::size_of::<Exception>())?;
            return Ok(Some(PreparedStep {
                next: e.after.clone(),
                unit: e.unit.clone(),
            }));
        }
        self.prepare_compute(point, hint, false, w)
    }

    pub fn step(
        &self,
        point: &mut Point,
        hint: usize,
        w: &mut DocumentWork<'_>,
    ) -> Result<Option<Unit>> {
        let prepared = self.prepare_step(point, hint, w)?;
        w.admit(0)?;
        Ok(prepared.map(|p| {
            *point = p.next;
            p.unit
        }))
    }
    fn prepare_resolved(
        &self,
        point: &Point,
        hint: usize,
        wrapper_bytes: usize,
        w: &mut DocumentWork<'_>,
    ) -> Result<Option<PreparedStep>> {
        w.admit_structure(std::mem::size_of::<Position>())?;
        if !point.identity.same(&self.text.identity)
            || point.offset != point.position.offset
            || point.cursor.cur_cursor() != point.offset
            || point.offset > self.text.len
        {
            return Err(Error::Invalid);
        }
        if point.offset == self.text.len {
            w.admit(0)?;
            return Ok(None);
        }
        w.admit_structure(std::mem::size_of::<ResolvedUnit<'_>>())?;
        w.admit_structure(wrapper_bytes)?;
        let p = self.prepare_step(point, hint, w)?.ok_or(Error::Invalid)?;
        if p.unit.projected.start != point.offset
            || p.unit.projected.end != p.next.offset
            || p.next.position.offset != p.next.offset
            || p.next.cursor.cur_cursor() != p.next.offset
            || p.next.offset <= point.offset
            || p.next.offset > self.text.len
            || !p.next.identity.same(&self.text.identity)
        {
            return Err(Error::Invalid);
        }
        #[cfg(test)]
        RESOLVED_PRECOMMIT.with(|hook| {
            if let Some(hook) = hook.borrow_mut().take() {
                hook();
            }
        });
        w.admit(0)?;
        Ok(Some(p))
    }
    pub(crate) fn step_resolved<'i>(
        &'i self,
        point: &mut Point,
        hint: usize,
        w: &mut DocumentWork<'_>,
    ) -> Result<Option<ResolvedUnit<'i>>> {
        let Some(p) = self.prepare_resolved(point, hint, 0, w)? else {
            return Ok(None);
        };
        let token = ResolvedUnit {
            index: self,
            start: point.position,
            end: p.next.position,
            unit: p.unit,
        };
        *point = p.next;
        Ok(Some(token))
    }

    pub(crate) fn seek_before_with_work(
        &self,
        offset: usize,
        w: &mut DocumentWork<'_>,
    ) -> Result<Point> {
        if offset > self.text.len {
            return Err(Error::Invalid);
        }
        let p = self.points.as_slice();
        let i = charged_upper(p.len(), w, std::mem::size_of::<Point>(), |i| {
            p[i].offset <= offset
        })?
        .saturating_sub(1);
        w.admit_structure(std::mem::size_of::<Point>())?;
        Ok(p[i].clone())
    }
    pub(crate) fn seek_line_with_work(
        &self,
        line: usize,
        w: &mut DocumentWork<'_>,
    ) -> Result<Point> {
        let p = self.points.as_slice();
        let i = charged_upper(p.len(), w, std::mem::size_of::<Point>(), |i| {
            p[i].line < line
        })?
        .saturating_sub(1);
        w.admit_structure(std::mem::size_of::<Point>())?;
        let mut at = p[i].clone();
        while at.line < line {
            if self.step(&mut at, 4096, w)?.is_none() {
                break;
            }
        }
        Ok(at)
    }
    pub(crate) fn seek_cell_with_work(
        &self,
        line: usize,
        column: usize,
        w: &mut DocumentWork<'_>,
    ) -> Result<Point> {
        let points = self.points.as_slice();
        let i = charged_upper(points.len(), w, std::mem::size_of::<Point>(), |i| {
            points[i].line < line || points[i].line == line && points[i].column <= column
        })?
        .saturating_sub(1);
        w.admit_structure(std::mem::size_of::<Point>())?;
        let mut at = points[i].clone();
        while at.line < line || at.line == line && at.column < column {
            if self.step(&mut at, 4096, w)?.is_none() {
                break;
            }
        }
        Ok(at)
    }
    pub fn seek_before(&self, offset: usize) -> Result<Point> {
        if offset > self.text.len {
            return Err(Error::Invalid);
        }
        let points = self.points.as_slice();
        let i = points
            .partition_point(|p| p.offset <= offset)
            .saturating_sub(1);
        Ok(points[i].clone())
    }
}

pub(crate) fn validate_visual_part(
    document: &Document,
    run: u32,
    byte: usize,
    part: u8,
    work: &mut DocumentWork<'_>,
) -> Result<()> {
    work.admit_structure(0)?;
    if part == 0 {
        return Ok(());
    }
    work.admit_structure(std::mem::size_of::<crate::document_model::Run>())?;
    let record = document.runs().get(run as usize).ok_or(Error::Invalid)?;
    if matches!(record.kind(), 2 | 3 | 5) {
        return Err(Error::Invalid);
    }
    work.admit_structure(std::mem::size_of::<crate::document_model::TextRef>() + 8)?;
    let text = document.text(record.text()).map_err(|_| Error::Invalid)?;
    let ch = text
        .get(byte..)
        .ok_or(Error::Invalid)?
        .chars()
        .next()
        .ok_or(Error::Invalid)?;
    if !matches!(ch, '\u{80}'..='\u{9f}') {
        return Err(Error::Invalid);
    }
    let mut bytes = [0; 8];
    let projected = text_projection::scalar(ch, None, &mut bytes);
    if usize::from(part) >= projected.len() || !projected.is_char_boundary(usize::from(part)) {
        return Err(Error::Invalid);
    }
    Ok(())
}

pub(crate) fn validate_ref_visual_part(
    document: &Document,
    id: u32,
    local: usize,
    part: u8,
    work: &mut DocumentWork<'_>,
) -> Result<()> {
    work.admit_structure(std::mem::size_of::<crate::document_model::TextRef>() + 8)?;
    let text = document.text(id).map_err(|_| Error::Invalid)?;
    if local > text.len() || !text.is_char_boundary(local) {
        return Err(Error::Invalid);
    }
    if part == 0 {
        return Ok(());
    }
    let ch = text
        .get(local..)
        .and_then(|s| s.chars().next())
        .ok_or(Error::Invalid)?;
    if !matches!(ch, '\u{80}'..='\u{9f}') {
        return Err(Error::Invalid);
    }
    let mut bytes = [0; 8];
    let projection = text_projection::scalar(ch, None, &mut bytes);
    if usize::from(part) >= projection.len() || !projection.is_char_boundary(usize::from(part)) {
        return Err(Error::Invalid);
    }
    Ok(())
}

fn charged_upper(
    n: usize,
    w: &mut DocumentWork<'_>,
    bytes: usize,
    mut check: impl FnMut(usize) -> bool,
) -> Result<usize> {
    let (mut lo, mut hi) = (0, n);
    while lo < hi {
        w.admit_structure(bytes)?;
        let mid = lo + (hi - lo) / 2;
        if check(mid) { lo = mid + 1 } else { hi = mid }
    }
    Ok(lo)
}
#[cfg(test)]
#[path = "document_text_tests.rs"]
mod tests;
/// A real TextRef payload selection; internal ordinal zero is a piece cursor,
/// never a fabricated Run, Domain, source range or Document.
#[allow(dead_code)] // Closed crate-private library pending consumer wiring.
pub(crate) struct TextRefIndex {
    id: u32,
    index: SegmentedIndex,
    _metadata: Charge,
}
#[allow(dead_code)] // Closed crate-private library pending consumer wiring.
pub(crate) struct RefCoverage {
    pub text_ref: u32,
    pub local: Range<usize>,
    pub source: Range<u32>,
    pub origin: u32,
    pub relation: Relation,
    pub control_projected: bool,
}
#[allow(dead_code)] // Closed crate-private library pending consumer wiring.
impl TextRefIndex {
    pub(crate) fn prepare(
        document: Arc<Document>,
        id: u32,
        budget: &ConsumerBudget,
        cancel: &AtomicBool,
        profile: IndexProfile,
    ) -> Result<Self> {
        let profile = profile.checked()?;
        if !budget.belongs_to(&document) {
            return Err(Error::Invalid);
        }
        if cancel.load(std::sync::atomic::Ordering::Acquire) {
            return Err(Error::Cancelled);
        }
        document
            .text_refs()
            .get(id as usize)
            .ok_or(Error::Invalid)?;
        document.text(id).map_err(|_| Error::Invalid)?;
        let metadata = budget.reserve(std::mem::size_of::<Self>())?;
        let text = SegmentedText::prepare_selection(
            document,
            0..1,
            PayloadSelection::TextRef(id),
            budget,
            cancel,
            profile,
        )?;
        let index = SegmentedIndex::prepare_with_profile(text, budget, cancel, profile)?;
        Ok(Self {
            id,
            index,
            _metadata: metadata,
        })
    }
    /// Provider-backed local/visual seek: ordinal zero remains internal to TextRef selection.
    pub(crate) fn seek_visual(
        &self,
        local: usize,
        part: u8,
        w: &mut DocumentWork<'_>,
    ) -> Result<Point> {
        let text = &self.index.text;
        let payload = text.document.text(self.id).map_err(|_| Error::Invalid)?;
        w.admit_structure(std::mem::size_of::<crate::document_model::TextRef>())?;
        if local > payload.len() || !payload.is_char_boundary(local) {
            return Err(Error::Invalid);
        }
        let points = text.checkpoints.as_slice();
        let i = charged_upper(points.len(), w, std::mem::size_of::<Checkpoint>(), |i| {
            (points[i].pos.raw.byte, points[i].pos.part) <= (local, usize::from(part))
        })?
        .saturating_sub(1);
        let mut at = points[i].pos;
        while (at.raw.byte, at.part) < (local, usize::from(part)) {
            at = text.character(at, w)?.ok_or(Error::Invalid)?.1;
        }
        if (at.raw.byte, at.part) != (local, usize::from(part)) {
            return Err(Error::Invalid);
        }
        let target = at.offset;
        let mut point = self.index.seek_before_with_work(target, w)?;
        while point.offset() < target {
            let before = point.clone();
            let unit = self
                .index
                .step(&mut point, 4096, w)?
                .ok_or(Error::Invalid)?;
            if unit.projected.end > target {
                point = before;
                break;
            }
        }
        Ok(point)
    }
    pub(crate) fn visual_local(
        &self,
        point: &Point,
        w: &mut DocumentWork<'_>,
    ) -> Result<(usize, u8)> {
        w.admit_structure(std::mem::size_of::<Point>())?;
        let p = point.position;
        if !matches!(self.index.text.selection,PayloadSelection::TextRef(id) if id==self.id) {
            return Err(Error::Invalid);
        }
        Ok((
            p.raw.byte,
            u8::try_from(p.part).map_err(|_| Error::Invalid)?,
        ))
    }
    pub(crate) fn id(&self) -> u32 {
        self.id
    }
    pub(crate) fn start(&self) -> Point {
        self.index.start()
    }
    pub(crate) fn len(&self) -> usize {
        self.index.text.len
    }
    pub(crate) fn step(
        &self,
        at: &mut Point,
        hint: usize,
        work: &mut DocumentWork<'_>,
    ) -> Result<Option<Unit>> {
        self.index.step(at, hint, work)
    }
    pub(crate) fn visit_projected(
        &self,
        range: Range<usize>,
        work: &mut DocumentWork<'_>,
        visit: impl FnMut(&str) -> Result<()>,
    ) -> Result<()> {
        self.index.text.visit_projected(range, work, visit)
    }
    pub(crate) fn visit_coverage(
        &self,
        range: Range<usize>,
        work: &mut DocumentWork<'_>,
        visit: impl FnMut(RefCoverage) -> Result<()>,
    ) -> Result<()> {
        let text = &self.index.text;
        if range.start > range.end || range.end > text.len {
            return Err(Error::Invalid);
        }
        let at = text.locate(range.start, work)?;
        self.visit_coverage_at(range, at, work, visit).map(|_| ())
    }
    fn visit_coverage_at(
        &self,
        range: Range<usize>,
        mut at: Position,
        work: &mut DocumentWork<'_>,
        mut visit: impl FnMut(RefCoverage) -> Result<()>,
    ) -> Result<Position> {
        let text = &self.index.text;
        if at.offset != range.start
            || range.start > range.end
            || range.end > text.len
            || !matches!(text.selection, PayloadSelection::TextRef(id) if id == self.id)
        {
            return Err(Error::Invalid);
        }
        work.admit_structure(std::mem::size_of::<crate::document_model::TextRef>())?;
        let record = &text.document.text_refs()[self.id as usize];
        while at.offset < range.end {
            let scalar = text.raw_scalar(at.raw, work)?.ok_or(Error::Invalid)?;
            let mut bytes = [0; 8];
            let len = text.projection(scalar, work, &mut bytes)?;
            let projected = len != scalar.ch.len_utf8();
            let relation = if record.kind() == 1 {
                Relation::EnclosingToken
            } else if projected {
                Relation::ControlProjection
            } else {
                Relation::ExactSource
            };
            let source = if record.kind() == 0 {
                let a = u32::try_from(scalar.raw.byte).map_err(|_| Error::Invalid)?;
                let b = u32::try_from(scalar.raw_end).map_err(|_| Error::Invalid)?;
                record.start().checked_add(a).ok_or(Error::Invalid)?
                    ..record.start().checked_add(b).ok_or(Error::Invalid)?
            } else {
                work.admit_structure(std::mem::size_of::<crate::document_model::Origin>())?;
                let origin = text
                    .document
                    .origins()
                    .get(record.origin() as usize)
                    .ok_or(Error::Invalid)?;
                origin.start()..origin.end()
            };
            work.admit_structure(std::mem::size_of::<RefCoverage>())?;
            visit(RefCoverage {
                text_ref: self.id,
                local: scalar.raw.byte..scalar.raw_end,
                source,
                origin: record.origin(),
                relation,
                control_projected: projected,
            })?;
            let consumed = (len - at.part).min(range.end - at.offset);
            let part = at.part.checked_add(consumed).ok_or(Error::Invalid)?;
            if !std::str::from_utf8(&bytes[..len])
                .map_err(|_| Error::Invalid)?
                .is_char_boundary(part)
            {
                return Err(Error::Invalid);
            }
            at = Position {
                raw: if part == len { scalar.next } else { scalar.raw },
                part: if part == len { 0 } else { part },
                offset: at.offset.checked_add(consumed).ok_or(Error::Invalid)?,
            };
        }
        work.admit(0)?;
        Ok(at)
    }
    pub(crate) fn step_resolved<'i>(
        &'i self,
        point: &mut Point,
        hint: usize,
        work: &mut DocumentWork<'_>,
    ) -> Result<Option<ResolvedRefUnit<'i>>> {
        if !matches!(self.index.text.selection, PayloadSelection::TextRef(id) if id == self.id) {
            return Err(Error::Invalid);
        }
        let extra = std::mem::size_of::<ResolvedRefUnit<'_>>()
            .checked_sub(std::mem::size_of::<ResolvedUnit<'_>>())
            .ok_or(Error::Invalid)?;
        let Some(p) = self.index.prepare_resolved(point, hint, extra, work)? else {
            return Ok(None);
        };
        let token = ResolvedRefUnit {
            owner: self,
            core: ResolvedUnit {
                index: &self.index,
                start: point.position,
                end: p.next.position,
                unit: p.unit,
            },
        };
        *point = p.next;
        Ok(Some(token))
    }
}

#[cfg(test)]
#[path = "document_text_ref_tests.rs"]
mod ref_tests;

#[allow(dead_code)] // TextRef consumers are integrated in a separate accepted increment.
pub(crate) struct ResolvedRefUnit<'i> {
    owner: &'i TextRefIndex,
    core: ResolvedUnit<'i>,
}
#[allow(dead_code)]
impl ResolvedRefUnit<'_> {
    pub(crate) fn unit(&self) -> &Unit {
        self.core.unit()
    }
    pub(crate) fn visit_projected(
        &self,
        w: &mut DocumentWork<'_>,
        visit: impl FnMut(&str) -> Result<()>,
    ) -> Result<()> {
        self.core.visit_projected(w, visit)
    }
    pub(crate) fn visit_coverage(
        &self,
        w: &mut DocumentWork<'_>,
        visit: impl FnMut(RefCoverage) -> Result<()>,
    ) -> Result<()> {
        if !std::ptr::eq(self.core.index, &self.owner.index) {
            return Err(Error::Invalid);
        }
        let end = self.owner.visit_coverage_at(
            self.core.unit.projected.clone(),
            self.core.start,
            w,
            visit,
        )?;
        if end != self.core.end {
            return Err(Error::Invalid);
        }
        w.admit(0)
    }
}
