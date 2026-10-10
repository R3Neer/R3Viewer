# Dictamen adversario PDF P1 actor R2

2026-10-10. **SIN_CRITICAS_PENDIENTES**, exclusivamente sobre el diseño actor R2 exacto. PA-R1-01..05 quedan cerradas documentalmente. No implementación, compilación, runtime, ledger físico medido, adopción del wire R4, aceptación P1 productiva, RV204 ni QA Windows/Unix.

## Identidad y autoridad previa

Propuesta revisada: .tmp/root-pdf-p1-actor-review-2026-10-10/PROPOSAL-P1-ACTOR-R2.md SHA A347444D894961A6614A6AEF492A243AA628A3A4D429D5F8BACED5F7B025EF5C.
MANIFEST-ACTOR-R2.json SHA D94994A7A3B57228F3C8377C9E5721BBA0FF77C965AB26A7503A0AF6FCC657EC. Verificación propia de las36 entradas: SHA y tamaño coinciden, cero discrepancias; efectuada antes lectura de propuesta.

Primero leídos BASE-ORACULOS.md SHA9E60416DB34383D62D986562DBA030622E3405B5B695BFAD0E4671A2B23F3213 y REVIEW-R1.md SHABA3C9FB2D3F19CC11786D2AABD54AB18A4D39FAEC071FF4C9153EB5C59829DE0. A01–A12 propios permanecen intactos. Leídos R1 completo y EXPECTED-ACTOR-R1 para verificar inventario importado y la corrección explícita del esperado A07 del autor. Los IDs A01–A12 propios y los IDs del JSON autor son conjuntos diferentes; no se intercambian para declarar cobertura. Ningún output autor usado como oráculo; todos los casos futuros siguen NOT_RUN.

## Cierre individual del rechazo R1

**PA-R1-01 — Cerrada.** ExecutionCell tiene Parked/Loaned(epoch), pero la posesión mientras Loaned reside en CapsuleLoan<C>, no en marker. Guard noClone contiene C y Arc del escrow; executor recibe métodos &mut y no ownership extraíble. Se financia antes take; adquisiciones/failures se incorporan antes siguiente operación falible. Drop mueve de vuelta sin asignar/IO/destruir C, recupera poison con into_inner y no se ejecuta bajo locks de arbitraje. Epoch inválido no conduce a destructor/fallback: API privada lineal debe hacerlo inalcanzable. Thread bloqueado conserva préstamo y owner registrado; no se anuncia Reaped. El mecanismo es implementable en Rust estable con campo Option<C> privado, movimientos y guard Drop; no requiere implementar manualmente FnOnce ni unsafe escape.

La obligación futura abarca factories parciales por dentro: cada factory que adquiera varios recursos debe conservar sus adquisiciones en owner/guard propio antes otra operación falible, incluso unwind antes devolver owned failure. La promesa de devolución owned failure cubre Result, no acredita por sí sola panic interno. R2 exige conservar ante unwind en cada adquisición; D1/D2 deben revisar esos cuerpos reales. No se toma esa obligación prospectiva como runtime ya demostrado.

**PA-R1-02 — Cerrada.** ControlLifetime es objeto compartido exclusivamente por handles públicos, con Weak<Shared>; registry/core no lo retienen. Drop del último control activa ExitIntent y wake mientras el handle aún mantiene Shared fuerte. Orden control→Shared es realizable por orden de campos o campos Option con retirada explícita; no Arc::strong_count como disparador. try_alias preadmitido sustituye Clone infalible; no nuevo handle una vez exit fijado. Todos los ocupados ya están registrados desde admisión, de modo que no hay allocation de orphan tardía ni depender de Drop del ciclo Shared/core. Executor persistente y backstop actúan sobre ese owner original, conservan cargos si no pueden concluir. El ciclo se rompe sólo tras proof/retirada, fuera lock.

**PA-R1-03 — Cerrada.** Política process-wide elegida explícitamente conservadora: una residencia nativa session-bound por proceso CLI; procesos CLI independientes. Se conserva coexistencia histórica Document+Syntax legacy, pero todos los estados Session Starting/Live/Pending/Reaping/Retiring bloquean nueva residencia nativa. Local tiene faceta y slot propios; trabajo puro puede coexistir con SyntaxLive.

Registry→Shared es el único orden doble, preadmisión antes ambos guards y punto linealizado insertar registro+Starting bajo ambos. No Cell/IO/join/lastownerDrop dentro. Reaping mantiene entrada/core ocupados; Retiring mantiene exclusión mientras se retiran fuera lock transitorios; sólo después transición Idle/Empty. Hooks legacy reservan antes gates/spawn y retienen hasta proof, no BusyLease Drop. No claim universal hasta implementar y revisar todos esos hooks. Es un protocolo concreto para derivar carreras, no check de atomics.

La implementación deberá mover referencias desplazadas a locales antes soltar guards y destruirlas fuera lock, también en transición final Empty/Idle. Esa obligación está explícita en la prohibición de lastownerDrop y en retirada fuera lock; cambiar enum no acredita haberla cumplido.

