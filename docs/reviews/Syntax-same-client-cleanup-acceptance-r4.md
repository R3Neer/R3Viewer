# Dictamen independiente SYN-CLEANUP-SAMECLIENT01 R4

2026-10-10. **SIN_CRITICAS_PENDIENTES**, limitado al fix separable Syntax exacto y regresiones afectadas. SC-R1-01 queda cerrado por CodeAE y AE nativa propia. El bloqueo Clippy del candidato R3 queda cerrado en R4; no se acepta R3 como fuente de integración ni se borra su FAIL. No es aceptación D4b, activation/session/pump, RV, renderer, Nu, HTML/PDF ni QA Windows/Unix.

## Identidad y preparación

Freeze .tmp/d4b-scheduler-execution-2026-10-10/syntax-same-client-fix-r4: MANIFEST F561E123364F3B4230E6644279E09E84F76C160D59BF6ADCB85788838238A689; syntax.rs 8538D33389D38D3C2137AC861D2B680AB43FDABF6BD8F8E918F1ADD6AE1DDA9C. Verificación propia de las **25 entries**, bytes y SHA, antes lectura/runtime. La comunicación previa “24” era error de recuento del mensaje, corregido aquí; no entry excluida de la iteración de validación.

Criterios SC01–12 y oráculos SA01–13 anteriores intactos: CRITERIOS-PRE-CANDIDATO.md 4742C744D69835ACFA907B492E31C9E7E4851FBB424DA17D0E9CE7C57CF199CF; ORACULOS-PRE-CANDIDATO.md E8450AADC113885257D09C24003DB01C21E68F73315E45E0802A193D7A5AA80E. Suplemento SA14–16 AB15C533DE40D1E9773ECDB39AF128ACB8E06A6BA0A10D0123E66529B7AF0DD0 fijado antes R2. No expectativas derivadas de outputs autor.

Copia product-r4 desde BASE-CANONICAL.zip propio, sustitución exclusivamente fuente congelada y módulo congelado de tests. Prefix de syntax.rs propio cotejado byte por byte con los 98736 bytes del freeze; sufijos exclusivamente cfg(test). Ningún freeze/canónico editado. Build worker y ejecutores propios nuevos, no binarios históricos R3 del manifiesto como AE R4:
- OWN-WORKER-R4.exe A09BFF9793605CA9FFD03A50F727AE190A90B01E3BB32C2C4ED775C05D07FD14.
- OWN-EXECUTOR-R4-final.exe 171E50755165520BA1D9C1CA3534C5A80F78372C8B9928E5F089E4266187A003; harness SA14–16 FE4B73B27BE174C01CDE7B11D0A29BAE33F093BB74CBC7283442B8676A2EED29.
- OWN-EXECUTOR-R4-ORIGINAL.exe 4716EF07F97DCF0FAA8890F72F73091C674D204CFC288E2FBD72B9360AE2E163; sufijos originales R1 7C5946B552BC2295C8C05DE27819CF7257ECE4CA247B2E0A380DC7EA2DE6CFC8 y WaitFailure 87A119919CE00F4F7B3D807ED6FFF2223624DF5C084606AD7738548DB2F4F3F5, reutilizados sin cambio.
Inventarios PRE-RUNTIME-R4-attempt3.json y PRE-RUNTIME-R4-ORIGINAL.json escritos antes respectivas ejecuciones.

## CodeAE

Revisado delta completo canónico→R4 y R1→R2, más identidad incremental R2→R3 selector y R3→R4. R4 añade solamente cfg(test) a current_generation respecto de R3; código físico de cleanup sigue exacto R2. Getter productivo del scheduler compuesto D4b queda fuera de este candidato.

Connection enlaza owner_shared Arc real (889); reserva metadata antes clone/spawn (840+), incluyendo Connection272, Shared544, headerArc16 y Outgoing/4096: necesidad compilada5040. Captura adicional del hilo IPC Shared existe en954/962 y ya es parte del camino de IO; dos clones físicos mientras IO vive, no autoridad numérica ni nuevo reaper. Shared no contiene Connection: enlace no crea ciclo Arc.

