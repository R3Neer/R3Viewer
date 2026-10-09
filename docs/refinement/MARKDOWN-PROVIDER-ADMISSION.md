# Provider ledger R2 — concrete library boundary before inspector freeze

2026-10-09. Design only; no admission implementation or affected runtime. R1
65E1EE65 and review IP-P01/IP-P02 remain preserved. Base canonical 3db799c
(search R2 + accepted b2); inspector private composition corresponds to it.
19 AUTHOR controls pass independently of this open gate; not adversary acceptance.

## Scope and concrete adapter (IP-P01)

Add document_provider_gate.rs and document_inspection_preparation.rs. This is a
closed library adapter on existing DocumentView/DocumentPreparationInput and
PreparationAuthority<DI,DO>/OwnedPreparation, not a new session, kernel, worker,
thread or authority instance. It must compile and be exercised BEFORE inspector
freeze; it is not an assurance deferred to pump/D2c.

An inspectable initial view is created by
DocumentView::inspectable_initial_input(document,geometry). Shared helper of the
current initial_input creates the ACTUAL ConsumerBudget, source context and
source metadata once. The inspectable branch additionally creates ProviderGate
from those actual fields before any layout provider. Gate is retained in input
and final DocumentView; each subsequent view input aliases that SAME gate.
Existing initial_input remains legacy, outside this component guarantee; no
silent import of its uncapped providers. Inspector requires an inspectable view.
A legacy view returns typed ScopePending; migration is explicit before product
adoption, not a claim that every legacy caller is capped. Existing D1 factories
and legacy byte/point/profile limits remain unchanged.

Concrete input/output sums:
InspectionLibraryInput = Viewport(Arc<DocumentPreparationInput>) |
 Search(ScopedSearchInput) | Inspect(InspectionPreparationInput) |
 RetireAndInspect(CacheRetirement,InspectionPreparationInput).
InspectionLibraryOutput = Viewport(PreparedState) |
 Search(ScopedSearchOutcome) | Inspector(PreparedInspection).
The type alias specializes the SAME PreparationAuthority<InspectionLibraryInput,
InspectionLibraryOutput>; it replaces the Document-only specialization for this
library. There is ONE authority object with sequential jobs. start_inspection_job
uses its existing start_document_owner and OwnedPreparation::start. Viewport
calls existing run_prepare; Search calls the scoped core branch; Inspect calls
the actual inspector preparation. poll/take_validated(expected Binding,false)
and joined receipt are existing checks, not a fabricated completion. Generic
Source methods on the same kernel remain available; common Source work is being
implemented separately and is not copied here. No second active owner instance.

Capture constructors receive a REAL view, real current/cell anchor or sealed
SharedHit, and real gate. They keep actual source context/metadata leases and
view binding; query revision stays independent. Capture validates sealed input
anchor and view identities, charges the complete sum/input body/Arc headers
and any retained alias metadata BEFORE copying. It does not build a provider.
PreparedInspector contains the complete result or typed Pending. Publication
requires matching the original closed view Binding/query identities and the
actual joined completion, keeping old inspection/page/current untouched on
failure/stale/cancel. Gate guarantees the physical count even before publication.

## Gate state and per-provider ownership (IP-P02)

ProviderGate is a non-public Arc<GateInner>. GateInner holds actual Document Arc,
ConsumerBudget, source context/metadata aliases, a fresh internal identity and
Mutex<[Option<PermitEntry>;2]>. Gate can only be minted from the initial view
helper's actual owner fields. No numeric count, bool available, wire token or
public constructor. source metadata budget and actual Document/source/context
binding are verified. View revision is checked by inputs; the gate identifies
the immutable document/context/budget scope across legitimate view revisions.
No new global counter is added to ConsumerBudget or D1.

PermitEntry = { weak: Weak<PermitAllocation>, allocation_charge: Charge }.
PermitAllocation = { gate: Arc<GateInner> }. Its Arc is a PRIVATE linear
ProviderPermit, held only inside the real physical-provider wrapper. There is
no Clone/into-token/into-bare-provider API. Aliases borrow or clone the OUTER
SequenceSlot/HitAllocation/OwnedAllocation, never the permit independently.
A permit cannot outlive a discarded index in a descriptor/history bookmark or
escape before its associated provider. Helpers expose borrowed D1 methods.

Acquisition checks cancel/Work, document/budget/gate identity, then locks only
this fixed-size registry. It first PURGES entries whose weak strong_count is0:
remove/drop weak and its allocation receipt BEFORE a new reservation/allocation.
For an available slot, reserve size_of<PermitAllocation> +2*size_of<usize> before
Arc allocation; create permit and install its Weak/receipt before any D1 factory.
The slot already counts while D1 preparation is in progress. All quota/cancel
failures drop candidate only; old providers/pages remain. Two occupied entries
return closed ProviderBusy before any new provider/scratch/index creation.
No lock is held during D1 preparation or while dropping an index. There is no
custom Permit Drop that re-enters the mutex.

