# UNC R2: revisión y aceptación funcional acotada

Dictamen: **SIN_CRITICAS_PENDIENTES para la corrección UNC en el alcance funcional ejecutado**. MD-AE-UNC01/MD-UNC01 queda refutado por la misma fixture original sobre el delta R2. No es aceptación global A–C: MD-CALLOUT-EMPTY01 conserva sus tres PANIC R1 y requiere otra corrección/aceptación.

## Identidades

Candidato MANIFEST `3BB930B73579F56C7C92F3D318CF6A5B56CAF01F3CD9FDDC207249AE327FACC4`,37 registros verificados antes y después. Se copiaron sólo los overlays productivos previstos:

- document_obsidian.rs `88899609A10A743F3BD89FC7C19DCF5C51BA1AD8F471DD80926F84A9C1E712AE`.
- document_parser.rs `A686071CFC4E3A57B744BB101918AA4D7D4BA25E78FB7C082D7A2532376D4626`.
- document_pipeline.rs `21651288EACD1B378ADDA09BB72BA476556D405C1068414A072206CA8FCCFF79`.

El contexto privado contiene los 213 archivos R1:208 completos iguales,3 overlays R2 exactos y2 archivos con prefijo productivo R1 intacto/sufijo cfg(test) propio. Recursos/vendor suplementarios de la copia previa permanecen como contexto complementario, no se atribuyen falsamente al manifiesto213. El original R1 y sus evidencias no se alteran. La copia R2 y sus pruebas no modifican funciones productivas ni relajan el proof/API.

Oráculo independiente EXPECTED-before-output.json `FA707DAC58137C55BE728CC61ACA68808F1FD90A7D62F7DE2B879F9E8F886D07`, harness `1421573CDE733D0E81966924D186737D7F2D507721B49B480327B311DA5D6EDD`, registrados antes de ejecutar. Build --lib --no-run final0,50.77s. Ejecutable propio cfg(test) `E95DBE1DCD13440E99BE6BFF2B46E7C15935FE077AC6027E5585B9E8220F73C2`, copiado como own-unc-pure-tests.exe después de comprobar que la identidad no cambió.

## Resultado independiente

Un único filtro exacto ejecuta **33 controles**, exit0. No se ejecutan los242 tests filtrados ni baterías anteriores. La prueba llama al parser real project y al validador real sobre Candidate; no usa Debug como golden, ni devuelve un modelo simulado. No crea helper/worker/Job: no aporta nuevos recibos OS ni una ejecución del bin productivo R2 congelado.

1. La fixture original wiki-unc-crlf.md conserva SHA `E44A122AA4803F95B644974A6B469A61C4F6BE68CA077919D9A4DEEDC8619A0C`. Pasa destino con dos backslashes iniciales, rawToken18..53, fuente íntegra, Strong del alias y TextRef Decoded con Origin.EnclosingToken al token completo.
2. **24 casos propios**:2/3/4 backslashes iniciales ×tabla/párrafo ×link/embed ×alias/sin alias. Se acredita destino literal, rawToken entero, origen obligatorio del targetDecoded, fuente sin cambios y validación host. La barra estructural antes del alias se quita sólo en tabla; se conserva en párrafo. Strong del alias y dimensiones100x145 del embed permanecen según contexto; sin alias no inventa dimensiones.
3. **Cinco negativos propios**, cada uno sobre nuevo modelo validado: target start+1 (pérdida inicial), Origin ExactToken falso, origin start+1, target cambiado a Source y Source diferente del mismo largo (server→xerver). Todos rechazados por validate. No se inventa un código de error específico adicional: el oráculo exige rechazo, porque cada mutación puede infringir más de una comprobación estructural.
4. **Tres controles inertes**: Core, fenced Code y HTML. Sin enlaces/recursos wiki nuevos, fuente intacta, host valida. No se extrapola la recuperación UNC a estos contextos.

## Revisión del delta

expected toma la fuente como autoridad, limita first_body_loss al perfil Extended y al prefijo original que comienza con una pareja de backslashes. Antes de recuperar compara el destino real de la dependencia fijada contra prefix[1..] mediante comparación cancelable. No introduce un fallback para mismatches arbitrarios ni normaliza las otras barras. La adaptación de la barra estructural de tabla permanece separada y exige ownership contextual. Producer marca decoded/origen enclosing cuando hay cualquiera de las dos adaptaciones; host exige ese mismo origen acreditado. El desplazamiento ASCII de un byte no corta UTF8. No cambia wire, cuotas, controller ni helper Job.

Las nuevas pruebas del autor en document_pipeline incluyen una fixture por ruta absoluta de Cviz. Ese test necesita el archivo privado indicado; **esta aceptación local no acredita su portabilidad/reproducibilidad en un checkout distinto sin proporcionar esa fixture**. La prueba independiente copia la fixture original en su directorio propio y conserva su SHA; no altera el archivo del autor. Conviene integrar esa fixture mediante ruta relativa cuando se adopten los tests productivos, sin reinterpretar la evidencia actual.

## Límites conservados

SIN_CRITICAS_PENDIENTES se aplica a la corrección UNC y a los33 controles locales descritos. No aprueba todo Markdown, D/render, Obsidian resolver/IO, Unix, QA visual, SLA, pruebas del worker exacto R2 ni cierre de RV. No se repiten Source128MiB, deadline ni los43 semánticos originales, porque controller/wire/quotas están fuera del delta. No se rescatan procesos externos; no se crearon auxiliares que requieran teardown. Las sesiones de compilación y ejecución propias terminaron.

El informe R1 followup y su manifiesto quedan como padres independientes exactos; sus PANIC/FAIL no se borran ni se reemplazan por el PASS UNC R2.
