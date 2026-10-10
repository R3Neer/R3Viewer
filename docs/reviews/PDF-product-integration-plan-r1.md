# Dictamen final de integración PDF R1

Informe privado `REVIEW-PDF-PRODUCT-DESIGN-R1.md`, SHA256 `C4F4ACD686BA4E485C2DD151FF3F761A8D02B73BC9397D708D478F22D46B640A`, importado a LF. [Diseño canónico](../refinement/PDF-PRODUCT-INTEGRATION.md) y [adopción](PDF-product-integration-adoption-r1.md).

# Revisión independiente — diseño PDF producto R1

## Veredicto

SIN_CRITICAS_PENDIENTES, limitado al diseño exacto de PROPOSAL-R1.md C37ED8FEEA9BBEAC8932E75586E0E9A84D74A2DF1DDF6E0F2FA35680F7429A1B y su matriz prospectiva ORACLES.json 657433489C7A436EFE9D54C082F185ED7A920C880CBDAC4BE29D355B2E542D5B. No implementación, runtime, aceptación gráfica ni cierre RV-204/202/203/305/306. Ningún fallo semántico histórico se cambia a PASS.

Identidad MANIFEST.json 380D491EA561DC7FA87AE1D375C1C84735BC165E92D0D9B368ED0217747783F5. Verificación propia: cuatro artefactos y 35 referencias SOURCES coinciden en longitud/hash físicos; comprobación registrada en INTEGRIDAD-R1.json. No reejecuté los 1112 registros del freeze UTF16: comprobé su manifest/helper/módulos seleccionados referidos; el recibo de recuperación del root es integridad histórica, no mi aceptación de runtime. La propuesta declara base f04954817ecd3afc1e07dc30ad6527852632286b; su identidad se fija por bytes actuales, sin atribuir limpieza actual a esa declaración.

Criterios PD01–18 derivados y guardados antes de leer propuesta/oráculos: CRITERIOS-PDF-PREPROPUESTA.md SHA256 6C61B9168507C962469A65996B77879DDDCB78E8964A93193921AF48882D1876. Se conservaron sin adaptación tras lectura. Canonónicos: PDF-G0, PDF-RUST-GATE, UTF16 R3/delta R4/VALIDADOR, backlog y cierres independientes Rust R4/UTF16 R2. Toda evidencia de esta revisión es documental; los P01–P26 son escenarios previstos, no casos ejecutados.

## Correspondencia y resultado

| Criterio propio | Evidencia de diseño R1 | Escenarios prospectivos |
|---|---|---|
| PD01 alcance físico/lazy | Contrato p. física; Lazy y cortes P3–P5 mantienen producto completo, sin reducirlo al spike | P01/P03/P18/P21/P25 |
| PD02 snapshot/preparación | BinarySnapshot bytes inmutables/lease/hash; rechazo stale/fallo y destino tipado antes commit | P01/P02/P12/P16 |
| PD03 loader/procedencia | Referencias exactas DLL/headers; bundle/ABI Windows reutilizado con diff y packaging/Unix aún pendientes | P12/P26 |
| PD04 aislamiento/RAII | Actor serial contenido, préstamo/drop conservado; native-blocked kill separado de Drop nativo; sin acciones | P15/P16/P22 |
| PD05 port Rust | Nuevo PdfController y worker, Python sólo harness privado; scheduler real en lugar de atomics independientes | P13–P20 |
| PD06 receipts | Primer byte antes captura, cierre real/inflight conservador, errores sticky después de joins; cero publicación temprana | P13/P14/P17 |
| PD07 Pending | Autoridad física retenida, cleanup2s y retry mismo owner; bloquea actores de otros adaptadores | P15/P16/P25 |
| PD08 ledger | H256+D192 known heap; H incluye source/cache/old-new/history y otros adaptadores; Job512 committed separado | P18/P19/P20 |
| PD09 wire | Envelope versionado nuevo, cuotas antes descriptor, digest/End/EOF/exit/receipts, host valida; V separado opcional cobrado | P13/P14/P17 |
| PD10 UTF16 | Llamada única acotada, tablas sin reinterpretación, decode1024/carry; error sin prefijo ni replacement inventado | P04/P05/P06 |
| PD11 mapping/geom | Exact sólo índice nativo, SourceUnverified, bytes originales sin offsets PDF fingidos; raster independiente | P04/P05/P09 |
| PD12 literal | Todas ocurrencias y solapamientos, ventana256 y re-traversal; chunks/gaps/páginas separados, parcial/continuar explícito | P07–P11 |
| PD13 estados | NativeEmpty no SourceNoText, InvalidUnicode/Limit/APIerror distintos; capa legible válida conservada | P03/P04/P06/P10/P12 |
| PD14 raster/composición | Checked tamaños y tiles16MiB, crop/rotation/zoom/fit; borrado real backend y fallback visible | P09/P17/P21/P25 |
| PD15 cache/trabajos | Cache común64MiB dentro H, clave content/engine/options; cargos hasta últimoDrop, old-new preadmitidos | P19/P20/P21 |
| PD16 links/history | Whitelist inerte nueva, targets tipados bajo cuotas; history común quota128 depende freeze final aceptado | P12/P22/P23 |
| PD17 inline/embeds | Página primera/seleccionada, N/total y referencias; owner real/ciclos/profundidad, no modificar notas | P24/P25 |
| PD18 gates/QA | Cortes P1–P5, freeze/oráculos antes outputs y CodeAE/AE propios del delta; visual Windows y Unix pendientes | P01–P26 |

