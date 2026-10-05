# S03: cancelación de grafema patológico — evidencia de coordinador

2026-10-05. Hallazgo S03-UNICODE01 pendiente de refinamiento/corrección/revisión. No cambia aceptación ni afirma QA de interfaz. Experimento privado sobre SafeTextModel::new de la biblioteca existente; no es medida end-to-end TUI.

Fuente presentation.rs SHA5F885FFE43E7CC839642E72D6738012CF882C0151EFF5E0282C3C44DA2AC1F5C y rlib target/debug/deps/libr3viewer-0c2cad5a1c7460af.rlib SHA61EDA1991CEA10AA545B3BCB8FAB240544081610BEB2A0ED9183ADC8023F69D7. Harness rustc con toolchain privado1.99 enlaza esa rlib, no cambia producción ni Cargo. Fuente sintética propia 'a'+16,000,000 veces U+0301:32,000,001bytes, un solo grafema. AtomicBool se activa en thread retenido, Instant alrededor de new y del store; ambos threads terminan. Runner subprocess.run(timeout15s) posee/espera cada hijo; nunca archivo académico ni procesos ajenos.

| Delay solicitado | Señal observada | Fin preparación | Fin menos señal |
|---|---|---|---|
|500ms|504ms|512ms|8ms|
|1000ms|1000ms|1066ms|65ms|
|1500ms|1500ms|4988ms|3488ms|
|2000ms|2000ms|4321ms|2320ms|
|2500ms|2500ms|3848ms|1347ms|

Todas retornan Cancelled. Una ejecución por delay, build debug existente, variación observable; no extrapolar SLA ni resultado release. ControlASCII2,000,000bytes cancel250ms→251ms y combinantes2,000,001bytes terminan normalmente187–240ms antes de las señales posteriores. Estos positivos no cubren el gigante.

Inspección: checks de cancel en bucle de grapheme_indices antes/después de llamar al iterador, pero una sola búsqueda de frontera y UnicodeWidthStr::width puede recorrer millones de codepoints sin un check intermedio. También existen usos graphemes/grapheme_indices en viewport/inline/session; no concluir corregido sólo cambiando la primera preparación. unicode-segmentation bloqueada1.13.3 ofrece GraphemeCursor con chunks, NextChunk/PreContext para acotar segmentación sin reinventarUnicode; ancho y render de un cluster enorme requieren solución adicional revisada. Referencia primaria https://docs.rs/unicode-segmentation/1.13.3/unicode_segmentation/struct.GraphemeCursor.html.

Artefactos temporales Cviz/text-grapheme-probe.rs, text-grapheme-probe.py, text-grapheme-probe-results.json, text-grapheme-probe-large.py y text-grapheme-probe-large-results.json. Conservar originales. Agente de especificaciones independiente refina opciones para preservar fuente, cancelación y render acotado; no se ha aprobado un cap nuevo ni truncamiento. Backlog de rendimiento/robustez continúa abierto; esto no crea autorización aparte de encargo completo vigente.
