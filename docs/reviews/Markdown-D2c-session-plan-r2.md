# D2c: cierre de diseño y composición final

2026-10-09. Adopción documental por el coordinador sobre base `3db799c6d95513c08469295d592834b8bde79fe2`.

[Contrato final](../refinement/MARKDOWN-SESSION-PUMP.md), [oráculos previos R1](../refinement/MARKDOWN-SESSION-PUMP-ORACLES-R1.json) y [suplemento normativo R2](../refinement/MARKDOWN-SESSION-PUMP-ORACLES-R2.json) copiados byteexact. SHA256 del contrato: `FF6397F64D8DEB10FE091FDB8ED25E528072BBFCDA5B079E3A8F6ED7D6807D8D`. Los hashes de ambos oráculos se conservan en los informes siguientes.

Los dos informes independientes finales se reproducen abajo, normalizando sólo saltos de línea. Originales privados: REVIEW-D2C-R2.md SHA256 `7430270C70A0807A8C9CBACFDCA2B21D894956AA0D66FB1BCEE40F04BFE2AE2D`; ADDENDUM-COMPOSICION-FINAL.md SHA256 `7B19AA72BD0011E75ACE7378E4405F9B7F4C57ED4BC3E04FA10417FD7A042FDB`. Las propuestas y observación intermedias permanecen fuera Git.

Cierre de diseño, sin aceptación de código o sesión. IP-P01/IP-P02 del registro de proveedores siguen abiertos. Source común e inspector en implementación privada; historia, pump, routing D3c, fences D4, gráficos, ConPTY documental y QA multiplataforma pendientes. Ninguna tarea RV principal se cierra por esta adopción.

# Revisión independiente D2c R2 — cierre limitado de diseño

2026-10-09. markdown_actions_adversary. Sólo diseño de sesión/pump Markdown completo, sin código, harness/runtime, QA visual ni edición canónica. R1/informe/oráculos16familias/fixtures preservados. D3c routing/Nu/help/inline combinados y D4 fences siguen obligatorios posteriores; no RVdone.

R2 exacta D2C-SESSION-PUMP-PROPOSAL-R2.md SHA256 06744E51E5521E8F6194A63539C801541BE7ACDEF77947A1FF40E5A6D0C0BA01; EXPECTED-D2c-before-output-r2.json SHA256 D4B291E83809DABB27FE05F1C5E24457CC47599F6714BD5C92A395A9FBA08229; MANIFEST-D2c-refinement-r2.json SHA256 90C397A83F2A3566DC9B1A858178D61629BD94AAF001F7DC304D4866EB9B2C9B. Ambos deliverables y8fixtures conservados verificados bytes/hash. R1 SHA DD3931BB197A4CFD082AEE50CDEE686004632F56731ADEE70CB555D87802B1B2 y EXPECTEDR1 SHA6B9FEA8DD351FFE6A8274DEE23C7E6EA7AD8D8B3EBE7B7CF5EBEF075B94759FD siguen intactos.

Base3db799c6d95513c08469295d592834b8bde79fe2 declarada/confirmada por coordinador tras adoptar b2; no gitstatus ni tests propios de código alegados. Criterios propios previos a R1/EXPECTED preservados: CRITERIOS-D2C-PREOUTPUT.md SHA BAF672C08751595F2585750C8D1095CE6AAFDF4367C0A592F990909463C9108F. Lecturas focales actuales: ActorArena/Lease, Document::reserve_consumer y Controller::prepare, ConsumerBudget, terminal/renderer, SourcePreparationModel/Owner, SafeTextModel receipt y funciones *_with. Las firmas nuevas siguen siendo propuestas y exigen compile/freeze/adversario antes pump.

## Disposición R01/R02/R03

**D2C-R01 cerrado en diseño.** ActorArena única real creada antes Loading y entregada al Controller, TRANSPORT_RESERVE3MiB/64MiB vigentes. FrameRetentionCharge distingue LeaseArena frente a ChargeConsumer del Document real. Loading/chrome/shell/Source payload y metadata nuevos usan Arena; body/slots/hit/pages/history documentales usan Consumer existente1MiB ya reservado dentro ESA arena, sin leaseArena adicional para la misma allocation. Framework2MiB es admisión de grids/readback físicos, no cargo que sustituya esos owners. No Consumer vacío/fake Document ni otro Frame256K por origen. Old charges no cambian por flagpane ni se liberan antes reset/destrucción real. Source visible/model/índices conservan ledger propio; no nueva cuota oculta de Actor64M para todo el modelo. Root debe verificar arena identity/getters y capacidades reales en adapter compilado, incluidos current+candidate mixtos y Source fallback sin Document.

