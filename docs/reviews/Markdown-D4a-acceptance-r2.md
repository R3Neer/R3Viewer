# D4a R2 — aceptación independiente focal

**SIN_CRITICAS_PENDIENTES dentro de D4a.** MD-D4A-OWN01 corregido; aprobado el incremento input retenido/codec host/lifecycle Syntax dentro del alcance combinado R1+delta R2. No D4b/c, TUI, renderer, native Unix o cierre RV. R1 sigue rechazado e inmutable; sus fallos no se convierten retrospectivamente en PASS.

## Identidad

Freeze72 registros MANIFEST4083717EA4C72466AFB75EEAC5CE54EDB3190D7DAF8F1827F81033745090FB6F antes/después sin diferencias. Zip368 rehash sin diferencias; nueva copia/target propios. Copy367 intactos y syntax.rs PRODUCT prefix exacto más sufijo cfgtest; archivos de harness separados, sin cambiar funciones productivas. Worker FINAL copiado F6868C1346604CC4B8DA05903D8FFBAD9B290A2560689EBD173CE5DADA226B6B usado en aceptación nativa, sin atribuirle los logs previos del autor.

Tres expectativas R1 copiadas byteexactas. Addendum previo8269C3380C6A703F482CE9D12512567A599C5747FB9E8B729A55BBD376B94E74 fija los cambios de API sin alterar criterios de rechazo. Outputs del autor sólo contexto, nunca golden ni aceptación propia. Ningún shared/candidato/profile modificado.

## OWN01 cerrado

Original RetainedRoleSet::new(Vec,input,None) conserva snippet independiente de misuse: compilación negativa exit101/E0599 por método inexistente; COMPILE-negative-original-misuse-attempt1.log. No se cambia la antigua expectativa para aceptar un vector sin cargo.

Revisión de las tres fuentes R2: OwnedRoleAllocation sólo se construye por decoder, contiene vector/capacidad/cargo/Input/header privados y sin Clone o salida mutable. RoleCharge ahora privado. read_host_frame deriva Consumer del Arc input exacto y retiene la misma allocation con cargo de capacidad real; no existe argumento de cargo/budget intercambiable. FrameReceipt posee el bundle, take_roles mueve único owner. verify_owner coteja ptr_eq/input/generation/request/capacidad real y presencia de cargo para allocation Document. Constructor RetainedRoleSet consume receipt con esos checks, acceso pub(super), metadata cobrada antes de Arc. Empty allocation se admite por decoder; helpers cfgtest Legacy niegan Document incluso vacío. Supervisor mantiene check_frame→owner→validate_spans(original UTF8/orden/cobertura/viewport)→deadline→from_receipt. Ownership cerrado y validación semántica son pruebas distintas.

Cinco grupos puros propios PASS (0.00s), binDCEA4E311131565761766A7425ACEC392B984F98B80375BC3DF1F023D748603E:

* Receipt/input ajeno del mismo Consumer y de otro Document rechazados; generation/request alterados rechazados; exacto aceptado; cargos de ambos budgets retornan a baseline.
* Consumer input lleno rechaza decoder antes de cuerpo, read32B, aunque otro Consumer dispone de espacio; no segundo budget que financie ese input.
* Capacidad vector materializada y alias final: cargo exacto + input retienen hasta último alias, no copy Vec; Weak input y ledger se liberan entonces.
* Document vacío decoder válido capacity0; raw helper Document negado; Legacy read_frame aún entrega Vec wire original separado y receiptLegacy válido.
* Body+roles simultáneos exact/+1: quota exacta peak=limit,1byte menos niega antes body; cargo retorna; solicitudes Read<=4096.

Un control nuevo adicional PASS (0.00s), bin4154324BDD11081C21D0802AA7755B019B83642787F23BFF7BFC811684CE799E: Styles opacos de decoder con header/request correcto retenidos como event; poll_at(deadline) los descarta, produce Plain timeout sticky y no output Styles tardío; expirado antes set mantiene updateNone/generation0, sin spawn. Harness inyecta event para probar exclusivamente el último poll, no atribuye esa inserción al productor. Cargos retornan a baseline.

