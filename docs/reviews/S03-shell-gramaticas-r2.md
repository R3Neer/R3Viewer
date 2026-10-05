# S03 shell — revisión independiente de propuesta R2

2026-10-05. **SIN_CRITICAS_PENDIENTES** en el diseño experimental revisado. SH-01 y SH-02 cerrados en propuesta; no equivalencia ni resaltado runtime acreditados.

Propuesta SHA256 verificado `E7242991B3E2A10E56EE23C170AA6DFA72EA399C3FCA9543713EF221660D928C`, archivo `S03-shell-grammar-proposal-r2.md`. Inventario `S03-shell-grammar-inventory-r2.json` SHA256 verificado `10A1E82294CD3FFC20D97EB7ADDAEB11083FA03DDA432D37BB7F112BA546C7CA`. Revisión anterior `S03-shell-grammar-review-r1.md` SHA256 `0A03F51B9D28D83C6AB17F5659AED68AC0247513D66422D7F7A2C4032212ECDD`.

Sólo lectura y recorrido estático privado JSON/plist. Sin fuentes/Cargo producción, build raíz, ejecución de ejemplos/exportadores/módulos/LanguageServer, conversión implementada ni QA visual. Esta revisión es de un diseño de otro autor y no del spike gráfico anterior.

## Inventario comprobado independientemente

Se recorrieron los dos artefactos oficiales fijados, sin usar el inventario como generador de expectativas. Resultado:205 match/begin/end y14 capturas con patterns, incluidas capturas dentro de otras capturas. Se cotejaron source/JSON pointer/valor completos: coinciden las14 capturas y las205 regex, sin diferencias. Se recalculó SHA256 UTF8 de cada regex del inventario:205 hashes coincidentes. Las dos regex use-module con definición path/subrutina están incluidas; no son referencias externas a archivos o módulos.

Artefactos: Nu SHA256 `017BEE0FC8A9822B12D1D9F194DB249A237F46FECBD434A16DB3BDB94D5AE2AC`; PowerShell SHA256 `7F4095D4CC0D33BAAE981AF206D4281DC7EB40EDAEACB6E9E888C9E721D37A0B`, conservados en `.tmp/syntax-shell-research`. La comprobación del inventario acredita exhaustividad estática para esos recursos, no corrección de su conversión.

## SH-01 cerrado en diseño

R2 abandona el supuesto de converter universal y el mapeo captures→scope que perdía retokenización. N01–N14 identifican cada owner, hipótesis de lowering secuencial, intervalos/scopes que deben conservar y fixtures de colisión/nesting. Mantiene funcionalidad implicada por las capturas, no omite patterns ni los sustituye silenciosamente por una categoría genérica.

El límite difícil está reconocido: un contexto nuevo no recupera por sí solo el extremo b de una captura ya consumida. N04/N05/N06/N11–14 tienen prueba adversaria prioritaria de greediness, strings/delimitadores, head/args y capturas simultáneas. Si no conserva intervalos/estado, falla el patch y se revisa estrategia; no se modifica el oracle para aprobar. Ese stop es una condición de adopción del experimento, no abandono del soporte Nu final.

Oracle AE anterior al recurso generado: intervalos UTF8 exclusivos, stack ordenado íntegro, capturas0/superpuestas y estado después de delimitador/siguiente línea; no deduplicación ni colores como sustituto de scopes. TextMate UTF16→UTF8 tiene muestras astrales/controles/combinaciones y versión/hash fijados antes de ejecutar. La comparación upstream es auxiliar cuando una regex upstream no compila; los resultados candidato no generan expectativas.

**SH-01 cerrado para proceder a prototipo privado.** Las hipótesis específicas siguen por demostrar y no se declara igualdad TextMate completa.

## SH-02 cerrado en diseño

R2 identifica y no copia lookbehind variable de function-multiple-inout. after-params se arma sólo al cerrar parámetros de def; propone consumir whitespace y colon/bracket mediante transición explícita, conservando scopes, firma simple y separación de extern/body. Corpus incluye espacios0/1/muchos, tabs, firma múltiple/simple, bracket fuera de def, incorrectos y política LF/CRLF/multilinea decidida manualmente antes de implementación. No elimina firmas múltiples para cargar.

La incompatibilidad adicional de `\g<path>` queda registrada y vinculada a N02/N03. Expandir el fragmento fijo no recursivo por AST y remapear grupos/capturas explícitamente es una hipótesis viable de investigación. Su aceptación exige manifiesto regex fuente→emitida/hash/remapeo y comparación de límites/scopes; no se aplica reemplazo textual ciego a subrutinas arbitrarias. El fragmento path original no contiene recursión de subrutina, por lo que esta expansión candidata no promete resolver el caso general recursivo.

Preflight abarca205 regex origen y todas las emitidas, con errores upstream clasificados y patches trazados. Corrige la distinción eager load_from_str/try_compile con placeholders frente a Regex lazy y backrefs runtime. Dos crates/procesos para fancy y Onig evitan feature unification que sólo ensayaría Onig. Dump generado por fancy se prueba en Onig original y fancy; instrumentación privada se identifica y no reemplaza aceptación del binario real. Raw delimitadores, PS \G/applyEndPatternLast y errores degradados a no-match conservan gate funcional independiente.

**SH-02 cerrado en propuesta.** Ni preflight ni ausencia de panic representan evidencia semántica suficiente.

## Coherencia restante

No se encontraron nuevas críticas accionables en este nivel de diseño. Mantiene recursos fijos/offline, licencias MIT/orígenes/copyright de derivados y herramientas adoptadas, catálogo/dump del mismo set, aliases reales separados de extensiones, defaults+TOML y --format text plano. No añade evaluación ni carga de grammars/modulos usuario.

No introduces postprocesador runtime TextMate/overlay como solución automática. La derivación manual y cambios de owners requerirán revisión del candidato completo, incluidas prioridades, scopes envolventes, delimitadores, prototype/selfrefs y no duplicar source.nushell. Estos casos pertenecen al oracle declarado, no se presuponen resueltos por esta revisión.

El corpus44+10 deja de ser cap y se amplía según matriz N01–14 y toda transformación de regex. Viewport/checkpoints equivalentes a parse desde principio, cancel/deadlines/reap, fallback plano de TODO el documento y canary de ausencia de I/O siguen bajo límites S03 vigentes. Paleta Nu/visual, Windows funcional y Unix nativo son gates separados. Compilar/cargar YAML o producir ANSI no cierra ninguno por sí solo.

Fuentes primarias cotejadas en esta revisión iterativa: [Syntect5.3 loader](https://github.com/trishume/syntect/blob/v5.3.0/src/parsing/yaml_load.rs), [VS Code TextMate rule.ts](https://github.com/microsoft/vscode-textmate/blob/main/src/rule.ts), [retokenización TextMate](https://github.com/microsoft/vscode-textmate/blob/main/src/grammar/tokenizeString.ts), [Sublime syntax](https://www.sublimetext.com/docs/syntax.html), [fancy-regex0.16.2 fuente](https://docs.rs/crate/fancy-regex/0.16.2/source/src/compile.rs). Fuentes main sólo explican semántica; R2 exige fijar artefactos ejecutables antes de oracle. No hubo tokenizer/regex runtime ejecutado por este revisor.

**SIN_CRITICAS_PENDIENTES** exclusivamente para el diseño del siguiente prototipo. SH-01/02 cerrados documentalmente; adopción de gramáticas, equivalencia del perfil, integración, QA/Unix y backlog permanecen pendientes de evidencia real independiente.
