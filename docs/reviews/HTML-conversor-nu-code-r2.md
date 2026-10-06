# HTML / PluginCommand R2 — revisión de código y regresión de raíz

2026-10-06. Autor graphics_ipc_acceptance; raíz revisa código y ejecuta oráculos propios. SIN_CRITICAS_PENDIENTES para la corrección HNU-R01 y el delta de registro revisado. No acredita DOM, conversor de producto ni RV-404 completo; aceptación funcional separada pendiente.

## Identidad y código

Identidad compuesta parent MANIFEST69EACB781B29635D12FAB4CA80403A2E3A6FBE7E35FF5F4C773C478DF3D6BAA7 (14583 registros previamente verificados por raíz) + DELTA-MANIFEST97F80B62E909B6A02744483C639CF5A9F2288BB678BFB7EEB861C988A6FDAAFD (49 registros, hash/tamaño verificados ahora). Unión de rutas; overlay prevalece, resto parent permanece. No nueva compilación Rust/SDK.

Comparación byteexacta de driver.py confirma exclusivamente añadir receipt['exit']!=0 al predicado de fallo, además del exit bootstrap, errores y wait/closed/joined ya comprobados. Driver6E4BC865A273E09127277A7CF552757EB27B6F354F911F98C71D4B759235F251. Nu exit no cero sigue cleanup del Job/handles/captures y devuelve FAIL; Nu cero no omite los checks de autoridad/cierre. Ningún cambio de límite, wire, PluginCommand, bootstrap, SDK ni semántica sentinel. La revisión R1 de dichas rutas permanece aplicable a este delta; no se extiende a parser HTML/entrada grande.

## Regresión independiente

Copia propia html-nu-root-review-r2 de nueve archivos runtime elegidos, byteexactos antes/después. Dos archivos propios (oráculo y driver) copiados sin cambio desde R1, no del autor: ROOT-NU-EXIT-ORACLE-before-output.json027151F196FF13D269A3FA231DE54CAD69ABAD2F6C32F732854772D8F23F2828 y ROOT-NU-EXIT-PROBE.py8C32041559638723F80DFF7D16EA1BF5F19E19CB169EAFAD42B0A6ECF17B0C74.

Tres controles originales PASS: exit7 y print broken+exit9 ahora driverFAIL, print ok/exit0 driverPASS. Los negativos R1 no se reescriben. Los tres terminan Reaped, wait/closed/joined verdaderos, JobActive0, Pendingfalse, MainThread1, sin rescate externo. Scripts sólo Nu -n, sin perfiles/registro persistente. No se interpreta exit0 del colector como aceptación: raíz inspeccionó los tres verdicts y receipts efectivos.

Resultado propio SHA FE6B32B502CA416CEE72265D1FEFAE19200E20192E9D9DC4EFC6A48EB76B4189, ROOT-POSTVERIFICATION.json confirma nueve archivos intactos. Estos tres controles sólo cierran falsa aceptación HNU-R01, no sustituyen matriz propia open/http/raw/sentinel con error de la otra aceptación independiente.

## Límites

Tiny PluginCommand/sentinel es spike de descubrimiento de comandos nativos: aún faltan DOM ordenado/atributos/origen, presupuesto real, cancelación, parser compartido, integración del companion productivo, matrices Unix/terminales/distribución. Sin cierre RV-404, sin perfiles ni publicación. Job512MiB por proceso no es agregado/RSS/sandbox. Los controles de fallo y Pending originales permanecen historia, no garantía universal de cierre.
