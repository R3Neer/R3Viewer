# Inspector y procedencia: contrato R2 revisado

2026-10-09. Private archive baseline12d28a48df7701bea5cf2d1e60b66c9674d2ef41,
which already includes independently approved core search R2. Canon history docs
later advanced tof849c36 without changing that code. R1 is preserved verbatim.
This proposal resolves root I-R01 (provider ownership) and I-R02 (whole-glyph
classification after complete coverage). No inspector code, b2/canonical edit,
matcher change or Source/history publication is included.

## Inputs and proof

InspectionSeed is closed and snapshot-bound. Constructors accept a ready actual
DocumentHit; current actual State/PresentationAnchor with its closed provider and
Flow snapshot; or a logical cursor accredited by State::prepared_visual or
prepared_role and D1. No query is needed for the selection/cursor constructors.
PositionPending or hit.highlight()==None stays InspectionPending.

The input identifies the entire D1 Unicode unit, including all styled/link Runs.
Metadata retains RefOwner(ResourceID or BlockRole), field, TextRefID, local/raw
byte and projection_part. Selection in a table uses the actual selected cell and
complete b2 anchor/TableResume; never a painted subrow or reconstructed ordinal.
Resource lane and decorative Role also accept actual b2 anchors. NoLiteral Role
can offer a separately labelled real Enclosing node, never synthetic Exact.

Scope retains actual Document/source Arc identity, generation/request/hash and
actual PreparedSourceContext binding. No guessed file origin or terminal-string
provenance. Constructors for actual b2 selection remain an explicit adapter
requirement; if they are not present in the eventual freeze, that acceptance
scope remains pending. Numeric fabricated State/anchors are not substitutes.

## Precisión I-A01 revisada: unidad del anchor y control único

Desde DocumentHit se inspecciona la UNIDAD D1 que contiene el anchor lógico/inicio del highlight, resuelta por el proveedor real. El highlight completo sigue perteneciendo al hit/query y se conserva para navegación y Back. Una coincidencia multiglyph no se trata como un único glyph y no se añade SelectionRange.

La etiqueta de control con scalar único exige exactamente un codepoint UTF8 raw acreditado. Coverage fusionada de dos controles no acredita esa etiqueta; mantener su relación/rango de componente sin promoverla como scalar.

IN13: source/query `ab`, hit0..2, inspector inicial unidad `a`0..1. IN14: raw ESC ESC, query `␛␛`/highlight0..6; la cobertura completa puede ser ControlProjection0..2, pero el primer anchor inspecciona scalar0..1 y el segundo1..2. El rango merged0..2 no es un scalar único. Oráculos sellados antes de implementación, sin cambiar los bytes privados de R2.

## I-R01: concrete provider ownership and ledger

Current DocumentHit owns Option<HitProvider> by value; HitProvider is private and
contains a SegmentedIndex or TextRefIndex. There is no Arc/alias accessor today.
The proposed additive adapter moves the COMPLETE hit, including its provider,
source/document/cursor and existing Charge, into this immutable allocation:

    HitAllocation { hit: DocumentHit, _arc_metadata: Charge }
    SharedHit { inner: Arc<HitAllocation>, _handle: Charge }
    HitLease { inner: Arc<HitAllocation>, _handle: Charge }

These are proposed closed types, not existing APIs. No Index or DocumentHit clone.
Original hit and D1 body/point/exception charges move with the value and remain
live until the last allocation owner drops. Arc::new adds no second provider.
Before conversion reserve checked:

    size_of::<HitAllocation>() - size_of::<DocumentHit>()
        + 2 * size_of::<usize>()
    size_of::<SharedHit>()

The first expression charges allocation padding/new Charge plus Arc counters,
following actual SharedFlow's formula. It does not reclaim or assume reuse of
SearchRequest's original conservative hit metadata reservation. Each HitLease
reserves size_of::<HitLease>() before Arc::clone, including the Charge field.
An inspection/cursor/page containing that lease charges the remaining inline
metadata exactly once: its full struct size minus the already charged lease
size. No subtraction of dynamic Index/Vec/Arc allocation costs is permitted.
Actual size_of values and actual capacities will be frozen with the compiled
candidate; no illustrative numeric sizes are acceptance evidence.

