# Historial b3: almacenamiento y commit R2 revisados

2026-10-09. Refinamiento final que sustituye la representación incompleta R1;
complementa [el contrato atrás/adelante](MARKDOWN-HISTORY-FORWARD.md).
Los apartados comunes de scope/proof revisados se incluyen abajo y los siete
oráculos canónicos no cambian.
No history code yet. Search R2 freeze is separate and immutable.

H01 is accepted. History must preserve the COMPLETE accredited PresentationAnchor,
rather than Role/Resource IDs alone. The canonical b2R2 design is the authority:
Domain(LayoutAnchor), Role(block/phase with stable owner), Ref(RefAnchor including
RefOwner Resource or BlockRole+field, TextRefID, local byte, projection_part),
Resource(ownerRun/resource/lane/position), Table(TableResume). FlowOrdinal is only
a cache of the live candidate Flow, never a persisted history identity.

Long KIND/INFO/LABEL/input/title retain owner+field+TextRefID/local byte/part.
Fields remain distinct when resources share TextRefID; ResourceID from the
corrected core search is preserved. Tables retain RowID, selected CellID, selected
Domain/LayoutAnchor and EVERY participating local cell's At(anchor) or Done.
Vertical mode has the active cell only. Done is a typed value, never an offset or
an implied restart. Resize/remap reconstructs wrap from those logical positions,
without reinterpreting a geometric subrow or restarting metadata at0.

R1's "closed Copy entry" is therefore insufficient for TableResume, whose actual
local progress is a ChargedVec. Corrected representation:
- fixed129 slots Option<HistoryEntry> still include current; m/k bounds unchanged.
- HistoryEntry has fixed pane/scope/hit/origin/geometry fields and an owned,
  accredited PresentationBookmark.
- PresentationBookmark is a closed exact projection of PresentationAnchor,
  stripping only stale Flow ordinal/geometry caches. Fixed anchor variants keep
  every raw/local/projection_part and owner/field identity. Table variant owns or
  charged-aliases an immutable TableBookmark containing exact local progress.
- TableBookmark may hold its own charged Vec of real logical CellProgress values
  or a shared immutable anchor allocation already charged by b2, depending on
  b2's actual frozen representation. It cannot retain the whole LayoutWindow,
  Point, Flow stack, provider or model by accident.
- This is an explicit implementation dependency, not a claim that b2 anchor API
  or its size is frozen already. Select one representation and measure exact
  size/capacity only from the b2 freeze, then save the concrete type/ledger receipt
  before implementing. No fabricated numeric metadata harness substitutes for
  real getter/provider/Flow/anchor restoration tests.

A slot is fixed metadata; optional table progress is physical retained metadata
with its own actual ConsumerBudget charge. Every new copy/alias obtains its own
appropriate value/capacity charge while old+new remain live. No128 free large table
rows are promised: metadata budget may reject candidate retention before commit
without evicting the oldest or changing m/k. There remains one semantic quota
back+forward<=128 and one physical1MiB consumer ledger.

Exact admission plan after b2 freeze:
size_of::<Timeline>() + actual identity Arc metadata for the129 fixed slots;
size_of::<HistoryPlan/JoinedHistoryReady/preflight>() before descriptor/result;
actual table-anchor payload capacity/arc/value receipts before every copy/alias;
Source Box/model/indices remain solely in the actual SourceAllocationReceipt;
no reused NavigationStorage128 plus this allocation during combined adoption.
No speculative numeric size is asserted before inspecting the actual type.

Commit refinement for owned metadata:
prepare allocates/charges a bounded retired-entry container BEFORE ready/preflight.
During infallible commit, displaced current/target and future entries are MOVED
into it; no anchor/model traversal, deallocation or last-owner Drop occurs after
truncation. Maximum129 displaced fixed entry values (future128+current1).
The commit receipt retains their charges/leases until dropped outside publication.
One candidate/result only; preflight fails quota/stale/exit before any state change.
None preserves future, updates current through a real captured PresentationAnchor
and increments checked revision, invalidating old candidate plans.
All indices, m/k next values and next revision are checked before the guard.

