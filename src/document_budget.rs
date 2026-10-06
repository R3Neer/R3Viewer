//! Controlled arena allocation accounting, separate from source and parser allocations.
use crate::document_model::{Error, ErrorKind, Result};
use std::{
    mem::size_of,
    ops::{Deref, DerefMut},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
pub const ARENA_LIMIT: usize = 64 * 1024 * 1024;
pub const TRANSPORT_RESERVE: usize = 3 * 1024 * 1024;
#[derive(Clone)]
pub struct ActorArena(Arc<Inner>);
struct Inner {
    used: AtomicUsize,
    peak: AtomicUsize,
    limit: usize,
}
impl ActorArena {
    pub fn new() -> Self {
        Self::with_limit(ARENA_LIMIT)
    }
    pub(crate) fn with_limit(limit: usize) -> Self {
        Self(Arc::new(Inner {
            used: AtomicUsize::new(TRANSPORT_RESERVE),
            peak: AtomicUsize::new(TRANSPORT_RESERVE),
            limit,
        }))
    }
    pub fn used(&self) -> usize {
        self.0.used.load(Ordering::Acquire)
    }
    pub fn peak(&self) -> usize {
        self.0.peak.load(Ordering::Acquire)
    }
    pub(crate) fn reserve(&self, bytes: usize) -> Result<Lease> {
        let mut old = self.used();
        loop {
            let new = old
                .checked_add(bytes)
                .filter(|&n| n <= self.0.limit)
                .ok_or_else(|| Error::new(ErrorKind::ArenaLimit))?;
            match self
                .0
                .used
                .compare_exchange_weak(old, new, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => {
                    self.0.peak.fetch_max(new, Ordering::AcqRel);
                    return Ok(Lease {
                        arena: self.clone(),
                        bytes,
                    });
                }
                Err(actual) => old = actual,
            }
        }
    }
}
impl Default for ActorArena {
    fn default() -> Self {
        Self::new()
    }
}
pub(crate) struct Lease {
    arena: ActorArena,
    bytes: usize,
}
impl Drop for Lease {
    fn drop(&mut self) {
        self.arena.0.used.fetch_sub(self.bytes, Ordering::AcqRel);
    }
}
pub(crate) struct ArenaVec<T> {
    data: Vec<T>,
    arena: ActorArena,
    charge: Option<Lease>,
}
impl<T> ArenaVec<T> {
    pub fn new(arena: ActorArena) -> Self {
        assert!(size_of::<T>() > 0);
        Self {
            data: Vec::new(),
            arena,
            charge: None,
        }
    }
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }
    pub fn reserve_exact(&mut self, target: usize) -> Result<()> {
        if target <= self.capacity() {
            return Ok(());
        }
        let bytes = target
            .checked_mul(size_of::<T>())
            .ok_or_else(|| Error::new(ErrorKind::ArenaLimit))?;
        // Existing allocation remains charged until reserve returns; reserve worst-case new.
        let mut new = self.arena.reserve(bytes)?;
        self.data
            .try_reserve_exact(target - self.data.len())
            .map_err(|_| Error::new(ErrorKind::Allocation))?;
        let actual = self
            .capacity()
            .checked_mul(size_of::<T>())
            .ok_or_else(|| Error::new(ErrorKind::ArenaLimit))?;
        if actual > bytes {
            match self.arena.reserve(actual - bytes) {
                Ok(mut extra) => {
                    new.bytes += extra.bytes;
                    extra.bytes = 0;
                }
                Err(e) => {
                    self.data = Vec::new();
                    self.charge = None;
                    return Err(e);
                }
            }
        }
        self.charge = Some(new);
        Ok(())
    }
    pub fn push(&mut self, value: T) -> Result<()> {
        if self.len() == self.capacity() {
            self.reserve_exact(self.capacity().saturating_mul(2).max(16))?;
        }
        self.data.push(value);
        Ok(())
    }
    pub fn extend_from_slice(&mut self, value: &[T]) -> Result<()>
    where
        T: Copy,
    {
        self.reserve_exact(
            self.len()
                .checked_add(value.len())
                .ok_or_else(|| Error::new(ErrorKind::ArenaLimit))?,
        )?;
        self.data.extend_from_slice(value);
        Ok(())
    }
    pub fn truncate(&mut self, n: usize) {
        self.data.truncate(n);
    }
}
impl<T> Deref for ArenaVec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.data
    }
}
impl<T> DerefMut for ArenaVec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn growth_admits_old_plus_new_before_reserve() {
        let arena = ActorArena::with_limit(TRANSPORT_RESERVE + 48);
        let mut bytes = ArenaVec::<u8>::new(arena.clone());
        bytes.reserve_exact(16).unwrap();
        assert_eq!(bytes.capacity(), 16);
        bytes.extend_from_slice(&[1; 16]).unwrap();
        bytes.reserve_exact(32).unwrap();
        assert_eq!(arena.peak(), TRANSPORT_RESERVE + 48);
        assert_eq!(arena.used(), TRANSPORT_RESERVE + 32);
        drop(bytes);
        assert_eq!(arena.used(), TRANSPORT_RESERVE);
    }
    #[test]
    fn growth_plus_one_rejects_without_losing_old_data() {
        let arena = ActorArena::with_limit(TRANSPORT_RESERVE + 47);
        let mut bytes = ArenaVec::<u8>::new(arena.clone());
        bytes.extend_from_slice(&[1; 16]).unwrap();
        assert_eq!(
            bytes.reserve_exact(32).unwrap_err().kind,
            ErrorKind::ArenaLimit
        );
        assert_eq!(&*bytes, &[1; 16]);
        assert_eq!(arena.used(), TRANSPORT_RESERVE + 16);
        assert_eq!(arena.peak(), TRANSPORT_RESERVE + 16);
    }
}
