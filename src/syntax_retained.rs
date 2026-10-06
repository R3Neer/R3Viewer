//! Retained syntax input and host-side admission. Never a second document budget.
use super::{FallbackReason, RoleSpan};
use crate::{
    document_consumer_budget::{Charge, ChargedVec, ConsumerBudget},
    document_model::Document,
};
use std::{
    mem::size_of,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

#[derive(Clone)]
pub(crate) enum OperationContext {
    LegacySource,
    #[allow(dead_code)] // Public crate-facing D4a API consumed by D4b next.
    Document {
        id: u64,
        deadline: Instant,
        cancel: Arc<AtomicBool>,
    },
}
impl OperationContext {
    #[allow(dead_code)] // D4b operation entry follows independent D4a acceptance.
    pub(crate) fn document(id: u64, deadline: Instant, cancel: Arc<AtomicBool>) -> Self {
        Self::Document {
            id,
            deadline,
            cancel,
        }
    }
    pub(crate) fn check(&self, now: Instant) -> Result<(), FallbackReason> {
        if let Self::Document {
            deadline, cancel, ..
        } = self
        {
            if cancel.load(Ordering::Acquire) {
                return Err(FallbackReason::Cancelled);
            }
            if now >= *deadline {
                return Err(FallbackReason::RequestTimeout);
            }
        }
        Ok(())
    }
    pub(crate) fn phase(&self, now: Instant, duration: Duration) -> Instant {
        let phase = now.checked_add(duration).unwrap_or(now);
        match self {
            Self::Document { deadline, .. } => phase.min(*deadline),
            Self::LegacySource => phase,
        }
    }
    pub(crate) fn same(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::LegacySource, Self::LegacySource) => true,
            (
                Self::Document {
                    id: a,
                    deadline: ad,
                    cancel: ac,
                },
                Self::Document {
                    id: b,
                    deadline: bd,
                    cancel: bc,
                },
            ) => a == b && ad == bd && Arc::ptr_eq(ac, bc),
            _ => false,
        }
    }
}

pub(crate) struct RetainedSyntaxInput {
    source: Arc<String>,
    budget: Option<ConsumerBudget>,
    lines: Option<ChargedVec<u32>>,
    _source: Option<Charge>,
    _overshoot: Option<Charge>,
    _owner: Option<Charge>,
}
impl std::fmt::Debug for RetainedSyntaxInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RetainedSyntaxInput")
            .field("bytes", &self.source.len())
            .finish_non_exhaustive()
    }
}
impl RetainedSyntaxInput {
    pub(crate) fn check_code_operation(
        context: &OperationContext,
        local_stop: &AtomicBool,
    ) -> Result<(), FallbackReason> {
        check_code_operation(context, local_stop)
    }
    #[cfg(test)]
    pub(crate) fn own_local_stop_after_copy(bytes: Option<usize>) {
        own_local_stop_after_copy(bytes);
    }