try_share takes ownership of DocumentHit and returns either SharedHit or
SharePending { cause, hit }, preserving the SAME original hit on quota/cancel or
failed preparation. All reservation/cancellation checks happen before moving it
into Arc. Publication replaces the caller's slot only after success; a failed
share reinstates the owned hit infallibly. Sharing is a representation-only
operation, not a selection/history commit or new PreparationAuthority. The precise
minimal adapter is Charge::budget(&self) -> &ConsumerBudget inside
document_consumer_budget, returning its actual private owner by borrow. try_share
takes no budget argument: it reserves through hit._charge.budget(). This avoids
an extra budget Arc/value or accepting another ConsumerBudget for the same
Document. Provider leases reserve through that same allocation charge owner.

ProviderRef<'lease> is returned ONLY by a borrowed SharedHit/HitLease method, with
lifetime tied to &self. It borrows the real private HitProvider and owner fields;
it cannot be retained after that lease drops or carry a naked static reference.
Any retained cursor/page owns its charged lease, not a borrowed provider. Frozen
core R1/R2 remain unchanged. A later reviewed additive adapter in document_search
must expose the sealed borrow and move conversion; immutable frozen artifacts
are not edited to pretend these APIs already exist.

For query inspection, live layout slot provider plus live hit provider are two
DISTINCT providers. Sharing adds handles/metadata only: inspection aliases the
hit provider, so provider-count remains two. The ledger includes both layout
slot's complete charges, the hit's complete charges and inspection metadata,
page buffers/samples/cursors plus candidate old+new allocations. It does not
pretend that sharing the hit releases the layout slot or the original hit index.
No third permanent provider is constructed. For no-query inspection the selected
current slot exposes a sealed charged lease of its actual immutable owner after
b2 freeze; inspection aliases that provider. If another provider is necessary
while two distinct providers remain occupied, preparation returns typed Pending
until a legitimate combined cache handoff can free capacity. It cannot silently
copy a provider or claim unimplemented selected-slot accessors.

## Incremental coverage, pages, samples and chooser

Existing callback visit_coverage cannot stop/resume with explicit accredited
progress. A minimal closed D1 CoverageCursor/RefCoverageCursor will retain the
actual provider lease, private projection/raw Position, range/end, pending merged
component and fixed state Charge. next_component(work) returns one actual merged
Coverage/RefCoverage, or sealed EndOfCoverage only after exhausting the range.
It reuses existing D1 projection/merging semantics, polling cancellation and
charging record/raw/projection work. No callback replay of previous pages, opaque
Stop sentinel or flattened/all-component vector. Coordinate additive TextRef
changes with b2; provider/page lifetime must remain valid after original HitDrop.

InspectionPage and LinkChoicePage use ChargedVec capacities<=64. Fixed closed
cursor seals inspection/source/provider/owner/field identities and current next
position/last LinkID. No free Clone. Candidate cursor/page and actual capacities
are reserved while the current page/cursor/provider remain live. Failure,
cancellation or stale identity returns no partially installed page and preserves
the current page and its current continuation. Explicit retry only.

Each Association retains the actual Coverage/RefCoverage relation, component
source/local ranges, styles, Run/Origin/Link IDs or real metadata owner+field.
Origin/Resource/Link records come from the actual validated Document. Display
raw and decoded target/title/alt separately through bounded safe D1 samples with
explicit continuation. No complete String flatten, eager link/record lists or
unprojected controls. Samples, scratch, record work and their current+candidate
capacities remain charged under ConsumerBudget/preparation Work.

Validated Link label spans are nonnested and contiguous, with every label Run
bearing that LinkID. Adjacent same-LinkID components may collapse via fixed
lastLinkID; distinct LinkIDs remain distinct even with identical target strings.
ConfirmChoice validates a sealed exact page/inspection option token and returns
SelectedLink(actual LinkID/Origin/field descriptors). No implicit first choice,
network/file/URL open, resolver/base path or history entry from chooser motion.

