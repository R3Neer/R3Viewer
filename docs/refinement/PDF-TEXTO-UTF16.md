# PDF: contrato operacional de texto UTF16

Contrato final aprobado en revisión independiente, antes de implementación. El contrato efectivo combina [R3](PDF-TEXTO-UTF16-R3.md) y [delta R4 prevalente](PDF-TEXTO-UTF16-DELTA-R4.md); ambos conservan el contenido de entrega, incluidas menciones históricas de revisión pendiente. Git normaliza CRLF a LF: los hashes de los informes identifican los artefactos privados originales, no los blobs normalizados de HEAD. La comparación de HEAD con esos originales tras normalizar sólo CRLF a LF ha pasado para R3, delta R4 y modo validador. El dictamen vigente es la [revisión final](../reviews/PDF-texto-utf16-plan-r4.md), SIN_CRITICAS_PENDIENTES para diseño.

Tamaños y perfiles en [receipt R3](PDF-texto-utf16-sizes-r3.json), contrastados mediante [aritmética independiente](../reviews/PDF-texto-utf16-arithmetic-r3.json). Matrices previas a outputs: [18casos originales](PDF-texto-utf16-oracle-r2.json), [suplemento R3](PDF-texto-utf16-oracle-r3.json) y [suplemento R4](PDF-texto-utf16-oracle-r4.json).

Implementación privada autorizada después de cerrar U16-P01/02/03. No hay candidato ejecutado de este contrato ni aceptación del producto. La investigación de truncado y los fallos originales permanecen; SourceUnverified no se convierte en completitud por acuerdo de APIs. RV-204/G1, integración gráfica, QA visual y runtime Unix siguen abiertos.


El gate privado usa un [validador Rust secuencial sin DLL](PDF-TEXTO-UTF16-VALIDADOR.md), con [revisión final](../reviews/PDF-texto-utf16-validator-plan-r1.md) sin críticas pendientes de diseño. Añade recibo tipado del wire y ledger combinado de buffers conocidos del supervisor/validador, manteniendo deadline único y publicación final tras ambas etapas. No hay aprobación de código ni runtime por esta revisión.


## Estado posterior de implementación privada

2026-10-06. El texto anterior de ausencia de candidato ejecutado es histórico. Candidato R1 congelado y revisado: [estado del gate](../research/PDF_UTF16_GATE.md), rechazado por PDF-U16-ROOT01 (EOF anticipado en supervisor). No cambia el contrato aprobado; requiere corrección/revisión/AE posterior y mantiene semántica y RV-204/G1 pendientes.
