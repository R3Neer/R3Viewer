# Renderer textual Markdown: aceptación adversaria final R2

2026-10-09. Dictamen: **SIN_CRITICAS_PENDIENTES dentro del primer renderer textual/spanstyles y owner compuesto revisado**. Cuatro controles propios PASS. Autoría, revisión y aceptación son independientes. Autor/root ejecutaron además sus controles focales; no se atribuyen como nuestros.

## Candidato exacto

Base código0cb0fe65363dae463d739502d5d3058cc740909c. Commit canónico de diseño e83c934 conserva el mismo código base; integración sólo del delta, sin sustituir documentos posteriores.

- MANIFEST-product-freeze-r2.json:4627B04BE8594C609DA9C77AA9C2DB72FFDE283EBC1AEA87502823E1D7750012.
- MANIFEST-delta-freeze-r2.json:C6C6687BB259827C456E2DF65B4621E3278FDBC0CC81FAAF06B08EFE3C220B85.
- EVIDENCE-freeze-r2.json:19D26781AD660F0A2642410011B098A4760141E3E9896EE6AE31EF2D68C2F45C.
- CANDIDATE-delta-freeze-r2.zip:D9B3E95874F64358B57211601FB484C23E7344165F7CC1890D3F7F15A1E1AA0D.

696 registros productivos verificados antes/después: cero diferencias. Logs y artefactos preservados del manifiesto de evidencia verificados, incluido fallo Work4MiB inmutable. Root verificó además los83 registros de delta. Copia adversaria enumerada desde el manifiesto; las695 fuentes sin mutación permanecen idénticas y document_renderer.rs conserva prefijo productivo byteexacto: sólo se añadió una inclusión cfg-test al final y un archivo de tests propios. READBACK-AE-final-r2.json documenta ambos controles.

## Ejecución propia y oráculos

Oráculos previos EXPECTED-style-wide-before-output-r1.json SHA6816635A3F24AF3563F0086503FEAB7320C2498DBF6B400417E14157303498C2 y EXPECTED-cancel-before-output-r1.json SHA1A0B5C089F73369B55E7DC403A1C147332F681155600A01BDDDB5BB5360FBF38 intactos. Archivo propio product/src/adversary_renderer_tests.rs SHA68F3830C2185D1CFA379D9C3DFFCFE1C5D4AC61679E2414FB0819A94AFB7A967.

Comando: cargo test --offline --locked --manifest-path <copia adversaria>/Cargo.toml --lib document_renderer::adversary_tests:: -- --test-threads=1 --nocapture. Rust1.99 Windows, TERM=xterm sólo en proceso. Reutilizada caché de dependencias del target del autor libre de procesos; la librería se compiló desde nuestra copia y filtro ejecutó exclusivamente cuatro tests propios. Build36.03s; ejecución0.31s; exit0,4PASS,414 filtrados. No se ejecutó ninguna batería global ni caso32MiB nuevo.

Provenance del ejecutable: se reutilizó un target mutable compartido. No se preservó una copia independiente del ejecutable de los cuatro controles; por ello no se declara un hash binario congelado ligado a esa ejecución. La observación F0B20B7D posterior a la ejecución y el hash distinto del readback posterior no tienen esa autoridad: raíz recompiló después la composición canónica en el mismo target. Log OWN-AE-attempt1.log SHA789DCA517B25743B97E16814A79A0D47657DB27CB93C1239CA00F2DAB9A955DD. Captura y vector de updates son memoria privada del harness; no se cuentan como writer de producto ni justifican cuota8192.

