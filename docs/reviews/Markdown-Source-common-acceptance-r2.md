# Adopción del informe independiente Source Common R2

Informe original conservado en `.tmp/markdown-source-common-adversary-2026-10-09/ACCEPTANCE-own-source-common-freeze-r2.md`, SHA256 `9FEE8C7FE5636DF267EE22CE9CC72C0DD584018069AF355503AC4AEACB293E1D`. Copia canónica normalizada a LF. La composición posterior se registra en [adopción](Markdown-Source-common-adoption-r2.md).

# Aceptación independiente Source Common — freeze R2

2026-10-10. **SIN_CRITICAS_PENDIENTES** en el corte de biblioteca SourceShared/SourceScope PreDocument+Document y preparación Source Build/Viewport Common. **SC-W01 cerrado para R2** por inspección de código exacto y AE propia. Permite adoptar el delta Source R2 y comprobar composición con el fix B2 separado; este informe no acredita esa composición aún.

## Candidato y garantías de procedencia

FREEZE-SOURCE-COMMON-R2.json SHA **FC9070B90C7DF4787686B6975AD670976978A141A7AFB2D4793E5393661C74FF**.
Snapshot SHA9CA5A9EB8DC1FC970C7016AF8AF0240D0B7D88225242A144F74C7226EE88CF03; base3db799c; delta base58299601ABA889B5B9356A8595F1AF679199D6421BFA1526A250AE6286116718; R1→R2 2C706D8EB3498D82AB86D73FB56223C28E86545F6C78BDA998708E03F131D8E6. Handoff295282BF6922343079CAA63EEED216A7EAE9D61A8BEC68F4919D512A51903EB5.
Extracción propia extracted-r2;19 rutas del manifest verificadas SHA+bytes ANTES de tests. Ningún cambio a canónico, product-r2 ni freezes del autor. Source R2 NO contiene fix B2 posterior; aplicar sólo delta y preservar módulos canónicos nuevos, incluida prueba document_resource_owner_tests.

Criterios R1 y oráculos originales conservados sin edición, más suplementos previos 0E5D897DDCD7FD702D29F0DC0798143A0DC8E0EDDF128027750DF10AF0975A50 y1F2AC5354500A9AF73157274707C387E07E1E41A98D338EEC4A0D30803128C77. Su estado NOT_RUN histórico permanece; la ejecución cumplida se registra AQUÍ y en EVIDENCE-own-source-common-final-r2.json, no se reescribe la evidencia anterior.
La aprobación de diseño/calibración3CABCC89… no se utilizó como aceptación de código. FAIL256K/R1 y diagnóstico512K se conservan: aumento privado Common512K aprobado explícitamente, sin bajar cargos ni mover cuotas de memoria.

## Revisión de código y SC-W01

presentation checkpoint_metered_with cobra headers, tuple por probe, comparación efectiva, selección final; provenance_cursor_with cobra constructor real y mantiene cursor incremental. Endpoints source_range_metered_with/visible_range_metered_with y cell_for_offset_metered_with usan SAME Work.
Index incorpora métodos aditivos metered para start_offset/start_cell/position/line_entry/step. Probes leen sólo campos prestados con costes de comparación; Point clone/constructor y Exception clone tienen cargo antes de ejecutarse; cancel/Work fallibles. El fallback line_entry hace constructor acotado, no un lookup Legacy. Common usa estas APIs en apertura, selección y cada nextUnit; source_range_with posterior del cursor sólo recorre incrementalmente, no contiene apertura omitida.
Ningún partition_point Legacy en rutas Common. Work no se resetea por fila, selección, fallback ni clone. SOURCE_COMMON_WORK512K está cerrado en la preparación Common; FrameBudget::new y Work::frame Legacy permanecen256K. El recibo work() exporta el gasto de esa fase; admitir NativeSource/chrome en la operación completa sigue gate D2c.
Prueba independiente: retirar SÓLO bloque aditivo7778 bytes de text_engine R2 deja el texto completo EXACTO R1/base, SHA E0D3F08E7A386FD66EC39D941C9355E0997C47BE70CA1660EA2E69EB9EF59B3C. SHA archivo NUEVO DFFACE42B471663275AFED82DA6C09CC40221C418840DFB617AB9A3D2EABAC76; no se afirma archivo entero byteexact. Cargo.toml/lock originales exactos. APIs Legacy presentation conservan cuerpos previos y Common llama nuevos métodos.

