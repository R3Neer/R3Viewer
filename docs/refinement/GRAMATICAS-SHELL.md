# S03: propuesta de gramáticas shell R2

Responde SH-01/SH-02 de reviewR1 SHA0A03F51B9D28D83C6AB17F5659AED68AC0247513D66422D7F7A2C4032212ECDD. R2 sustituye la conversión sencilla de R1; investigación y diseño previo a prototype, sin build raíz, modificaciones producción, ejecución de scripts, QA visual ni aceptación de gramáticas. Fuentes públicas fijadas y MIT se mantienen; descargas privadas D:/Tools/R3Viewer/.tmp/syntax-shell-research. Inventario completo205regex y14capturas retokenizadas en S03-shell-grammar-inventory-r2.json SHA10A1E82294CD3FFC20D97EB7ADDAEB11083FA03DDA432D37BB7F112BA546C7CA, con JSON pointers, valores y hashes.

## Decisión y límites del perfil

Proponer dos gramáticas derivadas **manualmente con patches explícitos**, no un converter universal TextMate→Sublime. Nu oficial y EditorSyntax PS son procedencia y comparación; no afirmar equivalencia completa TextMate. El perfil conserva las construcciones de resaltado enumeradas por R1, incluida toda funcionalidad implicada en14capturas, y fija scopes por intervalos antes de implementar. Cada patch registra owner upstream, reglas emitidas y oracle. No se borran captures.patterns ni se reemplazan por scope genérico. Si un lowering no reproduce el contrato de intervalos y estado, se detiene adopción y se revisa estrategia; no se rebaja el oracle para obtener verde.

Producto continúa sólo Syntect5.3+recursos fijos, sin postprocesador runtime TextMate, overlays de capturas ni evaluación Nu/PS. Reescrituras cambian los owners que consumían grupos enteros a estados secuenciales; los tokens internos se procesan en contextos concretos sobre el mismo tramo. Las hipótesis siguientes requieren prueba: son diseño, no equivalencia acreditada. Prototype privado primero después de revisión.

## SH-01: inventario y lowering explícito

Pointers con prefijo /repository/. C significa captures, B beginCaptures. En cada fila el contexto originador mantiene sus scopes envolventes; espacios y delimitadores mantienen los stacks de su intervalo. Nunca usar include genérico si permite consumir fuera del intervalo original.

