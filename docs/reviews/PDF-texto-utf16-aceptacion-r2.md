# PDF UTF16 R2-final: aceptación independiente del delta EOF

Autor del candidato: unix_r3_review. Aceptación: graphics_gate_experiment. Dictamen: SIN_CRITICAS_PENDIENTES en el delta helper EOF/observación/retención y los controles concretos ejecutados. No constituye aceptación integral de PDF ni de SourceComplete.

Candidato: pdf-utf16-mapping-candidate-r2-final, manifest E3FA1AED161E589A448E250ED12C01E0952B2B2CA67618353A76824A9DA9D435 (1112 entradas), supervisor BB36465CED2EA1BAADF8D68F85D6BDFBF41347799DEAD61C56A030CB479EBA18, bin B34BD1A5CF394025A1DFAB6BA6E9C73EE2430E11D9D812210E516836C4B9D68B. Ejecución en pdf-utf16-independent-ae-r2/candidate, con outputs OWN-artifacts fuera de la copia. No cambios al helper/bin/oráculos originales ni al baseline ROOT-EOF-RESULTS-attempt1.json.

## Expectativas previas y evidencia independiente

OWN-PREOUTPUT.json SHA D233185A7AEB1B14921073DFEF92793E5440C2164E0F0BE470063526AEBA5C07 se fijó antes de outputs; OWN-RUNNER-PREOUTPUT-MANIFEST.json fija scripts iniciales. El PDF se contrastó por su árbol de páginas (3) y streams originales descomprimidos: alpha alpha, beta beta y página gráfica. No se generó golden desde candidato. Peer propio valida descriptor/source/hash y observa EOF real en stdin antes de emitir Failure de protocolo; V original valida ese wire. Se inspeccionó supervisor completo y wire_gate.rs/main.rs primarios del snapshot.

OWN-RUN-attempt4.log y OWN-RESULTS.json: 12 familias PASS, final un hilo, Pending falso y 0 rescates. Todos los procesos transferidos al helper terminaron con wait_proof, Reaped, handles/streams cerrados y threads unidos; Job exacto leído 536870912 bytes. El runner auditó además todos sus Popen retenidos.

| Control | Evidencia observada |
| --- | --- |
| PDF real | Native+V validan, ReadableLayer; PageBegin kind11 informa 3 páginas; semantic Unverified |
| Fuente corrupta propia | Native+V validan diagnóstico ContentFailure, sin confundir éxito de protocolo con contenido legible |
| EOF real | Peer independiente lee EOF, stderr incluye hash de fuente; Native+V validan |
| READY y wire juntos | early_response sticky, primer byte wire0, NotStarted y stdin abierto; no V |
| Primer byte temprano | Byte R basta para latch InFlight/abierto, incluso sin encabezado completo; no V |
| close retorna sin cerrar | Failed sticky; cleanup no lo convierte en Succeeded; no V |
| close lanza antes de cerrar | Failed + error de escritor; cleanup completo; no V |
| ClosedInFlight | Peer sí lee EOF físico; observer ve InFlight y closed=True y rechaza conservadoramente. No afirmamos envío antes de EOF |
| Observer antes de lock | Bloqueo propio de Capture.lock; observer registra previous6/early=True mientras lock sigue retenido, antes de copy/hook |
| Fragmentación | READY byte a byte y wire en 6 fragmentos, después de EOF; Native+V aceptan |
| V temprano propio | Native real acepta; V observa wire0 InFlight/abierto y rechaza antes de publicación |
| Retención GCdisabled | 3 solicitudes reales, sin gc.collect; 0 referencias weak a capturas Native vivas después de cada solicitud; LAST V intencional y cerrado |

## Intentos y fallos propios preservados

Attempt1 falló en una comprobación posterior al control PDF exitoso: confundió counts[0] de caracteres con páginas. Attempt2 corrigió campo pero apuntó a Boot kind10 en lugar de PageBegin kind11. Ambos scripts/logs/resultados están preservados. Fuente wire_gate.rs emit_control declara doc_pages en PageBegin payload offset8; el oracle original de 3 páginas permanece. Attempt3 pasó diez familias pero falló el control V: Event propio ya señalado por Native dejó pasar cierre de V, que fue rechazado por hash sin probar latch. Se preservó y se corrigió exclusivamente Event propio en lanzamiento del target; Attempt4 prueba latch temprano V. OWN-ASSERTION-CORRECTION.md registra todo. Ningún error propio se atribuye al candidato ni se oculta como green. En todos estos intentos ejecutados los procesos ya estaban Reaped/allclosed al fallar comprobación; no se usó rescate.

## Reproducción y límites

Receta ejecutada: C:\Python314\python.exe -X utf8 pdf-utf16-independent-ae-r2/OWN-runner.py. Scripts usan copias nuevas, Job nativo, capturas acotadas del candidato, barreras propias de 1s y deadline 2s para cierres fallidos/10s demás. Una nueva ejecución reutiliza archivos OWN solamente; preservar intentos antes de rerun. OWN-verify.py contrasta SHA/tamaño de cada entrada de original y copia y produce OWN-FINAL-VERIFY.json; OWN-EVIDENCE-MANIFEST.json identifica evidencia exacta.

No se ha recompilado Rust ni repetido su matriz semántica R1, no se ha demostrado atomicidad temporal de transmisión remota ni llamada nativa bloqueada, no se ha hecho QA visual/terminal/Unix ni producción. La garantía observada es respuesta local observada solamente después de cierre confirmado; rechazo ClosedInFlight es conservador. Texto/ligadura/Unmapped/generated/CMap/truncamiento preservan semantic Unverified. Queda vigente la aceptación semántica anterior con sus límites, no extendida por esta AE.

Verificación final ejecutada: original 1112/1112 y copia 1112/1112 byteexactos. OWN-FINAL-VERIFY SHA D48467E6749D8515A543AE537BBEDA9D155176DDE5ED15D5142B01D9F3EE428F. OWN-RESULTS SHA 7F94836DB658C77EFD312D66DE05CFF819EC2168252675DF51909839DCB61F1A. OWN-EVIDENCE-MANIFEST SHA 102DB24581BEC898C9809D0F16514989769B8D42BE252A64BD4D45DB2104A996. Runner final C407B922888C38A695FDFD4033EDF1340F78C647AB39C7EE6C007CE0E700C5CE; peer 70EB52921DC5A6CAB78494DBEC5F876D74EEB0297D2A2E48D5A77DBE9D1FDF59. Sin sesiones/checks propios pendientes.
