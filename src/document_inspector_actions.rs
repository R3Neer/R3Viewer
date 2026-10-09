//! Inert Source descriptors. Complete-unit actions require exhausted coverage proof.
use crate::{
    document_consumer_budget::{Charge, ConsumerBudget},
    document_model::Document,
    document_text::{Coverage, DocumentWork, Error, RefCoverage, Relation, Result},
    preparation_owner::Binding,
    source_preparation::PreparedSourceContext,
    text_engine::Identity,
};
use std::{mem::size_of, ops::Range, sync::Arc};
pub(crate) struct Scope {
    pub(super) document: Arc<Document>,
    pub(super) context: Arc<PreparedSourceContext>,
    pub(super) binding: Binding,
    pub(super) gate: Arc<crate::document_provider_gate::ProviderGate>,
    pub(super) identity: Identity,
    pub(super) budget: ConsumerBudget,
    _source_metadata: Arc<Charge>,
    _charge: Charge,
}
impl Scope {
    pub(super) fn new(
        document: Arc<Document>,
        context: Arc<PreparedSourceContext>,
        budget: &ConsumerBudget,
        source_metadata: Arc<Charge>,
        binding: Binding,
        gate: Arc<crate::document_provider_gate::ProviderGate>,
        work: &mut DocumentWork<'_>,
    ) -> Result<Arc<Self>> {
        work.admit_structure(size_of::<Self>())?;
        if !budget.same(source_metadata.budget())
            || !budget.belongs_to(&document)
            || !Arc::ptr_eq(context.source(), &document.source_handle())
            || !gate.validates(&document, budget)
            || !Arc::ptr_eq(gate.context(), &context)
            || context.binding(binding.revision) != binding
        {
            return Err(Error::Invalid);
        }
        let charge = budget.reserve(size_of::<Self>() + 4 * size_of::<usize>())?;
        work.admit_structure(0)?;
        Ok(Arc::new(Self {
            document,
            binding,
            gate,
            context,
            identity: Identity::new(),
            budget: budget.clone(),
            _source_metadata: source_metadata,
            _charge: charge,
        }))
    }
    pub(super) fn checked_range(
        &self,
        range: Range<u32>,
        work: &mut DocumentWork<'_>,
    ) -> Result<Range<usize>> {
        let a = range.start as usize;
        let b = range.end as usize;
        work.admit_structure(0)?;
        if a > b || self.document.source().text().get(a..b).is_none() {
            return Err(Error::Invalid);
        }
        Ok(a..b)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ComponentAction {
    Exact(Range<u32>),
    ControlProjection {
        range: Range<u32>,
        single_scalar: bool,
    },
    Enclosing {
        range: Range<u32>,
        origin: u32,
    },
    NoLiteral,
}
impl ComponentAction {
    pub(super) fn from_run(
        scope: &Scope,
        coverage: &Coverage,
        work: &mut DocumentWork<'_>,
    ) -> Result<Self> {
        Self::classify(
            scope,
            coverage.relation,
            coverage.source.clone(),
            coverage.text_origin.unwrap_or(coverage.origin),
            work,
        )
    }
    pub(super) fn from_ref(
        scope: &Scope,
        coverage: &RefCoverage,
        work: &mut DocumentWork<'_>,
    ) -> Result<Self> {
        Self::classify(
            scope,
            coverage.relation,
            Some(coverage.source.clone()),
            coverage.origin,
            work,
        )
    }
    fn classify(
        scope: &Scope,
        relation: Relation,
        source: Option<Range<u32>>,
        origin: u32,
        work: &mut DocumentWork<'_>,
    ) -> Result<Self> {
        work.admit_structure(size_of::<Self>())?;
        match relation {
            Relation::Synthetic => Ok(Self::NoLiteral),
            Relation::ExactSource => {
                let r = source.ok_or(Error::Invalid)?;
                scope.checked_range(r.clone(), work)?;
                Ok(Self::Exact(r))
            }
            Relation::EnclosingToken => {
                let r = source.ok_or(Error::Invalid)?;
                scope.checked_range(r.clone(), work)?;
                Ok(Self::Enclosing { range: r, origin })
            }
            Relation::ControlProjection => {
                let r = source.ok_or(Error::Invalid)?;
                let checked = scope.checked_range(r.clone(), work)?;
                let raw = &scope.document.source().text()[checked];
                let mut chars = raw.chars();
                let first = chars.next();
                let second = chars.next();
                work.admit_structure(
                    first.map_or(0, char::len_utf8) + second.map_or(0, char::len_utf8),
                )?;
                Ok(Self::ControlProjection {
                    range: r,
                    single_scalar: first.is_some() && second.is_none(),
                })
            }
        }
    }
    pub(super) fn range(&self) -> Option<Range<u32>> {
        match self {
            Self::Exact(r)
            | Self::ControlProjection { range: r, .. }
            | Self::Enclosing { range: r, .. } => Some(r.clone()),
            Self::NoLiteral => None,
        }
    }
}
/// No range/relationship can be supplied by a caller to create an actionable descriptor.
pub(crate) struct SourceAction {
    scope: Arc<Scope>,
    component: ComponentAction,
    _charge: Charge,
}
impl SourceAction {
    pub(super) fn prepare(
        scope: &Arc<Scope>,
        component: &ComponentAction,
        work: &mut DocumentWork<'_>,
    ) -> Result<Self> {
        work.admit_structure(size_of::<Self>())?;
        if let Some(range) = component.range() {
            scope.checked_range(range, work)?;
        }
        let charge = scope.budget.reserve(size_of::<Self>())?;
        work.admit_structure(0)?;
        Ok(Self {
            scope: scope.clone(),
            component: component.clone(),
            _charge: charge,
        })
    }
    pub(crate) fn relation(&self) -> &ComponentAction {
        &self.component
    }
    pub(crate) fn binding(&self) -> Binding {
        self.scope.binding
    }
    pub(crate) fn source(&self) -> &Arc<crate::document_source::DocumentSource> {
        self.scope.context.source()
    }
    pub(super) fn check(&self, scope: &Scope) -> Result<()> {
        if self.scope.identity.same(&scope.identity) && self.scope.binding == scope.binding {
            Ok(())
        } else {
            Err(Error::Invalid)
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum WholeAction {
    Exact(Range<u32>),
    ControlScalar(Range<u32>),
    Enclosing { range: Range<u32>, origin: u32 },
    NoLiteral,
    Disjoint,
}
#[derive(Clone)]
pub(super) struct Aggregate {
    count: usize,
    first: Option<ComponentAction>,
    union: Option<Range<u32>>,
    compatible: bool,
}
impl Aggregate {
    pub(super) fn new() -> Self {
        Self {
            count: 0,
            first: None,
            union: None,
            compatible: true,
        }
    }
    pub(super) fn push(
        &mut self,
        component: &ComponentAction,
        work: &mut DocumentWork<'_>,
    ) -> Result<()> {
        work.admit_structure(size_of::<Self>())?;
        self.count = self.count.checked_add(1).ok_or(Error::Work)?;
        if let Some(first) = &self.first {
            self.compatible &= match (first, component, &mut self.union) {
                (ComponentAction::Exact(_), ComponentAction::Exact(next), Some(union))
                    if union.end == next.start =>
                {
                    union.end = next.end;
                    true
                }
                (
                    ComponentAction::Enclosing {
                        range: a,
                        origin: oa,
                    },
                    ComponentAction::Enclosing {
                        range: b,
                        origin: ob,
                    },
                    _,
                ) => a == b && oa == ob,
                (ComponentAction::NoLiteral, ComponentAction::NoLiteral, _) => true,
                _ => false,
            };
        } else {
            self.first = Some(component.clone());
            self.union = component.range();
        }
        Ok(())
    }
    // Called only by the inspection's closed cursor exhaustion branch, never page-prefix len.
    pub(super) fn exhausted(&self) -> Result<WholeAction> {
        if !self.compatible {
            return Ok(WholeAction::Disjoint);
        }
        Ok(match self.first.as_ref().ok_or(Error::Invalid)? {
            ComponentAction::Exact(_) => {
                WholeAction::Exact(self.union.clone().ok_or(Error::Invalid)?)
            }
            ComponentAction::ControlProjection {
                range,
                single_scalar: true,
            } if self.count == 1 => WholeAction::ControlScalar(range.clone()),
            ComponentAction::Enclosing { range, origin } => WholeAction::Enclosing {
                range: range.clone(),
                origin: *origin,
            },
            ComponentAction::NoLiteral => WholeAction::NoLiteral,
            _ => WholeAction::Disjoint,
        })
    }
}

impl SourceAction {
    pub(crate) fn check_view(&self, view: &crate::document_view::DocumentView) -> Result<()> {
        if self.scope.binding != view.render_binding()
            || !Arc::ptr_eq(&self.scope.document, view.inspect_document())
            || !Arc::ptr_eq(&self.scope.context, view.inspect_source_context())
            || !Arc::ptr_eq(&self.scope._source_metadata, view.inspect_source_metadata())
            || !self.scope.budget.same(view.budget())
        {
            return Err(Error::Invalid);
        }
        Ok(())
    }
}
