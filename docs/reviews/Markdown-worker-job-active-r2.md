# ChildJob active_processes/terminate — aceptación independiente R2

**SIN_CRITICAS_PENDIENTES para este delta auxiliar.** No aceptación de document_controller ni de descendants arbitrarios.

MANIFEST77420FF92D2D01EB8D25B9177E641DD96C778107FD7531013E13648E4039E6E1, diez registros verificados antes/después y copia exacta. PRODUCT-worker_process.rsB8796B852CE7DE4999F9674697F8BEF56B772FF2EC6CFCBFC1209CE938B40593. Paquete propio usa únicamente PRODUCT como producción, no source con tests root. Módulo propio agregado después del prefijo íntegro original; oráculos previos y fuente/hashes propios conservados. Bloque Windows close_checked/close_with/close exacto byteporbyte al R1 aceptado: no repetición innecesaria de la cobertura anterior.

Revisión de código: terminate llama TerminateJobObject sobre su handle asociado, conserva &self y captura last_os_error inmediatamente si BOOL0; Ok no implica reap. active_processes consulta JobObjectBasicAccountingInformation hacia struct zeroed de tamaño exacto, comprueba BOOL y bytecount antes leer ActiveProcesses; devuelve io::Error si query/length incorrectos. No reduce ausencia de prueba a0, ni consume propiedad del Job. NonWindows mantiene Unsupported para ambas APIs con &self intacto; revisión estática, no ejecución Unix.

Dos pruebas propias nativas Windows pasan, anteriores a outputs:

1. Dos procesos propios del mismo testexe, cada uno esperando EOF en stdin tras publicar ready en archivo propio. Child asociado por attach_verified y segundo asociado explícitamente al mismo Job, verify_attached para ambos. ActiveProcesses2; EOF de primero y wait exit0→Active1; EOF de segundo y wait exit0→Active0; close_checked éxito.
2. Dos procesos propios gated, aún sin esperar EOF, mismo Job verificado Active2. terminate(137) retorna éxito, ambos wait exit137, luego query Active0 y close_checked éxito. Demuestra efecto del Job sobre dos procesos asociados distintos; no equivale a child.kill ni acredita un árbol de descendants/grandchildren.

Harness controla ownhandles/PIDs únicamente, no enumeración global. Propiedad Child establecida inmediatamente tras spawn; waits con deadline3s, capturas child≤4096 cada stream, ready limitado2s. Guard de fallo registra OWN_TEST_FAIL_TEARDOWN y mata sólo proceso propio si necesario: no ocurrió en los PASS. Runner externo H/C acotado también usa Job antesSTART/readback, exits0, wait/close/join, Active0 y Reaped; Pendingfalse/thread1 después. Ningún rescate externo ni timeout usado como PASS. El código de fixture no ejecuta parser ni accede a archivos externos o red; sólo archivo ready propio y stdin gate. No se atribuye aislamiento de permisos al proceso; no cambio de perfiles/registro permanente/sharedrepo.

Evidencias: OWN-ORACLE-before-output.json, OWN-PRECHECK.json, OWN-BIN-before-output.json, OWN-BUILD-attempt1.log, own_active_two_natural_then_zero-RESULT.json, own_terminate_two_associated_processes-RESULT.json, OWN-RESULTS.json, OWN-POSTCHECK.json y cuatro ready files. Registro nu_pid identifica al testexe, no al producto Nushell real. No se reclama error natural Query/Terminate probado por inyección ni aceptación de receipts/lifecycle completo/controller.
