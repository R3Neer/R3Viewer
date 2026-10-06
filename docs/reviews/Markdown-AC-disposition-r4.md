# Markdown A–C: aceptación R4 y disposición compuesta

Dictamen: **SIN_CRITICAS_PENDIENTES en el código productivo A–C revisado y en la aceptación independiente descrita. No se identifica impedimento funcional pendiente para adoptar este incremento A–C.** Esta disposición combina la revisión completa R1 y sus gates con las correcciones UNC R2/callout R3/body R4; no se deduce sólo de los18 casos nuevos. No declara cierre general de Markdown, D1, portabilidad de tests, UI, Unix, SLA ni RV.

## Identidad R4 y prueba independiente

Candidato MANIFEST `7E6316CCDDB6684627658687BFD982F2AAA780AAA4A58EC30D769EAAE98088EF`,11 registros verificados antes y después. Delta exacto contra R3:

- parser `073E5F51B9B75C0CF2A02DAEACD799335FA1D92C9AA5BC986F8A7F277C1DC65E`.
- pipeline `018257D8E3A8E6EAF950D5CC1C5A47AA214DD6D8F91BE1247E49D5746320668F`.

La copia propia contrasta315 archivos con el padre privado R3:312 completos iguales,2 overlays exactos y lib.rs con prefijo anterior exacto/sufijo cfg(test) nuevo. No cambia ninguna función productiva ni el repositorio compartido. El código productivo restante se conserva por identidad del padre, no por semejanza de texto.

EXPECTED-before-output.json `7DBC187C451FDBB0720CA236535779C6EF11B8EBB41E20CECA15F91068F53F34` precede a los outputs. R3-EXPECTED-byteexact-preserved.json conserva íntegro el oráculo anterior `2CBAB3C39E0B4BCB921FF1381C638EAE9B51D4940A3FD8A026422534AF7FFA50`. Marker-only lower se incorpora como **nuevo control Quote** acreditado por la fuente primaria ya contrastada; su FAIL anterior permanece en R3.

Build --lib --no-run termina0 en1m42s, misma sesión45750 sin reinicio. Ejecutable cfg(test) propio `6809FC379E48A6EF97BF658440E7C7533A6357B68DB0484F5D3FD725FBA46BCD`, idéntico antes/después, copiado como own-callout-pure-tests.exe. Dos filtros exactos llaman project/validate reales, sin usar Debug como golden ni simular un modelo. **18 casos semánticos PASS y6 mutantes rechazados**, ambos exits0.251 tests filtrados no se ejecutaron ni se cuentan como aceptación. No se crean helpers/workers/Jobs ni nuevas pruebas OS; el bin worker productivo congelado R4 no se ejecuta en esta continuación.

## Defectos corregidos y pertenencia acreditada

MD-CALLOUT-BODY01: ambas fixtures originales `> [!note]\n> [](y)\n` y varianteNOTE pasan sin reinterpretación. Callout.aux1=NONE, un DomainBlock con runCount0, owner Paragraph hijo del Callout, SpanLink y Link.labelDomain recíprocos0..0, destino y y rawToken `[](y)`. Origin de Span/Link apunta al token Source12..17. Source permanece íntegra. El upstream uppercase sigue imprimiendo Link12..17,y sin Text/SoftBreak; la corrección no modifica los eventos de biblioteca.

Se repiten además las tres fixtures originales de MD-CALLOUT-EMPTY01, Both-empty, Both-styled, CRLF/Unicode y EOF sin newline: título0Runs conserva DomainCalloutTitle, cuerpo vacío conserva otro DomainBlock/Paragraph, owners distintos, destinos/origins acreditados y Strong real. Título normal, Quote ordinaria, crossing de estilo y marker-only lower/upper conservan sus roles. Cuatro casos adicionales de cuerpo CRLF/EOF/custom/TIP pasan sin título sintético.

Los seis mutantes de pertenencia siguen rechazándose: título ausente con span vivo, owner incorrecto, Span.domain cambiado solo, Link.labelDomain cambiado solo, ordinal1..1 en dominio0Runs y Origin fuera de Source. Son comprobaciones estructurales concretas, no una promesa de detectar cualquier interpretación semántica falsa de un productor malicioso.

## Revisión estática exhaustiva terminada

No queda ningún módulo productivo A–C pendiente de lectura. Se inspeccionó el cuerpo productivo completo del R1 y se revisaron por diff exacto los cambios UNC/callout/body. Las pruebas del autor se distinguen del código productivo y de los oráculos propios. La lectura comprende:

| Parte | Comprobaciones realizadas |
|---|---|
| document_model/source | records/IDs/enums y getters, Document inmutable, autoridad Arc Source/hash/identity, SourceExact frente a Decoded y origin acreditado; límites lógicos separados de memoria física |
| document_budget/utf8 | reserva y leases, transporte/scratch/admisión de conteos, growth old+new/capacity overshoot; comprobación cancelable UTF8 antes de lecturas unchecked; pool proof sin tabla de boundaries invisible |
| document_validate | ranges/enums/reserved, ownership/reciprocidad/use-once, grafos y profundidad, Item mixto, perfiles, Span laminar y estilos recalculados, empty spans, Alt/Title y recursos; construcción privada después de quarantine |
| document_protocol | dirección/IDs/header/lengths antes de body, batches/stride/counts/offsets, hash incremental/End/tail/EOF, Failure allowlist, Source input completo y UTF8; bounded read y admisión previa |
| document_parser/obsidian | eventos/proyección real, conservación fuente/origins, Source/Decoded, atributos/perfiles, mixed Items, Alt/Title, recursos, spans/nesting, compact/remap y cuota; adaptación wiki contextual sin regex global ni normalización de otras barras |
| document_worker | START exacto/EOF antes de source y parse, framing/Failure y límites; no interpretar destinos como permisos IO |
| document_controller/helper | prepare/poll/cancel y deadline original, pareja IdentityLatch consistente, slots/busy/Pending, primer error sticky, escritor/lector/source leases, preparación/Job/membership antes de START, recepción EOF/exit/joins/cierre, reap/Active0 antes de liberar autoridad y siguiente spawn |

