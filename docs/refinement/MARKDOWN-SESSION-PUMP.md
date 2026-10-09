# D2c — contrato FINAL unificado de sesión Markdown

2026-10-09. Contrato de diseño unificado R1+R2+R04, revisión independiente cerrada; todavía no aceptación runtime.
Base de código 3db799c6d95513c08469295d592834b8bde79fe2, b2 aceptado. Inspector/history runtime y proveedor2 siguen pendientes.
Procedencia: R1 DD3931BB197A4CFD082AEE50CDEE686004632F56731ADEE70CB555D87802B1B2;
R2 06744E51E5521E8F6194A63539C801541BE7ACDEF77947A1FF40E5A6D0C0BA01;
revisión R2 7430270C70A0807A8C9CBACFDCA2B21D894956AA0D66FB1BCEE40F04BFE2AE2D.
Originales/oráculos privados inmutables; esta composición no cambia comportamiento ni cuotas.

## 1. Resultado y cortes obligatorios

Una sesión real recibe original Arc+origen, muestra Loading antes de hash/parser/Flow/Source; permite teclado,
Documento completo con bloques y recursos pendientes, Fuente lazy, búsqueda/headings/inspector/historia única,
resize, q0 y CtrlC130, y restituye terminal antes del cleanup. Conserva contenido/current durante preparación.
Sin resolver/enlazar archivos/red ni gráficos; eso sigue en contratos posteriores, no declarar RV completo.
D2c instala API de sesión y la acredita llamándola desde probe propio ConPTY. No flags públicos ni routing nuevo.
D3c posterior OBLIGATORIO conecta main/nu/help a esa API y valida todos selectors/transports/routing/inline combinados.
D3a/b reales ya integrados se conservan; main hoy muestra Fuente para Markdown interactivo y ese diagnóstico cambia en D3c.
D4 Syntect/scheduler fences OBLIGATORIO después: no usar SyntaxClient Source como coloreado del Documento ni simularlo.
No granularizar en decenas de cortes: una composición D2c completa y freeze; pruebas internas por frontera.

## 2. Mapa de APIs comprobadas y delta explícito pendiente

| Superficie real | Uso y cambio necesario para D2c |
|---|---|
| viewer_session::run(Arc<SafeTextModel>,typed,title,config,syntax,structure,initial_structure,cancel) | Legacy Source/datos queda byteconducta; NUEVO run_markdown entrada original/origen sin modelo inicial. App actual/SearchWorker NO owner Markdown. |
| KeyboardReader spawn/new, queue256, TICK25ms, cancel observado fuera cola, Drop join | Reusar sole reader y control_terminal; extraer scope reutilizable. NUEVO explicit stop/finish que no join vivo antes guard.restore. |
| TerminalGuard::enter/restore, modos exactos | Mismo owner; restore explícito en éxito/error/unwind y antes waits de preparación; no reinterpretar Drop como prueba. |
| SourcePreparationInput/new/binding; start_hash/start_source; PreparedSourceContext::from_document | Factories reales shared owner; HashReady permite Controller; Source lazy usa MISMO Arc y origen. PreparedSourceModel tiene getters prestados; SourceShared/SourceScope común propuesto mueve el Box real y conserva contexto/cargos, sin clone. Build y Viewport usan misma autoridad conforme §5a. |
| PreparationAuthority poll/take_validated/stop/cleanup/retry_cleanup + global_pending/busy | MISMA autoridad física máximo1 HASH/SOURCE/D2, resultado1 y último descriptor1. Adaptador de DO para b3/panes según freeze real; no segundo kernel/owner/thread por búsqueda. |
| DocumentController::prepare/poll/cancel/pending_cleanup/retry_cleanup | Parser real current_exe/ProfileExtended/ActorArena/IdentityLatch, deadline10s ya incluido. Autoridad ChildJob separada, no acreditada por join del preparer. |
| b2 DocumentView::initial_input/from_prepared/request/commit; Operation Home/End/Resize/Seek/Continue/Fold/Line/Horizontal | Request captura pequeño snapshot; construcción en start_document fuera UI. Presentation es variante SIN payload: Input privado conserva ancla de ventana/current. Falta preflight/borrow candidate y commit guard conjunto, añadir tras b3 freeze. |
| b2 TextWindow glyphs/symbols/paint/cell_anchor/presentation_row/presentation_next; Role/Ref/Alt/Table typed | Procedencia actual; no recuperar posición desde pantalla. Gutter/chrome y panes nuevos no existentes. Tabla conserva TODOS At/Done; Role ordinal sólo caché actual, history bookmark remapea. |
| SearchRequest prepare/find_with_work/retry_hit; DocumentHit/SourceHit; HeadingCursor::start/page, HeadingPage::targets/next | Core real; ejecutar en authority shared, adaptar resultados b3 remap todosancestros. Inspector/share/coverage/history/SourceTransition aún NO aceptados ni API garantizada. |
| b2 renderer::prepare(view,config,Rect,PreviousFrame,cancel), closed MandatoryRenderPermit | Sólo cuerpo y Geometry==Rect. NUEVO prepare_session/candidate con Rect completo+body y pane source proof; full-frame receipt concreto, no dar estos nombres por existentes. |
| BudgetedTerminal new/prepare_area/readback/draw_document/previous_frame | Owner real 2 grids/preclears/ForcedWidth/Skip/poison. Extender sólo consumo closed session plan; ningún Terminal paralelo/Buffer/widget de chrome sin cargo. |
| viewport_admission FrameBudget, Source Work256KiB y SafeTextModel allocation_receipt | Reusar proyección/width/coverage exacta Fuente fuera UI. No invocar render Legacy que construye widgets dentro draw ni rebajar sus cuotas/modelo. |

