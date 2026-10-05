# Revisión adversaria — experimento gráfico G1/G2 R1

Dictamen: **SIN_CRITICAS_PENDIENTES dentro del alcance experimental parcial descrito**. No autoriza adoptar constructores directamente ni cierra G1/G2, RV o QA visual. Las restricciones y los límites no acreditados están reconocidos por el autor y siguen abiertos.

## Identidad

Informe autor r3viewer-graphics-gate-r1.md SHA8C10F75F6A9D0946BA72F7023BF9753CD520FB79F4802356C38F3376C4DAC831. Snapshot D:/Tools/R3Viewer/.tmp/graphics-gate: MANIFEST E670491A61475E438F1AD1ACD43F2EA1FC97D3DA2049B1C51CD1016E05F614A3, 41 entradas verificadas por tamaño y SHA, ninguna divergencia. Exe A10861CEA1A913C77B996EEE66422B5F1DFD25263CBE59F5928728F08A543D01. Fuentes/binarios/script/corpus congelados no modificados; todos los resultados propios están en esta carpeta de revisión.

Se leyeron AGENTS, contrato gráfico canónico, investigación, fuente completa del harness, scripts de Job/corpus/aserciones, Cargo.toml/lock y logs finales. Construcción/Clippy PASS son evidencia del autor inspeccionada; no atribuyo recompilación al adversario. El exe original sí fue ejecutado independientemente. No se ejecutó GUI, terminal gráfico, Nu ni código de producción.

## Reejecución y casos nuevos

independent.py / independent-results.json:93invocaciones CLI PASS.

- Reejecutados los50casos del autor sobre el exe exacto, comparando estado/código, dimensiones, bytes cabecera/decoded, canales, payload y decisión de cuota cuando presentes. No comparo microsegundos ni heap como oráculo determinista. Las86aserciones del autor son50estados+14lecturas cabecera+2canales alpha+20payloads pequeños, y se inspeccionó esa cuenta.
-43casos nuevos generados independientemente con struct/zlib:9truncaciones0..32B,3firmas dañadas,2length/typeIHDR,6campos ilegales con CRC recalculado,15combinaciones legales bitdepth/color,1CRC dañada,4casos frontera pixels/zero/u32max,1RGBA16 válido y2longitudes de adquisición.
-RGBA16 1×1 decodifica8bytes exactos, canales nativos little-endian [52,18,205,171,0,0,255,255]; prueba ese decode pequeño, no preadmisión del máximo16bit.
-Archivos privados sparse de longitud lógica67108864 y67108865: modo header del primero devuelve header_only, segundo rechaza input_limit/código1. Sólo se leyó cabecera, sin decode/captura de64MiB; ambos archivos privados se eliminaron por sus rutas concretas tras registrar resultado. Esta prueba nueva amplía frontera de adquisición en modo cabecera; no valida snapshot concurrente ni el modo full decode a ese máximo.

## Aislamiento/cancelación independiente

independent-cancel.py deriva sólo la función run del job.py congelado, tras verificar SHA del exe. Lanza4hijos propios con handshake READY, Job sin nombre antes de liberar trabajo, IsProcessInJob(handle,job) exacto y QueryInformationJobObject flags0x2100/ProcessMemoryLimit536870912.

Cuatro trabajadores estaban vivos20ms después del marker seleccionado. Cerrar únicamente el lease Job y esperar recoge decode3,22ms; Sixel encode5,40ms; iTerm2 encode4,26ms; Kitty encode5,49ms. Son observaciones de esta máquina, no SLA. Todos terminaron0 por mecanismo WindowsJob; no equivalen a Ctrl-C130 ni checkpoint cooperativo del codec. Threads Python observados regresan de MainThread a MainThread después de100ms: no quedó lector del arnés en esta comprobación. Cada hijo tiene wait material; no se busca ni mata PID ajeno.

Los markers están inmediatamente antes de llamadas síncronas; prueban actividad y cancelación externa alrededor de fases, no instrucción exacta interna en el instante de cierre. Bootstrap hasta READY precede asignación, como declara autor; no se atribuye contención de ese bootstrap ni límite agregado al producto.

## Conclusiones técnicas contrastadas

La fuente usa Sixel/Iterm2/Kitty/Halfblocks directamente y render en Buffer; no Picker/StatefulImage/ThreadProtocol, probes, tmux ni lectura de teclado. Única lectura stdin del harness es byte del handshake explícito; JSON transmite prefixes como números, no escapes gráficos a un terminal. Es validación de construcción/byte preparation, no prueba de negociación universal o píxeles visibles.

Métrica GlobalAlloc cuenta tamaños solicitados vivos de Rust, no overhead/stacks/RSS ni todos los transitorios internos de System.realloc. Job pico commit es una métrica distinta. Estas limitaciones impiden tratar heap como límite duro; no invalidan el contraejemplo Kitty: resultados congelados registran symbols85604366>33554432 y pico worker342569893>268435456. Revisé esos logs y fórmula rawRGBA→base64, sin repetir encode16M costoso. Son evidencia original del autor, no medición nueva del adversario.

iTerm2 sólido comprime bien, pero un PNG con ruido puede producir base64 por encima del presupuesto. image::Limits.max_alloc no es cuota estricta del pipeline entero; finaldecoded_bytes y within_payload_32m se comprueban después de reservar. Aislar bajo Job no convierte constructores en encoders acotados256MiB ni payload32MiB. El informe reconoce expresamente que falta Writer acotado/preflight y presupuestos de temporales; por tanto no los aprueba como implementación.

## Pendientes conservados

Snapshot mutable/identidad/hash consistente; admisión previa a cada reserva de raster/representación/temporales/IPC/caché y payload; RGBA16 máximo y extremos; detección/géometría/probes/teclas/restauración; proporción/alpha mate/zoom/resize/clipping; lifecycle/IDs/liberación por protocolo; inline scrollback separado de datos TUI con clear_area; redirección/config/integración; límites y ejecución nativa en Windows/macOS/Linux; QA visual de terminales y producto. La prueba nueva64MiB header no elimina ninguno de estos gates completos.

No se encontraron nuevas críticas que invaliden la evidencia/conclusiones parciales del autor. El plan posterior debe resolver sus pendientes antes de producción; este dictamen no sustituye esa revisión ni vacía backlog.
