# XML Timeout: aceptación independiente focal R1

2026-10-10. SIN_CRITICAS_PENDIENTES en CodeAE+AE focal del candidato exacto. XT-P01 cerrado; ejecución propia de los nueve tests indicados, sin tests nuevos ni cambios a aceptación.

Bin físico bin-before-output/r3viewer-tests.exe SHA541A1D3E001FB8C6392A24A6BF8F37900A8D0D3ABA8C694BEE40B2483C080A13. MANIFEST-before-output.json SHA3AB7AED889B26D7D9C349D33AA827ADD312E2E8089E81D0D16AA475F093384B0;493/493 entradas recalculadas antes y después, cero discrepancias, exe intacto. Cwd absoluto del candidato. Comando exacto: bin-before-output/r3viewer-tests.exe xml_worker::tests --test-threads=1.

Resultado propio:9PASS/0FAIL/0ignored/510filtered, exit real0, duración interna0.01s. expired_validation_is_timeout_and_cancellation_keeps_priority y real_relation_sort_preserves_timeout_and_cancel_cause positivos, además de siete regresiones XML del filtro. Log SHAEB781980BA2133336FB5D9E1D36324908659FC758F0119936F40EDABE1B2331D. Sesión12803 recolectada con exit0; ningún proceso propio pendiente. No Cargo ni target compartido.

Revisión previa exacta de seis deltas sigue en REVIEW-CODE-independent-r1.md SHA7D6CC886C3CFDC063805FD08DD87A676F88F0645CAD21F5B43EE822B28845C18. Se confirma propagación Timeout independiente de Cancelled y prueba sort real con>=2relaciones. No se atribuyen los resultados de raíz a este revisor.

Este dictamen admite exclusivamente reparación de categoría y controles focales. No prueba causalidad de3695.62s/fallo paralelo, deadline2s/5s end-to-end, máximos materiales de arena/items, presión Job, QA visual o Unix ni cierre S03 completo. La integración canónica es tarea de raíz; aquí sólo se ejecutó candidato congelado.

---

# XML Timeout — revisión independiente del candidato exacto

Fecha2026-10-10. SIN_CRITICAS_PENDIENTES de revisión de código focal para los seis deltas examinados. XT-P01 cerrado. No ejecución propia de tests ni build: el harness de raíz estaba activo. No adopción, S03global, límites materiales, visual/Unix ni causa histórica acreditados por este dictamen.

## Identidad

Base declarada308ab023608bf34e8737362a1175aed43d756928; comparación física contra D:/Tools/R3Viewer/.tmp/s03-closure-2026-10-10/source-before-build. Candidato .tmp/s03-xml-timeout-2026-10-10/candidate. MANIFEST-before-output.json SHA3AB7AED889B26D7D9C349D33AA827ADD312E2E8089E81D0D16AA475F093384B0:493/493 bytes y SHA recalculados, cero discrepancias. Incluye492fuentes/recursos/configs y bin-before-output/r3viewer-tests.exe SHA541A1D3E001FB8C6392A24A6BF8F37900A8D0D3ABA8C694BEE40B2483C080A13. No ejecuté ese bin.

Deltas exactos observados:

- src/json_worker.rs SHA D79682F750713795AD7221304AA7E6BD8F04832C30F292E30094BCC4D9329F56
- src/structure_model.rs SHA 9E9CC1202284486D9672427C6F784513D3AA3ECF8AD035CE8D41DEE962BBAA22
- src/structure.rs SHA CECA316B5B640ACF0E4D3ADFE8AAC0574081F55CB65447260F7F80B808958ADE
- src/toml_worker.rs SHA 0551620CB475019233B2FE49E93C95F2D7F1E2B3AE901AA44DD0FA468C7AB5CD
- src/xml_worker.rs SHA B1BDC43609B23839E71B996758A8796084CD3A4A7165F0B50893254AAE35DFD8
- src/yaml_worker.rs SHA E1C7CCB2477082C0EEA6B185354799FDBA1F9227F069F24C5C38CFBAEA5D574B

## Código y criterios

ModelError añade Timeout y Display explícito. Es enum interno sin repr/wire; StructureFailure/codec/protocol/límites no cambian. Todos los mappers exhaustivos afectados añaden Timeout correctamente; Cancelled histórico conserva significado. structure::receive_sequence conserva el branch Cancelled y check original; nuevo Timeout entra mapper como WorkError::Failure(Timeout).

xml_worker::validation_error único distingue Cancelled, Timeout, Budget, Depth y restoInvalid. validate_source_until usa ese mapper para parse y checked_sort. Wrapper productivo mantiene deadline Instant::now()+5s; helper privado sólo recibe el reloj para prueba determinista. Ningún flag/config/env/IPC nuevo ni deadline reiniciado por el fix.

expired_validation_is_timeout_and_cancellation_keeps_priority fabrica modelo válido mediante project real con cancel=false, mantiene control ordinario exitoso y expira helper sin sleeps. Verifica ModelErrorTimeout y cadena worker FailureTimeout; callbackcancel=true con reloj vencido devuelve Cancelled.

real_relation_sort_preserves_timeout_and_cancel_cause obtiene relaciones desde el modelo validado `<r><a/><b/></r>` y exige len>=2. Usa checked_sort real, revierte las relaciones de entrada, ejercita sano/vencido/cancelprioritario y comprueba orden sano mediante los u32 internos, sin añadir Ord ni cambiar NodeId productivo. XT-P01 resuelto. Test no usa stub de sort ni expectativas derivadas del output. Compilación fallida previa por NodeId<= fue ajuste de assertion conforme la API existente, no corrección productiva de identidad.

## Alcance

Lectura y diff suficientes para aceptar mecanismo focal de propagación de causa y sus ensayos. Resultados del harness/Clippy/fmt/regresiones serán evidencia separada de raíz; este informe no los declara PASS ni reinterpreta intentos anteriores.

El viejo Timeout→Cancelled→WorkerExited queda corregido en estas dos rutas. Esto no demuestra por qué ocurrieron3695.62s o el fallo paralelo, no verifica deadlines extremos end-to-end y no reemplaza matrices S03/D3/D4 ni prueba de presión Job. Propuesta de materiales R1 permanece intacta para revisión por adversario.