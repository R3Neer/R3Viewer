# Guardian privado MathJax: evidencia nativa R6

2026-10-10. Herramienta de investigación privada, sin importar MathJax ni ejecutar módulos de terceros. Complementa el [plan frío R2](MATHJAX_ENSAYO_FRIO_PLAN.md). La [revisión independiente final](../reviews/MathJax-guardian-own-ae-r6.md) acepta únicamente los 18 controles originales y un negativo propio del selector; no cierra StageB ni autoriza StageC.

## Resultado y procedencia

Raíz ejecutó los 18 casos prospectivos con el mismo guardian R6 y obtuvo recibos físicos completos. El adversario copió fuentes exactas a otro directorio, fijó expectativas y auditor propios antes de resultados y reprodujo los 18 casos: 11 procesos originales, arranque positivo, cancelación, timeout de 30 segundos y 15 controles de adquisición/contención. Comprobó whitelist de tres stdio contra objetos del kernel desde el hijo suspendido, exclusión del decoy, wait/exit/EOF/active0 y cierres checked. Los stops de adquisición son inyecciones declaradas; assign-invalid sí observa un rechazo de API real.

Raíz cotejó las 36 entradas físicas del manifest independiente: rutas contenidas, tamaños y SHA-256. Manifest DB508974668AB842FE4EAE26B37B552004A2F512DF64333CD3DEFC4861799307; informe F1B2C6A207C2222D867B617C7FE28D129B82BBDE57FBE708BF0D62A4468F2504. Freeze original de raíz: BC2D851135B26619CED214B55881BB1D3082D1AD4FC3B096E78B5B68C2800672, 50 archivos físicos. Guardian ejecutado: 923FC87B6B8642FDDA0C64F364DF67FB3BDA9B865ADB5CFD6C53796753EC7282. Fuentes, binarios, oráculos, fallos y logs permanecen privados bajo .tmp; este documento conserva trazabilidad, no los incorpora al producto.

R5 falló antes de adquirir recursos por resolver CompareObjectHandles desde kernel32. R6 cambia sólo ese binding a KernelBase, conforme a la [referencia oficial](https://learn.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-compareobjecthandles). El retorno observado del fallo R5 se conserva; no existe un rawstderr guardado y no se reconstruye como tal. Las versiones y revisiones anteriores permanecen intactas.

## Límites y siguiente gate

El ensayo usa Node 24.16.0 fijado, buffers stdout 2 MiB, stderr 64 KiB y scratch 4096 bytes. JobMemory 512 MiB es una cota del Job; no prueba RAM total del padre ni equivale al ProcessMemory del diseño PDF. Allocator y peakRSS del padre siguen UNKNOWN. Sólo se ha probado la escritura de START de seis bytes, sin garantía para entrada arbitraria de 64 KiB.

Faltan desbordamientos, errores de IO y cierre, recuperación del recibo de exit, Pending y retención física ante fallos del observador. Debe revisarse su mecanismo exacto y fijar oráculos antes de código/outputs, conservando los 18 controles anteriores y el deadline original de limpieza. Después sigue el cierre de módulos/loader y el ensayo de conversión. No se acreditan motor, métricas, raster, mezcla con terminal, QA visual, Unix, distribución ni RV-008/303/501.
