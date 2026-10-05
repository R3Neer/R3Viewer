# R3Viewer: análisis de HTML y páginas web

Fecha: 2026-10-04. Estado: análisis; recomendaciones salvo decisiones explícitas registradas en DECISIONES.md. No se ha implementado ni probado en ejecución.

## Intención confirmada

Lectura de HTML semejante al contenido de una página sin CSS, usando convenciones visuales Markdown. Compartir presentaciones interactiva e inline con el lector de documentos. Nombre de repositorio aprobado: D:\Tools\R3Viewer.

## Arquitectura recomendada

HTML o URL -> obtención y decodificación -> parser HTML5 -> árbol HTML -> adaptación al modelo común de documento -> presentación interactiva o inline.

Compartir compositor con Markdown, sin exigir serializar HTML a una cadena Markdown. Conservar URL de origen, URL final tras redirecciones, base efectiva, identificadores y destinos. El árbol HTML estructurado para Nushell conserva elementos y atributos aunque no se muestren en la vista de lectura.

Un parser HTML5 tolerante es necesario: HTML real no equivale a XML. Candidatos: html5ever en Rust o parse5 en Node; lenguaje y motor todavía sin elegir. No incorporar Chromium solamente para leer HTML estático.

## Mapeo visual propuesto

| HTML | Lectura |
|---|---|
| h1–h6, p, br, hr | Títulos jerárquicos, párrafos, saltos y separadores discretos. |
| strong, em, del, blockquote | Énfasis, tachado y citas según capacidades de terminal. |
| ul, ol, dl | Listas, numeración y pares término/definición. |
| pre, code | Preservar espacios y saltos; resaltar si existe lenguaje identificable. |
| a, id | Enlaces navegables e índice de destinos internos. |
| table | Tablas simples; conservar rowspan/colspan y degradar tablas complejas de forma explícita. |
| img, figure, figcaption | Texto alternativo y pie; gráficos usando backend común. Recursos remotos sujetos a la política de carga explícita. |
| checkbox/radio | Representar estado como [ ]/[x] u otra marca equivalente. Interacción pendiente; propuesta inicial solo lectura. |
| input, select, textarea, button | Etiqueta, tipo y valor no sensible legible; no enviar formularios. Política concreta pendiente. |
| details, summary | Resumen y contenido; expansión en TUI posible, inline debe seguir siendo comprensible. |
| audio, video, iframe, canvas | Identificar recurso o contenido alternativo disponible; reproducción y documentos anidados no incluidos por inferencia. |
| script, style, metadatos head | No presentar código como prosa; conservarlo en inspección estructural y fuente. |

No se busca fidelidad de píxeles con un navegador: cambia el flujo, ancho y representación de controles. Mantener navegación, header/footer y texto por defecto; extracción automática de artículo sería opción independiente y puede omitir contenido.

## Límites y casos importantes

- Sin ejecución JavaScript: páginas cuyo contenido lo genera JS pueden quedar vacías o incompletas. Comunicar esta limitación sin adivinar lo que falta.
- Ignorar CSS implica que clases de ocultación visual, menús duplicados y maquetación no se comportan como en el navegador. Definir aparte hidden y otros atributos semánticos.
- No activar por inferencia scripts MathJax/Mermaid de una página. MathML, fórmulas delimitadas y bloques Mermaid reconocibles requieren contratos explícitos; no interpretar cualquier dólar como fórmula.
- URL relativa depende de origen y base, incluso después de redirecciones. Una cadena recibida por tubería puede carecer de esa información: aprovechar metadatos si existen y sobreviven; considerar opción base explícita.
- Introducir una URL en viewer es acción explícita para descargar el documento; no equivale a autorizar cargar todas las imágenes, iframes o destinos enlazados.
- Codificación de texto, entidades, HTML mal formado, URLs relativas, cancelación, tamaño y tiempos de red forman parte de la robustez. Limpiar controles de terminal del contenido recibido.
- URL que responde con PDF/imagen/otro formato debe dirigirse por contenido al adaptador apropiado, sin forzar HTML.

