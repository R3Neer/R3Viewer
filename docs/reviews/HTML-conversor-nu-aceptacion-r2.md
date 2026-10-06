# HTML Nu PluginCommand R2 — aceptación independiente

2026-10-06 Europe/Madrid. Autor graphics_ipc_acceptance; AE unix_r3_review; root revisión de código separada. **SIN_CRITICAS_PENDIENTES en esta aceptación de registro pequeño y rechazo de Nu exit!=0.** No parser/DOM, producto, adopción ni cierre RV404 acreditados.

## Identidad compuesta

Parent `html-nu-plugin-spike-r1/MANIFEST.json` SHA `69EACB781B29635D12FAB4CA80403A2E3A6FBE7E35FF5F4C773C478DF3D6BAA7`,14583registros: hash revalidado; verificación completa previa realizada por root, no repetida ni atribuida a esta AE. Overlay `html-nu-plugin-spike-r2-delta/DELTA-MANIFEST.json` SHA `97F80B62E909B6A02744483C639CF5A9F2288BB678BFB7EEB861C988A6FDAAFD`,49registros: todos hashes/tamaños verificados antes de copiar y después de runtime, cero discrepancias. Unión con precedencia del delta según contrato; no identidad de candidato standalone inventada.

Copia seleccionada propia `html-nu-independent-r2`: driver/common/bootstrap, bin Nu/plugin y fixtures sample.html/.htm, siete archivos byte-exactos del overlay, iguales después. Driver SHA `6E4BC865A273E09127277A7CF552757EB27B6F354F911F98C71D4B759235F251`; plugin `6EEB46D7DA61D6F89F1D6D0F38A1C7CEA1918102BDF485C2A61D2F5BCBF66AC1`; Nu `90E392149504EF8D38D827D88EFB7AE6B7AED26F7ECC189494B445154AE40507`. Sin reconstruir Rust/SDK, modificar Git/Cargo/freeze/perfiles, resolver fork ni registrar plugin permanentemente.

Leídos contrato canónico HTML-CONVERSOR-NU-R2, delivery, driver/common/bootstrap antes de ejecutar. Contraste propio driverR1→R2 demuestra único predicado nuevo `receipt['exit']!=0`; A.exit es C, receipt.exit es Nu. Se ejecuta driver final byteexacto, no driver reescrito para conseguir aceptación.

## Oráculos previos y ejecución

`AE-EXPECTED-before-output.json` SHA `6FF3538E35A66350A494E625DDB60CC7484515A856A5DDF389EF42AAA6729A10`, fijado antes de salidas propias. Fuente exacta25B `<p>Hello &amp; world</p>` +LF, SHA `BE9BC09261E5D258F96A4DA4955B291B27844FD08AA4A6547DE0F5B656513E2A`; sin trim. JSON esperado sentinel HTML_PLUGIN_SPIKE/source exacta o string fuente según ruta. Nu nuevo `-n`, plugin sólo `--plugins` temporal. Server propio127.0.0.1/puerto efímero, sólo fuente fija, cerrado/joined tras cada caso. Cada ejecución usa bootstrap/Job/captures/plazos originales: Job antes START, readback/membership, wait/handles/streams/readers finales; no búsqueda de procesos ajenos.

| Casos propios | Expectativa | Resultado intento2 |
|---|---|---|
| open .html / .htm | sentinel +fuente exacta, Nu0 | 2PASS |
| http text/html / charset UTF-8 | sentinel +fuente exacta, Nu0 | 2PASS |
| direct from html / from htm | sentinel +fuente exacta, Nu0 | 2PASS |
| raw open / raw http | string fuente intacta | 2PASS |
| ausencia plugin open / http | string fuente intacta | 2PASS |
| sentinel válido seguido Nu exit7 | fuente exacta, driver rechaza | PASS control negativo |
| sentinel válido seguido Nu exit0 | fuente exacta, driver aprueba | PASS control positivo |
| writer-fail / reader-late | driver FAIL, Reaped/Active0 | 2PASS controles negativos |
| PersistentClose | driver FAIL Pending, no siguiente spawn | PASS control negativo |

Total **15/15 expectativas propias satisfechas**,12casos+3controles; no 15success del driver. El caso Nu7 emite JSON válido con source exacta y LF, C exit0, receiptNu.exit7; driver PASSfalse, errorsticky C failed receipt, Reaped, Active0 y IO cerrado/joined. Queda cerrado el falso éxito detectado en R1 sin aprobar por el contenido legible.

PersistentClose retiene la misma autoridadJob y estadoPending; intento siguiente falla Pending: no next spawn y contador propioPopen=0. Después de preservar ese FAIL se hace cierre explícito del único Job retenido por este mismo driver; no se cambia el resultado a PASS. El control no es fallo natural de Win32 ni aprobación de rescate como éxito. Final Pendingfalse, serverPendingfalse, exactamenteMainThread1. Todos los otros resultados tienen Reaped/wait/closed/joined y JobActive0; sockets propios cerrados y hilos server joined. No procesos/checks propios vivos.

## Fallos conservados y reproducción

Primer intento con sandbox normal: exit1,11expectativas satisfechas/4HTTPFAIL. HTTP plain/charset/raw/ausencia-plugin devuelven Nu1 PermissionDenied, stdout vacío; los enabled registran además diagnóstico local-socket del plugin. Todos cleanup Reaped/Active0 y serverclosed/joined. No se reemplaza ni reetiqueta ese resultado como PASS.

Se repitió exactamente matriz/oráculos con require_escalated, autorizado para loopback/procesos privados; sólo nombres de archivos output cambiaron para preservar attempt1. Segundo intento exit0/15PASS. La combinación PermissionDenied también sin plugin y éxito elevado es consistente con restricción de entorno; no se imputa un bug del registro ni se demuestra exhaustivamente causa del IPC secundario. No ajuste de expectativas entre intentos.

Evidencia bajo `html-nu-independent-r2`: AE-run-attempt1.py/AE-run.py, AE-RUN-attempt1.log/-attempt2.log, AE-RESULTS-attempt1.json/-attempt2.json, AE-FINAL-attempt1.json/-attempt2.json, AE-IDENTITY-before-output.json y AE-DRIVER-DELTA.patch. Result2 SHA `29DB9A1E491D3A5EC1FFC38ABE4E8482825C895C449297827AFD851BD3CC12B2`; receipt `AE-RECEIPT-r2.json` SHA `00D43EE3D83B592C93636DBBE36BD3E648FD26869718442BB5D48A0C21E350E9`, hashes/tamaños de scripts, expectativas, intentos y cierre. Receta Python3.14 -Xutf8 AE-run.py en copia, PYTHONDONTWRITEBYTECODE1, sin matrices autor ejecutadas ni corpus generado desde sus resultados.

## Límite de conclusión

Demostrada vía PluginCommand opcional de Nu0.115.1 para native open/http y direct from html/from htm con fixture pequeña sintética, raw y ausencia intactos; driver no acepta Nu fallido. R1 FAIL histórico/oráculos/reportes permanecen intactos. Root revisión/corpus propios son otra evidencia, no se atribuyen a esta AE.

No se acepta parsing HTML, DOM/tree completo, provenance/encoding/base/recursos,128MiB, input inválido, into_value acotado, callbacks/cancel/flow-control de encoder, coexistencia de terceros/versionmatrix, paquete/adopción ni renderer. Job512MiB porproceso no demuestra límite agregado/RSS ni sandbox filesystem/network. No Unix/visual/RV404closure ni edición/publicación realizadas.
