# MD-CALLOUT-EMPTY01 — revisión del plan de corrección R3

2026-10-06. Revisor root; autor graphics_gate_experiment. Plan Markdown-callout-empty-correction-plan-r1.md SHA93DA45E19D29CD8A819D6D05F193AD534D963140C5B4C3CD9F409AE8EC96985C. Base UNC R2 congelada3BB930B7. Sólo revisión de diseño, no código ni aceptación.

Veredicto SIN_CRITICAS_PENDIENTES en la corrección acotada. Crear TitleDomain aunque no tenga Runs cuando los Spans del encabezado requieren owner preserva el EmptySpan/enlace de contrato Wire72. Retener ParagraphDomain del cuerpo cuando tenga Span propio pero cero Runs es la simetría necesaria; no fabricar texto ni perder el enlace. Root contrastó las ramas callout y remap de compact en R1 con la regla de partición actual y sus contadores. La conservación de Quote ante cruce/marker overlap sigue obligatoria.

El guard antes de remap debe impedir indexar NONE/fuera de rango, pero no reemplaza crear y conservar owners reales de entradas válidas. Necesidad y remap usan exactamente la misma clasificación; checked_sub precede mutación, Spans vacíos conservan rangos0..0 y Links label_count0. La eliminación de párrafo vacío sin semántica continúa, y Origin original válido se conserva cuando no existe bodyrun para recortarlo. Dominios nuevos consumen el mismo ArenaVec/ledger64MiB y pases cancelables, sin cuotas adicionales.

Repro independiente original y controles fuente/upstream permanecen fijos. Strong aparentemente vacío sólo se prueba como evento si la dependencia lo emite; un candidato wire manual se distingue de parsing real. Cases título/cuerpo vacíos, headerOnly, CRLF/EOF, Quote normal y crossing tienen que comprobar owners/Source/provenance, no sólo ausencia de panic. Mutantes rechazan dominios/rangos/owners falsos. No modificar validator/wire, no aceptar errores como éxito ni dropSpan para green.

Autorizado implementar R3 privada tras UNC R2, preservando ambos freezes. Entrega y aceptación independientes pendientes; MD-CALLOUT-EMPTY01 sigue abierto hasta evidencia del código exacto. No cierre A–C general, D, RV, QA visual o Unix; UNC R2 todavía en revisión de delta/AE.