Adapters b3 disponibles sólo al freeze aceptado. El mapa se revalida por hash antes del código; incompatibilidad concreta vuelve a revisión,
no reemplazo con fake Ready/anchor, ni paquete vacío de inspector/history presentado como sesión completa.

## 3. Propiedad de la sesión y loop

Nuevo markdown_session.rs (orquestador) y markdown_session_frame.rs (preparación full frame) conectados por wrapper viewer_session.
Session mantiene snapshot/source context, DocumentView, SourceShared completo lazy con leases cerrados, timeline único b3,
pane enum Loading/Document/Source/Headings/Inspector/Help, query editor y hits/continuations reales.
Controller y PreparationAuthority únicos; capturas/cargos de candidatos permanecen hasta Drop físico, no dupe model/provider.
Antes de start_hash: stdout terminal+control válido, entrar guard, instalar reader y Loading mínimo admitido.
Loop orden: observar cancel/error/exit, procesar eventos aceptados y revisiones, poll controller/preparer no bloqueante,
rechazar stale, tomar sólo take_validated(expected,exit_intent) con receipt físico y prevalidar resultado/frame/history,
aplicar publicación descrita en §5, dibujar sólo si frame/chrome cambió, recv_timeout25ms.
No parser/hash/Source.from_shared/Flow/provider preparation/search/heading/inspect/range mapping/width en UI/key/draw.
UI captura request cerrada y checks constantes; falta provider produce Pending, nunca constructor largo dentro renderer.
Supersede activa stop, retiene owner; un descriptor último cobrado reemplaza al anterior. Spawn sólo tras join real.
Loading conserva pane/frame actual, query/hit y causas; redraw estable no reintenta Limit ni renueva Work.
HashReady->Controller Ready->initial_input/start_document->full frame. Cada avance requiere token actual y no ExitIntent.
Failed inicial termina owner físicamente antes Source fallback; SourceReady completo0 con causa parser, ambosfallos1 sin vista falsa.
Document current + SourceFail no cambia pane/current/history. Reintento sólo explícito y causa recuperable bajo contrato D3-R02.

## 4. Teclado y presentación observables

