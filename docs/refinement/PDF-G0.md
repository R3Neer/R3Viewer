# PDF: propuesta de contrato y gate R2

Refinamiento paralelo, no sprint nuevo ni producción. S03 permanece activo. RV-204/202/203/306/305 conservan toda su aceptación. Diseño revisado independientemente sin críticas pendientes; experimentos pendientes; no adopta ABI, wrapper ni binario móvil.

## Contrato observable

Abrir un PDF sin destino muestra su primera página física. Un destino tipado Page(N) es 1-based, independiente del número impreso, etiquetas PDF o numeración de una nota. Validar rango antes de cambiar documento/vista: fallo conserva el origen. El resolver Obsidian entrega ruta y Page(N) separados, sin reanalizar un string de destino en el renderer. La sintaxis de ruta directa con # y posibles colisiones con nombres reales debe pasar el contrato de entrada antes de incorporarse a CLI; no reutilizar -p, que ya significa --path.

Documento con páginas carga y renderiza únicamente la página solicitada; navegar y ampliar usa el historial común con posición/zoom/destino. En backend sin raster queda referencia y texto de la página cuando haya capa de texto, con navegación física conservada. El PDF escaneado sigue siendo gráfico; búsqueda comunica que esa página no tiene texto extraíble. No invoca OCR.

Buscar literal sensible a mayúsculas (igual que texto actual), n/N por texto extraído y página, no por texto inventado desde pixels. Buscar el documento puede visitar páginas secuencialmente bajo cancelación, liberándolas; no indexar todo al abrir. Publicar una coincidencia sólo cuando conserva página e identidad de generación. Si el presupuesto/plazo impide completar el recorrido, declarar búsqueda incompleta; no responder 'sin coincidencias' como si fuera exhaustiva. BBox/selección exactas se acreditan con corpus antes de anunciarse; salto físico sigue obligatorio.

Inline/redirección muestran sólo la página solicitada o, si no hay destino, la primera: Q08 exige no renderizar el libro entero por defecto. Encabezado físico Página N/total y referencia al documento/otras páginas hacen visible este alcance, sin fingir extracción de todo el libro. El texto/referencia de esa página permanece en orden; raster inline sólo cuando la infraestructura gráfica lo permite. Búsqueda interactiva sigue cubriendo el documento bajo demanda; imprimir todas las páginas no se incorpora silenciosamente a este gate. No imprimir datos binarios ni enviar contenido PDF al shell. Índice/bookmarks y links se convierten en destinos inertes; seguirlos es acción explícita y usa resolver del visor. /Launch/JavaScript/formularios/adjuntos no se ejecutan. URL visible no se descarga automáticamente.

Documento corrupto, protegido que no abre con contraseña vacía, página inválida o biblioteca ausente dan diagnóstico específico y conservan vista previa/origen. Este primer gate no incorpora contraseña ni destraba cifrado; soporte con contraseña es decisión pendiente, no sustituto silencioso del PDF original. Cero páginas no se finge primera página válida.

## Fronteras de implementación propuestas

Adquisición binaria separada de UTF-8, con snapshot/origen/hash y cuota previa. No encadenar PDF a TextSource ni decodificar con pérdida. El lector PDF recibe bytes del snapshot del padre, nunca reabre la ruta ni accede por sí mismo a URLs. Núcleo tipado/URL y archivos binarios conservan sus contratos futuros completos; este gate comienza con archivos locales sintéticos.

PDFium en worker propio contenido antes de inicialización/carga del documento. DLL/so/dylib de bundle privado por ruta y hash exactos; ningún fallback a cwd/PATH. ABI/build/versiones/target/avisos se fijan antes de ejecutar el gate. Build sin V8/XFA preferido, sin API de acciones/form fill; no afirmar 'sandbox de archivos/red' sólo por usar Job. Un thread serializa todas las llamadas nativas; la API declara que no son thread-safe.

RAII libera bitmap/text-page/page/document/library en orden; buffer original permanece vivo hasta cerrar documento. Cancelación/timeout de una llamada nativa bloqueada exige supervisor externo y kill/reap del worker propio. No ejecutar más de un parser/renderer activo por sesión según scheduler compartido; publicar raster/text sólo tras descriptor/generación/hash/fin/EOF/exit válidos. No retener handles PDFium en UI.

Bitmap visible tiene dimensiones/stride checked y cuota previa, nunca bitmap a resolución natural gigante. Raster→G2 common body y parent framing; matte/rotación/crop/DPI forman identidad de cache. Cache global gráfica cobra PDF y PNG conjuntamente. Páginas lazy no implican parseo nativo gratis: la memoria interna se contiene por OS y se mide con fuentes representativas. No inventar métricas portables desde Windows.

