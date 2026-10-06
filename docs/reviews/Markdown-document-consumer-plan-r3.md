# Consumidor Document D — revisión adversaria final R3

2026-10-06. Revisor root; autor unix_r3_review. Candidato Markdown-document-consumer-plan-r3.md SHA256 08432D36C81A5339DC1E2D32AC3C6417F9E310078C2D07A5AB0F9E74940784C7. Revisión del diseño operacional, contrastado con contratos Markdown/A–C, CLI, SyntaxClient y límites reales del repositorio. No implementación ni ejecución del consumidor.

Veredicto: SIN_CRITICAS_PENDIENTES dentro del alcance del plan D1–D3 y de la asignación/refinamiento posterior obligatorio D4. Autoriza implementar D1–D3 tras disponer de la frontera A–C congelada y coordinar integración. D4 requiere su propio plan específico revisado antes de editar protocolo o integrar snapshots léxicos; esta aprobación no lo sustituye.

## Disposición de hallazgos

- DC-R01 cerrado: Markdown reconocido abre Documento; --format markdown reutiliza la opción existente. --language sigue gramática y Q14 Fuente inicial sólo datos; no conversión universal de valores Nu.
- DC-R02 cerrado: inline produce texto completo seguro o Fuente completa etiquetada. Fallo/cancel de preparación impide emitir prefijo; escritura efectiva de pipe se distingue expresamente y conserva política BrokenPipe vigente.
- DC-R03 cerrado: FlowCursor y endpoints globales preparados cancelablemente, capacidad cobrada al mismo ConsumerBudget y sin suponer IDs preorder. End no recorre Root ni hace reflow global; DC17/18 verifican recuperación, visitas, cuotas y stale.
- DC-R04 cerrado: D4 obligatorio conecta Syntect5.3/catálogo/paleta con CodeText. Estado léxico aislado por bloque, scheduling de un propietario, receipts/identity/leases y fallback completo de texto. La API real set_document(Arc<String>) exige snapshot cobrado o extensión específicamente revisada, sin inventar zero-copy. DC19 incluye Nu/PowerShell/Rust, multiline, desconocido, controles, gigante, stale y redirección sinANSI antes outputs.

El límite global4096 es capacidad combinada de índices, no4096 por Domain ni garantía de caber en1MiB. Source y Document conservan ledgers separados; Consumer1MiB se reserva de arena host64MiB y incluye old/new. Publicación y hit/anchor son transaccionales. Unicode multirun no reinicia cursor por estilo/Run, mantiene contexto y admite sólo glyph completo.

## Evidencia y límites

Se leyó R3 completo y se comprobó SHA256; R1/R2 y críticas temporales se conservan fuera de Git. La revisión contrasta diseño y fronteras reales, no acredita Codehighlight, renderer, búsqueda, TUI/inline ni prestaciones gráficas funcionando. Cada incremento exige código revisado y aceptación independiente sobre candidato exacto. Gráficos, resolución Obsidian/web/PDF, QA visual, ejecución Unix, distribución y cierre de RV siguen pendientes. La palabra propuesta del documento original describe su procedencia; este dictamen registra su aprobación limitada actual.