Weak is to the small permit allocation, NOT to SequenceSlot/HitAllocation/index.
Its body/header receipt lives in PermitEntry through last Strong and the retained
Weak tombstone until purge/ledger Drop. Thus a Weak does not keep an uncharged
large index body/header alive. The permit's Arc gate pointer drops at last Strong;
Weak does not retain live fields. Gate ->WeakPermit and Permit ->ArcGate has no
strong cycle. Gate destruction drops tombstones/receipts; fixed registry memory
is charged as part of GateInner. Last actual index/provider Drop remains outside
atomic pane publication. The registry does not release capacity when a logical
current handle disappears while other aliases still retain that physical value.

## Wrappers: no D1 struct/profile change

Central closed factory returns GatedText, GatedIndex or GatedRefIndex, each with
{value: actual D1 value, permit: ProviderPermit}; fields private to gate module.
Value drops BEFORE permit. GatedText::prepare obtains one permit then performs
exactly one SegmentedText::prepare_with_profile. into_index consumes it and calls
SegmentedIndex::prepare_with_profile on that SAME text, transferring the SAME
permit: no second slot, no index clone. GatedRefIndex::prepare obtains one permit
then calls the real TextRefIndex::prepare; its inner SegmentedText/SegmentedIndex
are the SAME physical provider and consume no second permit. This fixed factory
cannot invoke an arbitrary closure that manufactures several indices under one
permit. No modifications to document_text.rs are required for admission.

Layout/Search/Inspector owner enums retain gated wrapper or legacy D1 value;
borrowing visits/seek/coverage never removes a gated wrapper's permit. Their
metadata reservations use ACTUAL compiled layouts, subtracting ONLY the concrete
D1 body whose existing scratch/metadata charge is still retained. All new inline
variant/Option capacity, handles, lease metadata and Arc counters are funded even
for absent variants. Gate body charge is size_of<GateInner> + Arc counters +
internal Identity allocation counters; actual Registry/Mutex layouts measured,
not fixed guessed bytes. Input/Ready/retirement body charges use compiled sum
sizes. Alias metadata is charged separately; D1 bodies are not copied/recharged
merely by sharing. Near-quota affected parent-size profiles must be rerun even
though no D1 profile/cap changes. Exact charge report follows implementation.

## Factory inventory (canonical line references, not provisional surface docs)

* SearchRequest::provider, document_search.rs691/697: runs text construction
  enters gate factory; conversion at769 transfers its permit to the same index.
* Search TextRefIndex::prepare at705: gated Ref factory reserves ONE physical
  permit BEFORE the real internal preparation. retry_hit at650+ uses the same
  scoped path. Found/FoundPending retain wrapper/permit if provider retained.
* SequenceSlot::prepare, layout245/253: gated text ->same gated index; prepare_ref
  at263/273: gated Ref factory. Each invocation is individually admitted.
* Domain slot call sites1204 (window/domain),1641 (table),2396 (build_window),2547
  (selection/endpoint), Ref call1417 each pass the actual scope gate. None may
  retain old/current candidate and construct next provider without new admission.
  WindowBuilder/table helpers propagate optional scope borrowed from real input.
  If two distinct slots are retained, construction fails ProviderBusy/Pending;
  no third transient is built. Aliasing an existing matching slot costs0 slots.
* Inspector prepare_provider at481/489/496 uses gated runs/index or Ref factories.
  Slot/Hit aliases must show their exact SAME gate permit. Metadata/cursors/page
  fork/summary/link/sample operations do not create another index.
* HeadingTarget::visit_title at950 creates a temporary projection provider and
  counts in scoped use. Add visit_title_in_view(realView,...) using the same gate;
  legacy visit_title stays outside scope. Heading pages/targets themselves are
  metadata descriptors, not providers. Scoped adapter cannot invoke the legacy
  title preparation path secretly.
* document_inline.rs531 is the separate streaming INLINE compositor, not a
  retained Markdown view/search/inspection operation of this adapter. It remains
  outside this guarantee and retains accepted legacy profile. No inline cap.
* TextRefIndex's nested prepare_selection/SegmentedIndex at document_text1554/1562
  is covered by its outer gated Ref factory as ONE provider. Existing public D1
  unit-test/direct factories outside this scope remain uncapped, byte/points
  bounded as accepted. These are not alternate product paths to mint gated hits.

All scoped producer entry points receive gate from an accredited input or view.
ScopedSearchRequest::from_view validates actual view Doc/budget/gate; arbitrary
legacy SearchRequest::new cannot be relabelled scoped by passing numbers. An
ungated legacy hit presented to inspectable view returns unchanged via pending
conversion, not silently registered as a second-counted or unobserved index.
Inspector tests must create views/search through real scoped factories.

## Search best+new: prospective core delta, not an operation permit

