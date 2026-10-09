//! Provenance inspection over actual D1 units. Preparation-only; inert choices/descriptors.
use crate::document_provider_gate::{IndexOwner, RefOwner as ProviderRefOwner};
use crate::{
    document_consumer_budget::{Charge, ChargedVec, ConsumerBudget},
    document_layout::SequenceSlot,
    document_model::{Document, Link, NONE},
    document_navigation::State,
    document_presentation_anchor::{CellProgress, PresentationAnchor, RefField, RefOwner},
    document_roles as roles,
    document_search::{
        SearchDomain,
        inspect_hit::{HitLease, ProviderRef, SharedHit},
    },
    document_text::{
        CoverageComponent, CoverageCursor, CoverageStep, DocumentWork, Error, IndexProfile, Point,
        Result, SegmentedIndex, TextRefIndex, Unit,
    },
    document_view::DocumentView,
    text_engine::Identity,
};
use std::{
    mem::size_of,
    ops::Range,
    sync::{Arc, atomic::AtomicBool},
};
#[path = "document_inspector_actions.rs"]
mod actions;
use actions::{Aggregate, Scope};
pub(crate) use actions::{ComponentAction, SourceAction, WholeAction};
pub(crate) const PAGE: usize = 64;
const SAMPLE: usize = 4096;
const PROFILE: IndexProfile = IndexProfile {
    projection_points: 2,
    unicode_points: 2,
    exceptions: 512,
};
struct SlotLease {
    slot: Arc<SequenceSlot>,
    _charge: Charge,
}
struct OwnedAllocation {
    provider: OwnedProvider,
    _charge: Charge,
}
enum OwnedProvider {
    Runs(IndexOwner),
    TextRef(ProviderRefOwner),
}
struct OwnedLease {
    inner: Arc<OwnedAllocation>,
    _charge: Charge,
}
enum ProviderOwner {
    Hit(HitLease),
    Slot(SlotLease),
    Owned(OwnedLease),
    Role,
}
impl ProviderOwner {
    fn covered_inline(&self) -> usize {
        match self {
            Self::Hit(_) => size_of::<HitLease>(),
            Self::Slot(_) => size_of::<SlotLease>(),
            Self::Owned(_) => size_of::<OwnedLease>(),
            Self::Role => 0,
        }
    }
    fn provider(&self) -> Result<ProviderRef<'_>> {
        match self {
            Self::Hit(h) => h.provider(),
            Self::Slot(s) => Ok(s.slot.inspect_provider()),
            Self::Owned(o) => Ok(match &o.inner.provider {
                OwnedProvider::Runs(i) => ProviderRef::Runs(i),
                OwnedProvider::TextRef(i) => ProviderRef::TextRef(i),
            }),
            Self::Role => Err(Error::Invalid),
        }
    }
    fn try_alias(&self, budget: &ConsumerBudget, work: &mut DocumentWork<'_>) -> Result<Self> {
        work.admit_structure(size_of::<Self>())?;
        Ok(match self {
            Self::Hit(h) => Self::Hit(h.try_alias(work)?),
            Self::Slot(s) => {
                let c = budget.reserve(size_of::<SlotLease>())?;
                work.admit_structure(0)?;
                Self::Slot(SlotLease {
                    slot: s.slot.clone(),
                    _charge: c,
                })
            }
            Self::Owned(o) => {
                let c = budget.reserve(size_of::<OwnedLease>())?;
                work.admit_structure(0)?;
                Self::Owned(OwnedLease {
                    inner: o.inner.clone(),
                    _charge: c,
                })
            }
            Self::Role => Self::Role,
        })
    }
    fn cursor(&self, range: Range<usize>, work: &mut DocumentWork<'_>) -> Result<CoverageCursor> {
        match self.provider()? {
            ProviderRef::Runs(i) => CoverageCursor::runs(i.text(), range, work),
            ProviderRef::TextRef(i) => CoverageCursor::text_ref(i, range, work),
        }
    }
    fn next(
        &self,
        cursor: &mut CoverageCursor,
        work: &mut DocumentWork<'_>,
    ) -> Result<CoverageStep> {
        match self.provider()? {
            ProviderRef::Runs(i) => cursor.next_runs(i.text(), work),
            ProviderRef::TextRef(i) => cursor.next_ref(i, work),
        }
    }
}
pub(crate) struct Inspection {
    scope: Arc<Scope>,
    provider: ProviderOwner,
    range: Option<Range<usize>>,
    anchor: Option<PresentationAnchor>,
    ref_owner: Option<RefOwner>,
    resource: Option<u32>,
    block: u32,
    _charge: Charge,
}
impl Inspection {
    pub(crate) fn from_hit(
        hit: &SharedHit,
        view: &DocumentView,
        work: &mut DocumentWork<'_>,
    ) -> Result<Self> {
        let lease = hit.lease(work)?;
        let raw = lease.hit();
        if !Arc::ptr_eq(raw.document(), view.inspect_document())
            || !lease.budget().same(view.budget())
        {
            return Err(Error::Invalid);
        }
        let scope = Scope::new(
            raw.document().clone(),
            view.inspect_source_context().clone(),
            lease.budget(),
            view.inspect_source_metadata().clone(),
            view.render_binding(),
            view.inspect_provider_gate()
                .cloned()
                .ok_or(Error::ScopePending)?,
            work,
        )?;
        if !raw.provider_gate().is_some_and(|g| g.same(&scope.gate)) {
            return Err(Error::ScopePending);
        }
        Self::from_hit_scope(lease, scope, work)
    }
    pub(crate) fn from_hit_scope(
        lease: HitLease,
        scope: Arc<Scope>,
        work: &mut DocumentWork<'_>,
    ) -> Result<Self> {
        let raw = lease.hit();
        if !Arc::ptr_eq(raw.document(), &scope.document)
            || !lease.budget().same(&scope.budget)
            || !raw.provider_gate().is_some_and(|g| g.same(&scope.gate))
        {
            return Err(Error::Invalid);
        }
        let _scratch = scope
            .budget
            .reserve(size_of::<Point>() + size_of::<Unit>())?;
        let range = lease.initial_unit(work)?;
        let block = raw.block();
        let resource = raw.resource_id();
        let ref_owner = if raw.domain() == SearchDomain::ResourceTitle {
            let id = resource.ok_or(Error::Invalid)?;
            let r = scope
                .document
                .resources()
                .get(id as usize)
                .ok_or(Error::Invalid)?;
            Some(RefOwner::Resource {
                block,
                run: r.owner_run(),
                resource: id,
                field: RefField::Title,
            })
        } else {
            None
        };
        let provider = ProviderOwner::Hit(lease);
        let charge = scope
            .budget
            .reserve(size_of::<Self>() - provider.covered_inline())?;
        work.admit_structure(0)?;
        Ok(Self {
            scope,
            provider,
            range: Some(range),
            anchor: None,
            ref_owner,
            resource,
            block,
            _charge: charge,
        })
    }
    /// Selection comes from the real current view or real projected cell, never caller-supplied bytes.
    pub(crate) fn from_view(
        view: &DocumentView,
        cell: Option<usize>,
        live_hit: Option<&SharedHit>,
        cancel: &AtomicBool,
        work: &mut DocumentWork<'_>,
    ) -> Result<Self> {
        let anchor = if let Some(cell) = cell {
            view.layout()
                .text()
                .ok_or(Error::Invalid)?
                .cell_anchor(cell, work)?
                .ok_or(Error::Invalid)?
        } else {
            view.presentation_anchor().cloned().ok_or(Error::Invalid)?
        };
        Self::from_actual_anchor(view, anchor, live_hit, cancel, work)
    }
    /// Logical cursor factory resolves the already accredited State through the exact current Flow.
    pub(crate) fn from_cursor(
        view: &DocumentView,
        state: &State,
        live_hit: Option<&SharedHit>,
        cancel: &AtomicBool,
        work: &mut DocumentWork<'_>,
    ) -> Result<Self> {
        if state.position_pending() {
            return Err(Error::Work);
        }
        view.navigator().index().check_location(state.location())?;
        if state.run() == NONE {
            let actual = State::prepared_role(view.navigator().index(), state.block(), cancel)?;
            let scope = Scope::new(
                view.inspect_document().clone(),
                view.inspect_source_context().clone(),
                view.budget(),
                view.inspect_source_metadata().clone(),
                view.render_binding(),
                view.inspect_provider_gate()
                    .cloned()
                    .ok_or(Error::ScopePending)?,
                work,
            )?;
            roles::block_role(&scope.document, actual.block(), work)?;
            let charge = scope.budget.reserve(size_of::<Self>())?;
            work.admit_structure(0)?;
            return Ok(Self {
                scope,
                provider: ProviderOwner::Role,
                range: None,
                anchor: None,
                ref_owner: None,
                resource: None,
                block: actual.block(),
                _charge: charge,
            });
        }
        let actual = crate::document_navigation::State::prepared_visual(
            view.navigator().index(),
            state.run(),
            state.raw_byte(),
            state.projection_part(),
            cancel,
        )?;
        Self::from_actual_anchor(
            view,
            PresentationAnchor::Domain(crate::document_view::LayoutAnchor {
                raw: crate::document_view::RawAnchor {
                    block: actual.block(),
                    run: actual.run(),
                    byte: actual.raw_byte(),
                },
                projection_part: actual.projection_part(),
            }),
            live_hit,
            cancel,
            work,
        )
    }
    pub(crate) fn from_actual_anchor(
        view: &DocumentView,
        anchor: PresentationAnchor,
        live_hit: Option<&SharedHit>,
        cancel: &AtomicBool,
        work: &mut DocumentWork<'_>,
    ) -> Result<Self> {
        let scope = Scope::new(
            view.inspect_document().clone(),
            view.inspect_source_context().clone(),
            view.budget(),
            view.inspect_source_metadata().clone(),
            view.render_binding(),
            view.inspect_provider_gate()
                .cloned()
                .ok_or(Error::ScopePending)?,
            work,
        )?;
        if let Some(hit) = live_hit
            && (!Arc::ptr_eq(hit.hit().document(), &scope.document)
                || !hit
                    .hit()
                    .provider_gate()
                    .is_some_and(|g| g.same(&scope.gate)))
        {
            return Err(Error::Invalid);
        }
        let slot = view.layout().text().and_then(|t| t.slot_alias());
        Self::from_anchor_scope(scope, anchor, slot, cancel, work)
    }
    pub(super) fn from_anchor_scope(
        scope: Arc<Scope>,
        anchor: PresentationAnchor,
        slot: Option<Arc<SequenceSlot>>,
        cancel: &AtomicBool,
        work: &mut DocumentWork<'_>,
    ) -> Result<Self> {
        let block = anchor.block();
        let mut resource = None;
        let mut ref_owner = None;
        let resolved = match &anchor {
            PresentationAnchor::Role(a) => {
                roles::block_role(&scope.document, a.block(), work)?;
                if let Some(r) = scope
                    .document
                    .runs()
                    .get(a.run() as usize)
                    .filter(|r| r.kind() == 5)
                {
                    resource = Some(r.resource());
                }
                None
            }
            PresentationAnchor::Ref(a) => {
                crate::document_presentation_anchor::RefAnchor::checked(
                    &scope.document,
                    a.owner(),
                    a.text_ref(),
                    a.local(),
                    a.part(),
                    work,
                )?;
                ref_owner = Some(a.owner());
                if let RefOwner::Resource { resource: r, .. } = a.owner() {
                    resource = Some(r);
                }
                Some(Selection::Ref {
                    id: a.text_ref(),
                    local: a.local(),
                    part: a.part(),
                })
            }
            PresentationAnchor::ResourceAlt(a) => {
                resource = Some(a.resource());
                Some(Selection::Run {
                    domain: a.domain(),
                    run: a.position().raw.run,
                    local: a.position().raw.byte,
                    part: a.position().projection_part,
                })
            }
            PresentationAnchor::Domain(a) => {
                let r = scope
                    .document
                    .runs()
                    .get(a.raw.run as usize)
                    .ok_or(Error::Invalid)?;
                if r.kind() == 5 {
                    resource = Some(r.resource());
                    None
                } else {
                    Some(Self::run_selection(&scope.document, *a, work)?)
                }
            }
            PresentationAnchor::Table(t) => {
                let c = t
                    .cells()
                    .iter()
                    .find(|c| c.cell == t.selected())
                    .ok_or(Error::Invalid)?;
                match c.progress {
                    CellProgress::At(a) => Some(Self::run_selection(&scope.document, a, work)?),
                    CellProgress::Done => None,
                }
            }
        };
        let (provider, range) = if let Some(selection) = resolved {
            let _scratch = scope
                .budget
                .reserve(size_of::<Point>() + size_of::<Unit>())?;
            let existing = if let Some(slot) = slot {
                if !slot.inspect_budget().same(&scope.budget)
                    || !slot.provider_gate().is_some_and(|g| g.same(&scope.gate))
                {
                    return Err(Error::Invalid);
                }
                let range = Self::selection_unit(slot.inspect_provider(), selection, work);
                match range {
                    Ok(r) => {
                        let c = scope.budget.reserve(size_of::<SlotLease>())?;
                        Some((ProviderOwner::Slot(SlotLease { slot, _charge: c }), r))
                    }
                    Err(Error::Invalid) => None,
                    Err(e) => return Err(e),
                }
            } else {
                None
            };
            if let Some((p, r)) = existing {
                (p, Some(r))
            } else {
                let p = Self::prepare_provider(&scope, selection, cancel, work)?;
                let r = Self::selection_unit(p.provider()?, selection, work)?;
                (p, Some(r))
            }
        } else {
            (ProviderOwner::Role, None)
        };
        let charge = scope
            .budget
            .reserve(size_of::<Self>() - provider.covered_inline())?;
        work.admit_structure(0)?;
        Ok(Self {
            scope,
            provider,
            range,
            anchor: Some(anchor),
            ref_owner,
            resource,
            block,
            _charge: charge,
        })
    }
    fn run_selection(
        document: &Document,
        a: crate::document_view::LayoutAnchor,
        work: &mut DocumentWork<'_>,
    ) -> Result<Selection> {
        crate::document_view::validate_raw_anchor(document, a.raw, work)?;
        crate::document_text::validate_visual_part(
            document,
            a.raw.run,
            a.raw.byte,
            a.projection_part,
            work,
        )?;
        let run = document
            .runs()
            .get(a.raw.run as usize)
            .ok_or(Error::Invalid)?;
        Ok(Selection::Run {
            domain: run.domain(),
            run: a.raw.run,
            local: a.raw.byte,
            part: a.projection_part,
        })
    }
    fn selection_unit(
        provider: ProviderRef<'_>,
        selection: Selection,
        work: &mut DocumentWork<'_>,
    ) -> Result<Range<usize>> {
        match (provider, selection) {
            (
                ProviderRef::Runs(i),
                Selection::Run {
                    run, local, part, ..
                },
            ) => {
                let offset = i
                    .text()
                    .locate_raw(run, local, work)?
                    .checked_add(part as usize)
                    .ok_or(Error::Invalid)?;
                ProviderRef::Runs(i).unit_containing(offset, work)
            }
            (ProviderRef::TextRef(i), Selection::Ref { id, local, part }) if i.id() == id => {
                let at = i.seek_visual(local, part, work)?;
                ProviderRef::TextRef(i).unit_containing(at.offset(), work)
            }
            _ => Err(Error::Invalid),
        }
    }
    fn prepare_provider(
        scope: &Arc<Scope>,
        selection: Selection,
        cancel: &AtomicBool,
        work: &mut DocumentWork<'_>,
    ) -> Result<ProviderOwner> {
        work.admit_structure(size_of::<OwnedAllocation>() + size_of::<OwnedLease>())?;
        let provider = match selection {
            Selection::Run { domain, run, .. } => {
                let d = scope
                    .document
                    .domains()
                    .get(domain as usize)
                    .ok_or(Error::Invalid)?;
                let selected = scope
                    .document
                    .runs()
                    .get(run as usize)
                    .ok_or(Error::Invalid)?;
                let mut first = selected.ordinal();
                let mut end = first + 1;
                while first > 0
                    && scope.document.runs()[(d.first_run() + first - 1) as usize].kind() != 5
                {
                    work.admit_structure(size_of::<crate::document_model::Run>())?;
                    first -= 1;
                }
                while end < d.run_count()
                    && scope.document.runs()[(d.first_run() + end) as usize].kind() != 5
                {
                    work.admit_structure(size_of::<crate::document_model::Run>())?;
                    end += 1;
                }
                let text = crate::document_provider_gate::TextOwner::prepare(
                    scope.document.clone(),
                    domain,
                    first as usize..end as usize,
                    &scope.budget,
                    cancel,
                    PROFILE,
                    Some(&scope.gate),
                )?;
                OwnedProvider::Runs(text.into_index(&scope.budget, cancel, PROFILE)?)
            }
            Selection::Ref { id, .. } => OwnedProvider::TextRef(ProviderRefOwner::prepare(
                scope.document.clone(),
                id,
                &scope.budget,
                cancel,
                PROFILE,
                Some(&scope.gate),
            )?),
        };
        let charged_body = match provider {
            OwnedProvider::Runs(_) => size_of::<SegmentedIndex>(),
            OwnedProvider::TextRef(_) => size_of::<TextRefIndex>(),
        };
        let metadata = scope
            .budget
            .reserve(size_of::<OwnedAllocation>() - charged_body + 2 * size_of::<usize>())?;
        let handle = scope.budget.reserve(size_of::<OwnedLease>())?;
        work.admit_structure(0)?;
        Ok(ProviderOwner::Owned(OwnedLease {
            inner: Arc::new(OwnedAllocation {
                provider,
                _charge: metadata,
            }),
            _charge: handle,
        }))
    }
    pub(crate) fn anchor(&self) -> Option<&PresentationAnchor> {
        self.anchor.as_ref()
    }
    pub(crate) fn ref_owner(&self) -> Option<RefOwner> {
        self.ref_owner
    }
    pub(crate) fn resource_id(&self) -> Option<u32> {
        self.resource
    }
    pub(crate) fn projected_unit(&self) -> Option<Range<usize>> {
        self.range.clone()
    }
    fn cursor(&self, kind: CursorKind, work: &mut DocumentWork<'_>) -> Result<PageCursor> {
        let provider = self.provider.try_alias(&self.scope.budget, work)?;
        let coverage = self
            .range
            .clone()
            .map(|r| provider.cursor(r, work))
            .transpose()?;
        let charge = self.scope.budget.reserve(
            size_of::<PageCursor>()
                - provider.covered_inline()
                - if coverage.is_some() {
                    size_of::<CoverageCursor>()
                } else {
                    0
                },
        )?;
        work.admit_structure(0)?;
        Ok(PageCursor {
            scope: self.scope.clone(),
            provider,
            coverage,
            role_done: false,
            last_link: None,
            kind,
            _charge: charge,
        })
    }
    fn checked_cursor(
        &self,
        after: Option<&PageCursor>,
        kind: CursorKind,
        work: &mut DocumentWork<'_>,
    ) -> Result<PageCursor> {
        if let Some(c) = after {
            if !self.scope.identity.same(&c.scope.identity) || c.kind != kind {
                return Err(Error::Invalid);
            }
            c.try_fork(work)
        } else {
            self.cursor(kind, work)
        }
    }
    pub(crate) fn page(
        &self,
        after: Option<&PageCursor>,
        work: &mut DocumentWork<'_>,
    ) -> Result<InspectionPage> {
        let mut cursor = self.checked_cursor(after, CursorKind::Associations, work)?;
        let mut values = ChargedVec::new(&self.scope.budget, PAGE)?;
        let charge = self.scope.budget.reserve(size_of::<InspectionPage>())?;
        let mut next = true;
        while values.as_slice().len() < PAGE {
            match cursor.next(work)? {
                Next::Component(c) => {
                    let action = match &c {
                        CoverageComponent::Run(c) => {
                            ComponentAction::from_run(&self.scope, c, work)?
                        }
                        CoverageComponent::TextRef(c) => {
                            ComponentAction::from_ref(&self.scope, c, work)?
                        }
                    };
                    values.push(Association {
                        coverage: Some(c),
                        action,
                    })?;
                }
                Next::Role => values.push(Association {
                    coverage: None,
                    action: ComponentAction::NoLiteral,
                })?,
                Next::End => {
                    next = false;
                    break;
                }
            }
        }
        work.admit_structure(0)?;
        Ok(InspectionPage {
            scope: self.scope.clone(),
            values,
            next: if next { Some(Box::new(cursor)) } else { None },
            _charge: charge,
        })
    }
    pub(crate) fn source_component(
        &self,
        page: &InspectionPage,
        index: usize,
        work: &mut DocumentWork<'_>,
    ) -> Result<SourceAction> {
        self.check_scope(&page.scope)?;
        let value = page.values.as_slice().get(index).ok_or(Error::Invalid)?;
        SourceAction::prepare(&self.scope, &value.action, work)
    }
    fn check_scope(&self, scope: &Scope) -> Result<()> {
        if self.scope.identity.same(&scope.identity) {
            Ok(())
        } else {
            Err(Error::Invalid)
        }
    }
    pub(crate) fn summary_start(&self, work: &mut DocumentWork<'_>) -> Result<Summary> {
        let cursor = self.cursor(CursorKind::Summary, work)?;
        let charge = self
            .scope
            .budget
            .reserve(size_of::<Summary>() - size_of::<PageCursor>())?;
        Ok(Summary {
            scope: self.scope.clone(),
            cursor: Some(cursor),
            aggregate: Aggregate::new(),
            complete: None,
            _charge: charge,
        })
    }
    pub(crate) fn summary_advance(
        &self,
        current: &Summary,
        work: &mut DocumentWork<'_>,
    ) -> Result<Summary> {
        self.check_scope(&current.scope)?;
        if current.complete.is_some() {
            return Err(Error::Invalid);
        }
        let cursor = current.cursor.as_ref().ok_or(Error::Invalid)?;
        let mut next = cursor.try_fork(work)?;
        let mut aggregate = current.aggregate.clone();
        let charge = self
            .scope
            .budget
            .reserve(size_of::<Summary>() - size_of::<PageCursor>())?;
        let mut complete = None;
        for _ in 0..PAGE {
            match next.next(work)? {
                Next::Component(c) => {
                    let action = match &c {
                        CoverageComponent::Run(c) => {
                            ComponentAction::from_run(&self.scope, c, work)?
                        }
                        CoverageComponent::TextRef(c) => {
                            ComponentAction::from_ref(&self.scope, c, work)?
                        }
                    };
                    aggregate.push(&action, work)?
                }
                Next::Role => aggregate.push(&ComponentAction::NoLiteral, work)?,
                Next::End => {
                    complete = Some(aggregate.exhausted()?);
                    break;
                }
            }
        }
        work.admit_structure(0)?;
        Ok(Summary {
            scope: self.scope.clone(),
            cursor: Some(next),
            aggregate,
            complete,
            _charge: charge,
        })
    }
    pub(crate) fn choices(
        &self,
        after: Option<&PageCursor>,
        work: &mut DocumentWork<'_>,
    ) -> Result<LinkPage> {
        let mut cursor = self.checked_cursor(after, CursorKind::Links, work)?;
        let mut links = ChargedVec::new(&self.scope.budget, PAGE)?;
        let charge = self
            .scope
            .budget
            .reserve(size_of::<LinkPage>() + 2 * size_of::<usize>())?;
        let mut more = true;
        while links.as_slice().len() < PAGE {
            match cursor.next(work)? {
                Next::Component(CoverageComponent::Run(c)) => {
                    if let Some(id) = c.link
                        && cursor.last_link != Some(id)
                    {
                        work.admit_structure(size_of::<Link>())?;
                        self.scope
                            .document
                            .links()
                            .get(id as usize)
                            .ok_or(Error::Invalid)?;
                        links.push(id)?;
                        cursor.last_link = Some(id);
                    }
                }
                Next::Component(_) | Next::Role => {}
                Next::End => {
                    more = false;
                    break;
                }
            }
        }
        work.admit_structure(0)?;
        Ok(LinkPage {
            scope: self.scope.clone(),
            identity: Identity::new(),
            links,
            next: if more { Some(Box::new(cursor)) } else { None },
            _charge: charge,
        })
    }
    pub(crate) fn choose(
        &self,
        page: &LinkPage,
        index: usize,
        work: &mut DocumentWork<'_>,
    ) -> Result<LinkChoice> {
        self.check_scope(&page.scope)?;
        let link = *page.links.as_slice().get(index).ok_or(Error::Invalid)?;
        let charge = self.scope.budget.reserve(size_of::<LinkChoice>())?;
        work.admit_structure(0)?;
        Ok(LinkChoice {
            scope: self.scope.clone(),
            page: page.identity.clone(),
            index,
            link,
            _charge: charge,
        })
    }
    pub(crate) fn confirm(
        &self,
        page: &LinkPage,
        choice: &LinkChoice,
        work: &mut DocumentWork<'_>,
    ) -> Result<SelectedLink> {
        self.check_scope(&page.scope)?;
        self.check_scope(&choice.scope)?;
        if !choice.page.same(&page.identity)
            || page.links.as_slice().get(choice.index) != Some(&choice.link)
        {
            return Err(Error::Invalid);
        }
        work.admit_structure(size_of::<Link>())?;
        let record = self
            .scope
            .document
            .links()
            .get(choice.link as usize)
            .ok_or(Error::Invalid)?;
        let charge = self.scope.budget.reserve(size_of::<SelectedLink>())?;
        work.admit_structure(0)?;
        Ok(SelectedLink {
            scope: self.scope.clone(),
            link: record.id(),
            _charge: charge,
        })
    }
    pub(crate) fn resource(
        &self,
        work: &mut DocumentWork<'_>,
    ) -> Result<Option<roles::ResourceDescriptor<'_>>> {
        self.resource
            .map(|r| roles::resource_descriptor(&self.scope.document, r, work))
            .transpose()
    }
    pub(crate) fn enclosing_block(&self, work: &mut DocumentWork<'_>) -> Result<SourceAction> {
        work.admit_structure(
            size_of::<crate::document_model::Block>() + size_of::<crate::document_model::Origin>(),
        )?;
        let b = self
            .scope
            .document
            .blocks()
            .get(self.block as usize)
            .ok_or(Error::Invalid)?;
        let origin = self
            .scope
            .document
            .origins()
            .get(b.origin() as usize)
            .ok_or(Error::Invalid)?;
        SourceAction::prepare(
            &self.scope,
            &ComponentAction::Enclosing {
                range: origin.start()..origin.end(),
                origin: b.origin(),
            },
            work,
        )
    }
}
#[derive(Clone, Copy)]
enum Selection {
    Run {
        domain: u32,
        run: u32,
        local: usize,
        part: u8,
    },
    Ref {
        id: u32,
        local: usize,
        part: u8,
    },
}
pub(crate) struct Association {
    coverage: Option<CoverageComponent>,
    action: ComponentAction,
}
impl Association {
    pub(crate) fn coverage(&self) -> Option<&CoverageComponent> {
        self.coverage.as_ref()
    }
    pub(crate) fn action(&self) -> &ComponentAction {
        &self.action
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum CursorKind {
    Associations,
    Links,
    Summary,
}
pub(crate) struct PageCursor {
    kind: CursorKind,
    scope: Arc<Scope>,
    provider: ProviderOwner,
    coverage: Option<CoverageCursor>,
    role_done: bool,
    last_link: Option<u32>,
    _charge: Charge,
}
enum Next {
    Component(CoverageComponent),
    Role,
    End,
}
impl PageCursor {
    fn try_fork(&self, work: &mut DocumentWork<'_>) -> Result<Self> {
        let provider = self.provider.try_alias(&self.scope.budget, work)?;
        let coverage = self
            .coverage
            .as_ref()
            .map(|c| c.try_fork(work))
            .transpose()?;
        let charge = self.scope.budget.reserve(
            size_of::<Self>()
                - provider.covered_inline()
                - if coverage.is_some() {
                    size_of::<CoverageCursor>()
                } else {
                    0
                },
        )?;
        work.admit_structure(0)?;
        Ok(Self {
            scope: self.scope.clone(),
            provider,
            coverage,
            kind: self.kind,
            role_done: self.role_done,
            last_link: self.last_link,
            _charge: charge,
        })
    }
    fn next(&mut self, work: &mut DocumentWork<'_>) -> Result<Next> {
        if let Some(c) = &mut self.coverage {
            match self.provider.next(c, work)? {
                CoverageStep::Component(c) => Ok(Next::Component(c)),
                CoverageStep::End(proof) => {
                    if proof.is_for(c) {
                        Ok(Next::End)
                    } else {
                        Err(Error::Invalid)
                    }
                }
            }
        } else {
            work.admit_structure(0)?;
            if self.role_done {
                Ok(Next::End)
            } else {
                self.role_done = true;
                Ok(Next::Role)
            }
        }
    }
}
pub(crate) struct InspectionPage {
    scope: Arc<Scope>,
    values: ChargedVec<Association>,
    next: Option<Box<PageCursor>>,
    _charge: Charge,
}
impl InspectionPage {
    pub(crate) fn values(&self) -> &[Association] {
        self.values.as_slice()
    }
    pub(crate) fn next(&self) -> Option<&PageCursor> {
        self.next.as_deref()
    }
}
pub(crate) struct Summary {
    scope: Arc<Scope>,
    cursor: Option<PageCursor>,
    aggregate: Aggregate,
    complete: Option<WholeAction>,
    _charge: Charge,
}
impl Summary {
    pub(crate) fn complete(&self) -> Option<&WholeAction> {
        self.complete.as_ref()
    }
}
pub(crate) struct LinkPage {
    scope: Arc<Scope>,
    identity: Identity,
    links: ChargedVec<u32>,
    next: Option<Box<PageCursor>>,
    _charge: Charge,
}
impl LinkPage {
    pub(crate) fn links(&self) -> &[u32] {
        self.links.as_slice()
    }
    pub(crate) fn next(&self) -> Option<&PageCursor> {
        self.next.as_deref()
    }
}
pub(crate) struct LinkChoice {
    scope: Arc<Scope>,
    page: Identity,
    index: usize,
    link: u32,
    _charge: Charge,
}
pub(crate) struct SelectedLink {
    scope: Arc<Scope>,
    link: u32,
    _charge: Charge,
}
impl SelectedLink {
    pub(crate) fn id(&self) -> u32 {
        self.link
    }
    pub(crate) fn record(&self, work: &mut DocumentWork<'_>) -> Result<&Link> {
        work.admit_structure(size_of::<Link>())?;
        self.scope
            .document
            .links()
            .get(self.link as usize)
            .ok_or(Error::Invalid)
    }
}
#[cfg(test)]
#[path = "document_inspector_tests.rs"]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LinkField {
    Target,
    Raw,
    Title,
    ReferenceLabel,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ResourceField {
    Input,
    Raw,
    Title,
}
#[derive(Clone, Copy)]
enum SampleOwner {
    Link { link: u32, field: LinkField },
    Resource { resource: u32, field: ResourceField },
    Ref(RefOwner),
}
pub(crate) struct SampleCursor {
    scope: Arc<Scope>,
    owner: SampleOwner,
    text_ref: u32,
    local: usize,
    _charge: Charge,
}
pub(crate) struct FieldSample {
    scope: Arc<Scope>,
    text_ref: u32,
    local: Range<usize>,
    bytes: ChargedVec<u8>,
    next: Option<Box<SampleCursor>>,
    _charge: Charge,
}
impl FieldSample {
    pub(crate) fn text(&self) -> &str {
        std::str::from_utf8(self.bytes.as_slice()).expect("scalar projected complete UTF8")
    }
    pub(crate) fn next(&self) -> Option<&SampleCursor> {
        self.next.as_deref()
    }
    pub(crate) fn provenance(&self, work: &mut DocumentWork<'_>) -> Result<roles::TextRefRelation> {
        roles::text_ref_view(&self.scope.document, self.text_ref, work)?
            .relation(self.local.clone(), work)
    }
}
impl Inspection {
    pub(crate) fn sample_link(
        &self,
        selected: &SelectedLink,
        field: LinkField,
        work: &mut DocumentWork<'_>,
    ) -> Result<Option<FieldSample>> {
        self.check_scope(&selected.scope)?;
        let l = selected.record(work)?;
        let id = match field {
            LinkField::Target => l.target_decoded(),
            LinkField::Raw => l.raw_token(),
            LinkField::Title => l.title_decoded(),
            LinkField::ReferenceLabel => l.reference_label(),
        };
        if id == NONE {
            Ok(None)
        } else {
            self.sample_ref(
                SampleOwner::Link {
                    link: l.id(),
                    field,
                },
                id,
                0,
                work,
            )
            .map(Some)
        }
    }
    pub(crate) fn sample_resource(
        &self,
        field: ResourceField,
        work: &mut DocumentWork<'_>,
    ) -> Result<Option<FieldSample>> {
        let Some(r) = self.resource(work)? else {
            return Ok(None);
        };
        let id = match field {
            ResourceField::Input => Some(r.input_ref()),
            ResourceField::Raw => Some(r.raw_ref()),
            ResourceField::Title => r.title_ref(),
        };
        id.map(|id| {
            self.sample_ref(
                SampleOwner::Resource {
                    resource: r.id(),
                    field,
                },
                id,
                0,
                work,
            )
        })
        .transpose()
    }
    pub(crate) fn sample_metadata(
        &self,
        work: &mut DocumentWork<'_>,
    ) -> Result<Option<FieldSample>> {
        let Some(owner) = self.ref_owner else {
            return Ok(None);
        };
        let id = match owner {
            RefOwner::Resource {
                resource, field, ..
            } => {
                let r = roles::resource_descriptor(&self.scope.document, resource, work)?;
                match field {
                    RefField::Input => Some(r.input_ref()),
                    RefField::Title => r.title_ref(),
                    _ => None,
                }
            }
            RefOwner::BlockRole { block, field } => {
                match (roles::block_role(&self.scope.document, block, work)?, field) {
                    (roles::BlockRole::Callout { kind_text, .. }, RefField::Kind) => {
                        Some(kind_text)
                    }
                    (roles::BlockRole::Code { info, .. }, RefField::Info) => info,
                    (roles::BlockRole::Footnote { label }, RefField::Label) => Some(label),
                    _ => None,
                }
            }
        };
        id.map(|id| self.sample_ref(SampleOwner::Ref(owner), id, 0, work))
            .transpose()
    }
    pub(crate) fn sample_next(
        &self,
        after: &SampleCursor,
        work: &mut DocumentWork<'_>,
    ) -> Result<FieldSample> {
        self.check_scope(&after.scope)?;
        self.sample_ref(after.owner, after.text_ref, after.local, work)
    }
    fn sample_ref(
        &self,
        owner: SampleOwner,
        id: u32,
        start: usize,
        work: &mut DocumentWork<'_>,
    ) -> Result<FieldSample> {
        let view = roles::text_ref_view(&self.scope.document, id, work)?;
        let text = view.semantic_text();
        if start > text.len() || !text.is_char_boundary(start) {
            return Err(Error::Invalid);
        }
        let charge = self.scope.budget.reserve(size_of::<FieldSample>())?;
        let mut bytes = ChargedVec::new(&self.scope.budget, SAMPLE)?;
        let mut at = start;
        while at < text.len() {
            let mut chars = text[at..].chars();
            let ch = chars.next().ok_or(Error::Invalid)?;
            let next = chars.next();
            let mut scratch = [0; 8];
            let projected = crate::text_projection::scalar(ch, next, &mut scratch);
            work.admit_structure(ch.len_utf8() + next.map_or(0, char::len_utf8) + projected.len())?;
            if bytes.as_slice().len() + projected.len() > SAMPLE {
                break;
            }
            for &byte in projected.as_bytes() {
                bytes.push(byte)?;
            }
            at += ch.len_utf8();
        }
        let next = if at < text.len() {
            let c = self.scope.budget.reserve(size_of::<SampleCursor>())?;
            Some(Box::new(SampleCursor {
                scope: self.scope.clone(),
                owner,
                text_ref: id,
                local: at,
                _charge: c,
            }))
        } else {
            None
        };
        work.admit_structure(0)?;
        Ok(FieldSample {
            scope: self.scope.clone(),
            text_ref: id,
            local: start..at,
            bytes,
            next,
            _charge: charge,
        })
    }
    pub(crate) fn origin_records<'a>(
        &'a self,
        page: &InspectionPage,
        index: usize,
        work: &mut DocumentWork<'_>,
    ) -> Result<(
        Option<&'a crate::document_model::Origin>,
        Option<&'a crate::document_model::Origin>,
    )> {
        self.check_scope(&page.scope)?;
        let association = page.values().get(index).ok_or(Error::Invalid)?;
        let (run, text) = match association.coverage() {
            Some(CoverageComponent::Run(c)) => (Some(c.origin), c.text_origin),
            Some(CoverageComponent::TextRef(c)) => (None, Some(c.origin)),
            None => (None, None),
        };
        let mut get = |id: u32| {
            work.admit_structure(size_of::<crate::document_model::Origin>())?;
            self.scope
                .document
                .origins()
                .get(id as usize)
                .ok_or(Error::Invalid)
        };
        Ok((run.map(&mut get).transpose()?, text.map(get).transpose()?))
    }
}

impl Summary {
    pub(crate) fn disjoint_cursor(&self, work: &mut DocumentWork<'_>) -> Result<PageCursor> {
        if self.complete.as_ref() != Some(&WholeAction::Disjoint) {
            return Err(Error::Invalid);
        }
        let original = self.cursor.as_ref().ok_or(Error::Invalid)?;
        let kind = CursorKind::Associations;
        let provider = original.provider.try_alias(&self.scope.budget, work)?;
        let range = original.coverage.as_ref().ok_or(Error::Invalid)?.range();
        let coverage = provider.cursor(range, work)?;
        let charge = self.scope.budget.reserve(
            size_of::<PageCursor>() - provider.covered_inline() - size_of::<CoverageCursor>(),
        )?;
        work.admit_structure(0)?;
        Ok(PageCursor {
            scope: self.scope.clone(),
            provider,
            coverage: Some(coverage),
            role_done: false,
            last_link: None,
            kind,
            _charge: charge,
        })
    }
}

impl Inspection {
    pub(crate) fn binding(&self) -> crate::preparation_owner::Binding {
        self.scope.binding
    }
    pub(crate) fn block_role(&self, work: &mut DocumentWork<'_>) -> Result<roles::BlockRole> {
        roles::block_role(&self.scope.document, self.block, work)
    }
    pub(crate) fn source(&self) -> &Arc<crate::document_source::DocumentSource> {
        self.scope.context.source()
    }
    pub(crate) fn whole_source(
        &self,
        summary: &Summary,
        work: &mut DocumentWork<'_>,
    ) -> Result<Option<SourceAction>> {
        self.check_scope(&summary.scope)?;
        let action = match summary.complete().ok_or(Error::Work)? {
            WholeAction::Exact(r) => ComponentAction::Exact(r.clone()),
            WholeAction::ControlScalar(r) => ComponentAction::ControlProjection {
                range: r.clone(),
                single_scalar: true,
            },
            WholeAction::Enclosing { range, origin } => ComponentAction::Enclosing {
                range: range.clone(),
                origin: *origin,
            },
            WholeAction::NoLiteral => ComponentAction::NoLiteral,
            WholeAction::Disjoint => return Ok(None),
        };
        SourceAction::prepare(&self.scope, &action, work).map(Some)
    }
    pub(crate) fn enclosing_resource(
        &self,
        work: &mut DocumentWork<'_>,
    ) -> Result<Option<SourceAction>> {
        let Some(r) = self.resource(work)? else {
            return Ok(None);
        };
        let raw = r.origin(work)?;
        let record = self
            .scope
            .document
            .resources()
            .get(r.id() as usize)
            .ok_or(Error::Invalid)?;
        SourceAction::prepare(
            &self.scope,
            &ComponentAction::Enclosing {
                range: raw.start()..raw.end(),
                origin: record.origin(),
            },
            work,
        )
        .map(Some)
    }
}

#[path = "document_inspection_preparation.rs"]
pub(crate) mod preparation;

#[cfg(test)]
#[test]
fn inspector_compiled_layout_inventory() {
    macro_rules! item {($($t:ty),*)=>{$(eprintln!("LAYOUT {} {}",stringify!($t),size_of::<$t>());)*};}
    item!(
        Scope,
        ProviderOwner,
        SlotLease,
        OwnedProvider,
        OwnedAllocation,
        OwnedLease,
        Inspection,
        PageCursor,
        InspectionPage,
        Summary,
        LinkPage,
        LinkChoice,
        SelectedLink,
        SampleCursor,
        FieldSample,
        Association,
        Selection
    );
}
