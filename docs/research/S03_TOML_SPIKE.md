# RV-106: spike TOML y gramática original R1

Fecha: 2026-10-05. **EVIDENCIA_TEMPORAL; GRAMÁTICA ORIGINAL INSUFICIENTE PARA TOML1.1**. No cierra RV-105/106, aceptación de producto ni QA multiplataforma. No hubo cambios en producción o C. Harness privado Windows nativo: `D:\Tools\R3Viewer\.tmp\rv106-spike`, target `target-private`, binario exclusivo `rv106-adversary-toml-spike.exe`.

## Resultado implementable

`toml_edit::Document<Shared(Arc<String>)>` permite mantener la fuente sin copia de texto y preservar lexemas/spans; `Shared` implementa `AsRef<str>`. Es candidato viable en worker propio. **Lexema original es autoridad para números y fechas**; campos `f64`/`Datetime` nativos son auxiliares de clasificación y validación, porque redondean/truncan. Jerarquía por primera declaración; índice ligero de declaraciones/hojas con spans para orden físico. Tabla implícita no tiene necesariamente un rango contiguo real.

Recomendación mínima para el plan: worker mantiene DOM nativo durante parsing/construcción y entrega arena de navegación de archivos; padre conserva solo arena y Arc fuente, sin DOM duplicado y sin convertir Nu. Contabilizar pico DOM+arena+serialización dentro del hijo y modelo/recepción en padre. No publicar estructura hasta validación final, liberar DOM temporal y terminar/recolectar worker al acabar. El diseño de Windows Job, lease, deadlines, kill/wait/join e IPC acotado se revisa antes de producción. Este spike demuestra cancelación del proceso propio, **no** esas otras propiedades aún ausentes.

La gramática original fijada desde bat funciona para corpus TOML básico, pero asigna `invalid.illegal` a construcciones válidas TOML1.1. **No reducir versión ni retirar esas construcciones del alcance**: hace falta patch temporal auditado o otra gramática demostrada. El coordinador ha solicitado patch mínimo separado después de este cierre original; sus resultados no pertenecen a R1.

## Dependencias y procedencia exactas

Toolchain privado del proyecto: cargo1.99.0 (`5f94df478`, 2026-08-27), rustup/cargo home bajo `.tools`, sin instalación global. Resolución del harness:

```toml
toml_edit = { version = "=0.25.15", default-features = false, features = ["parse"] }
syntect = { version = "=5.3.0", default-features = false,
            features = ["default-syntaxes", "regex-onig", "yaml-load"] }
```

Cargo.lock resuelve `toml_edit0.25.15+spec-1.1.0`, `toml_parser1.1.3+spec-1.1.0`, `toml_datetime1.1.1+spec-1.1.0`, Syntect5.3.0, Onig6.5.3/onig_sys69.9.3. `unbounded` desactivado. `toml_edit` usa lexer tokens→events→DOM internamente, no callback cancelable público en Document::parse. Su constante interna de profundidad es80; no equivale al presupuesto propuesto64 del modelo.

