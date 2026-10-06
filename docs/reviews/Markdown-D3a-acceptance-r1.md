# D3a R1 — revisión y aceptación independiente acotada

2026-10-06. Autor raíz; revisor/aceptación graphics_ipc_acceptance distinto. Candidato `markdown-d3a-private-r1/MANIFEST.json` SHA256 `6AFBCE5A55B5C38333740F132747EC387F312567507F6F9654BDC3B20B550E58`,15registros; base Git `a431ea2275a087f4f745e1d326f9f03fe6001f3c` con BASE-CONTEXT534. Propuesta D3 R2 cerrada en diseño, no aceptación automática de código.

**Dictamen: SIN_CRITICAS_PENDIENTES en las seis fuentes D3a y las APIs ejercitadas.** Aceptación de incremento privado selector/sharedSource solamente. No adoptar ni publicitar FormatMarkdown en CLI hasta routing D3c coherente. Main/Nu/routing/TUI/inline, loading/pump/q/restore, fallbacks de transición, D2/D4 y producto/RV no se acreditan aquí.

## Identidad, copia y contraste de fuente

Verifiqué15registros del freeze antes y después. BASE-CONTEXT identifica534archivos; candidato cambia exactamente5 existentes y añade document_selection.rs. Copia privada535archivos antes de tests, byteexacta; después535 siguen byteexactos al candidato y sólo se añadió tests/d3a_independent.rs. SETUP-VERIFICATION y AFTER-VERIFICATION conservan hashes/bytes; source/bin/manifests del autor no se modificaron.

Lectura de los seis deltas: cli añade únicamente Format::Markdown; lib registra módulo. DocumentSelection puro usa formato explícito o extensión md/markdown case-insensitive de SourceOrigin::LocalFile real, comprobando textual_input. Nu no textual explícito devuelve InputError2; Auto no reinterpreta sus metadatos. Profile Extended es el elegido. Language no altera selector. SyntaxSelection conserva precedencia language explícito y añade Markdown default sin extensión que lo derrote; StructureSelection consulta selector y evita JSON/diagnóstico estructural falso bajo Markdown. No ciclos: textual_input no vuelve a select estructura/documento.

DocumentSource.original_handle clona Arc, sin copiar String, alterar origin/generation/request/hash ni reabrir. SafeTextModel::new delega from_shared; mantiene algoritmo/proyección/rangos/índice/límites previos y almacena ese Arc. from_shared añade cancel temprano antes de visible allocation; no eleva cuota de Source ni afirma Consumer1MiB/Actor64MiB como límite Source/RSS. Constructor fallido no modifica el Arc del caller.

Diffs guardados en privado. El contraste inicial con working cli omitió generar su diff porque el hash raw no coincidía BASE; el intento de leer DIFF-cli inexistente se observó como error de lectura, no fallo semántico. Contraste posterior demuestra que CRLF→LF único coincide SHA BASE `D3DD8D2DA7415D91E9FBE63AEAA147A80A8CC4C6868802B022AD1358B64A45F2`; DIFF-cli final muestra sólo la variante Markdown. No se confundió hash raw con blob ni cambió fuente compartida.

## Oráculos propios previos y resultados reales

EXPECTED-before-output.json se escribió antes de runtime. Seis tests de integración propios usan APIs públicas reales, no Debug/goldens del candidato ni tests del autor como aceptación:

1. Origen real/extensión mixedcase y overrides; .mdx, archivo llamado sólo .md, literal/stdin/Nu no infieren; languageRust sobre file.md no altera selección.
2. Tres decisiones independientes para Markdown explícito sobre .json: Extended, Structure None sin diagnóstico, Syntax Markdown. Rust/desconocido afectan sólo SourceSyntax; auto.md+structure no genera diagnóstico falso.
3. Nu String/StringStream aceptados explícitamente pero no Auto; Nothing/Bool/Float/Date/Record/List/Binary y ByteStreams Unknown/Binary rechazados2, incluso con metadata/rutas aparentes. No se abre esa metadata.
4. Fuente4095ASCII+carácter multibyte/controlC1/ESC/CRLF+END cruza chunks: DocumentSource/SourceModel comparten Arc; visible esperado seguro, source_range del spelling C1 vuelve al scalar original; wrapper antiguo produce la misma proyección/line_count con su Arc propio. Hash/generation/request comprobados y bytes inmutables tras drop modelo. Conservación del origin se acredita estáticamente en el delta del getter, no como aserción runtime adicional.
5. Cancel ya activo sobre fuente vacía y no vacía impide SourceModel/DocumentSource; bytes originales intactos.
6. Frontera real SourceLines:999999LF admite1000000líneas;1000000LF devuelve el Limit existente. Fuente/hash/Arc conservados. No elevación de límites, no prefijo publicable y no parser/SLA128MiB inferidos.

Resultado TEST-attempt1.log: **6PASS,0FAIL,0filtered**, runtime0.50s. Compile28.68s, cargo offline/locked, sólo test target propio. Sesión agent-local50253 terminó exit0 y no se reinició por timeout/observación. No nuevos Child/Job/workerOS del producto ni pruebas de contención/ConPTY. El bin propio se preserva en bin/d3a-independent.exe SHA256 `05D237C5FB205D1723A69C613E708407D8C7C579D407818BE33D4A873656597D`; corresponde a candidato byteexacto más integración de tests propia, no al bin unit del autor.

Se reutilizó únicamente cache/target privado anterior de este revisor, sin target autor o compartido. Warnings de canonicalización/hardlinks bajo sandbox se conservan en el log; no alteraron exit0 ni las pruebas. Scripts Python emitieron diagnóstico de localización de ejecutable y luego terminaron exit0 con verificación completa; no se silencian como inexistentes. No fue necesaria escalación, rescate externo, cambio de expectativas ni corrección de código candidato.

## Límites de la aceptación

Las21pruebas reportadas por autor, fmt y clippy son evidencia de entrega; no se suman a nuestros6 para inflar aceptación independiente. No se rerun megacorpus/Source gigante/OSworker/full A–C. La prueba de límite1MiB SourceLines no acredita visible256MiB frontera,128MiB total, allocator total, read/hash SLA o transición TUI; esos contratos previos siguen vigentes y la adaptación sólo conserva sus métodos. Loader real Nu/CLI/main no se ejecutó: éste es selector API con inputs tipados, no routing funcionando ni stdout/exit del producto final.

No pendiente concreto de código dentro de este corte identificado. Las garantías D3-R01/R02/R03 se implementarán/aceptarán en D3b/c, no se cierran por tener selector y from_shared. D2/D4, NativeUnix, QA visual y producto completo continúan pendientes.
