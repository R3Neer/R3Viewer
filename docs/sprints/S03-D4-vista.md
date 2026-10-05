# S03 XML B: propuesta de integración prestada R2

2026-10-05. Alcance B sobre núcleo A en desarrollo; no otro sprint. Contratos X01–X09 y S03-D4-core sección10 son autoridad. No modifica perfil XML, cuotas ni aceptación visual/Unix. Revisar esta propuesta y API A exacta antes de candidatura B.

## Resultado y reparto

Fuente sigue inicial; extensión local real xml en Auto/Code ofrece Estructura. --format xml interpreta únicamente fuentes textuales; language xml solo sintaxis, ningún SVG/HTML autointerpretado. Añadir selector CLI y dispatch privado main cuando A haya completado el núcleo. Módulo Nu admite valor xml sin alterar encoder inicial. Root/B posee cli/syntax_selection/main/nu/viewer_session/search/structure_output/xml_view y pruebas de integración; A posee arena, lexical/perfil/parser, IPC/controlador/worker y lib/Cargo. Coordinación lib.rs explícita con A.

## Navegación y filas

Navigator XML distinto de YAML: Arc<Structure>, NodeId actual, cursor, dominio activo, último hit y128 estados previos fijos. Document/Element muestran hijos con orden de arena acreditado: atributos y declaraciones namespace primero en orden lexical, contenido mixto después en orden físico. Filas con ordinal/tipo y resumen prestado; nunca dict por QName ni string recursivo de descendants. Element vacío sigue identificable como empty. Enter/r abre nodo seleccionado, K abre nombre del nodo seleccionado cuando exista; m abre metadata del nodo actual. Ningún alias, constructor ni IDREF. Backspace restaura selección, dominio, scroll y hit; history lleno rechaza transacción sin cambiar estado. q/Escape/Ctrl-C mantienen reglas existentes.

Hoja abre contenido cooked en panel seguro; names/metadata tienen etiquetas no buscables por sí mismas. QName/local/namespace son campos separados con identidad de dominio, aunque coincidan sus textos. 'o' siempre selecciona SourceSpan completo del nodo/atributo/token actual o fila seleccionada, no rango cooked ni subspan de nombre reconstruido. Tab/BackTab mantienen búsqueda/posición Fuente vs Estructura, no concatenan ambas. None sólo en metadata realmente sintética y se explica sin salto inventado.

## Texto prestado y presupuesto

XML no se convierte a data::Value/FileValue ni a YAML. El modelo presta strings mediante TextRef y XmlNodeRef. Se permite extraer utilidades mecánicas de ventana/sanitización de yaml_view a módulo neutral si conservan mismas reglas y se verifica regresión YAML; no importar etiquetas/aliases/path YAML. Alternativa inicial: XML own small bounded implementation con mismas reservas, sin generalización prematura. La decisión depende de coste real del diff y pruebas.

El1MiB de reserva de sesión cargado por A cubre conjuntamente Navigator, search scratch y Window, con lease antes de reservas; capacities simultáneas cuentan. Window hasta128KiB, checkpoints4096, grapheme scratch16KiB. No duplicar toda hoja grande; controles visibles y tabs/CRLF coherentes con visor. Cluster ordinario puede prestarse sin imponer límite128bytes de label. Ventana agotada deja Fuente completa disponible con explicación, no invalida arena. IDs/estado de búsqueda descartan generaciones antiguas.

## Búsqueda con dominios

Recorrido iterativo físico finito, atributos/content una vez, no graphlink. Por nodo, orden estable QName, LocalName, NamespaceUri, ValueDecoded, Metadata: cada uno unidad separada, sin match a través de campos o nodos. Metadata busca sólo contenido propio de Comment/PI/Decl/DocType y datos de inspección aprobados, no labels genéricos ni números de ordinal/tipo insertados por UI. PI target QName se mantiene separado de data. Declaration y DOCTYPE conservan token raw como una unidad Metadata etiquetada por nodo; no volver a parsearlos en B ni expandir DTD. Comment/PI data cooked son unidades Metadata, no duplicarlas también como ValueDecoded. LocalName/QName idénticos pueden producir dos hits con dominios distinguibles: el panel lo indica, no los colapsa silenciosamente.

