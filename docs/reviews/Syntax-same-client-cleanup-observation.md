# Defecto observado: reintento de cleanup en el mismo SyntaxClient

2026-10-10. SYN-CLEANUP-SAMECLIENT01, cerrado por el incremento R4; la observación siguiente conserva la historia anterior. Base cbc79b7a34c6baa5b0a2da27800ccb1d484821e8. La integración privada D4b requiere un único cliente para Source y Code; su prueba de Pending físico detecta que retry_cleanup no logra recoger la conexión pendiente desde ese mismo cliente. No hay corrección ni nueva aceptación en este documento.

## Evidencia y causa

AUTHOR-PRODUCER-runtime-attempt3.log, bajo .tmp/d4b-scheduler-execution-2026-10-10, registra seis controles ejecutados con test-threads=1: uno PASS y cinco FAIL. El primero comprueba physicalPending=true, blocked=true, resumed=false; los cuatro siguientes quedan PendingCleanup. Es un fallo primario de recuperación con fallos posteriores contaminados por la retención global, no cinco defectos independientes. Log SHA256: 418C2541988D315A62BC091805CADB278C1D209F59921B2738E138B061F4EDCC.

Lectura causal corroborada por raíz en src/syntax.rs, SHA256 28D0C75D8CA7E85068743CEA29EF2A946F1F0F78577F4ACCC47246064CAD139A: retry_cleanup sólo marca State.retry_cleanup y despierta el controller. En controller_run, Update se toma antes de close_connection; si el cierre falla, el bucle termina. El wrapper controller transfiere Connection al slot PENDING global y termina su hilo. El reintento del cliente anterior no dispone ya de ese consumidor de la señal. La prueba histórica de recuperación crea otro SyntaxClient, por lo que su PASS no acredita el contrato D4b de recuperación con un único cliente.

La retención física y el bloqueo del nuevo spawn funcionan en el escenario fallido; no retirarlos para lograr un resultado verde. No se cambia startup/request deadline, cuotas, expectativas ni controles previos. Los resultados iniciales producer-only que pasaron no sustituyen este caso.

## Criterio de cierre

Corregir el ciclo de vida del controller para que el mismo cliente conserve una ruta real de reintento mientras Pending sigue reteniendo proceso, Job, streams, threads y cargos. Sólo el cierre físico completo permite nuevos Source/Code; no usar otro cliente/thread como rescate. El fallo original permanece irreversible y no se reanuda como éxito un Update antiguo fallido. El descriptor siguiente tiene contexto y petición vigentes.

Congelar el delta exacto antes de revisión independiente de código y AE con worker real. Comprobar Pending persistente, segundo intento Busy/cero spawn, retry explícito del mismo cliente, recogida física, nuevo request y cargos hasta último Drop. La aceptación previa D4a queda limitada a lo efectivamente probado; la modificación de syntax.rs obliga a regresiones de sus rutas afectadas. D4b/sesión/RV y QA visual siguen abiertos.

## Frontera del propietario confirmada en el candidato privado R1

La recuperación focal R1 obtuvo controles positivos con worker real, pero no se adopta: la revisión independiente confirmó SC-R1-01. SyntaxClient::new permite otro Shared mientras hay Pending global. Si ese cliente recoge Connection del propietario original todavía vivo, retry_pending_connection actualiza sólo el Shared llamante; el original conserva cleanup Pending aunque el slot esté vacío. Sus Set/request y reintentos posteriores quedan bloqueados. Es un defecto de asociación con el propietario, sin necesidad de crear un segundo worker.

Identidad del candidato rechazado: syntax.rs SHA256 6F7EDCAA460EE15AA159A31D6741E96EC3EC5FBEACBD34F036F76BA4C72453BB. Suplemento independiente privado SUPPLEMENT-SHARED-OWNER-R1.md SHA256 AB15C533DE40D1E9773ECDB39AF128ACB8E06A6BA0A10D0123E66529B7AF0DD0. Observación previa de raíz GLOBAL-SHARED-OBSERVATION-R1.md SHA256 DC1462509CBB9C236EE4D38A968E1F400416265C16F0335486C84F950A5BC72C. Las críticas y versiones intermedias permanecen fuera de Git; esta sección conserva el defecto y su criterio.

El cierre requiere vincular la recogida al Shared propietario real, o rechazar la recogida ajena mientras aquél sigue vivo. Mantener la recogida persistente después de shutdown del propietario. No inferir Reaped sólo de globalfalse, borrar el fallo sticky ni reproducir el Update viejo. Los oráculos independientes SA14–16 preceden al próximo candidato; nueva revisión y regresiones nativas necesarias. No se demostró un defecto de ConsumerBudget: sus cargos siguieron vivos con los aliases hasta último Drop. No se afirma aceptación del arreglo ni cierre de SYN-CLEANUP-SAMECLIENT01, D4b o RV.

## Cierre del incremento focal

2026-10-10: [R4 integrado y comprobado](Syntax-same-client-cleanup-adoption-r4.md), con [aceptación independiente](Syntax-same-client-cleanup-acceptance-r4.md). SYN-CLEANUP-SAMECLIENT01 y SC-R1-01 cerrados; los rechazos y fallos anteriores se conservan como evidencia histórica. D4b compuesto, RV y QA permanecen abiertos.
