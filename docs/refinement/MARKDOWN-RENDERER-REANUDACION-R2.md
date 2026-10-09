# Reanudación: owner y primer renderer Markdown real — propuesta R2

Fecha 2026-10-09. Base exacta 0cb0fe65363dae463d739502d5d3058cc740909c, worktree limpio confirmado por coordinador con Git elevado.
Encargo actual: propuesta y luego candidato privado revisado. Sin escribir producción/Cargo, commits, perfiles, publicación ni corpus Clases.
Destino exclusivo: D:/Tools/R3Viewer/.tmp/resume-renderer-2026-10-09.
Esta propuesta concreta D2b2; pump D2c se conecta después sobre el mismo renderer. No sustituye los planes canónicos.

## 1. Evidencia y delta mínimo

Los cuatro ZIP coinciden con INDEX.json. Verificados SHA de MANIFEST y cada registro size/SHA: framework candidato85, aceptación18, Work11, packed18.
Framework owner aceptado sólo para construcción Fixed0, capacidades, reservas y geometría: new/prepare_area/receipt; no entrada draw.
Owner base histórica2f070ac difiere de HEAD: importar únicamente document_terminal.rs/tests y hooks vendor, jamás sustituir Cargo/lib/contexto íntegros.
Registry ratatui-core0.1.2 actual:67 archivos idénticos; modificados src/buffer/cell.rs y src/terminal.rs; añadido src/terminal/r3_storage.rs.
Antes adopción: registro DERIVATION con versión/checksum Cargo.lock, upstream VCS e665c36cb14752a61cd777fbd06dbef8474f2add, lista2+1, hashes, licencias intactas.
Normalizar CRCRLF sólo en copia derivada; ZIP y evidencia original byteexactos. Nuevas extensiones añadirán sus propias rutas/hashes al registro.

HEAD document_layout TextWindow conserva ChargedVec<Glyph> (Glyph72B); DocumentWork.frame/bounded están limitados256KiB.
Ensayo packed verificado muestra Text real con memoria ORIGINAL Window128KiB/Frame256KiB/Consumer1MiB y DocumentWork2MiB:
80×24:1440glyphs/24rows,59744B ventana,404164Work;120×40:4800glyphs/40rows,122848B ventana,1327023Work.
Fixture4920B:40 líneas de120X más hardbreak;80×24 muestra12 líneas fuente repartidas80+40, no4800X.
Estas mediciones no incluyen renderer, buffers, adornos o cargos diff. Work256KiB Reference y otros fallos históricos permanecen intactos.

## 2. Primer corte que se implementará tras revisión

Copia privada de HEAD a product/, target privado. Reusar los módulos b0/b1/ResolvedUnit/Roles ya integrados.
Delta explícito: src/document_terminal.rs y hooks pinned; compactar storage en src/document_layout.rs con PackedGlyph20B/GlyphClass40B;
perfil DOCUMENT_FRAME_WORK=4MiB en src/document_text.rs, manteniendo Source text_engine y Work/preparation/quotas Legacy.
Importar sólo transformación storage del ensayo, revisar contra HEAD: reconstrucción Glyph por valor con accessor glyph(index)/iterator, sin Vec<Glyph> temporal.
Conservar exactos first/projection_part/links/relation_mask/estilos; clases/capacidades old+new cargadas por ConsumerBudget y Window.
No copiar document_flow/document_view del ensayo ni normalizar fixtures a conveniencia.

Nuevo src/document_renderer.rs consume DocumentView.layout()/geometry()/budget() y celdas acreditadas, sin parser, worker, índice, IO recursos o segmentación.
Nuevo test/harness privado hace factory start_document real -> authority.poll/take_validated(join físico) -> DocumentView::from_prepared -> renderer -> CrosstermBackend real.
Primero ejecutable sintético de una pantalla, dos frames y resize con writer Vec de harness; después ConPTY, antes de conectar selector público.
Es renderer real aunque su invocación inicial sea privada: utiliza las mismas APIs owner/diff/backend que consumirá D2c; TestBackend es otro control, no prueba de writer.

## 3. Contrato concreto de render y ownership

BudgetedTerminal<B> conserva terminal privado; añadir backend_area() por préstamo y draw_admitted(plan,permit,cancel,current).
No exponer Frame, Buffer, Terminal ni backend mutables a caller; renderer sólo entrega descriptores cerrados.
RenderInput identifica binding DocumentKey/geometry y préstamo inmutable de LayoutWindow; owner no conserva ese préstamo después del retorno.
prepare_render produce RenderPlan+MandatoryRenderPermit no Clone. Coteja posiciones/rangos UTF8/width/continuations y presupuestos ANTES de mutar grid.
El permiso es spend-once, sin refund; contiene binding, geometría, identity ventana, Work de layout ya consumido y cargos de render/diff/readback/reset.
Cada transacción frame tiene UN techo DocumentWork4MiB: no reiniciar allowance al entrar renderer ni por celda/pane.
Redraw estable requiere nuevo permiso de replay: carga de nuevo el costo lógico original del layout sin reejecutar Unicode, más costos reales de output.
No introducir Work2MiB para Source ni cambiar Workpreparation; 4MiB es candidato para la composición, nunca cierre por mediciones de layout2MiB. Si no admite costo total real120×40, bloquear y refinar perfil medido antes adopción.

