//! Charged aliases to one exact query snapshot, never a recreated cursor identity.
use super::*;
struct Allocation {
    request: SearchRequest,
    _metadata: Charge,
}
pub(crate) struct SharedQuery {
    inner: Arc<Allocation>,
    _handle: Charge,
}
pub(crate) struct QueryCursor {
    cursor: SearchCursor,
    _charge: Charge,
}
impl SharedQuery {
    pub(crate) fn new(request: SearchRequest, work: &mut DocumentWork<'_>) -> Result<Self> {
        work.admit_structure(size_of::<Allocation>() + size_of::<Self>())?;
        let metadata = request.budget.reserve(
            size_of::<Allocation>() - size_of::<SearchRequest>() + 2 * size_of::<usize>(),
        )?;
        let handle = request.budget.reserve(size_of::<Self>())?;
        work.admit_structure(0)?;
        Ok(Self {
            inner: Arc::new(Allocation {
                request,
                _metadata: metadata,
            }),
            _handle: handle,
        })
    }
    pub(crate) fn alias(&self, work: &mut DocumentWork<'_>) -> Result<Self> {
        work.admit_structure(size_of::<Self>())?;
        let handle = self.inner.request.budget.reserve(size_of::<Self>())?;
        work.admit_structure(0)?;
        Ok(Self {
            inner: self.inner.clone(),
            _handle: handle,
        })
    }
    pub(crate) fn cursor(
        &self,
        cursor: &SearchCursor,
        work: &mut DocumentWork<'_>,
    ) -> Result<QueryCursor> {
        let request = &self.inner.request;
        if !request.identity.same(&cursor.identity) || request.domain != cursor.domain {
            return Err(Error::Invalid);
        }
        work.admit_structure(size_of::<QueryCursor>())?;
        let charge = request.budget.reserve(size_of::<QueryCursor>())?;
        work.admit_structure(0)?;
        Ok(QueryCursor {
            cursor: SearchCursor {
                identity: cursor.identity.clone(),
                domain: cursor.domain,
                sequence: cursor.sequence,
                start: cursor.start,
            },
            _charge: charge,
        })
    }
    pub(crate) fn find(
        &self,
        direction: Direction,
        after: Option<&QueryCursor>,
        cancel: &AtomicBool,
    ) -> SearchOutcome {
        self.inner
            .request
            .find(direction, after.map(|c| &c.cursor), cancel)
    }
}

#[cfg(test)]
#[test]
fn query_compiled_layout_inventory() {
    eprintln!(
        "LAYOUT QueryAllocation {} SearchRequest {} SharedQuery {} QueryCursor {}",
        size_of::<Allocation>(),
        size_of::<SearchRequest>(),
        size_of::<SharedQuery>(),
        size_of::<QueryCursor>()
    );
}
