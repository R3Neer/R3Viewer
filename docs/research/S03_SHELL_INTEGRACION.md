# S03: entrega de integración shell R1, corrección documental R2

Candidato integrado para revisión adversaria y AE independientes. No es cierre RV ni QA visual. No he creado commit.

Snapshot `shell-integrated-r1/MANIFEST.json`: SHA256 **257332BF8D0B2BE8B4B649B264E21E69D95B5555D689BD78C92596FC7C5EAB67**, 427 entradas (391 fuentes). Se copió la fuente completa versionada más los recursos nuevos autorizados, binarios, dump/catálogo generados y evidencia privada. Todos los hashes de fuente se compararon antes/después de copiar y coincidieron. HEAD observado al congelar: `a1a600a9271e1a21527971c3dd27ebee6e67e542`; al comenzar era `765fbb83bb5149c6690250377816f5a922d5825f`. Los cambios propios siguen sin commit: build.rs, src/language_catalog.rs, src/syntax_selection.rs, src/syntax_worker.rs y tres directorios resources/syntax/nushell,powershell,shell. Sin Cargo ni otros subsistemas modificados.

## Cambios y contratos

Build único: defaults de Syntect + TOML existente + Nushell/PowerShell R2. Del mismo SyntaxSet se generan bundled.syntax.pack y language_catalog.rs, en el orden original del set; referencias no enlazadas o recursos inválidos hacen fallar el build. Catálogo de 78 entradas. El test existente generated_catalog_matches_the_packaged_worker_grammar_set pasa. TOML, extensión/resolución, nombres existentes, Fuente, cuotas, Job y scheduler conservan sus rutas.

Nushell R2 exacto SHA256 1A0972876A8EE4BF1C3E97E12B9B71FDF82F42F87685062620484818B3D577AF. PowerShell R2 exacto SHA256 2A41915FDB3314E2DCBFDE93CBE6EF19BF5694595A5C173AAADB538159AF6B36. Se conservan fuente original, licencia MIT y ledger por gramática, con procedencia en resources/syntax/shell/README.md. No converter universal ni carga de gramáticas de usuario.

Alias pwsh sólo con --language explícito, insensible a mayúsculas. No se inventa extensión .pwsh: auto .pwsh y entrada literal sin selector conservan Plain Text. Formato text sigue Plain Text y text+language sigue error de validación. Se probaron nombres, extensiones, uppercase, alias y esos límites.

Desviación aclarada por root: la referencia del plan/propuesta a --languages no correspondía a la CLI vigente. No se añadió flag. El catálogo generado documenta los recursos reales; no se modifica la CLI para agregar un enumerador.

## Evidencia ejecutada por el autor integrado

Windows MSVC con Rust 1.99.0 y Nushell 0.115.1 existentes. Los logs y runners están dentro de evidence/.

- cargo test --locked --workspace: 215 PASS (159 lib +23 CLI +5 Nu wire +12 companion +16 wire), cuatro ignores XML preexistentes conservados. No se contabilizan como PASS.
- cargo test --locked --lib shell_ -- --nocapture: dos pruebas nuevas PASS. Engine::step real del worker, pack real y límites reales: 84 documentos, 269 intervalos UTF8 con stack completo ordenado y 16 endStacks. Oracles independientes fijados antes de R2; hashes de tres JSON en oracle-fixtures.rs. Validación de cobertura continua, roles, seek(0) y replay idéntico. Captura de scopes opcional sólo cfg(test), sin API/traza/asignación en producto.
- cargo clippy --locked --workspace --all-targets -- -D warnings, cargo build --locked --workspace, cargo fmt --all --check y git diff --check: PASS. Aviso futuro preexistente proc-macro-error2; crosscheck macOS muestra warning de dependencia crossterm existente.
- cargo check --locked -p r3viewer -p r3viewer-wire para x86_64-unknown-linux-gnu y aarch64-apple-darwin: PASS de compilación. No runtime Unix ni worker companion Unix aprobado por esto.
- Harness privado con public SyntaxClient y worker real dentro de Job: los 84 documentos pasan fuente/roles/generación y continuidad. ReapTicket=Reaped antes del siguiente caso. Nu y PowerShell: línea 65536 bytes aceptada, 65537 rechazada LongLine antes de Ready, latch y reap; viewport570→0 en documento600 líneas y checkpoints/reverse seek; All inline para ambos; cancel/shutdown/join. Todo con límites predeterminados.
- Dos fallos controlados del dispatcher exclusivo del harness: boot-stall→StartupTimeout ~2.03 s; exit→WorkerExited ~0.067 s. Latch/reap comprobados. Ejecutables y PIDs hijos retenidos por el runner; finally sólo mata/wait su propio hijo si hiciera falta y cierra lectores. No señales ni acciones sobre procesos ajenos.
- CLI y Nu redirigidos: 18 casos PASS, salida exacta UTF8 sin ANSI, selectores explícitos, archivo auto, stdin, string tipado Nu, archivo Nu, record Nu inert, text y conflicto. Canario inexistente verifica que comandos touch/New-Item dentro de la fuente no se evaluaron.
- ConPTY propio: .nu TUI con búsqueda/scroll enviados y q=0; .ps1 Ctrl-C=130; entrada503/salida7/cursor visible restaurados en ambos; inline colour con ANSI y sin alternate screen. Nu propio termina por exit0, wait confirmado, reader cerrado y sin terminate. Evidencia funcional de bytes/modos; sin captura visual ni juicio de apariencia.