1. Fuente propia4920B, parser/validador real, sourceArc original, factory start_document y take_validated después de join físico.80x24 escribe1440X en24filas: filas alternas80/40;120x40 escribe4800X en40filas de120. Cada coordenada semántica aparece exactamente una vez. Backend observado intercepta sólo tuples y los entrega al CrosstermBackend nativo; captura ANSI contiene1440/4800X exactos. Segundo frame estable no vuelve a imprimir X. Work2555300 y7063279, respectivamente; límites exactos admiten y exact−1 rechazan.4MiB conserva rechazo120x40 antes de IO.
2. Fuentes pequeñas a**U+0301**, a*U+0301*, **a**U+0301, tachado y enlace: símbolo indivisible completo y modifiers efectivos en las celdas entregadas al backend. Énfasis/strong/strike/link coinciden con el mapping cerrado; code y link usan paleta Accent. Colournever emite celdas sin modifiers y colorReset. No golden de contenido tomado del candidato.
3. Transiciones ab->VS16 yCJK, wide con estilo->x: preclear de columna1 antes de baseglyph y blank posterior al volver a narrow. Modelo propio conserva el viejo b hasta recibir un clear explícito, evitando presumir que imprimir wide lo borra. Caso ab->candidatoVS16 cancelado en último callback->x no emite durante cancelación, conserva PreviousFrame y receipt heap, y luego limpia columna1. El test es funcional de tuples y escritura nativa; no QA visual de terminal real.
4. Clusters propios25B y33B conservan heap efectivo32/33B en receipt físico y ConsumerBudget; Drop devuelve el consumo a baseline. Cancel de prepare devuelve Cancelled, no Work, sin cambios de presupuesto ni stdout. Mapas internos de admisión durante preflight revisados estáticamente: todos preservan causa Cancelled. No se afirma inyección temporal determinista dentro de cada lectura sin un hook; final-preIO sí está probado mediante callback determinista.

## Revisión estática y cierre de hallazgos

MD-RENDER-R01 cerrado: iterator prestado amplía diff_iter con continuations blank/stylebase ANTES de glyph; no cae en is_skip del diff. last_known_cursor_pos observa iterator ampliado. No width/resegmentación, Vec de diff ni clone de grid. Flujo hide_cursor/swap/reset/backend.flush/count y API Legacy original preservados. Backend error después de IO/swap marca poison; no promete terminal físicamente atómico.

MD-RENDER-R02 cerrado: mapping modifiers separado de paleta, ORstyles de cluster y colournever probado sobre celdas reales. No atribuye roles estructurales/decoraciones posteriores a este corte.

RenderPlan/permit tienen campos privados y préstamo inmutable a ventana; noClone ni mutable escape. PreviousFrame opaco sólo procede del owner. Cada draw toma previous real y carga ventanas/payload/copies old+new, metadata y heap; símbolo nuevo Consumer reservado antes de copia. Framework conserva capacities y oldsymbol+newsymbol hasta readback/Drop. Cancel/stale final resetea sólo current, reteniendo previous y su Charge. Error de readback/IO aplica cleanup y poison apropiados.

Packing utiliza PackedGlyph20B y GlyphClass40B cobrados por capacidades; reconstrucción Glyph por valor, sin Vec por frame. Preserva first/local/projection/styles/links/relations. Reserva Window128KiB y Frame256KiB antes allocation; overflow de clase conserva fallback/negativos. Cambios de tests revisados adaptan acceso a glyphs y nuevo límite explícito sin borrar fallos históricos. Binding visible se actualiza sólo al commit del nuevo viewport.

DocumentWork8MiB usa Work::permit existente, techo cerrado y SAMEWork de layout+preflight/materialización/readbacks/diff/reset/formatter. Guard conserva los costos que hicieron fallar4MiB. Source text_engine es byteexacto E0D3F08E7A386FD66EC39D941C9355E0997C47BE70CA1660EA2E69EB9EF59B3C; no elevar SourceWork ni memoria Window/Frame/Consumer. Patch Cargo añade sólo ratatui-core vendorizado fijado; derivación/version/checksum/VCS/licencia documentados, upstream diff/apply Legacy intactos y cache.cargo-ok excluida.

## Disposición y límites de pausa

Aceptable integrar este delta mínimo y crear commit local coherente, con verificaciones pertinentes de la combinación. No adoptar copias de docs ni WIP completos. No críticas accionables pendientes en este alcance/candidato exacto.

Entrega una biblioteca privada de renderer real textual/spanstyles con harness nativo controlado en Windows. No cambia selector interactivo público ni acredita b2 completo, pump/D2c, Source↔Documento funcional, recursos/tabla/roles completos, fences color, gráficos, búsquedas/history/actions, ConPTY, StdoutLock/TerminalGuard de sesión ni validación final writer metadata8192. Estos quedan pendientes explícitos; la captura propia no los sustituye. Tampoco QA visual WindowsTerminal/WezTerm, Unix, distribución, publicación o RV301/producto completo. Al cerrar hito/commit solicitado, pausar y dejar sin trabajo/procesos activos.

Corrección de provenance: sólo documentación, sin nueva ejecución. Los logs cuatro PASS, el source propio, la inclusión cfg-test y el prefijo productivo byteexacto sí están preservados y verificados. Los informes previos a esta corrección se conservan identificados como before-provenance-correction; no usar sus hashes binarios como identidad del runtime propio.
