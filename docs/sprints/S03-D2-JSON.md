# RV-106 siguiente corte — JSON: propuesta delta R1

2026-10-05. Contrato observable revisado sin críticas pendientes; parser/adaptador aún no aprobados ni implementados. TOML D1 continúa como corte activo. Base: S03-D1.md, ARQUITECTURA.md, structure_model.rs actual y r3viewer-rv106-refinement-r1.md. No otro framework ni revisión de metodología.

## Decisiones de producto compatibles

Reutilizar Fuente/Estructura, --structure, Tab/BackTab soloPress, `o`, búsquedas separadas, límites/cancelación/ownership/publicación/inline de D1. Añadir --format json; Auto/Code infieren JSON exclusivamente de extensión local real .json. Pipeline/literal textual requieren selector explícito; --language no selecciona parser. Valores Nu ya tipados conservan navegación y nunca se convierten a JSON ni se reabren. Gramática JSON integrada existente se acredita con candidato final, no bundle TOML nuevo.

Interpretación RFC8259: raíz puede ser cualquier valor. Un documento exacto y EOF con whitespaceJSON permitido; vacío no es objeto vacío. JSON5/JSONC/NDJSON, comentarios, trailingcomma, NaN/Infinity y varios valores concatenados NO son extensiones implícitas. Fallo estructural conserva fuente completa0+diagnóstico, nunca prefijo o from-json serialización. CRLF/whitespace/origenFuente no se reconstruyen.

Objeto conserva todas las entradas en orden documental, incluso claves repetidas; identidad ordinal existente, jamás HashMap/first/lastwins. Detectar repetición por clave decodificada exacta: a y \u0061 equivalentes; no casefold ni normalizaciónUnicode. Ready puede incluir advertencia acotada de duplicados; no convertir esta advertencia en Unavailable ni negar acceso a ocurrencias. El orden mostrado es documental, no una garantía de semántica ordenada de los objetos JSON. La proyección de tablas solo es elegible con claves textuales únicas por objeto; objeto repetido conserva vista de entradas, sin columnas que pierdan valores.

Número conserva lexema validado completo como autoridad; no rangoi64/f64, redondeo ni BigInt/decimal aritmético. 9007199254740993, entero mayor dei64,1e400,-0 y1.000 todos inspeccionables con categoría número; string"1" sigue String. `null` es hoja semántica propia, distinto de ausencia de key/emptystring/emptyarray/emptyobject y de NuNothing como tipo recibido.

Strings/keys válidos se decodifican sin pérdida; spans completos raw inclcomillas y escapes. ParUTF16surrogate válido produce scalar; surrogate aislado permitido por la gramática pero no representable como StringRust: recomendar Unavailable local con diagnóstico de representaciónUnicode y fuente íntegra, nunca U+FFFD silencioso ni etiquetarlo equivocadamente error sintáctico. Esto explicita una degradación de representación, no afirma soporte cookedUTF16 completo. Adversario debe evaluar esta política contra requisito de conservación; no esconder el gate bajo parserestricto.

## Delta técnico mínimo sobre modelo existente

Mapping/Sequence/String/Boolean y EntryId ordinal ya sirven. Añadir únicamente clases Null y JsonNumber (lexema fuente), sus casos de validación/IPC/descriptor; no fingir que Integer/FloatTOML imponen representaciones nativas a JSON. No copiar wire::Value ni usarlo como dominio. Objeto/array tienen span contiguo validado completo; key/value/member spans propios. Si se reusa índice de declaraciones, introducir rol JSONMember explícito o dejarlo vacío documentadamente: no fabricar AssignmentTOML ni rutasdot para keysJSON. La representación de key"a.b" es una única clave y no jerarquía.

Candidato parser Jiter0.17.0 pull, fijado y auditado antes spike: next_number_bytes/current_index/next_key y finish, con pila acotada y arena directa. No next_value→DOMmap que colapse repetidos; no skip como oráculoUnicode. API documental no garantiza por sí sola rangos, unicidad de ocurrencias ni aceptación de todos lexemas grandes; spike debe comprobarlos. Mantener proceso estructural propio existente: tokens grandes pueden ser llamadas no cooperativas. Reusar selector/formato en Init del protocolo propio, versionando los nuevos discriminantes; no otro transporte ni importación de IPCsintaxis.

