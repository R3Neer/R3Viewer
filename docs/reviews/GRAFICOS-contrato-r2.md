# R3Viewer — revisión adversaria del contrato gráfico R2

Fecha: 2026-10-05. Candidato exacto: C:\Users\Usuario\AppData\Local\Temp\r3viewer-graphics-contract-r2.md. SHA256 comprobado: F444CAEC38A7744F6BF65BB83235BDC852BA31FD4CEF1651AFA67357F4A3AD0D. Base de producción: HEAD23722241092e0e94b1feeb4a1d1aa6ece8cdc674. Revisión de contrato observable, exclusivamente de lectura; no se implementó, compiló, instaló ni ejecutó QA de terminal.

Veredicto: SIN_CRITICAS_PENDIENTES dentro del contrato observable R2. GC-01–GC-04 cerrados por texto comprobable del candidato, tras relectura completa y contraste con las opciones canónicas y código actual. No implica seguridad o funcionamiento de APIs, rendimiento, píxeles visibles, ejecución nativa ni cierre del producto/backlog.

## Resolución de la ronda anterior

| ID | Evidencia de cierre R2 |
|---|---|
| GC-01 | Gramática decimal ASCII positiva1..1024, x ASCII minúscula, sin signos/espacios/ceros iniciales; ceros internos admitidos. Ejemplos 10x20/1024x1 válidos y 0x20/010x20/10X20/10x020/Unicode inválidos2; compartidos por flag/config/Nu. |
| GC-02 | Tabla R/N/D/F reemplaza explícitamente generalización R1: apertura/lectura/adquisición global1; daño o presupuesto de representación local0 con original/referencia; N/D/F sólo firma/IHDR33bytes máximo e integridad completa no validada. EOF incompleto0 se distingue de IO1; cuerpo no leído no se certifica. Límites no autorizan prefijos/truncamiento exitoso ni reservas hostiles; snapshot mutado exige rechazo antes de candidato. |
| GC-03 | Symbols emplea política efectiva vigente: flag > config > Auto; Always sustituye NO_COLOR con TTY, Never desactiva, Auto respeta NO_COLOR/TERM=dumb; redirección siempre plana. Casos config/flag preservados y raster separado. Contraste directo con src/config.rs:172–178 y README; sin regresión de AE S03. |
| GC-04 | PNG auto local/explicito con structure/language da2 antes de recursoIO; envelope Nu puede leerse antes de conocer ruta. Format explícito text/code/datos mantiene lector actual y PNG binario textual inválido1. Números de línea no-op sólo PNG, conflicto ambos flags2 vigente. Tipos binary/unknown/listas/registros mantienen presentación/validación actual y language no impone una prohibición nueva global. Literal/valor sin path con format png2, sin retirar capacidad futura. |

## Comprobación completa y límites

La R2 mantiene un origen único, distinción argumento/ruta versus valor Nu y -p/-v incompatibles; no adivina rutas por metadata. Nuevos flags son comunes al ejecutable y módulo Nu, sin default que tape config y conservando encoder inicial. Selección force no certifica soporte ni píxeles y no permite eludir geometría/lifecycle/redirección. Hints no se convierten en garantías; alternativas reference/symbols mantienen identidad de recurso.

Geometría desconocida no inventa tamaño de celda. Alpha/fondo mate, proporción, zoom/vuelta, clipping/resize y retirada están ligados a evidencia futura. Inline scrollback persistente se distingue de limpieza TUI y de liberar tareas locales. Probes conservan propietario único, deadline/volumen acotados, entrada documental separada, cancelación/teclas/restauración; rutas stdio global, detached y mutation tmux no quedan autorizadas. La adaptación técnica debe pasar su propio gate.

Trabajo/cache/memoria/payload/cancelación tienen presupuestos y medición abiertos; no se certifican por el contrato ni se decide arquitectura gratuita. Gates G1–G7, visual/matriz nativa y candidato de biblioteca siguen abiertos. G0 puede cerrarse para este contrato exacto. Una integración/cambio de contrato exige revisión pertinente nuevamente.

PNG local constituye un corte vertical, sin retirar JPEG/WebP/GIF/SVG/PDF/Markdown/math/Mermaid/HTML, composición mixta, persistencia completa, recursos remotos mediante acción explícita ni Windows/macOS/Linux. S03 continúa activo y las39tareas conservan aceptación original. Halfblocks/symbols no se elevan a solución final. Los límites de acceso a QA no reducen aceptación gráfica final.

Autoridad local leída en R1 y mantenida en R2: AGENTS, README, CONTRIBUTING, docs/spec/{DECISIONES,REQUISITOS,CONTRATOS}, docs/architecture/ARQUITECTURA, docs/BACKLOG, docs/VALIDACION, docs/research/GRAFICOS_API_PREPARACION y docs/process/METODOLOGIA. Relectura R2: propuesta completa exacta, src/cli.rs, src/syntax_selection.rs, src/input.rs, src/nu_input.rs; política efectiva config y módulo Nu leídos en esta revisión continuada. Las fuentes primarias iTerm2/Kitty consultadas en R1 documentan protocolos, no R3Viewer; R2 no introduce hechos externos nuevos que sustituyan gates.