retry_pending_connection(1710+) rechaza Shared extranjero si original vivo ANTES reset de cleanupdeadline/cierre. Retry original toma mismos objetos; orphan sólo autorizado por stopping del Shared físico original. Success elimina owner tras close físico y publica cleanup/memory exclusivamente en original; foreign no recibe estado/cargos ajenos. Store globalfalse antecede lock/publicación original, ventana transitoria segura Pending: AE espera conjunción, no confunde flag con proof.

Wrapper conserva mismo thread original aparcado, Update/request fallido no se reanuda; gates Set/request protegen statePending aunque desaparezca flagglobal transitoriamente. Sticky se conserva hasta nueva operación auténtica; successful cleanup posterior de nuevo documento puede legítimamente devolver failedfalse. SyntaxMemory Default es observación del owner recogido, no descuento ConsumerBudget: alias de roles/input mantiene cargo real.

No wire/gramática/roles/deadlines/límites aumentados. Cambios a antigua regresión cancelación ajustan Set Legacy ahora rechazado durante Pending y separan terminal shutdown/orphan backstop de recuperación del cliente vivo.

Clippy **fuente exacta, antes sufijos**, --lib --offline -- -D warnings PASS; CLIPPY-EXACT-R4-LIB.log SHA424E950A08CBE7DFC1290C0BBE70A7485D8E106422B7F6CFBF07A667C33E17B0. Build de tests informa helper cfg(test) own_close_pipe_fault unused, advertencia de test exclusiva; no supresión global, no warning productivo y no obligación aquí de Clippy compuesto/all-targets D4b.

## Matriz criterios y evidencias

| Criterio | CodeAE y ejecución propia R4 |
|---|---|
| SC01 identidad / SA01–02 | Original SAMECLIENT: Legacy A14B → Document C26B → nueva PowerShell15B, ThreadId/Shared idénticos, real Styles/cobertura completa, reap y fresh. |
| SC02 Pending / SA01,08,12 | Mismo PID/input, Job/status wait/pipes/sticky físicos; roles/input vivos, metadatos hasta close y último Drop; código mantiene mailbox/IO/chain. |
| SC03 admisión / SA09 | Original: retainedSet typed ContainmentFailed, Legacy Set y requests bloqueados, generation/ledger/source preservados. SA14 añade foreign retry sin mutar PID/deadline/cargos/owner. |
| SC04 retry / SA02–03 | Original: fault activo + retry adicional continúa mismo PID/input; luego faultoff retry explícito recoge. SA14 original recupera con mismo controller, SA15 orphan separado. No se afirma haber ejecutado dos retries fallidos adicionales como secuencia distinta. |
| SC05 sticky / SA04–05 | Original no oldUpdate/request replay, request antiguo denied; WaitFailure RequestTimeout sticky y fresh legítimo. Update tomado antes close revisado en código, no escenario nuevo separado. |
| SC06 contexto / SA06 | Original ctx701/702/703, generaciones2/3/4 y nuevos requests; SA14 ctx902→903 fresh en mismoA. Regresión native activecancel prueba cancel/fault/retención. Stale/context preservado en tests puros. |
| SC07 deadlines / SA07 | Defaults intactos por diff; foreign retry no cambia cleanup_deadline real. WaitFailure usa viewport_total ZERO como reducción prospectiva determinista, no amplía ningún techo. |
| SC08 cuotas / SA08,16 | SA16 need5040 exact acepta/−1 SourceBudget antes spawn/clone, ledger y aliases reales; Weak Shared/input/roles y último Drop baseline. |
| SC09 señales/UI | CodeAE: cleanup en controller; caller retry sólo señal, locks largos no se sostienen sobre Shared del caller. AE samecontroller vivo y foreignretry procesado; no medición UI/QA. |
| SC10 shutdown / SA10,15 | Original shutdown/join inactive; SA15 terminalA mantiene Pending physical y reaperB actualiza original exclusivamente; aliases siguen cargados. |
| SC11 regresiones / SA11–12 | R4 original SourceLegacy/retainedDocument/fresh + WaitFailure; 14tests Syntax normales PASS, 8ignored; activecancel nativo1PASS. Setup/lateIO no modificados revisados CodeAE; no rerun matriz completa Job/kernel. |
| SC12 evidencia / SA13 | Hashes propios previos, tests seriales, globalsinactive y ledgerbaseline al terminar éxitos, FAIL/errores preservados, no test0 como AE. |
| SA14 extranjero vivo | own b_live_foreign_and_orphan orphan=false: B idle sin segundo worker; ownerA vivo, PID/deadline/Shared/input/cargos intactos, mismoA retry→fresh. |
| SA15 orphan | mismo test orphan=true: shutdownA termina controller, original stopping, objetos retenidos hasta reapB, state/memoryB intactos. |
| SA16 último Drop | own a_metadata_exact_minus_alias: dos casos −1 y exact; clone IO+owner y Weak Shared, cargo recuperado tras close/Drop; más alias RetainedRoleSet hasta último Drop. |

