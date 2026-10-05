# PDF UTF16/mapping — delta operacional R4

Autor unix_r3_review; pendiente dictamen root. **R4 = R3 más este delta**, que prevalece únicamente sobre la frase «Failure diagnosticado requiere input válido y receipts finales». R3 SHA256 **0CDBA3171F996584B6D1D753856E20FC1B3F77DB98319E03AF3A8AFE833A4BEA** permanece íntegro; layouts, Counts, cuotas, mapping, wire, receipts/sizes y oráculos anteriores no cambian. No código/runtime ni modificación de PDF Rust R4, Unicode, FAIL históricos o semanticUnverified.

## Válido y preflight

«Input válido» para validar una respuesta Failure significa **petición completa y válida en transporte**: Init con perfil/campos/reserved/request/generation/página representables admitidos, chunks contiguos dentro de sourceLength, SourceEnd con longitud y SHA256 coincidentes, hash calculado por D y stdin EOF real sin trailing. NO significa PDF semánticamente válido, documento cargable, texto extraíble ni completeness. Un PDF binario malformado puede tener transporte válido y producir Failure.LoadFailure. Los bytes PDF no están sujetos a UTF8.

H valida antes de cualquier spawn o fuente enviada: sourceLength≤1048576; página solicitada entre1 y256 inclusive; sourceLength/IDs/aritmética/capacity/particiones admitidos y fuente inmutable/hash propio. Fallo produce HostPreflightError/diagnóstico, conserva source y NO crea un hijo ni writer. El número real de páginas y la presencia de la página se comprueban en D después de cargar el PDF: conocer page≤256 en H no implica que exista en ese documento. El valor maxPages de Init es exactamente256; perfiles/reserved/flags siguen los de R3.

Failure.code1 InputLimit se conserva como defensa de D ante una petición fuera de perfil de un peer defectuoso/control inyectado; el host normal no debe enviar source>1MiB. Una rechazo de input por D NO transforma una petición inválida en «válida» ni da derecho a validación sólo porque haya un Failure frame.

## Rechazo temprano por D

Si D detecta Init inválido/fuera de cota antes de consumir source, puede cerrar stdin y terminar; puede emitir Boot+Failure para diagnóstico acotado, pero **H no lo publica como Failure validado** sin la petición completa y todos los recibos finales. No existe obligación de drenar fuente ilimitada ni de esperar que llegue SourceEnd bajo un Init ya rechazado.

H cancela la transferencia por su latch de error, deja de encolar chunks y entra en cleanup acotado de R3 (deadline de request10s, intentos cleanup2s, Pending retenido si falta prueba). El writer conserva su autoridad hasta close/join; source leases no se liberan por la mera recepción de Failure. BrokenPipe, write/flush/close/panic, overflow o error reader tardío quedan sticky y se evalúan después de joins. Esos errores **impiden Failure validado**, aunque la respuesta tenga código reconocido, EOF y exit0; el resultado es HostError(InputRejected/TransferIO o etapa acreditada) con evidencia del frame recibido sólo como diagnóstico no validado. Nada resetea esos errores al clasificar un fallo de contenido como esperado. Reaped/handles/streams cerrados prueban cleanup, no validez de transporte.

Un rechazo temprano tampoco implica que siempre haya BrokenPipe: puede haber carreras en las que H ya hubiera entregado todo. H verifica su preflight, transferencia completa de la petición construida (incluidos SourceEnd y hash), EOF y recibos finales del writer. D, por contrato de implementación, sólo emite Failure de contenido después de verificar longitud/hash/EOF. El frame Failure16B no contiene un ACK de consumo ni permite demostrar contra un peer arbitrario que D leyó cada byte: no afirmar esa prueba independiente. No se añade ACK. La revisión/AE del productor debe probar ese orden. Code1 InputLimit se clasifica HostError de petición, nunca Failure de contenido validado. Si Failure se observa mientras H aún no ha completado SourceEnd/solicitud de cierre de stdin, se fija un latch EarlyInputRejected y se cancela transferencia: HostError permanece aunque el writer termine luego sin error. Un productor que emitió después de EOF no requiere un drenaje nuevo; H espera los joins y sus receipts finales antes de validar. Una carrera conservadora puede terminar en HostError sin falso éxito; debe registrarse y medirse, no presentarse como receipt de consumo. Sin BrokenPipe, exit0 por sí solo tampoco acredita validez. La AE prueba por separado el código del productor y el host ante peer temprano controlado.

## Casos previos adicionales

`PDF-UTF16-mapping-oracle-supplement-r4.json` añade expectativas antes de outputs, sin cambiar matriz18IDs R2 ni suplemento R3:

- **U16R4-A:** H sourceLength1048577 o página0/257 → HostPreflightError; contador Popen/spawn=0; source original intacta.
- **U16R4-B:** peer D rechaza Init antes de consumir source y cierra stdin; H recibe Boot+Failure/EOF/exit0 y writer BrokenPipe → HostError, cero Failure validado/Artifact; cleanup completo o Pending retenido explícito, sin drenaje sin límite.
- **U16R4-C:** D devuelve Failure de PDF LoadFailure tras input de transporte completo, pero writer flush/close emite error tardío publicado durante join → HostError sticky, cero Failure validado; todas las pruebas de cierre se registran y no convierten el error en PASS de respuesta.
- **U16R4-D:** PDF malformado de longitud admitida con framing/hash/EOF correctos, Failure.LoadFailure antes de Begin y todos los recibos IO correctos → Failure validado de contenido. No se llama PDF válido ni SourceNoText ni semanticComplete.
- **U16R4-E:** respuesta Failure temprana sin error observable de writer, observada antes de SourceEnd/solicitud de EOF de H → latch EarlyInputRejected y HostError/diagnóstico no validado; no usar ausencia de BrokenPipe como prueba de consumo.

Este delta retira la ambigüedad, no rebaja receipts ni habilita ejecución antes del dictamen adversario. Root decide cierre de diseño; autor no se autoaprueba.
