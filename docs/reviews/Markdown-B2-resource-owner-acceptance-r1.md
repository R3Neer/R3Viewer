# B2-RESOURCE-OWNER01: aceptación independiente final

2026-10-10. Informe final del adversario sobre el fix congelado, con cinco casos propios y 45 regresiones aprobados. Alcance: clasificación del bloque contenedor de recursos y su cargo de trabajo; inspector, Source común, sesión y gráficos siguen pendientes. Original privado SHA256: 512B5C9B6AA0B74006541D3D2EF2D7C9BA3B1EE9AE0B258CF9F79D750DA306B0; se conserva el texto con normalización de saltos y final de archivo. [Integración canónica](Markdown-B2-resource-owner-adoption-r1.md).

---

# Revisión independiente FIX B2-RESOURCE-OWNER01 R1

Dictamen: SIN_CRITICAS_PENDIENTES en este corte. El freeze mínimo corrige B2-RESOURCE-OWNER01 sin tratar Domain.owner ResourceID como BlockID. Apto para integración por el coordinador. No aceptación de inspector completo, gate/proveedor scoped, Source común, history, sesión/pump, D3/D4 ni QA visual.

## Identidad y copia exacta

Base código 3db799c6d95513c08469295d592834b8bde79fe2; HEAD documental al preparar 1a553f647f7a92f6b03db186ceab827bf0c1b9e5. Freeze oficial .tmp/markdown-b3-inspector-2026-10-09/b2-resource-owner01-frozen-r1/FREEZE-B2-RESOURCE-OWNER01-R1.json SHA87847E755ED0425B48514B497E902DE8C6F21FC33488FF16D75E7E03D9104A35; delta ZIP SHAC7CE402202F46DDE1302DDDBEBD33D8D6F48BA32E43F0B2CD9C1D6634C94A3F1. Verificados los 10 hashes fuente/fixtures y bytes del manifiesto. document_layout.rs SHA3DF6CA52AB7368C99C03BB00287B8768A361569709322346E97068BB6637338D.

Copia propia fix-r1-product = archive base3db + diez archivos congelados. Revisión completa de 745 archivos existentes contra baseline-product: sólo difieren document_layout.rs, document_renderer_tests.rs, lib.rs. Siete archivos nuevos son módulo de tests/6 fixtures del freeze. Único enlace propio cfg(test) en document_view.rs idéntico al baseline; harness propio byteexact009FFA0C38707B10CDD5855FF5BF5775DE1CEECD9585B08647842E73D50D6CE3. Recibo fix-r1-copy-receipt.json SHA A092FDB71E608831D785F406C614D4DA5DCCBF2090D87D0C5F2A3604D76778B4. Ningún código mutable Inspector/Gate/Source leído ni mezclado.

## Cierre del hallazgo

Baseline propio conserva NO_APROBADO 2 PASS / 3 FAIL y sus incidencias/harnesses previos; REVIEW-BASELINE-R1.md SHA ADE0285EAF17A0716792CBC865B0D625F3470E2654040F6C8431480D55DC420B. Oráculos prospectivos/fixtures originales intactos: EXPECTED-B2-RESOURCE-OWNER01.md BD77F3E2C2704DC6319D92C46ABE9674E33C5C2F6833EF0C05CECD38502012A5; FIXTURES.json F36FDADFABC31FB24460646600AC04E6EB0EA471B900F1F21EEB54D5BCEC4020.

Revisión de código: sequence admite sizeof::<Block>() mediante DocumentWork antes de leer/classificar el BlockID real, usa blocks().get(block).ok_or(Invalid), calcula code una sola vez fuera del loop, elimina la lectura blocks()[Domain.owner]. Tres callsites acreditan el contenedor: celda Table real, bloque padre de Resource real y bloque de Event::Run Flow real. Prefix y horizontal usan el mismo block. No cambios de dominios/anchors, parser, Unicode, renderer, navegación ni otras políticas; lib añade sólo módulo cfg(test).

