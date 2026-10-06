# Revisión raíz del delta UNC R2

2026-10-06. Revisión estática dirigida, separada de la aceptación funcional independiente.

Candidato: markdown-document-candidate-r2-unc/MANIFEST.json, SHA256 3BB930B73579F56C7C92F3D318CF6A5B56CAF01F3CD9FDDC207249AE327FACC4. Rehash fresco: 37 registros, cero discrepancias. Base A–C R1 permanece congelada y rechazada por los dos hallazgos conocidos.

Se compararon document_obsidian.rs y document_parser.rs R1/R2 y se contrastó el plan canónico MARKDOWN-UNC-CORRECCION.md. El delta productivo introduce únicamente prueba del comportamiento upstream fijado: Extended y prefijo Source con dos barras iniciales requiere que el destino del productor sea exactamente prefix[1..]. La comparación cancelable conserva bloques de 1024 bytes; no crea un fallback genérico para discrepancias. Después aplica la regla existente de escape único de pipe dentro de celda acreditada. No colapsa barras interiores ni normaliza rutas.

Ambos callers reales (Link y Embed) fuerzan Decoded cuando hay pérdida inicial o adaptación de pipe. La reconstrucción host exige contenido semántico igual a Source y procedencia EnclosingWhole del token completo, sin flags nuevos. El slicing de un byte es válido por el prefijo ASCII comprobado y no reserva memoria. No cambia cuotas, framing, controller ni Job.

SIN_CRITICAS_PENDIENTES para este delta estático UNC. La aceptación independiente sigue pendiente y este informe no autoriza adopción general A–C: MD-CALLOUT-EMPTY01 exige su candidato R3 y revisión propia. No acredita resolver de bóveda, visualización, navegación de enlaces ni cierre RV-301/304/305. No se ejecutaron tests nuevos desde esta revisión; los resultados de autor y la aceptación ajena tienen autoridades separadas.
