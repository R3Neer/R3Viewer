# Unicode/layout ResolvedUnit R1: integración, 2026-10-06

Base2f070ac. Se incorporan únicamente cuatro fuentes PRODUCT congeladas de la candidata ResolvedUnitR1 y la [aceptación independiente final](Markdown-resolved-unit-acceptance-r1.md). No se copia el contexto con scheduler WIP ni su perfil de diagnóstico.

## Comportamiento

El token opaco conserva posiciones validadas para recorrer cada cluster sin volver a buscar su prefijo. Las consultas de referencia usan TextRef reales y cobertura propia, sin Run/Domain sintéticos. Antes de publicar el cursor se admiten token/wrapper y todas las comprobaciones, incluida cancelación; un fallo conserva la posición anterior. Los visitantes aleatorios mantienen su API y comportamiento.

El padre de texto/referencias coincide con HEAD2f070ac. El padre de layout difiere exclusivamente en el hint4 anteriormente ensayado frente4096; se incorpora esa reducción junto con el token, sin modificar las reglas Unicode. La cuota de Work de producción sigue256KiB. Una mejora de coste no acredita que todos los documentos densos quepan; su calibración explícita está en un ensayo separado.

## Evidencia

Root verificó39 registros de candidata y22 de aceptación sin discrepancias. El primer lector del manifest AE esperaba objeto cuando era lista; falló antes de modificar y se corrigió la lectura. Integración byteexacta de PRODUCT4, documentada en resolved-unit-shared-adoption-readback.json privado.

Composición compartida:36 pruebas texto/Ref,17 navegación/window y16 inline, total69PASS; formato y Clippy todos los targets -DwarningsPASS. Se excluyó explícitamente la prueba antigua32MiB; no se ejecutó matriz Unicode universal ni QA visual/Unix.

La aceptación independiente acredita7 grupos propios más17 regresiones b1. Conserva fallos históricos del harness y sus correcciones de montaje; no se cambiaron los oráculos de producción para obtener verde.

## Límites

No renderer, scheduler Syntect, ruta CLI ni recursos gráficos acreditados. El ensayo separado de Work2MiB y buffers/framework2MiB no forma parte de este commit y no modifica las cuotas vigentes. RV-301 continúa pendiente.
