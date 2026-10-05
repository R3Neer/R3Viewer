# S03 D4 XML — arquitectura concreta del núcleo R1

2026-10-05. PROPUESTA PARA REVISIÓN. No implementación ni gate cerrado. RV-106 continúa abierto. Este documento refina X01–X09 de docs/sprints/S03-D4-XML.md sin sustituir ese contrato. No convierte XML en registros Nu ni altera TOML/JSON/YAML. Preparado leyendo el contrato, docs/research/S03_XML_PARSER_SPIKE.md, el código actual structure_model/protocol/controller/worker y yaml_model/view, y el corpus acceptance-xml-preparation.

## 1. Decisión y evidencia

Proponer quick-xml 0.42.0 únicamente como lector de eventos sobre un slice UTF8, con adaptador de perfil propio. No usar Serde, read_text, read_to_end, DecodingReader ni NsReader como autoridad semántica. enable_all_checks(true) activa comentarios/endnames; no cubre el perfil requerido ([Config exacta](https://docs.rs/quick-xml/0.42.0/quick_xml/reader/struct.Config.html)).

El spike existente ejecutó 24 casos Reader y 4 NsReader en Windows; no se reejecutó aquí. El informe registra crate SHA256 41B1177FDF999D2321D3FB46FF47159D9C1FB9AD66A4879F8C50A0B504615E9B, MSRV1.86, MIT, features default vacías y memchr obligatorio. El producto todavía no contiene XML. Las observaciones que condicionan esta propuesta son: contador omite BOM inicial3bytes; EOF no exige raíz cerrada; no verifica varias restricciones de nombres/caracteres/declaración; NsReader compara xmlns raw. namespace-output.log confirma que amp/numeric equivalentes dan URIs distintas y que xml URI válido codificado se rechaza. No se propone parchear el crate.

Los 31 fixtures y22selecciones raw existentes son oráculos de preparación independientes, no ejecución del adaptador. Conservar sus hashes y añadir suplementos versionados para las lagunas; nunca ajustar el oracle al resultado del candidato.

## 2. Fronteras concretas y orden del trabajo

Añadir módulos xml_worker (adaptación), xml_model (validación e invariantes) y posteriormente xml_view/search. Incorporar StructureFormat::Xml con discriminante nuevo, conservando los actuales Toml0/Json1/Yaml2; kinds XML nuevos al final del enum. Reutilizar Arc<String>, SourceSpan, TextRef, NodeId, EntryId, StructureBuilder, StructureController, ChildJob y el transporte privado. Los enums del protocolo no se asumen abiertos: actualizar encoder/decoder y rechazar tags desconocidos.

El controlador conserva propiedad del hijo, generation/request, fuente inicial, deadlines y latch de fallo. El worker recibe únicamente la fuente inmutable. Su pipeline es: UTF8/BOM/perfil → cursor lexical acotado → evento Reader cotejado → cooked mínimo y resolución namespace → arena provisional → validación XML final → Begin/frames/End. Solo el receptor completamente validado emite Ready. Fallo en cualquier punto destruye todo el resultado provisional; Fuente permanece completa en posición0 cuando la entrada textual es válida. Cancelar Building vuelve a NotRequested; no auto-retry de Unavailable.

A no incorpora XML a la selección de entrada ni a la presentación hasta cerrar los gates de parser/modelo/IPC. B incorpora entrada, vista y búsquedas sobre ese candidato cerrado. C es aceptación y revisión independiente del binario exacto. Esto permite contratos de interfaz verificables sin llamar producto al núcleo.

## 3. Modelo propuesto y significado de cada tabla

Structure conserva las tablas generales nodes/entries/cooked/parents y añade xml_metadata: Vec<XmlMetadata> ordenada por NodeId. Una fila por nodo XML, incluido Document. No usar YamlMetadata, DeclarationSpec TOML ni claves Mapping para codificar XML. finish_xml valida estas tablas; finish TOML/JSON y finish_yaml conservan sus invariantes.

Kinds: XmlDocument, XmlElement, XmlAttribute, XmlNamespaceDeclaration, XmlText, XmlReference, XmlCData, XmlComment, XmlPi, XmlDeclaration, XmlDocType. Document es wrapper sintético location=None, aunque sus hijos físicos tienen span; solo Document/Element contienen entries. Todo atributo y declaración namespace es un nodo seleccionable, con padre y ordinal. Element.entries contiene primero sus atributos/declaraciones namespace intercalados en orden lexical, después contenido mixto en orden fuente; XmlMetadata lleva attribute_count para partir ambas regiones, sin añadir grupos sintéticos gratuitos. Todos los EntrySpec tienen key=None. Document.entries contiene whitespace exterior/Decl/DocType/Comment/PI/raíz en orden fuente. Los hermanos repetidos son nodos distintos. No hay DAG, alias, IDREF ni agregación por nombre.

XmlMetadata es un discriminated record de campos fijos (sin String/Vec por fila):
- Document: root_element NodeId, has_doctype warning.
- Element: XmlName, attribute_count, empty_form bool, open_span y close_span opcional.
- Attribute/NamespaceDeclaration: QName span, quoted_value span, value_content span, decoded TextRef, y XmlName o binding_prefix opcional; None prefix significa default binding. Namespace declaration tiene identidad propia; no fingir atributo ordinario con URI xmlns raw.
- Reference: predefined/numeric kind, rawname span, decoded TextRef de un único carácter.
- PI: target span y data TextRef; Comment/CDATAText/Text: contenido TextRef; Declaration y DocType: raw payload span y campos de perfil mínimos realmente acreditados.

XmlName contiene qname/local SourceSpan, prefix Option<SourceSpan>, namespace Option<TextRef>. Ausencia namespace se expresa None, nunca URI vacía. URI xml implícita se guarda una vez en cooked y se comparte por TextRef. URI normalizada de xmlns se referencia desde el valor cooked/source ya retenido, sin copia por uso. Cada campo incompatible con kind debe estar ausente; wire usa layout explícito, no memoria Rust cruda. Names/local son raw y no sufren normalización Unicode.

TextRef::Source solo cuando el contenido completo coincide exactamente con cooked esperado y no incluye delimitadores. Si EOL o referencias cambian bytes, usar cooked. Nunca convertir valores a bool/int/null. No unir nodos de texto a través de referencias, markup o CDATA. <e/> y <e></e> difieren por empty_form y spans; ambos tienen cero contenido, sin inventar hoja string vacía.

## 4. Cursor lexical, eventos y rangos

La fuente UTF8 conserva BOM. Pasar al Reader source[base..], base=3 solo si los tres primeros bytes son EF BB BF; en todos los demás casos0. Un U+FEFF posterior no es BOM inicial. Cada conversión de buffer_position/error_position comprueba overflow y suma base. No calcular start como end-payload.len().

Antes de read_event, LexicalCursor reconoce el siguiente token sobre source, sin copiar ni almacenar una lista global. Conserva solo estado/índices y un stack acotado. Valida delimitadores y partición esperada (incluidas GeneralRef), contabiliza longitud y profundidad antes de que Reader pueda copiar un nombre abierto. Su span completo y clase deben concordar con evento y contador ajustado; desacuerdo → XmlRepresentation, nunca elegir arbitrariamente una de las dos coordenadas. Esta doble frontera es un gate experimental: su correspondencia real debe probarse antes de comprometer implementación. El scanner no es un segundo DOM.

Element.location abarca apertura hasta cierre inclusive; empty incluye />. Nodes Text/Ref/CDATA/Comment/PI/Decl/DocType conservan todo token físico. Attribute.location empieza en QName y termina después de quote final; excluye whitespace precedente y siguiente. quoted_value incluye quotes, value_content las excluye. El scanner de starttag reconoce S XML, QName, S? '=' S? y quotes correspondientes, sin interpretar '>' dentro de valores. Conserva todas las posiciones, no usa pointers de Cow. Close span incluye S antes de > y se valida su gramática.

Todas las coordenadas son UTF8 bytes start0/endexclusive sobre fuente original. Comprobar source bounds, char boundaries, igualdad de QName esperado, clase de delimitadores y contención del atributo en open_span. Referencias raw tienen span &...; completo. Metadata sintética es la única candidata a None. Los errores pueden llevar offset fiable independiente, pero no necesitan fingir un nodo parcial.

## 5. Perfil y normalización

Estado documento: AtStart → Prolog (máximo un DOCTYPE) → InRoot(stack) → Epilog → Eof. Declaration solo al byte base inicial, antes de cualquier whitespace/comment/PI. Prolog/epilog permiten S, comentarios y PI; rechazan CDATA/referencias/texto no S; una sola raíz obligatoria, stack vacío al EOF. End compara QName lexical exacto, no solo nombre expandido. Empty abre/cierra el scope y cambia estado raíz sin push persistente. Depth cuenta elementos, Document0, raíz1; 64permitido65rechazado antes de read_event del Start problemático.

Validar producciones XML1.0 quinta edición NameStartChar/NameChar explícitas, sin confundirlas con Unicode alphabetic. Elementos/atributos son QName con cero o un colon y ambos componentes NCName no vacíos. PI target NCName según Namespaces§7 y no cualquier combinación ASCII-case-insensitive de xml. No prohibir por inferencia todos los nombres que comienzan xml: los prefijos reservados adicionales tienen tratamiento normativo distinto. Namespace profile excluye conformidad interna de DTD.

Recorrer caracteres de fuente y contenido cooked: XML1.0 Char admite tab/LF/CR, U0020–D7FF, E000–FFFD y10000–10FFFF; rechazar NUL, ESC y surrogates; U009B es legal. Rechazar '<' literal en atributo, ']]>' en character data, comentarios '--' o final '-' antes del terminador. References: cinco predefined exactas o numeric decimal/hex con gramática correcta, checked arithmetic, punto de código Char; custom con nombre NCName sintácticamente válido → UnsupportedXmlEntity, no InvalidXml absoluto; referencia mal formada o nombre no NCName → InvalidXml. Un '&#xD;' produce CR cooked, distinto de CR literal normalizado.

Normalizador propio medido en dos pasadas, cancelable: calcula tamaño exacto antes de admitir cooked y luego escribe en reserva acreditada. Text/CDATA/comment/PI aplican CRLF/CR→LF; CDATA no interpreta entidades. Attribute aplica EOL primero y sustituye S literal por espacio, sin colapsar secuencias; numeric whitespace conserva carácter referido. Predefined y numeric se resuelven una sola vez; amp seguido de texto que parece referencia no se reexpande. Tratar atributos como CDATA por ausencia de DTD. No llamar normalized_value/xml10_content cuando puedan asignar sin admisión; sirven como comparación experimental, no dueño del ledger.

Declaration: grammar version obligatorio primero, encoding opcional después, standalone opcional al final, S/equals/quotes exactos, sin duplicados/refs/extras; valores standalone yes/no. Version distinta de1.0 sintácticamente correcta → UnsupportedXmlVersion. Encoding ausente o UTF-8 ASCII-case-insensitive admitido; otro encoding válido → UnsupportedXmlEncoding; sintaxis inválida → InvalidXml. UTF8 inválido sigue el fallo vigente de entrada, sin prometer Source0 textual.

La gramática y transformaciones se basan en [XML1.0](https://www.w3.org/TR/xml/#sec-well-formed) y [Namespaces1.0](https://www.w3.org/TR/xml-names/#Conformance). Ready certifica únicamente el perfil de inspección declarado.

## 6. Resolver namespace propio

Por cada Start/Empty: scanner consume TODOS los atributos y verifica duplicados QName; normaliza valores; recoge declarations; verifica bindings reservados; aplica el frame completo; resuelve después elemento y atributos ordinarios, independientemente del orden de declarations. Cada frame tiene marca de undo del scope; End se resuelve/compara antes de restaurar; Empty restaura inmediatamente al terminar su nodo. Ninguna declaración afecta hermanos posteriores.

Bindings se comparan por string normalizada exacta: urn:x&amp;y y urn:x&#38;y dan el mismo URI. No casefold, percent-decode, resolver base, Unicode-normalize ni dereference. Default afecta elemento sin prefijo, nunca atributo sin prefijo. xml está ligado implícitamente a http://www.w3.org/XML/1998/namespace; su redeclaración solo puede dar ese valor normalizado. Nadie más/default puede usar ese URI. xmlns no puede redeclararse ni ser prefijo de elemento; http://www.w3.org/2000/xmlns/ no puede ligarse mediante declarations. xmlns:p='' es error Namespaces1.0; xmlns='' restaura ausencia default.

Unbound prefix falla; expanded duplicates (namespace Option + local exacto) fallan aunque QName difiera. Verificar duplicados con tabla/índice propio presupuestado y igualdad completa tras hash; no O(n²) gratuito. Scope por binding stack y mapa current propio con undo, todos capacidades en ledger; no copiar mapa completo por elemento. Incluir tiempo de hashing de strings grandes en checkpoints. No validar sintaxis URI-reference en este corte: Namespaces§8 exime al procesador de esa comprobación. Conservar exactamente el valor normalizado y documentar esta frontera; deprecación de URI relativo no autoriza rechazarlo automáticamente. No introducir librería URI ni normalización adicional.

[Namespaces§2–6](https://www.w3.org/TR/xml-names/#NSNameComparison) establece comparación después de normalizar atributos, restricciones y alcance. El fallo experimental de NsReader afecta directamente esos resultados; su pending-pop observado no elimina este requisito.

## 7. DOCTYPE inerte y límites de la afirmación

Scanner de envelope reconoce DOCTYPE, root QName lexical, externalID SYSTEM/PUBLIC y sus literals, subset opcional y cierre; respeta quotes, comentarios y PI al localizar brackets/terminador. No tratar cualquier '>' como final. No exigir que nombre DOCTYPE coincida con raíz: eso pertenece a validación DTD excluida. La estructura interna del subset se conserva opaca, sin construir declaraciones, reemplazos ni defaults; no prometer well-formedness interna DTD. Delimitación ambigua/mal cerrada falla XmlRepresentation/InvalidXml según evidencia, nunca un span parcial.

Metadata conserva token raw y aviso «DTD no procesada; sin expansión ni validación». Custom refs usadas fuera del subset causan UnsupportedXmlEntity. El subset puede contener declaraciones sin uso; su presencia no ejecuta ni expande nada. SYSTEM/PUBLIC, namespace URIs, xml:base, PI stylesheet, XInclude/schemaLocation no tienen acceso al resolutor de recursos. Adapter recibe source/cancel/budget, ninguna capability de archivos/red.

La documentación reconoce que este comportamiento no satisface todas las obligaciones de un procesador XML no validador ([XML§5.1](https://www.w3.org/TR/xml/#proc-types)). Pruebas de ausencia de side effects son necesarias; la ausencia de un resolver en el diseño es evidencia de arquitectura, no ejecución.

## 8. Presupuesto, admisión y validador final

Límites existentes: source128MiB, persistent+ownscratch+UIreservation64MiB, nodes+edges1M, profundidad64, JobWindows512MiB; deadline2sTUI/5sinline. Source compartida se mide aparte y no se duplica en arena. Capacidades reales, no solo len, cuentan. XML nodes incluye Document/atributos/namespace/metadatos físicos; cada entry/parent relation contabiliza edge una vez en cuota lógica, tablas de índice cuentan bytes. Referencia namespace por TextRef no añade grafo navegable ni copia.

Ledger de worker y receptor admite cada reserva antes de allocation/copy: nodes/entries/xml metadata/cooked/parents; token scratch, normalize scratch, namespace maps/undo, duplicate indices, stacks, expectedcounts, seen bitsets y workspace de validación. Incluir reservas simultáneas durante crecimiento/rehash y errores; el diseño no depende de corregir exceso después de try_reserve. Guardar modelo aún provisional no permite liberar cargos antes de transferencia efectiva.

Reservar explícitamente1MiB para futura sesión prestada (precedente YAML) dentro de64MiB y transporte acotado4*MAX_FRAME mientras esté vivo. Recalcular sobre tamaños reales/layout, checked arithmetic y overhead fijo. No declarar memoria física total del padre64MiB: librerías/allocator/source/worker pertenecen a presupuestos distintos. Asignaciones internas quick-xml no controladas por ledger quedan bajo Job512MiB; inspección local state.rs muestra copia de nombres abiertos incluso con check_end_names=false. Cursor previo y profundidad limitan entradas peligrosas; memoria/deadline externas permanecen defensa obligatoria. Gate de token enorme pendiente.

finish_xml/receptor verifican: un único Document raíz, alcance de todos nodes/entries sin ciclos/huérfanos/padres múltiples, kinds y campos XML exclusivos, counts exactos, attributes prefix range de cada Element y contenido después, parent ordinal consistente, physical spans/endpoints, nested source containment y orden siblings sin overlaps indebidos, empty/end distinction, depth real, UTF8 cooked/textrefs, nombres/refs/declaration/namespace y duplicates, warning DOCTYPE derivado de nodo. Las selecciones se solapan con su Element padre de forma intencionada; atributos nunca se confunden con hijos de contenido. No confiar en depth/counts ni URI expandida declarada por worker.

Para no convertir parent validation en un segundo parser DTD, la validación semántica comparte rutinas puras de lexical/profile/normalization pero vuelve a comprobar source→metadata y scopes. No duplicar cooked a ciegas: comparar por stream sobre source contra TextRef. Arrays seen/stack admitidos previamente, recorrido iterativo. Cancel/deadline cada bloque acotado de bytes y de items, incluidas comprobaciones finales y búsquedas; el timeout del controlador también cubre llamadas indivisibles del parser.

## 9. IPC estricto propuesto

Mantener CHUNK65536/MAX_FRAME1MiB/BATCH2048 observados. Añadir Counts.xml_metadata y Message::XmlMetadata; reservar representación única y tags nuevos explícitos. Counts XML prohíbe declaraciones/parts TOML y metadata/alias YAML. El transporte es entre binarios privados de misma build; no prometer compatibilidad con versiones anteriores. Documentar revisión schema y fallo inequívoco de peer incompatible.

Estado solicitud: Boot→Init(Xml,bytes)→Source chunks consecutivos/Ack→SourceEnd único→BeginXml→Nodes offsets contiguos→Entries por padre/ordinal→Cooked bytes contiguos→XmlMetadata contigua→End único→EOF limpio. Failure permitido en fase documentada termina resultado; ninguna Source/Ack/Begin tardía ni frames tras End. Frame generation/request iguales a solicitud en todas fases. Contadores, root, profundidad, longitudes y tags se validan antes de utilizar.

Begin hace preadmisión agregada del plan entero mediante checked arithmetic: tablas persistentes + cooked + parents + receiver verification scratch + UIreservation + transport headroom ≤64MiB; nodes+edges≤1M; depth≤64; metadata==nodes para XML. Esto ocurre ANTES de reserve de expectedcounts y de decodificar batches variable-size. Limitar longitud de frame antes de alocar payload; decodificar fields/counts en buffer fijo/reutilizable admitido, asegurar count*wirewidth cabe en bytes restantes y en cuota. No copiar Vec<Entry/Metadata> grande para luego pedir permiso al builder. Cada tabla también comprueba capacidad real al construir.

End no produce Ready hasta validar todo modelo y observar fin limpio del canal; tail/laterframe invalida candidato. Un hijo colgado después de End sigue bajo deadline/reap. Failure code nuevo distinguible: InvalidXml, UnsupportedXmlVersion, UnsupportedXmlEncoding, UnsupportedXmlEntity, XmlRepresentation, junto a fallos existentes de budget/depth/timeout/IPC/containment. Receptor nunca toma reason desconocido como éxito ni reutiliza IDs obsoletos.

## 10. Contrato prestado para B: vista y búsqueda

XmlNodeRef/XmlAttributeRef prestan QName/local/namespace/decoded/rawlocation desde Arc<Structure>. Iteradores por rangos de arena; ningún Vec/String por cada acceso. Address estable NodeId + dominio (QName/local/namespace/value/metadata) + ordinal; identidad no depende del label. o usa location completa acreditada; Source/Structure conservan consultas/posición independientes.

B no convierte árbol XML a data::Value ni crea una línea gigante por elemento con sus descendants. Ventana bounded de texto prestado, SafeText compartido, mapa cooked↔display por checkpoints acotados; búsquedas comparan cooked original y devuelven NodeId/domain/range, no string renderizado. Nunca concatenar vecinos/ref/markup. n/N wrap finito. U009B muestra placeholder seguro sin equipararlo al glifo literal, y ambos continúan diferenciados en matching. Límite de ventana deja Fuente completa accesible; no invalida el modelo.

Reservas Navigator/search/Window comparten el1MiB de sesión ya cargado, historia128 y ventana128KiB como punto de partida concreto compatible con YAML. Solo extraer utilidades compartidas cuando sus reglas XML sean equivalentes; no importar tipos de etiqueta/alias/path YAML. Inline/redir estructura usa escritor iterativo acotado y etiquetado, no serializer XML.

## 11. Gates concretos antes de producción

| Gate | Experimento y oracle | Criterio de salida |
|---|---|---|
| G1 token/spans | 31fixtures/22spans congelados; suplemento BOM+EOF,error primer token, quotes y whitespace atributos, Unicode límites nombres, ref& vs numeric, close S y delimitadores | Todos spans/raw exactos y fronteras válidas; cursor/event/base concuerdan; ningún range reconstruido |
| G2 perfil | Matriz estado vacío/two roots/open EOF/prologref/CDATA/lateDecl, names5e/PI NCName/xmlcase, Char literal/ref, ]]> y '<' atributo, declaration orden/dups/quotes/version/encoding | Ready solo perfil admitido; fallo completo y reason correcto incluso error tardío |
| G3 cooked/ns | Oráculos nuevos de EOL literal/ref, CDATA/noexpand, amp double-looking, declarations después del atributo, default/reset/rebind/Empty/End, xmlURI encoded, URI relativo sin normalizar, expandeddups | Igualdad exacta cooked; resolver nuevo supera4contraejemplos NsReader y reglas restantes sin side effect |
| G4 DTD envelope | Quotes con >/[ ], comments con ]>, PI con brackets/quotes, subset truncated y external IDs; customrefs usadas/no usadas | Token completo acreditado o fallo íntegro; aviso perfil opaco; monitor de archivos/red no observa cargas |
| G5 alloc/cancel | Huge name/token/attribute/namespace list, profundidad64/65; varios buffers cerca64MiB; growth/rehash, source128MiB boundary, 1Mitems boundary | Ledger preadmite toda memoria propia; Job real mata exceso interno, deadline/cancel/reap conservan fuente/terminal; sin Ready parcial |
| G6 IPC hostile | Begin agregado excesivo, u32overflow, unknowntags/format, phase inversions, repeatedSourceEnd, badcounts/parents/textrefs/namespaces, frames trasEnd/EOFtruncated/stalegeneration | Rechazo antes de reservar tamaño atacante; modelo forjado no pasa finish_xml; canal/reap termina en plazo |
| G7 B/AE | CLI local vs literal/Nu/String/ByteStream/flags, fuente predeterminada, búsqueda por dominios/C1/glifo/CRLF; mixed rows/o/Tab/nN/cancel/manualretry; regresiones TOMLJSONYAML | Candidato exacto ejecutado, aceptación/adversario independientes; QA visual/Unix separados y pendientes hasta evidencia |

No se ejecutaron esos gates en esta redacción. G1–G4 preceden integración de quick-xml; G5–G6 preceden ReadyXML del núcleo; G7 precede cierre del corte de producto. Si cursor/event no puede acreditar token o presupuesto, revisar candidato/adapter, no ampliar excepciones para obtener verde. Revisión adversaria concreta de esta arquitectura debe cerrarse antes de A; la implementación y la aceptación tienen sus propias revisiones.

## 12. Evidencia y siguiente entrega

Lectura del informe canónico y código local confirmó la separación actual de YAML metadata, reserva UI, IPC Begin y budget. No se hizo Cargo build/test, modificación del repo, commit, descarga ni apertura de recursos del corpus. La propuesta necesita evaluación adversaria y experimentos enumerados; no hay resultado de arquitectura aprobado ni parser aceptado todavía. Próxima entrega: gate privado medido, con hashes, oráculos y deltas por G1–G6, después diseño R2 si cambian fronteras. X01–X09 y RV-106 conservan todos sus pendientes.
