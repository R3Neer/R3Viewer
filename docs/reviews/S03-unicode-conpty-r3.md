# S03 Unicode: aceptación independiente ConPTY R3

2026-10-06. **SIN_CRITICAS_PENDIENTES para los seis controles funcionales delimitados CP01–CP06 con harness R2.** No es cierre global de Unicode, RV, QA visual, rendimiento ni Unix. El primer fallo de harness R1 se conserva.

## Candidato y preservación

Padre `unicode-integrated-r2/MANIFEST.json` SHA256 **56EA09960104D701CB906E752CD403A7BA400FA80481EA8B75EFEBDBD8886F95**, 528 registros. Delta `unicode-nu-key-root-r3/DELTA-MANIFEST.json` **2D524BDCC68B7AE59C461CB6151917FC970B2F1AF321E91A09D8C2A442FE40A7**, 10 registros. Verificados todos antes y después: **538 registros, cero discrepancias**. Copia ejecutada `bin/r3viewer.exe` **66D017DC93E09C4142DD9D316E9B0C4C3E3A3C4D51947148C5B9168860600582**. Overlay fuente producción PRODUCT-viewer_session.rs **09D3324D63775E6E844FFE23145F60A28E33190AF7C40702144CA77F0F848413**; la fuente repo/ de la delta es harness, no ese overlay. No rebuild ni workspace compartido.

Oráculo funcional previo `unicode-conpty-independent-r1/EXPECTED-before-output.json` **C4F7DF1C60CB6DAD9636A076C352597A1378F5253CB107245E5BCB336DABB8B8**, copiado byteexacto a R2. Ajuste de observación y persistencia, fijado antes de R2 outputs: `unicode-conpty-independent-r2/REPAIR-and-EXPECTED-before-output.json` **CF0B07D3E6287AA0CB16B5BF2FC23649383E68D0DD67115481DBC722279BFACB**.

## Resultados propios

| ID | Expectativa y observación |
|---|---|
| CP01 | Source22 000B con cluster gigante y 64 filas. Frame inicial de referencia; End revela AFTER-CONPTY-OWN-63; q retorna0. Match referencia raw409..437 después alternate-enter102..110; AFTER raw1406..1425 observado después de enviar End y reiniciar cursor. |
| CP02 | Invocación tiny siguiente en el mismo ConPTY; marcador raw1998..2022 después OWN-START:2 raw1525..1536; q0. Primera y segunda restauran input/output **503/7**, idénticos al baseline. |
| CP03 | Source **33 554 432B**, hash **27CD8C2ED5CEB2AEB24880E5554BC0071C1BCDF8CF3FF383EA7B014CFD4E3ED4**. Testigo child-alive/raw input496 frente baseline503; byte03 enviado sólo después. Retorna **130** sin kill para lograr el resultado; input/output restaurados503/7. |
| CP04 | Después de CP03, misma consola, tiny actual raw1377..1401 después START2 raw904..915; q0, restaura503/7. |
| CP05 | Inyección previa al primer reader.start: funcional rechazado y error sticky; cleanup completo. No PASS de producto ni rescate convertido en resultado normal. |
| CP06 | Overflow inyectado en captura: rechazo sticky, sin respuesta funcional aprobada; cleanup completo. |

Cuatro grupos ejecutados (dos contienen invocación siguiente): **4/4 expectativas**, seis IDs; no seis ejecuciones independientes ni batería de panes. Capturas funcionales R2:2546 y1925B, cap1MiB. Controles negativos0B. Todos los grupos H: wait/closed/joined, JobActive0 después cleanup, Reaped, Pendingfalse, main thread1. Los positivos además C: HPCON retornó, lectorEOF, writer/reader joined, todos sus handles cerrados, archivo de captura cerrado; D: proceso esperado wait/reap y handle cerrado, observer joined y handles de modos cerrados.

Modos consultados sobre handles propios reales **CONIN$/CONOUT$**, abiertos dentro del actor conectado a ese ConPTY; no sobre stdout del receipt H/B. No se usó Nu, plugin, perfil ni configuración del usuario. Config vacío propio y formato text explícito.

