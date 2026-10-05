# S03 XML B — revisión adversaria de propuesta R2

2026-10-05. Candidato exacto S03-XML-B-plan-r2.md SHA256766CB72323EAA3188708988713354C3462CD9418DA08942B3D2BFD0927F4BCC3 comprobado. Relectura completa, contraste con X01–X09, core sección10, metadata A comunicada y referencias prestadas YAML/search/session/output. Alcance exclusivo de contrato y propuesta de integración B; no aprobación de implementación, núcleo A, gates G5/G6, parser/IPC, runtime, AE, QA visual ni Unix.

Veredicto: SIN_CRITICAS_PENDIENTES dentro de la propuesta B R2 exacta. No se modificó producción ni se ejecutó compilación/tests/GUI.

## Hallazgos anteriores cerrados

XB-01 cerrado: matriz finita por kind define unidades QName/LocalName/NamespaceUri/ValueDecoded/Metadata, origen prestado, orden y campos ausentes. Helpers/flags/counts/spans/NodeIds no se serializan a texto buscable. Document ninguna unidad; Element/Attribute nombres reales, NamespaceDeclaration QName acreditado en A y prefijo explícito como LocalName etiquetado, sin NamespaceUri ordinaria inventada. Text/Reference/CData value únicamente; Comment cooked metadata, Pi target separado de data, Declaration/DocType token raw completo. Repeticiones entre dominios/nodos mantienen identidad; default binding no crea prefijo vacío buscable. K QName/target o no-op explicado sin history; m inspector no se vuelve una unidad genérica; hit abre su dominio real y o salta al nodo propietario completo. Enter/r container abre fila; leaf Metadata tiene comportamiento definido. Nada reconstruye nombres buscando header en B.

XB-02 cerrado: snapshot/raw interno íntegro y spans son exactos; Fuente en pantalla/stdout aplica SafeText a todo el snapshot, incluidas entradas XML fallidas y C1 literal. El fallback0 no hace trim, reserializa ni escribe prefijo. Oracle de salida se compara contra render seguro completo, no contra bytes inseguros del input. Matching cooked mantiene carácter original; Fuente raw y presentación se distinguen y no legitiman buscar placeholders como cooked. Caso C1 legal seguido de error XML tardío incorporado a criterios futuros.

## Revisión completa de coherencia

Fuente inicial y selector XML sólo sobre fuente textual u origen local real; literal/metadatos/binary Nu no se reinterpretan por adivinación. Language XML no selecciona parser, SVG/HTML no se infieren. Nu mantiene encoder inicial; opciones/regresiones existentes siguen en alcance.

Filas ordenadas, attributesfirst físico y contenido mixto separado; sin dict/Value ni concatenación descendants. QName/local/URI/value/metadata permanecen unidades distintas; sin matches cruzando nodos/markup/ref ni alias/IDREF. IDs y generaciones protegen búsquedas viejas; n/N reanuda original_range y evita SafeText de cada candidato backward. History128 y navegación transaccional preservan estado ante rechazo/cancel; vuelta/tab y scroll mantienen dominios independientes.

Sesión prestada1MiB cubre Navigator/search/window con admisión previa y capacities simultáneas; window128KiB/checkpoints4096/grapheme scratch16KiB. Fuente íntegra disponible ante ventana agotada, sin invalidar arena. Output inline completo bajo64MiB con growthpeak o fallback antes de primer write; no se anuncia estructura parcial exitosa. Cancel/IO/uso/BrokenPipe mantienen contratos vigentes. La realización técnica y latencias concretas deberán revisarse en candidato, especialmente búsquedas de unidades namespace compartidas repetidas y ventanas/cluster extremos; el diseño no acredita esos resultados por declaración.

La propuesta exige API A estable, G5/G6 y revisión antes de ReadyXML/candidatura; una propiedad faltante de QName obliga resolver/revisar A, no reconstruir ni omitir en B. A y B tienen ownership coordinado y revisión/AE de candidato exacto posterior. Se conserva DOCTYPE inerte y namespaces sin resolutor/autoenlace; avisos discretos, no etiquetas buscables.

S03 sigue activo; RV-106, aceptación visual y ejecución Unix mantienen pendientes. El cierre de esta propuesta no reutiliza aprobación de gráficos ni núcleo A y no autoriza declarar producto aceptado. Cambios posteriores de API/contrato requieren revisión pertinente nuevamente.
