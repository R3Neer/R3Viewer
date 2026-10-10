Copia canónica del dictamen independiente final sobre el [diseño ResponseLease R3](../refinement/HTML-RESPONSE-LEASE.md). Se conserva su alcance exclusivamente documental. Original privado SHA256 0F8A4F1A820F8EED83E7C83E91F570EF5A1E07F7AE15A38A951E04E577E26029. [Adopción y gate siguiente](HTML-response-lease-adoption-r3.md).

# Revisión focal independiente ResponseLease R3

2026-10-10. SIN_CRITICAS_PENDIENTES sobre el DISEÑO EXACTO R3. RL-R1-01 y RL-R2-02 resueltas documentalmente; candidatos/dictámenes rechazados conservados. Ningún código, compile, runtime SDK/parser ni spike Nu ejecutado. No aprobación del conversor, serializer productivo, QA o cierre RV401–404.

## Identidad e independencia

Hashes físicos verificados ANTES leer R3: propuestaE757E265406A5B13A8699063ED07540E4D935635C429673F7E1AE75CD8B6AE4E; oráculosF3C3B07D5FE22B47BD41397E5345620AFCCFD93792962D6B8E244C7DF193EDE7; manifest9B0071C814E2A4DE0A20A093698EF733E635AC739FF680DF2EABDF6C8E7FFDF3; seal55BB5CD669A5DA7B3AA8E98C3AFDBF2725B6F09F2EF27F42D84C4D4413315867. Cuatro artefactos manifest, nueve fuentes y once originales R1/R2 pasan bytes/hash físicos:24 controles en IDENTITY-R3.json. Además criterios propios D397BA5E54D638FE46DFB56624E7EF80BC7A79BF86280BAB94790D8C09DE7C34 y dictámenes R1 55574A71FF51A2F3C3D1E326205C5E9CD2EAA4AE7C79D30F9B6A76735A911F0F/R2 E0EF016FEDCF769FF7D3BB8A35C33EC9B7523E59FD4EF370DDBA61D50170D629 coinciden e íntegros.

Manifest declara base071f6d56 y prosa inicial conserva HEAD93658 histórico. Se evalúan bytes sellados, no se infiere limpieza actual desde esa etiqueta. Se leyó delta completo R2→R3, conjunto de oráculos y SDK real previamente fijado; el owner conjunto R2 no cambia. Los29 oráculos son prospectivos, cero familias runtime aprobadas aquí. Criterios RL01–12 precedían propuestaR1 y permanecen normativos intactos.

## Cierre RL-R2-02: callback y funding realizables en diseño

R3 sustituye F:FnOnce por LeaseCallback<T,R> propio, consumible, no dyn, con método invoke<'lease>(self,&'lease mut T)->Result<R,ShellError> y R:'static. HtmlExportCapture nominal puede implementar un trait ordinario sin implementación manual Fn*/rust-call/nightly. La vtable no-genérica conserva ErasedLeaseInvocation prestado y invoke(&mut dyn Any); bridge genérico sólo está en façade. Downcast real, Option<C>.take una vez y R fijo/'static separan tipo/Key/funding y limitan el préstamo al callback. Error de tipo deja C sin consumo; segunda invocación no ejecuta otro callback. No se acredita aún compilación: la futura compile-only debe corroborar constraints Send/Any/lifetimes, implementación y compile-fail de préstamo escapado.

La secuencia concreta no requiere una instancia C para conocer Layout: tipos HtmlExportCapture/bridge/result/permits/args/failure se declaran primero. prepare_value_reply_invocation<T,R,C> obtiene permiso desde el mismo controller/Key/HTMLWork antes fields; new_prefunded recibe ese permiso y originales Nu, construye C después y devuelve (C,permit) con fase CaptureConstructed privada. Permit es prueba de funding/moves, no otro owner del worker ni un contenedor que requiera crear otra C; el diseño no prescribe contener C recursivamente. Tuple y CaptureBuildFailure con input/options/permit originales se inventarían y financian antes la operación que los produce. Heap/capacity nueva exige reserva separada previa; inputs Nu ya existentes se distinguen de cuerpos/copias controlados nuevos.

Bridge mantiene OptionC y OptionResult storage completos aun None, permisos/tickets/layout/key/guards y capturas/moves concretos cobrados. Factory no crea closure/Box auxiliar ni cambia presupuesto. Constructorfailure conserva input/options/permit y bundle original; invoke error/unwind retorna LoanGuard al mismo slot. TypeId o phase observacional no acredita admission. No circularidad lógica de financiación es exigida: tamaños compilados son descriptores de tipos, el owner startup/admission ya existe y emite funding tipado; construir el permiso/capture/return requiere sus cargos y Work incluidos en ese inventario antes operar. La implementación deberá materializar esta secuencia, incluyendo almacenamiento/errores del propio permiso, sin introducir allocations transitorias gratuitas.

