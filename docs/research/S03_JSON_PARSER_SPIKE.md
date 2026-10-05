# JSON: spike de API y adaptación candidata R1

2026-10-05. Investigación temporal paralela a TOML; no dependencias ni código de producción. Contrato JSON R1 revisado por adversario, elección de parser aún abierta.

Jiter0.17.0 MIT oficial, VCS2b5ec63e505b44775e29b865c3161ea13bdad16b, crateSHA BC2F2B4E673D798B3DBAD0AF24BC23B92013E680A6B9AD300B4E5E52FAFE0586. Original `.tmp/json-research/jiter-0.17.0` intacto. Inspección fuente + dos ejecuciones reales del spike, WindowsRust1.99 debug; no SLA de producto.

## Resultado original

next_number_bytes preserva exactamente 9007199254740993,9223372036854775808,1e400,-0,1.000,1E+0003 sin conversión aritmética. Jiterpull devuelve todas las claves/valores de {b:0,a:1,escaped_a:2,a:3} en orden; no Value/Map. finish rechaza segundo valor/trailinggarbage; ocho formas numéricas inválidas rechazadas. next_str decodifica parUTF16 a un scalar y rechaza surrogates aislados; deben producir diagnóstico de representación según contrato, no U+FFFD ni afirmar semántica cooked disponible.

Original NumberRange::decode limita también raw integers a4300dígitos. 4300aceptado y4301rechazado NumberOutOfRange,16MiBrechazado inmediatamente. Es incompatible con política lexema bajo presupuestos generales; Jiter original NO aprobado como parser final.

## Adaptación temporal propuesta

Derivada `.tmp/json-research/jiter-raw-range-no-digit-cap`, única modificación en NumberRange::decode: retirar digit_start y la comprobación >4300, manteniendo loop/chunk/validación decimal/exponente/EOF. Las rutas de conversión Integer/BigInt/Float no se modifican. NumberRange también se utiliza en el helper de diagnóstico float_error: no afirmar aislamiento absoluto ni equivalencia de todos los diagnósticos; la revisión probó next_float grande sin panic. SHAarchivo modificado60F789A6F8AF138E06CA4A0802FA5BCA5FDDC198E20A771A30631C80C9BBEA78. Se añadió comentario delimitando que fuente/proceso/deadline son responsabilidad del consumidor. Cambio fuente acotado, no un parser casero; si se acepta, vendoring/licencia/originales/UPSTREAM exactos como adaptaciones previas y revisión independiente. No modificar dependencia compartida global sin pin/patch explícito del workspace.

Driver `.tmp/json-research/spike/src/main.rs` SHA1A6E6483430251F4714F8F5A00769C992D349904F868C8EC843AEF05ED72161F. Cargo.toml independiente + pathpatch temporal; targetpropio `.tmp/json-research/target`. Resultado derivado:4300/4301aceptados;8formasválidas de4301/65536dígitos con decimal/exponente mantienen bytes,10inválidas y8estrictasrechazadas. Entero16MiB raw exacto aceptado en94ms debug en esta ejecución, sin prueba de128MiB/memoria/cancelación. No reusar este tiempo como objetivo de parsing+IPC del producto.

next_object_bytes y next_bytes prestan fragmento original interior de comillas: pointeroffset dentro fuente inmutable permite span completo start-1/end+1 tras validar bounds/UTF8/quotes. Corpus escapedkey tiene rawspan3..12 y valor15..24, mientras current_index previo al value14 aún incluye whitespace. NO usar current_index como rawstart sin peek/whitespace. Keydecoded se obtiene aparte y copia cooked contabilizada cuando hace falta; bytesAPI valida escapes, también rechaza surrogate aislado y NO validaUTF8 por sí sola. EntradaUTF8 estricta sigue obligatoria; rango no es oráculo cooked. No asumir patrón pointerrange para next_str retornado desde scratch/tape.

## Gate siguiente

La adaptación exacta pasó revisión independiente sin críticas pendientes y5/5pruebas pertinentes; comparación de55archivos confirma único cambio declarado. El cierre acredita este gate, no el adaptador ni runtime final. Si aceptada, especificar arena directa con pila64, spans de contenedores/miembros y presupuestos; integrar sobre controlador TOML existente cuando termine revisión/AE. La publicación final mantiene validación transaccional, EOF, deadline2/5s y proceso terminable para llamadas largas. Duplicados requieren detección exacta presupuestada sin HashMap que pierda ocurrencias. Este spike no acredita el adaptador final, límites/cancelación/IPC ni QAvisual/matriz.

Fuentes primarias: paquete publicado https://crates.io/crates/jiter/0.17.0 , repositorio https://github.com/pydantic/jiter/tree/2b5ec63e505b44775e29b865c3161ea13bdad16b/crates/jiter , RFC8259 https://www.rfc-editor.org/rfc/rfc8259 . Código exacto y evidencia local prevalecen sobre latest.


[Revisión del gate](../reviews/S03-D2-parser.md). El contrato observable sigue en [D2](../sprints/S03-D2-JSON.md); no se ha añadido todavía el parser al producto.