| ID | Pointer completo después del prefijo | Patch fijo candidato y scopes que debe conservar | Fixtures específicos |
|---|---|---|---|
| N01 | variables/captures/1 | Ordenar regla interna `$nu/$env/$in` antes de ordinaria; probar boundary del nombre, consumir cellpath en estado separado con variable.other. Captura principal interna variable.language, ordinaria variable.other. | `$env.PATH`, `$nu.version`, `$in`, `$input`, `$envx`, `$x."a b"`; nombre y cellpath separados. |
| N02 | use-module/patterns/2/captures/3 | Reescribir owner a estado import-path: keyword→ruta entrecomillada/bare→nombre importado. Mantener string.bare en ruta entera; namespace sólo tramo final seleccionado por regex retokenizada original, excluyendo delimitador y extensión donde corresponda. No simplificar a basename sin oracle. | `use ./foo.nu bar`, `use "dir/my mod.nu" thing`, comillas simples, ruta Windows, sufijo .nu y wildcard. |
| N03 | use-module/patterns/3/beginCaptures/3 | Mismo estado de ruta de N02, transición a lista import después de bracket; no perder scopes del comienzo ni importar comment al grupo anterior. | `use "dir/my mod.nu" [foo, bar]`, multilinea/lista con comentario y cierre. |
| N04 | for-loop/beginCaptures/4 | Abrir meta.for-loop en for; cabeza variable/in, estado iterable que incluye #value, seguido de apertura `{` y body. Terminal `{` debe corresponder a la captura5 del owner y no a record/string anidado. Reconocimiento previo de la cabeza completa evita comprometer región en falso positivo. | `for x in [1 2] { $x }`, iterable record/paren/string con `{`, expresión anidada y cabeza incompleta. |
| N05 | braced-expression/patterns/2/beginCaptures/2 | Owner record key interpolada doble se divide en entrada de región, apertura `$"`, interior con #paren-expression, cierre y colon. Variable.other cubre grupo1 y grupo2 con la profundidad de scopes que el oracle original exige; no deduplicar scopes repetidos. | `{ $"a(1 + 2)b": 3 }`, paréntesis anidado/string/escape dentro de paren, quotes y colon internos. |
| N06 | braced-expression/patterns/4/beginCaptures/2 | Igual N05 con `$'` y cierre simple; preservar límites originales y record-entry desde comienzo de key. | `{ $'a(1 + 2)b': 3 }`, EOF/interior anidado, key seguida de otra entry. |
| N07 | define-variable/captures/4 | Split declaración keyword/name/colon; estado type include #types hasta '=' de ese owner; types list/record anidados deben cerrarse antes de assignment. No consumir RHS como tipo. | `let x: int = 1`, `mut y: list<int> = []`, `const z: record<a:int> = {a:1}`, omitido/mal cerrado. |
| N08 | define-variable/captures/5 | '=' en transición type/head→RHS se resuelve con misma prioridad #operators; scope exacto de operator. Esta fila se prueba aparte de N07. | declaración sin tipo, con tipo, `==`/`=~` como falso assignment, RHS tras '='. |
| N09 | define-alias/captures/3 | Owner alias keyword/name y '=' direct operator; no #operators sobre resto de línea. | `alias foo = echo 1`, `export alias foo-bar = ls`, alias incompleto/operador doble. |
| N10 | pre-command/beginCaptures/2 | Cabeza env/name y '=' operator; valor en contexto acotado termina en primer whitespace permitido por owner; #value sólo interior. | `FOO=1 echo x`, `FOO="x" echo x`, separación y falso `FOO==1`. |
| N11 | command/beginCaptures/2 | Abrir meta.command con lookahead validando cabeza; dentro head ordenar control-keywords→ansi/char→builtin trie→external quoted→external bare→value como upstream. Cabeza termina donde begin original terminaría, no convertir args en nombres builtin. Estados de las tres capturas anidadas siguientes retokenizan args. | `if true`, `ansi red`, `str length`, `str lengthened`, `foo arg`, `^foo`, comando seguido de pipe/paren/semicolon; desempate control/builtin y multiword prefijo. |
| N12 | command/beginCaptures/2/patterns/2/captures/2 | Owner builtin se divide en nombre exacto del trie (keyword.other.builtin), espacio y args #value hasta boundary de captura2 de N11. Mantener nesting meta.command sin capa extra source.nushell. | `str length 42`, `echo $env.PATH`, valor anidado en command-head y argumento que parece otro builtin. |
| N13 | command/beginCaptures/2/patterns/3/captures/1 | Owner external quoted tras caret consume quote, interior #paren-expression, quote; entity.name.type.external cubre owner, retoken interno conserva paren scopes. `$` opcional/caret mantienen scopes correspondientes. | `^$"tool(1 + 2)"`, `^'tool'`, comillas con paren, falso cierre e interacción con args. |
| N14 | command/beginCaptures/2/patterns/4/captures/2 | Owner external bare se divide en head entity.name.type.external y args #value dentro de capture2 de N11; orden posterior a builtin/control. | `foo 42`, `foo $nu.version`, nombre con puntos/guiones y args anidados; fixture comparte N11/N14 como captura contenida. |

**Acotación verificable:** antes de sustituir owner, instrumentar tokenizer de referencia privado para obtener los intervalos upstream [a,b) de captura cuando compila. El lowering usa contextos específicos del owner y sus terminales; no puede conservar por sí solo un b arbitrario obtenido de una captura ya consumida. Por ello N04/N05/N06/N11–14 requieren prueba adversaria prioritaria: greediness, paréntesis en string y separadores internos pueden divergir. Si la regla fija no puede determinar el mismo límite, falla el gate de ese patch; no se finge que Syntect ejecuta captures.patterns. No introducir una retokenización runtime casera como sustitución automática sin nueva decisión arquitectónica.

## Oracle de intervalos, no de colores

Antes de producir recursos, AE distinta del autor fija fixtures y JSON: fixtureSHA, line, UTF8 byte start/end exclusivos, scopeStack ordenado completo y estado esperado al pasar delimitador/siguiente línea. Capturas0 y scopes superpuestos se incluyen: no deduplicar, no ordenar alfabéticamente. Stack base source.nushell exactamente una vez. Traducción explícita offsets UTF16 del tokenizer TextMate a UTF8 con muestras astrales, C1 literal y caracteres combinados; no usar columnas visuales para spans.

Cada N01–14 tiene al menos un positivo y un negativo/colisión; N05/N06/N12–14 además nesting y comienzo de región con captures simultáneos. Añadir strings/records que contengan delimitadores y compare estado después del owner. Diferencia documentada con upstream inválido se decide desde contrato manual del lenguaje antes de tests; no desde output candidato. Comparación upstream es fuente auxiliar, no oracle infalible. Fijar versión/commit/hash de vscode-textmate y vscode-oniguruma/WASM antes de ejecutar; no main móvil, plugins ni LS. El tokenizer sólo interpreta regex y archivos privados fijos, no scripts.

