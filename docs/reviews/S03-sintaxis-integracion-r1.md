# S03 C04–C07 integración — revisión exacta R1

Veredicto: **SIN_CRITICAS_PENDIENTES** para la integración congelada de C04–C07 y sus interacciones afectadas con A/B y C01–03/C08. No certifica aceptación ConPTY, apariencia, matriz Unix ni cierre de S03/RV. El core aislado previamente revisado es evidencia separada; no repito ni atribuyo su AE como ejecución propia de integración.

## Identidad

Base6605cc237c37fbfcf49b6b1ecf399434f9a60a56. Snapshot R1/SNAPSHOT.json SHA256 9E7095A746EAF750C1DB35853C351A22794988A599452D4D8F77576A9FCEF959. Verifiqué los176archivos declarados: cero discrepancias. Binario app9AB78E023BD7FE2CE4BECBC7E136BC7274925B9454F5193026A19883B509B30C; companion0B9B04E6B7D1AC4ABCC871D99576BDEA5805AA4E0759619CA3E61E17BF488184.

Leí integración de CLI/main/origen/Nu, selección/catálogo, presentación/procedencia, inline y sesión TUI, y fronteras públicas de controller/worker/cleanup con impacto en integración. Las declaraciones de114pruebas se corresponden con logs70lib+11CLI+5wire-app+12plugin+16wire; fmt/Clippy/build son comprobaciones del autor, no QA propia ni visual.

## Hallazgo previo cerrado

S03-CINT-01 P2, resuelto: textual_input admite SingleValue String y ByteStream String explícito; Unknown/Binary mantienen descriptor y no reciben sintaxis. syntax_document aplica la misma frontera a panes. NodeIdentity::StringStream{pointer,length} conserva el Arc del modelo durante n/N y evita crear una nueva generación/reiniciar Plain latched para la misma hoja. La prueba de contraste tres kinds incluye preservación de identidad y fallback después de aplicar búsqueda en el stream. No se infiere extensión desde metadata, cwd o título y no se reabre el origen.

## Integración revisada

SourceOrigin LocalFile procede de apertura real, incluidas rutas Nu --path; literales/stdin/Nu conservan origen propio. Solo extensión real o --language seleccionan gramática. Catálogo real evita cargar Syntect en UI y coincide con resolver local: extensión exacta primero, después extensión sin punto opcional, ambas en reversa, y nombre case-insensitive en orden normal. Esto preserva extensiones que contienen punto inicial como .bashrc; no se atribuye el orden local de nombres a find_syntax_by_token. Gramática Plain Text/text/colour never/redirección evitan child; los contenedores solo permiten sintaxis al abrir strings/string_stream.

Cursor sparse monotónico conserva procedencia dentro de placeholders C1 y evita mapa por byte. Spans originales ordenados se localizan por partition_point inicial y avance por fila. Primer scalar original determina rol del graphema entero, con Value si no tiene span; búsqueda añade Reversed/Bold por intersección. El renderer no remapea globalmente50k spans ni copia la fuente para pasársela al controller.

SyntaxView identifica documento mediante Arc y valida generación/request/viewport. Scroll/resize/búsqueda de la misma hoja no reinician generación; otro pane sí. Plain del gen actual domina requests obsoletos y borra todos los spans; estilos tardíos no reviven la generación. Un Plain de otro gen no borra el actual. SyntaxOwner se declara antes del guard, inicia shutdown no bloqueante y hace join después de restore; el orden de Drop también protege unwind. Un controller por sesión conserva la recogida del child antes del siguiente.

Inline no modifica modos ni escribe durante preparación: obtiene All completo, compone buffers con límites/cancelación y solo entonces entrega salida. Fallback reconstruye documento completo plano; exceso de expansión de estilos no imprime prefijo. CRLF, tabs, fuente segura y último salto/vacío se conservan; números son presentación TTY y no dependen de color. Errores de CLI preceden config/documento, fallback de sintaxis conserva0/diagnóstico y lectura/salida mantienen errores propios. La ruta redirigida evita sintaxis/ANSI/números.

## Comprobaciones propias del binario congelado

Ocho casos aislados, sin perfiles ni producción: literal con ESC/C1/acento/CJK/combining/ZWJ redirigido con code/rs/always/números conserva fuente segura exacta sin ANSI/números; wire artesanal String stream conserva el mismo texto; Unknown y Binary conservan sus descriptores. String stream con scalar UTF-8 de cuatro bytes en offset65535, dividido entre frames64KiB, reconstruye todos los bytes correctamente. Conflicto text+language y lenguaje vacío devuelven2/diagnóstico antes de config/documento inexistentes. Idioma desconocido devuelve0 con fuente completa y diagnóstico. Fixtures temporales borrados; estos casos no prueban colores TTY ni ausencia de child por enumeración de procesos.

Pendiente AE de integración SC01–SC14 sobre TUI/inline reales, transición Plain integrada, procesos propios/cancelación/restitución y costes por fase. Modelo/TestBackend comprueban roles/offsets; ConPTY funcional no certifica forma Unicode ni contraste. macOS/Linux y QA visual siguen pendientes conforme contrato y gate provisional. No detecté otros defectos accionables en este snapshot.

## Limitación del motor identificada durante investigación paralela

Verifiqué fuente local Syntect5.3.0 parsing/regex.rs:183–187: search_with_param usa MatchParam::default y cualquier Err se convierte en no-match mediante matches!(matched, Ok(Some(_))). El worker map_err a GrammarError protege errores que parse_line/ScopeStack EXPONEN; no puede observar por esa API todos los fallos internos de Onig. La investigación todavía no reprodujo esta condición con gramáticas integradas ni el worker congelado; no se presenta como fallo runtime del producto ni motivo para modificar core sin evidencia. Fuente segura intacta y watchdog/budgets externos conservan protección. Registrar el spike y describir la cobertura como «errores expuestos por Syntect»; evitar promesa de captura de cada error interno del motor. La exactitud léxica de casos patológicos no queda certificada por esta revisión.