## Resultados y fallos conservados

AE-OWN-R4-attempt3.log: **2 tests PASS**, escenarios combinados SA14/15 y dos cuotas SA16; SHAC781A05576FA1867865B6C6A2160483F9A3A293AC0939065FCCF138DF0AC1183.
AE-ORIGINAL-SAMECLIENT-R4.log **1PASS**, SHA F4E706432EA982A9784BC7701865249C707C84FCEEFE7F9862A8A8A34E85BDC2.
AE-ORIGINAL-WAITFAIL-R4.log **1PASS**, SHA F60701A907725741334CFC93483D7DFF5E9D549F4BE3E1A271FA686E2E45EC50.
REGRESSION-SYNTAX-R4.log **14PASS/8ignored**, SHA76DAFD89D72A1FAC4170442CEEC779BD37A29519D6474E3ADA8D1287080083EF.
REGRESSION-ACTIVECANCEL-R4-attempt2.log **1PASS**, SHA CD7726AAAB3653232027CA036722209E1AD4C80FFEA6F8B49027D0999F6894B9. Primer selector equivocado REGRESSION-ACTIVECANCEL-R4.log ejecutó0tests: NO aceptación.

R4 BUILD-OWN-R4.log E0599 del harness (variant inventada NotStarted) conservado; se sustituyó por snapshot del estado real, no producto.
AE-OWN-R4-attempt1.log **2FAIL**, SHA6A47958AA4CEB6DF53E83154F82AC3E7603471F3AEB2B8E3DE113149987C7EC2: expectation refcount+1 omitía cloneIO; otra assertion failedtrue se hacía tras freshshutdown correcto, no justo tras viejo reap. No PASS retrospectivo.
AE-OWN-R4-attempt2.log **1PASS/1FAIL**, SHAB434178C1DFCA30D2770B2EB9013F0244E48DC769A14CCF052AF8A7D146234F5: cleanup_status extranjero inicial proyectaba globalPending, después reap proyectaba localReapedfalse. Oracle exige SharedB local intacto: comparación final corregida a state.cleanup, sin cambiar producto/oráculo. Espera globalfalse+originalReaped conserva mismo plazo6s; evita ventana de publicación legítima.
Harness snapshots y ejecutores anteriores preservados. Antes nueva ejecución los PIDs47248/3316/20724 del primer FAIL no existían; cada nuevo escenario arranca Pendingfalse y termina inactive; no nuevo cliente usado como rescate/PASS de A vivo. Primer proceso fallido no equivale a proof Rustreap aceptado.

Legacy cold StartupTimeout del composite R1 sigue **FAIL** íntegro: CreateProcess2.1063s >startup2s. R4 positivo usa ejecuciones posteriores en host ya utilizado; no demuestra garantía de arranque frío. No ampliación deadline, retries aleatorios ni relabel del fallo. No QA visual/interactiva, plataforma Unix, failure matrix OS completa ni D4b compuesto.

La adopción autorizada puede usar únicamente fuente R4 exacta8538D333… y selector documentado. Historia R1 rechazo SC-R1-01, R3 ClippyFAIL y errores propios queda preservada junto a este cierre.