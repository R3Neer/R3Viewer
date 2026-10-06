//! Consumer allocations are a lease within the validated document's arena.
use crate::document_budget::Lease;
use crate::document_model::{Document, Error, ErrorKind, Result};
use std::{
    mem::size_of,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
pub const CONSUMER_LIMIT: usize = 1024 * 1024;
#[derive(Clone)]
pub struct ConsumerBudget(Arc<Inner>);
struct Inner {
    document: Arc<Document>,
    _lease: Lease,
    used: AtomicUsize,
    peak: AtomicUsize,
    checkpoints: AtomicUsize,
}
impl ConsumerBudget {
    pub fn new(document: &Arc<Document>) -> Result<Self> {
        let lease = document.reserve_consumer(CONSUMER_LIMIT)?;
        let overhead = size_of::<Inner>() + 2 * size_of::<usize>();
        Ok(Self(Arc::new(Inner {
            document: document.clone(),
            _lease: lease,
            used: AtomicUsize::new(overhead),
            peak: AtomicUsize::new(overhead),
            checkpoints: AtomicUsize::new(0),
        })))
    }
    pub(crate) fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    pub(crate) fn belongs_to(&self, document: &Arc<Document>) -> bool {
        Arc::ptr_eq(&self.0.document, document)
    }
    pub fn used(&self) -> usize {
        self.0.used.load(Ordering::Acquire)
    }
    pub fn checkpoint_capacity(&self) -> usize {
        self.0.checkpoints.load(Ordering::Acquire)
    }
    pub(crate) fn reserve_points(&self, bytes: usize, points: usize) -> Result<Charge> {
        let mut charge = self.reserve(bytes)?;
        let mut old = self.checkpoint_capacity();
        loop {
            let new = old
                .checked_add(points)
                .filter(|&n| n <= 4096)
                .ok_or_else(|| Error::new(ErrorKind::ArenaLimit))?;
            match self.0.checkpoints.compare_exchange_weak(
                old,
                new,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    charge.points = points;
                    return Ok(charge);
                }
                Err(actual) => old = actual,
            }
        }
    }
    pub fn peak(&self) -> usize {
        self.0.peak.load(Ordering::Acquire)
    }
    pub(crate) fn reserve(&self, bytes: usize) -> Result<Charge> {
        let mut old = self.used();
        loop {
            let new = old
                .checked_add(bytes)
                .filter(|&v| v <= CONSUMER_LIMIT)
                .ok_or_else(|| Error::new(ErrorKind::ArenaLimit))?;
            match self
                .0
                .used
                .compare_exchange_weak(old, new, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => {
                    self.0.peak.fetch_max(new, Ordering::AcqRel);
                    return Ok(Charge {
                        owner: self.clone(),
                        bytes,
                        points: 0,
                    });
                }
                Err(n) => old = n,
            }
        }
    }
}
pub(crate) struct Charge {
    owner: ConsumerBudget,
    bytes: usize,
    points: usize,
}
impl Drop for Charge {
    fn drop(&mut self) {
        self.owner.0.used.fetch_sub(self.bytes, Ordering::AcqRel);
        self.owner
            .0
            .checkpoints
            .fetch_sub(self.points, Ordering::AcqRel);
    }
}
pub(crate) struct ChargedVec<T> {
    data: Vec<T>,
    _charge: Charge,
}
impl<T> ChargedVec<T> {
    pub fn new(budget: &ConsumerBudget, capacity: usize) -> Result<Self> {
        Self::new_points(budget, capacity, 0)
    }
    pub fn new_points(budget: &ConsumerBudget, capacity: usize, units: usize) -> Result<Self> {
        let bytes = capacity
            .checked_mul(size_of::<T>())
            .ok_or_else(|| Error::new(ErrorKind::ArenaLimit))?;
        let charge = budget.reserve_points(
            bytes,
            capacity
                .checked_mul(units)
                .ok_or_else(|| Error::new(ErrorKind::ArenaLimit))?,
        )?;
        let mut data = Vec::new();
        data.try_reserve_exact(capacity)
            .map_err(|_| Error::new(ErrorKind::Allocation))?;
        let actual = data
            .capacity()
            .checked_mul(size_of::<T>())
            .ok_or_else(|| Error::new(ErrorKind::ArenaLimit))?;
        let extra = budget.reserve_points(
            actual.saturating_sub(bytes),
            data.capacity()
                .saturating_sub(capacity)
                .checked_mul(units)
                .ok_or_else(|| Error::new(ErrorKind::ArenaLimit))?,
        )?;
        let total = charge.bytes + extra.bytes;
        let points = charge.points + extra.points;
        let mut charge = charge;
        let mut extra = extra;
        charge.bytes = total;
        charge.points = points;
        extra.bytes = 0;
        extra.points = 0;
        Ok(Self {
            data,
            _charge: charge,
        })
    }
    pub fn push(&mut self, value: T) -> Result<()> {
        if self.data.len() == self.data.capacity() {
            return Err(Error::new(ErrorKind::ArenaLimit));
        }
        self.data.push(value);
        Ok(())
    }
    pub(crate) fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }
    pub(crate) fn last_mut(&mut self) -> Option<&mut T> {
        self.data.last_mut()
    }
    pub(crate) fn capacity(&self) -> usize {
        self.data.capacity()
    }
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }
}
