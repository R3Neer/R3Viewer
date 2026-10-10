# Sintaxis SC06–11: aceptación focal Windows

2026-10-10. [Revisión final independiente](../reviews/S03-SC06-11-final.md): SIN_CRITICAS_PENDIENTES para los criterios SC06–11. No cierra S03 ni incorpora instrumentación al producto. Base de código29993a0; fuentes, Rust/binarios/DLL y cuotas originales permanecen identificados en los snapshots privados.

El recorrido ocupado usa Nushell real: Strings A/B/A, cambios de pane, scroll y resize; el último resultado coincide con generación, petición y punteros Model/Source actuales. Al volver al contenedor no revive el estilo anterior. La misma instancia de hoja mantiene snapshot y generación en scroll/resize, mientras la petición cambia. Los controles UI originales de selección String/Key/numeric y arbitraje se reutilizan explícitamente; no se atribuye nueva ejecución propia a esos tests.

El autor ejecutó cinco casos: View/All pasaron en R3, con progreso real antes del park; Busy/Startup/Transfer fallaron y se preservaron. R4 corrigió únicamente el arnés Python: primera entrada en Value,22filas por height24−2 y diagnóstico de muestras/HANDLE antes del assert. No cambió el producto ni sus clocks. Startup usa un testigo directo positivo del HANDLE original aunque el sampler no observe vida; no se fabrica una muestra.

El adversario reprodujo Busy/Startup/Transfer una vez cada uno: los tres PASS. Busy0→Tiny0; Startup/Transfer130→Tiny0 con Ctrl-C ETX real en la consola propia y callback original. Hay testigo físico de proceso/generación/fase antes de cancelar y wait/cierre después. En cada caso: EOF, joins de IO, HANDLE/capturas cerrados, JobActive0, HPCON cerrado y ningún Pending o dueño retenido. Restauración anterior a los joins finales y modos/cursor recuperados. Tiempos propios2.204s/0.583s/0.653s son observaciones aisladas, no SLA.

View/All, inventario, dos controles UI y fronteras/deadlines L3 anteriores se reutilizan con fuente exacta y cobertura delimitada. No se promete interrumpir una llamada nativa Onig por dentro ni observar todas las intercalaciones mediante sampling. Los límites originales no se renuevan con ACK/progreso.

Raíz verificó86/86 entradas por bytes/SHA del manifest final DB3D605CA333F310C39652F73AD49CEC267D33C3A1E1217CEC94F89580FC030B. Informe B36251FD02C76C416F0CA29BEC5ABFF11D3AB9883FF85D0D4A7E0F3B0E4D5FFD; auditoría086A3B3267E53A0360A483CC3C653FF840E53337DBAD04990BB2798DB1393666. Artefactos: `.tmp/s03-closure-adversary-2026-10-10/sc-own-ae-r4`, con captura histórica R3 preservada y before/post de19identidades propios.

C01, W1/W2/W3, QA visual Windows Terminal/WezTerm, eventos físicos y contención/runtime Unix conservan sus obligaciones. ConPTY acredita funcionamiento, no apariencia. PDF/Markdown/MathJax siguen en checkpoints; RV-105/106/107 abiertos.
