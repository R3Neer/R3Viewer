# S03 Unix — revisión independiente de propuesta R2

2026-10-05. Resultado: **SIN_CRITICAS_PENDIENTES** en el diseño revisado. UX-01 y UX-02 cerrados en propuesta. No aprobación de implementación ni certificación de ejecución/soporte Unix.

Propuesta exacta `S03-Unix-containment-proposal-r2.md`, SHA256 verificado `5776546D64E9E26B87AA86270120259D18AFFDAC99D74AD429A8608B47302F09`. Revisión anterior `S03-Unix-containment-review-r1.md`, SHA256 `26B1238ABA4156D8D8E6E70152F4F0E00035A9F08E8C4227651D2814EB055A2B`. Contraste con contratos/requisitos y S03/S03-D1-core/S03-D4-XML, worker_process actual y fuentes primarias de R1. Sólo documentos y fuente; sin cambios producción/Cargo/XML, build, instalación, CI/remoto o ejecución Unix.

## Cierre UX-01

U06 fija G externo obligatorio en macOS: UI es dueña de G; G es dueño de P. Sólo G hace waitpid(P); UI espera G y exige ACK Reaped válido, sin adjudicarse recolección de su nieto. Un EOF/exit de G sin recibo del dueño real no autoriza otro parser: latch de fallo y cleanup verificable. La topología ya permite distinguir señal de terminación, wait del parser y wait del supervisor.

L tiene escritor único UI, lector G y whitelist heredada explícita; P/companion no conservan escritor ni lector que falseen EOF. H tiene escritor G y lector P. G no depende de ejecución del parser para actuar ante SIGSTOP(P)+pérdida UI. La defensa interna de P runnable ante pérdida G se distingue del caso suspendido y no se anuncia cleanup universal cuando ambos guardianes fallan. Bootstrap y parámetros efectivos se confirman antes de documento; costes/clocks y fallos tienen clasificación explícita.

El nuevo U-G3 exige SIGSTOP(P)+muerte abrupta UI tanto en bootstrap como en parse, sin rescate oculto del harness; ACK+wait(G), fuga de escritor L y fallo único G se comprueban por separado. Esta prueba sigue pendiente y deberá demostrar terminación real, cierre de FDs y recolección; la firma de wait/kill no es evidencia runtime.

**UX-01 cerrado en diseño.** No se necesita otro guardián indefinido para prometer tolerancia a fallo arbitrario simultáneo: la frontera limitada está declarada, conserva cap y no inventa Reaped.

## Cierre UX-02

U08 selecciona grupo configurado y leído antes de nacimiento con clone3+CLONE_INTO_CGROUP, sin CLONE_VM/CLONE_FILES y con SIGCHLD; no migra UI ni ejecuta bootstrap/malloc documental fuera para mover un PID después. La rama AS sólo sirve como alternativa si pasó sus propios gates; indisponibilidad de clone3/delegación/controller no convierte migración tardía en protección acreditada.

COW previo a exec, páginas compartidas y cache de binarios conservan ownership potencialmente externo y no se venden como total físico/committed en memory.current. El source llega mediante recepción privada tras ContainmentReady, sin importar un mapping documental cargado fuera. El probe causal compara explícitamente asignar/tocar fuera→migrar con nacimiento/exec dentro y ensaya nuevas cargas privadas, herencia, exceso y fallos de configuración/clone.

**UX-02 cerrado en diseño.** Permisos, API/kernel/ABI de clone3, seguridad de la rama hija multithread y cobertura real de cargas deberán verificarse en el candidato nativo; la propuesta no los certifica por su firma.

## Inspección adicional de coherencia

- Un parser syntax/structure activo sigue siendo la condición de exclusión. G es auxiliar adicional declarado, no segundo parser ni recurso gratuito. La UI no inicia otro parser mientras falte recibo Reaped y recolección del supervisor.
- macOS propone AS512MiB para G y para P por separado; declara dos techos potenciales además del padre, no512MiB agregado. Capacidad propia auxiliar1MiB es candidato cargado a la cuota propia64MiB y no se confunde con stack/loader/allocator externo. El gate tendrá que medir su viabilidad sin reducir cuotas/source ni ocultar coste.
- Linux AS, cgroup charges y Windows committed mantienen unidades/ámbitos distintos. Cap futuro aplicado a mappings/kernel no demuestra representación propia, RSS ni memoria física total; R2 conserva esta distinción.
- Deadline empieza antes de preparar/spawn/handshake y no se renueva con progreso. UI/restauración100ms objetivo, salida, señal y reap se miden separadamente; SIGKILL no se usa como prueba de reap instantáneo bajo bloqueo kernel.
- AS macOS queda como hipótesis prioritaria con corpus nativo/versión exacta, no soporte certificado. La fuente XNU referencia bsd y VM del mismo tag. Fallo de baseline/startup requiere siguiente defensa e hipótesis concreta revisada; ni Unsupported indefinido ni AS baseline+512 llamado cap512 cierran el objetivo.
- App/companion, libproc/CLT, gramáticas/dispatch Unix, regresión funcional, PTY y QA visual conservan gates independientes. Check/crossbuild no sustituye runtime. Soporte final Windows/Linux/macOS no se reduce.

No se encontraron nuevas críticas accionables en este nivel de diseño. La futura implementación debe materializar salida de G y joins/cierre de lectores dentro del recibo descrito; no basta recibir ACK e ignorar wait(G). Errores/fuga/subproceso residual son fallo de gate, aunque Fuente permanezca usable. Estas son condiciones ya exigidas por R2, no hallazgos nuevos.

## Fuentes primarias y límite de evidencia

[Apple sigaction](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/sigaction.2.html), [Apple wait](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/wait.2.html), [Linux getrlimit](https://man7.org/linux/man-pages/man2/getrlimit.2.html), [Linux clone3](https://man7.org/linux/man-pages/man2/clone.2.html), [Kernel cgroup v2](https://www.kernel.org/doc/html/latest/admin-guide/cgroup-v2.html), [Linux PDEATHSIG](https://man7.org/linux/man-pages/man2/PR_SET_PDEATHSIG.2const.html), [XNU kern_resource.c xnu-12377.121.6](https://raw.githubusercontent.com/apple-oss-distributions/xnu/xnu-12377.121.6/bsd/kern/kern_resource.c), [XNU vm_map.c mismo tag](https://raw.githubusercontent.com/apple-oss-distributions/xnu/xnu-12377.121.6/osfmk/vm/vm_map.c). Consultadas en la ronda R1 de esta revisión; R2 se cotejó con esos contenidos y APIs, no con un host Unix ejecutado.

**SIN_CRITICAS_PENDIENTES** sólo para la propuesta de probes/topología. U-G1–U-G5, contención/runtime/companion Unix, QA y S03/backlog continúan abiertos hasta implementación y aceptación nativa independientes. Ningún soporte se acredita aquí.