Hit = generación + NodeId + dominio + unidad ordinal + rango UTF8 original dentro de esa unidad + rango visible derivado. Match contra caracteres originales prestados; proyección SafeText sólo para hit elegido. Backward no proyecta cada candidato; presupuesto CPU lineal por recorrido, cancel/checkpoints cada64KiB. Reanudación n/N usa identidad y original_range, nunca rango visible escapado. C1U009B difiere de string literal '\u{009b}', ambos con o fuente exacta. No unir Text/Reference/CDATA para buscar una frase. Fuente mantiene búsqueda raw independiente. Query<=4096B, sharedSearchWorker coalescing existente.

## Inline y redirección

--structure produce inspección etiquetada XML, no serializer ni XML rehecho. Orden físico y tipos visibles, cooked quoted seguro y nombres/URI independientes; no Number/Boolean inferred. Preparar salida completa acotada antes de primer write con arena+output+growth peak dentro64MiB; si no cabe, fuente completa y diagnóstico0. Nunca escribir prefijo estructural exitoso. El snapshot interno conserva bytes UTF8 originales íntegros; Source default en pantalla/stdout aplica SafeText vigente, incluyendo C1 visible, sin emitir controles activos. Para entradas sin controles transformados puede coincidir byte a byte; no es promesa de salida raw universal. Failure whole-source0, entradaUTF8/IO1, uso2, Ctrl-C130 existentes; BrokenPipe contrato actual. DOCTYPE aviso explícito y namespaces nunca recursos descargados.

## Criterios de candidata

API A estable y gateG5/G6 antes de ReadyXML. Pruebas de selector archivo real vs metadata/literal/stringstream/binary; lista mixta ejemplo X03 orden b,a,text,i,text,ref,cdata,comment,PI; nombres/default/rebinding y atributos; o spans raw BOM/CRLF/Unicode/entity quotes; cooked search C1 vs glifo literal, dominios y n/N/resize/history128; output control-safe y fallback de fuente completa comparado contra SafeText de todo el snapshot en fallos tardíos, incluido C1 literal seguido de error de perfil. Regresión TOML/JSON/YAML y budgetleases/drop/transactional cancellation. Candidato exacto revisado adversario distinto del autor A/B y AE independiente CLI/Nu/ConPTY. No acreditar pixels, QA visual, Unix runtime o todo RV106 por estas pruebas.

## Matriz cerrada de unidades y acciones (XB-01)

Unidades ausentes/None no producen texto ni hit. Orden global: DFS de nodes físico incluidas attributesfirst; dentro de cada nodo el orden de la tabla. Unit ordinal0 por dominio, sin unidades adicionales derivadas de helper labels. QName y local iguales son dos unidades con dominios distintos. Namespace URIs iguales en distintos nodos/dominios siguen hits distintos, nunca deduplicar.

| Kind | Unidades buscables, en orden | Origen prestado exacto | Acción K / panel m |
|---|---|---|---|
| Document | Ninguna | root/hasdoctype/NodeId sólo estado | K no tiene nombre, no cambia navegación; m muestra root/doctype/tipo como información no buscable |
| Element | QName, LocalName, NamespaceUri si Some | name.qname, name.local, name.namespace | K abre QName; m muestra los nombres reales/URI y empty/count/open/close; flags/cifras/spans/labels no buscables |
| Attribute | QName, LocalName, NamespaceUri si Some, ValueDecoded | name.qname/local/namespace y node.decoded | K QName; Enter/r ValueDecoded; m nombres y posiciones, nunca inferir bool/number |
| NamespaceDeclaration | QName, LocalName sólo si prefijo explícito, ValueDecoded | QName físico completo xmlns o xmlns:p acreditado por A; binding_prefix p si Some; node.decoded URI del binding | K QName; Enter/r ValueDecoded; m binding prefix/value/posición. No NamespaceUri para declaration: no fingir nombre expandido ordinario ni URI xmlns. QName original debe prestarse desde A, no escanear header en B |
| Text / Reference / CData | ValueDecoded | node.decoded exacto de perfil | K sin nombre; Enter/r cooked; m tipo/rawspan/reference_kind sólo información no buscable, sin raw ref duplicada como metadata |
| Comment | Metadata | node.decoded cooked del comentario | K sin nombre; Enter/r/m abre contenido Metadata. Delimitadores raw accesibles por o, no nueva unidad buscable |
| Pi | QName, Metadata | metadata.target SourceSpan y node.decoded data cooked | K target; Enter/r/m Metadata data; target no tiene LocalName/NamespaceUri inventados |
| Declaration / DocType | Metadata | SourceSpan raw completo del token admitido | K sin nombre; Enter/r/m Metadata raw segura; DTD aviso inerte no buscable, sin parser B ni versión/encoding inferidos como unidades extras |

