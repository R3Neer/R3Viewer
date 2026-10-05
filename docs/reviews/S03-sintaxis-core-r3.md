# S03 C — revisión adversaria de código R3

Fecha: 2026-10-05. **SIN_CRITICAS_PENDIENTES** para los tres módulos neutrales congelados. Las seis críticas R1/R2 quedan cerradas dentro de este alcance; no se han encontrado defectos nuevos accionables. No acredita integración del ejecutable/TUI, QA visual ni soporte Unix.

## Identidad y alcance

Hashes comprobados al entrar y al terminar:

- `src/syntax.rs`: 9B9C8B06D4C245457B81C45BAAB6F3099D667F9AB2EE56ECF366F5479201DD57.
- `src/syntax_worker.rs`: 17A0872BAFEA68B190F57D3DC2D1983E134D9F70A78003A3FE026C6B4B79324F.
- `src/syntax_process.rs`: 6DC309050A87552BD6A6993DE9126928D52A2F461E23D515A70FDC316E21F6AB.
- Informe autor leído: `.tmp/s03-product-syntax/EVIDENCE-R3.md`, SHA256318EF9001B80C4A0CF2C482F5C005AC993E832C6684C49357B5E5DCF5B716332.

Contrastados propuesta aprobada, sus enmiendas y C04–C07. Revisión integral de controlador/protocolo/parser/cache/contención, incluyendo diff contra copia R2 preservada. Worker/process fueron releídos y conservan sus hashes. Ninguna edición de producción, arnés autor, índice Git o perfiles.

Arnés independiente: `.tmp/syntax-adversary-r3`, paquete/binario/target propios. Referencia a los módulos reales mediante `#[path]`, sin copia instrumentada. Main temporal SHA25693E9089A71C81AE06042BCD14A7DBEBBF238B8A3BDED6C08996CA82807A9E2C4. Executable release SHA2567946C8ED531ED74A252E6F7BAE41E1DA0D449A37FE15987EC4F3BE9EDB9073BD. Windows nativo, dependencias offline Syntect5.3.0/Onig/windows-sys. Delays/salida77 existen solo en main temporal para escenarios de protocolo o salida del worker propio. No GUI/TUI ni corpus externo.

## Cierres R1/R2 y evidencia independiente

| Hallazgo | Verificación R3 | Cierre |
|---|---|---|
| C-R1-01, petición durante initialize | Boot retrasado700ms; publicar documento2 y Request2 antes de arrancar parser real. Se reciben Ready2 y Styles2 para Request2, sin reenvío. | Handler Changed vuelve al ciclo de Update; controller comprueba Update/revisión bajo el mutex antes del take. |
| C-R1-02, plazo de ACK | ACK de Chunk demorado700ms con límites por defecto. Plain LineTimeout a551ms desde setter, incluidos spawn/Boot. | Fase IPC min(per_line,total original); ACK no renueva startup. |
| C-R1-03, muerte idle y cache | Primer Styles real; hilo temporal termina su worker con77 al segundo. Tras1300ms poll entrega Plain WorkerExited; solicitud cacheada retorna0 y memoria controller/child/cache0. | Supervisor idle y previo a cache; Plain prioritario, caché invalidada y recursos recogidos sin solicitud adicional. |
| C-R1-04, vacíos sin coste |20.000 viewports distintos sobre fuente vacía producen Styles vacíos válidos y retención de cache0. Prueba nueva verifica metadata, cap256, eviction LRU y Clear liberando backing buffer. | Vacíos excluidos; capacidad de metadata y spans cobrada; entradas limitadas. |
| C-R2-01, CPU tras Clear | Clear7, esperar200ms, request7 devuelve0;1000ms wall y0ms CPU user+kernel medida del proceso propio, ningún evento. | has_document y admisión comparten mutex con Set/Clear; el request imposible no queda pendiente ni impide condvar. |
| C-R2-02, take de Progress | Prueba con barreras del módulo real pasó independientemente. Además300 reemplazos durante parseo real de50.000líneas, variando el momento, producen siempre último Styles y ninguna petición perdida. | take_current_request valida stopping/cancelled, revisión/generación, has_document/Update y generación de petición bajo el mismo mutex antes de take. |