El corpus de scopes se ejecutó en el motor de producto Windows Onig. Cargo.toml fija regex-onig para la dependencia Windows; regex-fancy pertenece a build-dependencies para generar el pack. Syntect5.3.0 da precedencia Onig si ambas features se unifican. R2 privado había sido revisado y AE había pasado fancy y Onig; ese antecedente no se presenta como ejecución fancy del Engine de producto. Lifecycle/protocolo sólo expone roles, por lo que no se atribuyen scopes de protocolo inexistentes: scopes/endStacks proceden del Engine real instrumentado en tests.

## Intentos del tooling

Se corrigieron tres defectos del harness antes del PASS: el serializador Rust inicial no admitía CRLF literal en raw string (se cambiaron sólo escapes de entrada); Python write_text traducía LF de fixture a CRLF (se pasó a write_bytes); open --raw produce byte stream Nu, se añadió into string para el caso de string tipado. ConPTY reutilizaba marcador __CODE__ y recogía la línea0 repintada de la ejecución previa; marcadores únicos por extensión resolvieron esa colisión. Se conserva intento1 ConPTY en evidence/. Ningún resultado de esos errores se usó como crítica o fix de producto.

## Identidades y gates pendientes

Binario r3viewer.exe SHA256 **2F6F41D3DC32B471C5AEA88F050827A594394485BECF04C255718198758DE7AC**; companion SHA256 **4EB4CCE0EFE1B99197C8DC2CA4EF1BED1030375A704A5A528547D3E115AB06C2**. Harness privado SHA256 06645A936E7DBC103CF5FCE1CE57B9F51EC34C4A02CF983C5DB1D2CF804998A2.

Preparado para adversario y AE sobre el snapshot exacto. Sus resultados independientes aún pendientes. Sin commit, instalación, publicación, perfiles, ejecución fuente, modificación de Clases, habilitación Unix ni cierre RV. QA visual humana/independiente sigue separada.

## Corrección de atribución del motor

Ambos revisores independientes señalaron la atribución inexacta del párrafo de motores. Se confirmó contra source/Cargo.toml del snapshot (líneas29–33) y código LOCAL Syntect5.3.0 src/parsing/regex.rs133/191–192: regex-onig gana cuando ambas features están habilitadas. El PASS Engine Windows ya ejecutado es Onig; no existe un nuevo PASS fancy de producto. Esto es sólo delta documental; no se ejecutó Cargo ni se tocaron fuentes, recursos, binarios o MANIFEST. R1 y snapshot427 se conservan íntegros.


## Estado vigente de integración

2026-10-05. Revisión de código y aceptación independientes terminadas sin críticas pendientes dentro del alcance funcional Windows ejecutado. Véanse ../reviews/S03-shell-integracion-code-r1.md y ../reviews/S03-shell-integracion-aceptacion-r1.md. Las frases anteriores de candidato pendiente describen la entrega congelada; este apartado fija el estado posterior. El coordinador contrastó los hashes de los cuatro archivos de código y todos los recursos nuevos con el snapshot antes de integrar. Los documentos añadidos posteriormente no alteran esos bytes. QA visual, contención/runtime Unix y aceptación completa RV-105/106/107/S03 siguen pendientes.
