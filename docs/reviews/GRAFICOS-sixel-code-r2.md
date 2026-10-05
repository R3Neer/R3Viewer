# Sixel G2 R2 final — revisión independiente del delta

2026-10-05. Revisor root, autor delta unix_r3_review. Candidato exclusivo graphics-sixel-gate-r2-final, manifest CF03AC33E0217BB7666D66F081000C88C035CA12F022C383D5286C15DCF04286,782 entradas. Helper9F01636654301B400A78A2F3C71F76C3D6FE16CE4571ECD447A096C88DC44983. No editar candidato ni adoptar código de producto.

**SIN_CRITICAS_PENDIENTES** en el delta del harness. G2SIXEL01 y02 de R1 resueltos. No es aceptación del conjunto gráfico ni nueva certificación del encoder: su revisión previa acotada R1 conserva alcance; AE R2 independiente todavía pendiente.

## Inspección

Popen entra inmediatamente en try/finally. Capture/Thread retenidos antes de start, incluidos fallos después del inicio real. Job configuration/asignación/readback/membership anteriores a bootstrapX. Cleanup intenta cierres aunque falle otro paso, poll protegido y wait_success necesario para cerrar handlePopen. Logging posterior a cleanup; errores conservados invalidan invocación. Captura de bloques4096/line4096/channel1MiB/Queue64/event4096 y deadline15s desde antes de spawn, sin renovarlo por progreso. QueueFull/UTF8/overflow son fallo explícito, no descarte seguido de PASS. Memoria temporal de split/decoding acotada por bloque/línea. Estos son límites del harness, no de memoria total del producto ni SLA.

## Ejecución propia

sixel-root-review-r2.py ejecutó diez controles propios, no sólo los35 del autor: normal positivo; fallo reader2 constructor/start, writerstart, asignaciónJob y logging; salida UTF8 inválida; línea stdout4097 y stderr4097; Thread.start real del segundo lector que inicia el thread y después lanza excepción. Todos los nueve negativos fallan y muestran waited/handleclosed/streamsclosed/threadsterminated/Jobclosed; ninguno necesita rescate desde el test. Normal devuelveJSON esperado yexit0. No se ejecuta archivo de usuario, terminal ni PID ajeno. Resultados sixel-root-review-r2-results.json SHA846CFECA201A8558A7201E918C484B336EAA9D8962E3CD0ECBCE11D8599D63F4.

Antes/después se verificaron las782 entradas. Verificación adicional completa de R1 763 y R2 782 y comparación propia de294 rutas protegidas src/bin/source-inputs/licenses/Cargo/oracles-before: bytes iguales. El filtro propio es explícito en sixel-root-integrity-r2.py; no se presenta como el conteo295 distinto del autor ni como diferencia encontrada. Fuente/encoder/validator y bin8DE473305F5998E8876331E00C57E437030484162E96FB51B5C9D2FF337C3D7C permanecen intactos.

Intento propio inicial conservado sixel-root-review-r2-attempt1.py: esperaba campo entries/files y manifest usa records. Falló antes de spawn. Se corrigió el lector del test sin modificar candidato ni expectativas. Una lectura de INTEGRITY-R2 en raíz falló por ruta abreviada; archivo real author-evidence/INTEGRITY-R2.json inspeccionado. Ninguno es fallo de producto.

No reejecuté las491 invocaciones ni11testsRust del autor en esta revisión; AE prepara contraste independiente de encoder/validator/cancel y captura. No afirmar estrés universal, aggregate256MiB, snapshot, IPC, cache, Ready/EOF real, framing, terminal/paleta/cursor/scrollback, inline, visual o Unix. G2/S03/RV permanecen abiertos.
