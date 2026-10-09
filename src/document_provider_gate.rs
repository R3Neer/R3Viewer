//! Scoped physical-provider admission. Existing D1/INLINE callers remain unchanged.
use crate::{
    document_consumer_budget::{Charge, ConsumerBudget},
    document_model::Document,
    document_text::{
        DocumentWork, Error, IndexProfile, Result, SegmentedIndex, SegmentedText, TextRefIndex,
    },
    source_preparation::PreparedSourceContext,
    text_engine::Identity,
};
use std::{
    mem::size_of,
    ops::{Deref, Range},
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};
struct PermitAllocation {
    gate: Arc<ProviderGate>,
}
struct PermitEntry {
    weak: Weak<PermitAllocation>,
    _allocation: Charge,
}
struct ProviderPermit {
    inner: Arc<PermitAllocation>,
}
pub(crate) struct ProviderGate {
    document: Arc<Document>,
    budget: ConsumerBudget,
    context: Arc<PreparedSourceContext>,
    _source_metadata: Arc<Charge>,
    identity: Identity,
    slots: Mutex<[Option<PermitEntry>; 2]>,
    admitted: AtomicUsize,
    rejected: AtomicUsize,
    peak: AtomicUsize,
    _charge: Charge,
}
impl ProviderGate {
    pub(crate) fn from_owner(
        document: Arc<Document>,
        budget: &ConsumerBudget,
        context: Arc<PreparedSourceContext>,
        metadata: Arc<Charge>,
        work: &mut DocumentWork<'_>,
    ) -> Result<Arc<Self>> {
        work.admit_structure(size_of::<Self>())?;
        if !budget.belongs_to(&document)
            || !budget.same(metadata.budget())
            || !Arc::ptr_eq(context.source(), &document.source_handle())
        {
            return Err(Error::Invalid);
        }
        let charge = budget.reserve(size_of::<Self>() + 4 * size_of::<usize>())?;
        work.admit_structure(0)?;
        Ok(Arc::new(Self {
            document,
            budget: budget.clone(),
            context,
            _source_metadata: metadata,
            identity: Identity::new(),
            slots: Mutex::new([None, None]),
            admitted: AtomicUsize::new(0),
            rejected: AtomicUsize::new(0),
            peak: AtomicUsize::new(0),
            _charge: charge,
        }))
    }
    pub(crate) fn same(&self, other: &Self) -> bool {
        self.identity.same(&other.identity)
    }
    pub(crate) fn validates(&self, document: &Arc<Document>, budget: &ConsumerBudget) -> bool {
        Arc::ptr_eq(&self.document, document) && self.budget.same(budget)
    }
    pub(crate) fn context(&self) -> &Arc<PreparedSourceContext> {
        &self.context
    }
    fn acquire(
        self: &Arc<Self>,
        document: &Arc<Document>,
        budget: &ConsumerBudget,
        work: &mut DocumentWork<'_>,
    ) -> Result<ProviderPermit> {
        work.admit_structure(size_of::<PermitEntry>() * 2)?;
        if !self.validates(document, budget) {
            return Err(Error::Invalid);
        }
        let mut slots = self.slots.lock().map_err(|_| Error::Invalid)?;
        for entry in slots.iter_mut() {
            if entry.as_ref().is_some_and(|e| e.weak.strong_count() == 0) {
                drop(entry.take());
            }
        }
        let Some(index) = slots.iter().position(|entry| entry.is_none()) else {
            self.rejected.fetch_add(1, Ordering::Relaxed);
            return Err(Error::ProvidersBusy);
        };
        let entry = &mut slots[index];
        let receipt = budget.reserve(size_of::<PermitAllocation>() + 2 * size_of::<usize>())?;
        work.admit_structure(0)?;
        let inner = Arc::new(PermitAllocation { gate: self.clone() });
        *entry = Some(PermitEntry {
            weak: Arc::downgrade(&inner),
            _allocation: receipt,
        });
        self.admitted.fetch_add(1, Ordering::Relaxed);
        self.peak.fetch_max(
            slots
                .iter()
                .filter(|e| e.as_ref().is_some_and(|e| e.weak.strong_count() > 0))
                .count(),
            Ordering::Relaxed,
        );
        Ok(ProviderPermit { inner })
    }
    pub(crate) fn stats(&self) -> (usize, usize, usize) {
        (
            self.admitted.load(Ordering::Relaxed),
            self.rejected.load(Ordering::Relaxed),
            self.peak.load(Ordering::Relaxed),
        )
    }
    #[cfg(test)]
    pub(crate) fn permit_allocation_bytes(&self) -> usize {
        size_of::<PermitAllocation>() + 2 * size_of::<usize>()
    }
    #[cfg(test)]
    pub(crate) fn occupied(&self) -> (usize, usize) {
        let s = self.slots.lock().unwrap();
        (
            s.iter()
                .filter(|e| e.as_ref().is_some_and(|e| e.weak.strong_count() > 0))
                .count(),
            s.iter().filter(|e| e.is_some()).count(),
        )
    }
}
pub(crate) struct Gated<T> {
    value: T,
    permit: ProviderPermit,
}
pub(crate) enum TextOwner {
    Legacy(SegmentedText),
    Scoped(Gated<SegmentedText>),
}
pub(crate) enum IndexOwner {
    Legacy(SegmentedIndex),
    Scoped(Gated<SegmentedIndex>),
}
pub(crate) enum RefOwner {
    Legacy(TextRefIndex),
    Scoped(Gated<TextRefIndex>),
}
macro_rules! borrowed {
    ($owner:ident,$value:ty) => {
        impl Deref for $owner {
            type Target = $value;
            fn deref(&self) -> &Self::Target {
                match self {
                    Self::Legacy(value) => value,
                    Self::Scoped(gated) => &gated.value,
                }
            }
        }
        impl $owner {
            pub(crate) fn gate(&self) -> Option<&Arc<ProviderGate>> {
                match self {
                    Self::Legacy(_) => None,
                    Self::Scoped(gated) => Some(&gated.permit.inner.gate),
                }
            }
        }
    };
}
borrowed!(TextOwner, SegmentedText);
borrowed!(IndexOwner, SegmentedIndex);
borrowed!(RefOwner, TextRefIndex);
impl TextOwner {
    pub(crate) fn prepare(
        document: Arc<Document>,
        domain: u32,
        runs: Range<usize>,
        budget: &ConsumerBudget,
        cancel: &AtomicBool,
        profile: IndexProfile,
        gate: Option<&Arc<ProviderGate>>,
    ) -> Result<Self> {
        let permit = gate
            .map(|g| g.acquire(&document, budget, &mut DocumentWork::preparation(cancel)))
            .transpose()?;
        let value =
            SegmentedText::prepare_with_profile(document, domain, runs, budget, cancel, profile)?;
        Ok(match permit {
            Some(permit) => Self::Scoped(Gated { value, permit }),
            None => Self::Legacy(value),
        })
    }
    pub(crate) fn into_index(
        self,
        budget: &ConsumerBudget,
        cancel: &AtomicBool,
        profile: IndexProfile,
    ) -> Result<IndexOwner> {
        match self {
            Self::Legacy(text) => {
                SegmentedIndex::prepare_with_profile(text, budget, cancel, profile)
                    .map(IndexOwner::Legacy)
            }
            Self::Scoped(Gated { value, permit }) => {
                let value = SegmentedIndex::prepare_with_profile(value, budget, cancel, profile)?;
                Ok(IndexOwner::Scoped(Gated { value, permit }))
            }
        }
    }
}
impl IndexOwner {
    pub(crate) fn prepare(
        document: Arc<Document>,
        domain: u32,
        runs: Range<usize>,
        budget: &ConsumerBudget,
        cancel: &AtomicBool,
        profile: IndexProfile,
        gate: Option<&Arc<ProviderGate>>,
    ) -> Result<Self> {
        TextOwner::prepare(document, domain, runs, budget, cancel, profile, gate)?
            .into_index(budget, cancel, profile)
    }
}
impl RefOwner {
    pub(crate) fn prepare(
        document: Arc<Document>,
        id: u32,
        budget: &ConsumerBudget,
        cancel: &AtomicBool,
        profile: IndexProfile,
        gate: Option<&Arc<ProviderGate>>,
    ) -> Result<Self> {
        let permit = gate
            .map(|g| g.acquire(&document, budget, &mut DocumentWork::preparation(cancel)))
            .transpose()?;
        let value = TextRefIndex::prepare(document, id, budget, cancel, profile)?;
        Ok(match permit {
            Some(permit) => Self::Scoped(Gated { value, permit }),
            None => Self::Legacy(value),
        })
    }
}

#[cfg(test)]
#[test]
fn provider_compiled_layout_inventory() {
    macro_rules! item {($($t:ty),*)=>{$(eprintln!("LAYOUT {} {}",stringify!($t),size_of::<$t>());)*};}
    item!(
        ProviderGate,
        PermitAllocation,
        PermitEntry,
        ProviderPermit,
        Gated<SegmentedText>,
        Gated<SegmentedIndex>,
        Gated<TextRefIndex>,
        TextOwner,
        IndexOwner,
        RefOwner,
        SegmentedText,
        SegmentedIndex,
        TextRefIndex,
        Charge,
        Identity
    );
}
