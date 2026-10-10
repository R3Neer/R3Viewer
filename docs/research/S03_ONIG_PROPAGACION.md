# S03: error interno de Onig y propagación por Syntect

2026-10-10. Investigación focal completada, con [dictamen y resultados del agente](../reviews/S03-Onig-spike-final.md) y aceptación adicional ejecutada por el coordinador. No altera producción ni amplía las garantías de GrammarError.

## Resultado

Con Onig6.5.3/onig_sys69.9.3 y Syntect5.3.0, el patrón `(a|b|ab)*bc` y la entrada adversa de60 bytes del test upstream producen un error nativo de exceso de intentos (-17, retry-limit-in-match over). Sobre el mismo patrón, texto, opciones y MatchParam por defecto, SyntectRegex::search devuelve false. El wrapper convierte Err y Ok(None) en ese mismo resultado. ParseState continúa por una regla de fallback en la gramática sintética del control y devuelve Ok; ese error concreto no llega a GrammarError.

Se mantiene el límite original observado de10.000.000 intentos, sin setters globales ni parámetros de límite nuevos. Controles sanos: abc casa desde0 en ambos motores; xyz no casa; una expresión inválida sí expone error de compilación. Los cinco oráculos estaban fijados antes de outputs. La errata inicial del oráculo sano esperaba1 en vez de0; primer fallo y binario se conservan, corregidos prospectivamente, sin atribuirlos al producto.

## Identidad y ejecución independiente

Paquete privado standalone/offline, sin cambiar Cargo.toml/Cargo.lock productivos. Segundo bin9B314A8CCC74A2F58AFB4733B78B707AD9DDF7759278A69A961374A86C1A07B3; manifest previo14AF48B71D1F47565D54B0A00CFF737CB20293B95B8FEAFE1A157982A4A20382. Manifest final8FC6FBACAA79782CE14E4F5B0EF62631E5490E7C6F633C16DBFAC7E4AC9F7555 conserva42 artefactos físicos, incluidos ambos intentos; raíz cotejó42/42 y examinó fuente del segundo intento y actor.

Autor: cinco controles pasan, proceso original1388 exit0,1545ms y wait físico. Raíz copió binario/actor exactos a root-acceptance, los identificó antes de ejecutar y repitió los mismos cinco controles: proceso26180 exit0,1549ms, sin timeout/kill, hasExited y originalProcessWaited verdaderos, hash binario pre/post idéntico. Receipts/logs/inputs quedan bajo .tmp/s03-onig-adversary-2026-10-10. Revisión focal del coordinador: SIN_CRITICAS_PENDIENTES para este mecanismo y evidencia.

## Consecuencia para el visor

GrammarError cubre errores que las APIs utilizadas exponen. No promete detectar todos los errores internos de Onig; el fallback del parser puede ocultar éste. La preservación de fuente, los límites externos y el watchdog del visor mantienen sus contratos y validaciones pendientes. Estos tiempos pertenecen a este ensayo y no son una garantía universal de duración o memoria.

Queda cerrado el spike focal de retry-limit/propagación de la matriz S03. No cierra lifecycle adversarial/cache/IPC/timeouts del producto, QA visual, runtime Unix ni S03 completo; tampoco explica los fallos históricos Unicode/XML. Las fuentes primarias inspeccionadas son los crates exactos locales: syntect parsing/regex.rs174–188, onig src/lib.rs (test_regex_error_find) y onig_sys/oniguruma regint.h/regexec.c, con sus copias identificadas en el manifest del agente.