Advertencia de duplicados debe ser acotada en metadata/modelo y salida, p.ej. flag+contador saturado validado y primeraubicación: no almacenar mensajes/hashsets ilimitados fuera64MiB. Si detección exacta no cabe, estructura completa debe degradarse por presupuesto; nunca omitir arbitrariamente entradas o afirmar unicidad. Tabla/cellpath y búsquedas siempre resuelven ordinal, aunque etiqueta sea igual.

## Aceptación observable específica J01–J09

| ID | Corpus y resultado |
|---|---|
| J01 Selección | Fuentedefault .json con whitespaceCRLF; explícitoJSONliteral/StringCSV; homónimo/metadata no inferidos/reabiertos; languageToml cambia solohighlight. CLI incompatible y codes siguen D1. |
| J02 Raíz/null | null,true,0,"",[],{} y árbol[{"x":null},{},[false,"",{}]] distinguidos. Emptyinput/trailinggarbage/segundovalor fallan a fuente0; nullroot no se confunde noinput. |
| J03 Númeroexacto | 9007199254740993,9223372036854775808,1e400,-0,-0.0,1.000,1E+0003 y fracción100dígitos mantienen bytes/descriptor/hoja completos. +1,01,1.,1e,NaN,Infinity rechazados como JSONsinextensiones. No compararroundtripf64 como oráculo. |
| J04 Duplicados/orden | {"b":0,"a":1,"\u0061":2,"a":3}:4entradas visibles en orden, tresvaloresa distintos accesibles/buscables con ordinal; advertencia discreta. Unicodeé vs ecombining distintos. AoObject heterogéneo con un repetido no oculta ocurrencias en tabla. |
| J05 Spans/Unicode | KeyUnicode antes raw"\u0041"; cookedA search y o saltan rawspancompleto, nunca cookedoffset. Par\uD83E\uDD8A→🦊; surrogateaislado diagnóstico representación+fuenteintacta. Delimitadores y whitespace no alteran rangos bytes; keya.b no dotted. |
| J06 Escapes/seguridad | \b/\f/\n/\r/\t/\u001B/\u009B y glifo␛literal presentados seguros, noOSC/ANSI; \q y controlraw inválidos. SurrogateUTF8 inválido o bytes noUTF8 mantienen frontera de entrada vigente sinreplacementoculto. |
| J07 Profundidad/nodos | Objetos/arraysmixtos depth64/65, índiceordinal y leafgrande>4096; límites/deadline sourcefallback completo, ningún Ready parcial. Memoriawarningduplicados y cooked contabilizados; splitsUTF8chunk y EOFfinal validan. |
| J08 Controles/aislamiento | CambiarFuente/Estructura mantiene consultas/últimoshits; o+n/N conserva dominioFuente; CancelBuilding→NotRequested/manualretry vsUnavailablelatched; Syntax/parserreaped antes nuevosspawn. Misma aceptaciónD1 sin inventar SLA ni repetir su framework. |
| J09 Regresión | TOML cuatrofechas/lexemas/duplicadosinválidos conservados; Nuorderedrecord/heterogeneoustable/nothing/units/binary/range/closure no efectos ni conversión; CLIinline/redir sinANSI. Hashescandidatoexacto y QAvisual/hosts separados, sin cierreRV completo. |

Spike de Jiter pendiente antes detalle final: verificar `finish()`rejects trailing; rawnumbernooverflow; rawkey/string offsets alrededor escapedUTF16; duplicados no colapsados; EOF/64vs65 y token muy largo bajo deadline/proceso. Si falla API, alternativa visitor/raw custom serde_json preservando secuencia, no Value-map; misma aceptación sin rebajar precisión/duplicados.

Referencias primarias consultadas: [RFC8259 secciones2/4/6/8.2](https://www.rfc-editor.org/rfc/rfc8259.txt) y [Jiter0.17 API](https://docs.rs/jiter/latest/jiter/struct.Jiter.html). La elección de orden documental, conservación de duplicados, modelo/fallback/budgets es política de R3Viewer; JSONnorma no exige una implementaciónfirst/lastwins. Nada aquí modifica producción ni acredita ejecución. Sigo disponible para AE TOML cuando Root congele candidato.

Cierre del contrato en [revisión del plan](../reviews/S03-D2-plan.md). El spike de Jiter y su adaptación rawOnly se revisan por separado; no cerrar RV-106 ni comenzar integración del siguiente corte antes de disponer del núcleo D1 revisado.
