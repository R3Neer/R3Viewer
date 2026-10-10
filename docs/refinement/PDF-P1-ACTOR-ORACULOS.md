# Base adversaria independiente PDF P1 actor — 2026-10-10

Estado: criterios prospectivos fijados antes de consultar candidato privado. Alcance documental: autoridad física nativa de sesión, kernel local finito, Pending/reap/shutdown/orphans y plan de ledger compilado. Sin implementación, runtime ni QA UI. No se ha consultado .tmp/pdf-p1-actor-refinement-2026-10-10.

Primarias leídas: AGENTS.md, README.md, CONTRIBUTING.md, docs/spec/DECISIONES.md, docs/BACKLOG.md, docs/process/METODOLOGIA.md, docs/architecture/ARQUITECTURA.md, investigación técnica relevante PDF/worker, PDF-PRODUCT-INTEGRATION.md y PDF-PRODUCT-ORACULOS.json; src/document_controller.rs, src/document_budget.rs, src/syntax.rs y src/preparation_owner.rs, centrando contratos/ownership y transiciones.

Base observable: Documento conserva BUSY/PENDING propios y owner proceso; Sintaxis conserva SYNTAX_ACTIVE/PENDING_PRESENT y Connection propia; preparación finita usa OCCUPIED/PENDING y cajas tipadas. Ningún flag local por separado acredita exclusión común. ActorArena cobra reservas controladas; no acredita memoria total ni estructuras de un diseño sin compilar. Producto exige max1 actor nativo por sesión, Pending global real, recuperación del mismo owner y ledger físico hasta retiro completo.

## Oráculos independientes

A01 — Dos handles de la misma sesión intentan PDF/Documento/Sintaxis en barrera. Una adquisición linealizada gana y únicamente ella puede iniciar bootstrap/spawn; el perdedor no obtiene un permiso nativo o autoridad de cleanup ajena. Pending y Running conservan ocupación. Identidad sesión distinta no debe accidentalmente compartir autoridad si el contrato es por sesión; un bloqueo global conservador adicional debe declararse.

A02 — Constructor público alternativo, default, clone, factory genérica, API legacy o llamada directa a spawn intenta saltarse el scheduler. Todas las rutas que reclaman cobertura común exigen capability de sesión verificable, privada y no falsificable; rutas no migradas declaran explícitamente su exclusión del claim. Copiar flags de familias no basta. Debe haber inventario preciso de puntos de spawn/adopción y alcance exacto de max1.

A03 — Fallo/panic/cancel tras cada adquisición (permiso, owner box, pipes, Child, attach Job, reader, writer, supervisor thread). Cada recurso queda en el mismo propietario tipado o escrow recuperable; transferencia Pending no asigna memoria imprescindible tarde. Un destructor no libera permiso por abandonar un objeto con recursos sin prueba. Permiso sin child también tiene retirada definida.

A04 — Finish lógico/result Ready antes del cierre físico; exit0 con pipe/reader/writer/Job pendientes; owner local bloqueado mientras nativo limpia. Admisión permanece cerrada hasta recibo conjunto de wait/active0/handles/streams/joins; el kernel finito puede tener un slot local separado sólo si no permite segundo actor nativo y queda clara la política de overlap/cancel.

A05 — Dos callers concurrentes solicitan retry_pending/poll/drop/cancel del mismo owner. Un único reclamante extrae el owner para limpiar fuera del lock; la celda conserva estado Reaping ocupado, owner identity y cargos mientras está fuera. Ningún hueco Idle ni error al reinsertar pierde autoridad. Retry devuelve al mismo estado si prueba incompleta, conserva causa original y jamás publica candidato fallido.

A06 — Session/handle último Drop durante Running, Pending o Reaping; supervisor termina más tarde; shutdown sin acceso UI. Orphan registry conserva owner, sesión, permiso y cargos, queda accesible para recuperación posterior independiente del handle perdido, sin búsqueda por nombre/PID genérico. Shutdown tiene deadline absoluto y resultado distinguible; reingreso/segunda sesión no crea un actor que invalide Pending que el contrato diga global.

A07 — Lock poisoned, callback de cleanup/retiro que reentra y recurso cuyo Drop toma el mismo lock. Ningún join vivo, espera OS, close potencialmente largo, destructor de snapshot/ledger/candidato ni callbacks se ejecutan bajo lock de autoridad. Extraer/retirar es seguro; un error de lock no se convierte en fuga, falsa Idle ni pérdida de owner. El contrato enumera lock order y objetos cuyo drop se difiere.

A08 — Token/generation/request wrap, stale reap receipt, alias de handle y cancel sobre request anterior mientras uno posterior está admitted. Identidad no reutilizable o fallo checked previo; recibo sólo puede retirar su actor/epoch exacto; cancel scoped no modifica otro actor. Cancel/deadline/stale son sticky aun cuando limpieza posterior tenga éxito.

A09 — Snapshot Arc aliasado entre writer/history/provider/candidate y dos familias. Se cobra una asignación física/capacity real una vez, metadatos owners por separado, y cada alias retiene capacidad sin liberar cargo. Retiro LRU o output move no desconta hasta último owner y cleanup físico. Tras crédito, recurso ya no accesible por alias; old+new y crecimiento coexistentes se reservan antes de asignar.

A10 — Reservas boundary/+1/overflow, thread spawn fallido, entrega channel Full/Disconnected, completion tardía retenida, Pending/orphan persistente. Plan ledger enumera cada owner allocation/capture/mailbox/receipt/slot/candidate/transitorio y transferencia sin duplicar ni descontar anticipadamente; allocator/thread/runtime/OS/native desconocidos quedan fuera del claim. Cifras size_of solo se denominan compiladas tras compilar tipos concretos con target/toolchain y hash exactos; ahora son prospectivas.

A11 — Writer anuncia SourceEnd/enqueue/close_requested pero físico stdin sigue abierto; respuesta primer byte gana carrera; error flush/close/panic o EOF late tras End. Receipt físico observado bajo misma sincronización precede respuesta; todo fallo post-join se incorpora antes de commit. No se libera autoridad o valida/publica por flags lógicos, ni hay segundo actor V sin permiso/cargo.

A12 — Cola viewport reemplazada varias veces y stale Pending; cancel q/CtrlC/shutdown antes bootstrap y durante llamada nativa bloqueada. Coalescing y callbacks no acumulan owners ilimitados; dropped queued work retira cargos fuera lock. El request deadline absoluto no se renueva por progreso/cleanup. Native bloqueo sólo se trata con supervisor propio; no se requiere Drop nativo tras kill, ni unsafe Unix fallback. Alcance P1 interno/perfil original separado de P2 cuota producto, RV204/QA pendientes.

## Gate de revisión

Aceptación de diseño exige contrato suficientemente preciso para derivar transiciones y futuras pruebas de A01..A12; evidencia actual de legacy identifica puntos a adaptar y evita afirmar migración ya realizada. No exigir implementación/runtime para cierre documental; sí exigir plan prospectivo verificable. Hallazgos tendrán ID/escenario/evidencia/consecuencia/resolución y sólo se cerrarán/refutarán contra candidato exacto. Veredicto SIN_CRITICAS_PENDIENTES limitado al diseño focal, sin aceptación de integración ni ledger medido.
