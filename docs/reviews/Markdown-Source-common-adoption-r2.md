# Source Common R2: integración de biblioteca

2026-10-10. Delta Source R2 integrado sobre `9a7cc6e`, conservando la corrección B2 por bloque contenedor y su módulo de regresión. [Aceptación independiente](Markdown-Source-common-acceptance-r2.md): SIN_CRITICAS_PENDIENTES dentro de la biblioteca; SC-W01 cerrado en código. No cierra sesión/pump ni los RV de Markdown.

La Fuente comparte el modelo preparado, su contexto y el original sin clonar la proyección; los handles mantienen cargos y continuidad tras cancelación, Pending y join. La preparación del viewport usa aperturas, posición, selección y procedencia con el mismo contador de trabajo. Source Common512KiB aplica la [calibración explícita](../refinement/MARKDOWN-SOURCE-COMMON-WORK.md); Legacy256KiB y cuotas de memoria permanecen intactos.

## Procedencia y composición

Manifest `FC9070B90C7DF4787686B6975AD670976978A141A7AFB2D4793E5393661C74FF`; snapshot `9CA5A9EB8DC1FC970C7016AF8AF0240D0B7D88225242A144F74C7226EE88CF03`; delta `58299601ABA889B5B9356A8595F1AF679199D6421BFA1526A250AE6286116718`. Root verifica 822 comprobaciones de snapshot/producto/deltas y evidencia independiente. Los cuatro ejecutables propios retenidos se verificaron adicionalmente; ocho casos distintos aceptados con intentos y correcciones de arnés preservados, no una suite única inventada.

Diecinueve rutas Source; `lib.rs` combina los dos módulos nuevos con la regresión B2 canónica. Los demás archivos coinciden con el candidato. Los archivos base con diferencias CRLF/LF se comprobaron normalizados antes de modificar. `text_engine.rs` incorpora un bloque aditivo de 7778 bytes: retirarlo reproduce exactamente el archivo base; no se afirma archivo nuevo byteexacto al anterior. Cargo y lock no cambian. Fixtures sintéticos Source conservan bytes mediante atributo específico.

Root conserva el binario antes de ejecutar y obtiene 132 pruebas distintas PASS: Source15, checkpoints3, kernel19, vista34 (incluye metadata4), motor Legacy7, presentación Legacy7, owner B2 4, renderer13, búsqueda14 e inline16. Tres filtros iniciales seleccionaron cero tests y no cuentan como cobertura; los nombres reales se ejecutaron sobre el mismo binario. Formato y Clippy de workspace/todos los targets PASS. Evidencia en `.tmp/root-source-freeze-verification-2026-10-10`.

## Comprobación adicional de procesos

Tras el commit de integración, `cargo test --offline --locked --test document_pipeline --no-run` compiló el ejecutable de integración y el worker real del checkout. Se conservó el binario de pruebas antes de ejecutarlo: 12 pruebas PASS, incluyendo tablas, roles/procedencia, cancelación y casos Obsidian del parser. La ruta del worker es `CARGO_BIN_EXE_r3viewer`; este control es de integración del checkout, no un freeze independiente del worker ni QA de terminal. Log y recibo del binario en `COMPILE-process-R2.log`, `TEST-process-R2.log` y `PROCESS-BIN-before-runtime-R2.json` del directorio de evidencia root. No se volvieron a ejecutar suites ya aprobadas.

## Alcance pendiente

El despacho financiado común del inspector, Hash previo/Loading común, sesión/pump y frame nativo con chrome todavía requieren integración y aceptación. El recibo de trabajo Source debe exportarse una sola vez a la operación completa, no crear una segunda cuota. No acredita Markdown interactivo visible, history/routing, gráficos, QA visual ni ejecución Unix.