Owner proof remains closed: only the existing authority.take_validated wrapper
after physical join can construct JoinedHistoryReady, and only a real complete
pane/selection/remap proof can satisfy it. Raw receipt(joined:true), caller flags,
invented State, unpositioned hit/heading or SourceMatched cannot commit. Two guards
(pane and history) are acquired before replacement; both apply infallibly on the
same UI authority. Source uses the unique retained Box<PreparedSourceModel>.
Proof/getter adapters will be implemented with b2's actual frozen APIs, not a
provisional owner or second kernel.

Added pre-output gates to canonical7:
- long ResourceTitle/Kind/Info/Label middle anchor: Back/Forward after resize keeps
  exact owner/field/TextRef/raw byte/projection part and reaches suffix+TAIL.
- two resources share one title TextRef: restored owner differs correctly.
- table row in column mode with mixed At anchors and Done: resize+Back/Forward
  preserves selected cell and every local progress; no completed cell resumes0,
  no later TAIL disappears. Switch vertical/columns keeps selected semantic text.
- failed/quota/cancel/stale remap leaves both branch and exact full current anchor
  unchanged, including tables/metadata controls.
- old+new table bookmark payload and displaced-entry receipt remain charged until
  Drop; no uncharged index/Source model clones or whole-window retention.

Await actual b2 anchor freeze and root readback before history code.

## Scope y prueba cerrada conservados del contrato revisado

HistoryStorage owns the same ConsumerBudget and one Arc<Document> snapshot scope.
Scope stores the actual PreparedSourceContext binding (input,generation,request,
digest) and the document/source pointer identities once, not129 Arc handles/models.

## Preparation and closed commit proof

Effect = None | AppendBranch | MoveBack | MoveForward.
HistoryPlan is private, captures exact timeline identity+revision+m+k, snapshot
binding, departed-current bookmark, target bookmark and effect. prepare checks
known capacity before any target work. Append at k128 fails, including m128;
Append at k<128 is permitted without truncating anything yet.

A raw PreparedCompletion or public OwnerReceipt(joined:true) is NOT a commit proof.
Only a wrapper that itself calls the EXISTING authority.take_validated(expected,
exit_intent) can manufacture JoinedHistoryReady. Its fields/constructor are private.
It accepts only actual completed Ready output after physical join; Pending,
Limit, Failed, cancellation, exit and stale return no ready token. It does not
create an authority, spawn another owner or accept a validate=true boolean.

The wrapper requires a closed pane-position proof:
- Document proof derives from actual PreparedViewport's candidate selection/
  window/key/Flow. It validates planned anchor against State::prepared_visual or
  prepared_role in that candidate Flow, exact DocumentKey, old Flow pointer and
  snapshot before producing the ready token.
- Source proof retains the actual Box<PreparedSourceModel> output (or the already
  retained owner) and checked original+visible selection with same Arc/hash/key.
  It cannot be built from a range or fabricated Ready boolean.
- Inspector proof consumes the real prepared page/anchor provider.
- PositionPending, SourceMatched and unremapped HeadingTarget cannot produce it.

Dependency is explicit: current PreparedViewport has no borrowed selection/key/
candidate-Flow getters, and FlowIndex has no document getter. The combined b3
adaptor needs minimal closed accessors/prevalidation in document_view and a
borrowed FlowIndex snapshot getter after b2 freeze; this area will NOT invent an
unchecked constructor or alter b2 while its candidate is being reviewed.

Storage can be implemented/tested independently with closed proof factories
derived from real validated Flow/State and Source models; the final owner adapter
must use the same DocumentPreparationAuthority/result enum at combined adoption.
Any temporarily generic test harness still uses the actual PreparationAuthority
kernel/physical join, never a new owner implementation. It is library evidence,
not a claim that current DocumentView already has public history commands.

Copia canónica unificada de R2 SHA33E0CB4243D5E5194F4A4A18A86F5F4093FD6E2CBBD950B75652BF356B288E94 y apartados R1 conservados SHA040DB973BEE763F27D029F977209505680E1AE0B3D58E08DE7DE1D84036F2699; sólo se ajustan encabezado, referencias y composición documental. Originales privados intactos. [Revisión independiente](../reviews/Markdown-D2b3-history-plan-r2.md): SIN_CRITICAS_PENDIENTES de diseño. No código ni RV-306 aceptados.
