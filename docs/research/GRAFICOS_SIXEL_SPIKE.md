# Sixel G2: helper privado R2, entrega para adversario y AE

Candidato **graphics-sixel-gate-r2-final/**. MANIFEST SHA256 **CF03AC33E0217BB7666D66F081000C88C035CA12F022C383D5286C15DCF04286**, 782 registros verificados. Helper actual author-evidence/owned_job.py SHA256 **9F01636654301B400A78A2F3C71F76C3D6FE16CE4571ECD447A096C88DC44983**. Autor del delta: unix_r3_review; dictamen independiente y AE pendientes. No autoaprobación/cierre G2/RV/S03.

R1 original permanece intacta: MANIFEST7F6773E6D316930389A02BAF215C743217EAF2975A7533657FFD2CAFA000CB20,763registros comprobados. Los295 registros protegidos de src/bin/Cargo/licencias/source-inputs y los tres oracles-before principales coinciden exactamente en R2. Bin de ejecución **8DE473305F5998E8876331E00C57E437030484162E96FB51B5C9D2FF337C3D7C**. No cambios en encoder, validador, Rust, bin, fórmulas, geometría, cuotas de producto o expectativas.

## Delta de G2SIXEL01: ownership y cierre parcial

Toda preparación no poseída precede spawn. La sentencia inmediatamente siguiente a Popen exitoso es try/finally. Capture y Thread quedan retenidos antes de start; lector1/2 y escritor se supervisan, incluido start que haya iniciado thread y luego lance excepción. Escritor bootstrap envía START sólo después de asignación y readback del Job512MiB flags0x2100 y membership del handle propio.

Cleanup intenta CloseJob, kill sólo del Popen propio si hace falta, wait, joins y todos los streams aun cuando falle un paso. poll durante cleanup está protegido; su error no puede saltar los demás cierres. Wait exitoso queda en flag separado; si el primer wait falla se intenta una segunda vez, acotada. ClosePopen sólo tras ese wait probado. Errores conservados impiden PASS aunque se hayan cerrado todos los recursos. Auditoría y hook de logging se construyen sólo después de los intentos de cierre. No enumera procesos ni actúa sobre nombres/PIDs ajenos.

Los pipes son bufsize0 y Capture usa read(4096): esto elimina el flush implícito de un stdin buffered pendiente que producía BrokenPipe al close durante un fallo inyectado. Writers/lectores terminan antes de retornar; el guard no se pone en el test externo ni se rescatan hijos para aprobar.

35 controles propios: normal;15faults de hooks ctor/start de readers/escritor, Job create/config/assign/readback, read stdout/stderr, write/flush y logging;8controles de captura/deadline (línea, cola, eventos, bytes, timeout hold y progreso continuo, línea/bytes stderr);9fallos REALES de Thread.__init__/Thread.start ordinal1/2/3, antes o después de start exitoso;2fallos de cleanup poll/primer wait. Todos verifican waited, Jobclosed, ClosePopen, streamsclosed y threadsterminated. Los34 negativos deben elevar fallo; el control es PASS porque el helper falla y recoge recursos, no porque su ejecución negativa se declare verde. Dos controles de cleanup además exigen diagnóstico de cleanup_error conservado. No rescue en test.

## Delta de G2SIXEL02: captura y plazos acotados

Bloque≤4096bytes; línea≤4096bytes antes de append; acumulado≤1MiB por canal antes de aceptar más eventos; Queue(maxsize64), put_nowait y Full fatal; eventos≤4096. UTF8 inválido, lectura/escritura fallida, línea/bytes/cola/eventos overflow invalidan la invocación. No bloqueo en put, ni pérdida de evidencia convertida en PASS. El supervisor recoge la condición fatal y termina Job/hijo propio, wait y lectores. El contador de bytes puede observar hasta un bloque adicional rechazado; no lo almacena en la cola ni retiene todo el canal.

Deadline15s único calculado ANTES de Popen, cubre bootstrap/setup/escritor/progreso/final. Recibir progreso no lo renueva. Cleanup usa wait2s por intento (máximo dos), join2s por thread y conserva inconcluso si no confirma cierre. No promesa de SLA ni límite OS padre+cache. Diagnósticos de excepción limitados a256caracteres, hasta16 errores de cleanup; no construir repr arbitrario completo antes de truncar.

Son techos del helper de este corte. En491invocaciones anteriores ahora reproducidas, máximo observado: línea370bytes, canal3649bytes y178eventos. Cotas4096/1MiB/64/4096 permiten holgura sobre ese corpus y se verifican con controles que las superan. No garantizan que toda combinación extrema de PHASE_EVENTS pase; el overflow de evidencia es rechazo explícito del harness, no reducción silenciosa de la cuota del encoder.

El driver final ejecuta runners in-process, con logs externos acotados y sin Popen exterior parcialmente poseído. Cada encoder conserva su propio owned_job. Los scripts de replay se adaptaron sólo a rutas relativas del snapshot, con bin/ y source-inputs/. El oracle PNG-SIXEL-before permanece intacto: se cotejan expectativas generadas determinísticamente excluyendo únicamente path, sin reescribir el oracle. Se añadieron originales del runner PNG/preflight y su baseline de resultados para reproducción local; son entrada previa, no output esperado inventado desde el candidato.

## Replay del helper final y evidencia

491invocaciones sobre bin exacto bajo este helper final:90pixels+25hostiles+204regresiónPNG+165PNG→Sixel+3cancelJob+1noise16M+2bodybudgetexact/+1+1panorama. Todas retornan con auditoría sin capture/write/cleanup errors, waited/handles/streams/threads confirmados, sin forced fallback normal. FINAL-REPLAY.json fija bin antes/después. Cancelaciones Job no se anuncian como exit130 del producto. No emisión de cuerpos ni framing a terminal.

11testsRust PASS, log RUST-TESTS-R2.log. Cargo exclusivo de copia privada (offline/locked), sin Cargo raíz; sus binaries test rebuild no sustituyen el bin8DE473 de491invocaciones. Rust/source/Cargo/licencias del candidato continúan idénticos a R1. No fue necesario repetir fmt/clippy porque no hubo cambios Rust; su PASS previo no se presenta como nueva ejecución.

INTEGRITY-R2.json SHA256 **CA85D13298A6586D200966F713B5C5122D243972B0F87A20293B9CB2010993B8** enumera comparación protegida, conteos y máximos. HELPER-CONTROLS-R2.json/log y REPLAY-R2.log son evidencia de autor. Reproducir en COPIA propia escribible del snapshot: helper-controls-r2.py y final_replay.py. El snapshot congelado no se usa como destino de nuevas evidencias.

## Intentos conservados y límites

owned_job-attempt1.py y HELPER-CONTROLS-R2-attempt1.json conservan fallo de flush buffered/close con recursos recogidos pero cleanup_error; se corrigió y no se contabiliza ese intento como PASS. Primera prueba desde sandbox restringido produjo warning del intérprete Python en stderr antes de READY; fue FAIL del bootstrap esperado, y se repitió con ejecución autorizada del Python nativo. No se relajó READY para ocultarlo.

Snapshot intermedio graphics-sixel-gate-r2/ MANIFESTF1DB1ED32CD16973726F04825A66BC3C08B0E31400D56544D793494FA5EAC6AD se conserva como intento anterior, NO candidato final: detecté consulta poll no protegida antes de entrega. owned_job-attempt2.py conserva ese código; delta final añade poll protegido y wait-success explícito, con nuevos controles y replay491 posterior. Sólo r2-final/ y su hashCF03... se entregan a revisión/AE.

Sigue abierto corpus exhaustivo/stress, agregado/snapshot/IPC/cache/identity/Ready/EOF del producto, framing real/terminal/celdas/paleta privada/restauración/cursor/scroll/inline/visual y Unix. Este helper no demuestra atomicidad de SAVE_PREFIX ni transportes reales. Sin producción, Cargo raíz, perfiles, terminal, remoto, CI, instalación, nuevo sprint ni modificación de aceptación G2 completa. Integración shell no se tocó.


## Estado posterior de revisión y aceptación

2026-10-05. Revisión independiente del delta del helper y AE independiente del candidato final terminadas sin críticas pendientes en el alcance ejecutado: ../reviews/GRAFICOS-sixel-code-r2.md y ../reviews/GRAFICOS-sixel-aceptacion-r2.md. El texto anterior de pendientes corresponde a la entrega original. R1 y el intermedio rechazado permanecen temporales intactos. Gate privado, sin producción gráfica ni cierre G2/RV/S03.
