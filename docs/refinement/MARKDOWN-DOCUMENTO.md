# Documento Markdown enriquecido — refinamiento operacional R1

Autor root. Propuesta para el siguiente incremento, pendiente adversario; no abre otro sprint ni modifica producción. Base99c220d, mientras el implementador integra Unicode completo en S03. No recorta Markdown complejo: imágenes, Mermaid oficial, matemáticas gráficas inline/bloque y Obsidian siguen parte de la aceptación final.

## Base verificada y objetivos

Gate parser R3 aceptado por agente distinto del autor: pulldown-cmark0.13.4/Event/OffsetIter, perfiles explícitos, wiki/math, fuente y texto decoded diferenciados. Reutilizar fuentes/licencias/lock congelados como procedencia; código del ejecutable experimental no es adaptador productivo ni supervisor instalable. La cuota32KiB del gate no se transporta al producto.

Objetivo incremental: documento semántico con origen/procedencia y recursos tipados, layout compartido TUI/inline y navegación estable; integrar recursos gráficos cuando sus contratos y ejecución estén listos. Un incremento textual del documento no acredita RV-301/302/303/305 final, cuya presentación enriquecida completa permanece obligatoria. Valores Nu mantienen su modelo propio, no convertirlos universalmente en Markdown. HTML compartirá modelo documental, pero mantiene DOM/from html separado.

## Contrato del modelo

SourceSnapshot inmutable mantiene bytesUTF8, generation/documentId, origen explícito local/literal/pipeline/URLfinal y base acreditada. Metadata no reabre una ruta ni convierte contenido de pipeline en archivo. DocumentId y nodeId valen sólo dentro de ese snapshot; reload genera identidad nueva.

Arena plana de Block y runs inline, IDs numéricos y enlaces parent/children/siblings validados sin recursión no acotada. Tipos: párrafo, título con nivel, cita/callout, listas y tareas readonly, tabla con alineaciones, código cercado/indentado, regla, footnotes y bloque HTML inerte/contextual. Runs contienen estilos semánticos anidados, texto, código, break, link, image/embed y math. Mantener Text consecutivos y nesting correcto; coalescer sólo conservando origen y orden.

Texto usa BorrowedRange únicamente tras comprobar igualdad exacta con substring; OwnedDecoded en normalización/entity/escape. OriginSpan enum ExactToken/EnclosingToken: no dar offsetsdecoded como bytes fuente ni mappingescalares falsos. Selección/búsqueda del documento usan dominio documental; Source sigue íntegra separadamente. Cada hit conserva dominio+run/offset; volver a Source usa span acreditado, indicando contenedor cuando no sea exacto. Links retienen rawtarget/label/title/kind y provenance; no resolver ni descargar durante parser.

ResourceSpec tipado Image/Mermaid/Math/Embed con input/range+origin+generation y displayintent (inline/block, alt y dimensiones pedidas). Código mermaid conserva fuente exacta y sólo genera ResourceSpec en fenced language mermaid acreditado. Matemática genera ResourceSpec de eventos correspondientes, no regex sobre todo documento. ResourceSpec nunca contiene escapes terminal ni comandos. Una imagen/wikiembed no es un link ordinario: mantener fragmentPDFpage, alias/tamaño y tipo para resolver posteriormente.

Frontmatter no ejecuta settings ni comandos. Contexto HTML/código excluye reinterpretación indiscriminada de wiki/math. CalloutsObsidian personalizados/title/+/- se reconocen en contexto de BlockQuote y primercontenido, preservando fallback del texto si no se acepta el marcador. Resolver headings/bloques/footnotes y duplicados en índice explícito por políticaObsidian revisada, no asumir anchors de pulldown ni aplicar slug arbitrario universal.

## Worker y presupuestos candidatos

Constructor/parser next de pulldown no garantizan cancelación: proceso separado antes de parsear. Fuente128MiB original permanece; parser entrega schema tipado con generation/hash/origen y respuesta final completa validada, nunca Debug de enums como wire. Framing bounded y EOF sin trailing antes de publicación, latches de cancel/stale y reap obligatorio. Worker parser no obtiene archivo ni red adicional: sólo snapshot prestado por transporte propietario. No callbacks externos ni ejecutar HTML/código.

Reutilizar mecanismo de ownership/cleanup de worker del producto, no scripts privados Python. Debe validar contención efectiva de plataforma antes de admitir fuente. WindowsJob512MiB committed candidato comparable con workers existentes; Unix mantiene capacidad pendiente hasta ejecución de guardian/containment. Fuente transferida/modelo/outputs/lector se contabilizan separadamente; source no puede seguir capturado en writer después de transferencia.

