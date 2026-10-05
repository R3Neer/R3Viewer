# Markdown: gate inicial de parser y procedencia

Propuesta del coordinador para refinamiento del siguiente incremento; no abre S04 ni cierra S03/RV-301. No implementación ni descarga nueva realizada por esta propuesta. Se mantiene Markdown final enriquecido con imágenes, Mermaid, matemáticas inline/bloque, Obsidian y navegación entre formatos.

## Pregunta comprobable

¿pulldown-cmark 0.13.4 permite construir el modelo documental común sin convertir primero a HTML ni perder orden, contenido o relación con la fuente? El gate examina eventos/offsets y extensión wiki/math; no acredita render ni búsqueda, contención, navegador, PDF o terminal. La biblioteca tiene licencia MIT. Versión candidata exacta; sólo se incorporará a producción tras lock, avisos y aceptación pertinente.

Fuentes primarias consultadas el 2026-10-05:

- https://github.com/pulldown-cmark/pulldown-cmark/releases: 0.13.3 corrige offsets wiki y 0.13.4 un panic de TightParagraph; esto no prueba ausencia de otros fallos.
- https://raw.githubusercontent.com/pulldown-cmark/pulldown-cmark/v0.13.4/pulldown-cmark/src/lib.rs: Event/Tag, opciones, licencia, eventos consecutivos de texto posibles, API sin HTML obligatorio.
- https://raw.githubusercontent.com/pulldown-cmark/pulldown-cmark/v0.13.4/pulldown-cmark/src/parse.rs: iteración con offsets sobre la entrada; una API pull no garantiza que construcción ni next sean cancelables en entradas arbitrarias.
- https://raw.githubusercontent.com/pulldown-cmark/pulldown-cmark/v0.13.4/pulldown-cmark/Cargo.toml: revisar feature graph exacto antes de construir, no habilitar SIMD/HTML/serde por conveniencia sin necesidad del gate.
- https://docs.rs/pulldown-cmark/0.13.4/pulldown_cmark/enum.LinkType.html: WikiLink y etiqueta alternativa.
- https://docs.rs/pulldown-cmark/0.13.4/pulldown_cmark/struct.Options.html: tablas, tareas, matemáticas y wikilinks explícitos; no asumir que GFM equivale a todas las extensiones Obsidian.

## Experimento privado propuesto

Paquete Cargo aislado de Cargo raíz/producción, toolchain existente, lock y fuentes/hash antes de build. Sólo fixtures UTF8 sintéticas propias, máximo 32KiB por fixture y 4096 eventos por invocación del experimento. Capturar stdout máximo 1MiB, deadline externo 10s y proceso retenido hasta wait/cleanup. Estas cuotas son límites de ensayo, no límites de Markdown del producto ni demostración de contención. Usar proceso separado para que un panic/fallo del parser no mate al coordinador; no ejecutar contenido ni resolver recursos. No corpus Clases.

Imprimir eventos y offsets con campos tipados suficientes para leerlos, sin depender exclusivamente de Debug como oracle: Start/End con tipo, Text/Code/Html/InlineHtml/InlineMath/DisplayMath y payload, SoftBreak/HardBreak/Rule/TaskListMarker/FootnoteReference; Link/Image con tipo/destino/title/id; heading level y metadata. Conservar entrada y parámetros exactos. Ningún callback broken_link externo.

Expectativas propias congeladas ANTES de outputs; adversario añade fixtures independientes. Comprobar eventos balanceados, rangos dentro de fuente y en límites UTF8, contenido/orden semántico esperado. No imponer que todos los rangos sean disjuntos: Start/End pueden describir contenedores superpuestos. Texto decodificado y código normalizado no implican identidad byte a byte con el substring indicado. Si offsets wiki fallan, preservar evidencia y corregir elección/adaptación antes de producción.

## Matriz mínima con expectativas previas