## Fallo R1 conservado y corrección demostrada

`unicode-conpty-independent-r1/RESULTS-attempt1.json` conserva CP01/02 agotando30s: H contuvo el Job y probó Active0/Reaped; faltó recibo C y **no se aprobó**. CP03/04 y controles negativos de aquel intento se conservan también.

Causa concreta del matcher: el dump CP03 anterior contiene `referencia ESC[1C de ESC[1C viewport`; no contiene la secuencia literal ASCII con espacios buscada por hostR1. El test source-only sobre ese dump mantiene literalIndex=-1 y muestra el nuevo match. R2 admite exclusivamente espacio ASCII o CSI1C entre esas tres palabras, conserva offsets raw y rechaza CSI2C/Home. Cursor se reinicia por invocación/navegación; comprobación adicional source-only fuera del cursor rechaza marcador viejo. No se reconstruye una pantalla ni se eliminan arbitrariamente CSI para fabricar el texto. Mismo oráculo y deadline30s; no repetición de una carrera de timing ni aumento de plazo para obtener verde.

R2 persiste captura limitada por bloques<=4096 durante lectura, con write/flush/close sticky; cierre del archivo ocurre después del join del reader y antes del recibo final. El fallo anterior, binario y snapshots no se modificaron.

## Ownership y límite de la evidencia

Envelope driver/common/bootstrap HTML R2 copiado byteexacto e inspeccionado: Job512MiB por proceso, flags exactos0x2100, readback/membership antes START; no cota agregada512MiB ni ActiveLimit2. C retiene inmediatamente handles/attribute-list/threads y ejecuta cleanup en finally. Lectura permanece independiente durante ClosePseudoConsole; si este cierre bloqueara y H tuviera que contenerlo, el caso quedaría FAIL. Controles CP05/06 muestran rechazos y cierre bajo este envelope, no una aprobación general de todos sus posibles fallos. H Pending impide otro spawn; ninguno de estos cuatro grupos terminó Pending.

Buffer PTY fijo1MiB, bloque4096, a lo sumo copia final1MiB; cola input16, mensajes<=64B, acumulado<=4096B; capturas H/B siguen acotadas. Son requested/knownbuffers del harness, no una medición de heap/RSS ni garantía global. Deadline30s global de caso, sin renovarlo por progreso, cleanup2s por capa bajo H. Python/conhost son infraestructura de prueba; ningún PID ajeno fue señalado.

CP03 es **cancelación interactiva sobre fuente32MiB después de frame de referencia**: en R2 la referencia raw409..437 precede testigoRAW raw693 y envío03. CPU42343750ticks100ns acumulada no localiza una función. **No acredita cancel dentro prepare ni inicialización Source anterior a TerminalGuard**. Se mantiene diferenciada de la AE de funciones previa. Tampoco explica la causa de3695.62s histórico ni WorkerExited; no SLA. Sin Unix ni revisión visual de terminal/panes.

## Evidencia congelada para revisión raíz

Carpeta `unicode-conpty-independent-r2/MANIFEST.json` **E3376697C4AD457F79E4DB536B80FAA3B202C9CA1BF2D5692437B8F748A84F43**,25 registros: fuentes harness/bin/fixtures/oráculos/outputs. `RESULTS-attempt1.json` **A0434F2485DEEBB8BC1DBB882A84F20DE03465A1F72415484B229901710C8091**. `FINAL-RECEIPT.json` **1FAAE34A14787812CB729D9515DB7E559976A2355E3238758D2A6960A91FACC3**. El root puede revisar código del harness y estos recibos sin editar los oráculos. R1 separado permanece íntegro.

Fuentes Microsoft consultadas para el cierre: [ClosePseudoConsole](https://learn.microsoft.com/en-us/windows/console/closepseudoconsole), que exige cerrar o drenar output y no devuelve estado, y [creación de sesión](https://learn.microsoft.com/en-us/windows/console/creating-a-pseudoconsole-session), que recomienda servicing independiente de canales. Estas fuentes sustentan el harness; los resultados funcionales proceden del binario exacto y capturas propias.
