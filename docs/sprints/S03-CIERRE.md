# Estado vigente del cierre de S03

Actualizado 2026-10-10. **Sprint abierto.** La parada solicitada se ejecutará cuando termine todo S03; una aceptación focal no cierra el sprint. La [auditoría inicial](../reviews/S03-closure-audit-2026-10-10.md) conserva la historia y los criterios originales. Este registro resume las actualizaciones posteriores sin convertir evidencias anteriores en ejecuciones nuevas.

| Área | Estado y siguiente condición |
|---|---|
| Inspección y búsqueda Nu; Fuente/Estructura TOML, JSON, YAML, XML | Implementación y aceptación funcional Windows previas. Conservar límites de sus informes; faltan las verificaciones específicas indicadas abajo. |
| Categoría de timeout XML | Arreglo integrado y aceptación focal independiente: [evidencia](../research/S03_XML_TIMEOUT.md). No explica la duración/fallo históricos. |
| Catálogo y propagación Onig | [Catálogo](../research/S03_CATALOGO.md) y [spike Onig](../research/S03_ONIG_PROPAGACION.md) aceptados. No se promete detectar todos los errores internos de Onig. |
| Sintaxis: Legacy, caché y contexto | [N01–03](../research/S03_SINTAXIS_NATIVA.md) aceptados con controlador real; los demás casos de lifecycle se revisan separadamente. |
| Ctrl-C durante preparación | [Proyección e índice](../research/S03_CANCELPREPARE.md) aceptados, con recogida física y segunda apertura. No se atribuye esa fase a cancelación dentro del parser de sintaxis. |
| Memoria/material YAML y XML | [M01–07](../research/S03_MATERIAL.md) aceptados, incluidas fronteras reales, Job y cancel/retry. Exacto1M XML material no aplicable por paridad; precedencia de arena y guard de protocolo diferenciados. |
| Lifecycle de sintaxis SC06–11 | Plan mínimo revisado sin críticas; candidato privado en preparación. Reutiliza evidencia válida y añade solamente ventanas causales y recorrido ocupado que faltan. No aceptación runtime todavía. |
| Preparación/coste A06 y B07 | [Conteo A06](../research/S03_A06_VIEWPORT.md) aceptado independientemente; B07 mantiene la medición/cancelación de búsqueda en refinamiento. No nueva obligación de optimizar cada fase ni promesa SLA. |
| Paleta Nushell C01 | Propuesta condicionada viable; detalle técnico en revisión antes de implementar. Encoder sigue primero, sin closures de usuario ni cambio automático del wire. |
| QA visual Windows | Pendiente en Windows Terminal y WezTerm con versiones, fuente/DPI, temas y corpus reales. ConPTY/TestBackend no acreditan apariencia. El control de terminales está excluido por la habilidad Computer Use disponible; no sustituirlo por capturas simuladas. |
| Contención/runtime Linux y macOS | Pendiente U-G1–5. Sólo Windows confirmado; WSL no instalado. El backend Unix conserva Unsupported en las rutas aún no validadas. Diseño/probes preparados y crosscheck no equivalen a soporte. Una ejecución nativa futura necesita entorno y destino habilitados. |
| Freeze/regresión/aceptación final | Al terminar los cambios pertinentes y obtener evidencia que falta, revisar candidato exacto, reconciliar backlog/documentación y cerrar sólo criterios acreditados. |

PDF, Markdown enriquecido y MathJax mantienen sus checkpoints; no se inicia otro sprint para suplir pendientes de S03. RV-105/106/107 permanecen abiertos. No se modifica el corpus académico, perfiles, remotos o publicación.
