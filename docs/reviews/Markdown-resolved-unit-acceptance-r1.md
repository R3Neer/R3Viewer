# ResolvedUnit R1 — aceptación independiente

Dictamen: **SIN_CRITICAS_PENDIENTES dentro del delta de biblioteca ResolvedUnit/ResolvedRefUnit y su consumo en layout**. Siete grupos propios pasan, más17 regresiones b1 pequeñas directamente afectadas. No aceptación de scheduler WIP/contexto íntegro, renderer, CLI, ventana densa Normal, backend ni cuotas nuevas.

Freeze MANIFEST90242D01612AADE75F1824FA164AFDD8761F4946217F841B9F0EE9B9AF2D4547:39 registros verificados antes/después sin discrepancias. CONTEXT607 archivos extraídos a copia/target propios; PRODUCT4 exactos. Única mutación privada: sufijo cfg-test en document_text_tests.rs, prefijo del candidato byteexacto. BASE document_text918F93.../layout3CF617... y cuatro deltas revisados. No patch/adopción compartida. CONTEXT contiene scheduler WIP sólo para factories existentes; no se adopta por este informe.

Oráculo propio DBB1B302BBCC98ED19B85953CDB1E26269E60842C94F6437D472E50F5EDAF695 precede outputs. Addendum0B5DD441E590B135986518B76E367F028FC1F9975A32E1C44144D4138C92AE39 precede suplemento. La construcción del Document de Runs usa helper fixture existente validado, sin copiar oráculos del autor; Ref usa parser/validador reales. No Debug como comparación de modelo.

## Ejecución y evidencia

BUILD-attempt1 terminal0/56.24s, sesión90812 cerrada; bin OWN-test177EE3ECDE6B0AD153CBA7193FBE809EBEC62AB53929ED85327101CDDF5CACCB. Seis grupos pasan0.03s:

- I01 Point de otro documento rechazado sin mutación; token legítimo/rango original.
- I02 cluster a+comb y ZWJ intercalan Runs sin segmentación adicional; Coverage de ambos Runs conserva estilos/rangos Source0..1/1..3. Decoded é+comb conserva dos Enclosing0..8/8..14.
- I03 compute y Exception18001: frontera totalWork−1/total/+1, preadmisión insuficiente, cancel del AtomicBool original justo antes del commit mediante hook cfg-only del candidato. Point.offset/line/column/Position/previous/cursor iguales al original al fallar; continuación por step legado coincide con control. Cargos ejecutados conservados, no refund.
- I04 Ref real Decoded con C1/entity/ZWJ produce A\u{0085}&👩‍💻, RefCoverage con id/origin/local reales, callback fallido propaga error cobrando trabajo; wrapper falla sin publicar ante Work/cancel.
- I05 C1 se proyecta a componentes visuales seguros; CRLF unidad y tab conservados. Giant18001 oversize mantiene token prestado, visitante completo en chunks≤4096 y tailQ. No convertir su copia de prueba en glyphString obligatorio del producto ni admitirlo en Window.
- I06 EOF no token y Point intacto; replay del token sigue cobrando; callback coverage erróneo se propaga, cancel precede nuevo callback.

BUILD-attempt2 terminal0/6.24s. TEST-attempt2-more exit101 es **harness error preservado**: lookup suponía SourceTextRef x en input de recurso, pero ese input se fabrica Decoded. Ningún fallo de token se produjo: unwrap de setup antes del índice. Fuente primaria document_parser text(force_decoded) y camino resource justifican retirar sólo esa suposición. Original fixture/bin/test/log permanecen. HARNESS-CORRECTION-before-output fija párrafo literal x\n cuyo TextRef Source sí es fiel; no cambia expectativas de cobertura ni producción.

BUILD-attempt3 terminal0/3.46s; TEST-attempt3-more I07 PASS0.00s. Parser/validator real produce SourceRef x con local0..1/source0..1/ExactSource/id propio; totalWork−1/total/+1 comprueba atomicidad wrapper. RI triple, Hangul, Indic y Arabic pequeños contrastan frontera con unicode-segmentation pinned sobre fuente explícita. No corpus universal. Seis grupos originales no se repiten innecesariamente tras sólo añadir el suplemento.

REGRESSION-b1-17 PASS17/0.10s sobre bin final: navegación/anchors/Code/Unicode/old+new/Reference/autoridad/retención existentes. Son regresiones del autor ejecutadas aquí, distintas de siete grupos propios. Sin32MiB, sin suite general, sin helper/proceso nativo o rescate. Identities finales/binaries en READBACK-after. Aviso Python314 de localización al generar documentación no impidió exit0; diagnóstico read-only de ruta inexistente document_consumer.rs no representa fallo de producto ni alteración de fuente.

## Revisión estática completa del delta

Tipos core/ref crate-private noClone/noCopy, campos privados, token prestado al índice inmutable exacto. Constructor único prepare_resolved valida Point.identity/offset/cursor y límites, preadmite sizeof Position+core+wrapper incremental checked antes de prepare_step; prepara sólo next local y Unit. Compute conserva su único clone previo de Point; Exception su único e.after.clone. Rangos/start/end/cursor/identity y último cancel admit(0) se verifican antes de mover token y Point. Fábrica Ref comparte preparación sin comprometer core primero; publicación final no tiene cargos/callbacks/Result adicionales. No deshacer un Point ya comprometido mediante rollback extra.

visit_projected_at arranca Position real y avanza chunk.end_pos con scratch4096; visit_coverage_at Runs/Ref recorre esa misma posición con lógica Source/Decoded/Synthetic/control/style/link existente. Token comprueba Position final completa Raw/part/offset contra end y último cancel. No locate/seek en los visitantes resueltos; random-access permanece con locate y su rango vacío legacy. Ref usa RefCoverage auténtico y selección/id propio; no RunCoverage o RawAnchor ficticio. Ownership del wrapper compara dirección del índice propietario para cobertura. Privacidad revisada estáticamente; no test compile-fail ni unsafe que fabrique tokens.

Layout sólo sustituye step y visitantes por token prestado: tabs/fill/coverage/copia siguen cobrados y callbacks no cambian metadata. SYMBOL oversize sigue Reference, no relleno enorme obligatorio. Work256KiB intacto y Consumer/cancel original; no colección de tokens ni caches nuevas. Token120B/Ref128B del autor concuerda con layout de tipos; aquí no se ejecuta una medición nueva del denso ni se elimina ese coste.

## Límites

Resultado dense885 vs207 y Reference(Work), así como36texto/17b1 del autor, permanecen evidencias del autor; no son siete pruebas propias ni Normal120×40. La propuesta ya reconoce que120B por4800 excede Work256KiB antes del resto. Este dictamen permite componer exclusivamente PRODUCT4 bajo identidad BASE indicada; no reemplazar contexto WIP completo ni sobrescribir incrementos posteriores. B2/D3b deben cotejar sus deltas al integrar. Sin renderer/UI/QAvisual/ConPTY/Unix/globalacceptance o calibraciónWork2MiB.

Sin críticas concretas pendientes del delta revisado. Ninguna sesión, build o helper propio activo al cierre.
