# Inspector, procedencia y admisión física: adopción R2

2026-10-10. Base canónica antes de integrar: `af3d3d823062ec5cc56c9a3e5c761e8bd29f503a`; base de código del freeze: `5d8cd6793567a9a6cf6ae9e3cace5d490e1c2471`. Los cambios intermedios eran documentación. Se incorporan 18 rutas Rust y ocho fixtures sintéticas, con una regla específica de `.gitattributes`; README, backlog y este registro conservan el estado actual. Sin contenido privado, perfiles ni publicación.

## Cambio aceptado

La biblioteca inspecciona unidades reales desde Hit, celda, posición documental y State, conservando owner/campo, RefID, byte/parte, relación original y progreso Table At/Done. Las asociaciones y enlaces se paginan64+1; sólo el agotamiento real permite una relación de fuente completa. Samples raw/decoded permanecen separados y acotados. SourceAction es un descriptor validado e inerte; su ejecución corresponde a la futura sesión.

SharedHit retiene la asignación y proveedor originales. ProviderGate admite un máximo físico de dos proveedores scoped antes de cada factory, incluidos temporales. Weak conserva un token pequeño financiado hasta purge; el valor del proveedor se destruye antes del permiso. Búsqueda hacia atrás conserva descriptor entre secuencias y reconstruye al ganador con admisión real; Busy/cuota deja FoundPending con identidad acreditada. Alias, páginas y retirement mantienen los cargos hasta el último Drop. D1/core/INLINE históricos conservan sus contratos y no reciben una nueva cuota global.

El adaptador usa la PreparationAuthority existente financiada con sumas reales Viewport/Search/Inspect/RetireAndInspect. Kernel, SourceR2, modelo, ConsumerBudget y text_engine conservan sus fundamentos. No es todavía el dispatcher de producto.

## Revisión iterativa

CGR1-W01 rechazó R1: el cuerpo de captura/proof tenía cargo Consumer correcto, pero Work incompleto. R2 admite el cuerpo restante antes de construir: Scope144+seed112=256 para hit/view/cursor, proof64; alias HitLease32 se cuenta una sola vez. Consumer seed288/proof64, Consumer1MiB, Work8MiB, query4096, página64, sample4096 y gate2 permanecen intactos. La corrección sólo cambia `document_inspection_preparation.rs` y añade `document_inspection_work_tests.rs`; los otros789 archivos del producto R1 son byteexactos. Rust canónico se normaliza a LF; fixtures permanecen byteexactas, también en el índice Git.

[Dictamen independiente final](Markdown-inspector-gate-acceptance-r2.md): SIN_CRITICAS_PENDIENTES para el freeze biblioteca. Las30 familias originales se acreditan con runtime y CodeAE explícitos:25 controles propios (3 modelos y22 escenarios conductuales/preparación),39 secundarios R2 y94 secundarios R1, en rondas separadas. Los37 controles del autor son evidencia distinta. Rechazos R1 y tres correcciones de harness permanecen privados, sin reinterpretarlos como aprobación anterior.

## Composición canónica

Build `cargo test --lib --offline --locked --no-run`: PASS. Binario copiado/hash antes de runtime y verificado igual después: `F196A395CEB49278F9F5C3F839DB787368F433F0DFDEB61BA44DAAFF755F6274`. Veintiún filtros no vacíos ejecutan219 pruebas distintas; no se cuenta una prueba varias veces por selector ni un filtro vacío. Fmt y Clippy workspace/all-targets `-D warnings`: PASS. Aviso upstream de futura incompatibilidad de proc-macro-error2 separado del resultado actual.

| Filtro | PASS |
|---|---:|
| cgr1_w01 | 6 |
| document_inspector::tests:: | 17 |
| document_provider_gate_tests:: | 13 |
| boxed_search_completion_exact | 1 |
| document_inspector_fixtures:: | 3 |
| document_search::tests:: | 14 |
| document_view::tests:: | 30 |
| document_view::kernel_metadata_tests:: | 4 |
| document_renderer::tests:: | 14 |
| document_resource_owner_tests:: | 4 |
| document_roles::tests:: | 2 |
| document_text::tests:: | 31 |
| document_text::ref_tests:: | 6 |
| document_inline::tests:: | 16 |
| document_terminal::tests:: | 5 |
| document_terminal_storage_tests:: | 3 |
| source_shared::tests:: | 14 |
| preparation_owner::tests:: | 19 |
| presentation::tests:: | 7 |
| presentation::checkpoint_metered_tests:: | 3 |
| text_engine:: | 7 |

El recolector inicial de nombres exigía `test…ok` en una sola línea; cinco pruebas de DocumentView imprimían diagnósticos entre ambos. La ejecución30PASS quedó preservada; se corrigió únicamente la lectura del formato libtest, sin repetir esos siete filtros ni modificar expectativas/tests. Los nombres de inicio y el resumen completo exitoso permiten contar correctamente las219 pruebas.

Integridad del freeze R2:937 comprobaciones de hashes, bytes, snapshot completo791archivos, parent R1 y ambos deltas. Se verifican18Rust+8fixtures, aunque el manifiesto agrupa las26rutas como sources y fixtureCount0 es metadato de esa agrupación. Veredicto/binario propios y14registros de evidencia adicionales contrastados. Ajustes de esquema del verificador (artifacts en R2, directorio vacío en ZIP) se conservan privados y no modifican el candidato.

Evidencia local privada: `.tmp/root-inspector-freeze-2026-10-10/VERIFICATION-R2.json`, `ADOPTION-before-build-R2.json`, `CANONICAL-BIN-R2-before-runtime.json`, `CANONICAL-RESULTS-R2.json`, logs build/fmt/Clippy y notas de harness. No se afirma ejecución de toda la biblioteca,32M, ConPTY documental, QA visual ni Unix.

## Identidades y alcance restante

- Manifest R2: `98623E824CE794B60CC63C9097B772F60D9CAB63FFD7998770569741BDEE453C`.
- Seal: `EE4E71E00C18FAF34AE628E6663F4BA2DF9EB04CBF0D247D22835A93CE1975CA`.
- Snapshot: `27BC4F084B25BBD6801923F91C83FA3AE60FAA5CA2573DF2AFE6E9F4CAF91BB3`.
- Delta base5d→R2: `F8E8FA6F7B82AA1E322534808C9ACC0EF81F17E3AF54512EC4E460D80FA7DF1D`.
- Overlay R1→R2: `43812031AA2B0B89DDB216E763172E8C112AA5E62E002A74EC666A49473DCCA7`.
- Informe propio final: `A8E97B60E0959E644CE02AD40E63B1BECFFEEC424D4713BE3BC8049522B15D0D`.

La biblioteca registra Work reales por preparación, gate/D1/query/find con perfiles separados; aún no exporta un recibo CPU total de operación. D2c debe componer APIs metered con el mismo Work real, sin sumar allowances ni reiniciar por fase. Loading/Hash financiado, frame completo, publicación conjunta de pane/historial, SourceReady, run_markdown y su prueba ConPTY siguen en implementación privada. D3c routing y D4 fences siguen obligatorios después. No cierre de RV-301/306, gráficos, Obsidian, HTML, PDF, distribución ni QA por este incremento.