Resultado: alineado en diseño para las 18 familias; cero familias acreditadas por runtime en esta revisión.

## Factibilidad y fronteras que siguen abiertas

El port del supervisor Python a Rust constituye un delta nuevo, con estados y ownership concretos y fallos históricos convertidos en requisitos, no en una traducción automáticamente aprobada. Validar streaming en H es una elección viable porque conserva un validador ajeno al actor nativo y la misma semántica; exige su freeze técnico y negativos de schema/tablas/digests/receipts. No exige proceso V para aparentar equivalencia con el experimento. Las tablas versionadas pueden encapsularse/fragmentarse bajo la nueva cota de frame: ésta no autoriza reinterpretar strides/enums ni copiar un batch experimental entero mayor que la cota. La futura especificación wire exacta debe fijar framing/batches antes de construir y ejecutar P1.

Los perfiles producto son candidatos explícitos, no mediciones ni garantías de RAM total. H256/D192 sólo limitan requested heap conocido; el Job512 committed y allocations nativas/allocator/GPU/terminal quedan separados. Una fuente128MiB con cache64MiB permite sólo64MiB restantes en H: las combinaciones que superen ese saldo se rechazan antes de asignar, preservando el anterior. Quota128 de history no implica admitir128 snapshots máximos. P19/P20 verifican la agregación física y lastDrop; deberán fijar los cargos exactos de cada fixture antes de runtime. P18 no es todavía un golden de admisión: el plan pide fixtures/oráculos físicos prospectivos y no presenta esta matriz como corpus ejecutable ya congelado.

Los resultados textuales se mantienen NativeDerived/Unverified aun al completar el recorrido o concordar APIs/flags; la búsqueda declara cobertura del texto extraído, nunca del PDF fuente. Missing de asociación geométrica no borra Unicode válido ni convierte por sí mismo la cadena en un hueco textual; la separación real del dominio tiene que estar declarada. Esto permite buscar fuera CropBox o sin bbox con salto físico/rango y diagnóstico. Raster Ready no queda bloqueado por esos fallos de exactitud.

RAII/FFI/layout y el helper EOF aceptados no acreditan los nuevos transforms/tile/clipping, metadata/links/bookmarks, scheduler/global ledger, validador H, Nu Binary selector, compositor gráfico, historial e incrustaciones. El diseño los conserva como dependencias de P1–P5 y prevé gates/QA independientes. Los futuros fixtures exactos, coordenadas/oráculos y fallos de inyección deben congelarse antes de outputs, sin derivarlos de la implementación.

No ejecuté parser/PDFium, compile, tests, instalación ni pruebas visuales; no modifiqué código ni contratos canónicos. QA WindowsTerminal/WezTerm real, native Unix/macOS, packaging/licencias y procedencia reproducible permanecen pendientes. No hay crítica accionable de diseño sobre este candidato; esto permite avanzar al refinamiento técnico/implementación del primer corte bajo los gates descritos, sin declarar producto PDF terminado.
