# S03 C — aceptación independiente del candidato R1

2026-10-05. Ejecución propia sobre snapshot congelado; funcional Windows x86_64/Nushell 0.115.1/ConPTY. No cambios en producción, perfiles, docs canónicos ni backlog. No acredita apariencia ni macOS/Linux. A/B se comprueba solo en interacciones afectadas.

## Identidad

- Snapshot: Cviz/r3viewer-s03-c-review/R1.
- SNAPSHOT.json: 9E7095A746EAF750C1DB35853C351A22794988A599452D4D8F77576A9FCEF959.
- App: 9AB78E023BD7FE2CE4BECBC7E136BC7274925B9454F5193026A19883B509B30C.
- Companion: 0B9B04E6B7D1AC4ABCC871D99576BDEA5805AA4E0759619CA3E61E17BF488184.
- Módulo y binarios también hashados en r1-redirect-evidence.json.

## Resultado y fronteras

No se ha confirmado defecto del producto. Los 16 casos CLI/redirección pasaron. Los 10 TUI iniciales, 7 TUI adicionales y 4 TUI de origen/viewport pasaron. Inline corto, números, final sin LF, gates, límites y corpus All se ejecutaron; ver límites del decoder abajo. Los códigos de salida y restauración se observan desde la sesión Nu propia. Máximo observado1worker de sintaxis; tras cada cierre no quedaban app/worker propios. Muestreo25ms no demuestra ausencia de procesos cuya vida completa dure menos que el intervalo.

La primera ejecución extra terminó por expectativa incorrecta del harness: open --raw produce Unknown, no Binary. Se conservó transcript, se corrigió la expectativa y se repitió el corpus completo. No es hallazgo del producto. La comprobación de Binary usa explícitamente corpus neutral de wire propio; no se presenta como productor Nu Binary real.

## Matriz SC01–14

| Criterio | Evidencia independiente | Límite / pendiente |
|---|---|---|
| SC01 | PASS matriz principal: Text/language y selector vacío/largo code2 antes de config inexistente; real .rs/.RS; pipeline metadata transformada no reopen; pipeline homónimo literal; --path abre archivo real; py explícito sobre .rs; contenedor conserva estructura; Unknown/Binary neutral no child; String CSV explícito sí. | No exhaustividad de todos los nombres de gramáticas. |
| SC02 | PASS roles observados con paleta única: Success de string Rust; Process al final multiline; desconocido y Code literal sin idioma plano0 con diagnóstico; archivo gramática como token no se carga. | No revisión visual de cada scope/contraste. |
| SC03 | PASS parcial comentario/raw string multiline900+líneas, End/Home conserva worker y rol numérico posterior; búsqueda stream no reinicia worker. | No expulsión controlada de checkpoints/cache ni comparación léxica exacta de cada línea. |
| SC04 | PASS parcial ESC real/C1/literal␛/CRLF/tabs/CJK/combining/ZWJ corpus seguro, control visible y rol String; redirección coincide con fuente saneada esperada. | Intervalos/graphemas exactos y composición búsqueda requieren pruebas modelo/TestBackend; transcript no certifica forma visual. |
| SC05 | PASS UTF-8 de4bytes inicia65535 y cruza chunk64KiB: apertura TUI e inline con worker sin fallo; final sinLF conservado; vacío/redirecciónCRLF seguros. | Líneas muy largas se envuelven en ConPTY: fuente exacta integral de esas capturas no afirmada. |
| SC06 | PASS parcial raíz record/claves sin child; String pane con1child; volver raíz recoge child; n/N mismo String stream conserva PID; End/Home conserva PID. | No estrés completo de resize/pending coalescing/alternancia repetida entre2strings. |
| SC07 | PENDING independiente latched Plain de petición obsoleta y estilos tardíos. | El test de integración del autor syntax_generation_request_and_plain_arbitration_never_revives_obsolete_styles es evidencia distinta; no se ejecutó como AE propia. |
| SC08 | PASS Ctrl-C activo devuelve130; input503/output7/cursorvisible exactamente iguales antes/después. RestauraciónTTY28ms; código observado en Nu1546ms desde cancel, separados. Sesión reutilizada después. | No inducción independiente de cada fase startup/transferencia/All/bloqueoOnig. |
| SC09 | PASS parcial cierre/reap propio en21casos TUI; máximo muestreado1worker; sin acumulación tras salida y pane. | No captura todos los instantes; no estrés repetido worker ocupado ni handles privados. |
| SC10 | PASS65536bytes incluyendoLF elegible: worker;65537 plano sinworker y sinSuccessSGR, salida0. Redirección fuente completa. | Comparación exacta completa TTY de línea larga no afirmada por wrapping. |
| SC11 | PASS fallback real SpanBudget All: diagnóstico demasiados intervalos, fuente íntegra y ceroSuccess; LongLine; idioma desconocido; lecturaCLIprioridad. | Timeouts startup/línea/viewport/All, IPC malformado, Plain antiguo de misma generación no inducidos independientemente. |
| SC12 | PASS All1000 fuente25025B/1001líneas exacta tras decoder cursor-forward, SuccessSGR; All12000 fuente300022B/12001líneas exacta +diagnóstico, sinSuccessSGR; corto/números/finalsinLF/gates; sinAltScreen inline. | Bytes rawConPTY son VT transformado, no bytes WriteFile originales; atomismo antes de escritura respaldado por InlineOutput/core, no inferido solo de captura. |
| SC13 | PASS16CLI: --colour always redir sinANSI/números y preservación; TTY text/never/unknown sinworker niSuccess; overridealways sobre NO_COLOR1 habilita; Unknown/Binary neutral noresaltado. | No garantía de detección de child transitorio<25ms; gate además legible en fuente. |
| SC14 | PENDING medición independiente de memoria/cache/checkpoints y separación fases. | No trasladar cifras A/B a sintaxis/child ni afirmar RSS global512MiB. |

