# Matriz CI local pendiente de activación

Solicitada2026-10-10. Es una preparación local; sin remoto, workflow instalado, dispatch, secretos ni consumo de Actions. El YAML adjunto describe la matriz y sus gates; no es un workflow operativo. Ningún job ejecutado. Mantener S03/U-G1–5/QA abiertos.

| Plataforma | Runner candidato | Arquitectura exigida en ejecución |
|---|---|---|
| Windows | windows-2025 | x86_64 |
| Linux | ubuntu-24.04 | x86_64 |
| macOS | macos-14 | arm64 |

Labels verificados en [documentación GitHub](https://docs.github.com/en/actions/reference/runners/github-hosted-runners) y [runner-images](https://github.com/actions/runner-images)2026-10-10; volver a confirmar disponibilidad/plan/imagen al activar. Registrar arquitectura efectiva, versión imagen/kernel y Rust/Nu exactos. No usar latest para fijar la identidad ni asumir que un label conserva imagen inmutable.

## Lanes separadas

1. Build/calidad: Cargo.lock intacto, fmt, Clippy y tests pertinentes, reader+companion reales en target nativo. Nu0.115.1 es referencia Windows, no compatibilidad certificada Unix. Rust1.99.0 es toolchain privado observado: fijar proveedor/release disponible antes de instalar en CI. Actions y descargas deberán quedar fijadas por SHA/digest revisado; resolver pins al configurar, no inventarlos en esta preparación.
2. Contrato Nu: plugin/módulo/reader reales, corpus tipos/framing/origen/cwd, enumeración única, colour/redirección/paleta y estructuras. Fallo de build companion en macOS es FAIL de preparación, no skipverde.
3. Contención U-G1–3: ejecutar probes R4 y verificar baseline/ABI/límites/cobro antes nacimiento/exec, guardian/parser/leases/ACK/FD, memoria y pérdida de dueños sobre procesos propios. Preparar copia versionada/revisada de los probes privados y comandos antes activar; sus rutas .tmp no están disponibles por checkout público. No activar backend Unsupported sin implementar y aceptar previamente el contrato nativo. RLIMIT_AS512MiB no equivale a committed512MiB de Windows.
4. Runtime U-G4–5: reader+companion+Nu reales, gramáticas/estructuras/budgets, PTY y restauración/q0/Ctrl-C130/nextsession, wait/EOF/joins; stdout/exit/hashes y fuente exacta. Crossbuild Windows no sustituye estos jobs.
5. QA visual manual: fuera de hosted jobs, documento y capturas reales por host. CI genera/valida artefactos funcionales, no certifica apariencia.

## Condiciones de habilitación

Configurar sólo después de escoger repositorio/destino, disponibilidad y coste de runners, y versión/pins de toolchain/Nu/acciones. No publicar ni crear remoto desde este documento. Fijar permisos mínimos contents:read y disparo manual workflow_dispatch inicial. El dispatch requiere workflow en rama por defecto según [GitHub](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/manually-run-a-workflow). Sin ejecución desde forks no confiables con credenciales.

Inventario prospectivo: commit/lock/acciones/dependencias/imagen/SO/arch/Nu/binarios/fixtures/expectativas antes outputs. Publicar únicamente fixtures sintéticos y logs sin apuntes, paths personales ni secretos; conservar logs incluso FAIL/INCONCLUSIVE, no rerunverde reemplazando originales. Timeout exterior de job es rescate CI, no deadline del producto ni prueba de reap. Ausencia de backend/permisos/entorno/harness produce gate pendiente y resultado no aceptado; skipped/ignored no cuentan como PASS. Un agregador final sólo cierra gates ejecutados/aceptados y mantiene QA manual distinta.

El YAML es una matriz de preparación consumible al escribir workflow futuro, con estado pending para todas las lanes. No contiene comandos que aparenten certificación o pasos echoPASS. La activación y el backend nativo siguen trabajo pendiente.

## Comandos locales disponibles antes de configurar jobs

Windows PowerShell (desde el checkout):

```powershell
. .\scripts\dev.ps1
cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) { throw 'Check falló; conservar log y detener' }
cargo build --offline --locked --release --workspace
if ($LASTEXITCODE -ne 0) { throw 'Check falló; conservar log y detener' }
$env:R3VIEWER_DOCUMENT_TEST_EXE = Join-Path (Get-Location) 'target\release\r3viewer.exe'
$env:R3VIEWER_SYNTAX_TEST_EXE = $env:R3VIEWER_DOCUMENT_TEST_EXE
$ciRoot = Join-Path (Get-Location) '.tmp\ci-s03-r1'
if (Test-Path -LiteralPath $ciRoot) { throw 'Elegir destino CI nuevo' }
New-Item -ItemType Directory -Path $ciRoot -Force | Out-Null
$env:R3VIEWER_DOCUMENT_TEST_CONFIG = Join-Path $ciRoot 'empty.toml'
[IO.File]::WriteAllText($env:R3VIEWER_DOCUMENT_TEST_CONFIG, '')
cargo test --offline --locked --release --workspace -- --test-threads=1
if ($LASTEXITCODE -ne 0) { throw 'Check falló; conservar log y detener' }
cargo clippy --offline --locked --release --workspace --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) { throw 'Check falló; conservar log y detener' }
& 'C:\Users\Usuario\AppData\Local\Programs\nu\bin\nu.exe' -n -c 'nu-check --as-module nu/r3viewer.nu'
if ($LASTEXITCODE -ne 0) { throw 'Check falló; conservar log y detener' }
```

Comprobar `$LASTEXITCODE` tras cada comando; detener ante fallo y conservar log. Destino config privado nuevo; no reutilizar configuración del usuario. El modo offline requiere dependencias privadas ya preparadas; un runner nuevo primero necesita instalación/downloads por hashes autorizados.

POSIX candidato después de instalar toolchain/Nu exactos y preparar dependencias; NO ejecutado aquí:

```sh
cargo fmt --all -- --check || exit $?
cargo build --offline --locked --release --workspace || exit $?
mkdir -p .tmp || exit $?
if [ -e .tmp/ci-s03-r1 ]; then printf '%s\n' 'Elegir destino CI nuevo' >&2; exit 2; fi
mkdir .tmp/ci-s03-r1 || exit $?
printf '' > .tmp/ci-s03-r1/empty.toml || exit $?
export R3VIEWER_DOCUMENT_TEST_EXE="$PWD/target/release/r3viewer"
export R3VIEWER_SYNTAX_TEST_EXE="$R3VIEWER_DOCUMENT_TEST_EXE"
export R3VIEWER_DOCUMENT_TEST_CONFIG="$PWD/.tmp/ci-s03-r1/empty.toml"
cargo test --offline --locked --release --workspace -- --test-threads=1 || exit $?
cargo clippy --offline --locked --release --workspace --all-targets -- -D warnings || exit $?
nu -n -c 'nu-check --as-module nu/r3viewer.nu' || exit $?
```

No atribuir PASS nativo a los tests Windows cfg-excluidos. Los comandos de contrato plugin/PTY/contención todavía requieren harness versionado y backend real: gates pendientes, sin shell echoPASS ni invocaciones a rutas privadas inexistentes. La receta de probes existente en S03_UNIX_PROBES.md se copia/revisa sólo al preparar esos jobs; sus controles `--execute-native` exigen host nativo y no certifican producción. Estos bloques de build/calidad no son el workflow final ni resolución de pins.
