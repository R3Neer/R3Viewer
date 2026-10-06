//! Immutable identity/source lease for complete semantic documents.
use crate::{
    document_model::{Error, ErrorKind, Result},
    input::SourceOrigin,
};
use sha2::{Digest, Sha256};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
#[derive(Debug)]
pub struct DocumentSource {
    original: Arc<String>,
    origin: SourceOrigin,
    generation: u64,
    request: u64,
    hash: [u8; 32],
}
impl DocumentSource {
    pub fn new(
        original: Arc<String>,
        origin: SourceOrigin,
        generation: u64,
        request: u64,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        if original.len() > 134_217_728 {
            return Err(Error::new(ErrorKind::SourceLimit));
        }
        let mut hash = Sha256::new();
        for bytes in original.as_bytes().chunks(4096) {
            if cancel.load(Ordering::Relaxed) {
                return Err(Error::new(ErrorKind::Cancelled));
            }
            hash.update(bytes);
        }
        if cancel.load(Ordering::Relaxed) {
            return Err(Error::new(ErrorKind::Cancelled));
        }
        Ok(Self {
            original,
            origin,
            generation,
            request,
            hash: hash.finalize().into(),
        })
    }
    /// Share the immutable original without reconstructing or reopening it.
    pub fn original_handle(&self) -> Arc<String> {
        self.original.clone()
    }
    pub fn text(&self) -> &str {
        &self.original
    }
    pub fn len(&self) -> usize {
        self.original.len()
    }
    pub fn is_empty(&self) -> bool {
        self.original.is_empty()
    }
    pub fn capacity(&self) -> usize {
        self.original.capacity()
    }
    pub fn origin(&self) -> &SourceOrigin {
        &self.origin
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn request(&self) -> u64 {
        self.request
    }
    pub fn hash(&self) -> &[u8; 32] {
        &self.hash
    }
    pub fn range(&self, start: u32, end: u32) -> Result<&str> {
        self.text()
            .get(start as usize..end as usize)
            .ok_or_else(|| Error::new(ErrorKind::InvalidContext))
    }
}