    pub(crate) fn legacy(source: Arc<String>) -> Arc<Self> {
        Arc::new(Self {
            source,
            budget: None,
            lines: None,
            _source: None,
            _overshoot: None,
            _owner: None,
        })
    }
    /// The copy is optional color input: failure never reduces the original document.
    /// Caller must supply the SAME budget that its document view owns.
    #[allow(dead_code)] // D4b caller follows after D4a independently accepted.
    pub(crate) fn copy_document(
        source: &str,
        document: &Arc<Document>,
        budget: &ConsumerBudget,
        context: &OperationContext,
    ) -> Result<Arc<Self>, FallbackReason> {
        context.check(Instant::now())?;
        if !budget.belongs_to(document) {
            return Err(FallbackReason::InvalidIpc);
        }
        if source.len() > crate::syntax_worker::MAX_SOURCE
            || source.len()
                > crate::document_consumer_budget::CONSUMER_LIMIT.saturating_sub(budget.used())
        {
            return Err(FallbackReason::SourceBudget);
        }
        let mut count = 1usize;
        for chunk in source.as_bytes().chunks(4096) {
            context.check(Instant::now())?;
            count = count
                .checked_add(chunk.iter().filter(|&&b| b == b'\n').count())
                .ok_or(FallbackReason::SourceBudget)?;
        }
        let owner = budget
            .reserve(size_of::<Self>() + 4 * size_of::<usize>() + size_of::<String>())
            .map_err(|_| FallbackReason::SourceBudget)?;
        let charge = budget
            .reserve(source.len())
            .map_err(|_| FallbackReason::SourceBudget)?;
        let mut text = String::new();
        text.try_reserve_exact(source.len())
            .map_err(|_| FallbackReason::SourceBudget)?;
        let overshoot = budget
            .reserve(text.capacity().saturating_sub(source.len()))
            .map_err(|_| FallbackReason::SourceBudget)?;
        let mut lines = ChargedVec::new(budget, count).map_err(|_| FallbackReason::SourceBudget)?;
        lines.push(0).map_err(|_| FallbackReason::SourceBudget)?;
        let mut at = 0;
        while at < source.len() {
            context.check(Instant::now())?;
            let mut end = (at + 4096).min(source.len());
            while !source.is_char_boundary(end) {
                end -= 1;
            }
            for (local, b) in source.as_bytes()[at..end].iter().enumerate() {
                if *b == b'\n' {
                    lines
                        .push(
                            u32::try_from(at + local + 1)
                                .map_err(|_| FallbackReason::SourceBudget)?,
                        )
                        .map_err(|_| FallbackReason::SourceBudget)?;
                }
            }
            text.push_str(&source[at..end]);
            at = end;
        }
        context.check(Instant::now())?;
        Ok(Arc::new(Self {
            source: Arc::new(text),
            budget: Some(budget.clone()),
            lines: Some(lines),
            _source: Some(charge),
            _overshoot: Some(overshoot),
            _owner: Some(owner),
        }))
    }
    /// One complete copy of validated CodeText, never an intermediate flat String.
    /// Domain is validated again at this boundary; all allocations use the input's document budget.
    #[allow(dead_code)] // Retained compatibility entry; kernel uses borrowed stop-aware entry.
    pub(crate) fn copy_code_domain(
        document: &Arc<Document>,
        budget: &ConsumerBudget,
        domain_id: u32,
        context: &OperationContext,
    ) -> Result<Option<CodeDomainCopy>, FallbackReason> {
        Self::copy_code_domain_with_stop(
            document,
            budget,
            domain_id,
            context,
            &AtomicBool::new(false),
        )
    }
    pub(crate) fn copy_code_domain_with_stop(
        document: &Arc<Document>,
        budget: &ConsumerBudget,
        domain_id: u32,
        context: &OperationContext,
        local_stop: &AtomicBool,
    ) -> Result<Option<CodeDomainCopy>, FallbackReason> {
        use sha2::{Digest, Sha256};
        check_code_operation(context, local_stop)?;
        if !budget.belongs_to(document) || !matches!(context, OperationContext::Document { .. }) {
            return Err(FallbackReason::InvalidIpc);
        }
        let domain = document
            .domains()
            .get(domain_id as usize)
            .ok_or(FallbackReason::InvalidIpc)?;
        let block = document
            .blocks()
            .get(domain.owner() as usize)
            .ok_or(FallbackReason::InvalidIpc)?;
        if domain.kind() != 0
            || block.kind() != 10
            || block.domain() != domain_id
            || block.aux0() != 1
        {
            return Err(FallbackReason::InvalidIpc);
        }
        let end_run = (domain.first_run() as usize)
            .checked_add(domain.run_count() as usize)
            .ok_or(FallbackReason::InvalidIpc)?;
        let runs = document
            .runs()
            .get(domain.first_run() as usize..end_run)
            .ok_or(FallbackReason::InvalidIpc)?;
        let mut segments = ChargedVec::new(budget, domain.run_count() as usize)
            .map_err(|_| FallbackReason::SourceBudget)?;
        let mut total = 0usize;
        let mut count = 1usize;
        let mut line_bytes = 0usize;
        for (ordinal, run) in runs.iter().enumerate() {
            check_code_operation(context, local_stop)?;
            #[cfg(test)]
            CODE_DOMAIN_STATS.with(|slot| {
                let mut n = slot.get();
                n[0] += 1;
                slot.set(n);
            });
            if run.kind() != 6 || run.domain() != domain_id || run.ordinal() as usize != ordinal {
                return Err(FallbackReason::InvalidIpc);
            }
            let part = document
                .text(run.text())
                .map_err(|_| FallbackReason::InvalidIpc)?;
            let prefix = total;
            total = total
                .checked_add(part.len())
                .filter(|&n| n <= crate::syntax_worker::MAX_SOURCE)
                .ok_or(FallbackReason::SourceBudget)?;
            segments
                .push(crate::document_code::CodeSegment {
                    run: run.id(),
                    payload_start: prefix as u32,
                    payload_end: total as u32,
                })
                .map_err(|_| FallbackReason::SourceBudget)?;
            let mut at = 0;
            while at < part.len() {
                check_code_operation(context, local_stop)?;
                let end = utf8_chunk_end(part, at);
                for &b in &part.as_bytes()[at..end] {
                    line_bytes = line_bytes.checked_add(1).ok_or(FallbackReason::LongLine)?;
                    if line_bytes > crate::syntax_worker::MAX_LINE {
                        return Err(FallbackReason::LongLine);
                    }
                    if b == b'\n' {
                        count = count.checked_add(1).ok_or(FallbackReason::SourceBudget)?;
                        line_bytes = 0;
                    }
                }
                at = end;
            }
        }
        check_code_operation(context, local_stop)?;
        if total == 0 {
            return Ok(None);
        }
        // Pre-admit eventual snapshot metadata only AFTER the first pass proves nonempty,
        // and BEFORE allocating/filling its retained input. The exact concrete type closes the charge.
        let snapshot_owner = budget
            .reserve(size_of::<crate::document_code::CodeSnapshot>() + 2 * size_of::<usize>())
            .map_err(|_| FallbackReason::SourceBudget)?;
        let owner = budget
            .reserve(size_of::<Self>() + 4 * size_of::<usize>() + size_of::<String>())
            .map_err(|_| FallbackReason::SourceBudget)?;
        let charge = budget
            .reserve(total)
            .map_err(|_| FallbackReason::SourceBudget)?;
        let mut text = String::new();
        text.try_reserve_exact(total)
            .map_err(|_| FallbackReason::SourceBudget)?;
        let overshoot = budget
            .reserve(text.capacity().saturating_sub(total))
            .map_err(|_| FallbackReason::SourceBudget)?;
        let mut lines = ChargedVec::new(budget, count).map_err(|_| FallbackReason::SourceBudget)?;
        lines.push(0).map_err(|_| FallbackReason::SourceBudget)?;
        let mut hash = Sha256::new();
        for run in runs {
            check_code_operation(context, local_stop)?;
            #[cfg(test)]
            CODE_DOMAIN_STATS.with(|slot| {
                let mut n = slot.get();
                n[1] += 1;
                slot.set(n);
            });
            let part = document
                .text(run.text())
                .map_err(|_| FallbackReason::InvalidIpc)?;
            let mut at = 0;
            while at < part.len() {
                code_copy_checkpoint(context)?;
                check_code_operation(context, local_stop)?;
                let end = utf8_chunk_end(part, at);
                let base = text.len();
                for (local, &b) in part.as_bytes()[at..end].iter().enumerate() {
                    if b == b'\n' {
                        lines
                            .push(
                                u32::try_from(base + local + 1)
                                    .map_err(|_| FallbackReason::SourceBudget)?,
                            )
                            .map_err(|_| FallbackReason::SourceBudget)?;
                    }
                }
                text.push_str(&part[at..end]);
                #[cfg(test)]
                CODE_COPY_BYTES.with(|slot| slot.set(slot.get() + end - at));
                hash.update(&part.as_bytes()[at..end]);
                at = end;
            }
        }
        check_code_operation(context, local_stop)?;
        if text.len() != total || lines.as_slice().len() != count {
            return Err(FallbackReason::InvalidIpc);
        }
        let digest = hash.finalize().into();
        #[cfg(test)]
        CODE_DOMAIN_STATS.with(|slot| {
            let mut n = slot.get();
            n[2] += 1;
            slot.set(n);
        });
        Ok(Some(CodeDomainCopy {
            input: Arc::new(Self {
                source: Arc::new(text),
                budget: Some(budget.clone()),
                lines: Some(lines),
                _source: Some(charge),
                _overshoot: Some(overshoot),
                _owner: Some(owner),
            }),
            segments,
            digest,
            snapshot_owner,
        }))
    }
    #[cfg(test)]
    pub(crate) fn own_copy_cancel_after(chunks: Option<usize>) {
        CODE_COPY_CANCEL_AFTER.with(|slot| slot.set(chunks));
        CODE_COPY_BYTES.with(|slot| slot.set(0));
    }
    #[cfg(test)]
    pub(crate) fn own_domain_stats(reset: bool) -> [usize; 3] {
        CODE_DOMAIN_STATS.with(|slot| {
            let n = slot.get();
            if reset {
                slot.set([0; 3]);
            }
            n
        })
    }
    #[cfg(test)]
    pub(crate) fn own_copy_bytes() -> usize {
        CODE_COPY_BYTES.with(|slot| slot.get())
    }
    pub(crate) fn text(&self) -> &str {
        &self.source
    }
    pub(crate) fn len(&self) -> usize {
        self.source.len()
    }
    /// Document inputs have charged precomputed line starts; Legacy has no such table.
    pub(crate) fn line_count(&self) -> Option<usize> {
        self.lines.as_ref().map(|lines| lines.as_slice().len())
    }
    pub(crate) fn budget(&self) -> Option<&ConsumerBudget> {
        self.budget.as_ref()
    }
    pub(crate) fn viewport_range(&self, first: u32, count: u16) -> Option<std::ops::Range<usize>> {
        let lines = self.lines.as_ref()?;
        let start = lines
            .as_slice()
            .get(first as usize)
            .copied()
            .unwrap_or(self.len() as u32) as usize;
        let end = lines
            .as_slice()
            .get((first as usize).checked_add(count as usize)?)
            .copied()
            .unwrap_or(self.len() as u32) as usize;
        Some(start..end)
    }
}
impl std::ops::Deref for RetainedSyntaxInput {
    type Target = str;
    fn deref(&self) -> &str {
        self.text()
    }
}