## Nushell: comportamiento verificado documentalmente

- open abre archivos y busca from <extensión> en el ámbito; --raw/-r evita conversión. No ofrece transporte HTTP nativo.
- No aparece conversor HTML integrado en el catálogo from consultado. Sin conversor adicional no se obtiene automáticamente un árbol HTML.
- http get realiza transporte de URL y ofrece --raw/-r. En el código upstream consultado, text/html produce la búsqueda de from html; si falta conversor devuelve el cuerpo sin esa conversión.
- query web pertenece a nu_plugin_query: extrae por selectores, atributos o tablas. Es una capacidad complementaria; no confundir extracción con parser completo ni asumir que el plugin está instalado.

Propuesta de extensión opcional: proporcionar from html. Así open pagina.html produce estructura y open -r pagina.html conserva fuente; http get URL puede convertir según Content-Type y http get -r URL conserva cuerpo. No requiere sustituir open ni compilar un fork de Nushell. Verificar posteriormente contra versión instalada, registro del conversor y metadatos de pipeline.

Propuesta de modelo: documento con nodos anidados, atributos y nodos textuales separados conservando orden; la tabla plana es una proyección opcional con id, parent, order, kind, tag, attributes, text. No confundir esta tabla del DOM con las tablas incluidas en la página. El árbol parseado puede insertar elementos implícitos o reparar anidación; la fuente original se obtiene por raw.

open URL sería wrapper o cambio upstream adicional, con diferenciación de errores locales/HTTP y política de red. Se desaconseja por ahora: la pareja open archivo / http get URL ya es idiomática en Nushell. viewer URL sí encaja como comodidad del lector.

## Complejidad relativa

- HTML estático básico como adaptador del visor: media, aprovecha compositor existente.
- Conversor from html con parser disponible: baja/media; el contrato de datos y preservación requieren más trabajo que el puente técnico.
- Robustez web y navegación, tablas complejas y medios: media/alta.
- Fidelidad de navegador con CSS, JS, sesiones y formularios: producto mucho mayor, fuera de esta propuesta.

## Pendientes

Q24 cerrada: controles solo muestran estado, sin modificación ni envío de formularios. Q25 cerrada: incluir from html como utilidad opcional. Las propuestas previas deben leerse con estos cierres.

Pendientes técnicos/de cobertura: tablas combinadas; modo artículo; MathML y convenciones matemáticas/Mermaid; cabeceras/autenticación y sesiones; base de documentos por tubería; HTML del visor frente al árbol estructurado; versión de Nushell objetivo. No se incorporan automáticamente al alcance funciones avanzadas por aparecer en esta lista.

## Fuentes

- [open](https://www.nushell.sh/commands/docs/open.html)
- [http get](https://www.nushell.sh/commands/docs/http_get.html)
- [from](https://www.nushell.sh/commands/docs/from.html)
- [query web](https://www.nushell.sh/commands/docs/query_web.html)
- [Conversión HTTP upstream](https://github.com/nushell/nushell/blob/main/crates/nu-command/src/network/http/client.rs)
- [html5ever](https://github.com/servo/html5ever)
- [parse5](https://parse5.js.org/)


## Contraste de ejecución posterior

Nu0.115.1 Windows: `open` ejecuta un `from html` custom temporal y raw conserva fuente, pero `http get` no puede ejecutar ese mismo comando custom: su ruta llama Command.run directamente y devuelve error interno. La evidencia y primarias fijadas están en [inspección HTML](../research/HTML_PARSER_API.md). Q25 sigue vigente; registrar un conversor apto para ambas rutas y conservar activación opcional es trabajo pendiente. Se contrastará la vía de PluginCommand, sin sustituir open/http get ni instalar perfiles por esta investigación.
