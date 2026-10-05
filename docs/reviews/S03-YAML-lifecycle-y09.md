# YAML Y09 — deadline, exclusión y configuración del Job

Dictamen: PASS en los casos ejecutados sobre el exe congelado R3 original. Cobertura funcional Windows mediante ConPTY directo; no QA visual, aceptación Unix, integración Nu adicional ni cierre de RV-106.

## Identidad y alcance

MANIFEST.json SHA472C490C51152D80A6FF76393A46BFD2D4BCE3DC41ABD10AEDCB46E4093EFBF3: 337 entradas verificadas byte/hash, ninguna divergencia. r3viewer.exe SHA29EBF22FF82B03FDBF039B46FBD086E766EB3F67BB4B0945E796F0E3C5F6B15A; companionE885DE7F826DA9520EA7E6A11139F57F966A9BDB4E9CFEB5653A6BFDB6D5567E verificado pero no ejecutado en Y09. Sin cambios de producción. Script derivado privado del arnés ConPTY existente, sin GUI ni perfiles. CreateProcessW crea directamente r3viewer.exe con fixture --format yaml --language yaml --colour always. No Nu, plugin ni invocación del módulo.

Fixture regenerable: b'|\n' + (b'  '+b'x'*60+b'\n')*120000; 7560002 bytes. Las líneas cortas permiten worker de sintaxis real. Se usan exclusivamente procesos descendientes de este app propio. Antes de cada NtSuspendProcess se revalidan parent PID, executable/command --syntax-worker o --structure-worker y creation FILETIME, con handle al proceso. Watchdog restaura sólo PID+creation coincidentes; procesos suspendidos son eliminados por el producto durante el experimento, por lo que no fue necesario NtResumeProcess en las ejecuciones PASS.

## Ejecución completa y evidencia principal

complete.py / complete.json / complete.transcript.ansi conservan la ejecución final, con app16808, syntax11604, structure26056 y syntax reanudado44944. El arnés posee handles de los dos primeros workers hasta consultar fin de proceso; no infiere exclusión únicamente por muestreo.

1. Alt-screen alcanzada, sintaxis activa identificada y suspendida. Tab solicita estructura mientras la sintaxis no puede cooperar. GetProcessTimes da fin syntax134356670932276890; creation structure134356670932316134: syntax terminó3,9244ms antes de existir structure. WaitForSingleObject(handle syntax,0)=WAIT_OBJECT_0. Esto acredita exclusión material en esta transición sometida a fallo; no es prueba universal de cualquier interleaving.
2. Structure propia suspendida de forma efectiva; el visor vuelve a Fuente con “interpretación estructural agotó su plazo”, sin Ready parcial. Observación2,071773s desde Tab y2,011780s desde suspensión; no corresponde al reloj interno exacto de arranque ni a SLA universal. Incluye scheduling y lectura/dibujo de pantalla del arnés. No queda structure en la observación de deadline.
3. Doble Tab no reintenta estructura tras Timeout; la Fuente sigue mostrando diagnóstico y vuelve sintaxis, identificada por --syntax-worker. Fin structure134356670952281262 precede creation syntax134356670953394367 por111,3105ms; handle structure señalado. Acredita exclusión inversa en este caso.
4. q termina código0, emite salida de alt-screen y no quedan r3viewer app/workers propios. close del arnés no fuerza cleanup. Esta comprobación no es auditoría exhaustiva de todos los procesos del SO.

## Job observado por handle del app propio

NtQueryInformationProcess(ProcessHandleInformation) enumera sólo handles del app propio; DuplicateHandle copia cada handle para QueryInformationJobObject sin NtQueryObject ni inspección de memoria ajena. Se selecciona el objeto que contiene el PID del structure propio. Se consulta JobObjectExtendedLimitInformation y JobObjectBasicProcessIdList.

Configuración efectiva devuelta: ProcessMemoryLimit536870912 bytes; LimitFlags8448 (0x2100 = JOB_OBJECT_LIMIT_PROCESS_MEMORY | JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE). Owner del handle consultado es app16808, worker26056 miembro. ListaJob[26056,28000]; snapshot Toolhelp limitado a estos PIDs acredita 26056 r3viewer.exe parent16808 y28000 conhost.exe parent26056. El segundo miembro es descendiente del worker propio; no se asumió que un Job contiene sólo un proceso.

Esto supera IsProcessInJob genérico: identifica un handle retenido por el app con la membrecía y la configuración concretas. No prueba asignación justo antes de Init ni aplicación del límite mediante una allocation forzada, no acredita memoria agregada512MiB de app+worker, ni atribuye JobMemoryLimit512MiB: el flag observado es por proceso. El escaneo no enumera handles de ningún app ajeno, no activa ETW, no altera Jobs del sistema y no solicita privilegios extra de Windows.

## Intentos y límites preservados

- direct.py inicial empleó quoted ASCII de línea8MiB. Producto cayó correctamente a texto plano por línea>64KiB; no había syntax worker. Es limitación de fixture, no fallo. Error del arnés llevó a close forzado de su app propia.
- multiline.py usó línea corta y logró transición syntax→structure, pero una guarda comparaba el dict completo del observer, incluido threads. Structure pasó de1a5threads; assert falló antes de suspenderlo. Error del arnés, no producto. Se corrigió guarda a PID+parent y se conserva intento y cleanup forzado.
- bounded.py consiguió timeout efectivo2,055178s desdeTab, sin Job consulta ni FILETIME de fin. q0/cleanup no forzado.
- verified.py añadió FILETIME de exclusión y Job; PASS2,055069s. Su segundo miembroJob29504 no fue identificado en aquella ejecución; complete.py añadió metadata Toolhelp de miembros y transición inversa para resolver la ambigüedad en una ejecución nueva. No reconstruye identidad pasada ni la presenta como confirmada.

No se detectaron defectos de producto en Y09. Arena persistent+ownscratch64MiB total e items1M exactos siguen pendientes; configuraciónJob no sustituye dichas comprobaciones. También pendientes estrés de aplicación del límite, QA visual WT/WezTerm, runtime/contención Unix y resto de backlog. Y09 se limita al camino exe/ConPTY; la aceptación Nu previa sigue siendo evidencia separada.

## Procedencia de esta copia

Informe original SHA87F7F302609C7581735C671FCEFDB54F5219A3C0BAB83F4778FFECD4AFDBCC25; scripts/JSON/transcript originales en C:\Users\Usuario\.codex\visualizations\2026\10\04\01a10823-9b5a-78d3-8c6a-c64ff5709d5f\yaml-lifecycle-y09. ARTIFACTS.json SHA427634245C92319E4A0576A647FAB81752B16DA905F6A4E127F0103F0A36082B. Los nombres de scripts anteriores se refieren a esa carpeta, no a docs/reviews. No reproducir contra el candidato integrado nuevo sin registrar hashes y diferenciar la ejecución anterior.
