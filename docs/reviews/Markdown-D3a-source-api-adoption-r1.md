# Adopción de APIs Source compartidas de D3a

2026-10-06. Base0b95b1550313137e618cd385414b76e3315dcb5b. Corte coherente de las2 APIs Source del candidato D3a revisado: SafeTextModel.from_shared mantiene originalArc/proyección/límites y new delega; DocumentSource.original_handle comparte snapshot sin reabrir/reconstruir. Se copian sólo src/presentation.rs y src/document_source.rs byteexactos desde freeze15records6AFBCE5A55B5C38333740F132747EC387F312567507F6F9654BDC3B20B550E58.

[Revisión/aceptación D3a](Markdown-D3a-acceptance-r1.md) propia6PASS0.50s,14records34848E1B1D0A46ED9F504B41D187483C9972693B9121C3964A0EB13367EB5AC1 yexe05D237C5FB205D1723A69C613E708407D8C7C579D407818BE33D4A873656597D; La copia del adversario acredita535archivos del candidato, incluyendo nuevo módulo, no toda implementaciónD3. Autor5D3a+9selección+7Source PASS/fmt/Clippy; no se atribuyen al adversario.

Selector/FormatMarkdown/libmodule/syntaxdefault aceptados como candidato privado pero todavía NO se adoptan: CLI debe seguir sin anunciar Documento hasta routingD3c. Main/nu/TUI/inline/loading no cambian. No cierreD3/RV ni runtimeMarkdown enriquecido, gráficos/QA/Unix/distribución. La adopción de Source APIs permite coordinar D2c/D3 conservando una fuente común; las mismas políticas Source se mantienen.

## Verificación en el repositorio compartido

Las dos fuentes coinciden byte a byte con el candidato revisado (SHA-256: document_source.rs 3B6CB46510890DEEDF60A719F9CBD22B74C25A79C17DEC3365626CAB9B33F06E; presentation.rs 970D64DD9A1FA53A0126D096227371C49D120977FEA568354F3AB0D68F04AF0D).

Regresión Source: siete pruebas aprobadas, 0.82 s; cargo fmt --all -- --check y cargo clippy --offline --locked --workspace --all-targets -- -D warnings terminaron con código 0. Evidencia temporal: Markdown-D3a-shared-source-regression-r1.log y Markdown-D3a-shared-clippy-r1.log. El aviso de compatibilidad futura de proc-macro-error2 pertenece a una dependencia y no constituye un fallo de estos controles.

Se integran únicamente las dos APIs Source. Las cuatro modificaciones restantes del selector y CLI siguen en el candidato privado hasta conectar la ruta Documento en D3c. Estas verificaciones no acreditan QA visual ni soporte nativo de Linux/macOS.
