# b2: presentación completa de bloques y recursos seguros — R2

2026-10-09. Código base353418c19f2152614d5650d605aa05df72a1cbca limpio; documentos posteriores b3c294e sin delta de código.
Entrega privada .tmp/renderer-b2-2026-10-09. Sin implementación hasta SIN_CRITICAS_PENDIENTES de diseño.
Refina D2b-R3§3/4/6 sobre renderer textual aceptado. Objetivo único: biblioteca b2 funcional con todas las familias de bloque, recursosPending y WholeReference realmente dibujables; después b3+pump.
No cambia parser/model/wire, SourceLegacy, CLI, resolver, gráficos o Syntect. Ninguna prueba de captura acredita ConPTY/QA visual.

## 1. Contrato observable

Usar helpers document_roles::block_role/resource_descriptor/text_ref_view y PresentationConfig::style; marcadores ASCII discretos del inline aceptado.
Una fila semántica sigue Items reales: texto anterior, hijo y texto posterior conservan ese orden. Los Leave estructurales no crean filas vacías duplicadas; los bloques terminales finalizan su línea.
No agrupar todos los Runs de un padre atravesando sus hijos. SoftBreak sigue dentro secuencia; resource/domain/bloque corta cluster. Wrap siempre por unidad Unicode acreditada.
ColourNever conserva los mismos caracteres y posición sin estilos. ORstyles de cluster no se divide; Heading usa Role::Heading+bold, código/link Accent, texto Value, marcadores/prefijos Secondary.

|Familia|Salida b2 y criterio|
|---|---|
|Root/Paragraph|Texto seguro, wrap local; sin banners ni párrafos sintetizados.|
|Heading1–6|Decoración '#' repetida level más espacio; texto Heading+bold. Continuaciones alineadas al comienzo del texto, con espacios decorativos.|
|Quote|'> ' en cada fila visible/wrapped de ese contexto; quotes anidados repiten el prefijo.|
|Callout|Primera fila '[callout KIND +] TITLE' si cerrado, '-]' si abierto. Kind vía TextRefIndex, Title por su dominio real. Body abierto lleva '| ' por fila; cerrado no materializa body. El signo deriva FoldOverride vigente, no sólo initially_closed.|
|List/Item|Unordered '- '; ordered start+Item.ordinal checked en u64 seguido '. '. Task '[ ] ' o '[x] '. Indentación2 espacios por nivel de lista; continuación cuelga tras su marker. Loose añade una fila vacía entre Items; tight no.|
|Table|Modo elegido ANTES de medir contenido: vertical si ancho útil<8*columns+3*(columns-1); columnas de ancho fijo local en otro caso. Detalle §2.|
|Code fence/indented|Conservar todos los caracteres y líneas, tabs8 en columna local; hscroll excluye clusters enteros. Info/language disponibles como Role/TextRef, sin resaltado D4 ni ejecución. Encabezado discreto '[code: INFO]' sólo si fenced e info no vacía.|
|Mermaid fence|DescriptorPending en su posición y código íntegro navegable; ningún éxito gráfico.|
|HtmlInert/Frontmatter|Literal proyectado seguro, sin ejecutar HTML/YAML. HTML wrap; frontmatter/código hscroll. No convertir '&' decodificado en SourceExact.|
|Footnote|Etiqueta segura '[^LABEL]: ' como Role/TextRef, seguida cuerpo en orden. Definición visible aunque no tenga referencia previa.|
|Rule|'---' como decoración fija de tres celdas; fila sóloRole, no SourceExact.|
|Image/MathInline/MathBlock/WikiEmbed|Descriptor de estado pendiente y etiquetas seguras, §3; nunca carga archivos/red ni abre destinos.|

Prefix/indent no absorbe un combining inicial width0: se asocia sólo a contenido semántico anterior compatible, nunca a decoración. Sin base semántica => WholeReference WidthZero.
Si prefijos dejan cero ancho útil para una unidad obligatoria, WholeReference Width; no borrar prefijos silenciosamente ni copiar media unidad. Geometry0 produce Reference tipada sin IO.

## 2. Tablas y orden local

