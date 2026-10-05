# S03 shell: revisión independiente integrada R1

2026-10-05. Revisor independiente de la implementación de unix_r3_review. Contraste de plan canónico GRAMATICAS-SHELL-INTEGRACION y su review; sólo snapshot/copia privados, sin producción ni targets compartidos.

## Dictamen e identidad

**SIN_CRITICAS_PENDIENTES, acotado a esta integración de recursos fijos, selección y motor Windows.** No cierre RV/S03, QA visual, runtime Unix ni contención general certificada.

Snapshot `shell-integrated-r1/MANIFEST.json` SHA256 `257332BF8D0B2BE8B4B649B264E21E69D95B5555D689BD78C92596FC7C5EAB67`:427 tamaños/hashes comprobados antes/después, cero discrepancias. Bin reader `2F6F41D3DC32B471C5AEA88F050827A594394485BECF04C255718198758DE7AC`; companion `4EB4CCE0EFE1B99197C8DC2CA4EF1BED1030375A704A5A528547D3E115AB06C2`. Base de freezea1a600a; no se sustituyó por checkout mutable ni HEAD documental posterior.

Entrega R1 `FD59F7E0B86C88F94E051E5B7360BBC054B214957C689A73DCCC16677C6335F8`, conservada. Corrección exclusivamente documental R2 `61BEB6D810F6EA01ACD1F6809AB616F022EA35E1315A2727CF6D36E3DB7A559F`, leída/verificada sobre **el mismo candidato**. Evidencia independiente en `shell-integrated-adversary-r1`, manifest `DD61B12C598576C58E22A0D11E73B12A377B27325758A7CB4D23EE641CE63BE0`.

## Corrección de atribución del motor

Detecté que R1 atribuía el Engine de producto Windows a fancy. Cargo declara regex-onig en target Windows y regex-fancy en build-dependencies. Fuente primaria instalada Syntect5.3.0 `src/parsing/regex.rs`:133 activa Onig;191 declara que incluso con ambas features prevalece Onig. Resolver3 separa el builder. Logs de mi compilación incluyen Onig6.5.3. El worker/Engine real ejecutado es **Onig**; fancy genera el pack en build. R2 corrige ese hecho sin cambiar fuentes/binarios ni reclamar otro PASS fancy de producto. No es defecto de ejecución ni cambio de recurso; la atribución anterior no se reutiliza como evidencia.

## Fuente, recursos y coherencia

build.rs crea un único set defaults+TOML+Nu+PS y genera dump y catálogo del mismo set; orden preservado, strings escapados mediante Debug Rust, rerun para cada recurso y build.rs. Carga/enlaces inválidos abortan build; no descarga/carga usuario. La selección conserva recorrido reverso de extensiones y directo de nombres. Alias pwsh se limita a cli.language explícito: .pwsh auto queda plano. Text fuerza plano; validate conserva conflicto text+language2. Metadata/cwd de Nu no simula ruta ni selecciona estructura; --language sólo resalta.

Recursos exactos: Nu `1A0972876A8EE4BF1C3E97E12B9B71FDF82F42F87685062620484818B3D577AF`; PS `2A41915FDB3314E2DCBFDE93CBE6EF19BF5694595A5C173AAADB538159AF6B36`. Procedencia, upstream original, ledger y avisos MIT preservados/leídos. No nuevo converter ni equivalencia TextMate universal. No cambio de Cargo/dependencias de producto o scheduler/Job/Fuente/latch/cuotas; cuatro fuentes delimitadas por diff, más recursos. Instrumentación scopes se compila sólo cfg(test), no asignación/traza en runtime.

## Pruebas independientes

En copia propia y **isolated-target** propio, Rust1.99 offline/locked:

- Dos tests shell existentes PASS, sin ignores:84documentos en Engine real,269intervalos UTF8 exactos,16endStacks, continuidad y seek/replay. Oracles anteriores al candidato; se reprodujeron sus expectativas sin generarlas desde salida.
- Test catálogo contra SyntaxSet empaquetado PASS; ningún ignore. Catálogo78 verificado también por API del ejemplo propio.
- Ejemplo propio `source/examples/adversary-shell.rs`: selección de Nu/PS/TOML/JSON/Rust y colisiónh→Objective-C; mayúsculas/nombres/pwsh/espacios explícitos; negativos .pwsh/.PWSH/desconocido y literal/stdin/Nu auto plano; text+languageerror2; --language sin estructura. PASS.
- Ese ejemplo usa SyntaxClient y dispatcher privado reales: dos documentos propios CRLF/Unicode/EOF,610 líneas, cuatro viewports590→0→300→10 por lenguaje. IDs/generación, continuidad y límites UTF8 de spans comprobados; ReapState::Reaped antes del siguiente documento. All mediante highlight_inline tras reap conserva cobertura completa. Shutdown/join PASS. No simulación de worker ni cambio de límites.
-15 casos propios con bins congelados: seis selectors explícitos, cuatro archivos NU/psm1/PSD1/pwsh, stdin, text, conflicto, stringNu que parece ruta y recordNu inert. UTF8 exacto de fuente en casos textuales, salida redirigida sin ANSI, conflictoexit2. Comandos touch/New-Item dentro de la fuente no crean el canario privado; no evaluación de fuente.

El primer intento de mi ejemplo usó request_all, que no es API pública del candidato; falló compilación antes de ejecución. Log conservado `adversary-worker-attempt1.log`; corrección sólo del harness propio para highlight_inline vigente. No cambio de producto ni resultado fallido presentado como PASS. El helper original de evidencia `shell-integrated-harness-main.rs` conserva un intento anterior; la fuente ejecutable final está en lifecycle-harness/src/main.rs, que usa API vigente.

## Límites del cierre

No repetí workspace215PASS/fourignores, ConPTY, todos los fallos/límites del autor ni crosschecksUnix; esos resultados permanecen evidencia de autor, requieren AE independiente. Mis nuevos casos aportan integración real Windows y viewport/reap, no estrés/carreras exhaustivos ni SLA. roles/continuidad no sustituyen scopes: intervalos/endStacks proceden del test Engine, no del protocolo. No juicio de paleta ni apariencia.

No existe --languages CLI ni se agregó. No instalación de perfiles, ejecución de Nu/PS como fuente, gramáticas de usuario o modificación de Clases. QA visual, Unix runtime/contención, estrés y resto de formatos/RV/S03 siguen abiertos. La aprobación de esta revisión no sustituye AE independiente del candidato exacto ni autoriza ampliar la atribución de sus evidencias.