## Harness propio, adaptaciones y ejecución

OWN-source-common-harness-before-runtime-r1.rs y oráculos R1 preservados. R2 inicial4B9E5BE138CD2C4FB3860D8CA7AD942E8F1E8F3E17DA618F97C14498E194D856 reutiliza cuatro grupos y añade cuatro casos focales; única adaptación original de límite es usar constante Common aprobada.
Sólo cambios de harness privados: módulo cfg(test) en source_shared, archivo own_source_common_ae.rs y hook cfg(test) en common_lookup_admit. El hook activa cancel DESPUÉS de cuatro admisiones reales: headers iniciales y primer campo/probe de Point. No cambia importes ni caminos productivos cuando está desarmado; no hace fake Ready/receipt/index/occupancy.

Cada compilación fue --offline --locked --lib --no-run; cada ejecutable se COPIÓ y HASHEÓ ANTES de su runtime. Todas las ejecuciones usan copias conservadas, nunca hash retrospectivo del target mutable.

|Intento|Bin SHA|Manifest previo|Resultado y razón|
|---|---|---|---|
|R2 intento1|795200736BB97C82CD0349C3CA7F73F1F9656FF929D17324FB43BAE598FEEA69|0D76B37F24F1FB538FE294688C12B46E55C47BA5F36E30A5A76A8687F652A6A2|7 PASS/1 FAIL del harness de medición; log preservado|
|Intento2|B377A5950611DE020C11F1619CCAB75AA6BA7A5D816DCA2AE3CCBCA963FC03FE|6AEFD9654E9CC2B3C47EC2544BC530AAABE32DF330FCB322028C50F68C2F7463|Sólo query/lateC1:1 PASS|
|Intento3|C872E919C2465DEDA522E69F7A5D988C024C75AB974C0021FAFA477A3531CF4B|EA0905E49FA9F99B67CD278AA167D4740FFDAD024A70EA7E7C13C09693B8DABE|Sólo cancel durante probe real:1 PASS|
|Intento4|27CF097F017934D6E0B6C3CA21FAFCC58139520A9B8DBDCA88569BED93AFB229|3C35AAA2002220DA6B54B113226B825CE167DB4988D2C3C659B43DEA6B39A34E|Sólo adopción exacta/foreign/key:1 PASS|

No afirmar una suite8/8 en un solo bin: son ocho tests distintos aceptados con sus ejecuciones exactas indicadas; controles refinados se repitieron solos. No nuevas suites globales verdes.
Fallo intento1: el harness medía query con Work::preparation ilimitado; Unicode next usa chunks4092 con límiteMAX frente4 para permisos finitos. Al repetir con permiso exacto, el coste cambiaba912→562 aun manteniendo el resultado. Corrección de fixture de medición: usar MISMO perfil finito Common512K como operación real para determinar required, después exact yexact−1. No ajuste de contenido/oráculo ni cargo candidato; falla del harness preservada.
Intento3 reemplaza trigger3 de cancel (último header previo al probe) por trigger4 (probe real), para no exagerar cobertura. Coste admitido267 antes Cancelled, old/current/charges iguales.
Intento4 completa SC05: tras reserva insuficiente wrapper+handle71B, SharePending retiene SAME Box; retry con72B EXACTOS reales completa y Arena.used==64MiB. Original24B lease exact/−1 también probado.

## Cobertura y resultados independientes