**D2C-R02 cerrado en diseño.** SharedSourceAllocation mueve el MISMO Box único, mantiene su dirección/model/context/receipts y aporta metadata/Arc/handles Arena preadmitidos. SourceScopeLease conserva contexto HashReady o el exacto context/_source_metadata del DocumentView, no factory from_document por acción. Adopt wrapper obtiene el resultado únicamente mediante authority.take_validated tras join y devuelve mismo Box/scope/charges al fallar antes publicación. SourceLease retained bajo worker/result/Pending evita préstamo a UI y lastownerDrop anticipado; Current conserva SharedSource durante viewport preparation. TaskInput Build/Viewport y SourcePrepared::Viewport se incorporan al kernel Source existente, no owner nuevo, unsafe Send o from_shared repetido. Composición de tipos reales permite la dirección del diseño; NO se afirma Send/Sync compilado del adapter inexistente. source_shared/kernel común deberá compilar y medirse antes conectar pump, con cancel/supersede/drop y modelos/builds reales.

**D2C-R03 cerrado en diseño.** d fuera editor y CtrlD dentro abren selector5; foco j/k/flechas, Enter confirma, Esc restaura borrador sin revisión, mismo dominio no-op, qselector salida normal y CtrlC prioridad. Domain change incrementa checked SearchQueryRevision e invalida continuación/owner antiguo sin cambiar Binding de lectura/Source; no búsqueda automática ni historia por selector. Siguiente búsqueda sólo usa nuevo dominio/key y oldReady no publica/fallback. EXPECTED suplementario ahora contiene VT d+Downx4+Enter+/secret+Enter y CtrlD editor/confirm/cancel, sin hook de dominio sustitutivo. Original C06 abstracta se conserva como historial; suplemento es el oráculo operativo.

## Delta factual R04, cerrado por aprobación del coordinador

La frase R2 «Box allocation ya figura en SourceReceipt» necesita la precisión siguiente, aprobada explícitamente por coordinador y no atribuida a bytes privados de R2:

`PreparedSourceModel::allocation()` devuelve el receipt de SafeTextModel. presentation.rs128–158 calcula model_bytes=size_of::<SafeTextModel>() y known_heap_bytes EXCLUYE model_bytes; tampoco enumera por sí sola todo el Box<PreparedSourceModel>. El getter real output_box_bytes() devuelve size_of::<PreparedSourceModel>() completo.

El ledger Source retenido debe sumar con aritmética checked **known_heap_bytes + output_box_bytes()**, sin volver a sumar model_bytes ya embebido en ese Box. Si la partida previa era known_heap_bytes+model_bytes, añadir únicamente output_box_bytes()-model_bytes. Aliases del MISMO original siguen una allocation; wrapper/Arc counters/handles nuevos quedan Arena según R2. No cuota/API nueva ni clone/model rebuild. Boxes/current+candidate y scopes realmente vivos se enumeran hasta Drop físico; no multiplicar buffers internos por cada lease de la misma allocation.

## Fronteras preservadas

Composición sigue fullRect único con body/chrome/Fuente, Work8MiB/Frame256KiB/Symbol16KiB/framework2MiB y sublímite SourceWork256KiB, preflight/join y publicación pane-history-frame, output parcial poison/exit y retiredDrop fuera publicación. Restore antesjoin/cleanup2s, q0/CtrlC130/editor/cola/pending reader y Controller cleanup propio están conservados. ConPTY debe ejecutar pump/guard/worker/controller/D1 reales, con controles normales sinhooks y barreras sólo sobre progreso real; diseño/preoutputs no son PASS de sesión ni QA visual.

**El apartado de proveedores2 NO queda aceptado por esta revisión D2c.** El coordinador mantiene críticas IP-P01/IP-P02 (adapter real, gate antes de CADA índice, Weak y metadata retenida). R2 puede depender de su cierre futuro explícito como biblioteca b3/DocumentView independiente y previa al pump; no acredita APIs, pool nominal ni ledger por descripción. Inspector/history runtime y ese adapter deben freeze/compilar/revisarse por su corte antes implementación/adopción D2c consumidor. No resolver las plazas mediante tercer proveedor/copyIndex/mockReady o reduciendo inspector/no-query/tablas/metadatos.