Teclas se atienden en Loading y panes, con press/repeat; Release no ejecuta acción. CtrlC tiene prioridad130.
Flechas/jk/Page/Home/End navegan; code/frontmatter Left/Right ajustan Geometry.left lógico, nunca Geometry.x origen.
Texto normal Left/Right o foco de celda usa adaptador closed unit selection b3/provider actual en preparación, pendiente de freeze;
no reconstruir ancla buscando texto en buffer. Table focus mantiene selectedCell y vector At/Done real.
/ editor, Enter confirma, Esc cancela editor; q en editor es literal. n/N usa continuation/wrap core. Selector de dominios según §5c: d fuera editor, CtrlD dentro; MainBody/CalloutTitle/ResourceAlt/ResourceTitle/SourceOriginal; InputResource queda inspección, no body falso.
i headings page64; Enter confirma target real bajo folds. m inspector actual unidad de selección; Enter chooser
si multienlace, confirmación explícita descriptor inert; o sólo una SourceAction acreditada y seleccionada.
Space en callout actual cambia fold; sin callout avanza página. Tab/ShiftTab Documento/Fuente lazy conserva posiciones.
Backspace Back y CtrlY Forward timeline único; Esc cierra inspector/headings/help antes de salir; ? ayuda; L números.
Movimiento de páginas/opciones/editor/viewport y pending->ready no añade historia; JumpHit/Heading/Inspector/Source sólo al cambiar.
129 transiciones rechazadas conservando todo; ninguno de los 128 stacks Navigator previos convive con nuevo timeline129slots.

Layout fullscreen: header1 fila, footer1 y línea de estado/editor1 si height>=3; body height.saturating_sub(3), width real.
Tiny geometry conserva lógica; 0 dimensiones no native draw, 1/2 sólo chrome que cabe, sin minsize forzado.
Body width descuenta gutter opcional. Documento números de FILA VISIBLE1..bodyheight, explícitamente vN |;
no Source line ni ordinal global reflow. Fuente mantiene sus números originales ya acreditados por SafeTextModel.
Gutter/cabecera/ayuda/query/diagnóstico son decoraciones seguras NoLiteral, no search ni fabricated First.
Muestras títulos/resources originales se muestran por provider/safe bounded sample con continuation, no Strings gigantes.
Help/header/footer no usan Paragraph/Line/Span que resegmenten body: emisión de glyphs ya preparados/acreditados.
Roles Config actuales Heading/Secondary/Accent/Value; ColourNever quita modifiers/color conforme renderer/inline.

## 5. Único frame nativo y publicación transaccional

