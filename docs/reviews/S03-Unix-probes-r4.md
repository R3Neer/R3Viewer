# S03 Unix probes R4 — revisión independiente final del delta

2026-10-05. **SIN_CRITICAS_PENDIENTES** para la preparación privada y el delta que corrige UP-04. UP-01/02/03/04 cerrados en este alcance; no ejecución Linux/macOS, aprobación runtime Unix, soporte producto ni cierre U-G1–5/S03.

## Identidad

Revisado `.tmp/unix-containment-probes-r4` y copia Cviz idéntica, MANIFEST SHA256 `6756711265F822993C60A6441312D2D78A46857C32E8617A1B69F1197C74850D`,13entradas. Entrega `1FCE240DD9929CA0A996159DD825E4C8B7D57B945A7C818BF985F77E884D396E`; probe.py `C5389A737CFD3F0E2EA03F5CB2CAFBD9B720D672DDA93171CCD0D3A630178DEE`. Todas las entradas/bytes coinciden con manifesto en ambas copias; comprobaciones guardadas antes/después de exploración adicional en `unix-review-r4-before.json`/`unix-review-r4-after.json`, idénticas. Ninguna fuente ni producción modificada, ningún OS instalado ni perfil/remoto/CI alterado.

Autor distinto del revisor. Autoridad/diseño CONTENCION-UNIX y revisiones R2/R3 leídos en este mismo encargo. La revisión actual inspecciona el delta R3→R4, guards/ownership y regresiones del harness; conserva límites de ensayos anteriores.

## UP-04 cerrado

Fork y setup preceden al try/finally con ownership de G. Retorno0 entra en worker_child_entry, que protege closes/entrada worker y termina localmente2 sin recibo o cleanup padre. guardian retorna esa rama; sólo retorno int exacto positivo establece worker_pid en padre. Ninguna excepción de close en P puede alcanzar finally del guardian. Un fallo setup/fork previo a propiedad termina con diagnóstico best effort/exit2.

Las únicas llamadas os.kill y os.waitpid pasan por signal_owned/reap, con require_owned_pid antes OS. cleanup_owned también valida antes de señalar. Rechazan0, negativos y tipos no int, incluidos bool: no wildcard/grupo por identidad inválida. Int positivo sólo valida forma; la autoridad sigue siendo el hijo directo retenido de fork, no cualquier PID positivo. Wait terminal sigue precediendo a worker_pid=None; no rescate por nombres, enumeración o grupos.

Reproducción independiente del mismo contraejemplo R3, ahora sobre fuente R4: `unix-review-r4-own-control.py`, fork0 ficticio y close(100) fallido. Resultado guardado `unix-review-r4-own-control.json`: trace sólo close(100), exit2; **cero kill/wait/emit**. En R3 ese control alcanzaba kill(0,9)/wait(0,1); R4 elimina la ruta. Todos estos OS/FD/fork/signal/wait/exit son inyectados; no señales reales ni fork Unix.

Además se reejecutaron los siete controles child_branch_controls de fallos separados en seis closes y entrada worker: salida2 local sin cleanup/signal/wait/recibo. Sus18guards invalid-PID pasan. Exploración propia adicional exige rechazo True,1.0,0,-9,None,string en las tres funciones antes de OS, también18combinaciones; evidencia `unix-review-r4-own-pidguards.json`. **UP-04 cerrado** por recorrido estático y controles deterministas, sin ejecutar el defecto en Unix.

## Regresiones y fronteras

UP-03 padre conserva finally anterior al diagnóstico best effort. Dos guardian_controls propios reejecutados: admitted BrokenPipe tras STOP/wait ficticios, KILL/wait terminal o wait fallido, después segundo BrokenPipe; no Reaped artificial. Logs `unix-review-r4-guardian-controls.json`.

Seis failure_controls reejecutados con Child Windows propio real confirmado vivo, waited y readers cerrados. Setup/read/JSON/callback/persistencia/watchdog/stdout-only EOF conservan tratamiento inconcluso; readiness/read son inyectados. Logs `unix-review-r4-observer-controls.json`. UP-01/02 permanecen cerrados para el observador. Deadline único stdout/stderr,64KiB por canal,30s compiler/10s memory/6s case más cleanup2s explícito; no se equiparan con deadlines de producto. Popen antes de retorno, espera/reap universal y descendientes del compiler siguen fuera de garantía.

Seis fuentes Python compile-only PASS; --check CHECK_ONLY. native_guardian_control y native_pipe_control --execute-native rechazados en Windows antes del fixture, con INCONCLUSIVE; no nativePASS. Lease L/H, modes intencionalmente negativos, ausenciaACK no como Reaped, sólo wait(G) de UI viva y recogida huérfana no acreditada se conservan. El control nativo preparado necesita ejecución/revisión de evidencia en Linux/macOS antes de cualquier adopción. Fallo simultáneo G+P suspendido no se promete recogido.

AS después de bootstrap del stand-in,512MiB por G/P separado, no committed ni agregado; native ABI/baseline causalidad, antes de exec/charges, presupuesto guardian, corpus/parser/companion reales, ACK/FD across exec, pérdida guardian/orphan reap y cgroup birth permanecen pendientes. El cierre de este delta no valida esas capacidades ni acepta fallback polling como defensa.

**SIN_CRITICAS_PENDIENTES** exclusivamente en preparación privada R4 y correcciones revisadas. Se puede conservar este dictamen como cierre adversario del harness preparado; runtime Unix y producto siguen abiertos.
