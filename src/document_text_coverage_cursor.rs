//! Incremental D1 coverage. Same scalar projection and Run merging as visit_coverage.
//! A cursor is charged, identity checked, transactional, and contains no ownerless borrowed reference.
use super::*;
use std::mem::size_of;
#[derive(Clone)]
struct State {
    at: Position,
    end: usize,
    pending: Option<Coverage>,
}
pub(crate) struct CoverageCursor {
    identity: Identity,
    selection: PayloadSelection,
    range: Range<usize>,
    start: Position,
    state: State,
    _charge: Charge,
}
pub(crate) enum Component {
    Run(Coverage),
    TextRef(RefCoverage),
}
pub(crate) struct EndCoverage {
    identity: Identity,
    range: Range<usize>,
    start: Position,
    end: Position,
}
impl EndCoverage {
    pub(crate) fn is_for(&self, cursor: &CoverageCursor) -> bool {
        self.identity.same(&cursor.identity)
            && self.range == cursor.range
            && self.start == cursor.start
            && self.end == cursor.state.at
            && self.end.offset == self.range.end
            && cursor.state.pending.is_none()
    }
}
pub(crate) enum CoverageStep {
    Component(Component),
    End(EndCoverage),
}
impl CoverageCursor {
    pub(crate) fn runs(
        text: &SegmentedText,
        range: Range<usize>,
        work: &mut DocumentWork<'_>,
    ) -> Result<Self> {
        if !matches!(text.selection, PayloadSelection::Runs) {
            return Err(Error::Invalid);
        }
        Self::new(text, range, work)
    }
    pub(crate) fn text_ref(
        index: &TextRefIndex,
        range: Range<usize>,
        work: &mut DocumentWork<'_>,
    ) -> Result<Self> {
        Self::new(&index.index.text, range, work)
    }
    fn new(text: &SegmentedText, range: Range<usize>, work: &mut DocumentWork<'_>) -> Result<Self> {
        work.admit_structure(size_of::<Self>())?;
        if range.start > range.end || range.end > text.len {
            return Err(Error::Invalid);
        }
        let charge = text._scratch.budget().reserve(size_of::<Self>())?;
        let at = text.locate(range.start, work)?;
        work.admit(0)?;
        Ok(Self {
            identity: text.identity.clone(),
            selection: text.selection,
            range: range.clone(),
            start: at,
            state: State {
                at,
                end: range.end,
                pending: None,
            },
            _charge: charge,
        })
    }
    pub(crate) fn try_fork(&self, work: &mut DocumentWork<'_>) -> Result<Self> {
        work.admit_structure(size_of::<Self>())?;
        let charge = self._charge.budget().reserve(size_of::<Self>())?;
        work.admit(0)?;
        Ok(Self {
            identity: self.identity.clone(),
            selection: self.selection,
            range: self.range.clone(),
            start: self.start,
            state: self.state.clone(),
            _charge: charge,
        })
    }
    pub(crate) fn next_runs(
        &mut self,
        text: &SegmentedText,
        work: &mut DocumentWork<'_>,
    ) -> Result<CoverageStep> {
        if !matches!(
            (self.selection, text.selection),
            (PayloadSelection::Runs, PayloadSelection::Runs)
        ) {
            return Err(Error::Invalid);
        }
        self.next(text, work)
    }
    pub(crate) fn next_ref(
        &mut self,
        index: &TextRefIndex,
        work: &mut DocumentWork<'_>,
    ) -> Result<CoverageStep> {
        if !matches!((self.selection,index.index.text.selection),(PayloadSelection::TextRef(a),PayloadSelection::TextRef(b))if a==b)
        {
            return Err(Error::Invalid);
        }
        self.next(&index.index.text, work)
    }
    fn next(&mut self, text: &SegmentedText, work: &mut DocumentWork<'_>) -> Result<CoverageStep> {
        work.admit_structure(size_of::<State>())?;
        if !self.identity.same(&text.identity) {
            return Err(Error::Invalid);
        }
        let _scratch = self
            ._charge
            .budget()
            .reserve(size_of::<State>() + size_of::<Component>())?;
        let mut state = self.state.clone();
        let output = loop {
            if state.at.offset == state.end {
                break match state.pending.take() {
                    Some(c) => CoverageStep::Component(Component::Run(c)),
                    None => CoverageStep::End(EndCoverage {
                        identity: self.identity.clone(),
                        range: self.range.clone(),
                        start: self.start,
                        end: state.at,
                    }),
                };
            }
            let scalar = text.raw_scalar(state.at.raw, work)?.ok_or(Error::Invalid)?;
            let mut bytes = [0; 8];
            let len = text.projection(scalar, work, &mut bytes)?;
            let projected = !scalar.synthetic && len != scalar.ch.len_utf8();
            let record = match self.selection {
                PayloadSelection::Runs => {
                    if scalar.synthetic {
                        None
                    } else {
                        Some(
                            &text.document.text_refs()
                                [text.document.runs()[scalar.raw.run].text() as usize],
                        )
                    }
                }
                PayloadSelection::TextRef(id) => Some(&text.document.text_refs()[id as usize]),
            };
            let relation = if scalar.synthetic {
                Relation::Synthetic
            } else if record.is_some_and(|t| t.kind() == 1) {
                Relation::EnclosingToken
            } else if projected {
                Relation::ControlProjection
            } else {
                Relation::ExactSource
            };
            let source = record
                .map(|t| -> Result<Range<u32>> {
                    if t.kind() == 0 {
                        let a = u32::try_from(scalar.raw.byte).map_err(|_| Error::Invalid)?;
                        let b = u32::try_from(scalar.raw_end).map_err(|_| Error::Invalid)?;
                        Ok(t.start().checked_add(a).ok_or(Error::Invalid)?
                            ..t.start().checked_add(b).ok_or(Error::Invalid)?)
                    } else {
                        work.admit_structure(size_of::<crate::document_model::Origin>())?;
                        let o = &text.document.origins()[t.origin() as usize];
                        Ok(o.start()..o.end())
                    }
                })
                .transpose()?;
            let component = match self.selection {
                PayloadSelection::Runs => {
                    work.admit_structure(size_of::<Coverage>())?;
                    let r = &text.document.runs()[scalar.raw.run];
                    Component::Run(Coverage {
                        control_projected: projected,
                        text_origin: record.map(|t| t.origin()),
                        source,
                        run: r.id(),
                        local: scalar.raw.byte..scalar.raw_end,
                        origin: r.origin(),
                        styles: r.styles(),
                        link: (r.link() != NONE).then_some(r.link()),
                        relation,
                    })
                }
                PayloadSelection::TextRef(id) => {
                    work.admit_structure(size_of::<RefCoverage>())?;
                    Component::TextRef(RefCoverage {
                        text_ref: id,
                        local: scalar.raw.byte..scalar.raw_end,
                        source: source.ok_or(Error::Invalid)?,
                        origin: record.ok_or(Error::Invalid)?.origin(),
                        relation,
                        control_projected: projected,
                    })
                }
            };
            let consumed = (len - state.at.part).min(state.end - state.at.offset);
            let part = state.at.part.checked_add(consumed).ok_or(Error::Invalid)?;
            if !std::str::from_utf8(&bytes[..len])
                .map_err(|_| Error::Invalid)?
                .is_char_boundary(part)
            {
                return Err(Error::Invalid);
            }
            state.at = Position {
                raw: if part == len { scalar.next } else { scalar.raw },
                part: if part == len { 0 } else { part },
                offset: state
                    .at
                    .offset
                    .checked_add(consumed)
                    .ok_or(Error::Invalid)?,
            };
            match component {
                Component::TextRef(c) => break CoverageStep::Component(Component::TextRef(c)),
                Component::Run(c) => {
                    if let Some(last) = &mut state.pending {
                        if last.run == c.run
                            && last.local.end == c.local.start
                            && last.relation == c.relation
                            && last.control_projected == c.control_projected
                            && last.origin == c.origin
                            && last.text_origin == c.text_origin
                        {
                            last.local.end = c.local.end;
                            if matches!(
                                last.relation,
                                Relation::ExactSource | Relation::ControlProjection
                            ) && let (Some(a), Some(b)) = (&mut last.source, &c.source)
                            {
                                a.end = b.end;
                            }
                        } else {
                            let previous = state.pending.replace(c).ok_or(Error::Invalid)?;
                            break CoverageStep::Component(Component::Run(previous));
                        }
                    } else {
                        state.pending = Some(c);
                    }
                }
            }
        };
        work.admit(0)?;
        self.state = state;
        Ok(output)
    }
}

impl CoverageCursor {
    pub(crate) fn range(&self) -> Range<usize> {
        self.range.clone()
    }
}