## Regresión Legacy → Document real

Función original propia R1 copiada byteexacta, sin reescribir su requisito Styles tras los FAIL. Helper de contexto Document aquí5s conforme operación color; R1 helper usaba30s para controles de ledger, su primera rama Legacy falló antes de usar ese contexto. Startup2s/perline500ms/inline5s iguales, ningún plazo ampliado. Native execution sobre worker FINAL, procesos/Jobs propios, cierre interno antes de assertions; perfil PowerShell/exec Windows **escalado**, cwd del chat D:/Universidad/Clases. Builds privados también escalados por LNK1114/error5 ya observado históricamente, no ajustes de sandbox presentados como mismo intento anterior.

TEST-native-original-attempt1.log: 1PASS2.02s, executor91F6ED4AA26305AFF15801DF129B022F69345F7A774350153DA276B21878E7B8. Legacy gen22 fixture12B `let s="é";` con LF recibe Styles contiguos completos0..12, request1; join=true, SYNTAX_ACTIVE/PENDING_PRESENTfalse. Siguiente Document gen23 fixture11B `# retained` con LF recibe roles completos, shutdown joined, input sigue vivo con alias del resultado y se libera tras drop; budget64/64 y ambos globalsfalse. No QA visual, teclas, restauración terminal o gramática13matrix.

Los traces cfgtest del candidato exacto localizan **esta ejecución**: CreateProcess-return1.8537759s, attach1.8540536s; deadline startup restante145.6835ms, Boot23.4781ms, Init/Ack76.0482ms; chunks/Ack76.3524ms. Doc siguiente CreateProcess5.4267ms y Ack66.8974ms. Traces son instrumentación observacional cfgtest preexistente del candidato, sin cambios productivos de reloj. No explican causalmente FAIL históricos R1/autor sin traces. No SLA ni promesa de syscall cancelable.

Contraste estático independiente con syntax compartido preadopción (snapshot privado BASELINE-current-shared-syntax.rs/hash AFTER.json): deadline original ya se fijaba antes de preflight y Connection::spawn; R2 usa context.phase(now,startup), cuya rama Legacy sigue now+duration. Defaults/topes no aumentan. No evidencia de renovación o regresión concreta de ese presupuesto; agotamiento observado conserva Plain y cierre en R1, positivo de Styles real ahora acreditado en R2. Los dos FAIL propios R1 y FAIL del autor siguen distintos y no borrados. Ninguna repetición nativa aleatoria para obtener green.

## Cierre combinado y límites

Método Connection.close R1/R2 texto LF idéntico comprobado por script; authority/métodos wait/Active0/pipe/Job checked close/joins/persistent slot no sustituidos por opaque bundle. Nueve grupos propios R1 preservados incluyen cancelActivo+Pending con pipes físicos vivos, deny-nextspawn, retryReaped failedsticky y setup fail físicas. No rerun mecánico de esas invariables. R2 nueva asignación receipt y ownership comprobados por seis grupos puros + positivo nativo finalbin. Cierre de esa nueva ejecución se acredita por shutdown/join y globals, sin inventar nuevo CloseHandle custom de stdChild o foreignPID/descendant coverage.

Historial: un error E0596 del harness (slice no mutable) preservado y corregido sin tocar fuentes. Un intento de copiar el executable antes de terminar sesión de build36630 fue error de orquestación: no runtime/PASS, log vacío y nota HARNESS-copy-before-build-finished.md; se esperó misma sesión terminal0, luego copia/hash/run real con TEST-functional-attempt2.log. Compilaciones suplementarias previas a output ajustaron header para un evento válido; no outputs reinterpretados. No suite general/fixtures gigantes/rebuild de dependencias repetido. Se conservan todos logs y executors exactos.

No sesiones propias activas, sin rescate externo y globals sintaxis inactivos/Pendingfalse tras native. Excluidos D4b snapshot/profile/keys/scheduler, D4c render/overlay/inline/TUI, backend visual/ConPTY q/130/Unix/SLA. El atributo Git para preservar fixtures CRLF será packaging separado; aquí sólo raw working identities, ningún claim de blobsGit/checkout ajenos.
