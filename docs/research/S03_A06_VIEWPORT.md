# S03 A06: trabajo real limitado al viewport

2026-10-10. [Revisión y reproducción independientes](../reviews/S03-A06-final.md) sin críticas pendientes en el conteo de trabajo y las referencias prestadas. A06 combina esta evidencia con sus mediciones históricas de preparación/memoria; no impone un SLA ni afirma que toda la memoria del proceso pertenezca al modelo.

El adversario ejecutó una vez el test existente exacto sobre el binario integrado congelado, sin recompilar ni modificar expectativas: un PASS, exit0 y wait físico. La preparación de 100.000 filas cuenta 100.000 filas/200.000 campos; el renderer productivo visita nueve filas/18 celdas en80×12 y dos/dos tras resize30×5. El contador de filas preparadas permanece100.000. TestBackend acredita trabajo y estado, sin certificar apariencia.

DataRef, Cell y Navigator mantienen referencias prestadas al payload; el test retiene el mismo Arc. Metadatos, rutas y presentación segura tienen costes propios. No se clona todo el subárbol al dibujar ni se escanean todas las filas por frame.

Preparación observada36,2615ms en esta ejecución de desarrollo; no es tiempo del productor Nu/encoder/IPC ni garantía de rendimiento. Las mediciones históricas [release](S03_RENDIMIENTO_RELEASE.md) siguen atribuidas a su candidato. El requisito original pide separar preparación/memoria del trabajo de dibujo: una descomposición adicional de productor, encoder y transporte sería investigación, no un nuevo gate obligatorio de A06.

Bin SHA541A1D3E001FB8C6392A24A6BF8F37900A8D0D3ABA8C694BEE40B2483C080A13 intacto antes/después. Manifest final independiente6DDB71F183F3E931FC8BCC139B2F920523A8FA02C41930CBA1B43221959E7EBD; raíz cotejó13/13 entradas. Artefactos privados en `.tmp/s03-closure-adversary-2026-10-10/a06-own-r1`.

B07 mantiene su medición/cancelación de búsqueda separada; lifecycle, QA visual, Unix y S03 continúan abiertos. Los tiempos/fallos históricos de XML conservan causa desconocida.
