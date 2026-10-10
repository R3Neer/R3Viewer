# Dictamen S03: spike focal de error Onig/Syntect
SIN_CRITICAS_PENDIENTES en este control acotado de investigación. No es aceptación de un watchdog, cobertura universal, parser nativo completo ni cierre integral S03.

## Método e identidad
Paquete standalone privado offline con lock propio; Onig 6.5.3/onig_sys 69.9.3/Syntect 5.3.0, backend regex-onig. Sin editar Cargo/lock productivos, sin setters de retry/stack/globales.
Patrón (a|b|ab)*bc, input adverso de60 bytes literal del test upstream test_regex_error_find; opciones native idénticas al wrapper Syntect: REGEX_OPTION_CAPTURE_GROUP, Syntax::default(), búsqueda [0,len), SEARCH_OPTION_NONE, None región, MatchParam::default(). Límite original observado antes/después:10000000.
Oráculos OI01–05 fijados antes de compilación/output; fuentes de referencia físicas copiadas. Primer bin6776A8879572995C7B06EEB200FB722336F5C0E6557F19657966F10222E38DF6; manifest previo229B89A22B2A55A34E23F29ABB5AA07020321B9B33C4764A0DA2D5BB0BFF7C7A.
Segundo bin9B314A8CCC74A2F58AFB4733B78B707AD9DDF7759278A69A961374A86C1A07B3; manifest previo14AF48B71D1F47565D54B0A00CFF737CB20293B95B8FEAFE1A157982A4A20382.
Build propio offline/locked exit0, toolchain e inventario dependencias retenidos. Actor único original Diagnostics.Process, stdout/stderr asíncronos y wait físico; observador privado máximo30s sin nuevo límite de producto. No timeout/kill en los intentos ejecutados; sin procesos nuevos de rescate.

## Resultados propios
| Criterio | Evidencia segundo intento | Disposición |
|---|---|---|
| OI01 error retry original | Native Err -17 retry-limit-in-match over; SyntectRegex::search false sobre idéntico patrón/input/default | PASS |
| OI02 match sano | abc: Native Some(0), Syntect true | PASS con errata prospectiva del oráculo original |
| OI03 no match | xyz: Native None, Syntect false | PASS |
| OI04 compilación inválida | (: Native Err -117, Syntect try_compile Some(error) | PASS |
| OI05 ParseState | YAML regla adversa + fallback .: parse_line Ok,60 Push fallback,121 operaciones, ningún Push invalid.illegal.retry-rule | PASS focal, error retry no expuesto como GrammarError |

Segundo intento: proceso original PID1388 exit0, hasExited=true, wait original=true,1545ms; pre/postbin iguales. Este tiempo es observación de este caso, no SLA ni límite demostrado de todas las gramáticas.

## Fallo preservado y corrección
Primer intento: OI01 PASS, después panic por mi expectativa Some(1) para abc; proceso PID46968 exit101, wait real/no timeout,1339ms. La expresión puede consumir a antes de bc, por lo que el match izquierdo comienza en0. ERRATA-before-output.md corrige sólo esa expectativa y su print antes del segundo source/bin/output. Primeras fuentes/bin/logs/receipt siguen físicos íntegros; no relabel de FAIL ni defecto de producto.

## Lectura fuente y límites
Syntect parsing/regex.rs174–188 usa search_with_param(default) y matches!(matched, Ok(Some(_))): tanto Err como Ok(None) quedan false. Onig src/lib.rs conserva el caso adverso; onig_sys regint.h DEFAULT_RETRY_LIMIT_IN_MATCH10000000, ffi.rs código -17 y regexec.c inicializa cada MatchParam desde el límite global. El programa sólo lee ese límite; jamás lo modifica.
ParseState evidencia una gramática sintética concreta que continúa por fallback; no demuestra todas las vías de GrammarError, todas las gramáticas/capturas/inputs, límite agregado por línea/documento, cancelación dentro de Onig ni recuperación de sesión. Tampoco introduce watchdog o cuota adicional. No QA visual, Unix ni runtime de motores externos.
Cargo.toml/Cargo.lock productivos comparados pre/post sin cambio. No implementación productiva ni adopción automática; raíz decidirá la documentación del hallazgo acotado.