Vertical: cada Row conserva orden de Cells y añade 'column N: ' por celda; header real usa Heading y filaBody Value. N=Item.ordinal+1 real, no índice del viewport.
Rows separados por una fila vacía. No seleccionar sólo las primeras columnas ni perder row2; una ventana corta termina con continuación tipada hacia próxima celda/unidad.
Columnas: n y ancho disponible conocidos por BlockRole; distribuir ancho útil-3*(n-1) en n anchos casi iguales, residuo a primeras columnas. No scan global ni maxwidth.
Cada celda usa provider/domain propio con wrap local; altura de una Row es máximo de sus celdas LOCALES preparadas. Alignment Left/None/Center/Right posiciona unidades completas dentro del ancho fijo.
Header usa sus celdas reales una vez; separador decorativo de '-' con cruces '+' y padding se cobra. Se puede repetir header visible como Role derivado y nunca nuevo texto buscable.
Un Row muy alto se continúa por subfila/celda acreditada; no convertir altura en índice global. End usa terminal D2a/celda final, sin medir filas previas.
Fallo Work/capacidad al medir columnas => WholeReference, conservando gasto; no segunda pasada vertical con allowance nuevo. Vertical preseleccionado no mide columnas.

## 3. Recursos, etiquetas y WholeReference

Reutilizar tokens inline: '[Image pending: INPUT; alt: ALT; title: TITLE]', '[Math pending: INPUT]', '[Wiki embed pending: INPUT]' y '[Mermaid pending: INPUT]'.
Mantener campos ausentes distintos de vacíos; ALT sólo si alt_domain, TITLE sólo si title_ref. Kind/state/origin/intent/dimension_intent son metadata disponible, no solicitud de carga.
Cada INPUT/TITLE/KIND/LABEL pasa por TextRefIndex::step_resolved y RefCoverage, cada ALT/TITLE-callout por SegmentedIndex real; ningún raw str llega directamente a set_symbol.
Provider safe controls/entity/CRLF/tab/oversized conserva reglas D1/RU. No crear Domain/Run nuevos para TextRef ni llamar su offset SourceExact sin acreditación.
Alt/title/input son lanes separados, tipados y paginables. No se duplican en MainBody ni se mezclan en un cluster con tokens decorativos. Bloque e inline conservan intent metadata; b2 usa descriptor de referencia seguro, sin fingir geometría gráfica.
Mermaid conserva además dominioCode navegable en orden, Math conserva raw/input y Resource metadata. ResourcePending no equivale al fallo de cuota WholeReference.
WholeReference se dibuja como '[Document reference: REASON]' en ASCII seguro, con razón/anchor tipados. No muestra un prefijo de contenido semántico como si fuera la ventana completa.
Label puede ocupar sólo el viewport disponible; el resultado sigue Reference explícita. Source original íntegra y contexto retenidos; recovery Ordinary permite siguiente operación sin reparsing.
Cancel/stale no produce Reference tardía: descarta candidato y preserva frame/anchor anterior. Sin mínimos old+new conserva oldframe y devuelve rechazo; initial sin mínimos queda Fuente disponible para caller posterior.

## 4. Cambios concretos y anchors para b3

