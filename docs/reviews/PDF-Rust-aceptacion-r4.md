# PDF Rust R4: aceptación independiente del delta de receipts

**SIN_CRITICAS_PENDIENTES en la AE del delta privado de lifecycle/I/O R4.** Se cierra PRUST-ROOT02 con reproducciones propias de receipts tardíos; PRUST-AE01 permanece cerrado por las regresiones de Pending/retry. La aceptación semántica integral PDF continúa **FAIL/no acreditada**: esta AE no convierte native-indexed-layer Complete en contenido completo ni cierra PDF G0/RV-204/producto/Unix/visual.

Snapshot `pdf-rust-candidate-r4/MANIFEST.json` SHA256283D501428075005723D6476D646D18571260B1BBFC78986CF27E941D72B6381,949 entradas de tamaño/hash verificadas antes y después, intactas. Delivery48BCC543B2ED94380F3E4742F891EBEF305B76B4D5F50547A4F5345FE3F6B151 comprobado. Bin C8C6717A2161E23ED1A7B0D548C6008CB07DEC80F36D28FFC1A46B0E4C7D7B73 y todo src coinciden con R2/R3. Se inspeccionó helper antes de ejecutar y se trabajó sólo en copia propia `pdf-rust-ae-work-r4`; no rebuild/Cargo raíz, producción, perfiles ni remoto.

## Receipts propios posteriores al join

Ocho controles propios pasan como fallo terminal con reap: error EOF tardío y overflow tardío en stdout y stderr; writer.write con OSError; writer.write con RuntimeError (panic); flush fallido; close tardío del writer. Las barreras independientes liberan el error de lector al unir su thread y el close del writer al unir el writer. No dependen del momento habitual de scheduling ni de las salidas del autor.

**Todos los ocho procesos nativos terminan exit0**, pero ninguno devuelve validated_response: se conservan los errores finales y se rechaza el éxito. Wait proof, handle, tres streams y threads quedan cerrados/terminados, estado Reaped, sin Pending residual ni rescate externo. El write/flush de los proxies reenvía el contenido real antes de fallar; el close tardío cierra primero stdin para permitir respuesta nativa completa y demora únicamente el receipt. Esto distingue el rechazo de una simple terminación previa sin respuesta.

Así se resuelve la publicación falsa R3: muestrear errores después del join permite recoger la excepción EOF/overflow final; writer captura write/flush/panic y close por separado y el helper valida estos receipts antes de publicar. Reap correcto no borra el fallo de procesamiento. Evidencia propia `pdf-rust-ae-receipts-r4.py` y `pdf-rust-ae-receipts-r4-results.json`.

## Regresiones y semántica preservada

Se repiten39 controles propios R3 sin cambiar expectativas:15 setup/API faults, siete Thread.start/reader/streamclose/diagnóstico y17 rechazo/cancel/panic/overflow/stub. Todos pasan en su política esperada, sin rescates. Wait/kill persistentes conservan Pending fuerte, bloquean otro Popen (cero nuevos hijos), y retry explícito del mismo actor tras restaurar métodos obtiene reap manteniendo los errores. Close persistente conserva Pending aunque wait/handle ya estén acreditados, hasta completar el stream. Stub bloqueado no prueba bloqueo real de PDFium; no se repite ni se atribuye nuevo PASS al corpus FFI completo.

Los tres controles semánticos originales vuelven a ejecutarse con sus modos originales: Unicode-ToUnicode conserva InvalidUnicode/holes y Unverified; singleTj65536 conserva32767 chars/bytes y Unverified. Ambos continúan **SEMANTIC_COMPLETENESS_FAIL**. Astral offset1023 mantiene contraste UTF16 Complete y primary InvalidUnicode/holes2/Unverified: PASS sólo del contraste. EXPECTED-AE F3FA7F9E096A5AE987B336F4522180234D1A06E3A7B879CCD4C333B2B6646E5B verificado, sin regenerar goldens ni modificar requisitos. R1/R2/R3 y sus fallos se conservan.

El dictamen de código de raíz, agente distinto del autor y de esta AE, se consultó y verificó: `PDF-Rust-root-review-r4.md` SHA5EA49C70315AC2DA986978D7B4D1D229F12ECBD6A0578424FB5B5B2AC36DAE94. Su LateEOF propio coincide; las ocho reproducciones anteriores son independientes y no se atribuyen a raíz.

Evidencia propia60 archivos: `pdf-rust-ae-evidence-r4-MANIFEST.json` SHA256383E639F1230599ECEF08FDA0E649336FF32988F27C468A6356B4BF0797CC6E2. Incluye scripts/resultados y RAW nuevos de copia.50 controles propios (47 lifecycle/I/O y tres semánticos); los estados de fallo esperado del harness no son PASS de extracción semántica. El cierre queda limitado al delta del gate técnico revisado; no acredita soporte PDF completo ni adopción en producto.