Un BudgetedTerminal conserva ambos grids fixed del Rect COMPLETO. BodyRect checked contained y chrome rects disjuntos.
No cambiar prepare_area al bodyRect ni dibujar header con otro Terminal; mismoRect es noop readback-preserving ya aprobado.
SessionFramePlan propuesto (privado) presta candidato body Document/Source/Inspector/Headings/Help/Loading y chrome admitido,
retiene scopes/charges y contiene Rect completo, BodyGeometry, expectedold pane/history revision y candidate binding.
Inspection/History/Source pane conservan el PreparedSourceContext REAL ya retenido por DocumentView junto al alias
cobrado de _source_metadata y demás leases hasta el último Drop, incluidos retired receipts. Mismo SourceArc/hash
NO permite fabricar context.from_document o binding nuevo al inspeccionar/restaurar. from_hit requiere view/scope
cerrado actual conforme corrección b3 en curso; esa firma aún provisional se fija al freeze, no se simula en D2c.
Header/footer editor también se preparan fuera UI si necesitan D1; etiquetas estáticas acotadas no recorren payload en key.
Para Document la reserva held se calcula desde FULL_RECT y se entrega a layout(bodyGeometry)+chrome SAME operation.
No held pequeño de body seguido de draw full frame con otro allowance. Spent = layout+chrome+metadata/preflight+
copias/readbacks/diff/reset/preclears/formatter full actual; held nunca se suma al spent ni se renueva tras fallback.
Formula native actual mantiene 2updates*256/cell+payload+fijo256, cuatro scans de Cell, oldpayload/currentheap/readback;
metadata/roles/continuations/safe preview/copies cobran antes. Reusar ForcedWidth + REALblank Skip + admitted preclears.
Window128K incluye TODO metadata/symbol retenido de Document window; chrome físico/render copies también en Frame256K completo.
Consumer1M/checkpoints4096 incluye allocations documentales reales de history/current+candidate panes/pages/providers; shell/Fuente/chrome usan Arena según §5a,
Frame256K actual+candidate/oldcopies+chrome y Framework2M capacities/livepeak retienen ámbitos existentes.
Source modelo/visible/indices y originales conservan SourceAllocationReceipt más output_box_bytes según §5a; no 1MiB falso ni nuevo Window128K para Fuente.
Source viewport conserva su sublímite Work::frame256K/FrameSymbols256K/Symbol16K, preparado fuera UI desde modelo real;
su coste efectivo se añade UNA VEZ al recibo nativo de la operación full frame8M, sin un allowance extra para panes.
Si ambos sublímites/no total no caben, Reference completa reservada o rechazo mantiene current; sin nuevo límite silencioso.
Antes de nueva salida se fijan size_of/capacities y predicción completa80x24/120x40 incl chrome, Source y old+new.
No trasladar PASS dense cuerpo7417231 a composición con chrome: importe nuevo debe caber8M o volver a revisión, no subirlo.

Resultado join validado aún no cambia current. NUEVO preflight borrowed PreparedViewport/pane+history listo (b3 pending),
closed renderer permit se liga candidate/window/snapshot y OLD current revision. Reservar retiredentries/old charges antesguard.
Materializar sólo candidate grid, último current/cancel/exit check, native apply+flush; error previo reset candidate y old preservado.
Error después de output parcial/swap: poison owner y exit1/restaurar; nunca prometer oldvisible atomic ni hacer draw retry.
Tras native éxito, aplicar INFAILLIBLE pane/history guards juntos, sin alloc/traversal/drop de último owner ni despacho teclado entre ellos.
Retired receipt conserva old entries/payload hasta Drop fuera publicación; no Source/history commit por mensaje del canal o por draw false green.
Stable chrome redraw conserva mismos anchors/hit/history; fresh explicit frame receipt no vuelve a preparar input/index ni fallos deterministas.

## 5a. un cargo físico válido aun antes de Document

La sesión crea UNA ActorArena::new real ANTES de Loading y pasa ESA MISMA arena clonada al DocumentController::prepare.
ActorArena ya incluye TRANSPORT_RESERVE3MiB y límite64MiB. No segundo ActorArena para frames ni ConsumerBudget vacío.
Nuevo tipo cerrado FrameRetentionCharge = Arena(document_budget::Lease) | Document(document_consumer_budget::Charge).
Arena se obtiene sólo por ActorArena::reserve real; Document sólo por view.budget().reserve real, belongs_to snapshot comprobado.
NativeFrameCharges conserva cargos de cada origen y metadata real, current+candidate/old+new, hasta destrucción física.
BudgetedTerminal.symbol_charge pasa a ese conjunto cerrado, no Option<Charge> documental usado para todos los panes.
Plan preadmite capacidad/copias antes de materializar; after readback de ambos grids mide symbol_heap actual y verifica todas las sumas.
No crear un documento falso para obtener Charge. Tampoco considerar FrameBudget contador local como lease de símbolos nativos.