src/document_layout.rs: contexto roles profundidad<=65, generación decor y lanesResource/TextRef, row/next anchors explícitos y layout de tablas dentro del mismo WindowBuilder. No renderer con traversal.
src/document_flow.rs: accessor prestado/cobrado de stack cursor+parent Item.ordinal; contexto de seek conocido sin replay Root. Helper de fold efectivo cobra lookup, no copiar todos overrides.
src/document_view.rs: préstamo y almacenamiento cerrado PresentationAnchor para Continue/Line/Resize; same Authority y key Binding/Flow/fold/geometry actuales. validate lógico antes quota/Geometry.
PresentationAnchor discriminado Domain(LayoutAnchor), Role(block,flowOrdinal,phase), Ref(RefAnchor), Resource(block,ownerRun,resource,lane,posición acreditada), Table(TableResume). No campos públicos que caller pueda fabricar.
TableResume conserva RowID y CellID seleccionado con Domain/LayoutAnchor ACREDITADO de texto, más progreso local At(Domain/LayoutAnchor) o Done por cada celda que participa en la fila visible. Subfila geométrica sólo es metadata de esa ventana; jamás se reinterpreta como offset tras Resize.
El conjunto de progreso es ChargedVec limitado a las celdas de la fila LOCAL (vertical retiene sólo celda activa); guarda IDs+anchors, no Points/SegmentedIndex por celda ni tabla global. Cada reconstrucción/reposicionamiento usa el mismo presupuesto y el único slot candidato; conserva current protegido.
Resize restablece cada celda pendiente desde su posición semántica acreditada y recomputa wrap/alignment con ancho nuevo. Anchors de celda agotada quedan Done y no inventan offsets ni se resucitan por subfila. Cambio vertical/columnas conserva celda+texto seleccionado; ninguna celda/TAIL posterior desaparece.
La continuación a otra ventana guarda el primer contenido no mostrado por celda; una fila se termina sólo cuando TODAS sus celdas están Done. Selección/row_anchor conserva el primer contenido real de la fila visible, no el contador de máximo alto. Validar Row/Cell/domain y FlowIdentity antes Geometry. Si progresos mínimos no caben, WholeReference explícita sin prefix como completo.
row_anchor usa ancla explícita de fila: primer contenido semántico cuando existe; si sólo decor, Role; si descriptor, Resource. Nunca usa primer glyph decorativo como Source ni retorna None perdiendo Continue.
RefAnchor contiene TextRefID y posición local/projectionpart ACREDITADA por TextRefIndex, con propietario RefOwner::Resource(resource,field) o RefOwner::BlockRole(block,role,field). KIND callout, INFO/LANGUAGE code y LABEL footnote no se disfrazan de Resource ni Domain/Run.
El constructor comprueba que el TextRef es precisamente el aux/campo acreditado de ese block_role/resource_descriptor; no admite RefID ajeno, byte inválido o affinity sin provider. Lanes Input/Title/Kind/Info/Label distintas incluso si comparten texto o RefID.
La variante slot Domain/TextRef conserva dos slots máximo; adaptador mínimo TextRefIndex de locate/seek/visual_anchor cobra los mismos checks/provider real y entrega token opaco, sin fabricar Point ni Source offset. FirstCodeComponent permanece None en RefText.
PackedGlyph.local conserva posición del provider en RefText; accessor admitido cell_anchor reconstruye RefAnchor con sidecar propietario cerrado, nunca obtiene localSource por start/end de símbolos visibles. Toda lectura/metadata/relación se cobra.
Row/next anchors Ref permiten continuar y redimensionar en mitad de KIND/INFO/LABEL/ResourceTitle largos, hasta su sufijo y siguiente bloque TAIL. Una Row de marcadores sóloRole conserva RoleAnchor. Resize conserva lane y posición; Fold invalida FlowIdentity y remapea propietario real sin ordinalviejo.
Contexto/anchors no son historial nuevo; b3 consumirá préstamos tipados y aplicará sus efectos history/search propios. decor SourceAction=NoLiteral(RoleAnchor); TextRef mantiene Exact/Enclosing/ControlProjection.
src/document_renderer.rs: RenderPlan cerrado acepta Text y WholeReference con iterador admitted cells; prepare no inicia providers ni consulta Document. Style sólo metadata precomputada.
src/document_terminal.rs conserva rollback/current-only, previous opaque/Charge, widepreclear y poison; no nuevos hooks upstream ni Buffer scratch.

## 5. Storage adaptativo y cuotas vigentes

No aumentar Window128KiB/Frame256KiB/Consumer1MiB/checkpoints4096/Framework2MiB/DocumentWork8MiB ni tocar text_engine.rs.
Conservar PackedGlyph20 y GlyphClass40 por tags compactos: CellKind Text(PaintRole), Decoration(metaID), RefText(metaID); ID apunta a sidecar cobrado por ventana. FirstComponent sólo contenido real; decor/ref sin FirstCode ficticio.
Metadata de paint/origen lateral<=32B por descriptor, no por carácter; leyenda/bloque/resource compartida por sus celdas. Todos sus reads/reconstrucciones/copias se cobran.
Reservar classes inicialmente min(16,cells), crecimiento fallible al siguiente número necesario dentro Window/Consumer, cobrando oldallocation+newcandidate antes copiar; no scan de todas clases ni hash table sin cargo.
Symbols crecen igualmente sólo si la suma real cabe; Cell/Row capacities conservan bounds por geometría. Releer capacidades y publicar bytes reales sólo al terminar. Overflow conserva candidata íntegra descartada.
El cambio de reserva no relaja límites; evita Reference trivial por height*4+16 classes. Dense80/120 y tabla/lista multirun moderadas son gates prácticos obligatorios.
Frame cuenta old+new windows, sidecars, anchors, payload/copies y descriptors. Grids/heap quedan en Framework; Charge de símbolos también retiene Consumer hasta reset/Drop.

## 6. Reserva renderer y fallback con MISMA operación