Perfil inicial de arena/documento64MiB, un millón de nodos/runs, depth64, transporte128MiB como candidatos de ingeniería derivados del modelo estructural existente, NO garantía ni nuevos límites confirmados por usuario. Antes de integración fijar campos/tamaños/encoding y reservas concretas con pruebas exact/+1/old+new; evitar JSON completo duplicado. Fallo parser/cuota conserva fuente y diagnóstico, no llama documento completo a un prefijo. Budget/Rustrequested no equivale a OScommitted ni SLA.

## Layout, sesión y recursos

Layout responsive produce fragmentos de texto/recursos con anchor lógico independiente de geometry y viewport. Usa motor Unicode y preflight integrado/revisado; no llama Ratatui con unidades no admitidas. Tabla/código/lista/cita conservan estructura/roles discretos y etiquetas de fallback claras. Reflow no cambia IDs/hit/history. Inline y TUI comparten semántica/layout y difieren en emisión/lifecycle; redirección permanece textual sin escapes ni gráficos. Inline puede mostrar referencias explícitas de recursos pendientes; no conseguir red implícita para lograr una imagen.

Navegación mantiene documento/destino/posición en historial existente; resolver destino antes de reemplazar origen, conservar vista si falla. Localrelative desde base local; URLrelative desde URLfinal/baseHTML acreditados. Pipeline/literal sin base no inventa cwdnota. Obsidian: nota/heading/block/embedPDF/basenameambiguo/ciclos, pageN física1based y errores fuera de rango sin cambiar página silenciosamente. Índice local basado en snapshot/catálogolazy bounded, no copiar ni reescribir bóveda. CorpusClases sólo HEADreadonly cuando haga falta aceptación real autorizada.

URLargument autoriza documento explícito; secundarios remotos permanecen Reference hasta acción de carga. No introducir flagautocarga/permisopersistente inventado. Recursos locales también requieren base/origen/ruta autorizados y límites del resolver. Formularios/checkboxHTML y tareas sólo muestran estado, sin enviar o editar. from html sigue módulooptin separado.

Imagen→contrato gráfico/cache/transform, PDFembed→renderer lazyPDF physicalpage, Mermaid/math→servicios privados con bundle/licencias y workers propios. Compartir RenderArtifact y generation antes de composición. GIPC actual ActiveProcess2 no autoriza spawnChromium extra: aislamiento de Mermaid/MathJax/browser requiere contrato propio revisado; no introducir children por excepción informal. Resultado late/stale se descarta; error de recurso deja referencia/fuente y diagnóstico sin abortar documento entero. Fórmula inline necesita baseline/ascent/descent y tamaño real para layout, no sólo unaimagen debajo del párrafo.

No cerrar capacidad gráfica hasta mismo documento mixto en terminal objetivo con scroll/resize/exit sin residuos y protocolo negociado sin consumir teclas. ConPTY no prueba apariencia. Fallback legible no sustituye soporte complejo acordado.

## Etapas y ownership propuestos

1. Refinar wire/arena/budget/callout-context e invariantes de Document con fixtures y expectativas previas; usar bibliotecas originales como fuente, no outputs propios como golden. root autor del contrato; adversario independiente.
2. Un implementador integra parser/modelo/worker en nuevos módulosdocument_*, sin tocar archivos Unicode en paralelo. Cambio Cargo/lock/main/lib se coordina después del freeze del incremento activo. Revisión/AE de respuesta/procedencia/Source/search y rutas consumidoras reales antes de publicación interna.
3. Layoutdocumental y sesión con links/índice/history; prueba TUI/inline/redirect y reflow/anchors. Gráficos aún pending no permite cerrar RV final. Resolver común tipado crea destinos concretos sin perder contexto.
4. Conectar renderer gráfico aceptado, PDF lazy y servicios Mermaid/math, luego resolverObsidian y web según dependencias reales. Validar conjunto final mezclado en Windows; demástargets sin claim hasta pruebas nativas. Mantener fallos existentes/visual abiertos.

Preguntas técnicas pendientes, no preguntas al usuario por rutina: encoding/framing exacto; fuente transferida y chargearena; anchorObsidian normalización acreditada; aliasdimensiones y tipos; métricas inline de fórmulas; cache compartida y fallos de renderer; workersbundle/browser aislamiento; catálogo Obsidianambigüedad/ciclo. Se refinan antes de editar cada frontera. Las decisiones de usuario actuales no se reabren ni se cambian para pasar pruebas.

Referencias canónicas: docs/refinement/MARKDOWN-PARSER-GATE.md; docs/research/MARKDOWN_PARSER_GATE.md; docs/spec/CONTRATOS.md, WEB_HTML.md y REQUISITOS.md; docs/architecture/ARQUITECTURA.md. Primarias bloqueadas: pulldown-cmark0.13.4 fuentes/licencias en snapshotR3; diseño gráficoIPC/BINDING y worker_process.rs/structure_protocol.rs actuales. Esta propuesta no afirma runtime nuevo.
