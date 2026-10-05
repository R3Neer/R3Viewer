# Obsidian: resolución y escapes en tablas

2026-10-05. Investigación y nuevo defecto de compatibilidad abierto, OBS-PARSE01. No adopción de parser/adaptador ni cierre RV-301/304/305. [Contrato vigente](../spec/CONTRATOS.md), [modelo de documento](../refinement/MARKDOWN-DOCUMENTO-WIRE.md).

## Fuentes y consecuencias para el diseño

[Enlaces internos](https://help.obsidian.md/links): wikilinks de notas pueden omitir .md; otros adjuntos necesitan extensión. Las rutas explícitas usan barras / y raíz de bóveda incluso en Windows. Destino, etiqueta mostrada, encabezado y bloque son partes distintas. Encabezados pueden expresar jerarquía con varios #; bloques usan #^id. No asumir que un slug HTML reproduce la búsqueda de encabezados de Obsidian. Resolver nombres abreviados mediante índice y mostrar ambigüedad, conforme al requisito del usuario; nunca escoger el primer archivo.

[Incrustaciones](https://help.obsidian.md/embeds): distingue ![[...]] de navegación, admite notas/encabezados/bloques y PDF#page=N; las imágenes admiten ancho o ancho×alto en su etiqueta. La página necesita validación contra el PDF, no basta aceptar un número de sintaxis. Height de PDF figura en la documentación, pero su política de layout requiere refinamiento separado. Audio/canvas/search de esa página no amplían automáticamente el alcance acordado.

[Aliases](https://help.obsidian.md/aliases): la propiedad es lista YAML; al seleccionar un alias Obsidian genera un enlace al nombre real con texto alternativo. No justifica convertir automáticamente cualquier [[Alias]] en resolución por metadata. Mantener separación entre etiqueta de presentación y destino; índice/precedencia de aliases requiere contrato y pruebas, sin ejecutar código de frontmatter.

[Tablas](https://help.obsidian.md/advanced-syntax): la barra de separación de alias o tamaño se escapa con backslash dentro de una celda. Ese backslash estructural no representa un separador de ruta Windows. La adaptación debe conocer el contexto TableCell y el token/evento, preservar bytes fuente y verificar origen; no aplicar sustitución global de backslashes ni otro unescape al contenido completo.

## Corpus privado: observación sintáctica, no oracle semántico

Inspección autorizada de HEAD fba2da9086a30cc36666b6ab86ed5acc29681238 en D:/Universidad/Clases, comprobado estable y limpio:210 Markdown de Asignaturas y66PDF. Inventario por regex encontró1654 tokens sintácticos,41 embeds con PDF#page y105 prefijos crudos terminados en backslash antes de |. Son candidatos de contexto, no1654 enlaces semánticos validados. Las muestras examinadas de ese último grupo usan escape de alias en tabla; la regex que divide por | deja ese backslash. No se deduce que existan105 rutas Windows. El índice futuro debe producir eventos/adaptaciones antes de interpretar destinos.

Inventarios privados OBSIDIAN-HEAD-inventory-r1.json B91BA5986EB9574C85C628BA2610A94CA7DE0BCADF755F38CEA06745AB802AFF, pathparts34DF403124B2BE3ADDA10E40818399D7BB879F46D4263D8CDB255ED1A016A4E4 y backslash8EAA4556942398ECC2B74B73D887E9B3CAC916D12035163437DE1894785F342B conservan los conteos originales. No se copiaron notas ni PDFs al repo. La ausencia de basenames PDF duplicados en ese HEAD no permite eliminar el manejo de ambigüedad del producto.

## OBS-PARSE01: reproducción sintética nativa

Cinco fixtures propios y expectativas anteriores a output en obsidian-table-parser-probe-r1. Parser R3 exacto MANIFEST367B4BF11E313185963EABD4EBD1B48193B13DE7E879DD0702F50E1A04F87686, bin470B8355A63713315B51BC23A4C65A567C602C6E568B5D2ACCADB4B3B1C0A4C3. Fuente/parser congelados, copia del runner de ownership intacta. No recompilación ni modificación del candidato.

| Caso | Esperado antes de output | Observado | Resultado |
| --- | --- | --- | --- |
| Alias en párrafo | destino Nota, etiqueta Alias | Coincide | PASS |
| Alias en celda | destino Nota, etiqueta Alias | Destino termina en backslash | FAIL |
| Embed PDF en celda | Documento.pdf#page=3 | Fragmento termina en backslash | FAIL |
| Embed imagen en celda | Imagen.png y etiqueta100x145 | Destino termina en backslash | FAIL |
| Wiki aparente en código inline | Ningún enlace | Ningún enlace | PASS |

Oracle7B0DB2DDF792B48445C0C4004CF9C034C64DBA1C6DE1F5D18306025D0F3CB87F; resultados599B571C8995B88A37ADE464DA11E025228BD5998F30C15EA913671B7D639AA5.19 archivos de evidencia con manifiesto8BFBEAE1C8A32860D53F106854DFBE955C842C2898670ED06BC0B40655A6862C. Las cinco invocaciones tienen exit0, wait/handles/streams/readers cerrados sin rescate; éxito de lifecycle no convierte tres discrepancias semánticas en PASS.

## Siguiente aceptación

Revisión independiente y contrato de adaptación contextual antes de integrar documento: alias/imagen/PDF en encabezado y cuerpo de tabla, escapes repetidos y pipes literales, código/HTML/Markdown ordinario inertes respecto a esa transformación, offsets y source exactos, dimensión y fragmento preservados, cotas/cancelación y rutas ambiguas. La selección de parser sigue viable, pero aceptar sólo su destination actual rompería enlaces reales. El gate previo no certificaba este caso; se conservan su alcance y estos fallos nuevos sin ajustar expectativas. Resolución, render, ciclos, rutas y QA visual completos permanecen abiertos.
