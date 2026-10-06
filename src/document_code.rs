//! Pure CodeText constructor only. Scheduler/background/renderer follow accepted D2b factories.
#![allow(dead_code)] // D4b partial increment: real scheduler is intentionally not fabricated.
use crate::{
    document_consumer_budget::{Charge, ChargedVec, ConsumerBudget},
    document_model::{Document, NONE, Profile},
    syntax::{FallbackReason, OperationContext, RetainedSyntaxInput},
    syntax_selection::select_explicit_token,
};
use std::{
    sync::{Arc, atomic::AtomicBool},
    time::Instant,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CodeSegment {
    pub run: u32,
    pub payload_start: u32,
    pub payload_end: u32,
}

pub(crate) struct CodeBlockKey {
    document: Arc<Document>,
    source_generation: u64,
    source_request: u64,
    source_hash: [u8; 32],
    profile: Profile,
    block: u32,
    domain: u32,
    language: &'static str,
    payload_hash: [u8; 32],
}
impl CodeBlockKey {
    pub(crate) fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.document, &other.document)
            && self.source_generation == other.source_generation
            && self.source_request == other.source_request
            && self.source_hash == other.source_hash
            && self.profile == other.profile
            && self.block == other.block
            && self.domain == other.domain
            && self.language == other.language
            && self.payload_hash == other.payload_hash
    }
    /// Descriptor identity only: immutable validated document ownership proves the payload.
    /// This does not authorize an operation, a receipt, or color publication.
    pub(crate) fn matches_document_block(
        &self,
        document: &Arc<Document>,
        block: u32,
        language: &str,
    ) -> bool {
        Arc::ptr_eq(&self.document, document)
            && self.source_generation == document.source().generation()
            && self.source_request == document.source().request()
            && self.source_hash == *document.source().hash()
            && self.profile == document.profile()
            && self.block == block
            && self.language == language
            && document
                .blocks()
                .get(block as usize)
                .is_some_and(|b| b.kind() == 10 && b.domain() == self.domain)
    }
    pub(crate) fn language(&self) -> &'static str {
        self.language
    }
    pub(crate) fn block(&self) -> u32 {
        self.block
    }
    pub(crate) fn domain(&self) -> u32 {
        self.domain
    }
    pub(crate) fn payload_hash(&self) -> &[u8; 32] {
        &self.payload_hash
    }
}

