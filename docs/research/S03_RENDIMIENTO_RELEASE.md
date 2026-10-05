# R3Viewer C — medición independiente release de tablas Nu

2026-10-05. Objetivo exclusivo de medición, sin suite114, cambios de producción/docs canónicos, GUI ni QA visual. Windows x86_64, Nu0.115.1 sin configuración/perfiles, ConPTY propia110x32, una sesión Nu y una ejecución por tamaño. Fuente commit4a3fa9dfaafdde79197489fb695b0155d103a064. Build release confirmado cerrado por coordinador; copia privada `.tmp/specs-s03/release/package`, sin modificar debug ni target de compilación.

## Identidad

- App release SHA2569B8676324D507DEF2C564E35B8F0A2FD2210DC9DE4EE5366B0D6893874F00BE0.
- Companion release SHA256B06AC11EC15D342ABE4134C3227D9F452818F105F0042E37320FB329C800E3D7.
- Módulo privado nu/r3viewer.nu SHA25652D4CD120EDC6C79950AB3F1774E2D83C14A02DE641FCAF1AD7EECD4A225EBE9.

Nu-n cargó companion por lista bracket con ruta absoluta y módulo del paquete privado. No instalaciones/perfiles. Corpus igual al recorrido A/B histórico:

`0..<N | each {|x| {index: $x, text: "constant", nested: {value: $x}}} | collect | viewer --colour never`

Cronómetro desde enviar comando hasta primera pantalla con index/int(0). Incluye productor Nu, collect, encoder/IPC, lectura/proyección/modelo del app y planificación/transporteConPTY. No son fases separadas ni tiempo exclusivo de dibujado. Tras primera vista se enviaron40Down en viewport fijo hasta observar int(40); q devuelve0 y proceso app recogido.

## Resultado

| Filas | Primera vista total |40movimientos hasta int(40)| App private inicial→tras movimientos | Nu private inicial→tras movimientos |
|---|---:|---:|---:|---:|
|1000|181ms|63ms|2,293,760→2,359,296B|8,380,416→8,380,416B|
|100000|940ms|58ms|71,835,648→71,843,840B|10,362,880→10,362,880B|

Memoria observada mediante GetProcessMemoryInfo solo del Nu27804 y apps propias41240/7012. Baseline Nu antes corpus: working set24,879,104B/private7,176,192B.

| Filas | App working set inicial→movimientos | Nu working set inicial | Nu peak working set acumulado sesión |
|---|---:|---:|---:|
|1000|8,314,880→8,380,416B|26,439,680B|26,472,448B|
|100000|68,882,432→68,911,104B|27,697,152B|80,175,104B|

Nu después q tiene private8,912,896B y11,255,808B respectivamente; estas cifras son de proceso persistente de sesión y no demuestran retención problemática/GC. PeakNu es máximo del proceso desde arranque, no pico aislado de fase, ni medida del companion. Companion separado, hostPython/ConPTY y cachesSO NO están incluidos en Nu/app; no sumar tablas como límite de memoria total del producto.

Ambas apps finalizaron0; remaining_app_children vacío, forced_cleanup false. Memoria private adicional de40movimientos:65,536B para1k y8,192B para100k. Esta observación no demuestra complejidadO(viewport); la prueba instrumentada existente es evidencia distinta.

## Comparación y límites

Histórico debug A/B R4: primera vista112ms para1000filas y27,148ms para100000; appprivate2,334,720B/71,888,896B. Release C actual:181ms/940ms; tamaños de memoria app grandes semejantes. El caso100k cae por debajo de1s en ESTA ejecución completa. El caso1k no mejora aquí, lo que advierte de startup/variabilidad.

No son A/B controlados de un mismo commit/perfil bajo carga idéntica: debug era candidato R4 anterior, ambos binarios app/companion cambiaron a release y los horarios/warmcaches difieren. No atribuir27s a una función, no afirmar speedup universal ni prometer SLA por un único recorrido. Separar release ayuda a decidir si merece profiling antes de optimizar; estos números no justifican por sí solos modificar arquitectura. Repeticiones/fases serían un siguiente objetivo específico si hay una regresión real.

Artefactos: `.tmp/specs-s03/release/tables.py`, `table-evidence.json` (códigos/hashes/PIDs/métricas crudas) y `table-transcript.ansi` completo. Ningún cambio de aceptación S03/RV/matriz por esta medición.