Current accepted SearchRequest::find keeps Match.provider best while preparing
next provider. With pinned inspectorB this may require B+bestRef+nextRef. Scoped
mode MUST NOT claim one operation reservation covers both. The central factory
would reject next Ref before construction; the old inspector remains usable.
To avoid a permanent backward-scan blockage, scoped mode stores only a sealed
BestDescriptor between sequences, not the best physical provider. KMP/sequence
order/overlap/forward/back/onewrap/query4096 semantics stay unchanged.

For each fully visited sequence, compare its chosen match to current best. If
better, obtain raw-start anchor with the actual provider and existing coverage
algorithm under preparation Work/cancel, then record sequence (including real
ResourceID/field), matched range, eligibility and anchor/snapshot identity in
private BestDescriptor. Drop this sequence provider BEFORE constructing the next
one. Between sequences there is no best provider. After the walk, construct only
the winning provider through another gated factory. This costs another admitted
preparation and its REAL Work/counters; it is not CPU total find accounting.
The descriptor cannot be caller fabricated or published as positioned hit.

If the final winning provider is busy/quota/position-work limited, an accredited
DocumentFoundPending can preserve its semantic match/raw anchor without provider;
explicit retry obtains a fresh permit and retains owner/cursor/revision. Stale
snapshot/query is rejected. Cancel returns Cancelled, not an accepted partial
hit. Forward early stop still moves its current provider directly if eligible;
no needless reprepare required there. Original accepted core R1/R2 freezes remain
untouched. This is an explicit additional scoped search implementation delta
requiring independent review and real core regression fixtures before adoption.
Legacy find retains its accepted behavior outside scoped mode.

## Concrete cache handoff without replacement deadlock

DocumentView::retire_cached_provider(work) is a closed O(1) CAPTURE action. It
first reserves size_of<CacheRetirement>, validates actual scope, and only then
moves TextWindow.slot.take() into CacheRetirement {actual gate,slot,charge}.
No index construction/last-owner Drop occurs in capture/publication. It changes
only optional cache to None; glyph/symbol/paint/anchors/TableResume/Flow/navigation/
source metadata and frame receipt stay with the real view. It returns a funded
owner, not a bool that falsely reports a freed slot.

The caller passes this sealed retirement to RetireAndInspect input (constructor
checks exact gate/document/budget/context; mismatch returns original retirement).
The SAME existing authority's background operation drops retirement/last index
before acquiring the candidate permit. If another old_slot/input/page still
aliases A, actual permit remains live and C returns ProviderBusy. No restoration
of freed cacheA is promised after failed C; the current frame/oldB inspection and
continuation survive. Later cache reconstruction is explicit preparation and
must pass the same gate. Two independently pinned inspectors B/C correctly block
thirdD until actual last alias Drop; no autoloop, history reset or fake release.

No handoff of an index still needed for old page. If layoutA aliases inspectorB,
occupancy is1 and newC can be admitted directly without eviction. Old+new current,
retired cache, input.old_slot and all page aliases count automatically through
actual retained wrapper permit. History descriptors contain no provider/permit.

## Prospective evidence fixed before affected runtime

Keep P01–P07 R1. Additional P08 real parsed fixture of three nonempty image Titles
("same" in two distinct resources plus pinned providerB), actual scoped view,
cache handed to real authority and physically dropped: backward search processes
bestRef1 then newRef2, every gated factory observed; peak<=2 INCLUDING temporary
providers. Compare exact source/model-derived owner/range/order with legacy core,
not candidate goldens. Dropping best between sequences must keep pinnedB usable.
P09 explicitly call per-provider gate with B+bestRef alive: third factory Busy
BEFORE D1 prepare; no silent cap-only operation permit. Then drop best and admit
next. Token Weak/receipt remains charged after last strong until purge; purge
frees exact actual token allocation before next. One Ref nested index counts1.
P10 preserve old page/summary/continuation across quota/cancel/final-reprepare busy,
with queryRevision!=viewBinding.revision; actual full Source binding unchanged.
P11 same physical slot aliases view+input.old_slot+page: capture retirement alone
cannot free a permit; after real last alias Drop frees exactly1. Real render and
all actual cell/presentation anchors remain equal after cacheNone; navigation
reprepare with two pinned inspectors returns Pending, no UI build fallback.
P12 real authority sequential Viewport/Search/Inspect/RetireAndInspect jobs,
joined receipt required and stale/cancel rejected; no second authority instance.
P13 original D1/core/b2/INLINE regressions plus near-cap metadata parent sizes;
no new general Consumer slots or relaxed bytes/points/cancellation.

Disposition until independent review: OPEN IP-P01/IP-P02. No affected code is
implemented from this document. Proposed wrappers/gate/adapter must be delivered
coherently with inspector and exact freeze/binary/manifest; this prose alone does
not satisfy final implementation acceptance.