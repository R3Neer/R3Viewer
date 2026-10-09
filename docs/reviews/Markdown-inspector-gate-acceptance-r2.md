# Aceptación independiente final del inspector/gate R2

Informe final importado con finales de línea LF. Original privado REVIEW-INSPECTOR-GATE-R2.md SHA256 `A8E97B60E0959E644CE02AD40E63B1BECFFEEC424D4713BE3BC8049522B15D0D`; se conserva el texto del dictamen y sus límites. La composición canónica se registra en [adopción R2](Markdown-inspector-gate-adoption-r2.md).

# CodeAE + AE independientes — inspector/gate R2

**SIN_CRITICAS_PENDIENTES**, limitado al freeze R2 identificado abajo y al componente biblioteca inspector/providerGate/adaptador real. No aceptación de dispatcher/pump de producto, publicación de Source, history, ConPTY ni QA visual/manual.

## Identidad y separación

Manifest SHA256 `98623E824CE794B60CC63C9097B772F60D9CAB63FFD7998770569741BDEE453C`; seal `EE4E71E00C18FAF34AE628E6663F4BA2DF9EB04CBF0D247D22835A93CE1975CA`; snapshot `27BC4F084B25BBD6801923F91C83FA3AE60FAA5CA2573DF2AFE6E9F4CAF91BB3`; overlay R1→R2 `43812031AA2B0B89DDB216E763172E8C112AA5E62E002A74EC666A49473DCCA7`. Base 5d8cd6793567a9a6cf6ae9e3cace5d490e1c2471. Verificación propia antes extracción en R2-INTEGRITY-BEFORE-EXTRACT.json y R2-SOURCE-INTEGRITY.json. La integridad más amplia del root es evidencia separada, no sustituto de esta AE.

CRITERIOS.md permanece F6FA552B634DD495DADD13B6720B3B1E4714C126316F31AE98686216B00211CD, FIXTURES.json 534C140C55B2A7661570D860AE7030D527C280DD07390E0A542945DB957D72AD, EXPECTED-NOT-RUN.json 54F29B407EA53B292CA3745DEBA271844EEB55E6BC7AC9D92665B7A4D3E466FD: las 30 familias y 19 fuentes originales no se cambiaron. Modelos privados ajustados solamente antes quarantine real, volcados antes runtime del inspector: R2-MODELS-BEFORE-RUNTIME.log, R2-ADDITIONAL-MODELS-BEFORE-RUNTIME.log y R2-DISJOINT-MODEL-BEFORE-RUNTIME.log. Variante final disjunta conserva el último acute literal con origen1..3 ya utilizado, en lugar de contiguidad127..129; valor proyectado y 65 componentes se conservan y quarantine real lo valida. Se mantiene el oracle original Disjoint tras agotamiento.

Sólo product-r2/target-own-r2 privados. Binario final copiado y hash antes runtime: own-ae-r2-verdict.exe `A8E798B85A8DE475AD2398EB382960119F56C345888B40AAD6153461A15EFD3A` (R2-VERDICT-BIN-BEFORE-RUNTIME.json). Harness agregado en inspector_own_adversary.rs y inspector_preparation_own.rs; gate añade únicamente observadores cfg(test) de progreso real después de adquirir permiso y antes del cuarto acquire de búsqueda backward de tres secuencias. Ningún Ready, State aceptado, permit ni receipt se fabrica. Configuración de observador no decide admisión/identidad; actual Gate/Consumer/Source/key/join siguen siendo los del freeze.

## Hallazgo R1 y corrección R2

CGR1-W01 sigue confirmado y preservado en REVIEW-INSPECTOR-GATE-R1.md y logs FAIL de R1: seed hit/view/cursor y proof omitían cuerpo restante de Work, aun cobrando Consumer. R2 document_inspection_preparation.rs45–141 admite antes de reservar/copiar: hit Self−HitLease, view/cursor Self, proof Self−alias. Cursor elimina la admisión State separada para no doblecobrar.

R2-VERDICT-PREPARATION.log comprueba Work256 exact/255 rechazo para hit/view/cursor; proof64 exact/63 rechazo para las tres clases; rollback de Consumer, highlight0..2 y SAME hit intactos, cancel previo. Inventario compilado Scope144, Seed112, HitLease32, Proof64. Search Box: Input192/Output16/job616/request184/queryExtra40/alias32/parent592, worker1464; Consumer1463 devuelve Limit sin perder input,1464 Ready con coexistencia al cap y Drop restaura ledger. Captura Consumer288 exact/287 rechazo con ChargedVec real. No se cambiaron cuotas ni se resta D1/alias que no esté físicamente retenido.

## Resultado y cobertura original