Dominio buscable: capa de texto Unicode extraíble de cada página física, incluida una capa OCR existente invisible; no se ejecuta OCR. Esta capa puede incluir contenido fuera de CropBox y no equivale al texto visible en los pixels. La vista de texto se etiqueta como capa textual, y la búsqueda puede encontrar ese contenido: salta a la página y al intervalo del texto; ubicación gráfica fuera del recorte o no acreditada se comunica sin highlight inventado. No filtrar o declarar ausente un match sólo por no tener BBox. El corpus debe fijar explícitamente un texto fuera CropBox encontrado y un documento escaneado sin capa no encontrado.

Recorrido n/N distingue todas las ocurrencias dentro de una página y entre páginas; identidad incluye hash/generación, página, intervalo UTF-8 del texto extraído y mapping PDF cuando esté acreditado. No salta la segunda ocurrencia por haber visitado ya esa página. Literal puede cruzar chunks de la misma capa Unicode contigua; no cruza páginas ni huecos de caracteres sin mapping. Texto renderizado como etiqueta de error/unmapped no es texto del documento ni se busca como tal.

Texto por tramos acotados: diferenciar unidades UTF-16/índices PDF/UTF-8 y caracteres sintéticos. Preservar un high surrogate pendiente entre chunks y resolverlo antes de publicar texto válido; pareja rota/datos incoherentes diagnostican fallo o cobertura incompleta, sin fabricar U+FFFD buscable. Mantener mapping de cada tramo si se anuncia selección/BBox; no sumar longitud UTF-16 a offsets UTF-8 ni inventar offsets binarios PDF. Texto/links/bboxes/destinos se validan antes de UI y se sanitizan sin comandos.

Página sin texto sólo se anuncia tras extracción completada satisfactoriamente sin contenido Unicode ni fallos/unmapped pendientes. Fallo de API, permiso, límite, mapping ausente y surrogate roto son estados distintos de NoText; render gráfico puede seguir siendo válido. Cualquier página incompleta impide afirmar búsqueda exhaustiva sin coincidencias del documento. El criterio del gate cubre texto repetido, mapping parcial y errores además de UTF16 válido.

## Gates antes de integrar

1. Fijar binario+headers+wrapper+licencias/ABI y comprobar carga privada Windows propia; Linux/macOS nativos pendientes, no PASS por crosscheck.
2. Corpus sintético de tres páginas con etiquetas distintas de índices, tamaños/rotaciones/MediaBox-CropBox, texto UTF-16/surrogates/ligaduras/controles, página escaneada sin texto, destinos internos, enlaces inertes, protegido/corrupto/cero páginas y valores fuera de rango. Expectativas previas e independientes.
3. Parser/render/text/búsqueda y memoria en worker contenido; dimensiones extremas, snapshot/destino obsoleto, timeout/cancel/reap. Fijar cuotas bytes/páginas/texto/native/cache/bitmap/plazos con evidencia, antes de cerrar el contrato técnico. Job committed512MiB Windows no se renombra como RSS/AS portable.
4. Integración lazy+historial+inline/clipping/zoom/fallback en el producto exacto; q0/C130/restauración/ausencia de procesos y cache cobrados; adversario+AE independientes y QA visual real.
5. Corpus Clases sólo HEAD equivalente y sin modificar/copiar contenido privado al repo, tras gates sintéticos. Ninguna unidad se reescribe.

## Fuentes consultadas 2026-10-05

[PDFium fpdfview.h](https://pdfium.googlesource.com/pdfium/+/refs/heads/main/public/fpdfview.h), blob7c9328021a3e6459f42cd8feb08e6bac5d275324: llamadas seriales, LoadMemDocument64 conserva buffer, GetPageCount/LoadPage usa índice 0-based. [fpdf_text.h](https://pdfium.googlesource.com/pdfium/+/refs/heads/main/public/fpdf_text.h): extracción UTF-16, búsqueda y recursos explícitos; no asumir mapping exacto por longitud. [Wrapper candidato](https://github.com/ajrcarey/pdfium-render). Estas consultas a main orientan el gate; no fijan artefacto distribuido ni constituyen ejecución native.


## Respuesta a revisión R1

PDFG0-01: inline/redirección primera o seleccionada, header total y resto identificado, conforme Q08; no todas por defecto. PDFG0-02: dominio textual explícito fuera CropBox, offsets/n/N por ocurrencia, límites entre chunks/páginas/unmapped, UTF16 y estados de error separados de NoText. Revisión R1 original SHA74FF889B82D1BBC9424470B0513959EC3380EA3FAF49A031A8E2E474FFA793FC. Correcciones revisadas por adversario exacto sin críticas pendientes; ninguna API/cuota/corpus ejecutado ni capacidad PDF acreditada.

## Estado de aceptación

Revisión independiente final [PDF G0 R2](../reviews/PDF-G0-r2.md). Contrato observable aprobado para preparar gates, sin código, artefacto PDFium fijado, ejecución ni capacidad PDF acreditada. Documento temporal revisado SHA256 138387EF5223C35BA09A7D223E336E547C922DBB9658076708962324D49BCB04; la copia canónica añade este estado.