Outputs separados en esta carpeta: `syntax-review-r3-replace-init.txt`, `syntax-review-r3-startup-ipc.txt`, `syntax-review-r3-idle-exit.txt`, `syntax-review-r3-cache-empty.txt`, `syntax-review-r3-clear-request.txt`, `syntax-review-r3-progress-race.txt`.

**Distinción de concurrencia:** C-R2-02 no se reprodujo empíricamente en R2: fue una intercalación del código. En R3 su exclusión se sustenta en la transacción única y en la prueba determinista con barreras de esa transacción real. Esa prueba usa cliente dormido sin controller/child, métodos UI públicos y código de producción; no simula el runtime completo ni reproduce el antiguo binario. Los300 reemplazos con procesos reales son una muestra adicional, no demostración de todas las intercalaciones.

La medición0ms CPU tiene granularidad de GetProcessTimes y no significa coste absolutamente nulo bajo toda planificación. Lo observado elimina el giro continuo que anteriormente consumía985–1000ms CPU por segundo.

## Verificación integral

Se ejecutaron independientemente **10 pruebas unitarias reales**, release/locked/offline: todas PASS. Cubren admisión antes de Set y después de Clear, frontera wait-status/Progress-take con reemplazo/supersede/cancelación, metadata/cap/eviction de cache, prioridades Plain, protocolo/counts/truncación, cobertura UTF8, roles/multilínea, Clear/Restore/checkpoints y límites línea/spans.

La nueva transacción no retira peticiones de reemplazo ni peticiones canceladas; continúa admitiendo supersede de la generación actual. El total del request anterior se comprueba antes de sustituir y Progress no lo renueva. Respuestas de generación/request anteriores no desplazan el slot actual; el fallback de request obsoleto continúa invalidando toda la generación vigente. Clear impide solicitudes nuevas hasta Set, mientras Set pendiente permite encolar su viewport antes de Ready.

Límites de frame/count antes de reserva, chunks secuenciales con ACK, snapshot original, límites de línea/source/spans/operaciones, UTF8/cobertura íntegra e inline sin prefijo parcial conservan coherencia. ParseState y ScopeStack completos se clonan, las operaciones se aplican antes del rango incluido vacío, y checkpoints permanecen acotados. Caché cobra capacidades y limita entradas; contabilidad de fuentes/cache sigue sin presentarse como heap total exacto.

El controlador conserva ownership exclusivo de spawn/IO/kill/wait/join, cierre anterior antes de sucesor y shutdown owner para join fuera de TTY. Job privado512MiB committed y kill-on-close se asigna antes de Init; lease reader independiente detecta EOF/truncación/cola llena. Estas rutas no cambiaron en R3. Se conserva evidencia anterior de lifecycle del autor como evidencia separada; este adversario no repitió en R3 todos los escenarios parent-loss/preassignment/hermano/IOblocked ni un monitor de todos los instantes.

## Límites de aceptación

La aprobación corresponde únicamente a estos tres hashes y API neutral. No revisa integración root Cargo/main/UI, gates --colour/format/language, composición de offsets seguros, estado visual, restauración de TTY ni comportamiento conjunto con Nu. Una integración posterior requiere revisión pertinente del conjunto.

Fmt/Clippy y corpus/lifecycle ampliados de EVIDENCE-R3 son resultados del autor; no se atribuyen como ejecuciones independientes aquí. Sin ejecución macOS/Linux ni aceptación de contención Unix; UnsupportedPlatform sigue siendo impedimento explícito de RV107. No se cierra el sprint ni se acredita QA de producto.
