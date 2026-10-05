# RV106 — recurso TOML derivado R4, evidencia temporal

Fecha: 2026-10-05. Alcance: recurso y harness propios bajo `.tmp/rv106-spike`; sin cambios en producción. **G-01 y G-02 corregidos en el corpus observado; snapshot listo para revisión independiente.** No se declara conformidad completa TOML 1.1 ni cierre RV106.

## Procedencia y preservación

Base original: `jasonwilliams/sublime_toml_highlighting` commit `f5a57e8bff694a4e6c52a491dae579aabc7427cf`, submódulo fijado por bat `v0.26.1`. MIT, Copyright 2021 Jason Williams. El original es una gramática anterior a las extensiones TOML 1.1; R4 es derivado local, no recurso upstream certificado. R3 conserva su archivo y evidencias; R4 es un archivo separado. La licencia original sigue en `.tmp/rv106-spike/LICENSE`.

Fuente primaria fijada: [TOML.sublime-syntax](https://github.com/jasonwilliams/sublime_toml_highlighting/blob/f5a57e8bff694a4e6c52a491dae579aabc7427cf/TOML.sublime-syntax), [LICENSE](https://github.com/jasonwilliams/sublime_toml_highlighting/blob/f5a57e8bff694a4e6c52a491dae579aabc7427cf/LICENSE). Los hashes del original y R3 se verificaron después de construir R4, sin modificación.

## Correcciones concretas

G-01, P2 comunicado por el adversario de R3: continuación básica multilínea con backslash, espacios y CRLF aceptada por el parser pero marcada ilegal por el recurso. R4 añade, exclusivamente en el contexto básico multilínea y antes de `string-escape`, el patrón `\\[ \t]*\r?\n`. Se marca `constant.character.escape.toml` y se empuja `multiline-basic-continuation`, que consume `[ \t\r\n]+` hasta el siguiente carácter no blanco y vuelve al contexto básico multilínea. El contexto sobrevive al paso por líneas y a checkpoints. No se incluye en cadenas básicas de una línea ni literales.

G-02, P2 comunicado por el adversario: cierre de cuatro comillas en cadenas básicas/literales multilínea dejaba una comilla como trailing ilegal. Solo los patrones de cierre cambian a `"{3,5}` y `'{3,5}` respectivamente; los de apertura permanecen de tres comillas. El cierre consume de forma greedy el bloque completo. El bloque de cierre comparte `punctuation.definition.string.end.toml`; no se pretende distinguir visualmente las una/dos comillas que semánticamente forman parte del contenido antes del delimitador final. El lexema y valor del parser siguen siendo autoridad para el contenido.

El diff reproducible R3→R4 está en `.tmp/rv106-spike/grammar-r4/transformations.patch`. Además de estas reglas, contiene un comentario de procedencia temporal y diferencias de finales de línea localizadas en las inserciones; no hay nuevas gramáticas externas o inclusiones.

## Evidencia runtime

Windows local; cargo 1.99.0, Syntect 5.3.0 con `regex-onig`/`yaml-load`, toml_edit 0.25.15+spec-1.1 y lock separado. `SyntaxDefinition::load_from_str(..., true, None)` compila el recurso, y `ParseState::parse_line` se ejecuta sobre `split_inclusive('\n')`. Los errores retornados se verifican mediante `unwrap`; no se usó otra implementación de resaltado.

`grammar-r4/tests-final.log`: **10 PASS, 0 FAIL, 2.97 s** sobre el harness final formateado. Se conservan las siete pruebas de R3 y se añaden tres pruebas parametrizadas:

| Corpus nuevo | Casos | Aserciones observadas |
| --- | ---: | --- |
| Basic/literal, apertura 3/4/5 y cierre 3/4/5, LF/CRLF | 36 | Parser acepta; valor exacto con 0/1/2 comillas adicionales al inicio/final; `abc` conserva scope basic/literal; ninguna marca `invalid.illegal`; `flag=true` vuelve a `constant.language.toml` |
| Continuación basic multiline LF/CRLF; 0/espacios/tab/mixto antes y después; con/sin línea adicional blanca | 64 | Parser acepta y devuelve `helloworld`; backslash+blancos+newline tienen `constant.character.escape.toml`; `world` retorna al scope basic; booleano posterior correcto; ninguna marca ilegal |
| Basic de una línea con LF/CRLF; intento de continuación en basic de una línea; backslash+espacios/tab sin newline dentro de multilínea | 6 | Parser rechaza y al menos un segmento tiene `invalid.illegal` |

Cada caso nuevo compara las operaciones, `ScopeStack` y `ParseState` obtenidos al continuar normalmente contra un clon completo tomado en **cada** frontera de línea, incluida frontera inicial/final. Esto cubre en particular el checkpoint mientras está activo `multiline-basic-continuation`, después de una línea blanca y antes del cierre. No comprueba la integración del caché del visor.

Las siete regresiones conservadas cubren categorías de fechas/horas con segundos opcionales/fracciones; escapes anteriores y `e`/`x`; inline tables multilínea/comentarios/anidamiento/trailing comma; CRLF y retorno a tabla; negativos; arrays/números/cadenas existentes; checkpoints de estados anidados. `grammar-r4/scopes.log` conserva un volcado del caso complejo R3 ejecutado contra R4 y `R4 complex_parser_ok=true`.

`grammar-r4/build.log`: build satisfactorio. Dos avisos `dead_code` de helpers usados solo por tests; no se ejecutó ni se declara Clippy. No se lanzaron procesos del visor ni workers de producción: solo compilador/test runner y el binario temporal propio.

## Snapshot SHA256

Rutas relativas a `D:\Tools\R3Viewer\.tmp\rv106-spike`:

| Archivo | SHA256 |
| --- | --- |
| `TOML.sublime-syntax` original preservado | `581328AEA6C0028AB613FF26B3114C4AE37F9FA29F7BE1042419BA353A6EABC6` |
| `TOML-R3.sublime-syntax` preservado | `5DD5A433C034FABE85E062C33F4C1C7E64E545D9F0B9F5CD47E9E9D71AA16087` |
| `TOML-R4.sublime-syntax` | `06B80C0D48F6BC9E9B7339E1D2C79ACD1791C61465DAB4A334BDAE1723268F93` |
| `grammar-r4/src/main.rs` | `237D2183E9615747CDB98ADDD10F1CE9EB4F8B4D7B97FD41B6E7C5EE1DFFE3B7` |
| `grammar-r4/Cargo.toml` | `FCDB777F99B015FD35E3C8E16EBAF283CF983D0AFFF02AFDAE3E546E274D36FD` |
| `grammar-r4/Cargo.lock` | `7FF0A6FF201F90831FBD6D59F1A9E4EB8B3B54E8DE59C597B546D46BED50EE59` |
| `target-private/debug/rv106-adversary-grammar-r4.exe` | `19DE2F95F5E7EB54B4E049D55E8075A25AA1E64344CC0077A559FC472BD1A609` |

Copias de recurso/harness en la carpeta de visualizaciones: `TOML-R4.sublime-syntax` y `rv106-grammar-r4-main.rs`. El binario propio no sustituye el ejecutable del producto.

## Reproducción exacta

```powershell
$env:CARGO_HOME='D:\Tools\R3Viewer\.tools\cargo'
$env:RUSTUP_HOME='D:\Tools\R3Viewer\.tools\rustup'
$env:PATH='D:\Tools\R3Viewer\.tools\cargo\bin;'+$env:PATH
$env:CARGO_TARGET_DIR='D:\Tools\R3Viewer\.tmp\rv106-spike\target-private'
cargo test --offline --manifest-path 'D:\Tools\R3Viewer\.tmp\rv106-spike\grammar-r4\Cargo.toml' -- --nocapture
cargo build --offline --manifest-path 'D:\Tools\R3Viewer\.tmp\rv106-spike\grammar-r4\Cargo.toml'
& 'D:\Tools\R3Viewer\.tmp\rv106-spike\target-private\debug\rv106-adversary-grammar-r4.exe'
git diff --no-index -- 'D:\Tools\R3Viewer\.tmp\rv106-spike\TOML-R3.sublime-syntax' 'D:\Tools\R3Viewer\.tmp\rv106-spike\TOML-R4.sublime-syntax'
```

El último comando devuelve 1 por diferencias esperadas. Los logs inicial/final son independientes; el final corresponde al SHA del source listado. Las dependencias estaban disponibles en el runtime privado y no se instalaron globalmente.

## Límites y recomendación

Estos resultados cierran los dos repros comunicados dentro del corpus y dejan el derivado listo para un ciclo adversario independiente. No acreditan todos los documentos válidos/ inválidos TOML 1.1, prestaciones bajo entradas extremas, macroscopía de estilos en UI, Linux/macOS ni contención de producción. No cambia el contrato recomendado antes: raw span/lexema son autoridad para números/fechas; native f64/datetime auxiliares; tabla implícita sin span contiguo; arena neutral de navegación y DOM transitorio del hijo cuentan juntos en el pico. El proceso parser cancelable con Job/lease/kill-wait sigue requiriendo diseño antes de producción.
