# Observación privada G4 R3Viewer XML

2026-10-05. Candidato externo congelado: C:\Users\Usuario\AppData\Local\Temp\r3viewer-xml-gate-r1\target\debug\r3viewer-xml-gate.exe. No cambios de fuentes, producción, repositorio o commits.

## Evidencia disponible

- ETW StartTraceW privado de nombre UUID, sin providers habilitados, sin consumidores ni eventos, devolvió 5 (ERROR_ACCESS_DENIED). Token nativo: SeShutdownPrivilege, SeChangeNotifyPrivilege, SeUndockPrivilege, SeIncreaseWorkingSetPrivilege y SeTimeZonePrivilege; no SeSystemProfilePrivilege. Sin UAC ni cambios de permisos/políticas.
- logman query providers confirmó Microsoft-Windows-Kernel-File GUID EDD08927-9CC4-4E65-B970-C2560FB5C289 y Microsoft-Windows-Kernel-Network GUID 7DD42A49-5329-4832-8DFD-43D979153A88. Consulta solo de metadatos.
- Debugger propio independiente ejecutable: owned_nt_monitor.py. CreateProcessW DEBUG_ONLY_THIS_PROCESS (sin attach a PID ajeno), WaitForDebugEvent, ReadProcessMemory/WriteProcessMemory propios, GetThreadContext/SetThreadContext propios y NT-export breakpoints funcionaron con token estándar.
- Control positivo Rust own file y TcpStream hacia listener propio 127.0.0.1: PID8020 exit0; observó NtCreateFile de owned_positive.txt, NtCreateFile de Device/Afd/Endpoint, connect familia2/IP127.0.0.1/puerto57945; listener recibió OWNED_POSITIVE_CONTROL. Archivo owned_positive_result.json. Ensayo previo dentro del AppContainer detectó intento connect pero red bloqueada, exit101 sin listener recibido: NO se usa ese ensayo como control verde.
- Negativos: baseline, DOCTYPE URL example.invalid, DOCTYPE file canary, PI stylesheet file canary, XInclude file canary, namespace example.invalid y xml:base propio. Siete procesos exit0. Cada uno observó solo un NtCreateFile hacia su fixture; ningún open del canary externo, ningún AFD open, ningún connect. Archivo owned_negative_results.json. Runtime produjo cuatro NtOpenFile redacted por proceso y IOCTL varios: estos NO se interpretan como red.

## Alcance real y límites

Es observación dinámica independiente en frontera de API NT de proceso propio, no prueba ETW del kernel ni observación exhaustiva de todos los syscalls. Los breakpoints se arman en breakpoint inicial del loader, antes de main/lectura del fixture. NtCreateFile, NtOpenFile y NtDeviceIoControlFile se observan siempre; connect y WSAConnect se arman cuando carga ws2_32. Ninguna DLL ws2_32 cargada en candidato implica que los resultados negativos de red dependen de no AFD open y de las APIs NT observadas; el positivo sí cargó y ejecutó connect.

No cubre syscalls directos que eviten exports, accesos vía handles previamente existentes, ni hijos (DEBUG_ONLY_THIS_PROCESS). Los handles no se heredan. El candidato inspectado tiene main que lee un fixture y llama parser, sin creación de hijos. No se ha instalado job limit para impedir hijos: no afirmar protección de todo un árbol. Hay una ventana de un single-step con breakpoint temporalmente retirado: no certificar exhaustividad en proceso multihilo. Este candidato/positivo ejecutó I/O observado en hilo principal; ausencia de eventos en otros hilos no prueba ausencia de actividad durante dicha ventana.

Máximo 20 segundos/proceso, máximo256 eventos; timeout/límite/fallo termina el proceso propio y espera hasta2s su finalización. Success procesa EXIT_PROCESS_DEBUG_EVENT. No fuga de sesión ETW: StartTrace falló y no creó ninguna. Resultado por proceso con PID propio, máximo256 eventos. Solo se conservan paths bajo carpeta propia o AFD, otros nombres redacted; no contenido de archivos ajenos, paquetes ni memoria ajena. No enumeración de apps/PIDs ajenos ni capture de máquina.

## Reproducibilidad

python run_owned_positive.py
python run_owned_negatives.py
python etw_feasibility.py

Todos scripts y outputs están en esta carpeta. El positivo necesita ejecución estándar fuera del AppContainer para permitir loopback; no elevación UAC. Se compiló owned_positive.rs con toolchain privada R3Viewer, importando scripts/dev.ps1 que solo define CARGO_HOME/RUSTUP_HOME/PATH del proceso.

## ETW: receta condicionada, no validada aquí

Crear child propio suspendido; conservar handle/PID; crear logger de nombre/GUID frescos y buffers acotados. Antes de habilitar cualquier provider verificar que su filtro realmente impide incluir PIDs ajenos. No volcar ETL ni iniciar consumer antes de validar scope, no fallback global. EnableTraceEx2 con EVENT_FILTER_DESCRIPTOR Type0x80000004 Ptr a PID DWORD y Size4 no basta para afirmar aislamiento de providers del kernel: la documentación indica que el PID scope habilita solo providers user-mode, no registro del mismo provider por driver kernel. Un logger de nombre privado NO es sinónimo de private in-process logger ni de aislamiento de datos por proceso. Sin privilegios y sin validación de kernel-PID scope, mantener G4 kernel-observation abierto; conservar esta observación NT como evidencia parcial.

Fuentes primarias consultadas:
- https://learn.microsoft.com/en-us/windows/win32/api/evntrace/nf-evntrace-starttracea (permisos de StartTrace).
- https://learn.microsoft.com/en-us/windows/win32/api/evntprov/ns-evntprov-event_filter_descriptor (PID filter, limitaciones kernel y máximo8 PIDs).
- https://learn.microsoft.com/en-us/windows/win32/api/evntrace/nf-evntrace-enabletraceex2 (configuración y filtros).
- https://learn.microsoft.com/en-us/windows/win32/debug/process-functions-for-debugging (DEBUG_ONLY_THIS_PROCESS y memoria/contextos de debug).
- https://learn.microsoft.com/en-us/windows/win32/api/debugapi/nf-debugapi-waitfordebugevent (eventos/handles).

## Cierre canónico del criterio existente

La revisión independiente posterior distingue este monitor NT del experimento ETW no disponible: el criterio G4 canónico no exige ETW kernel y queda cumplido para el prototipo exacto y siete inputs observados. [Revisión y hashes](../reviews/S03-XML-gate-recursos.md). Las expresiones anteriores de G4 kernel-observation pendiente se conservan como investigación opcional, sin extender ni alterar el criterio canónico. No hay garantía universal, sandbox, aceptación de producción ni cierre G5–G7. Cambiar parser/features/resolutor requiere renovar comprobaciones de recursos.

Ubicación original de scripts/JSON mencionados como «esta carpeta» en el informe: C:\Users\Usuario\.codex\visualizations\2026\10\04\01a10823-9b5a-78d3-8c6a-c64ff5709d5f. No se copian ni instalan herramientas de captura en el producto; hashes de sus bytes en la revisión final. Las instrucciones de reproducción se refieren a esa evidencia original, no al directorio docs/research.