R2-VERDICT-OWN.log: **21 PASS**, incluyendo 3 controles de parse/quarantine/volcado de modelos y 18 escenarios conductuales combinados. R2-VERDICT-PREPARATION.log: **4 PASS**. Se mapearon todas las familias siguientes; la columna CodeAE aporta revisión de paths completos, no promete un test distinto por rama ni cobertura exhaustiva de combinaciones.

| Familia | AE propia concreta | CodeAE complementaria |
|---|---|---|
| OI01 | own_oi01_09_10_12_resource_and_metadata; own_oi01_10_table_role_and_resource_alt; own_gp01_legacy_fresh_scope_and_metadata_c1 | factories from_view/from_cursor/from_actual_anchor; selection enum y roles acreditados |
| OI02 | own_oi02_03_04_07_14; own_remaining_utf_control_table_and_quota: ab unit0..1/hit0..2, é0..2, SourceAction0..1 inválida | Scope.checked_range usa get UTF8 y SourceAction valida antes descriptor |
| OI03 | own_remaining_utf_control_table_and_quota: State real segundo ESC1..2, C1raw3..5 part1; own_gp01…metadata_c1 | CoverageCursor y classify sólo scalar acreditado, samples proyección segura |
| OI04 | own_oi02_03_04_07_14: cluster dos labels disjuntos LinkIDs distintos; own_oi05_13_chooser_65_stale_and_safe | chooser confirmado explícito, SourceAction inerte |
| OI05 | own_oi05_06_paging_quota_and_stale; own_oi05_13…; own_remaining…:64+1, cancel/Work0/ChargedVec cuota preservan página y continuación | page/choices piden siguiente cobertura real; old+new parents ChargedVec |
| OI06 | own_oi05_06…exact/mixed65; own_remaining…último65 literal disjunto, prefix Pending y End Disjoint | Aggregate sólo completa tras agotamiento; no bounding Exact |
| OI07 | own_oi02_03_04_07_14 entity; own_oi07_08_11_model_variants decodedEqual | kind/origin de TextRef acredita relación; ninguna igualdad textual |
| OI08 | own_oi07_08_11_model_variants shared titleRef con owner1/inputtwo, separación acciones | RefOwner conserva resource/field explícitos en hit/anchor/sample, búsqueda igual distingue sequence.resource_id |
| OI09 | own_oi01_09_10_12…anchors metadata reales sin query; own_gp01…actual Title local1/part1 y sample seguro | roles Kind/Info/Label y RefAnchor.checked owner/text/local/part; muestras continuadas no reconstruyen owner |
| OI10 | own_oi01_10…TableResume real y ResourceAlt; own_remaining…TableResume validado segunda celda y primera Done conserva todos los progresos | from_anchor_scope selecciona selected, acepta At y Done sin inferir fila pintada |
| OI11 | own_oi07_08_11_model_variants None/SomeEmpty y Role NoLiteral | optional RefID conservado; descriptor separa Input/Title/Raw/Alt/intención/origin y no inventa LocalFile |
| OI12 | own_oi01_09_10_12…sample-long chunks<=4096/continuación; Work/Consumer exact-/1 preparación | inventario de pages/summaries/samples/forks/Arc/Box y scratch; no provider nuevo por sample/link |
| OI13 | own_oi05_06…foreign cursor; own_oi05_13…crosspage/inspection/document y QueryRevision distinta Binding | scope identity/context/binding exactos y confirm SourceAction.closed |
| OI14 | own_oi02_03_04_07_14 Drop hit inspector usable; own_gp06…last alias; cuota captura conserva hit | SharedHit único Arc HitAllocation, alias receipts nuevos, value antes permit |
| GP01 | own_gp01…legacy view/hit Pending intacto, fresh gates y foreign Consumer inválido | gate mint sólo helper inspectable con SAME Doc/context/metadata/budget; Identity allocation, legacy D1 intacto |
| GP02 | own_heading_admission_and_postclaim_error; own_gp03_07_08_physical_before_d1; own_la02_03…barrier antes D1 con live2 | inventario factories: layout SequenceSlot Runs/Ref; search provider/retry/winner; inspector selection; Heading visit; GatedText.into_index mueve permiso |
| GP03 | own_gp03_07_08…thirdBusy inválido antes D1 con ledger intacto; own_la02_03…old page/frame | acquire antes constructor, rechazo no toca providers |
| GP04 | own_gp04_05_09_10_11_and_join: input.old_slot aliasA mantiene2 tras retiro/worker hasta Drop | retirement captura funded owner O(1), suma Box/input; no cache restore ficticio |
| GP05 | own_gp04_05…; own_gp06…sin cacheA; old glyph/symbol/frame/binding y B/page útiles | worker retira A antes nueva factory; slot aliasB permite1 plaza física; no autoloop |
| GP06 | own_gp06_last_actual_alias_releases_one: B/C pinned thirdBusy, summary último aliasB libera exactamente1 y admiteD | Gated.value antes permit y sin permit Clone externo; history descriptor no provider |
| GP07 | own_gp07_08_ultimate_gate_drop_ledger: D1sale, receipt24 permanece, purge previo y ultimate gateDrop baseline64/checkpoints0 | Weak apunta PermitAllocation8/header16, no Index; registry charge y tombstone, sin ciclo fuerte ni Drop relock |
| GP08 | own_heading…postclaim ID inválido libera provider; own_gp03…cancel; own_gp10…quota; own_gp07…ultimate Drop | acquire vivo durante factory; errores descartan sólo candidato; Byte/point cuotas previas intactas |
| GP09 | own_gp04_05…backward resource1/range0..4 luego0, B pinned+cacheA retirado peak2 | BestDescriptor sin provider; drop por secuencia antes nextfactory, owner en descriptor explícito |
| GP10 | own_gp10_final_reprepare_busy_quota_explicit_retry: fourth actual acquire pierde plaza o quota -> FoundPending owner1/range0..4/highlightNone, foreign Invalid, explicit retry same request Ready | forward transfiere mismo provider directo; backward reconstruye factory y conserva anchor acreditado |
| GP11 | own_gp11_overlap_querylimits_sourceoriginal: aa starts0,1,wrap0;4096/4097; SourceOriginal absolute sin slot ficticio | KMP/eligibility/directions/barriers unchanged; secondary core14 cubre backward y dominios, sin usar outputs como goldens propios |
| GP12 | own_capture_exact_work_and_consumer; own_preparation exact-/1 Seed/Proof/Search Box; own_remaining cuota real page/summary | layouts compilados y cargo completo enums/Box/Arc, Consumer1MiB/points4096 sin relajar; Work D1 distinto de matcher y no CPU total |
| LA01 | own_gp04_05…actual Authority/view worker; own_la02_03…Busy same input | una Authority arena existente, inputs Viewport/Search/Inspect/RetireAndInspect sumados, Source común intacto |
| LA02 | own_la02_03…poll false/take None prejoin, wrong Binding/exit; own_preparation metadata pre-capture | actual OwnedPreparation job/receipt/join y captura gate/context/owners, foreign arena/budget rechazado |
| LA03 | own_la02_03_real_factory_barrier_cancel_stale_exit_busy: stop/stale/exit y old page/binding/symbols preservados | output pendiente se desecha tras join, cache no reconstruye frame ni renderer; supersession sigue Busy/same input y stale key |
| LA04 | hashes previos y copia/target/bin propios;25 controles propios,39 secundarios R2;94 regresiones R1 | delta R2 sólo preparación+wiring test; fundamentos Source/kernel/model/Consumer/textengine/b2 intactos; no nuevas32M ni reapertura Source aceptado |