Clasificación única de cada allocation:
- Loading inicial/header/footer/help/query/dialogue/session descriptor/control transport nuevo: Arena real, aunque no exista Document.
- Fuente fallback/viewport nativo, samples/cell payload/chrome, Source handles/wrapper y retired Source frame: Arena real.
- Window/slots/hits/inspection/pages/history/bookmarks documentales y sus copias de body a ambos grids: Charge Consumer1MiB existente.
- Chrome de frame mixto: Arena; cuerpo Document: Consumer. Un conjunto puede contener ambos, sin cobrar una misma allocation DOS VECES en Arena.
ConsumerBudget::new reserva YA1MiB dentro la misma ActorArena mediante Document::reserve_consumer. No añadir lease directo Arena
para allocation cobrada dentro Consumer: el lease1MiB la cubre. Lease Arena de shell/Fuente es adicional REAL dentro64MiB vigente.
Original/visible/checkpoints/lines/index/model Box Source permanecen en SourceAllocationReceipt real y límites128MiB/256MiB/1Mlíneas;
no cargar toda proyección Source al Actor64M ni Consumer1M. Sus wrappers/handles nuevos pequeños sí se cobran Arena por bytes reales.
Grids/capacidades/metadata terminal/writer control siguen Framework2MiB live/peak real; es admisión física, no un segundo cargo Actor ficticio.
Frame256KiB y Symbol16KiB admiten TODO símbolos/metadata/copies actuales+candidatos+chrome conforme R1; no otro256KiB por origen.
Work8MiB conserva SAME operation completo y Source viewport su sublímite256KiB; no reset al alternar kind del cargo.
Si reserva shell+parser+Consumer1MiB no cabe64M, rechazar intacto; no elevar arena ni liberar oldframe virtualmente para PASS.

Al pasar Loading->Document o Document->Source, mantener los cargos viejos de su origen mientras sus símbolos estén físicamente vivos.
Tras reset/swap los cargos de grid destruido pueden retirarse; chunks/retired logical entries que aún viven retienen cargos propios.
La clasificación NO cambia por flag pane: no convertir un cargo viejo Arena en Consumer gratuitamente ni liberar Charge vivo al cambiar vista.
Doc retained mientras Source current conserva su Consumer completo; hidden Document windows siguen en ese ledger, no se suman como
nueva copia native Frame si no son su previous/current/candidate. PreviousFrame enum cerrado acredita allocations que realmente están en grids.
Metadata de session shell se reserva Arena una vez antes spawn; el owner preserva lifetime de la misma arena incluso si parser falla.

APIs existentes: ActorArena::reserve->Lease/used/peak; Document::reserve_consumer; ConsumerBudget::reserve/belongs_to; owner readback.
Deltas mínimos propuestos: FrameRetentionCharge/NativeFrameCharges + plan ownership getter privado; ActorArena::same por Arc::ptr_eq
para validar arena handed al controller; si getter Document arena requerido, prestado crate-private, no constructor de presupuesto nuevo.
Números exactos sizeof/capacidad y costes se fijan con adapter compilado ANTES output; R1 dice Framework no es RAMglobal StdoutLock.

## 5b. mismo Box Source alcanzable de forma Send+Sync+'static

Nuevo source_shared.rs común, implementable SIN session/pump/inspector/history. Mantiene el Box único, sin clone SafeTextModel.
SharedSourceAllocation privado contiene {model: Box<PreparedSourceModel>, scope: SourceScopeLease, _metadata: Lease}.
SharedSource y SourceLease poseen Arc<SharedSourceAllocation> y cada handle su Lease Arena privado; no Clone gratuito.
SourceScopeLease cerrado conserva contexto REAL ya preparado y cargos de su propietario:
PreDocument conserva el PreparedSourceContext real de HashReady y sus leases/metadata Arena;
Document conserva context exacto tomado de DocumentView y alias del _source_metadata realmente cobrado, más lease de handle.
No reconstruir context.from_document ni nuevo binding porque ArcSource/hash coincide. Scope valida arena/snapshot/input/generation/request/digest.
El modelo Box se mueve dentro wrapper Arc conservando dirección del MISMO SafeTextModel y su SourceAllocationReceipt completo.
Original Arc, original_capacity, visible/index/lines/checkpoints y sus identity allocations no se duplican al compartir.