**PA-R1-04 — Cerrada.** D0 contrato; D1 prototipo privado de tipos/capturas/inventario real, aún sin producto aceptado; D2 port candidato frozen con AE propia; D3 integración tras gates. D1 no exige AE física productiva previa para autorizar el código que la producirá. Cambios de tipo/capture/capacity invalidan inventario afectado y deben recotejarse en D2. OPEN_COMPILE conserva requested Rust, estimate Arc y ExternalUnknown separados.

**PA-R1-05 — Cerrada.** SourceLegacy↔CodeDocument, BlockKey/input exacto o OperationContext distinto exigen pause/reap físico y nuevo Init. El reuso corresponde al mismo SyntaxClient/scheduler/Shared, no al viejo proceso. Viewport/request puede mantener Connection únicamente con identidad exacta y contexto vigente; cachehit/redraw no exige respawn. Corrección del esperado A07 autor es explícita; no modifica el oráculo propio A08 ni acepta rebind de wire.

## Matriz contra A01–A12 propios

| Oráculo propio | Contrato documental R2 verificable en fases posteriores |
|---|---|
| A01 simultaneidad | Registry reservado+Starting bajo Registry→Shared, capability privada noClone, tabla Session bloqueante process-wide. |
| A02 rutas alternativas | Hooks legacy antes gates/spawn; interfaces/perfiles preservados; exclusión universal condicionada al inventario de todos los puntos reales. |
| A03 adquisición/fallo | CapsuleLoan conserva bundle durante unwind; campos/failures preadmitidos; mailbox de endpoints sobre threadspawn fallido; permiso NeverSpawned retira bajo Retiring. |
| A04 Ready vs cierre | Proof conjunto child.wait/Jobactive0/checkedhandles/IOjoins+errores/supervisorjoin/loanreturn antes retiro; Local no obtiene capability nativa. |
| A05 retry concurrente | Préstamo único, fases Reaping/ocupado y core original; no extracción que deje registry vacío; foreign owner vivo rechazado. |
| A06 último Drop/orphans | ControlLifetime independiente, ExitIntent idempotente, registry desde antes spawn, owner original accesible a backstop; Drop no espera. |
| A07 locks/reentrada | Registry→Shared, Cell fuera arbitraje, poison recuperado guard; ningún close/join/callback/lastDrop bajo locks; retirada desplazada fuera guards. |
| A08 stale/checked/cancel | Identidad privada checked y core ptr_eq, cancel scoped por core, exit terminal sesión, sticky irreversible, reaper publica sólo original. |
| A09 alias/capacidad | Snapshot/source únicos y recibos, old+new/roles/cache/history/candidate financiados hasta último Drop, transfer no refund. |
| A10 ledger | R1§6 inventory completo importado más ControlLifetime/Registry/Loan/Cell/executor; D1 captura real, límites exact/−1/+1 antes outputs, unknown nunca cero. |
| A11 transporte físico | Framing/receipts/end/EOF/joins importados por hash, no Ready por enqueue/end/OptionNone; rechazo tras error tardío. No aceptación wire asociada. |
| A12 cola/cancel/deadline | latest1/result1 inventory importado; exit antes/tras bootstrap, supervisor de worker bloqueado, original deadlines/sticky, finito Local separado. |

## Límites y gates concretos que siguen pendientes

D0 puede cerrarse para este actor. D1 requiere autorización de prototipo privado: tipos/capturas/capacities y source/toolchain/target/Cargo.lock exactos, financiación previa, instrumentación requested/estimate/ExternalUnknown y simultaneidad real. D2 debe congelar antes outputs las pruebas propias A01–A12, suplementos guard/último público/registry/inventario y los oráculos producto pertinentes; probar cada adquisición/factory parcial/unwind, spawnfault, cleanup pendiente, stale, alias/lastDrop y hooks legacy reales. Recuperación del mismo propietario no equivale a nuevo cliente reaper para un owner vivo.

El perfil H40MiB/D24MiB dentro del agregado, source1MiB/raster4MiB/body65536 y Job512MiB committed sigue prospectivo en estos tipos nuevos. No claims de RAM total, malloc de PDFium conocido, layout Arc ABI estable ni sizeof compilado ahora. No dos bitmaps ni captura completa inner wiretext para aparentar capacidad.

Request10s se entiende como plazo PDF original del perfil, no autorización para cambiar defaults/OperationContext vigentes de Syntax u otras familias. Los contratos de cada familia, cleanup por intento y contexto original siguen obligatorios; D1/D2 deben preservar esa distinción al conectar adaptadores. Reap exitoso no revive operación fallida.

R1 rechazo y sus cinco hallazgos permanecen íntegros. Esta revisión no ejecuta ni acepta framing/geometría R4 por importación; identifica exactamente su dependencia. No modifica canónico, SDK, worker ni código productivo. SIN_CRITICAS_PENDIENTES limitado a diseño actor R2 exacto.