Cells ordinarias: Cell::set_symbol sólo para cluster ya admitido; CellDiffOption::ForcedWidth(NonZeroU16(width)).
Continuaciones wide: CellDiffOption::Skip, vacías sin copiar cluster; transición wide->narrow y borrado probado por diff pinned.
width0 nunca se convierte a NonZero ni se envía como cluster; un recurso sin representación usa marcador pendiente completo en layout de b2, o WholeReference explícita.
No Buffer.set_string/Paragraph/Span/wrap/UnicodeWidthStr, append_symbol, Buffer.clone o Buffer.diff Vec en esta ruta.
Spanstyles: bits1italic/2bold/4crossout; Glyph.links>0 añade16underline; bit8code o link usa Role::Accent, otro texto Role::Value, vía PresentationConfig.style(Role). Colournever aplica Style::default sin modifiers. OR de coverage sobre un cluster se conserva (a**U+0301** => cluster entero bold); ninguna SGR interna. Encabezados/rolesdeblock, listas, tablas, adornos y descriptoresRefID no están implementados por b1: permanecen b2posterior. No gramáticas fences/overlay ni diseño visual nuevo.
Texto obligatorio se admite primero; roles/color opcionales sólo tras él. Fallo opcional conserva todo texto Plain.

Owner conserva ambos grids físicos. Preflight cuenta heap actual por buffer y capacidad nueva de cada CompactString (inline<=24B, heap real según pinned).
Cada set_symbol crea copia: cargo simultáneo Framework2MiB y Frame256KiB/Consumer1MiB donde corresponde; reutilización de Arc no duplica allocation.
Mantener cargos físicos de ambos buffers/símbolos hasta readback de reemplazo/reset/drop, incluidos backend failure o cancel.
Frame completo incluye windows current+candidate, payload copiado corriente/candidato y adornos; grids Cell pertenecen Framework, no se cuentan como una tercera ventana.
120×40 old+new windows245696B dejan16448B para payload/adornos; ASCII dos payload9600B dejan6848B. El harness usa adornos<=4096B y los cobra.
No preasignar todo8192B de writer como texto de frame: metadata writer pertenece Framework. Cualquier memoria de frame real extra sí se añade.
Framework private2MiB conserva reservas fallibles/overlap/admisión oldnew originales, metadata8192 candidata y receipts efectivos; no garantía RSS/kernel.

Cobrar también escaneo r3_buffer_storage (ambos grids), comparaciones diff_iter y reset/swap/formatter; incluir bytes de símbolos comparados.
Dry preflight revisa source pinned y guarda fórmula conservadora checked; controles instrumentados confirman ausencia de caminos no cobrados.
Evitar receipts repetidos innecesarios en draw: uno inicial y uno final, ambos cobrados; resize propio conserva sus readbacks y costo separado de preparación.
No postcobro de símbolo tras escribir: preadmitir pico oldsymbol+newsymbol, readback valida y conserva cargo físico si sobrepasa.
CompactString.new no es asignación fallible: Allocation/RSS real no se promete contenida. Las entradas a reserve Vec siguen siendo fallibles.

Pinned ofrece Terminal.get_frame y apply_buffer_with_cursor(None): owner materializa en get_frame, cierra préstamo, reread cancel/binding y aplica después.
Rechazo resetea current_buffer_mut sólo dentro owner y conserva cargos hasta readback; nunca publica Frame/Buffer mutable al caller.
ForcedWidth omite workaround VS16 upstream. Nuevo hook r3_apply_buffer_admitted preserva diff_iter prestado y el mismo backend, con preclears explícitos:
por cada update ancho>1, emitir primero las referencias de sus continuaciones blank+stylebase y luego el glyph base; sin comprobar texto/Unicode ni Vec.
Un iterador fijo checked recorre sólo currentgrid y widths ya admitidos; output como máximo2*nCells, celdas repetidas cobradas y filas nunca cruzadas.
Continuaciones Skip afectan diff original; el hook las emite directamente para la limpieza previa. Narrow tras wide actualiza sus celdas vacías normales.
El hook conserva cursor/swap/reset/backend.flush/CompletedFrame/count del flujo apply original. No altera diff.rs, enum ni apply original de Legacy.
Backend error incluso tras swap antes flush final: poison+restauración; no afirmar atomicidad física ni que la grid antigua siga current.
Geometría admitida antes materializar; Fixed y lifecycle manual impiden resize oculto. Resize.backend_clear sólo tras preflight/último check de cambio admitido.
Las sesiones Legacy siguen usando API ratatui upstream; probar que hooks nuevos no cambian init/autoresize/draw/flush Legacy.

## 4. Criterios de primer corte y controles previos

