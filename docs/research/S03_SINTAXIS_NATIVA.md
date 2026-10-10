# S03: aceptación nativa de alternancia, caché y contexto léxico

2026-10-10. Tres controles focales Windows con revisión de código y ejecución independiente sin críticas pendientes. No cambia el producto ni cierra todo S03. Código productivo `29993a0`; el HEAD `aa9d3c5` previo a esta entrega sólo añadía documentación.

## Método y candidato

El observador privado enlaza la biblioteca real y delega `--syntax-worker` al `run_private_stdio` original: `SyntaxClient`, parser, IPC, Job y límites no se sustituyen. Conserva startup2s, línea/IPC500ms, viewport2s, All5s, caché8MiB/256entradas,50000spans y32checkpoints. El horizonte externo60s sólo puede producir fallo; no es un plazo nuevo del producto.

Compilación release offline/locked en target privado, sin warmup ni repetición hasta verde. Los seis inputs comunes (observador, runner y cuatro fixtures) son idénticos al intento debug.325fuentes/Cargo/build/recursos locales fijados antes del build y cotejados después;13inputs de ejecución iguales antes/después.

- Ejecutable: `AB35C3244A9F8EBF2C8412CA6544C9CA63CA6FEDDDE2A1FF09200AC9F7A6E0EE`.
- Biblioteca: `2F06D03A39558F9D6CB71A4D31503F080B880330053D64894F3CF4DB2A558DD5`.
- Manifiesto antes de ejecución: `7EAA7723D319B1E1CD2D72AF06F61525A3362B99E1A502ABE80D877097D33736`.
- Manifiesto final del coordinador: `5234531DD24DDFC790F2E5B42454983ED3A8ADE6A9E7684664DD46BA74464813` (18entradas).

## Resultados

| Control | Evidencia | Límite de la conclusión |
|---|---|---|
| N01: dos Strings Legacy y reemplazos | Seis respuestas gen2–7 alternan dos Arc<String>, con generación/request/viewport actuales, offsets UTF-8 completos y roles del marcador esperado. Tras shutdown queda un dueño de cada fuente. | Intercalaciones ejercitadas; no todos los frames tardíos posibles. |
| N02: frontera material de caché |32viewports distintos de128líneas.33280spans/399360B mínimos por respuesta; acumulado12779520B. La caché se estabiliza en8386896B, bajo8388608B, y la primera vista solicitada de nuevo conserva exactamente sus spans. | Contabilidad del controlador; no heap total/RSS. Legacyhandoff399360B se registra por separado.32entradas no saturan el máximo256. |
| N03: retorno y contexto | Comentario multilineal Unicode, high8800→low300→high8800, roles Secondary conservados y regreso idéntico. | El regreso puede usar caché; los32checkpoints se cuentan en la prueba unitaria original, no se infieren de este runtime. |

Los tres cierres finales corresponden a generaciones7/10/20: ticketReaped, statusReaped sin fallo y shutdown.join correcto. Las respuestas2–7 de N01 no son seis tickets de cierre independientes.

Coordinador: proceso20332, exit0,13758ms, sin timeout del actor y espera física del mismo proceso. Adversario: copia exacta del candidato, una ejecución propia sin recompilar biblioteca; proceso33716, exit0,21941ms, sin timeout y con espera física. Stdout idéntico `63B4D93FB9F44A444BD39DD3B9D7EB8E7289DC4FAF15C23039C7136DAFA0ECFB`; stderr vacío y hash del ejecutable igual antes/después. Estos tiempos no son un SLA.

La [revisión final independiente](../reviews/S03-sintaxis-nativa-final.md) conserva expectativas anteriores, alcance y limitaciones. Raíz cotejó20/20artefactos independientes. Su manifiesto original `A699BAAEE351300EC13ED638E1CC56BE2D9883339F50F9AF4A63ACE6383251CE` permanece intacto; un suplemento corrige únicamente la descripción de respuestas frente a tickets, sin nueva ejecución.

## Fallo conservado y pendientes

El intento debug anterior falla en N01 con Plain(LineTimeout), generación2, exit1/1635ms; N02/N03 no llegaron a ejecutarse. CleanupReaped/joinOk. CodeAE no demuestra defecto del observador ni identifica la fase causal. Init→Ack incluye carga del pack bajo500ms, pero no se atribuye el fallo a esa fase sin una observación causal. El PASS release no explica ni borra el FAIL debug.

Se aceptan estos tres controles acotados. Siguen abiertos los demás criterios de obsolescencia, watchdog/IPC/fallos, cancelprepare, fronteras de estructuras, paleta Nu, QA visual y runtime Unix que no cubre esta entrega. Ningún incremento PDF/Markdown/MathJax se reanuda.

Artefactos privados: `.tmp/s03-syntax-native-2026-10-10` y `.tmp/s03-syntax-native-adversary-2026-10-10`; propuestas, fallos, binarios y recibos permanecen separados.