## I-R02: component actions versus whole-glyph summary

A page Association has ComponentSourceAction, explicitly labelled for that ONE
component. Its Exact/ControlProjection/EnclosingToken/NoLiteral classification
can be accredited from that component. It is not a whole-glyph action. A page
with next cursor never advertises WholeSourceAction::Exact from its prefix.

WholeSummary is a separate closed state:

    Pending { sealed summary cursor, fixed aggregate }
    Complete { closed WholeSourceAction, sealed exhausted-coverage proof }

The aggregate stores counts, first/last ranges, relation flags, source identity,
origin compatibility, contiguity and synthetic/no-literal flags, not all ranges.
Only next_component's real EndOfCoverage can manufacture the exhausted proof.
A caller boolean or a page length<64 does not prove exhaustion. The summary pass
may continue in bounded background work while accepted pages remain visible.
It shares the existing provider, has separately charged cursor/aggregate metadata
and never collects all components. Cursor progress and aggregate commit together
after work/cancel/source identity checks; failed work preserves the previous
accepted page/continuation and previous summary state. A partial summary cannot
publish an Exact option. Previously completed summaries remain scoped to their
own immutable inspection identity, not reused after selection/provider change.

Whole Exact requires every accredited component to be ExactSource and all ranges
to form a source-contiguous union of the same source. ControlProjection is kept
explicit: a single scalar's full raw range can produce a labelled control action;
multiple control or mixed relations are represented by component choices unless
an independently specified compatible relation proves a whole action. Decoded
TextRef always stays EnclosingToken even if its decoded bytes equal source bytes.
An all-decoded selection can offer one labelled Enclosing only when every
component has the same actual enclosing token/origin. Otherwise mixed/disjoint,
including Exact plus EnclosingToken, yields WholeSourceAction::Disjoint with a
sealed paginated component cursor. No bounding Exact absorbing markup or changed
relations. NoLiteral/synthetic components retain identity and no fabricated raw
range; real Enclosing is a distinct optional choice.

Disjoint's cursor streams the SAME accredited component set and individually
labelled actions. Its preparation does not allocate a full range list. Choosing
a component proves only that component's Source action. A whole no-literal
summary may still offer its real enclosing node distinctly. All actions seal
actual Arc/hash/generation/request/inspection identity, validate UTF8/range
limits on original source and remain descriptors. SourceReady/model publication,
shared history and authority join are later combined operations.

## Small model/source pre-output oracles before runtime

Preserve R1 I01-I12, including real selection without query, all b2 anchor
branches, multiRun/style/link union, resources and paginated labels. Core owner
fixture bytes are immutable, not mutable matcher goldens. Exact source/model
records and order/relation/range hashes will be saved BEFORE inspector runtime.

I-R01-A: actual ready hit with SegmentedIndex; reserve full live layout+hit costs,
convert by move and lease. A successful inspection remains usable after dropping
SharedHit's original handle. Exact source/provider identity stays the same, no
Index clone or third provider. Dropping inspection's last handle releases original
hit/D1 charge. Quota failure of Arc metadata or handle reservation returns the
same live hit, unchanged coverage. Same spelling/new owner and unrelated budget
must not create a usable lease. Repeat for TextRefIndex and ResourceID/field.

I-R01-B: actual no-query selected layout provider, charged alias then layout's
original handle drop; inspection cursor remains valid because its lease owns the
same immutable provider. Wrong slot/snapshot/selection is rejected. Current slot,
hit and inspection candidate costs are measured together at admission boundaries.
If b2 does not yet expose actual ownership, this fixture cannot be replaced with
an invented provider or fake State and no-query acceptance stays pending.