pub(crate) struct CodeDomainCopy {
    pub(crate) input: Arc<RetainedSyntaxInput>,
    pub(crate) segments: ChargedVec<crate::document_code::CodeSegment>,
    pub(crate) digest: [u8; 32],
    pub(crate) snapshot_owner: Charge,
}

pub(crate) struct RetainedRoleSet {
    allocation: crate::syntax_worker::OwnedRoleAllocation,
    _owner: Option<Charge>,
}
impl std::fmt::Debug for RetainedRoleSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RetainedRoleSet")
            .field("spans", &self.spans().len())
            .finish_non_exhaustive()
    }
}
impl RetainedRoleSet {
    // Only the syntax supervisor may commit after semantic/binding validation.
    pub(super) fn from_receipt(
        allocation: crate::syntax_worker::OwnedRoleAllocation,
        input: &Arc<RetainedSyntaxInput>,
        generation: u64,
        request: u64,
    ) -> Result<Arc<Self>, FallbackReason> {
        allocation
            .verify_owner(input, generation, request)
            .map_err(|_| FallbackReason::InvalidIpc)?;
        Self::admit_owner(allocation)
    }
    #[cfg(test)]
    pub(super) fn legacy(
        spans: Vec<RoleSpan>,
        input: Arc<RetainedSyntaxInput>,
    ) -> Result<Arc<Self>, FallbackReason> {
        let allocation = crate::syntax_worker::OwnedRoleAllocation::legacy(spans, input)
            .map_err(|_| FallbackReason::InvalidIpc)?;
        Self::admit_owner(allocation)
    }
    fn admit_owner(
        allocation: crate::syntax_worker::OwnedRoleAllocation,
    ) -> Result<Arc<Self>, FallbackReason> {
        let owner = allocation
            .input()
            .budget()
            .map(|b| {
                b.reserve(size_of::<Self>() + 2 * size_of::<usize>())
                    .map_err(|_| FallbackReason::SpanBudget)
            })
            .transpose()?;
        Ok(Arc::new(Self {
            allocation,
            _owner: owner,
        }))
    }
    #[cfg(test)]
    pub(super) fn empty_for_test(input: Arc<RetainedSyntaxInput>) -> Arc<Self> {
        use crate::syntax_worker::{Frame, Message, read_host_frame, write_frame};
        let mut wire = Vec::new();
        write_frame(&mut wire, &Frame::new(0, 0, Message::Styles(Vec::new()))).unwrap();
        let mut receipt = read_host_frame(&mut wire.as_slice(), &input, 1).unwrap();
        Self::from_receipt(receipt.take_roles().unwrap(), &input, 0, 0).unwrap()
    }
    pub(crate) fn spans(&self) -> &[RoleSpan] {
        self.allocation.spans()
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        self.allocation.capacity_bytes()
    }
    pub(crate) fn input(&self) -> &RetainedSyntaxInput {
        self.allocation.input()
    }
}
#[derive(Debug)]
pub(crate) enum RetainedSyntaxEvent {
    Ready {
        generation: u64,
    },
    Styles {
        generation: u64,
        request: u64,
        viewport: Option<super::Viewport>,
        roles: Arc<RetainedRoleSet>,
    },
    Plain {
        generation: u64,
        reason: FallbackReason,
    },
}