G1: Send+Sync+'static de tipos reales, Hash/Build/join, PreDocument real; parser/validator/view reales y DocScope del contexto exacto; SAME Box/model/original Arc/context/receipt tras cancelShare/retry. original.capacity4107 preservada. Wrapper no duplica proyección ni índice. Consumer source_metadata retenido tras Drop(view/doc), último Drop vuelve al baseline. Foreign arena rechazada.
G2: UNA PreparationAuthority<(),()> hace Hash/Build/Viewport; SourceLease en input/output/worker/Pending. Barrera después de preparación REAL, Busy/cleanupPending/globalPending bloquea otroHash; input/model/charges sobreviven current; reap/join reales y últimoDrop baseline. Stop/exit/cancel y staleBinding conservan modelo/viewport actual.
G3: controles C0/DEL/C1/CRLF safevisible literal independiente; source raw7..9 y visible13..21 completos. CJK clip y tabstop8 con espacios reales y rangos raw; combining/VS16/ZWJ completos. Fixture propia lateC1 después de cuatro líneas4096 conserva selección raw/visible y TAIL; selección parcial placeholder mapea scalar completo. Aperturas provenance, viewport, posición y endpoint: permiso0 rechaza, cancel previo retorna Cancelled; exact/−1 tipados con MISMO algoritmo finito. Costes query propios163/380/562/187.
G4: dense completo80×24 y120×40; coexistencia páginas old+candidate y todos cargos. Fulloperation current120 real, required300397:300396Limit sin output/publicación/cambio current;300397 Ready con glyphs/symbols iguales. SourceSymbol16KiB EXACT produce símbolo completo y sourceRange0..16384;16385 rechazo sin prefijo. SourceFrame frontera width10908 admite payload"x",10909 rechaza SourceFrame; geometry65535×65535 y overflow no falsa vista. Foreign scope samebytes retorna Pending originalScope/SAME Box, retry legítimo. Keys query distintas conservan Binding y se distinguen en Output; publicación completa de keys permanece responsabilidad del consumidor futuro.

|Fixture propia sin LF final|Work efectivo R2|Frame por página|Frame old+new|Arena.used coexistencia|Arena.peak|
|---|---:|---:|---:|---:|---:|
|80×24 ASCII|117336|48448|96896|3243128|3245032|
|120×40 ASCII|300397|123712|247424|3393656|3398632|

Son fixtures y mediciones propias, no cifras esperadas tomadas del autor129078/300398. Texto/glyphs/rows fueron fijados antes de outputs. Work incluye ahora probes/clones/header/constructor; límite512K mantiene margen medido. Frame antiguo+nuevo120 cabe256K en este caso; no es prueba de nativeFrame mixto con chrome.
Compiled sizes propios: SharedSourceAllocation56(+Arc16), SourceScope24, ScopeAllocation120(+Arc16), TaskInput200(+Arc16), Output320, SharedSource24, SharePending128; PreparedSourceModel304, SourceLease24, Authority48. Metadata usesizeof real y descuentos sólo de handles ya cargados. Source total known_heap_bytes+output_box_bytes conserva modelo una vez.

## Ledger y límites restantes

AuthorityStorage40+authorityvalue8 reservados antesBox en in_arena, mismo storage viaPending. Doc input funding privado y start_document contrastan ACTUAL authority/arena/mode/importe oldFlow; legacy default no adopta reducciónActor. Source job incorpora FinalReceipt136+capture real; Source metadata previa856 conserva lifetime input/result/Pending.
Exclusión histórica Doc FinalReceipt120 permanece EXPRESS: no nueva garantía RAM global ni scope asumido por Source. SharedDocumentDescriptor body-only/prepare recipe no acredita dispatcher sum fundedActor DI/DO/F ni proveedor2; esos gates siguen inspector.
start_hash/start_source Legacy scopeNone/metadataNone NO están financiados sólo por usar authority.in_arena. Hash común previoLoading aún pendiente. PreDocument SourceScope comparte contexto real joined y financia sus layouts al capturar; este corte financia start_scoped_source/Viewport.
Source modelo mantiene dominio de SourceAllocationReceipt, original128MiB/visible256MiB/1Mlíneas originales; no carga la proyección entera a Actor/Consumer. Handles/scope/wrappers reales Arena, sourceMetadata documental Consumer existente, sin doblecargo directoActor. Actor64MiB/Transport3MiB, Consumer1MiB, Frame256KiB, Symbol16KiB y DocumentWork8MiB intactos.

Ningún IO terminal/ConPTY/QA visual nativa acreditado. Pump/LoadingHash común, nativeFrame/receipt exportado una vez bajo8MiB, dispatcher inspector/proveedores, history/publicación y routing permanecen posteriores. No gaps accionables nuevos encontrados dentro de la biblioteca congelada; integración con canónico nuevo exige controles de composición root.
