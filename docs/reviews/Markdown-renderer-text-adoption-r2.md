# Adopción del primer renderer textual Markdown R2

2026-10-09. Hito acotado previo a la pausa solicitada por el usuario. Base canónica e83c93449c57d9f482ea2f443015cd60f7c2aefe; código base 0cb0fe65363dae463d739502d5d3058cc740909c.

## Alcance incorporado

Owner Fixed de Ratatui con reserva fallible y cargos reales; almacenamiento compacto de glifos; renderer de texto y estilos de spans sobre DocumentView real. El backend Crossterm conserva su escritura nativa, con limpieza de continuaciones anterior a glifos anchos. Cancelación anterior a IO conserva el espejo del frame visible y su cargo; errores parciales del backend invalidan el owner, sin prometer rollback visual.

Es una biblioteca interna: no se conecta todavía al pump ni al comando interactivo. Imágenes y demás recursos se rechazan antes de IO en este corte. Roles y decoraciones de bloques, búsqueda visible, navegación interactiva, fences y gráficos siguen pendientes. RV-301 permanece abierto.

## Integridad y aceptación

[Revisión y aceptación independiente](Markdown-renderer-text-acceptance-r2.md), con cuatro grupos propios PASS y disposición sin críticas pendientes para este corte. Oráculos, fuentes de pruebas y logs propios preservados fuera Git.

Raíz verificó 696 registros del producto y 83 del delta, todos byteexactos; los nueve modificados coincidían con HEAD en su base y los 74 añadidos no existían. Se copiaron sólo esos 83 archivos y se verificó nuevamente cada hash; no se sobrescribieron los documentos posteriores. SourceLegacy text_engine.rs permanece intacto. El marcador de caché .cargo-ok queda fuera de la vendorización. Procedencia, checksum, VCS, licencia y hashes del parche en vendor/ratatui-core/R3-DERIVATION.json.

Freeze final privado: .tmp/resume-renderer-2026-10-09. MANIFEST-product-freeze-r2.json SHA256 4627B04BE8594C609DA9C77AA9C2DB72FFDE283EBC1AEA87502823E1D7750012; MANIFEST-delta-freeze-r2.json C6C6687BB259827C456E2DF65B4621E3278FDBC0CC81FAAF06B08EFE3C220B85; EVIDENCE-freeze-r2.json 19D26781AD660F0A2642410011B098A4760141E3E9896EE6AE31EF2D68C2F45C; CANDIDATE-delta-freeze-r2.zip D9B3E95874F64358B57211601FB484C23E7344165F7CC1890D3F7F15A1E1AA0D. Intentos y fallos anteriores intactos.

## Perfil y límites

[Delta DocumentWork8MiB](../refinement/MARKDOWN-RENDERER-DOCUMENTWORK8M.md) y [revisión de diseño](Markdown-renderer-DocumentWork8M-plan.md). El intento 4MiB falló correctamente en 120x40 y se conserva; no se rebajaron cargos. Las fixtures densas de 80x24 y 120x40 consumen 2555300 y 7063279 unidades de trabajo para layout+DRAW, frente al techo 8388608. Las cifras no son RSS ni promesa de latencia o de soporte de cualquier documento.

Window128KiB, Frame256KiB, Consumer1MiB y Framework2MiB. El perfil de trabajo documental cambia explícitamente; dos tests históricos usan ahora bounded262144 para mantener sus anteriores oráculos de rechazo y posición, sin suprimirlos. La preparación de geometría conserva fase separada, no incluida en DRAW. La reserva de metadata del writer no certifica asignaciones arbitrarias de cualquier backend.

## Verificación de composición

En el árbol canónico, Windows/Rust1.99.0, cargo offline locked: renderer11, owner/storage8, flow16 y view16 (sin repetir b103 gigante), total51 PASS. cargo fmt --all -- --check y cargo clippy --offline --locked --workspace --all-targets -- -D warnings: exit0. Hay un aviso preexistente de compatibilidad futura en proc-macro-error2; no error del check. Antes de copiar, raíz repitió 19 pruebas focales sobre el freeze privado. El autor conserva además regresiones document_text31, roles2 y Ref5, entre otras, identificadas en su manifiesto de evidencia; no se atribuyen a la ejecución de raíz.

Captura funcional de comandos Crossterm; TERM=xterm sólo en el proceso hijo. No QA visual Windows Terminal/WezTerm, ConPTY de este renderer ni ejecución nativa macOS/Linux. Sin publicación, remoto ni cambios de perfiles personales.

Proveniencia final AE: informe SHA256 C96924BC16127B98783BDD28453B917B396B8F8B3E91798741DE6D77FE4BC983 y readback00970A4E7DE37EC583A567355FE858E755E82A45F83E50194196AAE314970BD2 verificados por raíz. Target de compilación compartido y mutable: el hash observado después no identifica el ejecutable de los cuatro tests adversarios; se conservan su log, fuentes propias y prefijo productivo exacto. Corrección sólo documental, sin repetir pruebas.

La copia canónica del informe elimina únicamente líneas vacías finales para satisfacer git diff --check; el original privado con su hash permanece intacto.