La revisión incluye las rutas de error productivas y la privacidad del proof: ValidatedCandidate no permite mutar el modelo aceptado; Document no expone una puerta de reconstrucción sin validación. No se añadieron API públicas ni HANDLE escape para probar. La lectura no sustituye evidencia de intercalaciones o fallos nativos; esas reclamaciones se limitan a los gates realmente ejecutados.

R4 usa **callout_span_header** tanto para decidir el owner necesario como para remapear. En spans vacíos consulta el Origin frente a title_start/line_end/next_line_start; conoce LF y CRLF. Nonempty conserva la regla ordinal anterior y los guards de crossing. Si la zona no es acreditable conserva Quote antes de mutar, en vez de forzar un owner. No reescanea texto por regex, no introduce Run sintético ni amplía arena/protocolo/controller. Los mismos controles cancelables y comprobaciones de índices/checked_sub siguen vigentes. No se identificó otro defecto concreto en este delta.

## Evidencia compuesta para adoptar A–C

Los siguientes informes propios se mantienen como evidencias separadas, con sus FAIL y aclaraciones originales. No se suman controles solapados para inflar cobertura:

| Evidencia | Resultado y alcance conservado |
|---|---|
| R1 parcial, informe `214BAF93375CD07B82AB1272F6C2067B888FA5A5E233C54E7A049667FA5E6E08`, manifiesto45 `3BC59D9404D16A3343362437D76E54D7396E862B988C4FE8BF5141DE444413F9` | 41 semánticos PASS en reevaluación, UNC FAIL y mixed-tight harness error preservados;14 mutantes rechazados tras corrección explícita del harness;6 cuotas y2 admisiones/growth/lease propios. Root20 están incluidos en los semánticos, no son20 adicionales. Gates controller nativos propios de cancel/late-poll/sticky Pending/retry/reap/nextReady con inyecciones claramente diferenciadas |
| R1 continuación, informe `2AD89B5580E8FDF457E7843513C49E27CD18A7C574A562387F25FA7DAAB49691`, manifiesto28 `5A1D4AB3DCF25272B63B3058A1609DFE4BA02D980384AE7EF6769D3277872936` | lectura productiva completa; Source128MiB exact/+1 y codec, plazo vencido/cola, fallos compuestos sticky y siguiente Ready; mixed-lazy correcto y nuevo Run/ChildCode/Run. EmptyTitle3PANIC conservados. Codec25.43s bajo control privado60s, **no parser/SLA10s ni memoria total** |
| UNC R2, informe `BBC9B7F4C8FCE0A6784DA89A91F6356937FD90ED83B539598D7E325B62983C29`, manifiesto17 `4D785BF4CAFE0FEB7FD939F8E908561908B20B0B3933892448C346269167C249` | misma fixture UNC corregida +24 casos de fuente/contexto,5 falsificaciones rechazadas y3 inertes. Sólo project/validate. Código protocol/controller/quota no cambiado |
| Callout R3, informe `175CA044FC2163F55BAC2724B0746C642FDCAF937F5421583961C7895E97BED4`, manifiesto24 `3C974999858447E11431408B22F2EA9AE64287532D6665BADC71763B3BEEF89E` | tres PANIC originales corregidos, owners título/cuerpo acreditados en Both-empty; BODY01 lower/upper FAIL y marker-only harness error preservados; seis mutantes rechazados. Sin aceptación R3 general |
| Esta R4 | corrige BODY01 y repite los casos necesarios del mismo parser,18PASS/6REJECT. Los gates invariantes anteriores no se repiten ni se extienden de alcance por este PASS |

Las críticas funcionales concretas UNC, EmptyTitle y BodyOnly quedan resueltas por fixtures independientes originales sobre deltas exactos. Las aclaraciones de harness no se presentan como fixes productivos. Los fallos históricos no desaparecen de su candidato anterior.

## Límites y pendientes externos a esta adopción

Esta disposición permite adoptar el incremento productivo A–C revisado sin impedimento funcional conocido. No aprueba D1/provider/render/UI ni portabilidad del test con fixture absoluta: esa corrección está pendiente y fuera de este encargo. Tampoco exige repetir los gates invariantes para declararlos nuevos PASS sobre el exe R4: la evidencia del controller nativo corresponde a las identidades y procesos R1 documentados, mientras esta continuación prueba parser/validate compilados con fuente R4 exacta.

No se declara cumplimiento general SLA, allocator interno del parser/memoria total del proceso, fallos naturales de kernel, todos los descendientes, todas las intercalaciones ni funcionamiento Unix. No se cierra RV ni producto completo. Las sesiones propias finalizaron, no hubo rescate externo y no se ampliaron las operaciones OS. Los tests cfg(test) heredados no ejecutados permanecen claramente separados de los dos filtros reales de esta continuación.
