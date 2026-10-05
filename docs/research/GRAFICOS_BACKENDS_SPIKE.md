# Experimento privado gráfico G1/G2 — R1, evidencia de autor

2026-10-05. No modifica producto, dependencias de producción, perfiles, terminales ni sprint activo. Base contractual: `docs/refinement/GRAFICOS-PNG.md` SHA7E14FF471E0355C5B92EA4DB19C187514162971E83F3246D6898AEFEBD9F77DB (copia canónica); R2 temporal original SHA F444CAEC38A7744F6BF65BB83235BDC852BA31FD4CEF1651AFA67357F4A3AD0D y revisión independiente sin críticas. Este experimento no cierra G1/G2 completos, RV ni QA visual.

## Resultado

Los constructores públicos de Sixel, iTerm2, Kitty y Halfblocks compilan juntos en Windows x86_64 con Ratatui0.30.2/Crossterm0.29.0, ratatui-image11.1.0 con default-features=false y crossterm, image0.25.10 sólo png. No Chafa, image-defaults ni codecs adicionales en cargo tree. Se construyen directamente: `Sixel::new`, `Iterm2::new`, `Kitty::new` y `Halfblocks::new`; jamás Picker, ThreadProtocol ni StatefulImage. Las secuencias se renderizan en Buffer en memoria y se inspeccionan como números JSON; ningún escape gráfico se escribe a un terminal.

Viabilidad parcial: iTerm2 es candidato sencillo para representación preparada fuera de UI, pero no adopción directa. Kitty directo sin compresión viola presupuestos con una entrada válida dentro del límite. Todas las APIs acumulan representación sin callback de cuota/cancelación; aislarlas permite terminación externa, pero no las convierte en codificadores con reservas controladas.

## Reproducción e identidad

Directorio privado `D:/Tools/R3Viewer/.tmp/graphics-gate`. Rust1.99.0 b940084d7 / Cargo1.99.0 5f94df478; Windows x86_64, ejecución real; no ejecución macOS/Linux ni terminal gráfico. Entorno de proceso: `. D:/Tools/R3Viewer/scripts/dev.ps1`.

1. `cargo build --manifest-path .tmp/graphics-gate/Cargo.toml --locked`.
2. `C:/Python314/python.exe .tmp/graphics-gate/make_corpus.py`, después `alpha_corpus.py` (añade fixtures palette/illegal-depth sin modificar fuentes).
3. `run.py`, `assert_results.py`: corpus funcional y oráculos explícitos.
4. `job.py`, `job_large.py`: únicamente hijos propios del ejecutable hash registrado, Job privado sin nombre, lease controlador, límites consultados tras asignar.
5. `cargo fmt --manifest-path .tmp/graphics-gate/Cargo.toml -- --check` y `cargo clippy --manifest-path .tmp/graphics-gate/Cargo.toml --locked --all-targets -- -D warnings`: PASS.

MANIFEST.json contiene hashes de fuentes, Cargo.lock, scripts, corpus sintético, logs, resultados y exe; excluye artefactos internos target y el informe/manifest para evitar autorreferencia. SHA256 MANIFEST `E670491A61475E438F1AD1ACD43F2EA1FC97D3DA2049B1C51CD1016E05F614A3`.

- Cargo.toml `C9C8952F9109D612A2B2F702749D745A4ED6B7ADBA4AE203960F78B73CC0C0B3`.
- Cargo.lock `D26D5D70E10CA9D4AAE56913BC1DE3D96E400AD37CC54EF431A7FB9200991A32`.
- src/main.rs `5B67C2EA05F242A9843307959C7B4F90090788926BE93278CE03437B1938E71C`.
- exe `A10861CEA1A913C77B996EEE66422B5F1DFD25263CBE59F5928728F08A543D01`.
- results.json `6A3060FFA68EFE5ADE3007ACCFE1801C99041C4EE10CF2CBFE933D5C836C6061`.
- job-results.json `34F269100E5933B49C7C5C5E4038019E5AE0B30E97E7855140E68954EBCC859C`.
- job-large-results.json `1C720FC479871564C23755DF255403725410DAF70757F111617D8E6694739C66`.

ratatui-image procede del crate exacto ya investigado `.tmp/graphics-research/ratatui-image-11.1.0`; no se modifica. Crate SHA9593D10C61EC2007F429D1FAFC4F4A19FECDBAE4D06B531A2D505227FB82F5EA según investigación previa, no recalculado por este informe. Referencia Job producción inspeccionada `src/worker_process.rs` SHA FD96735DA2C3FFFB0C75A7976B8C88F6103EF374A561D8DCDD4F4BF62B8E1D3A. El harness de Job reproduce flags y cuota; no reutiliza/muta esa fuente. Se observó cambio ajeno en structure_model.rs durante trabajo, preservado; no afirmar árbol global limpio.

