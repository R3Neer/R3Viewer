# AE independiente Sixel R2 final

2026-10-05. **SIN_CRITICAS_PENDIENTES dentro del corte ejecutado.** Autor del candidato distinto del autor de esta aceptación. No aprobación de integración, producto, terminal ni G2 completo.

## Identidad y reproducción

Se verificaron los 782 registros del snapshot `graphics-sixel-gate-r2-final/MANIFEST.json` antes y después de las pruebas, por tamaño y SHA256. Manifest **CF03AC33E0217BB7666D66F081000C88C035CA12F022C383D5286C15DCF04286**, helper **9F01636654301B400A78A2F3C71F76C3D6FE16CE4571ECD447A096C88DC44983**, exe **8DE473305F5998E8876331E00C57E437030484162E96FB51B5C9D2FF337C3D7C**. Lectura/inspección del helper precedió su ejecución. Todas las ejecuciones utilizaron `sixel-ae-r2/candidate-copy/`; el congelado permaneció intacto. Sin compilación ni fuentes de producción.

Receta: Python nativo `C:\Python314\python.exe sixel-ae-r2-run.py`, después `sixel-ae-r2-extra.py`, desde Cviz. Los scripts incluyen ubicación absoluta derivada de su propio directorio, hashes, controles y assertions; no emiten cuerpos/framing a terminal. Evidencia propia en `sixel-ae-r2/`, fuera del snapshot. AE-MANIFEST excluye la copia del candidato, cuya identidad se cotejó con su manifest original, e incluye scripts, entradas sintéticas y resultados propios.

## Oráculos anteriores a outputs

Preparación independiente `GRAFICOS-SIXEL-acceptance-preparation-r1.md` **AE7F1839B07E07580777854E1FBD63FC258C5ED4F544C5ABB039E16FCA1A036F**; EXPECTED **2B73B59B8BDCABBBA384D3FD94FB973523B517128591CC2BCFD71D52550D2A8A**, decoder **D4051A49E080514C82A4C815CF3E7A8BB18650CE18D92CFB7322421177258B0F**, HOSTILES **186C2EF72242572D12CEDF79675C5C9FDAE558CF59649D88A1955EFE16A375FB**. No expected/golden leído desde salidas de autor ni cambiado para aprobar.

La expectativa matemática RGB332 y mate straight, porcentaje RGB y selección nearest del crop se fijó antes del candidato. El decodificador Python propio reconstruye píxeles cuantizados a partir de la gramática del cuerpo; no llama al encoder ni al validador Rust. Palette esperada: 256 registros y porcentajes exactos, sin redefinición. HOSTILES conserva expectativas previamente fijadas y recetas de cuotas. Para bodies enormes se comprueba semántica/cuota sin construir un canvas de 16 millones de píxeles.

## Ejecución propia: 240 invocaciones

- **190 PNG PASS:** 162 dimensiones 1..9 en ambos ejes, estático/Adam7; 24 alpha 0/128/255, mates y sin mate, crop y fit; cuatro umbrales grises y palette completa. 186 `ready_sixel_body` con dest, palette y cada píxel exactos; cuatro `reference` por alpha sin mate. Body medido coincide con longitud+5 y contadores avance/pintura del decoder propio. Body máximo 5933 bytes en este corpus pequeño.
- **20 validator PASS:** 14 hostiles previos (ESC/C1, RLE0/overflow, máscara fuera de última banda, ancho/banda, register inexistente/256, redefinición, HLS, sintaxis/parámetros/dimensiones) más seis fronteras. Cuerpo 32MiB−5 admitido y +1 rechazado; 67,108,864 avances admitidos y +1 rechazado; 16,000,000 pinturas admitidas y +1 rechazazada. Los tres casos exactos son válidos semánticamente; no se equipara rechazo de fichero inválido con prueba de cuota.
- **Tres cancelaciones cooperativas PASS:** CANCEL_CHECK 1/50/200 retorna reference con diagnóstico cancel, cierre y reap. Este contador determinista no acredita latencia durante llamada nativa bloqueada.
- **24 controles negativos de helper PASS:** 13 puntos de fallo setup/Job/readback/write/flush/logging, línea excesiva stdout/stderr, UTF8 inválido, progreso continuo con deadline; cola64, bytes de ambos canales y eventos4096; Thread.start real que arranca y luego lanza en ordinal1/2/3. La expectativa es fallo del helper con recursos recogidos, no ejecución negativa declarada éxito. Cola observada64; canal rechazado dentro de cap+4096; eventos llegan exactamente4096. El plazo único no se renovó por progreso.
- **Tres cancelaciones Job PASS:** PNG sintético propio 1024×1024, fases DECODE/SIXEL_COUNT/SIXEL_EMIT. Child vivo observado antes del CloseJob y reap propio después; sin JSON de resultado. Exit Windows0 se clasifica **external_cancel_no_ready**, nunca éxito. Reap observado 1.42–1.58ms, observación de este host, sin promesa SLA.

Todas las invocaciones confirman wait, cierre del handle Popen, Job, streams y terminación de threads. Job flags0x2100 y límite512MiB leídos antes de bootstrap de codec en cada ejecución normal/Job cancel. **Sin rescate externo para aprobar ningún control.** Cleanup forzado que pertenece al helper en escenarios negativos se registra explícitamente, no se confunde con flujo normal.

Máximo requestedheap reportado en los 190 PNG: 57552 bytes; PeakProcessMemoryUsage del Job máximo1019904 bytes para ese corpus. Son contadores de naturaleza distinta, no medición agregada padre+hijo ni garantía universal de memoria.

## Intento propio fallido preservado

`extra-control-attempt1.json` conserva un error de nuestro generador: escape de newline dentro de exec del stub de cuota produjo SyntaxError, exit1 y missing final JSON; el helper recogió todos los recursos. Se sustituyó ese escape por bytes([10]) sin tocar candidato ni expectativas y se ejecutaron los controles de cuota correctamente. Este intento no cuenta como PASS de cuota ni como defecto del producto. El segundo fallo de assertion idéntico durante instrumentación se conservó mediante el progreso registrado; no se usó para cambiar el oracle.

## Límites

No recompilé los once tests Rust ni repetí los 491 casos del autor: esta evidencia es propia sobre 240 invocaciones y el corpus previo independiente. No acredita PNG exhaustivo, fidelidad fotográfica, framing real, celda/cursor/scroll/inline/restauración visual, palette privada del terminal, agregación padre/worker/cache, snapshot/IPC/Ready/EOF del producto, Unix ni llamado nativo bloqueado. Las fases canceladas son fases del encoder y no PDFium ni un bloqueo native arbitrario. El cuerpo nunca se mostró en una terminal. Los presupuestos son del gate privado; adopción e integración continúan sujetos a sus requisitos específicos.
