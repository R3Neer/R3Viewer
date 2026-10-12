# QA manual pendiente de S03

Preparación solicitada2026-10-10; NO ejecutada. S03 permanece abierto. Esta guía cubre datos, fuente, búsqueda y sintaxis; Markdown enriquecido/PDF/gráficos pertenecen a otros incrementos.

## Preparar una sesión aislada

Usar Windows Terminal y WezTerm reales, uno cada vez. Si falta WezTerm, registrar NO EJECUTADO: esta guía no instala nada. Desde el repo preparar un bundle privado nuevo con el reader y companion de target/release y una copia de nu/r3viewer.nu conservando su relación ../bin. Abrir Nu con --no-config-file y --plugins seguido de la ruta exacta del companion; importar ese módulo con use. No modificar config.nu, env.nu ni perfil personal. Fijar candidato commit/snapshot, hashes de reader/companion/módulo/fixtures y version de Nu antes y después; un rebuild necesita nueva identidad.

Registrar SO/build, terminal/release, fuente/tamaño, fallback fonts, DPI/zoom, teclado/layout y tema. Capturas reales de pantalla con caso/host/candidato; preservar resultado FAIL o INCONCLUSIVE. No reemplazar capturas por GridVT/ConPTY. Probar dark/light/auto en cada host, tamaños80x24 y40x12 y resize posterior; los valores son puntos de observación, no nueva restricción de tamaño mínimo.

## Casos y criterios

| Caso | Acción manual | Evidencia esperada |
|---|---|---|
| V01 datos Nu | [{z: null a: ""}, {a: "漢字 é 👩‍💻" m: [1 {x: true}]}] pipe viewer; Enter/retroceso/scroll/resize | Orden de columnas por aparición, ausencia/nothing/vacío distintos, contenido completo inspeccionable, selección estable por identidad. |
| V02 fuente segura | Abrir tests/fixtures/source-common/source-controls.txt con --format text; buscar literal y navegar n/N | Controles representados visibles sin acciones ESC/OSC; CJK/combining/ZWJ sin recorte/duplicación, highlight de graphema completo. No mostrar fixture crudo con cat. |
| V03 dense/checkpoint | source-dense.txt y checkpoint-selection.txt; /, Enter, n/N, ESC edición, scroll horizontal/vertical y resize | Posición y highlight coherentes; volver preserva selección/scroll; texto completo no reemplazado por elipsis. |
| V04 texto/código | Fixtures syntax-d4 NU/RUST/PS explícitos --format code --language respectivo; comentario multilinea antes viewport | Estilos correctos al scroll; cambio de pane/documento no revive estilos viejos; plain/never conserva lectura completa. |
| V05 borde/layout | Longline, tabs, CRLF, líneas vacías, CJK/emoji en última columna, números opcionales | No spill a statusbar/bordes, desplazamiento por celdas coherente y limpieza de restos después de resize/scroll. |
| V06 estructuras | Archivos propios TOML/JSON/YAML/XML: Fuente por defecto, Estructura y Value; claves Unicode/ausentes, hoja multilinea y regreso | Pane/identidad estables; hoja completa; números/string/bool se distinguen; Fuente conserva comentarios/espaciado. |
| V07 fallback | Lenguaje desconocido y línea sobre64KiB, colour never y fuente redirigida | Diagnóstico discreto, documento íntegro plano sin estilos parciales antiguos; inline sin teclado/raw. |
| V08 paleta | Config Nu aislada con string/header RGB#123456; auto y override TOML; dark/light; never | Foreground Nu visible en auto, overrides respetados; closure de color con efecto permanece sin ejecutar. Fondo/atributos fuera del alcance C01. |
| V09 teclado real | /, {}, @, acento, AltGr+C, Shift, teclas mantenidas Repeat y release | Texto exactamente una vez donde procede; AltGr+C no Ctrl-C; Escape edición y fuera mantienen contrato. No inferir resultado desde ConPTY. |
| V10 cierre | q; abrir otra sesión; Ctrl-C real; abrir otra sesión | q0/Ctrl-C130, cursor/modos recuperados y shell usable; incluir captura antes/después y exit, sin cerrar terminal para ocultar defecto. |
| V11 inline largo | Fixture JSON10000 de W3, pendiente de aceptación runtime y texto/code largo; scrollback y resize/zoom | Contenido completo legible, sin limpiar shell ni activar sesión interactiva; correspondencia automática W3 es evidencia distinta. |
| V12 eventos/lifecycle | Cambiar hojas rápido durante sintaxis y resize repetido; búsqueda/Value/Source/Estructura | UI sensible, estilo y selección del pane actual, cierre limpio; no reclamar exhaustividad de intercalaciones. |

Si el bundle/caso todavía no está disponible, marcar pendiente y enlazar su preparación; no PASS por ausencia. No reutilizar apuntes de Clases como fixtures. Los fixtures citados son propios del repo. Corpus extra generado para fronteras conserva receta/hash antes de observar resultados.

## Registro por ejecución

Copiar una fila por caso, host y variante realmente probada: fecha; commit/snapshot; hashes; SO/terminal/Nu/fuente/DPI/teclado; comando exacto; dimensiones/tema; acciones; resultado PASS/FAIL/INCONCLUSIVE; captura real; exit/restauración; observación y hallazgo. Un caso con fallbackfont o recorte ambiguo queda INCONCLUSIVE. Adversario contrasta criterios originales, evidencia e identidad, sin convertir una preferencia estética en fallo.

