//! Immutable sharing adapter for the actual search hit. No provider/index clone.
//! Included as a child of document_search so provenance stays closed.
use super::{DocumentHit, HitProvider};
use crate::{
    document_consumer_budget::Charge,
    document_text::{DocumentWork, Error, Result, SegmentedIndex, TextRefIndex},
};
use std::{mem::size_of, ops::Range, sync::Arc};
struct HitAllocation {
    hit: DocumentHit,
    _arc_metadata: Charge,
}
pub(crate) struct SharedHit {
    inner: Arc<HitAllocation>,
    _handle: Charge,
}
pub(crate) struct HitLease {
    inner: Arc<HitAllocation>,
    _handle: Charge,
}
pub(crate) struct SharePending {
    pub(crate) cause: Error,
    pub(crate) hit: DocumentHit,
}
pub(crate) enum ProviderRef<'a> {
    Runs(&'a SegmentedIndex),
    TextRef(&'a TextRefIndex),
}
impl SharedHit {
    pub(crate) fn try_alias(&self, work: &mut DocumentWork<'_>) -> Result<Self> {
        work.admit_structure(size_of::<Self>())?;
        let charge = self.hit()._charge.budget().reserve(size_of::<Self>())?;
        work.admit_structure(0)?;
        Ok(Self {
            inner: self.inner.clone(),
            _handle: charge,
        })
    }

    // Returning the charged original value is infallible even when quota rejects every allocation.
    #[allow(clippy::result_large_err)]
    pub(crate) fn try_share(
        hit: DocumentHit,
        work: &mut DocumentWork<'_>,
    ) -> std::result::Result<Self, SharePending> {
        let prepared = (|| {
            work.admit_structure(size_of::<HitAllocation>() + size_of::<SharedHit>())?;
            let extra = size_of::<HitAllocation>()
                .checked_sub(size_of::<DocumentHit>())
                .and_then(|n| n.checked_add(2 * size_of::<usize>()))
                .ok_or(Error::Budget)?;
            let metadata = hit._charge.budget().reserve(extra)?;
            let handle = hit._charge.budget().reserve(size_of::<SharedHit>())?;
            work.admit_structure(0)?;
            Ok((metadata, handle))
        })();
        match prepared {
            Ok((metadata, handle)) => Ok(Self {
                inner: Arc::new(HitAllocation {
                    hit,
                    _arc_metadata: metadata,
                }),
                _handle: handle,
            }),
            Err(cause) => Err(SharePending { cause, hit }),
        }
    }
    pub(crate) fn hit(&self) -> &DocumentHit {
        &self.inner.hit
    }
    pub(crate) fn same_shared(&self, shared: &SharedHit) -> bool {
        Arc::ptr_eq(&self.inner, &shared.inner)
    }
    pub(crate) fn lease(&self, work: &mut DocumentWork<'_>) -> Result<HitLease> {
        work.admit_structure(size_of::<HitLease>())?;
        let charge = self.hit()._charge.budget().reserve(size_of::<HitLease>())?;
        work.admit_structure(0)?;
        Ok(HitLease {
            inner: self.inner.clone(),
            _handle: charge,
        })
    }
}
impl HitLease {
    pub(crate) fn same_shared(&self, shared: &SharedHit) -> bool {
        Arc::ptr_eq(&self.inner, &shared.inner)
    }

    pub(crate) fn hit(&self) -> &DocumentHit {
        &self.inner.hit
    }
    pub(crate) fn budget(&self) -> &crate::document_consumer_budget::ConsumerBudget {
        self.hit()._charge.budget()
    }
    pub(crate) fn try_alias(&self, work: &mut DocumentWork<'_>) -> Result<Self> {
        work.admit_structure(size_of::<Self>())?;
        let charge = self.budget().reserve(size_of::<Self>())?;
        work.admit_structure(0)?;
        Ok(Self {
            inner: self.inner.clone(),
            _handle: charge,
        })
    }
    pub(crate) fn provider(&self) -> Result<ProviderRef<'_>> {
        match self.hit().provider.as_ref().ok_or(Error::Invalid)? {
            HitProvider::Runs(i) => Ok(ProviderRef::Runs(i)),
            HitProvider::TextRef(i) => Ok(ProviderRef::TextRef(i)),
        }
    }
    pub(crate) fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
    pub(crate) fn initial_unit(&self, work: &mut DocumentWork<'_>) -> Result<Range<usize>> {
        let start = self.hit().highlight().ok_or(Error::Invalid)?.start;
        self.provider()?.unit_containing(start, work)
    }
}
impl ProviderRef<'_> {
    pub(crate) fn unit_containing(
        &self,
        offset: usize,
        work: &mut DocumentWork<'_>,
    ) -> Result<Range<usize>> {
        match self {
            Self::Runs(i) => {
                let mut at = i.seek_before_with_work(offset, work)?;
                loop {
                    let u = i
                        .step_resolved(&mut at, 4096, work)?
                        .ok_or(Error::Invalid)?;
                    if u.unit().projected.end > offset {
                        return Ok(u.unit().projected.clone());
                    }
                }
            }
            Self::TextRef(i) => {
                let mut at = i.seek_before_with_work(offset, work)?;
                loop {
                    let u = i
                        .step_resolved(&mut at, 4096, work)?
                        .ok_or(Error::Invalid)?;
                    if u.unit().projected.end > offset {
                        return Ok(u.unit().projected.clone());
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[test]
fn hit_compiled_layout_inventory() {
    eprintln!(
        "LAYOUT HitAllocation {} DocumentHit {} SharedHit {} HitLease {} SharePending {}",
        size_of::<HitAllocation>(),
        size_of::<DocumentHit>(),
        size_of::<SharedHit>(),
        size_of::<HitLease>(),
        size_of::<SharePending>()
    );
}
