# YAML: investigación inicial de eventos y posiciones

Estado: experimento del autor; no es gate aprobado ni soporte del producto. RV-106 sigue pendiente para YAML/XML y QA general. El refinamiento YAML ocurre mientras se acepta JSON, sin integrar otro incremento.

## Candidato y procedencia

Saphyr-parser 0.1.0, edición 2024/Rust mínimo 1.85, MIT o Apache-2.0. Crate original privado `.tmp/yaml-research/saphyr-parser-0.1.0.crate`, SHA256 7429158804C36E705D9423B7848018E535B3E92D439A763E364EE12670EF473B; VCS 253682db98de0680fb20b1a965538a030baceddc, path_in_vcs parser. No añadido al Cargo.lock del producto, ni vendorizado en producción.

API pull con eventos Scalar(Cow<str>, estilo, anchorID, tag), Alias(anchorID), Sequence/Mapping, Document y Stream. Permite observar pares repetidos, claves complejas, múltiples documentos y referencias sin construir DOM ni expandir alias. Eso no demuestra conformidad completa ni validación de unicidad semántica de claves.

## Ocho experimentos ejecutados

Log original `.tmp/yaml-research/spike-output.log`: Unicode no ASCII, comillas/escapes/comentarios/CRLF, bloques literal/folded/chomping, alias cíclico, etiquetas, documentos múltiples, claves complejas y pares repetidos. Parser acepta `&foo [1, *foo]` sin expandirlo y devuelve cada par repetido. Etiquetas son información, no constructores ejecutados.

Se verificó una contradicción documental: Marker.index dice bytes en el getter pero implementaciones y resultados cuentan escalares Unicode. `é: 🦊\n后: x\n` tiene16 bytes/10 caracteres: fox tiene marker3..4 y no bytes3..4. Hace falta conversión explícita validada a bytes UTF8, con índice/presupuesto y cancelación; nunca publicar estos markers como SourceSpan directamente.

Dos rangos originales requieren tratamiento: DoubleQuoted incluye whitespace/comentario posterior, y Literal/Folded comienza en el primer carácter del cuerpo excluyendo header/indicadores. Los contenedores, propiedades tag/anchor y delimitadores de documento necesitan corpus adicional; no se consideran resueltos.

## Adaptación temporal de rangos

Candidato `.tmp/yaml-research/saphyr-parser-token-spans` conserva original aparte. Scanner original SHA256 7ADE9225E788FA5342E0F80CDE363BC08E7A3371D726DC5376263F36D68A4BC5; derivado 6ACD62AADAD8F520CD63074A462BC2A522F5FA014E7CD10A3A3BF15A0F9F8092. Diff `.tmp/yaml-research/token-spans.patch`:

- Guardar token_end después de consumir comilla final, antes de comprobar trivia posterior; usarlo como final del token, preservando validación de trailing content.
- Eliminar shadow de start_mark en blockscalar para conservar inicio del header; plegado/chomping sin cambios.

Log `.tmp/yaml-research/span-spike-output.log` sobre los mismos ocho casos: quoted `"\u0041"` ahora char3..11 y excluye comentario; Literal `|+\n  🦊\n  后\n\n` ahora char3..15; Folded comienza en `>-`. Estos son resultados puntuales, no aceptación de todas las posiciones ni equivalencia de gramática. Patch y mapping deben pasar revisión independiente y corpus ampliado antes de integración.

## Próximos gates

Contrato observable de árbol de serialización (orden/pares, claves complejas, stream, alias sin expansión, etiquetas inertes y scalars lexical/cooked), revisión adversaria; después diseño incremental sobre controlador/arena existente. Probar todas rutas Unicode y saltos de línea, comillas multiline/escape, block empty/indent/chomping/document boundary, propiedades anchor/tag y contenedores. Definir honestamente qué comprobaciones de esquema/unicidad se realizan, sin anunciar un compositor YAML completo por usar un parser de eventos.

## Referencias primarias

[YAML 1.2.2](https://yaml.org/spec/1.2.2/), [Saphyr-parser Event](https://docs.rs/saphyr-parser/0.1.0/saphyr_parser/enum.Event.html), [Parser](https://docs.rs/saphyr-parser/0.1.0/saphyr_parser/struct.Parser.html), [Marker](https://docs.rs/saphyr-parser/0.1.0/saphyr_parser/struct.Marker.html), [Span](https://docs.rs/saphyr-parser/0.1.0/saphyr_parser/struct.Span.html), y fuente exacta del crate indicada arriba. Las políticas de R3Viewer se deciden separadamente de las capacidades de la librería.

## Ampliación:19 casos y nuevos gates

Logs `.tmp/yaml-research/spike-expanded.log` y `span-spike-expanded.log`: añaden directivas1.1/1.2/1.3, streamvacío, dos documentosvacíos, rebindanchors, alias entre documentos, blockvacío, indent/chomping/commentCRLF, quotesmultiline y propiedades tag+anchor. Ambos candidatos aceptan versiones1.1/1.3 sin reportarla en EventDocumentStart; no pueden satisfacer por sí solos la política de versión de R3Viewer. `parser_process_directives` tiene la comprobación de versión comentada. Resolver antes gate, sin inferir desde texto scalar.

Un anchor definido en documento anterior se resuelve por parser como Alias(1) en siguiente documento sin nueva definición. R3Viewer debe rechazar ese caso y mantener alcance por documento; no confiar en IDs emitidos como prueba de scope. El rebind dentro del mismo documento sí produce IDs1 y2 y alias correspondientes, como se requiere.

En documento vacío `---\n...`, el evento Scalar("",Plain) sintético trae el rango del delimitador `...`. No es un token scalar literal: proyección debe acreditar si hay token real antes asignar SourceSpan. Streamvacío no contiene DocumentStart; no confundirlo con null explícito/documento vacío. La adaptación de rangos aún no resuelve esta distinción ni scope/version: todos siguen gates previos al detalle técnico e integración.

Los casos ampliados confirman en experimento quotedmultiline completo, blockvacío con header y blockindent+commentCRLF con header. Rangos siguen en caracteres; convertirlos a bytes permanece obligatorio. No se declara gate cerrado por estos19casos.

Comparación del autor de los178eventos/errores de19casos, excluyendo posiciones: original y patch conservan la misma secuencia y contenido. Resultado `.tmp/yaml-research/expanded-comparison.json`; logs originalSHA9708C37A7EEA559D3BAFC5E37B8574E5536CB7997158CBA635281C0B25897A6F y derivadoSHA DBE2E399AC3EEFFC40B5F08E72FDB79CFE4F40262DEA6995DA45EA8BA6629BA4. No demuestra equivalencia fuera de ese corpus ni resuelve versiones/scope/syntheticspans.