## SH-02: múltiples firmas y otras regex

La regla fija `(?<=]\s*)(:)\s+(\[)` no se copia. Proponer cambiar function-parameters para que al consumir su `]` pase mediante set a estado after-params (fin del meta.function.parameters antes de whitespace), dentro de función existente. after-params consume whitespace permitido y reconoce `(:)\s+(\[)` sin lookbehind: colon punctuation.definition.in-out y '[' meta.brace.square.begin. Entra signature-list, tipos/comma/arrow, `]` con scope de cierre; después vuelve a after-signature/body. La transición sólo se arma al cerrar parámetros de función, no cualquier ']' del documento. Firma simple `: int -> string` continúa soportada en su rama. Sintaxis anterior de function-body y extern no debe caer en firma múltiple accidentalmente. Cada transición con Pop único/set explícito; no pop2.

Fixtures: `def f []: [int -> string] {}`, uno/muchos espacios antes de colon, tabs, múltiples alternativas separadas por coma, espacios después de colon requeridos por upstream, firmas simples, ']:' dentro de string/list/record fuera de def, bracket incorrecto, LF/CRLF y cierre de parámetros al final de línea con colon al inicio siguiente. Cuando upstream no puede compilar lookbehind, AE manual fija scopes/política multilinea por contrato declarado; no usar su ausencia de tokens como expectativa ni eliminar firmas múltiples.

Inventario adicional detecta subrutina nombrada `\g<path>` en use-module/patterns2/3, con definición `(?<path>...){0}`. fancy-regex0.16.2 compile.rs Expr::SubroutineCall reporta FeatureNotYetSupported. Hipótesis: expandir el fragmento path fijo textualmente como grupo no capturante por AST, actualizar grupos/capturas según tabla de remapeo y preservar límites N02/N03. No ejecutar regex-replace ciego sobre '\g'; cada regex transformada queda en manifiesto fuente→emitida/hash→remapeo grupos→fixtures. También inventariar cada anchor \G, lookbehind, backref, regex begin/end/match en capturas; nuevas incompatibilidades paran preflight en vez de skip. PS \G commentEmbeddedDocs, applyEndPatternLast y comillas Unicode conservan matrices de R1.

## Fases y APIs exactas para el gate

A. Preflight offline de todos los205regex de origen y todos los regex emitidos, no sólo contexts visitados. Origen incompatible queda en ledger con error y patch, no aprobado. Sintaxis generada: SyntaxDefinition::load_from_str(...,true,None) para ambos backends en dos crates/procesos separados; features regex-fancy y regex-onig juntos eligen Onig, por lo que no sirven como dos mediciones. Mismo Syntect5.3 y lock/profile de dependencias. parse_regex resuelve variables, normaliza newlines y llama try_compile_regex (sustituye backrefs numéricos por placeholders); esto es eager de reglas presentes. Builder.build seguido find_unlinked_contexts vacío y validación de referencias/scopes duplicados. Parches nunca pueden ocultar una regla del inventario sin entrada de transformación.

B. Dump generado por fancy según build.rs actual, roundtrip con from_reader y ParseState::new/parse_line en Onig runtime y fancy de matriz Unix. Regex.new contiene OnceCell lazy; Regex.try_compile eager del loader con placeholders no demuestra compilación dinámica real. En raw-string end con backref1, ejecutar delimitadores capturados de0/1/varios hashes, falsos cierres parciales, UTF8, LF/CRLF y EOF; verificar regex sustituidas/escapadas y estado, dentro del worker real congelado. Dependiendo de visibilidad interna, instrumentación privada de Syntect para registrar patrón dinámico se identifica como tal; aprobación funcional del binario original se hace por sus spans/estado/reap, no por esa copia.

C. Ejecutar todas las filas/oracles de intervalos con ambos backends y comparación TextMate cuando viable. Señalar fallos de motor (incluye errores que fancy wrapper degrada a no-match) separadamente de loading; ausencia de panic o loadOK no es semántica aprobada. Probar viewport/checkpoints contra desde-principio, cancel/deadlines/fallback total y recolección con los límites de S03 intactos. Canary fuente/use/const permanece texto, monitor de worker owned para límites de ausencia I/O; no ejecución de ejemplos.

## Catálogo, licencias y cobertura restante