AE propia, mismo parser/quarantine y kernel de preparación reales, receipt.joined y glyph/cell_anchor reales, cinco casos sin cambios:

| Caso | Baseline | Fix congelado |
| --- | --- | --- |
| Resource2 con blocks.len2: ALPHA/BRAVO/CHARLIE | FAIL, worker panic/Cause::Panic | PASS, joined Viewport y tres ALT |
| Resource1 en Paragraph2 coincide con Code1, left100 | FAIL, BRAVO vacío | PASS, ALT/posición/ancla iguales a left0 |
| Resource1 en Paragraph2 coincide con FrontMatter1, left100 | FAIL, BRAVO vacío | PASS, ALT/posición/ancla iguales a left0 |
| ALT largo/wrap local left0 frente left100 | PASS | PASS |
| Code verdadero left3 conserva 3456789ABCDEFGHIJ | PASS | PASS |

Resultado: 5/5 AE propias PASS. Esto cierra la clasificación y el panic del hallazgo; no borra evidencia baseline.

## Cargo real y guard exacto

EXPECTED-COST-SUPPLEMENT.md SHA B4BAB4E286911805006FD3E0195CDA585EE8FF69D011A5013D67780C5E688102 fijado antes de leer/ejecutar el freeze. Fórmula anticipada: required antiguo7417231 + una sequence * sizeof::<Block>(); esquema y constassert compilados Block16words=64. Nuevo required7417295; parts[0]1373775+64=1373839; restantes seis partes intactas. Fuente de la expectativa: baseline+lectura nueva real, nunca golden copiado de output.

Delta renderer_tests sólo expresa required/parts[0] con sizeofBlock. Guard límite4194304 y Work8MiB no cambian. Familia renderer13 PASS incluye guard exacto: required-1 rechaza, required acepta, salida vacía en fallo y receipt anterior conservado. La admisión única sucede antes de clasificación/preparación/publicación; el cargo fallible propaga por el candidato sin modificar el viewport anterior. Familia view30 PASS incluye stale/cancel/old+new y Reference/geometry. roles2 PASS. No nueva API ni ledger/cargo ficticio.

## Evidencia propia inmutable

Compilación cargo test --offline --locked --lib own_b2_owner01 --no-run PASS sobre copia exacta, logs fix-r1-build.jsonl/.stderr. Bin propio capturado antes de cualquier reutilización posterior de target:

- fix-r1-evidence/own-b2-resource-owner01-fix-r1.exe SHA85F0F7C0E460BED46B72080A9662862E7CD564E2ED5B9C92060E8AB77EB85631.
- fix-r1-evidence/own-5.log SHA28148CF74609A76F76149F93804AB94909AA2D5AC358B2FBF5A4C53164D8CA39 — 5 PASS.
- fix-r1-evidence/regression-renderer-13.log SHA8DF718503343A1370BB751FF2A0D09B24A04497B50128DAE71DDBC4E11AF54B6 — 13 PASS.
- fix-r1-evidence/regression-view-30.log SHA5FAE2164F5C76D96D8C9381BE16D3F8D61F5B742CC2AE84F524696C46F3D43DC — 30 PASS.
- fix-r1-evidence/regression-roles-2.log SHA42873964408B8935DB797522F427DEEF7C2594CBEB87365EA7B09FB69790DB60 — 2 PASS.

Todas sobre el mismo bin propio inmutable, --nocapture --test-threads=1, exit0. Total ejecutado aquí50: 5 AE propias +45 regresiones existentes. Los49 checks del autor/root son apoyo separado, no se contabilizan como nuestros oráculos ni ejecutados aquí. No fmt/Clippy propios adicionales requeridos: este adversario no editó producto. Integración/commit/regresión workspace posterior corresponden al coordinador.

CRITERIOS.md/EXPECTED-NOT-RUN.json del inspector completo siguen intactos y NOT_RUN; este fix no anticipa su aceptación. No implementación ni canónico modificados por el adversario.
