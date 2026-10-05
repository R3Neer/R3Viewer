# Viabilidad del adaptador Unicode — gate privado R2

El gate privado tiene [revisión de código independiente](../reviews/S03-grafemas-gate-code-r2.md) y [aceptación independiente](../reviews/S03-grafemas-gate-aceptacion-r2.md) sin críticas pendientes dentro de su alcance. Fuente candidata99registros, MANIFEST69EA6361593E41C7389565476889C5D94DE0A7FC2257301D1A6DB842E6239E96; binarioD8819EA2522B829F1F7A56CB6DB3DF59E519177191B9E1159CFC803A996B2B90. Snapshot privado conservado fuera de Git.

R1 rechazado se conserva; R2 corrige limpieza del supervisor, preadmisión del presupuesto antes del ancho y overlap de cursor para RI. AE independiente contrasta más de un millón de fronteras y ancho con dependencias originales, pequeños hints y secuencias largas, además de controles de cancelación, cuotas, captura tardía y Pending. Los informes delimitan los corpus exactos y distinguen pruebas heredadas de pruebas propias.

La decisión de integración utiliza adaptación mínima licenciada de unicode-width0.2.2 y GraphemeCursor1.13.3 sin alterar reglas/tablas. Estado incremental de ancho, snapshots reales de cursor, metadatos sparse de excepciones y referencia antes de Ratatui conforme a [diseño R2](../refinement/S03-GRAFEMAS.md). No introduce cap nuevo de fuente;16KiB/símbolo y256KiB/frame/trabajo siguen candidatos de admisión de dibujo, sujetos a pruebas del producto.

S03-UNICODE01 permanece abierto. Falta integrar cancelación, procedencia, búsqueda y admisión en SafeTextModel, valores Nu, inspectores, inline, sesión legacy y panes prestados YAML/XML, verificando las rutas combinadas. El facade privado no acredita estas rutas ni QAvisual/SLA/matriz nativaUnix. S03 y RV-105/504 no se cierran por esta evidencia.