Windows funcional previo no certifica apariencia. La matriz Unix incluye QA manual WezTerm propia adicional después de habilitar runtime nativo; hosted CI no reproduce fuentes/DPI/ventana del usuario. Consultar S03-CIERRE y CONTENCION-UNIX antes de declarar cierre.

## Receta de bundle y corpus propio Windows

Ejecutar estos comandos manualmente desde PowerShell en D:\Tools\R3Viewer después de build aprobado. Es preparación local, sin cambiar perfiles. El destino nuevo debe estar vacío; si existe, elegir otro nombre en vez de borrarlo.

```powershell
. .\scripts\dev.ps1
cargo build --offline --locked --release --workspace
if ($LASTEXITCODE -ne 0) { throw 'Build falló; no continuar' }
$qaRoot = Join-Path (Get-Location) '.tmp\qa-s03-manual-r1'
if (Test-Path -LiteralPath $qaRoot) { throw 'Elegir destino nuevo' }
New-Item -ItemType Directory -Path "$qaRoot\bin", "$qaRoot\nu", "$qaRoot\fixtures", "$qaRoot\evidence" | Out-Null
Copy-Item -LiteralPath '.\target\release\r3viewer.exe' -Destination "$qaRoot\bin\r3viewer.exe"
Copy-Item -LiteralPath '.\target\release\nu_plugin_r3viewer.exe' -Destination "$qaRoot\bin\nu_plugin_r3viewer.exe"
Copy-Item -LiteralPath '.\nu\r3viewer.nu' -Destination "$qaRoot\nu\r3viewer.nu"
$qaText = "漢字 e$([char]0x301) 👩‍💻`tTAB`r`nESC=$([char]27)Xfoo`r`nC1=$([char]0x85)literal`r`n" + ('L' * 70000) + "`r`n"
[IO.File]::WriteAllText("$qaRoot\fixtures\unicode-controls.txt", $qaText, [Text.UTF8Encoding]::new($false))
[IO.File]::WriteAllText("$qaRoot\fixtures\data.json", '{"漢字":{"text":"é 👩‍💻","items":[1,true,null,""]}}')
[IO.File]::WriteAllText("$qaRoot\fixtures\data.toml", "# comentario`r`n['漢字']`r`ntext='é 👩‍💻'`r`nitems=[1,2]`r`n")
[IO.File]::WriteAllText("$qaRoot\fixtures\data.yaml", "# comentario`r`n漢字:`r`n  text: 'é 👩‍💻'`r`n  items: [1, true, null, '']`r`n")
[IO.File]::WriteAllText("$qaRoot\fixtures\data.xml", '<root><!--comentario--><item key="漢字">é 👩‍💻</item><empty/></root>')
[IO.File]::WriteAllText("$qaRoot\fixtures\empty.toml", '')
[IO.File]::WriteAllText("$qaRoot\fixtures\override.toml", "[colours]`r`nvalue='#ff0000'`r`n")
Get-ChildItem -LiteralPath "$qaRoot\bin", "$qaRoot\nu", "$qaRoot\fixtures" -File | Get-FileHash -Algorithm SHA256 | Format-Table -AutoSize | Out-String -Width 300 | Set-Content -LiteralPath "$qaRoot\evidence\hashes-before.txt"
& 'D:\Program Files\nu\bin\nu.exe' -n --plugins "$qaRoot\bin\nu_plugin_r3viewer.exe"
```

En cada terminal real, dentro de esa Nu aislada:

```nu
version
use D:/Tools/R3Viewer/.tmp/qa-s03-manual-r1/nu/r3viewer.nu *
let qa = 'D:/Tools/R3Viewer/.tmp/qa-s03-manual-r1/fixtures'
[{z: null a: ''}, {a: '漢字 é 👩‍💻' m: [1 {x: true}]}] | viewer --config ($qa | path join empty.toml)
viewer ($qa | path join unicode-controls.txt) --format text --config ($qa | path join empty.toml)
viewer ($qa | path join data.json) --structure --format json --config ($qa | path join empty.toml)
viewer ($qa | path join data.toml) --structure --format toml --config ($qa | path join empty.toml)
viewer ($qa | path join data.yaml) --structure --format yaml --config ($qa | path join empty.toml)
viewer ($qa | path join data.xml) --structure --format xml --config ($qa | path join empty.toml)
viewer D:/Tools/R3Viewer/tests/fixtures/syntax-d4/RUST-COMMENT.txt --format code --language rust --theme dark --colour always --config ($qa | path join empty.toml)
$env.config.color_config = ($env.config.color_config | merge {string: '#123456' header: '#123456'})
'Color de prueba' | viewer --theme auto --colour always --config ($qa | path join empty.toml)
'Override rojo' | viewer --theme auto --colour always --config ($qa | path join override.toml)
'Plano' | viewer --colour never --config ($qa | path join empty.toml)
```

Los tokens toml/json/yaml/xml se contrastaron con Format del candidato C01. Para V04 cambiar --language a nushell/powershell y el fixture exacto correspondiente; para temas usar dark/light/auto. Tras q/Ctrl-C, registrar `$env.LAST_EXIT_CODE` inmediatamente, antes de otro externo. Repetir manualmente en ambos hosts; no ejecutar los comandos mediante automatización UI.

Fuente/Estructura se alterna con Tab/Shift-Tab; Enter inspecciona y Backspace regresa; ? muestra ayuda. Sin --structure, los archivos de datos abren Fuente por defecto; --structure solicita Estructura inicial en TUI y salida estructural en inline. Registrar teclas exactas utilizadas en evidencia. V11 requiere antes un fixture W3/expected/hashes aceptado; este documento no atribuye aprobación a su diseño solamente.