1. Títulos H1/H2 y párrafos con español/emoji: niveles/orden y contenido, sin suponer anchura terminal.
2. Emphasis/Strong, escapado, entidad `&amp;` y code inline: texto visible decodificado, origen distinguible, estilos anidados y code separado.
3. SoftBreak y HardBreak: eventos distintos; CRLF y fin sin newline conservados en fuente.
4. Lista ordenada desde 3, lista anidada y tareas [x]/[ ]: inicio 3 y estado de tarea, solo lectura.
5. Tabla con tres alineaciones y celda con código/enlace: estructura y orden; sin asumir TableBody emitido.
6. Cita y callout GFM NOTE: tipo reconocido; callout Obsidian personalizado con título y +/- se conserva aunque no sea reconocido por GFM.
7. Código cercado `rust` y `mermaid`, e indentado: información de lenguaje y contenido íntegro; Mermaid no se ejecuta en este gate.
8. Matemática `$x^2$` y `$$...$$`: eventos inline/bloque correctos; dólares escapados y dentro de código no crean recursos matemáticos.
9. Imagen Markdown con alt/title/destino relativo y link de referencia: destino, texto alt, orden y referencia resuelta por parser cuando definición existe.
10. Wikilinks `[[Nota]]`, `[[Nota|Alias]]`, `[[Nota#Título]]`, `[[Nota#^bloque]]` y `![[Manual.pdf#page=3|300]]`: comprobar qué parser realmente distingue entre enlace/incrustación y cómo preserva alias/destino. Tamaños y resolución quedan a adaptador Obsidian, no asumir comportamiento nativo por nombre de feature.
11. Sintaxis wiki/math/imagen dentro de code y HTML: no reinterpretar contenidos de código como enlaces; HTML preservado como fuente inerte en este gate.
12. Enlace inexistente/de referencia no definida, destino percent-encoded, URL y fragmento vacío: no acceso de red/archivo; conservar texto/destino sin elegir un candidato ni normalizar doblemente.
13. Footnotes, encabezados repetidos y frontmatter: cobertura real identificada; parser no genera automáticamente política de anclas Obsidian. Frontmatter explícitamente inerte.
14. Entradas malformadas, markup anidado y muchos eventos hasta límite: fallo/limit explícito, sin salida parcial presentada como documento completo. Controles ESC/OSC guardados como fuente, nunca emitidos sin sanitización por un futuro presenter.

## Resultado para el modelo común, sujeto a revisión posterior

SourceSnapshot es la fuente UTF8 inmutable con identidad/origen/base. Documento usa arena de bloques y runs inline con IDs estables dentro del snapshot, no árbol recursivo de copias completas ni datos Nu transformados universalmente a Markdown. Eventos Text consecutivos son legales; sólo coalescer manteniendo procedencia. Propiedad de texto: rango prestable si se prueba idéntico; texto decodificado propio cuando corresponde. Un rango de evento es procedencia a nivel de token, no mapping escalar exacto para seleccionar glifos. No inventar posiciones mediante proporciones.

ResourceSpec describe Image/Math/Mermaid/Embed y conserva payload/raw-span/origen/opciones; no dispara IO. LinkSpec conserva destino original, interpretación y label; la resolución canónica será compartida con HTML/Obsidian y ocurre por acción autorizada. Renderer y scheduler recibirán recursos tipados con identidad/generación; no ANSI/arbitrary HTML como modelo de datos. Fórmula inline ocupa run con métricas gráficas a determinar por backend y fallback legible en su misma posición.

La primera integración textual de Markdown podrá preceder al gráfico según arquitectura vigente, conservando referencias explícitas y fuente accesible; NO satisface la elección compleja del usuario ni cierra RV-302/303/305. Parser/mapping/layout/navigation/protocol/budgets/cache/render requieren propuesta específica y revisión antes de implementación. Fuente completa y fallback local siguen obligatorios; límites experimentales no se convierten en truncamiento de producto.

## Salida y aceptación

Informe con lock/features/licencias/fuentes exactas, fixtures y oráculos previos, salida raw, manifest y hallazgos/refutaciones; revisión/AE independientes sobre candidato congelado. Verde sólo prueba este gate de parsing/offsets. No reclama QA visual, memoria de documentos grandes, scheduler/cancelación de producto, Unix nativo, resolución de bóveda ni Markdown enriquecido completo. Si la biblioteca sólo resuelve parte de wiki/callouts, la adaptación deberá ser contextual sobre eventos/fuente, no regex global que reinterprete código.