Only closed adopt_joined_source wrapper llama authority.take_validated(expected,exit_intent) real, obtiene Outcome::Ready(Model(Box))
tras join físico, coteja scope/context/pointer/binding, y preadmite wrapper size_of::<SharedSourceAllocation>()+2size_of::<usize>()
(Arc counters) más size_of::<SharedSource>() y metadata source scope handle. El ledger Source completo usa checked known_heap_bytes + output_box_bytes(); known_heap_bytes excluye model_bytes, y output_box_bytes incluye ese modelo embebido y todo PreparedSourceModel. No sumar model_bytes otra vez. Si una partida previa ya incluye model_bytes, añadir sólo output_box_bytes()-model_bytes checked. Aliases del mismo original conservan una allocation, wrapper/Arc/handles nuevos quedan Arena.
TODAS reservas/checks de cancel/identity ocurren ANTES de mover Box. En fallo devuelve SharePending{cause, SAME Box, scope/charges},
sin SourceReady publicado ni cambio pane/history/current; un resultado retained1 conserva modelo completo para retry explícito sin from_shared.
Arc::new se usa sólo después preadmisión como demás owners actuales; no prometer recuperar abort global por OOM de Arc.
No leases/aliases salen antes finalizar adopción, por lo que error local posterior prepublication puede recuperar wrapper único y SAME Box.
try_lease cobra size_of::<SourceLease>() antes Arc::clone, comprueba scope. Borrow model/context/allocation sólo con lifetime de &lease.
Lease de worker/result/Pending mantiene la allocation hasta ÚLTIMO Drop aunque UI salga, cambie pane o retire entry. No préstamo &UI ni unsafe Send.

SourceViewportInput privado OWNED contiene SourceLease, source selection original/visible acreditada, geometry y expected query/action/frame keys,
metadata Lease y receipt; es Send+Sync+'static por composición de tipos reales. SourceViewportPrepared OWNED conserva lease+page+receipt+keys.
Worker usa model().viewport_start_with/cell_for_offset_with/source_range_with/visible_range_with reales y Work acotado,
sin from_shared, nuevo hash, String visible clonada, join UI o otro thread. Current sigue con SharedSource handle durante preparación.
Cancel/quota/stale devuelve outcome cerrado y drop sólo candidato; current/model pointer/allocation/binding iguales. Supersede conserva owner viejo
hasta join; pending tras2s retiene Input/lease/model/result/cargos completos y bloquea TODOS preparers como kernel actual.
Source SourceHit/editor búsqueda emplea range original real y mapping visible sobre este mismo lease; ningún RawAnchor documental falso.

Adapter común específico al kernel existente (propuesto, NO actual):
SourcePreparationTaskInput = Build(Arc<SourcePreparationInput>) | Viewport(SourceViewportInput).
ActiveOwner::Source usa OwnedPreparation<SourcePreparationTaskInput,SourcePrepared>; SourcePrepared agrega Viewport(Box<SourceViewportPrepared>).
start_hash/start_source siguen factories Build y fórmulas/comportamiento b0 reales; nuevo start_source_viewport factory usa el MISMO authority.
Binding de Viewport viene contexto original retenido + revision específica, nunca SourcePreparationInput::new que invente inputIdentity.
Los getters Input/binding se resuelven por variant cerrado; no convertir Viewport en SourceModel reconstruido.
Adapter se compila y prueba instanciando PreparationAuthority<(),()> y Source BOTH Build/Viewport reales antes pump; assert_send_sync
sobre SharedSourceAllocation, SourceLease, Input/Output y SafeTextModel, sin unsafe Send/Sync. Si trait/API no compila, reportar delta concreto.
Fuente original/visible modelos no participan en los2slots Document D1; son el Source index real con su receipt/ámbito ya existente, sin borrar costes.


## 5c. selector alcanzable por teclado y revisiones