R01 archive/hash y derivación:85/18/11/18 registros, deltas mínimos, no WIP completo, fixtures protegidas.
R02 factory real fixture4920B en80×24/120×40:Text, symbols/rangos/filas idénticos a expectativas anteriores, conserva SourceArc, joined y no busy.
R03 packing:roundtrip TODOS campos, clases separadas, overflow/ref exact-1/exact/+1, oldnew/drop; replay D2b1 ANCHOR01/C1-PROGRESS01 y ResolvedUnit originales.
R04 output real (ab -> ♥️wide+Skip; CJK/ZWJ/VS16/styledwide -> narrow/blanks; preclear antes base):80×24 contenido1440X;120×40 contenido4800X, último glyph/última fila, dos frames y resize120↔121→80;
comprobar bytes ANSI sólo en writer real, aplicación a parser VT/ConPTY, ForcedWidth sin segunda medición, continuaciones y borrado.
R05 símbolo25/33B real y símbolos16KiB/+1; exact/-1 quotas; ambos heaps actual/previous, cargoConsumer/Frame y Drop, sin prefix publicado.
R06 backend geometría cambia entre medida/draw:Fixed conserva grids; geometría0, reserve2 falla tras1, errorbackend/poison y cancelcallback original.
R07 cancel/stale antes preflight/después reservas/último callback:no flush nuevo/no publicación de binding; permiso gastado no reusable.
R08 Work count incluye scans/diff/preclear/formatter/reset:4MiB candidato total debe conservar éxito120×40; runtime falla gate si supera. Una prueba32MiB unicode/control sólo al introducir esta frontera conserva Reference completa y posterior ordinary; no repetir gigante por cada ronda.
R09 writermetadata/guard:medir valores/capacidades reales camino StdoutLock/Crossterm; no incluir Vec harness en claim8192; fmt/Clippy locked y tests focales.
R10 regresión selector inline/Nu y SourceLegacy:sin cambio CLI en primer corte; no modificación text_engine; combinaciones afectadas y APILegacy pinned.

Adversario revisará plan exacto antes código y candidato exacto después; AE tendrá sus propios oráculos/testigos80×24/120×40.
Éxito del primer corte significa renderer real compuesto en biblioteca+harness privado, pendiente adopción. No cierre RV-301, QAvisual, pump o Unix.

## 5. Siguiente corte D2c sin rediseño genérico

src/document_session.rs público run_markdown conserva ArcOriginal/origin/config; main branch Markdown interactivo se conectará sólo tras cierre renderer.
TerminalGuard de src/session.rs y KeyboardReader de viewer_session.rs se reutilizan con extracción mecánica mínima común; no segundo lector concurrente.
Entrada:guard+reader+Loading antes hash/Source/model/Document; DocController separado de DocumentPreparationAuthority compartida HASH/SOURCE/D2.
Todo inicio y End/Line/Resize/Fold se solicita con DocumentView.request()/start_document, poll y take_validated; draw sólo preparedstate.
Una petición reemplazable al supersede, stop anterior y reap físico antes spawn; nada de SafeTextModel::new síncrono en main para Markdown.
q/Escape0, Ctrl-C130, readerFailure1 bloquean latecommit/fallback/spawn; primero detener reader/restaurar guard, después cleanup2s con owner entero Pending.
Fuente usa SOURCE real de autoridad, manteniendo ArcOriginal/procedencia y contexto; fallo local no afirma FuenteReady sin modelo completo.
Comandos iniciales:Home/End,j/k,páginas,resize y Tab Fuente/Documento; búsqueda/actions/history completos conservan D2b3 posterior y se anuncian pendientes.
ConPTY deberá comprobar Loading responsive, teclas con stdinpipeline, restitución y siguiente invocación; pruebas de código no acreditan QAvisual.
No exigir completarD2c para revisar/adoptar primer corte; no atribuirle pump cuando sólo existe renderer.

## Delta R2 y decisión de perfil
R1 y SHA498E6290 se preservan. Responde a ForcedWidth/VS16 y mappingstyles señalados por adversario; hook get_frame público evita cambio innecesario render.rs.
Dos readbacks de9600Cell a48B cuestan921600Work, ya sumados a1327023layout superan2MiB antes diff/reset/materialización: perfil4MiB explícito candidato.
Fórmula conservadora previa120×40: layout1327023 + dos lecturas/reconstruccionesGlyph(2*4800*72=691200) + readbacks921600 + diff2Cell por4800(460800) + reset230400 + formatter/preclears <=491520 + symbols/copias/rangos <=65536 =4188079, menor4MiB4194304 por sólo6225.
Esta es predicción estrecha, NO admisión garantizada: receipt/charged costs efectivos completos y pruebas independientes deciden el importe; scansextra no se ocultan.
No rebajar costo para caber, no segundo allowance; si4MiB falla, elevar perfil concreto únicamente tras medición explícita y dictamen adversario/AE. MemoriaWindow/Frame/Consumer no se eleva.
El primercorte es texto/spanstyle y recursos causan referencia segura; roles/decoraciones/table/resource textual completo permanecen b2. No RV301 ni b2completo.