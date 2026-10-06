# D2b0 R2: revisión independiente de continuidad HASH → SOURCE

**SIN_CRITICAS_PENDIENTES en la propuesta de continuidad.** No es aprobación de código R2, del kernel completo, de D2 ni del pump.

Propuesta exacta AE42FCE6486F254B92ACAD36C9168FCDA40B67D231B963CEC70F4F6C0C602BDC, contrastada con MARKDOWN-D2B-LAYOUT-SEARCH §10 (contenido BA4D78A983B4763239FD45FAAAB6418C73C8D5BF97E5FA1CD048352FEE139938), document_model/document_source/presentation actuales. No ejecución, outputs del autor, modificación de propuesta o producción.

## Continuidad y exposición

PreparedSourceContext conserva exactamente Input original más Arc<DocumentSource> completo y validado. Constructor cerrado en HASH o desde Document ya validado: Arc original/origin/generation/request deben coincidir; digest se presta del DocumentSource inmutable, no se suministra externamente. HASH fija inputIdentity una sola vez y binding con digest None; sólo tras join físico y recibo final aceptado se entrega el contexto. SOURCE utiliza ese contexto y digest real Some, con nueva revision permitida pero igual inputIdentity/generation/request/original. Esto evita exigir digest previo al propio HASH y evita fabricar otro contexto tras la transición.

El contexto puede construirse en la closure antes del join sin resultar adoptable: permanece candidato privado en el owner. La única entrega a quien puede iniciar SOURCE es take_validated tras final receipt y cotejo completo. El código posterior debe conservar esa frontera, no exponer el Arc de candidato desde poll ni promover un output con sendfailure.

Desde Document, source_handle()->Arc<DocumentSource> crate-private es delta mínimo coherente: Document actual sólo ofrece source()->&DocumentSource, y su Candidate conserva inner.source como Arc. Clonar ese Arc inmutable comparte identidad/allocation; no rehash, reopen, copia de String, nuevo constructor del modelo ni debilitamiento de proof. Input mantiene además Arc<Document>/leases mientras Source candidato o resultado vive.

## Dueño de la operación

La misma PreparationAuthority<DI,DO> conserva ActiveOwner::Source durante HASH y, una vez reap físico libera la operación HASH, instala la operación SOURCE capturando contexto/input originales. Compartir un contexto no duplica el owner del hilo ni permite ocupar otro slot simultáneo. Las factories cerradas y exclusión global/Pending deben aplicar igualmente a objetos/sesiones distintos; una continuidad semántica correcta no demuestra por sí sola ese kernel. La propuesta reconoce que la retención tipada concreta se contrastará con código R2; no se acepta el intento R1 ni se sustituye PreparationAuthority/ActiveOwner por un disclaimer.

Capturar context y context.input sólo crea aliases de allocations existentes. Receipt debe contar una vez cada allocation original, además de las nuevas headers/metadata/model capacities; retención tras old owner drop se acredita por leases del contexto/modelo, no por el mero recuento de Arcs. Source mantiene sus límites independientes del Consumer/RSS.

## Controles que siguen exigidos

HS01–05 son coherentes con los oráculos independientes 067E00B30DE7FF982CB351D197AC25557DE9FEEDCE5C08BE86E0828F8B81ACC6: HASH real y Source real con identidad/digest/context; stale/cancel/Pending sin Source start; digest/revision foreign rechazados; Document conserva ArcSource/lease sin rehash; error Source no Ready y siguiente operación sólo tras cierre. Mutantes de binding son controles internos del kernel, sin introducir constructor exterior de digest/context en producción.

Los controles se fijarán/adaptarán únicamente a las APIs del código congelado, manteniendo estos criterios antes de outputs. No resultados ni tests nuevos aquí. Sin aprobación de layout b1, TUI, restauración, ConPTY, scheduler Syntect, Unix o cierre de backlog.