I-R02-A: one Unicode cluster, 65 small valid Coverage components. The first64 are
source-contiguous ExactSource; component65 is decoded-pool EnclosingToken even
though its decoded bytes equal the source scalar. Page1 lists64 per-component
Exact actions and next cursor; whole summary is Pending, never Exact. After
accrediting65/end, whole summary is Disjoint (mixed Exact+Enclosing), never Exact.
Cancel/quota between64 and65 preserves accepted page1, its continuation and
Pending summary; retry reaches the same mixed result. Include a last65 disjoint
range variant and an all65 contiguous Exact control for positive completion.

Construct the relation-change fixture from source 'a'+64 combining acute scalars,
65 validated Runs/TextRefs/Items in one actual domain; first64 TextRefs source-backed,
last decoded-pool text containing the same combining bytes with actual enclosing
origin of its original last scalar. Submit through real Candidate/quarantine,
record resulting IDs/ranges/kinds/origins BEFORE inspector execution. This is a
real validated model fixture, not a fabricated accepted State or numeric owner
proof. No broad32M suite unless a boundary requires it.

I-R02-B: source markup separates styled/linked parts of one cluster. Component
choices keep original relation/Origin/Link IDs; chooser union never promotes
bounding markup range. 65 distinct links retain pages64+1 and no first-choice.
Summary cancellation and paging cancellation are independent and preserve their
accepted states with charged old+new cursor/page metadata.

## Freeze and adoption boundary

Separate private src/tests/target and exact executable/manifest after approval;
preserve core R1/R2 and inspector proposal R1. Independent adversary reviews the
real implementation and own model-derived oracles before root adoption. Exact
sizes/leases/budget-adapters and actual b2 selection ownership must be inspected
and compiled, not accepted from prose. No new kernel/owner/session/thread. No
whole b3, SourceTransition, history implementation, manual QA or RV closure.

## Oráculos I01–I12 conservados

I01 source a*combining-acute*: one D1 glyph, styled Runs0/1, source components
0..1 and2..4; whole selection/noquery and hit inspection agree; Disjoint, never0..4Exact.
I02 [a](one)[combining](two): one multiRun cluster, two distinct LinkIDs/Origins;
two explicit chooser options, no implicit first; same targets still distinct IDs.
I03 A&amp;B: selected '&' has EnclosingToken1..6 and separate raw '&amp;'/decoded '&'.
I04 ESC and C1 scalar: control glyph/spelling inspect ControlProjection raw exact
scalar, never visible spelling offsets. Source literal label differs.
I05 validated decoded-pool TextRef with same bytes as original: remains Enclosing,
never promoted to Exact by equality.
I06 duplicate resources shared Title TextRef (R2 owner fixture): same RefID/text,
different ResourceID/field; inspection/choice/retry/stale validation preserve owner.
I07 65 linked Runs in one small combining cluster: association/link pages64+1,
same semantic order and distinct IDs; charged current+new and quota/cancel preserve
old page. No full all-links/all-coverage Vec.
I08 Rule/decor Role: NoLiteral; optional real Enclosing block separately labelled.
I09 resource descriptors: actual input/raw/title/alt absent vs empty, intent/dimensions/
origin preserved, labels projected safe, targets inert; literal/pipe has no file base.
I10 actual current cursor/selection with no query: Run/metadata/resource/Role/table
branches inspect the same accredited union/provenance as corresponding hit. Table
cell selected anchor is used; no subrow ordinal or provider reconstructed from paint.
I11 stale page/option and failed range/projection/source limits preserve current;
confirmation emits descriptor only, no new session owner/thread or history entry.
I12 long target/raw token/title samples use bounded safe chunks and explicit
continuation, not complete String flatten; controls do not reach display unsafely.

Copia canónica unificada del contrato privado R2 SHADF609393155BDBB477E71FDB4B363F8BACEB56B26C8DEBF7FF181405004EAD74 y oráculos R1 SHA2B8A07D534D31F8813CDDD80773A674552ADAF5F9C79561DD17609D57BECE9C5, con delta I-A01 aprobado e incluido explícitamente. Originales privados intactos. [Revisión final independiente](../reviews/Markdown-D2b3-inspector-plan-r2.md): SIN_CRITICAS_PENDIENTES de diseño R2+I-A01. No código ni aceptación del inspector implementado.