pub(crate) enum CodeConstruction {
    Plain { reason: Option<FallbackReason> },
    Ready(Arc<CodeSnapshot>),
}
pub(crate) struct CodeSnapshot {
    key: CodeBlockKey,
    input: Arc<RetainedSyntaxInput>,
    segments: ChargedVec<CodeSegment>,
    context: OperationContext,
    _owner: Charge,
}
impl CodeSnapshot {
    pub(crate) fn construct(
        document: &Arc<Document>,
        budget: &ConsumerBudget,
        block_id: u32,
        context: &OperationContext,
        color: bool,
    ) -> Result<CodeConstruction, FallbackReason> {
        Self::construct_with_stop(
            document,
            budget,
            block_id,
            context,
            color,
            &AtomicBool::new(false),
        )
    }
    /// The borrowed kernel stop is additional to the original operation, never its replacement.
    pub(crate) fn construct_with_stop(
        document: &Arc<Document>,
        budget: &ConsumerBudget,
        block_id: u32,
        context: &OperationContext,
        color: bool,
        local_stop: &AtomicBool,
    ) -> Result<CodeConstruction, FallbackReason> {
        RetainedSyntaxInput::check_code_operation(context, local_stop)?;
        if !budget.belongs_to(document) || !matches!(context, OperationContext::Document { .. }) {
            return Err(FallbackReason::InvalidIpc);
        }
        let block = document
            .blocks()
            .get(block_id as usize)
            .ok_or(FallbackReason::InvalidIpc)?;
        if block.kind() != 10 {
            return Err(FallbackReason::InvalidIpc);
        }
        if !color || block.aux0() != 1 || block.aux2() == NONE {
            return Ok(CodeConstruction::Plain { reason: None });
        }
        let token = document
            .text(block.aux2())
            .map_err(|_| FallbackReason::InvalidIpc)?;
        let Some(language) = select_explicit_token(token).filter(|&l| l != "Plain Text") else {
            return Ok(CodeConstruction::Plain {
                reason: Some(FallbackReason::UnknownLanguage),
            });
        };
        let domain = document
            .domains()
            .get(block.domain() as usize)
            .ok_or(FallbackReason::InvalidIpc)?;
        if domain.kind() != 0 || domain.owner() != block_id {
            return Err(FallbackReason::InvalidIpc);
        }
        if domain.run_count() == 0 {
            return Ok(CodeConstruction::Plain { reason: None });
        }
        let result = Self::build(document, budget, block_id, language, context, local_stop);
        match result {
            Ok(None) => Ok(CodeConstruction::Plain { reason: None }),
            Ok(Some(snapshot)) => Ok(CodeConstruction::Ready(Arc::new(snapshot))),
            Err(reason @ (FallbackReason::SourceBudget | FallbackReason::LongLine)) => {
                Ok(CodeConstruction::Plain {
                    reason: Some(reason),
                })
            }
            Err(reason) => Err(reason),
        }
    }
    fn build(
        document: &Arc<Document>,
        budget: &ConsumerBudget,
        block_id: u32,
        language: &'static str,
        context: &OperationContext,
        local_stop: &AtomicBool,
    ) -> Result<Option<Self>, FallbackReason> {
        let block = &document.blocks()[block_id as usize];
        let domain_id = block.domain();
        let domain = document
            .domains()
            .get(domain_id as usize)
            .ok_or(FallbackReason::InvalidIpc)?;
        if domain.kind() != 0 || domain.owner() != block_id {
            return Err(FallbackReason::InvalidIpc);
        }
        let Some(prepared) = RetainedSyntaxInput::copy_code_domain_with_stop(
            document, budget, domain_id, context, local_stop,
        )?
        else {
            return Ok(None);
        };
        let owner = prepared.snapshot_owner;
        let input = prepared.input;
        let segments = prepared.segments;
        let payload_hash = prepared.digest;
        RetainedSyntaxInput::check_code_operation(context, local_stop)?;
        Ok(Some(Self {
            key: CodeBlockKey {
                document: document.clone(),
                source_generation: document.source().generation(),
                source_request: document.source().request(),
                source_hash: *document.source().hash(),
                profile: document.profile(),
                block: block_id,
                domain: domain_id,
                language,
                payload_hash,
            },
            input,
            segments,
            context: context.clone(),
            _owner: owner,
        }))
    }
    pub(crate) fn key(&self) -> &CodeBlockKey {
        &self.key
    }
    pub(crate) fn input(&self) -> &Arc<RetainedSyntaxInput> {
        &self.input
    }
    pub(crate) fn segments(&self) -> &[CodeSegment] {
        self.segments.as_slice()
    }
    /// Pure mapping, no role lookup, no projected/source offsets. Context remains original.
    pub(crate) fn offset_for(
        &self,
        key: &CodeBlockKey,
        run: u32,
        local: usize,
        context: &OperationContext,
    ) -> Result<u32, FallbackReason> {
        context.check(Instant::now())?;
        if !self.context.same(context) {
            return Err(FallbackReason::InvalidIpc);
        }
        self.semantic_offset_for(key, run, local)
    }
    /// Read-only immutable mapping. No operation deadline or authority is granted here.
    pub(crate) fn semantic_offset_for(
        &self,
        key: &CodeBlockKey,
        run: u32,
        local: usize,
    ) -> Result<u32, FallbackReason> {
        if !self.key.same(key) {
            return Err(FallbackReason::InvalidIpc);
        }
        let domain = &self.key.document.domains()[self.key.domain as usize];
        let ordinal = run
            .checked_sub(domain.first_run())
            .ok_or(FallbackReason::InvalidIpc)? as usize;
        let segment = self
            .segments
            .as_slice()
            .get(ordinal)
            .filter(|s| s.run == run)
            .ok_or(FallbackReason::InvalidIpc)?;
        let row = self
            .key
            .document
            .runs()
            .get(run as usize)
            .ok_or(FallbackReason::InvalidIpc)?;
        let text = self
            .key
            .document
            .text(row.text())
            .map_err(|_| FallbackReason::InvalidIpc)?;
        if local >= text.len() || !text.is_char_boundary(local) {
            return Err(FallbackReason::InvalidIpc);
        }
        segment
            .payload_start
            .checked_add(u32::try_from(local).map_err(|_| FallbackReason::InvalidIpc)?)
            .filter(|&n| n < segment.payload_end)
            .ok_or(FallbackReason::InvalidIpc)
    }
}

#[cfg(test)]
#[path = "document_code_tests.rs"]
mod tests;
