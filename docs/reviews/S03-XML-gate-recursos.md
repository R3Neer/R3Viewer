# Revisión independiente de evidencia XML G4

2026-10-05. Revisión solo de lectura de producción y candidato, sin ejecución de nueva captura, sin editar criterios y sin aprobación del producto.

## Veredicto

**SIN_CRITICAS_PENDIENTES para XML-G4-EVIDENCE en el gate experimental de este candidato.** El monitor de archivos/red exigido en S03-D4-core sección11 ya existe y no observó cargas de los recursos indicados en los siete ensayos registrados. Su control positivo detectó una apertura de archivo y una conexión real a un listener propio. El criterio canónico no exige ETW del kernel ni una prueba universal de ausencia de todo syscall. No introducir esa condición adicional retrospectivamente.

Este cierre es experimental y acotado al candidato exacto y a los inputs observados. No certifica ausencia universal de acceso a archivos/red, sandbox de seguridad, G5/G6, integración de producción ni aceptación de producto. G1–G3 y semántica envelope G4 conservan el alcance de la revisión anterior; aquí no se reejecutaron sus 245 comprobaciones.

## Identidad comprobada

MANIFEST.json SHA256 94869AFD7ACD4AA787AABFD54B2C4F4D10FB506325B75127F30F1CA17FDA685C: recorridos y recalculados los 237 registros path/hash, cero divergencias. Incluye candidato de 502272 bytes SHA256 265EEC5469FF14E0010768261B402C943CE69414019176C89D6019E6E7D0C623. run_owned_negatives.py usa exactamente ese path absoluto; no recompila ni sustituye el candidato.

Identidad de nueva evidencia, SHA256:

- G4-owned-observation.md: 4AD517D30923ABC260662B7D072E08B9F20F6766BD11D3DC94BB735DA6FCA8DF
- owned_nt_monitor.py: 1E9811C02973FF545E85BF5122077076F04F50F314538D20381804F603C67A9D
- run_owned_negatives.py: 71CE2B83FC5DD52567E5A83A1FED4E7CEFEBA331011E38EA3CCD76C5CE80F098
- run_owned_positive.py: D530E1E105DBF57B677112FDB0DDEAF048C273C8BE2B194AFE76D3E0064761CA
- owned_positive.rs: 27380003D76DCE72B7378D059C341CA1AC13513AA32CCEC6D50644DF7440168E
- owned_positive.exe: 765833D1E1F1862ECFA58F7BE259136635783320B1617EB5AB019F05EC08F330
- owned_negative_results.json: 0BAA57F23AC8DAAAAC173CABCF625F2BBF5BF6C85A28EBCC909D54D1E6A9E065
- owned_positive_result.json: 2692DA1A21EA3D4C2631A4D91300D79C4F2AD937744FD1248591C1F69E68CC93

## Por qué satisface el criterio real

El script crea su propio proceso DEBUG_ONLY_THIS_PROCESS, sin heredar handles; arma NtCreateFile/NtOpenFile/NtDeviceIoControlFile antes de main, y connect/WSAConnect al cargar ws2_32. Se revisaron sus offsets de contexto x64, detección de breakpoint, extracción de OBJECT_ATTRIBUTES y sockaddr, restauración/single-step y lease. Los siete negativos registran exit0, apertura explícita del fixture y ninguna apertura del canary conocido, AFD o connect. Los cuatro NtOpenFile redacted del runtime por proceso no se convierten falsamente en cero actividad de archivos. Los IOCTL genéricos tampoco se equiparan a tráfico.

El positivo PID8020 exit0 registra NtCreateFile del archivo propio, NtCreateFile de Device/Afd/Endpoint, connect IPv4 127.0.0.1:57945 y recepción independiente de OWNED_POSITIVE_CONTROL. Por tanto el resultado negativo no depende de un hook enteramente inoperante ni de un positivo con red bloqueada. El ensayo AppContainer fallido está explícitamente excluido.

Se revisó además la frontera estática actual: main lee únicamente el argumento fixture y llama adapter::parse; adapter usa Reader::from_str sobre slice y read_event, con quick-xml default-features=false y solo memchr en lock. Los módulos adapter/lexical/profile no contienen acceso fs/net, creación de procesos/hilos, FFI o syscalls directos. Esa evidencia complementa la captura limitada, no reemplaza su ejecución. La combinación basta para el objetivo concreto de descubrir carga accidental de SYSTEM/PI/XInclude/namespace/xml:base en este parser de eventos.

## Límites que deben quedar explícitos

No es ETW, observación kernel ni demostración exhaustiva. No cubre syscalls directos, accesos mediante handles preexistentes, procesos hijos ni todos los casos concurrentes; hay una ventana single-step. El redactor retiene paths propios/AFD y oculta otros nombres; un recurso relativo que solo apareciera con nombre sin directorio podría quedar redacted. Los fixtures de canary concretos usan URI absoluta y aquí no hay resolver estático, pero ese monitor no debe reutilizarse como garantía exhaustiva genérica.

Los JSON no guardan hash del binario en el instante de CreateProcess ni salida semántica del candidato; el hash se comprueba ahora y el launcher selecciona el candidato congelado. Exit0 del prototipo no distingue READY de FAIL. Por eso esta evidencia prueba observación de recursos durante ejecución de esos inputs, no una aceptación semántica nueva ni cadena de custodia hostil. No hay indicio de sustitución; exigir firma binaria por proceso o captura de stdout sería mejora de reproducibilidad para futuras ejecuciones, no un contraejemplo a este criterio.

La frase del autor «mantener G4 kernel-observation abierto» corresponde a una aspiración adicional: G4 canónico dice «monitor de archivos/red no observa cargas», sin exigir mecanismo kernel. Se puede conservar ETW como investigación pendiente opcional; no afirmar que se ejecutó ni instalar captura global/UAC para cerrar artificialmente este gate.

## Siguiente frontera

Cerrar únicamente el hallazgo XML-G4-EVIDENCE con esta evidencia y sus límites. El portado de A debe implementar capacidades/ledger/arena/Job/IPC y pasar G5/G6 antes de ReadyXML; G7 y aceptación independientes siguen pendientes. Cualquier cambio de parser/features o añadido de resolver invalida la extensión automática de esta observación a ese candidato nuevo y exige ensayos de recursos nuevamente.
