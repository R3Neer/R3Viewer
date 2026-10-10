# S03: cancelación real durante preparación

2026-10-10. Evidencia focal Windows de las cinco familias previstas, revisada sin críticas pendientes. Se acepta la comprobación; los hooks de observación permanecen privados y no se incorporan al producto. No cierra S03, QA visual ni Unix.

## Qué se comprueba

Ctrl-C real durante la proyección y el índice, antes de entrar en el visor o intentar TerminalGuard; salida130 original, propietarios recogidos físicamente y segunda apertura funcional en la misma ConPTY. Los controles temprano, tardío y de nonce inválido comprueban que no se atribuya cancelación de preparación a una fase que no ocurrió.

La instrumentación cfg-only observa el mismo store AtomicBool Relaxed del callback y los checkpoints originales; no pausa trabajo ni introduce barreras, ACK, relojes o cuotas nuevos. Un record compartido fijo512B/align8 registra slots independientes y secuencias coherentes. Los datos de progreso se publican después del trabajo observado. Se valida nonce/layout/selector; la identidad completa de las fuentes pertenece al freeze, sin prometer validación adversarial universal del hash por el probe.

El código de fase identifica tipo y estado, no generaciones de documentos: P Active65794→Cancelled65795; I Active131586→Cancelled131587. Callback/checkpoint observan el mismo identificador activo y el estado final pasa a Cancelled. No se afirma igualdad numérica de estados distintos.

## Resultado y autoría

| Familia | Resultado | Evidencia |
|---|---|---|
| P: proyección | Cancelación causal130, sin Viewer/Guard; reap y Tinyreal/q0 en la misma consola | Autor observación adicional5.429s y replay independiente4.892s |
| I: índice | Cancelación causal130→Tiny0; sin Viewer/Guard, propietarios cerrados | Autor3.531s, recibos cotejados por adversario |
| Temprana P | Callback/checkpointfase0, causalfalse,130→Tiny0 | Autor0.843s, recibos cotejados |
| Tardía I | PreparaciónComplete y frame interactivo; causalfalse, Ctrl-C130→Tiny0 | Autor13.549s, recibos cotejados; no se cuenta como callback durante prepare |
| Nonce incorrecto | Rechazo de setup del probe76 antes deSource, causalfalse→Tiny0 | Autor0.855s, recibos cotejados;76 es del observador, no código público |
| Normalcfg | Frame normal/q0→Tiny/q0; hooks excluidos | Autor4.039s, fuente y exclusión binaria revisadas |

El adversario reproduce P sobre una copia exacta: app41056, callback/checkActive65794, estado final65795, ETX/signal/done1, exit130, Viewer/Guard0. Ambas invocaciones esperadas, threads unidos, handles/probe cerrados y JobActive0; sin force ni Pending aceptado. Máscaras503/7 y tamaño/visibilidad del cursor restituidos. Los prints de diagnóstico desplazan el cursor de Tiny: no se anuncia posición universalmente invariante.

Mapping y callback tienen vida de proceso; la espera física del proceso acredita su finalización. No se inventa unregister/join de ctrlc ni se confunde Drop con cierre kernel. El inventario distingue Work72 instrumentado frente normal64, mapping/Arc, scopes y record: coste del observador, no cuota del producto ni RSS.

## Identidades y revisión

- App privada: `5764A475DD225F2959B569F0528DDC18CF26930BD34C346D8A85D8AC5AA62E78`.
- DLL: `AFBE165EC59A5630A06B52DE62C4E1F6433144FA8337B10B113E0C4EB69F6F61`.
- FreezeR1: `0A55EED8D5F4289B81FF7CCD5085C540C694E17176B30BE83577058A07DD667B` (1043entradas).
- ObservaciónR2: `D7A7B2D18841FA38C6DE2BDB24AD4C150D89B413987120288D2E417072831659` (27entradas).
- OwnP: `3A704929D06825FC3CFA769ABDB793F737D52C78B4A437E1963DF396D61A317B` (29entradas).

La [revisión final](../reviews/S03-cancelprepare-final.md) coteja1070identidades, seis diffs completos, protocolo/lifetimes y exclusión normal. Raíz verifica1043+27inputs,29artefactos de replay y los tres artefactos del informe. Cambios privados sobre build/lib/main/presentation/session/text_engine sólo son hooks cfg y check-cfg; fuente canónica relevante inalterada. El timeoutXML integrado posterior afecta otras rutas.

## Fallo preservado y alcance

PfinalR1 agotó el horizonte del actor30s después de OWN-START2, sin frame Tiny ni recibo completo; cleanup-only no se acepta como éxito. La observación adicional incorpora nueve registros acotados, sin acciones host, cuotas o condición de éxito nuevas. No reproduce un intervalo bloqueado; causa histórica desconocida. No se declara una corrección productiva ni se borra aquel FAIL por los PASS posteriores.

La aceptación original de estas cinco familias no exige duplicar cada ejecución del autor ni todas las intercalaciones. Los recibos del autor y el replay propio se distinguen. No garantiza ABI de otras arquitecturas, rendimiento histórico3695s, todas las carreras/descendientes ni apariencia visual.

Artefactos privados: `.tmp/s03-cancelprepare-execution-2026-10-10` y `.tmp/s03-closure-adversary-2026-10-10/cp-*`; mantienen intentos fallidos, fuentes, binarios, observaciones y recibos.