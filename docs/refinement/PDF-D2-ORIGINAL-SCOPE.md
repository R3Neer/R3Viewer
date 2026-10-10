# D2 original scope / failed containment implementation mechanism

Private proposal, no implementation of this delta and no runtime. Current build5/source snapshot75C7897252DA72BBD5DF6CBB74F682AA93708D0DE06B2C806E3AFF5F02C28B32 remains immutable. Automatic review rejected same-call proposal+source edit; this note separates review before any edit.

## Contract preserved

Canonical PDF-D2-PROCESS-OWNER-design-r1 PC05-06 explicitly requires original-process cleanup when attach failed before membership: TerminateJob/JobClose does not prove that process stopped. PC concrete boundaries require original cancel/deadline checked on irreversible operations. This proposal changes representation/funding, not allowed profiles, mintage policy, original10s/cleanup2s, process count or registry lock policy. No PID/name rescue or new kernel.

## Closed ownership / exact signatures

In actor.rs, OriginalNativeScope private fields shared:Arc<SharedSessionActor>, core:Arc<NativeOwnerCore>, metadata:Lease. No Clone/Default/public constructor; NativeStartLease::capture_original_scope(&self)->Result<OriginalNativeScope,PdfCause> first calls check_live, reserves actual sizeof OriginalNativeScope, then clones original Shared/Core. This type is a retained original-clock scope only, cannot mint a NativeStartLease or grant process permission. OriginalNativeScope::check_live returns typed Cancelled/Deadline using original Shared.exit/Core.cancel/Core.deadline; no caller-selected Instant or cancel argument.

OwnedPdfProcess gains inline original_scope. Its metadata reservation deducts sizeof scope (already prepaid by that scope lease); original Arc allocations retain original core/Shared charges until last Drop. Nothing treats ActorArena identity as authorization. All owner fields remain private. ABI/captures/layout inventory invalidates and is recompiled/frozen before output; estimates remain estimates. New alias count and Pending/lastDrop become explicit AE criteria.

OwnedPdfProcess::resume_checked(&mut self,&PdfOneProcessJob)->Result<(),PdfCause> checks retained original_scope, actual original Job attach/readback/membership, checks SAME retained original_scope again before ResumeThread. Only count1 permits ordinary resume; -1/0/>1 retain original owner and do not authorize START. Deadline/cancel failures remain typed PdfCause, never Budget, no allocation/errorformat on rejected boundary. Current method taking only Job cannot validate original deadline by substituting another lease; the retained scope closes that gap.

OwnedPdfProcess::terminate_original(&self,exit:u32)->io::Result<()> uses TerminateProcess(self.process_handle()?,exit) on original owned process handle only. It performs no allocation, PID lookup, registration, new process, authority or deadline renewal. This is cleanup only, allowed after Cancelled/Deadline. It is not a Reaped proof: original WaitForSingleObject -> GetExitCodeProcess -> checked process/thread closes PLUS active0/checked Jobclose/IO+supervisor joins/loan return are still required. Failure leaves handles/capsule registered Pending. Only original backstop may call this for original registered core.

## Prospective oracles (supplement all original O3/O7/T/P)

1. Retain scope with original fixed deadline/cancel, change caller/current operation to unrelated live scope: resume still rejects old original expired/cancelled before ResumeThread/START; aliases/funding remain retained until owner Drop.
2. Scope metadata0/N-1 rejects before aliases; exact admits SAME original aliases, no new Arc allocation, no clock renewal. Header counts real before outputs, original parent owner metadata excludes only paid inline component.
3. Force attach failure while original suspended process is outside Job: closing/terminating Job alone never publishes Reaped. TerminateProcess and physical wait/checked closes on original process then qualify only their own receipts; owned failure and sticky cause persist.
4. Native original belongs to Job/resume count1 stays valid; cancelled/expired/exit at both irreversible checks denies START. Return0/>1/-1 keeps same owner and cleanup required.
5. Registry/capsule owns original process before any fallible attach/resume. This proposal does not accept current declaration-only registry, unminted leases or native spawn by compilation. No mock Ready, foreign Actor or fresh Work may substitute the real actor ingress.