## Nu String frente a otros streams

Productor real `[[body]; ['let message = "hello";']] | to csv | r3viewer encode` guardado en csv-wire.bin. Frame Start MessagePack incluye `origin:{type:byte_stream,value:string}`; bytesCSV32. En TUI --language rs obtiene Success y1worker; auto sinidioma0worker. Después `/hello`, n/N el mismo childPID25088 permanece (app7552). Esto verifica CINT-01 y preservación de identidad del stream al navegar coincidencias.

`open --raw minimal.rs` observado como unknown_stream; proceso externo Python también Unknown, sinchild aunque --language rs. Corpus Binary neutral derivado solo de Start.kind string→binary conservando frames/payloadCSV produce binary_stream, sinchild ni Success. No modificar fuentes ni inferir semántica textual a partir de bytes aparentemente UTF-8.

## Transcripts inline y decoder

El stripping ANSI inicial marcó2casos false. ConPTY reemplazó4/7 espacios de líneas por CSI1C. Un decoder específico de estas fuentes ASCII cortas expande CSI nC antes de stripping. Resultado:

- all1000: body25025B exactamente igual a fuente25025B,1001LF; SyntaxSuccess presente; sin diagnóstico.
- budgetfallback: body300089B contiene fuente íntegra300022B,12001LF;67B extra son diagnóstico; SyntaxSuccess ausente en ALL transcript.

Se conservan r1-tui-evidence.json originales con los false del decoder inicial; r1-inline-decoder-evidence.json los reconcilia, sin borrar pruebas ni tratarlo como defecto. No reutilizar este decoder como emulador VT universal: wrap, backspace, tab, posicionamiento absoluto y graphemas requieren estado. Líneas65536/65537 y UTF-8boundary guardan transcript completo pero no prueba de byte igualdad TTY. Inline corto sí igualdad; redirección todos esos archivos sí byte igualdad/saneado.

## Artefactos propios

- r1-redirect-evidence.json:16casos, stdout/stderr/códigos/hashes.
- r1-tui-evidence.json y r1-tui-full-transcript.ansi: corpus principal, procesos/restauración/inline.
- r1-extra-evidence.json / r1-extra-transcript.ansi: CSV, Unknown, Unicodeboundary, inline números.
- r1-more-evidence.json / r1-more-transcript.ansi: Binary neutral, --path, overrideidioma, multilineEnd/Home.
- r1-inline-decoder-evidence.json y r1-inline-*.ansi:ALLcapturas, resultado refinado.
- fixtures/manifest y scripts reproducibles redirect.py,tui-inline.py,extra.py,more.py,inline-decoder.py,process_observer.py.

Este informe acepta únicamente los subcriterios funcionales observados; no cierra S03 ni RV105/107, ni transforma pendientes en PASS por pruebas del autor. Faltan gates de memoria/fallos controlados, matriz macOS/Linux y QA visual según especificación vigente.

## Ampliación independiente posterior — propagación de fallos y memoria

Se ejecutó failure.py sobre el mismo R1, verificando parentesco Nu18236→viewer14968→worker16640 antes de TerminateProcess(16640,17). Solo ese worker propio fue terminado. Ya se habían observado estilos Success. Resultado PASS: diagnóstico motor terminó inesperadamente, resize completo111x33 conserva fn/message/number y NO emite Success/Accent/Process; /hello+n/N+resize110x32 mantiene plano, mismo viewer, cerochild y q0. Restauración41ms; códigoNu212ms desde q. Esto cubre SC07 latch funcional tras WorkerExit y SC11 propagación integrada real; la variante exacta de petición obsoleta/Styles tardío todavía corresponde a test de estado existente, no a este runtime.

La comprobación Python explícito sí esperó Ready y muestra Accent→CSI2;1H→def en transcript. La regex inicial solo aceptaba SGR inmediatamente antes de def, marcófalse y se conserva. r1-role-decoder-evidence.json confirma keyworddef con cursorposition interpuesto. Es corrección de decoder, no defecto ni nueva ejecución del candidato. Minimal números inline muestra exactamente filas1–4, sin fila5ficticia; transcript completo conservado.

syntax-memory.py midió app y child por separado, ambos hijos verificados propios. Casos de archivos1000/100000líneas,25KB/2.5MB:

| Líneas | Preparación hasta pantalla inicial | App private inicial | Child private inicial | Después End/Home |
|---|---:|---:|---:|---|
|1000|1176ms|1,961,984B|2,740,224B|App private igual; child2,818,048B;1worker, q0/reap|
|100000|1749ms|9,789,440B|5,201,920B|End produce timeout real de resaltado; child recogido, Home mantiene plano sinchild; app9,744,384B, q0/reap|

Tiempos incluyen comandoNu/lanzamiento/app/lectura/modelo; no aislamiento de transferencia, load ni petición. No pretenden hardcapRSS512MiB, heap total exacto ni prueba de cache8MiB/checkpoints32. Memoria SC14 pasa como medición observada parcial, con desglose app/child, no como demostración integral de presupuestos internos. SC11 añade RequestTimeout real integrado con diagnóstico y fuente visible; SC03 caso100k acepta degradación explícita al superar presupuesto, no pierde contexto coloreando un prefijo aislado.

Artefactos adicionales: r1-failure-evidence.json/r1-failure-transcript.ansi, r1-role-decoder-evidence.json, r1-syntax-memory-evidence.json/r1-syntax-memory-transcript.ansi. Las evidencias primarias se conservan aunque una regex inicial haya marcadofalse. No hay defecto confirmado ni reducción del alcance pendiente.