Fuera editor d abre SearchDomainSelector; dentro editor CtrlD lo abre y retiene borrador exacto. CtrlD consume chord, no un scalar al query.
Orden fijo de5 opciones: MainBody, CalloutTitle, ResourceAlt, ResourceTitle, SourceOriginal. Foco inicial dominio confirmado actual.
Flechas Up/Down o k/j mueven foco sin tocar queryrevision/pane/hit/history/request; wrap sólo en selector de5.
Enter confirma; Esc cancela sin revisión ni pérdida borrador, retorna contexto original editor/pane. q con editor retenido se trata como query literal
sólo al volver editor; en selector q sale sesión normal0 (modal fuera escritura), CtrlC siempre130. Ayuda explicita estas rutas.
Confirm same domain no-op/revisión idéntica. Confirm distinto checked incrementa SearchQueryRevision; Binding de vista/anchor NO cambia.
Query escrita se conserva, continuation n/N del dominio anterior se invalida (no foreign cursor), cancela ownerSearch viejo por localstop.
No reusar viejo n/N cursor ni queryrev anterior. Confirm no inicia búsqueda automática ni añade historia; volver editor respeta borrador.
Siguiente Enter de query o n/N con query committed prepara nueva búsqueda con nuevo dominio/key desde anchor actual bajo política wrap aprobada.
Hit y pane actuales siguen publicables como lectura ANTERIOR hasta nuevo resultado; label muestra dominio/queryrevision actual y estado Pending.
Nuevo resultado cambia hit/current/history sólo por commit completo validado; NoMatch válido informa sin borrar lectura/historia arbitrariamente;
Limit/Failed/PositionPending/cancel/stale mantienen current/hit/page/history y no se convierten a NoMatch.
Un resultado oldQueryRevision después cambiar dominio no publica ni dispara Source fallback. Selector focus/cancel/paging no transiciones retornables.
SourceOriginal desde Documento produce SourceHit absoluto y lazy SourceShared/model completo, mientras MainBody nunca busca URL/metadata sintética.


## 5d. Dependencias aún pendientes: proveedores2 y b3

IP-P01/IP-P02 siguen ABIERTOS; este contrato NO acepta el registro/pool de proveedores descrito en R2.
Se exige biblioteca instanciable y gate antes de CADA índice, identidad de allocation real, Weak/metadata y lastDrop cobrados,
sin modificar D1 global ni añadir cuota. Debe compilar y aceptar su propio freeze antes del pump; no depende de run_markdown.
Inspector/history y adapters de selección/publicación sólo se conectan tras aceptación independiente de APIs reales.
SourceShared/SourceScope común es independiente de ese pool y se acredita antes del pump con Build/Viewport reales.
No crear tercer provider, copyIndex, dummy Ready o recortar no-query/tablas/metadatos para simular cierre.

## 6. Exit y cleanup físico

q0/Esc salida normal marca ExitIntent, localstop y controller.cancel; no globalcancel paraq, no lateReady/fallback nuevo.
Reader observa CtrlC fuera cola256. Para q/Esc overflow conservar UN evento candidato de salida con sequence propio;
pump lo interpreta DESPUÉS de los eventos aceptados anteriores bajo estado editor/help/pane real, antes poll/commit siguiente.
No latch q ciego por atomic editor aproximado: secuencia /q rápida debe escribir q, no salir. Repeticiones perdidas reportan loss.
Este slot transport1 no es resultado/preparation descriptor extra ni lista infinita; su metadata real entra receipt de session.
Reader failure1 detiene todos owners. CtrlC observado antes outcome final prevalece130; restored error real queda registrado.
Al salir: bloquear request/spawn, solicitar stop lector/preparers/controller, soltar préstamos backend y guard.restore explícito
ANTES de join vivo o wait reap. Después finalizar lector conforme poll25ms y recoger kernel/child bajo deadline único cleanup2s.
Thread no finished -> owner completo Pending y shared Hash/Source/D2 bloqueados incluso siguiente sesión; no detached handle.
Controller retiene su supervisor/ChildJob/Pending separado, retry_cleanup no spawn parser ni renueva deadline10s original.
Si reader no termina, conservar owner lector completo en pending control reader y bloquear otro reader/guard hasta reap;
no asegurar asesinato de thread/allocator. Esta extensión lifecycle es específica y necesita tests de handles reales, no SLA.
Restaurar inmediatamente no garantiza proceso termina inmediatamente con asignación no interrumpible; esa limitación se informa.

