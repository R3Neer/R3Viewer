# A06 — aceptación focal independiente R1

**SIN_CRITICAS_PENDIENTES** para conteo del trabajo real por viewport y referencias prestadas. No cierre S03, QA visual, Unix, coste productor/encoder/IPC, pico de RAM ni SLA.

Plan 7CCC3E7AD5777FB5831798D56F979374D8A5334B051F22F371B64D8CDCC7ED9F; seis entradas originales cotejadas. Copias src/data.rs y viewer_session.rs iguales al canónico al preparar el control. Contrato ACCEPTANCE-R4 conserva A06 como pendiente de conteo independiente, sin nueva matriz.

CodeAE: DataRef contiene referencias/slices; Navigator conserva root prestado y ViewState, cell devuelve Cell::Data prestado. RenderStats incrementa dentro del recorrido de filas/celdas realmente renderizadas; plain llama al renderer productivo con TestBackend. No recorre todo el subárbol por frame ni copia payload: el Arc clone del test retiene la misma fuente, mientras metadatos/paths y presentación segura tienen sus propios costes. La preparación inicial sí escanea filas/campos; no se confunde con viewport.

AE propia: una ejecución exacta del test existente, sin compilación ni modificación de expected, PID20420, wait físico, exit0. Un test PASS, cero fallos/ignored, 518 filtrados. Assertions reales: preparación100000 filas/200000 campos; viewport80×12 visita9/18; resize30×5 visita2/2; preparation.rows_scanned permanece100000. La comprobación posterior cubre rows_scanned, no un segundo assertion explícito de fields_scanned; no se atribuye ese assertion inexistente.

stdout registra un test PASS; stderr registra preparación36.2615ms y viewport9/18. Es tiempo observado de esta ejecución, no garantía ni medida completa del pipeline. Bin copiado SHA541A1D3E001FB8C6392A24A6BF8F37900A8D0D3ABA8C694BEE40B2483C080A13 igual antes/después; todos los inputs de MANIFEST-before-output C8CA1E95A425A8DC33D1322C39FF14E5766E5C2BFF9A77D5A37F5DD42FB38100 intactos. No recompilé biblioteca ni repetí suite. Sin nuevos procesos worker/ConPTY.

Este recibo cierra únicamente la laguna focal A06 del conteo independiente; memoria histórica y demás presupuestos conservan sus evidencias separadas.