Pin obtenido por GET público de [API bat v0.26.1](https://api.github.com/repos/sharkdp/bat/contents/assets/syntaxes/02_Extra/TOML?ref=v0.26.1), respuesta guardada `bat-submodule.json`:

- Submódulo `jasonwilliams/sublime_toml_highlighting` commit **f5a57e8bff694a4e6c52a491dae579aabc7427cf**.
- [Gramática del snapshot](https://github.com/jasonwilliams/sublime_toml_highlighting/blob/f5a57e8bff694a4e6c52a491dae579aabc7427cf/TOML.sublime-syntax), `scope: source.toml`, sin `version:2` en encabezado. Descargada sin modificar.
- [LICENSE del snapshot](https://github.com/jasonwilliams/sublime_toml_highlighting/blob/f5a57e8bff694a4e6c52a491dae579aabc7427cf/LICENSE): MIT, Copyright(c)2021 Jason Williams. Conservar aviso al distribuir recurso/derivado.
- Descargado `syntax_test_toml.toml` del mismo snapshot como material de referencia; no presentar sus anotaciones Sublime como un runner propio ejecutado.

| Artefacto | SHA256 |
|---|---|
| TOML.sublime-syntax ORIGINAL | `581328AEA6C0028AB613FF26B3114C4AE37F9FA29F7BE1042419BA353A6EABC6` |
| LICENSE | `87E6D5F3C4B540CBF3AE86BAFCE242A3CAD366EB16799A123CDE73B5FF5211F3` |
| src/main.rs del harness | `A4ACE48281DB7776532D705858BB0564D97ABFDC494222880A423A7DCA845703` |
| Cargo.toml | `42C648B6755C55C6D5A40FF58F0D2B4ED02D904EF23E54AE2632BB5476D67E54` |
| Cargo.lock | `2D77BE290541202A30EC959FA2CCF03D2756C69020DE4B8BC095798409F1CBD4` |
| binario debug privado | `27B2E941CAADD575DEFE5BB2F7EAF38495B6628127DE67BA8FF5FFF41E9D9F36` |

Fuentes primarias consultadas y ahora contrastadas con fuentes descargadas: [Document source](https://docs.rs/toml_edit/latest/src/toml_edit/document.rs.html), [toml_edit](https://docs.rs/toml_edit/latest/toml_edit/), [Syntect5.3 loader](https://raw.githubusercontent.com/trishume/syntect/v5.3.0/src/parsing/yaml_load.rs), [TOML1.1](https://toml.io/en/v1.1.0). Resultado native observado aquí prevalece sobre inferencias preliminares del refinamiento.

## Parser: evidencia runtime

6 pruebas pasan en ejecución serial, 0 fallos, 0.06s. Logs `tests-final.log` y `parser-and-scopes-final.log`:

| Comprobación | Observación exacta |
|---|---|
| Fuente compartida | `same_raw_pointer=true`, `strong_count=2`, fuente382bytes. Document usa la misma asignación, conserva final/raw original. |
| Offset datetime | `1979-05-27T00:32:00.1234567890123456789-07:00`, span6..51 completo, offset−420min. Campo nanosecond nativo=123456789. |
| Local datetime | `1979-05-27T00:32:00.999999999999`, date+time, offset=None; lexema completo, nanosecond999999999. |
| Local date / time | `1979-05-27` conserva solo date; `07:32:00.123456789012` solo time, ambos offset=None. Fracción raw completa. |
| Decimal | `1.2345678901234567890123456789` raw completo; native f64=`1.2345678901234567`. No usar Display(native) como valor autoritativo. |
| Otros números | `-0.0` native bits8000000000000000; raw `0xDE_AD`, `-inf`, `+nan` retenidos. Extremos i64 aceptados, ambos lados fuerai64 rechazados como overflow. |
| Dotted intercalados | `a.x=1; b.y=2; a.z=3`: DFS lógico `a.x,a.z,b.y`; hojas ordenadas por span `a.x,b.y,a.z`. Ese segundo orden es índice físico, **no jerarquía root aplanada**. |
| Tabla implícita | Fuente con espacios: `a` key/table span0..1, `b`8..9; esos spans apuntan al primer segmento de clave, no abarcan toda la tabla dispersa. |
| Unicode/CRLF | quoted key `"a.b"`, valor `'界'` y multiline CRLF con spans UTF-8 correctos; cooked string conserva CRLF. Fuente sin newline final exacta. |
| Profundidad | Arrays63,64,65 aceptados;127,128,129,256 rechazados `cannot recurse further; max recursion depth met`. No exploró frontera exacta80 ni demuestra validación del presupuesto64. |
| Errores | Offset+25:00, clave repetida y tabla `[a]` repetida rechazados. Recuperación mantiene fuente; no duplicado “primero/último gana”. |

Una prueba inicial tenía expectativa LF para cooked multiline CRLF y falló; se corrigió a CRLF observado. La norma permite normalización de newlines; no fue defecto de producción ni razón para normalizar fuente. `tests.log` conserva el intento inicial; `tests-final.log` identifica resultado final. Un ajuste inicial de API cambió `nanosecond` esperado a `Some(...)` y nombre de scope a `basic`; compilación posterior limpia. Son correcciones del harness, no cambios de parser/gramática.

## Gramática original: scopes comprobados y frontera1.1

`SyntaxDefinition::load_from_str(grammar,true,None)`, `SyntaxSetBuilder::build()` y `ParseState::parse_line` con ScopeStack real. Corpus comprueba scopes de **cada byte** de cuatro lexemas de fecha y cuerpo de multiline básicas/literales, más retorno a `constant.language.toml` para bool posterior. Pasan. Fechas llevan `constant.other.datetime.toml`; `hello 界\nworld` conserva `string.quoted.triple.basic.block.toml`; `literal # not comment` conserva `string.quoted.triple.literal.block.toml`.

No basta cargar gramática. Corpus adicional TOML1.1 **parser acepta completo** (`TOML11 ok=true`) produce estos scopes incorrectos en gramática original:

| Fuente válida1.1 | Scope original observado |
|---|---|
| `short = 07:32` | `07` invalid.illegal.value; `:32` invalid.illegal.trailing |
| `shortdt = 1979-05-27T07:32Z` | fecha inicial datetime; sufijo T07:32Z invalid.illegal.trailing |
| `escaped = "\e\x41"` | ambos escapes invalid.illegal.string.escape |
| `inline = { x = 1,` newline `y = 2, }` | segunda línea entera invalid.illegal.trailing |

Esta incompatibilidad es runtime exacto de un **recurso candidato**, no defecto del worker C congelado ni motivo para declarar RV-105/106 cerrado. TOML1.1 no se recorta. Patch posterior debe conservar origen/hash original y añadir corpus para optional seconds/fracción, escapes e/x, inline tables multilínea/comentarios/trailing comma y retorno a contexto.

## Cancelación y memoria observadas

Fuente sintética32MiB: un multiline literal largo válido. `--monolithic` mantiene atomic flag externa, hilo la activa a10ms. Antes de que parser retorne ya se lee flag=true; `Document::parse` termina a**1348ms** con éxito. No tiene hook para esa flag. Esto demuestra que revisar bandera antes/después no garantiza cancelar en100ms.

Allocator System contado en harness (bytes pedidos vivos, **no** RSS/commit/virtual ni cap): baseline33.554.628bytes, vivo al retorno67.111.170, pico268.437.050bytes. Incluye fuente, strings decodificados y temporales del parser; no hay arena todavía. No extrapolar linealmente ni prometer estructura128MiB bajo512MiB. Una prueba inicial reservaba fuente con capacidad insuficiente por un byte y duplicaba su capacidad al crecer; se corrigió `bytes+16` antes de medición final. `monolithic.log` histórico no representa la capacidad final; usar `monolithic-final.log`.

`--process-cancel` lanza SOLO binario privado propio `--parse-worker`, fuente32MiB generada antes de `PARSE_BEGIN`; parent recibe marcador, espera25ms, verifica hijo vivo, kill+wait+join lector: **3ms**, PID25396, salida1 por terminación. Ownership RAII mata/recolecta hijo aun ante errores. Comprobación posterior por nombre del binario no devolvió procesos. No hay tarea detach en esta prueba.

Esta prueba **no configura Windows Job ni lease de parent**, no comprueba crashdelparent ni startup de producto, no ejecuta Unix y no acredita contención. Antes de producción: proceso propio con Job cap y KILL_ON_JOB_CLOSE asignados antes Init, lease temprano, deadlines de bootstrap/total, IPC limitado, salida/cancelación y kill/wait/join propios. Presupuestos parser+DOM+arena+source calculados y medidos; padre no conserva DOM más arena. No sumar otros workers sin declarar total sesión. Modelo64MiB no limita por sí solo temporales monolíticos.

## Reproducción exacta

Ejecutar en PowerShell dentro de `D:\Tools\R3Viewer`; escrituras solo harness/target privados. Dependencias ya fijadas por Cargo.lock:

```powershell
$env:CARGO_HOME='D:\Tools\R3Viewer\.tools\cargo'
$env:RUSTUP_HOME='D:\Tools\R3Viewer\.tools\rustup'
$env:CARGO_TARGET_DIR='D:\Tools\R3Viewer\.tmp\rv106-spike\target-private'
$env:PATH='D:\Tools\R3Viewer\.tools\cargo\bin;'+$env:PATH
$spikeManifest='D:\Tools\R3Viewer\.tmp\rv106-spike\Cargo.toml'
& D:/Tools/R3Viewer/.tools/cargo/bin/cargo.exe test --locked --manifest-path $spikeManifest -- --test-threads=1
& D:/Tools/R3Viewer/.tools/cargo/bin/cargo.exe build --locked --manifest-path $spikeManifest
$spikeBin='D:\Tools\R3Viewer\.tmp\rv106-spike\target-private\debug\rv106-adversary-toml-spike.exe'
& $spikeBin
& $spikeBin --monolithic
& $spikeBin --process-cancel
```

`--locked` se añade como reproducción fijada; invocaciones observadas de test/build omitieron flag tras fetch y usaron el Cargo.lock indicado, sin cambios posteriores de dependencia. SHA binario corresponde debug, no release ni binario histórico/producto. Logs finales completos, fuente y manifiestos permanecen en harness. Copia de fuente para lectura: [rv106-spike-main.rs](C:/Users/Usuario/.codex/visualizations/2026/10/04/01a10823-9b5a-78d3-8c6a-c64ff5709d5f/rv106-spike-main.rs).

## Frontera de entrega

Windows runtime del harness acreditado para los escenarios enumerados; sin TUI/inline visual, sin núcleo/UI de producto, sin bundle integrado ni hardcap Job/lease, sin Linux/macOS. No se modificó `src/syntax.rs`9B9C8B06…, `src/syntax_worker.rs`7DF9764D… o `src/syntax_process.rs`6DC30905…; hashes leídos al final coinciden con el estado conocido antes de este spike. La arena padre frente a DOM worker, identidad de aristas, índices físicos y límites requieren el siguiente plan adversario. R1 permanece cerrado como **evidencia del original con incompatibilidad1.1 confirmada**, no aprobación de integrar la gramática tal cual.