Esta tabla sustituye «datos de inspección aprobados» de R1: no hay otros campos buscables. El panel m de containers/Attribute/NamespaceDeclaration muestra todas sus propiedades en orden de tabla seguido de helpers informativos; de allí no se construye un string buscable genérico. Los campos prestados de nombre/URI/value se presentan seguros conservando límites de unidad. Un hit abre panel de su dominio y lo etiqueta (QName/LocalName/NamespaceUri/ValueDecoded/Metadata); m sin hit abre inspector; K abre sólo QName cuando existe. No hace falta otra tecla para un nombre local/URI: son visibles en inspector y seleccionables por búsqueda global. No convertir etiquetas ni helpers en contenido. K en nodo sin nombre conserva estado y explica discretamente; no empuja history vacía. Enter/r en container abre fila seleccionada, aunque sea leaf Metadata. o de un campo/inspector/hit siempre rawspan del nodo propietario, no subspan de unidad.

Núcleo A garantiza QName físico de NamespaceDeclaration en metadata.name.qname (sus restantes componentes no conceden nombre expandido ordinario) o accessor equivalente acreditado antes de candidata B. Si falta la propiedad, completar A y revisión pertinente; no reconstruir QName en B ni omitirlo para aprobar. LocalName de namespace declaration significa prefijo del binding explícito y el inspector lo etiqueta «prefijo declarado»; default carece de esa unidad. Ninguna visita URI ni autoenlace sigue esos valores.

## Conservación segura y respuesta R1 (XB-02)

Snapshot/raw source interno íntegro, offsets UTF8 y o se verifican contra bytes originales. Presentación de Fuente completa siempre aplica SafeText: C1 legal XML aparece visible como texto, controles prohibidos del XML fallido también visibles. El fallback no hace trim, reserializa o limita a prefijo; sus bytes de stdout se comparan contra el resultado seguro de toda la fuente, no input inseguro. Matching cooked conserva el carácter original y nunca compara placeholder. Fuente busca raw internamente y mapea a presentación vigente, sin bypass de seguridad por XML/redirección. Nuevo caso obligatorio: C1 literal legal antes de un error XML tardío; diagnóstico+Fuente completa segura, sin escapes activos. No modifica perfil ni loader ni política de salida existente.

R1 crítica SHA8DE1D8D00C19C5336E91415664371379FC76AC44E52DE8C5352F56262BD55FBA; R2 responde XB-01 con matriz/actions y XB-02 con dos fronteras raw/visible. Pendiente revisión independiente de esta propuesta exacta, no aprobación por autor.

## Conservación canónica y cierre de propuesta

R2 exacta revisada independientemente SIN_CRITICAS_PENDIENTES. XB-01 yXB-02 cerrados; las expresiones anteriores de propuesta pendiente son estado de redacción. [Revisión final](../reviews/S03-D4-vista.md). Original SHA766CB72323EAA3188708988713354C3462CD9418DA08942B3D2BFD0927F4BCC3, revisión SHA17656FC956625AFA5602465A485BF112AD3F89BF486EC27D4B49BDB6658D2279. Implementación B iniciada coordinada con A; no aceptación código/AE/runtime/visual/Unix.
