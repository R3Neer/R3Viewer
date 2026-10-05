# YAML R3 - ampliación independiente Y08

Dictamen: PASS en los casos ejecutados; cobertura Y08 parcial. No cierra RV-106 ni acredita todos los máximos de memoria/nodos, deadlines o QA.

Snapshot binario R3: r3viewer.exe SHA256 29EBF22FF82B03FDBF039B46FBD086E766EB3F67BB4B0945E796F0E3C5F6B15A, integrado después en commit 2372224 según coordinador. Esta ejecución usa directamente el snapshot R3, no atribuye al commit una build nueva. No cambios de producción ni perfiles. Evidencia en esta carpeta TEMP.

## Lifecycle del producto

Receta privada: quoted ASCII de 8 MiB exactos, quote + x*(8388608-3) + quote +LF. Cinco casos materiales, con Nu -n y snapshot companion/módulo, usando ConPTY propio y observador descendientes limitado al Nu del arnés. Workers se identificaron por parent PID y comando --structure-worker. IsProcessInJob fue true; ello confirma pertenencia a Job, pero no verifica por sí solo qué Job ni su límite512 MiB.

1. Building observado; Tab cancela y muestra cancelación. Tras 200 ms se observa app propia sin worker. Tab manual vuelve a Building con worker de PID diferente. q termina0, vuelve a Nu en83 ms observados y no quedan app/worker.
2. q durante Building devuelve0, retorno a Nu80 ms, sin descendientes.
3. Ctrl-C durante Building devuelve130, retorno a Nu41 ms, sin descendientes.
4. YAML inválido [unfinished permanece en Fuente con diagnóstico. Cuatro Tab no crean worker nuevo; latch preservado. q con código 0, retorno58 ms, sin descendientes.
5. TerminateProcess intencional sólo a worker propio cuya ancestry/comando fueron revalidados produce fallback Fuente y diagnóstico respuesta estructural no válida (InvalidIpc observado), no Ready parcial. Worker reap después450 ms; q con código 0, retorno333 ms, sin descendientes.

Muestreo cada25 ms: máximo1 worker en todos los casos. Esto no demuestra ausencia absoluta de solapamiento entre muestras. Como --colour never desactiva sintaxis, no se acredita aquí exclusión runtime entre estructura y un syntax worker activo. Inspección del controlador sí muestra espera de ReapTicket syntax antes del spawn.

lifecycle.json conserva los tres primeros casos y un error posterior del arnés: esperaba redacción YAML inválido, pero producto mostró YAML no válido. lifecycle-tail.json repite sólo los dos casos restantes con la etiqueta correcta y comprueba restauración de modos input/output/visible. El intento preliminar necesitó limpieza forzada del Nu propio después del error del arnés; no se atribuye al producto. tail terminó sin cleanup forzado.

Lecturas de memoria de workers en momentos registrados: mayor PeakWorkingSetSize17.9 MB y PrivateUsage9.7 MB antes de la terminación intencional. No son máximos de todo el producto, garantía512 MiB ni medición exhaustiva de vida completa. Tiempos de retorno son observaciones de esta máquina, no SLA.

## Profundidad y fuente

Cuatro invocaciones CLI en depth.json: valor con64 sequences y clave compleja con63 sequences más mapping padre (profundidad total64) Ready/código 0. Versiones con profundidad total65 producen diagnóstico Depth y fuente completa exacta/código 0. Stream/document wrappers excluidos y mapping padre contado.

limits.json conserva la receta y hashes para frontera fuente global:

- 134217728 bytes: quoted ASCII válido, código 0, Timeout estructural; stdout de 134217728 bytes exactos, SHA 42E74C2C65788A6764A4C0A1CD601E59105303113044A2224572E3CAF070722B. Tiempo total14.60s incluye carga/render/fallback; no mide aislado el deadline ni prueba Ready o admisión arena64 MiB.
- 134217729 bytes: cargador rechaza con código 1, stdout0 y diagnóstico entrada excede134217728 bytes/no prefijo incompleto. SHA fuente E2367B41C4E55772CE28C805ECC91BC8DB0B3336097FD135ADDA21F1E611C73F. No es un caso de fallback estructural: falla admisión de fuente global antes.

Se usó un solo par input/output, insertando una X antes del cierre para la segunda receta; stdout a archivo y hashes por chunks64KiB, sin capturar128 MiB en RAM. Los dos archivos grandes fueron eliminados por ruta absoluta verificada bajo Y08, después de conservar recetas/hashes. large-fixture-cleanup.json registra exactamente cuáles; ningún corpus compartido fue borrado.

## Ledger

Ejecución nueva contra fuente congelada R3: cargo test --locked --offline --lib scratch con target temporal compartido confirmado libre. Dos tests PASS (ledger-test.log): live_scratch_prevents_arena_growth_before_allocation y scratch_vector_reservation_rejects_before_capacity_changes. Assertions inspeccionadas: carga scratch hasta64 MiB menos persistent, crecimiento de node/cooked rechaza antes de cambiar capacidad; reserva u32 rechaza con sólo 3 bytes disponibles y admite con 4. Prueban frontera local de admisión y capacidad, no suma real de todo el parser a64 MiB ni ausencia de allocations upstream.

La fórmula YAML inspeccionada cuenta nodes + yaml_edges, claves y valores de mappings, wrappers y alias edges. No ejecuté frontera1M con input masivo: memoria/deadline pueden anteceder y no deben reinterpretarse como validación de cuota 1 M. Arena64 MiB total y items 1 M exactos siguen pendientes de instrumentación material independiente.

## Intento adicional de timeout y limpieza

Se preparó timeout.py con guardas PID+creationtime+comando y comprobación de Job privado mediante handle duplicado; suspensión/rescate estaban previstos sólo tras esas comprobaciones. La ejecución terminó en watchdog20s esperando alt-screen, antes de realizar cualquiera de las llamadas de Job/suspensión. timeout.json sólo contiene ese error pre-TUI. Por tanto no se ha suspendido un worker, no se verificó Job privado512 MiB y no hay evidencia de bloqueo NtQueryObject ni de bug de deadline.

No se repitió el experimento. El arnés cerró su ConPTY con cleanup forzado. La consulta posterior filtrada a timeout.py y apps cuya línea contiene la ruta Y08 encontró0 harnesses y0 apps; post-timeout-owned-processes.json conserva comprobación. No quedó ningún proceso propio que rescatar. El PID chain/creationtime de ese intento no fue persistido antes del error y no se reconstruye retrospectivamente.

## Pendientes explícitos

Deadline TUI2s con fault injection efectivo; exclusión runtime syntax/structure; verificación runtime del límite del Job privado512 MiB; arena persistent+ownscratch64 MiB total y quota nodes+edges 1 M exacta. Tampoco QA visual de WT/WezTerm, runtime Unix ni cierre de producto. No se atribuyen los tests/code inspection a esos gates pendientes.