Hallazgo de base: LayoutWindow::Reference conserva gasto real, pero renderer actual la rechaza. Workfailure puede dejar saldo insuficiente para scans/diff de backend;8192 no cubre grids4800.
Introducir sólo recibo cerrado FrameWorkReceipt/RenderReserve en la frontera layout-renderer: cap8MiB, spentLayout, heldRenderCapacity y Binding/geometry. No allowance porpane ni caller-declared previous.
Antes de layout, reservar matemáticamente rendererEnvelope; ejecutar DocumentWork bounded sobre cap8MiB-envelope y conservar todo spent real, incluido Reference8192 inicial. ErrorWork no reinicia contador.
Envelope para nCells: n*(4Cell+2Cell+2Cell+512) +2*n*(glyph_read_bytes+16+metadata_read_bound32) +4*WindowBytes+256. Con Cell48, glyph_read_bytes132: n*1256+524544.
Predicción80x24=2936064 held,120x40=6553344 held; quedan5452544/1835264 para layout incluyendo8192 fallback. Dense base layout404164/1327023 entra. Importes predichos, no medición b2.
La reserva es capacidad retenida, NO trabajo ejecutado. window.work guarda spentLayout, nunca envelope+spent. Transferir RenderReserve sin Clone al renderer; spend coste renderer real<=held, sumar spentLayout+renderSpent<=8MiB. No doblecargo/reserva ni refund del gasto fallido.
Fallback gasta el MISMO reserve retenido y los bytes/metadata ya admitidos; no nueva operación ni nuevo permiso. Si guard real excede envelope, rechazo previoIO y hallazgo, sin subircuota silenciosa.
Redraw estable es nueva operación explícita: replay de spentLayout y nueva reserva dentro MISMO techo8MiB; no nuevo layout Unicode. Geometry preparation sigue fase separada R2§3 y no se anuncia como SAMEWork de draw.
Oráculos anteriores dense/coste7.063M y old4M preservados como historia; nuevo envelope cambia admisión pero no se atribuye coste real b2 antes medir. Negativos exact/-1/overflow/cancel sobre importe efectivo fijados antes nuevo output.

## 7. Aceptación y entrega

EXPECTED-b2-before-output-r1.json y fixtures privados fijan contenido/roles/anchors/modos antes de código. Adversario define sus propios testigos.
Gates: cada familia §1 real parser+validate+factory; itemText/sublist/text orden; folded body y title separados; tabla3cols2rows vertical/columns sin perder terceros; resource fields de todoskinds y controles seguros.
Anchors: Line/Continue no repite quote/list/descriptor, Rows sóloRule navegables, RefText multilínea/Title largo recupera TAIL, Resize/Fold sin fakeSource y cancel/stale conserva anterior.
Quotas: classes>176 caso práctico todavía Text, crecimiento old+new/drop, symbol16KiB/+1, Window/Frame/Consumer exactfrontera relevante, Work held/spent con fallback sin renovación y Frame previous opaque.
Renderer Crossterm real: twoFrames+resize, wide VS16/CJK, prefix no combiningbase, styleNever/Heading/ORcluster, cancel final y backendflush poison heredados. Captura fija separada de memoria productiva.
Final candidato contra base exacta, fmt/Clippy/tests focales+regresiones b1/RU/inline pertinentes, freeze hash para review código+AE independiente. Sin repetición gigante32MiB por cada ronda.
B2 completo como biblioteca textual/resource segura no cierra RV-301 completo: b3, pump, recursosgráficos,D4/Unix/visual siguen obligatorios posteriores, no alternativas que recorten el producto.
## 8. Delta R2 tras revisión independiente

R1 y oráculos previos se preservan SHA1418478E.../F1CCDF16...; informe adversario A8B20E8B... sólo B2-A01/A02.
A01: Table(row,cell,subfila) sustituido por TableResume con posiciones de dominio acreditadas por celda y progreso local limitado/cobrado; Resize no transforma subfila1@width8 en byte20@width20.
A02: RefAnchor acepta propietarios BlockRole y Resource con field y posición provider; KIND/INFO/LABEL largos tienen continuidad sin Resource/Domain ficticio.
EXPECTED-b2-before-output-r2.json añade únicamente oráculos A01/A02 antes de implementación/outputs. Contratos y cuotas R1 restantes conservados. Base código353418c; documentos canónicos posteriores7a7b9a7 mantienen mismo código.
