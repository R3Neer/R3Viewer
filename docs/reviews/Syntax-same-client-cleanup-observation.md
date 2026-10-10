# Defecto observado: reintento de cleanup en el mismo SyntaxClient

2026-10-10. SYN-CLEANUP-SAMECLIENT01, abierto. Base cbc79b7a34c6baa5b0a2da27800ccb1d484821e8. La integración privada D4b requiere un único cliente para Source y Code; su prueba de Pending físico detecta que retry_cleanup no logra recoger la conexión pendiente desde ese mismo cliente. No hay corrección ni nueva aceptación en este documento.

## Evidencia y causa

AUTHOR-PRODUCER-runtime-attempt3.log, bajo .tmp/d4b-scheduler-execution-2026-10-10, registra seis controles ejecutados con test-threads=1: uno PASS y cinco FAIL. El primero comprueba physicalPending=true, blocked=true, resumed=false; los cuatro siguientes quedan PendingCleanup. Es un fallo primario de recuperación con fallos posteriores contaminados por la retención global, no cinco defectos independientes. Log SHA256: 418C2541988D315A62BC091805CADB278C1D209F59921B2738E138B061F4EDCC.

Lectura causal corroborada por raíz en src/syntax.rs, SHA256 28D0C75D8CA7E85068743CEA29EF2A946F1F0F78577F4ACCC47246064CAD139A: retry_cleanup sólo marca State.retry_cleanup y despierta el controller. En controller_run, Update se toma antes de close_connection; si el cierre falla, el bucle termina. El wrapper controller transfiere Connection al slot PENDING global y termina su hilo. El reintento del cliente anterior no dispone ya de ese consumidor de la señal. La prueba histórica de recuperación crea otro SyntaxClient, por lo que su PASS no acredita el contrato D4b de recuperación con un único cliente.

La retención física y el bloqueo del nuevo spawn funcionan en el escenario fallido; no retirarlos para lograr un resultado verde. No se cambia startup/request deadline, cuotas, expectativas ni controles previos. Los resultados iniciales producer-only que pasaron no sustituyen este caso.

## Criterio de cierre

Corregir el ciclo de vida del controller para que el mismo cliente conserve una ruta real de reintento mientras Pending sigue reteniendo proceso, Job, streams, threads y cargos. Sólo el cierre físico completo permite nuevos Source/Code; no usar otro cliente/thread como rescate. El fallo original permanece irreversible y no se reanuda como éxito un Update antiguo fallido. El descriptor siguiente tiene contexto y petición vigentes.

Congelar el delta exacto antes de revisión independiente de código y AE con worker real. Comprobar Pending persistente, segundo intento Busy/cero spawn, retry explícito del mismo cliente, recogida física, nuevo request y cargos hasta último Drop. La aceptación previa D4a queda limitada a lo efectivamente probado; la modificación de syntax.rs obliga a regresiones de sus rutas afectadas. D4b/sesión/RV y QA visual siguen abiertos.
