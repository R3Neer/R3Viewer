# S03 shell — revisión independiente del diseño de integración R1

2026-10-05. **SIN_CRITICAS_PENDIENTES** en el diseño propuesto. Aprobación de propuesta únicamente, condicionada a cerrar revisión y AE del prototipo exacto antes de adoptarlo. No implementación, runtime, QA visual ni aceptación global S03.

Objeto: `S03-shell-integration-proposal-r1.md`, SHA256 verificado `92F8B0BE201CD15D5A5083CFF68650615EAEA4BCDEDECC43F415DE32134A3A62`. Contraste con GRAMATICAS-SHELL y su revisión, build.rs vigente, language_catalog.rs, syntax_selection.rs, syntax_worker.rs/syntax.rs, límites C de S03 y contención vigente. Sólo lectura de producción; informe separado del dictamen de código privado R2.

## Coherencia de catálogo y selección

Un único builder produce SyntaxSet/dump y catálogo Rust; recursos son los fijos Nu/PS exactos revisados más defaults/TOML. Fallo load/enlace aborta build, no elimina sintaxis. Procedencia/ledger/MIT y rerun-if-changed cubren recursos; ningún input de usuario genera sintaxis ni obtiene módulos/perfiles. Cambiar recurso invalida aprobación y requiere oracles/revisión nuevos.

La propuesta conserva el orden real del set y precedencia vigente, no otra lista manual. Actualmente resolve recorre extensiones en reversa y nombres hacia delante; prueba existente compara con set.find_syntax_by_extension y preserva colisiones como h→Objective-C. La integración debe mantener ese comportamiento tal como declara el diseño y la prueba de catálogo, además de aliases/selecciones nuevas.

pwsh se limita al camino explícito --language; extensión real .pwsh auto queda sin gramática. El diseño exige esa prueba negativa, y no autoriza añadir alias genérico al resolve de auto de modo que lo deduzca por accidente. Nu/PS por sus extensiones reales, mayúsculas y nombres; origen tipado/pipeline/cwd/metadata no se reinterpreta como archivo. Preservar resolución existente incluye error2 de --format text+--language, y plano sin child por text/Plain/never/redirección según contrato vigente. --language permanece sólo sintaxis, sin estructura ni eval.

## Worker y aceptación

La propuesta conserva Job antes de Init, único parser/scheduler, protocolo/gen/request/viewport, cuotas fuente/línea/frame/spans/ops/scope/cache y deadlines originales. No aumenta presupuestos para obtener verde ni habilita Unix porque el recurso compila. Timeout/grammar/worker/limit mantienen Fuente íntegra+diagnóstico/latch y reap; no continuar desde estado roto ni admitir otro child sin recogida previa.

El gate exige reproducir269intervalos y16endStacks independientes con procesamiento/límites del worker real. Si IPC sólo conserva RoleSpan, un harness del mismo procesamiento puede exponer stacks para verificar fidelidad, con parser exacto separado para endStack identificado como evidencia adicional; ni color/RoleSpan colapsado ni el runner privado anterior sustituyen esos intervalos. Spans/continuidad/longitudes y contenido completo complementan, no borran, el gate léxico.

CLI/Nu, inline/redirect/typed, strings hostiles/control/Unicode, fallback/cancel/reap/exclusión y TUI funcional de terminal tienen comprobaciones propuestas. Límites concretos ya vigentes (línea65536bytes incluido terminador, fuente128MiB, etc.) siguen normativos; faltante de gate runtime continúa pendiente, no se certifica por tests unitarios del parser. Viewport/checkpoints/restauración de terminal y I/O owned pertenecen a aceptación del worker integrado. ConPTY funcional no es QA de paleta/visual. Crosscheck lector+wire no es runtime/contención Unix.

Congelar implementación exacta, revisión adversaria y AE distinta del implementador antes de commit de aceptación mantiene independencia y no transforma este informe en aprobación de código futuro. RV105/106/107 y S03 retienen gates visuales/Unix/estrés/restantes formatos. Sin instalación perfiles/remoto ni flags/dependencias runtime nuevos.

**SIN_CRITICAS_PENDIENTES** sólo para el diseño R1. Implementar dentro de este contrato cuando las condiciones previas de prototipo se cumplan; la adopción resultante necesita su propia revisión y aceptación reales.