**SIN_CRITICAS_PENDIENTES de diseño D2c R1+R2+delta factual R04 aprobado.** Los tres hallazgos R1 quedan cerrados en diseño; no aceptación de código inexistente ni del adapter b3 pendiente. Conservar el delta R04 explícito al integrar contrato final y oráculos; confirmar size_of/capacidades/SendSync/cargos reales antes outputs del candidato.

# Addendum independiente: composición FINAL D2c

2026-10-09. Revisión únicamente de equivalencia/composición documental frente a R1+R2+R04 cerrados. No repetición de investigación/API ni código/harness/runtime/ConPTY/QA. Sin edición canónica.

Candidato final corregido D2C-SESSION-PUMP-CONTRACT-FINAL.md SHA256 FF6397F64D8DEB10FE091FDB8ED25E528072BBFCDA5B079E3A8F6ED7D6807D8D verificado. Candidato inicial SHA256 26AFCA282B35773A0099B131E6FA70BF1A3E4BA51D1EF9F16324C6C14EDA9A97 preservado como identidad en observación original OBSERVACION-COMPOSICION-FINAL-R1.md SHA256 60D9FBDB084E9D71844894EAAECFB8E1228A433D6F6D631AA8031CD76D5D3E97.

Referencias de cierre: R1 DD3931BB197A4CFD082AEE50CDEE686004632F56731ADEE70CB555D87802B1B2; R2 06744E51E5521E8F6194A63539C801541BE7ACDEF77947A1FF40E5A6D0C0BA01; REVIEW-D2C-R2.md 7430270C70A0807A8C9CBACFDCA2B21D894956AA0D66FB1BCEE40F04BFE2AE2D. Base3db799c histórica válida, estado de b2 aceptado conservado; no Git limpio propio alegado.

**CF-01 cerrado.** §7 ahora declara normativos ambos EXPECTED originales inmutables y fija explícitamente el suplemento R2 SHA D4B291E83809DABB27FE05F1C5E24457CC47599F6714BD5C92A395A9FBA08229. Conserva Loading sinDocument/fallbackSource/mixedcharges/oldnew; mismoBox/model/Arc/context/metadata y SourceViewport real retenido bajo cancel/resize/stale/Pending/lastDrop; teclas VT d/CtrlD/flechas/confirm/cancel/queryrevision separado y nooldcursor. C06 abstracta R1 se conserva histórica; secuencia operativa R2 es d/Downx4/Enter//secret/Enter. Controls normales Source viewport sinhooks y delayedfactory tras progreso real, sizeof/capacidades/meters/SendSync antesoutput, no fakeReady/receipt.

Comprobación exacta del delta: retirar únicamente el nuevo bloque normativo §7 y reconstruir el separador original UTF8 produce SHA256 26AFCA282B35773A0099B131E6FA70BF1A3E4BA51D1EF9F16324C6C14EDA9A97. Por tanto la corrección no modifica ningún otro byte del candidato inicialmente contrastado.

Composición inicial/final conserva reglas R1 de fullRect/body+chrome, owner/poll/preparación outsideUI, pane-history-frame/poison, keyboard/editor/queue/exit/restorebeforejoin/Pending y ConPTY workerreal, más cargos mixtos/ActorArena única, SourceShared/task común, selector real R2 y R04 exacto known_heap_bytes+output_box_bytes sin model_bytes duplicado. No nuevas cuotas ni comportamientos añadidos. §5d mantiene IP-P01/IP-P02 ABIERTOS y exige adapter/gate por CADA índice/Weak-metadata-lastDrop con aceptación propia antes pump. La composición NO acepta esa library ni sus APIs/código actuales. Inspector/history/source_shared/selección/publicación reales y composición funcional siguen dependencias de implementación/freeze/AE.

Adopción prevista por coordinador: contrato byteexact como MARKDOWN-SESSION-PUMP.md y ambos EXPECTED byteexact con enlaces en informe canónico. Esta revisión acredita equivalencia del candidato privado indicado, no que esa escritura/commit haya ocurrido. D3 routing/nu/help/inline combinados y D4 fences siguen obligatorios; código/ConPTY/QA/matriz/RV permanecen pendientes.

**SIN_CRITICAS_PENDIENTES en composición documental FINAL SHA FF6397F6… frente al diseño R1+R2+R04 ya cerrado.** CF-01 es la única omisión detectada y queda resuelta por el delta exacto verificado. No aceptación de runtime ni ampliación del alcance revisado.