#[cfg(test)]
pub(crate) fn fixture_document() -> Arc<Document> {
    use crate::{
        document_budget::ActorArena, document_model::Profile, document_obsidian::Control,
        document_source::DocumentSource, input::SourceOrigin,
    };
    let cancel = AtomicBool::new(false);
    let ctl = Control {
        cancel: &cancel,
        deadline: Instant::now() + Duration::from_secs(10),
    };
    let source = Arc::new(
        DocumentSource::new(
            Arc::new("text\n".into()),
            SourceOrigin::Literal,
            1,
            1,
            &cancel,
        )
        .unwrap(),
    );
    let c = crate::document_parser::project(source, Profile::Extended, ActorArena::new(), &ctl)
        .unwrap();
    let depth = crate::document_validate::validate(&c, &ctl).unwrap();
    Arc::new(Document::from_validated(
        crate::document_validate::quarantine(c, depth, &ctl).unwrap(),
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deadline_is_original_across_phase_queue_supersede_and_legacy_is_separate() {
        let start = Instant::now();
        let stop = Arc::new(AtomicBool::new(false));
        let ctx = OperationContext::document(19, start + Duration::from_secs(5), stop.clone());
        assert!(ctx.check(start + Duration::from_secs(4)).is_ok());
        assert_eq!(
            ctx.check(start + Duration::from_secs(5)),
            Err(FallbackReason::RequestTimeout)
        );
        assert_eq!(
            ctx.phase(start + Duration::from_secs(4), Duration::from_secs(5)),
            start + Duration::from_secs(5)
        );
        assert!(ctx.same(&ctx.clone()));
        assert!(!ctx.same(&OperationContext::document(
            20,
            start + Duration::from_secs(5),
            stop.clone()
        )));
        assert!(
            OperationContext::LegacySource
                .check(start + Duration::from_secs(500))
                .is_ok()
        );
        stop.store(true, Ordering::Release);
        assert_eq!(ctx.check(start), Err(FallbackReason::Cancelled));
    }
    #[test]
    fn snapshot_exact_quota_and_plus_one_preserve_original() {
        let doc = fixture_document();
        let budget = ConsumerBudget::new(&doc).unwrap();
        let base = budget.used();
        let ctx = OperationContext::document(
            1,
            Instant::now() + Duration::from_secs(5),
            Arc::new(AtomicBool::new(false)),
        );
        let text = "aé\r\n\x1bz\n";
        let required = size_of::<RetainedSyntaxInput>()
            + 4 * size_of::<usize>()
            + size_of::<String>()
            + text.len()
            + 3 * size_of::<u32>();
        let block = budget
            .reserve(crate::document_consumer_budget::CONSUMER_LIMIT - base - required)
            .unwrap();
        let input = RetainedSyntaxInput::copy_document(text, &doc, &budget, &ctx).unwrap();
        assert_eq!(input.text(), text);
        assert_eq!(
            budget.used(),
            crate::document_consumer_budget::CONSUMER_LIMIT
        );
        assert_eq!(input.viewport_range(1, 1), Some(5..8));
        drop(input);
        drop(block);
        assert_eq!(budget.used(), base);
        let block = budget
            .reserve(crate::document_consumer_budget::CONSUMER_LIMIT - base - required + 1)
            .unwrap();
        assert!(matches!(
            RetainedSyntaxInput::copy_document(text, &doc, &budget, &ctx),
            Err(FallbackReason::SourceBudget)
        ));
        drop(block);
        assert_eq!(budget.used(), base);
        assert_eq!(doc.source().text(), "text\n");
    }
    #[test]
    fn roles_hold_input_and_all_charges_until_last_alias() {
        let doc = fixture_document();
        let budget = ConsumerBudget::new(&doc).unwrap();
        let base = budget.used();
        let ctx = OperationContext::document(
            1,
            Instant::now() + Duration::from_secs(5),
            Arc::new(AtomicBool::new(false)),
        );
        let input = RetainedSyntaxInput::copy_document("é\n", &doc, &budget, &ctx).unwrap();
        let weak = Arc::downgrade(&input);
        let spans = vec![RoleSpan {
            start: 0,
            end: 3,
            role: super::super::SyntaxRole::Success,
        }];
        let mut bytes = Vec::new();
        crate::syntax_worker::write_frame(
            &mut bytes,
            &crate::syntax_worker::Frame::new(1, 1, crate::syntax_worker::Message::Styles(spans)),
        )
        .unwrap();
        let mut received =
            crate::syntax_worker::read_host_frame(&mut bytes.as_slice(), &input, 1).unwrap();
        let roles =
            RetainedRoleSet::from_receipt(received.take_roles().unwrap(), &input, 1, 1).unwrap();
        let oldframe = roles.clone();
        drop(input);
        drop(roles);
        assert!(weak.upgrade().is_some());
        assert!(budget.used() > base);
        assert_eq!(oldframe.spans()[0].end, 3);
        drop(oldframe);
        assert!(weak.upgrade().is_none());
        assert_eq!(budget.used(), base);
    }
    fn decoded(
        input: &Arc<RetainedSyntaxInput>,
        generation: u64,
        request: u64,
    ) -> crate::syntax_worker::OwnedRoleAllocation {
        use crate::syntax_worker::{Frame, Message, read_host_frame, write_frame};
        let mut wire = Vec::new();
        write_frame(
            &mut wire,
            &Frame::new(
                generation,
                request,
                Message::Styles(vec![RoleSpan {
                    start: 0,
                    end: 1,
                    role: super::super::SyntaxRole::Value,
                }]),
            ),
        )
        .unwrap();
        read_host_frame(&mut wire.as_slice(), input, 1)
            .unwrap()
            .take_roles()
            .unwrap()
    }
    fn retained_input(doc: &Arc<Document>, budget: &ConsumerBudget) -> Arc<RetainedSyntaxInput> {
        let context = OperationContext::document(
            1,
            Instant::now() + Duration::from_secs(5),
            Arc::new(AtomicBool::new(false)),
        );
        RetainedSyntaxInput::copy_document("x", doc, budget, &context).unwrap()
    }
    #[test]
    fn owner01_document_raw_capacity_cannot_enter_without_decoder_admission() {
        let doc = fixture_document();
        let budget = ConsumerBudget::new(&doc).unwrap();
        let input = retained_input(&doc, &budget);
        let held = budget
            .reserve(crate::document_consumer_budget::CONSUMER_LIMIT - budget.used())
            .unwrap();
        let spans = Vec::<RoleSpan>::with_capacity(1000);
        assert_eq!(spans.capacity(), 1000);
        assert!(matches!(
            RetainedRoleSet::legacy(spans, input.clone()),
            Err(FallbackReason::InvalidIpc)
        ));
        assert_eq!(
            budget.used(),
            crate::document_consumer_budget::CONSUMER_LIMIT
        );
        assert!(matches!(
            RetainedRoleSet::legacy(Vec::new(), input),
            Err(FallbackReason::InvalidIpc)
        ));
        drop(held);
    }
    #[test]
    fn owner01_receipt_rejects_foreign_budget_and_foreign_input_even_equal_content() {
        let doc = fixture_document();
        let a = ConsumerBudget::new(&doc).unwrap();
        let b = ConsumerBudget::new(&doc).unwrap();
        let ia = retained_input(&doc, &a);
        let ib = retained_input(&doc, &b);
        let ia2 = retained_input(&doc, &a);
        let before = a.used();
        let before_b = b.used();
        assert!(matches!(
            RetainedRoleSet::from_receipt(decoded(&ib, 7, 11), &ia, 7, 11),
            Err(FallbackReason::InvalidIpc)
        ));
        assert_eq!(a.used(), before);
        assert_eq!(b.used(), before_b);
        assert!(matches!(
            RetainedRoleSet::from_receipt(decoded(&ia2, 7, 11), &ia, 7, 11),
            Err(FallbackReason::InvalidIpc)
        ));
        assert_eq!(a.used(), before);
        for (generation, request) in [(8, 11), (7, 12)] {
            assert!(matches!(
                RetainedRoleSet::from_receipt(decoded(&ia, 7, 11), &ia, generation, request),
                Err(FallbackReason::InvalidIpc)
            ));
            assert_eq!(a.used(), before);
        }
    }
    #[test]
    fn owner01_full_consumer_rejects_decode_and_empty_receipt_is_bound_without_role_allocation() {
        use crate::syntax_worker::{Frame, Message, read_host_frame, write_frame};
        let doc = fixture_document();
        let budget = ConsumerBudget::new(&doc).unwrap();
        let input = retained_input(&doc, &budget);
        let before = budget.used();
        let held = budget
            .reserve(crate::document_consumer_budget::CONSUMER_LIMIT - before)
            .unwrap();
        let mut wire = Vec::new();
        write_frame(
            &mut wire,
            &Frame::new(
                4,
                5,
                Message::Styles(vec![RoleSpan {
                    start: 0,
                    end: 1,
                    role: super::super::SyntaxRole::Value,
                }]),
            ),
        )
        .unwrap();
        let mut cursor = std::io::Cursor::new(wire);
        assert_eq!(
            read_host_frame(&mut cursor, &input, 1)
                .err()
                .unwrap()
                .kind(),
            std::io::ErrorKind::OutOfMemory
        );
        assert_eq!(cursor.position(), 32);
        assert_eq!(
            budget.used(),
            crate::document_consumer_budget::CONSUMER_LIMIT
        );
        drop(held);
        assert_eq!(budget.used(), before);
        let empty = RetainedRoleSet::empty_for_test(input);
        assert_eq!(empty.retained_bytes(), 0);
        assert!(empty.spans().is_empty());
        drop(empty);
    }
    #[test]
    fn wrong_budget_and_expired_input_do_not_allocate() {
        let a = fixture_document();
        let b = fixture_document();
        let budget = ConsumerBudget::new(&a).unwrap();
        let base = budget.used();
        let now = Instant::now();
        let active = OperationContext::document(
            1,
            now + Duration::from_secs(5),
            Arc::new(AtomicBool::new(false)),
        );
        assert!(matches!(
            RetainedSyntaxInput::copy_document("x", &b, &budget, &active),
            Err(FallbackReason::InvalidIpc)
        ));
        let expired = OperationContext::document(2, now, Arc::new(AtomicBool::new(false)));
        assert!(matches!(
            RetainedSyntaxInput::copy_document("x", &a, &budget, &expired),
            Err(FallbackReason::RequestTimeout)
        ));
        assert_eq!(budget.used(), base);
    }
}

/// Advance at most4096 bytes, preserving a complete UTF8 boundary without scanning the prefix.
fn utf8_chunk_end(text: &str, at: usize) -> usize {
    let mut end = at.saturating_add(4096).min(text.len());
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    end
}

#[cfg(test)]
std::thread_local! {
    static CODE_COPY_CANCEL_AFTER: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
    static CODE_LOCAL_STOP_AFTER: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
    static CODE_COPY_BYTES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static CODE_DOMAIN_STATS: std::cell::Cell<[usize;3]> = const {std::cell::Cell::new([0;3])};
}
fn code_copy_checkpoint(context: &OperationContext) -> Result<(), FallbackReason> {
    #[cfg(test)]
    CODE_COPY_CANCEL_AFTER.with(|slot| {
        if let Some(remaining) = slot.get() {
            if remaining == 0 {
                if let OperationContext::Document { cancel, .. } = context {
                    cancel.store(true, Ordering::Release);
                }
            } else {
                slot.set(Some(remaining - 1));
            }
        }
    });
    context.check(Instant::now())
}

/// Check the real operation first, then the owner-local priority stop. Neither is renewed.
pub(crate) fn check_code_operation(
    context: &OperationContext,
    local_stop: &AtomicBool,
) -> Result<(), FallbackReason> {
    #[cfg(test)]
    CODE_LOCAL_STOP_AFTER.with(|slot| {
        if slot
            .get()
            .is_some_and(|n| CODE_COPY_BYTES.with(|bytes| bytes.get() >= n))
        {
            local_stop.store(true, Ordering::Release);
        }
    });
    context.check(Instant::now())?;
    if local_stop.load(Ordering::Acquire) {
        return Err(FallbackReason::Cancelled);
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn own_local_stop_after_copy(bytes: Option<usize>) {
    CODE_LOCAL_STOP_AFTER.with(|slot| slot.set(bytes));
    CODE_COPY_BYTES.with(|slot| slot.set(0));
}
