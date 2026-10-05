# S03: integración de gramáticas shell R1

Propuesta de incremento de producto, aún sin editar producción. Condición previa: snapshot privado shell-grammar-prototype-r2-final MANIFEST C6BFD9C471E4635720147EF0A25F927D238F3E0CE9A5D5EC2BF5FFBE48CB1418 debe cerrar revisión y AE independientes. El éxito de un runner no acredita el worker ni UI. S03 sigue activo, RV105/106/107 conservan pendientes visuales/Unix/estrés/resto de formatos.

## Cambio propuesto

Incorporar sólo recursos fijos Nu y PowerShell exactos revisados, con avisos MIT originales y procedencia (commit upstream, hashes, ledger de transformación manual y límites). No descargar gramáticas ni interpretar perfiles/scripts al abrir. No generador universal ni retocar scopes durante integración; cualquier modificación del recurso vuelve a revisión/oracle.

build.rs crea un único SyntaxSet: defaults Syntect5.3 + TOML actual + los dos recursos nuevos, con el loader ya vigente regex-fancy y enlaces compilados. Dump renombrado bundled.syntax.pack. Del mismo SyntaxSet generar en OUT_DIR un catálogo Rust de nombres/extensions por orden estable, con escapado Rust de strings y sin duplicar listas editadas a mano. language_catalog.rs incluye ese catálogo; syntax_worker::bundled_syntaxes usa el mismo dump. Rerun-if-changed para los tres recursos y build.rs; fallo de carga/enlace es build fallido, no eliminación silenciosa. El generador no genera sintaxis desde inputs de usuario ni usa metadatos del pipeline.

Preservar resolución existente y precedencia: --format text fuerza plano; --language explícito elige nombre/alias; archivo local deduce sólo su extensión real; texto stdin/literal/Nu no hereda lenguaje de cwd o contenido que parezca ruta. Nushell/nu resuelve nombre/extension del set; PowerShell/ps1/psm1/psd1 igualmente. Añadir pwsh sólo como alias explícito de --language: un archivo .pwsh no adquiere una extensión inventada. --language no invoca estructura ni intérprete. Nombres/extensions heredados siguen iguales (catálogo generado conserva el set real y orden). --languages muestra lo realmente integrado y aliases separados donde ayuda sea pertinente.

Scheduler, IPC, límites de fuente/línea/frame/spans/ops/scope/cache y Job antes de Init no cambian. Worker Onig carga dump del builder fancy y sólo publica spans validados sobre fuente exacta. Timeout/RegexCompile/SpanBudget/WorkerExit conserva Fuente completa y el latch/reap actual, sin reintento infinito. No cambiar presupuesto para que una gramática pase. No activar sintaxis Unix por tener recursos compilados.

## Verificación de integración

Pruebas pertinentes: catálogo generado igual al SyntaxSet embebido (Windows); resolución de todas las extensiones Nu/PS, mayúsculas, pwsh explícito frente a .pwsh auto, --format text y origen tipado; regresión TOML y otras gramáticas existentes. Reproducir 269 intervalos y16estados finales en el worker real sobre corpus fijo, con conversión explícita entre scopes internos y spans; si el worker no expone stack a UI, usar un harness de pruebas que llama al mismo procesamiento y límites, no el runner anterior como sustituto. Comparar spans/continuidad/longitudes de origen; oracle de endStack puede quedar adicional de parser exacto si el protocolo no lo publica, explicitándolo.

CLI/Nu: archivo .nu/.ps1, literal --language, strings por tubería y redirección conservan contenido completo sin ejecutar fuente; inline con ANSI cuando autorizado y plano cuando redirige. Corpus hostil de strings incompletas/regex largas/control/Unicode y fallo controlado del worker verifica fallback/cancel/reap/exclusión. TUI funcional propia con CtrlC/q/restauración y scroll/búsqueda conserva texto; ninguna prueba funcional se vende como paleta/QA visual.

Workspace tests, fmt/Clippy/build; crosscheck lector+wire Linux/macOS sólo como compilación, sin runtime ni contención. Congelar candidato integrado exacto y entregar a adversario y AE distintos del implementador antes de commit de aceptación. No cerrar RV/S03 si sólo estas capacidades pasan. No nuevos flags de formato, dependencias runtime ni instalación en perfiles.
