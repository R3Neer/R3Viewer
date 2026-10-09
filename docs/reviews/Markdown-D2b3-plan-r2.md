# Revisión adversaria b3 R2

2026-10-09. Revisor independiente markdown_actions_adversary; autor markdown_actions_specs. Revisión R1 preservada sin cambios.

Candidato exacto: `.tmp/markdown-b3-specs-2026-10-09/CONTRATO-B3-R2.md`, SHA256 `0D1AC65E7C2C6B3CD1551EC1BCE94B92371824D2DF720600C08966782D6A3448`, verificado. Base código `353418c19f2152614d5650d605aa05df72a1cbca`; HEAD documental `b3c294e8c89f414ab3b5cf503162e17be03dc52f`. Git read-only con ejecución autorizada confirma HEAD, estado limpio y diff de src vacío entre ambas bases. El fallo sandbox de git status de R1 permanece documentado; no se retocó configuración.

Alcance: revisión de diseño completa de búsqueda, inspector/acciones, headings, transición Source, historial y autoridad; cierre de los dos hallazgos R1 y comprobación de nuevas incoherencias relevantes. Misma evidencia canónica y APIs reales de R1, con readback focal de State::prepared_role, SafeTextModel::visible_range_with y PreparationAuthority::take_validated. Ninguna implementación/test nuevo, edición de Clases, publicación ni perfil. No aprobación implícita de b2/getters, pump, terminal, graphics o runtime.

## B3-R01 — RESUELTA

R2 §1 discrimina SourceHit absoluto con identidad/hash/generation/request/queryrevision y rango UTF8, sin exigir Block/Run/Domain ni fabricar RawAnchor. Matcher SourceOriginal compara el original; la proyección sólo prepara coordenadas de selección posteriormente mediante el SafeTextModel real. R2 cierra la secuencia lazy: búsqueda termina y join físico, luego SourceTransition en la misma autoridad, SourceReady íntegro + selección preparada + comprobación de key antes del único commit retornable. Modelo Ready retenido evita duplicarlo. SourceFoundPending conserva el hit nuevo aparte de la lectura y el hit actuales; Limit/Failed no se convierte en NoMatch, cancel/stale/exit impide transición. Esto encaja con las APIs prestadas y el owner único existentes; no requiere mapear Source a un Run.

Los ejemplos SourceOriginal `secret` original31..37 en `![fox](https://example.invalid/secret "caption")` y `**` en `**fox**` (0..2 y5..7) fijan los resultados raw antes de outputs. La aceptación incluye Sourceprep fallida y conservación del Documento. La crítica R1 queda cerrada en diseño.

## B3-R02 — RESUELTA

R2 §1/§2 dispone walker semántico sobre Items para construir páginas del snapshot, incluidos headings bajo folds, sin expandir Flow para enumerar. HeadingTarget contiene BlockID acreditado; FlowLocation es cache derivada opcional. La confirmación reutiliza expansión conjunta de ancestros y locate_block en el Flow candidato, seguido de State::prepared_role o anchor inicial acreditado, ventana y commit único. `prepared_role` real consume precisamente locate_block del Flow suministrado. Visible/no-op reutiliza Flow; fallo deja HeadingPositionPending y OLD, cancel/stale descarta. No usa ordinal del Flow anterior ni publica expansión exterior intermedia.

El ejemplo OUT/IN colapsados con `# Destino` fija inclusión sin expansión, una FlowIdentity al seleccionar, un retorno, cuota/cancel/stale y retry explícito. La crítica R1 queda cerrada en diseño.

## Coherencia restante y límites

No nuevas críticas accionables. Se conservan búsqueda completa streaming/overlap/cursor, dominios separados, diferencias entity/control/Source, selección paginada y provenance acreditada, límite128/129 transaccional, cargo old+new y cierre físico antes de commit. SourceFoundPending es preparación previa a la transición; su Ready realiza un único retorno, sin sumar otra entrada por una publicación pending intermedia. DocumentKey/revision y oldFlow acreditan stale sin expandir Binding genérico. Las decisiones ya declaradas de almacenamiento único de history, retención/adopción de PreparedSourceModel y getters b2 deben concretarse antes código, como dependencias explícitas; no se convierten aquí en aprobación de APIs inexistentes.

## Dictamen

SIN_CRITICAS_PENDIENTES de diseño para el candidato R2 exacto y este alcance. B3-R01/B3-R02 cerradas. Permite concretar e implementar la biblioteca b3 conforme sus dependencias; la entrega combinada requerirá aceptación y adversario independientes. Este dictamen no acredita código/runtime, sesión usable, QA visual, compatibilidad ni producto completo.
