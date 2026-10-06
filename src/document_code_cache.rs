//! Bounded semantic cache infrastructure. Only its future real producer can insert.
//! No color proof/activation constructors exist in this partial increment.
#![allow(dead_code)]
use crate::{
    document_code::CodeSnapshot,
    document_consumer_budget::{Charge, ConsumerBudget},
    document_model::Document,
    syntax::{FallbackReason, OperationContext, RetainedRoleSet, Viewport},
};
use std::{mem::size_of, ops::Range, sync::Arc, time::Instant};
const MAX_CODE_CACHE_ENTRIES: usize = 256;

/// Pure coverage descriptor, not evidence that a worker has returned Styles.
#[derive(Debug)]
pub(crate) struct SemanticCoverage {
    lines: Range<usize>,
    bytes: Range<usize>,
    input: Arc<crate::syntax::RetainedSyntaxInput>,
}
impl SemanticCoverage {
    pub(crate) fn from_request(
        snapshot: &CodeSnapshot,
        viewport: Option<Viewport>,
    ) -> Result<Self, FallbackReason> {
        let input = snapshot.input();
        let count = input.line_count().ok_or(FallbackReason::InvalidIpc)?;
        let (lines, bytes) = match viewport {
            None => (0..count, 0..input.len()),
            Some(v) => {
                if v.line_count == 0 || v.line_count as usize > crate::syntax_worker::MAX_VIEW_LINES
                {
                    return Err(FallbackReason::InvalidIpc);
                }
                let start = (v.first_line as usize).min(count);
                let end = (v.first_line as usize)
                    .checked_add(v.line_count as usize)
                    .ok_or(FallbackReason::InvalidIpc)?
                    .min(count);
                (
                    start..end,
                    input
                        .viewport_range(v.first_line, v.line_count)
                        .ok_or(FallbackReason::InvalidIpc)?,
                )
            }
        };
        Ok(Self {
            lines,
            bytes,
            input: input.clone(),
        })
    }
    pub(crate) fn contains_request(
        &self,
        snapshot: &CodeSnapshot,
        viewport: Option<Viewport>,
    ) -> Result<bool, FallbackReason> {
        let requested = Self::from_request(snapshot, viewport)?;
        Ok(Arc::ptr_eq(&self.input, &requested.input)
            && self.lines.start <= requested.lines.start
            && requested.lines.end <= self.lines.end
            && self.bytes.start <= requested.bytes.start
            && requested.bytes.end <= self.bytes.end)
    }
}

/// Immutable data is distinct from fresh activation. Fields remain private to the real producer.
struct OriginalColorProof {
    generation: u64,
    request: u64,
    viewport: Option<Viewport>,
    operation: OperationContext,
    accepted_at: Instant,
}
pub(crate) struct CachedCodeData {
    snapshot: Arc<CodeSnapshot>,
    roles: Arc<RetainedRoleSet>,
    coverage: SemanticCoverage,
    proof: OriginalColorProof,
    _owner: Charge,
}

pub(crate) struct SemanticCodeCache {
    entries: Vec<Option<Arc<CachedCodeData>>>,
    budget: ConsumerBudget,
    _owner: Charge,
    _slots: Charge,
    _overshoot: Charge,
}
impl SemanticCodeCache {
    pub(crate) fn new(budget: &ConsumerBudget, capacity: usize) -> Result<Self, FallbackReason> {
        if capacity > MAX_CODE_CACHE_ENTRIES {
            return Err(FallbackReason::SourceBudget);
        }
        let owner = budget
            .reserve(size_of::<Self>())
            .map_err(|_| FallbackReason::SourceBudget)?;
        let bytes = capacity
            .checked_mul(size_of::<Option<Arc<CachedCodeData>>>())
            .ok_or(FallbackReason::SourceBudget)?;
        let slots = budget
            .reserve(bytes)
            .map_err(|_| FallbackReason::SourceBudget)?;
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(capacity)
            .map_err(|_| FallbackReason::SourceBudget)?;
        let actual = entries
            .capacity()
            .checked_mul(size_of::<Option<Arc<CachedCodeData>>>())
            .ok_or(FallbackReason::SourceBudget)?;
        let overshoot = budget
            .reserve(actual.saturating_sub(bytes))
            .map_err(|_| FallbackReason::SourceBudget)?;
        // Logical length prevents growth beyond requested capacity, including allocator overshoot.
        entries.resize_with(capacity, || None);
        Ok(Self {
            entries,
            budget: budget.clone(),
            _owner: owner,
            _slots: slots,
            _overshoot: overshoot,
        })
    }
    pub(crate) fn lookup(
        &self,
        document: &Arc<Document>,
        block: u32,
        language: &str,
        viewport: Option<Viewport>,
    ) -> Option<&CachedCodeData> {
        self.entries
            .iter()
            .filter_map(|slot| slot.as_deref())
            .find(|entry| {
                entry
                    .snapshot
                    .key()
                    .matches_document_block(document, block, language)
                    && entry
                        .coverage
                        .contains_request(&entry.snapshot, viewport)
                        .unwrap_or(false)
            })
    }
    // Private: only the actual event producer in this module may call this in the next increment.
    // No public constructor of CachedCodeData/OriginalColorProof or synthetic test insertion.
    fn insert(&mut self, slot: usize, data: Arc<CachedCodeData>) -> Result<(), FallbackReason> {
        let input = data.snapshot.input();
        if !input
            .budget()
            .is_some_and(|budget| budget.same(&self.budget))
            || !std::ptr::eq(data.roles.input(), input.as_ref())
            || !Arc::ptr_eq(&data.coverage.input, input)
        {
            return Err(FallbackReason::InvalidIpc);
        }
        let target = self
            .entries
            .get_mut(slot)
            .ok_or(FallbackReason::SourceBudget)?;
        *target = Some(data);
        Ok(())
    }
    pub(crate) fn clear(&mut self) {
        for entry in &mut self.entries {
            *entry = None;
        }
    }
}
