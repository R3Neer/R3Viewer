# S03 Unix probes R4 — respuesta a UP-04

2026-10-05. Autor/refinador del harness, pendiente revisor independiente. R3 tenía un fallo real de flujo: el finally del guardian alcanzaba la rama hija con fork_result0; un close fallido podía llamar kill(0,SIGKILL). Se conserva R3 como evidencia y NO se ejecutó esa ruta en Unix. El reporte independiente BB89190D2F41448B8CADABFDCDC4BAF9411CC5300CC3AB6CFAB4B7D3A5E06AF3 cerró UP-03 padre y abrió UP-04 P1; no se presenta este arreglo como aprobación propia.

Snapshot R4 D:\Tools\R3Viewer\.tmp\unix-containment-probes-r4 y copia idéntica Cviz/unix-containment-probes-r4; MANIFEST SHA2566756711265F822993C60A6441312D2D78A46857C32E8617A1B69F1197C74850D,13 entradas verificadas byte/SHA. probe.pyC5389A737CFD3F0E2EA03F5CB2CAFBD9B720D672DDA93171CCD0D3A630178DEE. R1/R2/R3 manifiestos y5/8/11 entradas intactos. C y MIT sin cambios. No producción, Cargo raíz, CI, instalación, perfiles, señales ajenas ni native Unix.

UP-04: setup/fork separado del try/finally del guardian padre. Sólo retorno positivo validado establece worker_pid. Retorno0 entra exclusivamente en worker_child_entry: closes/setup/entrada worker protegidos y error termina localmente2 sin emitir recibos, llamar cleanup del padre o caer a él. guardian retorna esa rama, de modo que incluso un retorno de exit inyectado no puede continuar al ownership padre. Setup/fork fallido aún no tiene hijo retenido; diagnóstico best effort y salida2.

Defensa en profundidad: require_owned_pid exige int exacto positivo (también rechaza bool). signal_owned, reap y cleanup_owned llaman la defensa ANTES de todo OS signal/wait. La única llamada os.kill está dentro de signal_owned; la única os.waitpid dentro de reap. PID0/negativos no se convierten en grupo/wildcard. El signo positivo no otorga ownership de cualquier PID: callers siguen usando exclusivamente fork-parent return retenido y unreaped, sin enumeraciones/búsqueda de nombres. Reaped identity se borra sólo tras wait terminal. [Fuente primaria kill](https://man7.org/linux/man-pages/man2/kill.2.html), [fork/wait Python](https://docs.python.org/3/library/os.html#os.fork), consultadas2026-10-05.

Controles UP-04 guardados CHILD-BRANCH-CONTROLS.json SHA7FE7D61DE95EE48341AD1B35B7C2410D8FF8C8E6C5FA948F9DFD53FCB22BBD8A. Todos fork/FD/exit/signal/wait afectados son ficticios: siete fallos separados en close(lread/hwrite/awrite/readyread/fd1/leaked_writer) y preparación worker. Cada trace termina2 local; ningún cleanup/signal/wait/emit del guardian se permite. Dieciocho combinaciones (0,-1,-77,bool,None,string por signal_owned/reap/cleanup_owned) exigen ValueError y cero llamadas OS. No señal real para reproducir P1.

Regresiones: dos guardian_controls padre de UP-03 reejecutados PASS (STOP/wait, admitted BrokenPipe, KILL/wait o error, sólo después diagnóstico; sin Reaped falso). Seis failure_controls del observador reejecutados PASS, con Child Windows propio confirmado vivo/waited/readers cerrados; readiness/read se inyectan. Seis fuentes Python syntax PASS. --check CHECK_ONLY. Los dos controles nativos --execute-native en Windows rechazan2 antes de fixture spawn. No AS/POSIX/Mach/SIGSTOP/reap nativos comprobados. Sources/evidencia nuevas propias, licencia MIT exacta conservada.

Receta nativa futura NO autorizada por esta entrega hasta revisión independiente de R4:

```sh
python3 probe.py --check
python3 child_branch_controls.py
python3 guardian_controls.py
python3 failure_controls.py
python3 native_pipe_control.py --execute-native
python3 native_guardian_control.py --execute-native
python3 probe.py --execute-native --cc cc --output /absolute/new/private/probe-results-r4
```

Los controles de ownership son inyectados; ambos native controls y suite conservan INCONCLUSIVE/exit2, no skipped verde. Controles preparados no acreditan producto ni deadlines universales. Siguen pendientes ABI/baseline causalidad, antes de exec/charges, presupuesto native y aggregate, guardian-loss/orphan reap, parser/companion reales/FD/ACK, corpus y cgroup birth en corte distinto. Sin migración tardía ni raíz/grupo global. Necesita revisión del delta R4 por revisor distinto; no cierre U-G1–5/S03 ni soporte Unix.
## Estado final

[Revisión independiente R4](../reviews/S03-Unix-probes-r4.md): SIN_CRITICAS_PENDIENTES en la preparación del harness privado; UP-04 cerrado. La receta nativa anterior sigue sin ejecución: únicamente disponemos de Windows, por respuesta directa del usuario. Una futura matriz GitHub Actions requiere su preparación y destino autorizado; no sustituye QA visual. U-G1–5, runtime Unix y producción permanecen abiertos. Esta copia conserva la entrega previa y añade el cierre; no modifica snapshots.