## Corpus y cabecera

15 PNG propios: alpha RGBA0/128/255; palette+tRNS0/128/255; 1×1; cuadrado64;320×20 y20×320; truncado20B; CRC IHDR incorrecto; IDAT corrupto; cero dimensión; IHDR16M justo sin cuerpo;16M excedido; u32max×u32max; bitdepth ilegal;4000×4000 sólido válido. 50 ejecuciones y86 aserciones manuales de estado/cabecera/alpha/cuota pequeña PASS. `ASSERTIONS.md` no es aprobación independiente.

Sólo cabecera: abre archivo regular en lectura, comprueba longitud declarada≤64MiB, lee hasta33B, firma/primer chunkIHDR13B/CRC/dimensiones/bitdepth-color legal/compresión0/filtro0/interlace0|1; productos u64 antes reserva, máximo16,000,000px. Referencia no lee cuerpo. IDAT corrupto tiene header_only y luego decode_error;16M+1 rechaza pixels sin raster; el IHDR16M exacto admite dimensiones pero decode rechaza falta de cuerpo. EOF cabecera recuperable en harness; no simula la interfaz/exit exacta del producto.

Alpha decoder: RGBA primeros tres pixels `[0,0,100,0,1,0,100,128,2,0,100,255]`; palette+tRNS `[20,40,60,0,80,100,120,128,140,160,180,255]`. Esto prueba datos decodificados, no composición ni píxeles terminal. Halfblocks hace resize_exact y to_rgb8: directo pierde alpha y puede deformar; requerirá composición mate y fit físico explícitos. El harness fija8×4celdas para comprobar API y deliberadamente no acredita proporción.

No prueba el límite de adquisición64MiB exacto, modificación concurrente del archivo ni snapshot/hash. El descriptor no se reabre, pero comparar longitud/hash y rechazo de mezcla de versiones sigue pendiente. No usar este loader privado como producto ya aceptado.

## Mediciones

Heap: GlobalAlloc/System instrumentado, suma tamaños solicitados vivos/pico por alloc/realloc/dealloc en Rust. Excluye overhead del allocator, stacks, OS y asignaciones ajenas al allocator; no es RSS ni cuota dura. Commit: QueryInformationJobObject devuelve PeakProcessMemoryUsed/PeakJobMemoryUsed del Job exclusivo. No se midió memoria Python padre ni agregado producto/caché/IPC.

Corpus pequeño cinco imágenes, debug caliente: Sixel3.48–16.01ms (pico heap≤505,609B); iTerm2~0.10–4.20ms (≤114,870B); Kitty0.067–0.787ms (≤158,584B); Symbols0.174–1.698ms. Tiempos particulares sin SLA, no medición de lectura+emisión ni inicio frío. `buffer_symbol_bytes` suma símbolos almacenados: incluye payload incrustado y placeholders, excluye SGR de estilo/terminal y overhead. Es suficiente como cota inferior para detectar violación Kitty, no cuota exacta en transporte.

4000×4000 válido (16M,64,000,000B RGBA), todos bajo Job512MiB antes de abrir/decode:

| Operación | Heap pico | Job proceso commit pico | Datos en símbolos | Encode |
|---|---:|---:|---:|---:|
| Decode |64,431,302B |65,593,344B |sin encode |~0.47s |
| Kitty sin compresión |342,569,893B |344,543,232B |85,604,366B |1.270s |
| iTerm2 sólido |65,624,158B |67,514,368B |421,642B |1.709s |

**Kitty directo NO ADOPTABLE:** excede32MiB de payload y256MiB de objetivo adicional incluso sin medir padre. Chequeo posterior `within_payload_32m=false` sirve para diagnosticar experimento, no limita reservas. Mantener soporte de origen16M y preadmitir representación destinada al viewport/crop/zoom finito antes de constructor; no reducir máximo de imagen sólo para obtener verde.

Compresión Kitty es parámetro público pero no garantiza cuota: ruido puede no comprimir y deflate puede ampliar. Kitty interno usa String/Vec acumulativos privados, sin Writer acotado. Calcular antes al menos rawRGBA→base64(ceil(n/3)*4)+overhead de chunks y copias; presupuesto debe admitir temporales reales y Job. Para iTerm2, PNG solid comprime bien pero peor caso de ruido se acerca a raw+overhead y base64>32MiB al máximo; `.write_to` del constructor usa Vec sin cap. Adaptación candidata futura: PNG encoder a Writer que rechaza antes de superar presupuesto más salida base64 acotada, o preadmisión conservadora de viewport; no se implementa aquí. Sixel retorna String tras cuantizar y hay copia RGBA: no demuestra límite propio; revisar icy_sixel/quantette y aislar antes de adoptar. No se ejecuta extremo Sixel completo ni nuevo corpus gigante sin necesidad.

