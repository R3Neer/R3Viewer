//! Capacity planning only. Each actual allocation still obtains a shared budget charge.
use crate::{
    document_consumer_budget::{CONSUMER_LIMIT, ConsumerBudget},
    document_flow::FlowIndex,
    document_text::{Error, IndexProfile, Result},
};
#[derive(Clone, Copy, Debug)]
pub struct ConsumerProfile {
    pub flow_slots: usize,
    pub index: IndexProfile,
    pub index_slots: usize,
    pub requested_bytes: usize,
    pub checkpoint_capacity: usize,
}
impl ConsumerProfile {
    /// Caller reserves mandatory current/history/Reference/window/old+new before this optional plan.
    /// Stale plans cannot bypass admission: prepare/ChargedVec still reserve actual capacities.
    pub fn plan(
        budget: &ConsumerBudget,
        desired_flow: usize,
        desired_index: IndexProfile,
        index_slots: usize,
    ) -> Result<Self> {
        if !(2..=4096).contains(&desired_flow) || !(1..=2).contains(&index_slots) {
            return Err(Error::Invalid);
        }
        let mut flow = desired_flow;
        let mut index = desired_index.checked()?;
        let bytes = CONSUMER_LIMIT
            .checked_sub(budget.used())
            .ok_or(Error::Budget)?;
        let points = 4096usize
            .checked_sub(budget.checkpoint_capacity())
            .ok_or(Error::Budget)?;
        loop {
            let requested = flow
                .checked_mul(FlowIndex::checkpoint_size())
                .and_then(|n| n.checked_add(FlowIndex::fixed_requested_bytes()))
                .and_then(|n| {
                    index
                        .requested_bytes()
                        .ok()
                        .and_then(|b| b.checked_add(IndexProfile::fixed_requested_bytes()))
                        .and_then(|b| b.checked_mul(index_slots))
                        .and_then(|b| n.checked_add(b))
                })
                .ok_or(Error::Budget)?;
            let count = flow
                .checked_add(2)
                .and_then(|n| {
                    index
                        .checkpoint_capacity()
                        .ok()
                        .and_then(|p| p.checked_mul(index_slots))
                        .and_then(|p| n.checked_add(p))
                })
                .ok_or(Error::Budget)?;
            if requested <= bytes && count <= points {
                return Ok(Self {
                    flow_slots: flow,
                    index,
                    index_slots,
                    requested_bytes: requested,
                    checkpoint_capacity: count,
                });
            }
            // Reduce optional density, preserving logical endpoints and exact exception semantics.
            if flow > 2 {
                flow = flow.div_ceil(2).max(2);
            } else if index.projection_points > 2 || index.unicode_points > 2 {
                index.projection_points = index.projection_points.div_ceil(2).max(2);
                index.unicode_points = index.unicode_points.div_ceil(2).max(2);
            } else {
                return Err(Error::Budget);
            }
        }
    }
}