RL26/27 actualizados y RL29 fijan realización estable, once/downcast/compile-fail, exact-/1 de ctor/args/capture/tuple/failure/permit/bridge y tokens inválidos. Se conserva la exigencia previa de no derivar goldens de outputs; el cambio corrige técnica de callback, no reduce presupuesto/criterio de aceptación.

## Cierre RL-R1-01 preservado

ReplyPhysicalEnd/LoanReturned y WorkerClosed o NeverSpawned son pruebas distintas sobre MISMO HtmlCallOwned. Starting/Live/Pending contienen Child/Job/START/pipes/threads/capturas/source reales; fallo parcial conserva cada adquisición. WorkerClosed exige wait/join/close/queryJobActive0/close_checked y no números/reapTicket/None. finish_if_closed espera conjunción y ausencia de ReapGuard; Dropbundle termina fuera mutex antes Idle.

ErrorflushOK con workerPending conserva Busy/cargos hasta reap de la misma autoridad; workerClosed con writer bloqueado conserva otro eje Busy. Retry conserva fallo original y mismo Work/financialowner, funding/cancel failure sigue Pending. HtmlCompanionOwner permanece fuera serve y shutdown_retained entrega MISMOowner, no reset/service nuevo. RL21–25/28 siguen intactos y cubren ambas direcciones, fallos parciales, shutdown y retirement físico. No se atribuye RustDrop/reap a abort/exit fatal.

## Matriz de los doce criterios propios

| Familia propia | Correspondencia exacta R3 / oráculos prospectivos | Resultado documental |
|---|---|---|
| RL01 identidad real | CallId+flag/controller privado, context/foreign/dobleKey; RL03/04/18/19 | Satisfecho |
| RL02 opt-in/legacy | API hidden controllerSome sólo companion; legacyNone aun feature unificada; vtable concreta RL01/02/26 | Satisfecho |
| RL03 guard exterior | run/result/init/header/write/flush/writerNone/try_report mismo guard; RL05–10 | Satisfecho |
| RL04 loans/clones/reply único | LoanGuard único y closeAuthority del RunGuard, ninguna duplicación; RL03/04/11/12/19/26/29 | Satisfecho |
| RL05 cargos/slot hasta física | Value/raw/header bajo receipt bundle, reply+worker conjunción, SlotNone fullsize; RL05/16/20–25 | Satisfecho |
| RL06 preadmisión y Work | ctor tipado estable antes fields, sizeof/capacities old-new/capturas/returns y SAMEHTMLWork; RL14–16/27–29 | Satisfecho |
| RL07 controller único/Pending | HtmlCallOwned/WorkerResidence/shutdown mismoowner Busy hasta ambas pruebas; RL21–25/28 | Satisfecho |
| RL08 lock y writer bloqueado | préstamo/reap/IO/Drop fuera lock, estado/cancel accesible, no timeout ficticio; RL11–13/20/22/25 | Satisfecho |
| RL09 streamsRejected | Value/Error únicamente, Empty/List/Byte rechazados antes streaminit; legacy inalterado; RL01/17 | Satisfecho |
| RL10 fail/unwind/retry | initerror/helperprivado/tryreport mismo guard y outcomeFailed; RL06–10/24/29 | Satisfecho |
| RL11 últimoDrop | owners físicos/Archeaders/loan/registry/retiring y memoryshell posterior explícita; RL19–25 | Satisfecho |
| RL12 oráculos/alcance |29 escenarios antes código, sizeof/freeze futuro y compilefail; RL01–29 | Satisfecho |

## Límites y gate siguiente

Este cierre permite implementar el delta SDK/adaptador bajo revisión independiente futura; NO acredita una API ya existente, tamaños reales, memoria agregada medida ni ejecución del ledger. Antes runtime se requiere compile-only y freeze inventory/bin/hashes/callsite/fixtures: todos bodies, alignment/Arc header/capacities, actual+candidate, ctorArgs/tuple/CaptureBuildFailure/permit/result y loans/reapGuard físicos deben financiarse antes copy/take/spawn. Exact-/1/Work, downcast/once, fail/unwind, initerror/try_report/write/flush bloqueados, workerPending/replyDone, shutdown y últimoDrop requieren CodeAE/AE concretos; bools/observationalStatus no sustituyen autoridad.

Encoder legacy, core streamwriter y serializer no reciben cambio funcional propuesto: ausencia de regresión debe comprobarse en candidato compilado exacto. No se amplía guard a streams ni se promete CPU/RSS total SDK/allocator/shell. Registro Nu ya cerrado, DOM/selectedness/raw/encoding y caps HTML permanecen intactos. QA/parser/conversor/RV401–404/Unix siguen pendientes. Cero críticas accionables de DISEÑO en este candidato exacto.
