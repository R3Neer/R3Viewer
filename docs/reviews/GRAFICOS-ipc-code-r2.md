# GIPC R2 — revisión de código y regresión nativa de raíz

2026-10-05. Dictamen: **SIN_CRITICAS_PENDIENTES en el candidato privado Windows revisado**. GIPC-ROOT01 queda corregido en este experimento. No es adopción en producción ni aceptación de G2/RV; R1 conserva su rechazo y evidencia negativa.

## Identidad y lectura

Freeze MANIFEST F93A15AF6D9AFCD73459BAA8E74081EFB46F7536A02A5B531B4DFF246F44DAB8:383 registros verificados en original, espejo privado del repo y copia AE. AE-EVIDENCE-MANIFEST EF317F83EC5EFD71A3E27EFF8609B55DF4F34CA9A88692928DA56EB34967D756:63 registros verificados. Receipt propio GRAFICOS-IPC-root-verification-r2.json conserva comprobaciones.

Se contrastaron windows/managed/hooks y H/C, junto con la lectura anterior del resto de R1. Comparación propia: sólo esos cinco archivos Rust cambian; wire/codec/ledger/Sixel/D y Cargo.lock byteidénticos. Esto prueba identidad del delta, no valida por sí solo su conducta.

Owned conserva raw tras error de CloseHandle y lo invalida sólo tras éxito; el fallo permanece irreversible. Child.close intenta todos los streams aún propios además de proceso/hilo y exige reap. IoOwner se registra en Authority antes de lanzar Task: error/panic no descarta el último propietario. Join, receipt limpio y cierre son estados distintos. Recuperar mutex envenenado permite cierre, pero no convierte la operación en limpia. Try_close no espera un worker activo.

H/C retienen propietarios mientras cleanup sigue Pending y no admiten siguiente actor. Los errores de cierre no se descartan en la ruta de publicación. El duplicado de pull se guarda inmediatamente antes de validar; un candidato rechazado sólo se cierra, sin obligación de esperar/matar como D. Tras validar, la obligación de wait persiste aunque falle el polling posterior. La ruta normal exige IO limpios, joins/receipts, salidas, Job vacío y handles cerrados antes de publicar. No encontré otra crítica accionable demostrada en el corte leído.

## Regresión independiente del defecto R1

Copia propia graphics-ipc-root-review-r2; se añade exclusivamente módulo de tests al H original. Prefijo de funciones original byteidéntico, comprobado después. Oracle previo ROOT-PREOUTPUT-PROBE.json D91290F3049C51E3432885D5CDF0E8CD78D2131F486A20A04D822027E713D298; H propio 7F642F3E7A9361E074F1F963111E709C60BA62F6C2012B635862A90B693CCFC1.

Cargo test --locked --offline --bin gipc_h root_own_probes -- --nocapture --test-threads=1: exit0,1 test propio PASS,2 tests del autor filtrados. C propio suspendido y Job propios: cleanup=true y child_reaped=true. GetHandleInformation sobre los tres valores HANDLE originales, guardados antes del cierre, acredita stdin/stdout/stderr inválidos antes del teardown del test. No se mide raw=null como sustituto de la consulta nativa. Log ROOT-SETUP-CLEANUP-attempt1.log C3049F2C5495F62301F7FF4BF2D165D626BD01C24B0C9E41DF765F7F03B00C9D. No rescate externo. R1 había fallado este escenario con los tres handles vivos; ese resultado permanece intacto.

## Aceptación independiente y límites

AE separada: GRAFICOS-IPC-independent-acceptance-r2.md F3709E6A303A3D49BED8FD1BE8073D895DCCC2AB656C01A4DFEF5C83935EA903, dictamen sin críticas pendientes para su alcance.149 casos de binarios exactos:18 pixels propios,119 controles anteriores y12 cierres. Los seis persistentes siguen FailedPending sin publicación ni segundo actor; terminación posterior por watchdog exterior no cuenta como cierre/reap interno. Cuatro grupos nativos propios adicionales comprueban transferencias parciales, fallos simultáneos, duplicado rechazado y panic C con ownership retenido. Su fallo inicial del helper wait50ms se preserva y la repetición usa teardown explícito wait2s; no se atribuye retrospectivamente PASS al intento fallido.

Controles de close producen error antes de llamar al kernel, no fallos naturales de CloseHandle. Requested H40+C80+D136=256MiB, Job512MiB por proceso, RGBA24MiB privado y source64MiB privado no certifican todas las cotas del producto, RSS ni margen nativo. El máximo pequeño medido no es stress máximo. No terminal/framing/visual/inline/cache/coalescing/SLA/Unix, integración Markdown/PDF ni distribución aceptados mediante este gate. La integración posterior necesita revisión del conjunto.