## Cancelación y aislamiento observados

Python crea solamente exe propio. Hijo emite READY y bloquea stdin antes de abrir/decode. Controlador crea Job no heredable, flags KILL_ON_JOB_CLOSE0x2000 + PROCESS_MEMORY0x100,536,870,912B; consulta Job exacto y IsProcessInJob(handle,job)==true; después libera handshake. DECODE/ENCODE markers inmediatamente anteriores a llamadas síncronas. Tras20ms en fase, hijo seguía activo sin resultado final; cerrar lease Job y wait≤1s:

- Decode:5.74ms reap.
- Sixel encode:10.50ms.
- iTerm2 encode:5.53ms.
- Kitty encode:7.64ms.

No huérfanos del nombre exe en consulta final. Windows cierra proceso con exit0 en estos ensayos: no es Ctrl-C130 del producto ni aceptación UX. Resultado prueba terminación externa de trabajador activo y recogida de propio proceso; no checkpoint cooperativo del codec ni exacta instrucción interna en el instante de kill. El código no escribe representación parcial al terminal. Asignación Job antes de trabajo evita que decode empiece sin contención; proceso bootstrap antes READY no está aún en ese Job. Cuota512MiB es real configurada/consultada, pero no se hizo prueba de alcanzarla ni es objetivo agregado256MiB.

## Gates que permanecen abiertos

G1 sólo evidencia ruta de construcción sin Picker/stdin/tmux, por inspección de fuente y ejecución normal sin handshake; no trazado OS exhaustivo. Ningún probe implementado: negociación, cola respuestas/teclas, ausencia/tardanza, Ctrl-C/restauración/segunda sesión, raw/inline/redirección permanecen abiertos.

G2 parcial: compilación/corpus/cabecera/alpha/medidas y kill/reap externos. Snapshot mutable/hash, verdaderos límites preencode/temporales/payload, decoders16bit/extremos hostiles, ejecución bajo límites de cada SO, agregado padre+hijo/caché, retry y deadlines UI/producto pendientes. `image::Limits.max_alloc` es no estricto por API oficial; no sustituye aislamiento ni reservas propias. G3/G4/G7 visual/nativo no ejecutados. G5/G6 integración/config/cache/IDs/cleanup no ejecutados. Kitty placement transmite APC aquí en Buffer; no prueba liberación de datos terminal. Sixel e iTerm2 comienzan con clear_area y iTerm2 incluye doNotMoveCursor=1: **no imprimir data como inline**. Render Buffer no transmite ni certifica alpha/clipping/scrollback.

## Fuentes primarias

Consultadas2026-10-05 o fuente local exacta. [image0.25.10 Limits](https://docs.rs/image/0.25.10/image/struct.Limits.html): width/height estrictos; max_alloc mejor esfuerzo, no garantía general. [Windows Job Objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects): grupo/límites/terminación del Job. Fuente local exacta ratatui-image11.1 src/protocol/{sixel,iterm2,kitty,halfblocks}.rs y halfblocks/primitive.rs; constructor Kitty documenta id y compress, encode interno sin Writer. URL docs.rs Kitty devolvió error interno; no atribuir evidencia web a consulta fallida. [Kitty protocolo](https://sw.kovidgoyal.net/kitty/graphics-protocol/) e [iTerm2 imágenes](https://iterm2.com/documentation-images.html) ya referenciados por contrato; este experimento prueba bytes de la librería, no garantías de terminales. Licencias/dependencias registradas por Cargo.lock/paquete; no se distribuye bundle.

## Revisión independiente del alcance parcial

[Revisión final](../reviews/GRAFICOS-backends-spike.md): SIN_CRITICAS_PENDIENTES para experimento parcial,93invocaciones (50replays+43independientes) y4cancelacionesJob; 41hashesoriginales intactos. Incluye64MiB/+1 adquisición yRGBA16 como nuevos casos, sin atribuir al autor esa ejecución. No adopción directa ni cierre G1/G2. Kitty16M excede cuotas; iTerm2 sigue candidato con límites preencode/snapshot/inline/visual pendientes. Original autorSHA8C10F75F6A9D0946BA72F7023BF9753CD520FB79F4802356C38F3376C4DAC831; original revisiónSHA7320ADCF8E2D089429C834D4EE238979DA6D2FBBAF3C706D91318E952A1D6CE8. Refinamiento paralelo, no nuevo sprint ni cumplimiento RV gráfico.