Se mantiene R1: defaults_newlines+TOML aprobado+Nushell/source.nushell[nu]+PowerShell/source.powershell[ps1,psm1,psd1], aliases por nombre/extensión y pwsh separado. Catálogo generado del mismo builder que dump; selector y worker comparten identidad, alias no inventa extensión .pwsh. rerun-if-changed explícito de recursos fijos, sin loadfolder/usergrammars. --format text plano, --language sólo sintaxis, sin inferencia metadata/shebang. MIT/copyright íntegros de fuentes/derivados/herramienta realmente adoptada.

Corpus44+10 de R1 se amplía para garantizar matriz N01–14 y cada patch/regex; no se limita número de fixtures para ahorrar aceptación. Gate separado paletaNu/QA visual en hosts, Windows funcional y Unix/native distinto. Sólo se autoriza prototipo tras revisión de este plan; adopción requiere candidato congelado, AE y adversario. Si reglas secuenciales no conservan intervalos soportados, siguiente estrategia conserva objetivo Nu/PS y requiere revisión nueva.

Fuentes primarias: [VS Code TextMate rule.ts](https://github.com/microsoft/vscode-textmate/blob/main/src/rule.ts) y [retokenización](https://github.com/microsoft/vscode-textmate/blob/main/src/grammar/tokenizeString.ts), consultados sólo para semántica; futuras ejecuciones fijadas. [Syntect5.3 loader](https://github.com/trishume/syntect/blob/v5.3.0/src/parsing/yaml_load.rs), regex.rs y fancy-regex0.16.2 compile.rs instalados exactos, no ejecución realizada en esta ronda. Artefactos oficiales y hashes siguientes iguales a R1.

## Artefactos consultados

### kurokirasama/nushell_sublime_syntax

Commit `8a1bb9205859d0c2f362c6c5a2b7ef1a7a87c387`; consulta 2026-10-05.

- `LICENSE` SHA256 `EFF6BCCEF897CF2753EB8FFDBD9EF46EC1A6F86FE2AE6CF0CA36C87E666B2573`
- `nushell.sublime-syntax` SHA256 `2BA54EF98F6A4E0A51BA9DE8A8BAC7CE0446F72D1A4B4E8ED5F3C1D5B355E385`

### nushell/vscode-nushell-lang

Commit `7d4a73285807e2b554ea0ebd839478ac83943306`; consulta 2026-10-05.

- `LICENSE` SHA256 `2DC1F03F729C21902D869B4D8F8DC528FC730F4C6AD83FC128672BDA0AD69196`
- `syntaxes/nushell.tmLanguage.json` SHA256 `017BEE0FC8A9822B12D1D9F194DB249A237F46FECBD434A16DB3BDB94D5AE2AC`

### PowerShell/EditorSyntax

Commit `23eed76d483c887a1608609e9dd57e09666411cc`; consulta 2026-10-05.

- `LICENSE` SHA256 `1DD685A63AF94DF709490FBBC874CDA8A6E98BDD957DA4AE0BF7B85E363748A7`
- `PowerShellSyntax.tmLanguage` SHA256 `7F4095D4CC0D33BAAE981AF206D4281DC7EB40EDAEACB6E9E888C9E721D37A0B`

### SublimeText/PowerShell

Commit `2938700b586aeaa9c1d2dc0c04a1cc569ff3c24e`; consulta 2026-10-05.

- `LICENSE.txt` SHA256 `4CBACDF136EBD84B5276F18801C054E3EABC29BC1C17206F5E4D9D57965D5742`
- `PowerShell.sublime-syntax` SHA256 `0B3A72AA262A3998429952162818E2F68C433BB3228680A21E8D066CC7BFDA62`
- `syntax/embed/C# (for PowerShell).sublime-syntax` SHA256 `7D4C31719143864CFE1231341DA68175349003463D43B79DBF3537F403DED5C9`

## Estado canónico

Diseño R2 revisado independientemente: [informe final](../reviews/S03-shell-gramaticas-r2.md), SIN_CRITICAS_PENDIENTES dentro del diseño. [Inventario exacto](../research/S03-shell-grammar-inventory-r2.json) conserva las 205 regex y 14 capturas de origen. Los SHA del informe identifican los documentos temporales originales antes de normalización Git; no acreditan ejecución del prototipo.

Prototipo privado iniciado, con oracles preparados por aceptación independiente antes de observar el candidato. Compilación, fidelidad de scopes/estado, worker, presupuestos y compatibilidad nativa conservan sus gates. S03 permanece activo; no adopción de recursos de producción ni cierre de RV por aprobación de diseño.