## 7. Aceptación ConPTY propia, anterior a outputs

Son normativos ambos originales inmutables EXPECTED-D2c-before-output-r1.json (16 familias) y
EXPECTED-D2c-before-output-r2.json SHA D4B291E83809DABB27FE05F1C5E24457CC47599F6714BD5C92A395A9FBA08229.
El suplemento PREVIO exige Loading sin Document/DocFail->Source/mixed charges y old+new/exact límites;
SAME Box/model/Arc/context/metadata con SourceViewport real retenido bajo cancel/resize/stale/Pending/lastDrop;
y VT d/CtrlD/flechas/Enter/Esc, same/different dominio, QueryRevision separada y no oldcursor.
C06 abstracta de R1 queda histórica: el oráculo operativo es d, Downx4, Enter, /secret, Enter del suplemento.
Controles ordinarios Source viewport sin hooks y delayed factory tras progreso real; no fake Ready/receipt.
sizeof/capacidades/meters y Send+Sync se fijan antes output/freeze, nunca una cifra ilustrativa como PASS.


EXPECTED-D2c-before-output-r1.json y8fixtures bytehash sellados antes runtime. No comparar goldens derivados de output.
Probe libtest/private bin llama MISMO run_markdown real, config local ColourNever, stdout/CONIN reales y worker prod exacto.
D2c probe puede dar worker_executable owned/hash en adapter TESTONLY porque libtest exe no implementa worker subcommand;
producción wrapper usa current_exe real. No parser/models/providers simulados, no main selector nuevo para saltar D3c.
Reusar sólo mecánica ConPTY nativa existente .tmp/nu-pty/conpty.py/modes.py como referencia leída; harness propio nuevo,
CreatePseudoConsole/CreateProcessW owned, VT teclado/resize/cancel, transcript bounded y decoder de GRID por frame,
no ANSI stripping que mezcle frames viejos. Oracle checks actualscreen semántico/progreso/selection y native modes before/after.
No launcher contra procesos ajenos, killtree global ni console profiles. Timeout harness sólo cleanup de su child/Job propio,
marca FAIL si fuerza terminate; exit/cancel éxito exige shutdown real o Pending ownership tipado explícito, no forcedkill verde.
Run normal bin freeze: C01-08/C12-16 con80x24/120x40 y narrow18x12, navegación/editor/roles/domains/Reference/history/source.
Controlled actual-worker fault/barriers: C09-11 demoran publicación o frontera instrumentada tras progreso REAL Hash/Flow/Source;
misma función/output/canal/join, no reemplazar por sentinel. q/cancel/restored witness debe llegar antes release de barrier.
Eventos testonly pueden acreditar receipt/tokens/commit counts fuera pantalla pero NO sustituir las teclas, guard y pump reales.
Ready normal + procesos hijos reales exige controls sin hooks aparte; no usar exclusivamente delayed fake workers como PASS parser.
C15 helper CONIN/CONOUT modes/cursor + retorno alternate + siguienteinvocación; restore observado antes cleanupstart.
C16 predicciones/exact-/exact+ y negatives de budget/heap/scans incluyen whole chrome; fixture32M único sólo si frontera nueva loexige.
Unit owners/b3/renderer baselines permanecen; únicamente ConPTY funcional acredita sesión, QA visual/matriz/Unix sigue pendiente.

Entregar freeze exacto código/bin/worker/fixtures/config/env/commands/receipts/sourcebase, adversario+AE ajenos y composición final.
No borrar D3routing ni D4fences de backlog, no cierre RV301/306/104 completo por este plan o por capturaVT.