## Regresiones, fallos preservados y límites

Secundarios R2 sobre copia propia: inspector17, gate13, kernel metadata4, compiled-layout5 =39 PASS (R2-SECONDARY-*.log). No son oráculos independientes ni cuentan como25 propios. R1 había ejecutado94 secundarios: D1runs31/refs6, coresearch14, renderer14, INLINE16, resource-owner4, inventory5, kernel4 (REGRESSIONS-R1.json); los filtros vacíos se conservan y no cuentan. R2 no modifica esos fundamentos; se repiten las suites afectadas y no se afirma repetir toda la biblioteca ni32M. fmt/Clippy del autor están en freeze, no se presentan como revisión adversaria propia.

CGR1-W01 y todos los FAIL R1 quedan preservados. En R2 hubo tres fallos de harness, conservados en R2-OWN-REPREPARE.log, R2-OWN-LEGACY-C1.log y R2-ACCEPTANCE-OWN.log, explicados en R2-HARNESS-CORRECTION.md: una asignación adicional después de workerDrop podía admitir aunque se hubiese saturado cuota; un hit legacy emparejado con otro Consumer devuelve Invalid antes Pending; retener ConsumerBudget mantiene overhead64 tras gateDrop. Se corrigió preparación del caso/assert accesorio contra code/ledger, sin modificar las 30 predicciones. Repetición final25PASS arriba.

No se afirma agotar todos los interleavings ni todas las combinaciones de modelos: sharedTitleRef y backward se prueban separadamente, la unión owner explícita se revisa en CodeAE; backward overlap/barriers tienen revisión y regresión secundaria, no un escenario propio nuevo. Supersession se acredita por rechazo Busy y validación de key stale; no se prueba un pump de producto. TableResume adicional es constructor real validado con celda segunda y Done derivados del frame real, no un State aceptado fabricado. Ningún descriptor SourceAction acredita publicación/SourceReady. Estas limitaciones no dejan una crítica pendiente dentro de los contratos del freeze biblioteca.
