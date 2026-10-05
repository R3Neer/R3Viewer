# RV106 — revisión adversaria de delta JSON R1

2026-10-05. Solo propuesta del siguiente corte, sin producción ni pruebas ejecutadas. SHA verificado4F67B85337784F9ACCE4AB3881806CE9A823016DB8632359675DE048DB2DF8C7.

**SIN_CRITICAS_PENDIENTES** dentro del alcance de refinamiento R1. No se aprueba todavía Jiter para producción ni candidato implementado; TOML D1 permanece en integración y RV106 sigue abierto.

Leída completa propuesta y contraste con REQUISITOS.md201, contratos de origen/no universalJSON y D1. Fuente/Estructura, flags, Tab/o, estados separados, fallback local0, cancel130, publicación/límites/ownership reutilizan un contrato existente. Auto/Code solo extension local real; explícitoJSON para literal/textualNu y datos tipados mantienen su semántica. No introducir metadata reopening ni otro transporte/framework.

## Semántica revisada

Orden documental más conservación de todas las ocurrencias y ordinales evita first/last-wins. Igualdad por clave decodificada exacta distingue Unicode no normalizado y detecta a/escapea. Tabla solo con claves únicas por objeto evita que columna por etiqueta oculte valores repetidos; objeto repetido mantiene inspección de entradas. Warning acotado no invalida datos y scratch exacto debe presupuestarse, no hashsets externos ilimitados.

JsonNumber con lexema autoritativo evita el rango/roundtrip de Integer/FloatTOML: números fuera f64/i64, -0 y notación decimal distinta quedan inspeccionables sin aritmética prometida. Null clase propia no se confunde con ausencia, vacío o NuNothing recibido. Root escalar y EOF obligatorio, con las extensiones enumeradas rechazadas, son coherentes.

Raw spans incluyen delimitadores/escapes y no se mezclan con cooked search. JSONMember opcional explícito o índice de declaraciones vacío documentado evita fabricar Assignment/rutas dotted de TOML; la clave a.b permanece indivisible.

Surrogate aislado: degradación total explícita por representaciónUnicode, fuente íntegra y ningún reemplazo silencioso es compatible con la política de preservación/fallback. No promete cookedUTF16 completo ni declara el caso error sintáctico. Pares válidos se decodifican a scalar. La clasificación exacta del diagnóstico debe probarse contra los errores reales del parser; no inferir gramática inválida de una limitación de StringRust.

Fuentes primarias actuales: [RFC8259](https://www.rfc-editor.org/info/rfc8259/), secciones2/4/6/8.2/8.3/9, distingue objetos semánticamente no ordenados, riesgos de nombres repetidos, números de magnitud alta e incluso surrogates aislados admitidos por ABNF; permite límites de contenido/representación. Orden visual y preservación de ocurrencias son decisiones R3Viewer, no una exigencia de semántica ordenada JSON.

## Parser y gates

[Documentación Jiter actual0.17.0](https://docs.rs/jiter/latest/jiter/struct.Jiter.html) confirma superficie pull: números raw, strings, avance de objeto/array, posición y finish; next_skip advierte falta de comprobación UTF8. El documento R1 evita explícitamente usar skip como oráculoUnicode y un DOMmap como adaptador. La consulta por URL versionada devolvió error del sitio; latest identifica0.17.0. No se descargó ni ejecutó Jiter en esta revisión.

El gate previo exige fijar fuente/dependencias y comprobar finish/trailing, número raw sin overflow, rangos antes/después de claves/strings escapados, repetidos conservados, Unicode/EOF y tokens grandes en proceso propio. Tiene alternativa concreta visitor/raw sin Value-map con igual corpus, por tanto la viabilidad no queda fundada en API documental sola. J01–09 cubre casos de producto específicos y mantiene regresión TOML/Nu, aislamiento y QA separados. Detalle de IPC y nuevos discriminantes se versionará al implementar, con validación/capacidades de D1; aprobar este delta no acepta esas modificaciones ausentes.

No se detectaron críticas accionables bloqueantes o no bloqueantes que requieran modificar el contrato propuesto. El siguiente gate es spike independiente de parser y después propuesta técnica/candidato exacto; no cerrar RV106 ni atribuir pruebas realizadas a esta lectura.

### Adenda de fuente exacta del candidato

Tras el informe inicial, coordinador aportó crate exacto descargado. Verificado por lectura en .tmp/json-research/jiter-0.17.0/src/jiter.rs182–185: known_number_bytes usa consume_number<NumberRange>; number_decoder.rs557–558 devuelve NumberOutOfRange si el tramo entero supera4300dígitos. Es un límite real del candidato incluso con API raw, no conversión f64. No ejecutado aquí y no atribuido al producto actual.

Por tanto Jiter tal cual no debe aceptarse para producción con el contrato de lexema sin ese límite. Incluir4300/4301dígitos, signo y frontera entero/fracción/exponente en el spike; elegir adaptación explícita o alternativa que cumpla la misma aceptación. No recortar el requisito ni presentar NumberOutOfRange como número JSON sintácticamente inválido. Esto no reabre el diseño R1: su gate ya exige aceptación de lexemas grandes y alternativa sin rebajar precisión; hace concreto un riesgo conocido. El estado SIN_CRITICAS_PENDIENTES es únicamente del refinamiento, nunca aprobación del parser candidato.
