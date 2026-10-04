# Spike Nu 0.115.1: tipos, ausencia y cancelación

Fecha: 2026-10-04. Evidencia de ejecución Windows, no integración completa ni QA visual. Spike sintético ignorado en .tmp/nu-spike; sin registro de plugin ni perfiles. No copiar corpus privado.

## Ejecución comprobada

Nu instalado0.115.1 Windows MSVC; companion temporal compilado con nu-plugin/nu-protocol exact0.115.1 y Rust1.99.0. ServePlugin MsgPackSerializer y PluginCommand clasifican PipelineData sin convertir/ejecutar valor. EngineInterface.get_current_dir devuelve D:\Tools\R3Viewer.

Nu -n --plugins '[D:/Tools/R3Viewer/.tmp/nu-spike/target/debug/nu_plugin_r3viewer_spike.exe]' carga solo para la prueba. Resultado explícito recogido como lista:

| Entrada | Tipo recibido |
|---|---|
| Sin pipeline | Empty/absent |
| null | Value/nothing |
| cadena vacía | Value/string |
| lista vacía | Value/list<any> |
| 1.. | Value/range, sin enumerarlo |
| 1kb | Value/filesize |
| 1sec | Value/duration |
| Módulo passthrough directo sin entrada | Empty/absent |
| Módulo con let input = $in sin entrada | Value/nothing: pierde ausencia |

Esto confirma elección de PluginCommand y necesidad de no capturar $in universalmente. No acredita decoder propio, transporte de valores completos, límites ni UI tipada.

## Drop antes de consumir

Responsable specs ejecutó mismo spike soltando input sin iterarlo. ListStream infinito inmediato y segunda llamada42:0,1996ms incluido arranque. Productor sleep3sec y segunda llamada:0,139ms. ByteStream archivo129MiB y segunda llamada:0,162ms. No recolecta ni espera sueño. Son observaciones locales, no garantías temporales.

## Riesgo después de empezar a consumir

Fuente SDK descargada exacta: nu-plugin-core/src/interface/stream/mod.rs StreamReader::recv usa rx.recv bloqueante; Drop envía StreamMessage::Drop+flush. StreamWriter::write_all comprueba downstreamDrop antes y después de obtener cadaitem, sin interrumpir obtenerlo. Nu-plugin/src/plugin/interface/mod.rs construye entradas con Signals::empty. EngineInterface.signals/register_signal_handler disponibles; un Reset global no debe reactivar llamada cancelada.

Por tanto, un iterador lazy bloqueado esperando input puede retener reader/productor aun tras cierre consumidor. Hilo abandonado permitiría retornar pero no cierra liberación/reutilización; matar companion global afectaría otras llamadas y no es arreglo aceptado.

Presión SDK:100mensajes ListStream y50raw, bloquesraw8192bytes; cantidades de mensajes no garantizan un límite total de memoria. Aplicar presupuestos propios y medir conjunto.

## Decisión pendiente de spike

Resolver recepción cancelable de entrada y abandono por invocación: comprobar en ejecución cancelación/drops durante espera del primer/siguiente item y ByteStream sin bytes/EOF. Evaluar extensión localizada del SDK o alternativa que exponga control, sin recortar tipos ni matar llamadas ajenas. Latched token por invocación y guard de handler hasta destruir stream. No cerrar RV-003/102/008 ni anunciar cancelación acotada del puente por esta evidencia.

## Referencias

- [Nu plugins](https://www.nushell.sh/contributor-book/plugins.html).
- [StreamReader/Writer0.115.1](https://github.com/nushell/nushell/blob/0.115.1/crates/nu-plugin-core/src/interface/stream/mod.rs).
- [Plugin interface0.115.1](https://github.com/nushell/nushell/blob/0.115.1/crates/nu-plugin/src/plugin/interface/mod.rs).
- [ByteStream0.115.1](https://github.com/nushell/nushell/blob/0.115.1/crates/nu-protocol/src/pipeline/byte_stream.rs).

Build SDK emite aviso de futura incompatibilidad en proc-macro-error2 2.0.1; actualmente compila con Rust fijado1.99.0. No desactivar verificaciones ni afirmar versiones futuras.

## Prelude de módulo y conservación de entrada

Nu0.115.1 comprobado: `export def inspect-prelude [] { let args = []; r3viewer inspect-input }` pierde entrada de pipeline: null, cadena vacía y lista llegan como absent. No basta evitar capturar $in; también hay que mantener encoder como primer comando del pipeline del cuerpo.

`export def inspect-direct [--flag] { r3viewer inspect-input | ^python ... ...(if $flag { ['x'] } else { [] }) }` conserva absent/nothing/string/list. Implementar viewer con pipeline directo y construir argumentos mediante expresiones inline, sin prelude. Corpus de regresión obligatorio para el módulo definitivo.

`path self` es keyword const de Nu instalado: resuelve ruta del archivo/módulo al parsear y rutas relativas. `plugin use` necesita registro previo; no carga arbitrariamente un binario no registrado. Pruebas actuales usan --plugins explícito temporal. Distribución requiere preparar registro privado propio de instalación, sin alterar perfiles por pruebas; no insertar silenciosamente comandos de registro global en el módulo.
