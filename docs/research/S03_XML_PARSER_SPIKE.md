# Investigación preliminar del parser XML

2026-10-05. Preparación de RV-106 dentro de S03. [Contrato revisado](../sprints/S03-D4-XML.md); gate técnico todavía abierto. Experimentos privados, sin dependencia ni implementación XML en producción.

## Procedencia

quick-xml0.42.0, crate SHA256 41B1177FDF999D2321D3FB46FF47159D9C1FB9AD66A4879F8C50A0B504615E9B, VCS36a2c52a4f6c90878c3f60c3c3b5c62efaef28a9. Cargo original: edition2024, MSRV1.86, licencia MIT (Johann Tuffe2016), default features vacías, dependencia obligatoria memchr; encoding/serde/tokio optativos. Crate original intacto en .tmp/xml-research/quick-xml-0.42.0, sin vendoring ni patch. Spike privado resolvió memchr2.8.3; no añade decisión al lockfile del producto.

## Exploración ejecutada

24 entradas sintéticas mediante Reader::from_str + read_event, enable_all_checks(true), trim/expand desactivados por defecto. Build y ejecución Windows con Rust1.99 PASS. Resultado significa ejecución del spike; no aceptación del producto ni conformidad XML. Logs completos .tmp/xml-research/spike-output.log SHA08D08331608DD1CC1644E47BE931106A8180007D3B701DE1F3FE3ECB98CC8043 y spike-bom-fixed-output.log SHAB6AE59BC63F8C20C7430E743F2A7941F953B741B804A485B818C0DE0FDEED03E. main-original.rs conserva primera versión; src/main.rs la segunda.

- Eventos conservan Start/Empty/Text/GeneralRef/CDATA/Comment/PI/End ordenados sin DOM. Atributos conservan orden y el iterador detecta repetición de QName cuando se consumen sus errores; no comprueba repetición por nombre expandido.
- buffer_position es byteoffset, pero con BOM UTF-8 inicial el parser lo retira sin sumarlo al contador: 76bytes originales terminan en posición73; usar contador directamente produce rangos rotos, incluso límites interiores de Unicode. Segunda ejecución separó el prefijo de3bytes explícitamente antes de Reader y sumó base3 al convertir todos los rangos; selección raw exacta observada para declaración/CRLF/CJK/emoji/ref/End. No editar fuente ni suprimir BOM de la vista. Pendiente oracle independiente y cobertura de primer evento vacío/error.
- DOCTYPE con quotes conteniendo >, subset con comentario que contiene ] > y entidad literal produjo un único evento completo; subset sin cierre produjo UnclosedDoctype. DtdParser declara que supone DTD bien formado y no lo valida: estos ejemplos no prueban cobertura completa. GeneralRef custom se conserva sin expandir ni cargar recursos. Gate lexical quote/comment/PI/brackets y perfil de posición siguen abiertos.
- enable_all_checks detectó comentario con -- y atributos lexicalmente repetidos, pero permitió dos raíces, entrada vacía, texto exterior, raíz sin cierre al EOF, Name1bad, QNamea:b:c, prefijo no ligado, atributo con < literal, NUL, referencia ESC y PI targetXML. También aceptó versión1.1 y declaración ISO-8859-1 sobre texto UTF-8. El adaptador tiene que aplicar estado de documento, Name/QName/Char, namespaces y perfil version/encoding; no llamar a la API validación XML completa.
- resolve_char_ref rechazó surrogate y NUL, pero devolvió ESC; legalidad XML1.0 es gate independiente. U+009B literal se preservó como válido para comprobar presentación segura más adelante.
- normalized_value distinguió whitespace literal CRLF (espacio) y referencia &#xA; (LF), y resolvió amp. Puede reservar texto nuevo: no usarlo como asignación sin admisión propia. Leer eventos sobre slice evita buffer copiado por el llamante; el estado interno de nombres/namespace todavía necesita inspección de memoria/Job y cancelación.

## Gates pendientes

Oracle spans para BOM/Unicode/CRLF/atributos y delimitadores, parser state/Name/QName/Char/refs/declaration/DOCTYPE, namespaces y duplicados expandidos, cooked limitado sin expansión DTD, contadores/cancelación/allocations. Después revisión adversaria de candidato y arquitectura concreta. No integración anterior a estos gates ni sustitución de Y01–09/X01–09 por documentación.

## Fuentes primarias

[XML1.0](https://www.w3.org/TR/xml/), [Namespaces1.0](https://www.w3.org/TR/xml-names/), [quick-xml0.42 Event](https://docs.rs/quick-xml/0.42.0/quick_xml/events/enum.Event.html), [Config](https://docs.rs/quick-xml/0.42.0/quick_xml/reader/struct.Config.html). Inspección exacta de src/reader/mod.rs, slice_reader.rs, state.rs, src/parser/dtd.rs y eventos/atributos del crate descargado. Documentación y código respaldan posibilidades; no acreditan R3Viewer ejecutado con XML.

## Namespace: riesgo adicional comprobado

Cuatro casos con NsReader::from_str ejecutados aparte, log .tmp/xml-research/namespace-output.log SHAF898A82F541A6C7FCD8C1B8615DCE0444E0FD57A07D7730CCBA7AD58987022E5. NsReader resuelve xmlns:p="urn:x&amp;y" como URI lexical urn:x&amp;y, aunque normalized_value devuelve urn:x&y. Dos prefixes con urn:x&amp;y/urn:x&#38;y se resuelven a URIs distintas pese normalizarse al mismo valor; una declaración válida del URI reservado xml con referencia numeric final se rechaza por comparación lexical. No usar NsReader como autoridad sin corregir este gate. Código exacto NamespaceResolver::push pasa Attribute.value raw a add, with_checks(false) y deja de consumir ante error. Resolver propio sobre valores ya normalizados bajo presupuesto es candidato para arquitectura; no hay patch ni decisión implementada.

Scope default/rebinding/pending-pop funcionó en el ejemplo observado; no acredita casos restantes ni Name/QName/duplicados expandidos. xmlns por defecto y xmlns:prefix necesitan representación lexical preservada e identidad namespace separada, no inferirla de URI raw. Reader conserva internamente nombres abiertos en Vec/String aun con check_end_names=false; atributos guardan claves/hash para checks. Allocations internas bajo Job, propias bajo ledger, con límite de profundidad antes de continuar y cancel/deadline; detalle por revisar.
