# GIPC R2 — aceptación independiente

2026-10-05. Dictamen de esta AE: **SIN CRÍTICAS PENDIENTES en el corte privado probado**, incluida la corrección de ownership de GIPC-ROOT01. Este dictamen se combina con la revisión de código separada de raíz; no adopta automáticamente el gate en producción ni cierra G2/RV, todas las cuotas máximas, gráfica de terminal o Unix. R1 conserva su rechazo y evidencia negativa.

## Identidad y oráculos

Candidato MANIFEST F93A15AF6D9AFCD73459BAA8E74081EFB46F7536A02A5B531B4DFF246F44DAB8,383 registros verificados antes/después tanto original como copia `graphics-ipc-independent-ae-r2`. H C6C72D810B48F29E84D50E279B5B9CD4EF2161B31D7422F3C79A37EDB52EAAC3; C BA50BF423EE29BF53AB04AFFA255305DA2EF13BBB883EC1D458B6E7D2CD41088; D D8E533002EE5A7759C933799E6691C0FA4AA32930329DD9BC4522935E3ADB066. Ningún ejecutable suministrado se reconstruyó o modificó. Funciones originales intactas. Oráculos propios R1 OWN-ORACLE/OWN-CONTROL-ORACLE/OWN-ORACLE-DELTA se reutilizan byte por byte, incluidos hashes en OWN-PREOUTPUT-IDENTITY. Sus fuentes propias R1 se leen exclusivamente, sin alterar los originales. Nuevos OWN-R2-CLOSE-ORACLE, OWN-SOURCE-PREOUTPUT-ORACLE y OWN-C-PANIC-ORACLE escritos antes de resultados.

## Binarios exactos:149 casos distintos

Los18 casos de píxeles propios pasan: PNG5x7, crop(1,2,3,4), nearest2/1→6x8; canales independientes, alpha variable/opaca/transparenciafueraCrop; Kitty/PNG/Sixel y preserve/compose. Comparaciones byte a byte, CRC/zlib/EOF/filtros/píxeles PNG y parser Sixel RGB332 independiente, dos bandas y límites de última banda. Alpha straight integer+127 y rechazo de Sixel con target transparente se mantienen. Los119 controles propios anteriores pasan sobre R2 (campos descriptor C/D/requestQ, cuerpo PNG/End/hash/trailing/longitud/enums, handles/nonce/setup, IO/cancel/stale/overflow/stubs y seis barreras de receipt final). No nueva expectativa derivada de output; bytes binarios Kitty válidos siguen permitidos.

Doce controles nuevos usan el descriptor y fuente propios, no el oracle raster del autor. Seis fallos close de aplicación una vez se clasifican FailedReaped: Reference6 único, ceroReady/artefacto, salida0 después de cierre. Seis persistentes se clasifican FailedPending: H permanece vivo reteniendo autoridad, ceroReady/ceroReference/ceroartefacto y exactamente un H_ACTOR_SPAWN; tras estímulo en su stdin y120ms sigue sin admitir otro actor. El watchdog externo propio termina ese H y prueba su wait/IO cierre. **Ese teardown no es PASS de cierre interno ni H Reaped**. No se oculta el fallo persistente con salida del proceso. Son errores controlados antes de CloseHandle, no errores naturales observados de kernel Win32 ni prueba de persistencia de wait en kernel.

## Contraste nativo independiente de ROOT01 y propietarios

Se creó `own-source-probe` independiente con fuentes/Cargo.lock/profile hashes exactos. Únicamente se añadieron módulos de tests al final de H y C; hashes/deltas prueban que cada prefijo de funciones originales permanece byteidéntico. Los tests compilan contra la misma dependencia fijada y tienen cuatro grupos propios, distintos de los16 tests del autor. Los ejecutables test y sus procesos suspendidos son propios y están asignados a Jobs propios antes de cualquier ejecución. GetHandleInformation consulta únicamente esos valores de HANDLE guardados.

1. H: ocho combinaciones de transferencias parciales en orden distinto (stderr primero; stdin movida al tercer IoOwner; readerstdout con o sin stderr). Antes seis handles son válidos (tres streams, proceso, hilo y Job); después de Authority.cleanup=true todos inválidos y Child/all_io/Job cerrados. No se acepta Drop como prueba previa.
2. H: dos fallos close persistentes simultáneos en input e IoOwnerstdout. Cleanup no termina y ambos handles permanecen vivos; recuperar sólo input no basta, stdout sigue vivo y autoridad no acredita all_io_closed. Recuperar el segundo es teardown explícito: todos cerrados y fallo sticky/unclean permanece. Ningún PASS de cierre persistente.
3. H: DuplicateHandle candidato de C rechazado conserva owner, sin `d_validated` ni obligación `d_reaped`; cierre persistente conserva raw vivo y autoridad Pending. Teardown cierra ese duplicado, sin haber esperado/matado al candidato como D y con error irreversible. Job termina sólo su miembro propio.
4. C: writerTask entra en panic sosteniendo mutex de IoOwner; las tres IO siguen vivas bajo Authority después de terminar el Task. Cleanup prueba join, kill/wait propio, cierre native de tres streams, owner cerrado, poison/unclean y sticky failure. El panic esperado es parte del test, no una compilación/ejecución fallida ocultada.

Los cuatro grupos pasan. Este contraste comprueba cierres reales y conservación de ownership en los escenarios descritos; no afirma haber forzado fallos naturales de APIs Windows, todos los interleavings o un mutex de codec bloqueado de forma irrecuperable.

## Intentos fallidos preservados

Historia R1: primer positivo default sandbox timeout reader_missing_receipt y oráculo incorrecto c-body Kitty permanecen en ae-history-r1, con la corrección fundada previa a nuevas salidas. Los oráculos definitivos se mantienen, sin reinterpretar el timeout como éxito.

R2 close intento1: después de observar correctamente Pending.stdout, el helper exterior termina H y finalmente cierra, pero latchea TimeoutExpired en su primera espera50ms; el driver rechaza esa invocación. Se conserva script anterior, tres receipts previos y OWN-R2-CLOSE-FAIL-DELTA. No se atribuye el retraso del teardown al código de candidato ni se elimina el FAIL del harness. Driver intento2 registra teardown deliberado, termina sólo su H retenido, espera2s con prueba explícita y luego recoge IO/joins/handles; las12 clasificaciones se repiten sin errores del helper. Nunca se transforma FailedPending en PASS/Reaped de H. No rescate adicional fuera del watchdog propio definido.

## Cotas y límites

Partición fija H40+C80+D136=256MiB y guard de allocator old+new se conservan. Los medidores pequeños H observados están bajo40MiB; peak y muestras constan en OWN-SUMMARY. No son estrés de máximos simultáneos, aceptación de todas las capacidades de decoder ni prueba del margen native/stacks/committed. El corpus de152 pruebas del autor y sus cuotas source64/RGBA24 no se renombran AE propia. Requested/RSS/Job512 POR PROCESO siguen distintos. RGBA24 privado no satisface producto64. No terminal/framing, cache/UI/coalescing/SLA, Unix, publicación o instalación de perfiles se realizaron.

Aprobar este corte privado exige además dictamen de código raíz; la integración y el backlog completo quedan abiertos. Evidencia exacta con todos intentos en AE-EVIDENCE-MANIFEST.

Observación propia H peak 338267 bytes en 18 muestras pequeñas.
