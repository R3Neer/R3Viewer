# D3 R2 — revisión adversaria focal independiente

2026-10-06. Propuesta raíz `Markdown-D3-routing-inline-plan-r2.md`, SHA256 `AC84B9A3CCE32F0CFCAB95A24BDA216E13B8E6A918BAFC05A1E5E3CC35B6641F`, rehash propio correcto. Antecedente privado R1 review SHA `AC259C135A4314154B794D5DB3F57C957A1862AB9DDF490A454564BE199475D6`; propuesta R1 y expectativas originales18casos no modificadas. Adversario distinto del autor. Sólo lectura/refinamiento contractual; sin edición compartida, build, runtime, procesos auxiliares ni nueva matriz A–C.

**Dictamen: SIN_CRITICAS_PENDIENTES en el diseño D3 R2 y cierre de D3-R01, D3-R02, D3-R03.** No equivale a aceptación de implementación, D2/D4 dependientes, terminal funcional, Unix, SLA, QA visual ni producto completo.

## D3-R01 cerrado — geometría y numeración

§6 sustituye la referencia inexistente a una opción inline actual y fija API InlineGeometry{columns:u16}. Captura una vez columns TTY, fallback80 ante error y redirect80 sin consulta/worker color; resize no modifica salida preparada ni renueva plazo. Width0 conserva clusters y saltos semánticos y usa tablas verticales; width positivo envuelve sólo entre clusters y conserva entero el cluster mayor que disponible. Prefijos/gutters/decor entran en finaloutputlimit, útil saturating_sub no fuerza truncamiento cuando queda0.

Line_numbers es ordinal streaming de lectura1..N, prefijo ASCII N | sin ancho derivado de total ni reflow global. CRLF incrementa una vez, SoftBreak espacio no incrementa; wrap incrementa, transición9→10 recalcula gutter antes del siguiente cluster y newline final no crea gutter vacío adicional. No se fabrican Sourceoffsets, Sourcecoverage para gutter ni numeración TUI global. La decisión nueva es concreta y comprobable con los oráculos0/1/80, 9→10, CRLF y newline final; no se presenta como API ya existente.

La aclaración menor también está resuelta: formato Markdown explícito sin --language selecciona gramática Markdown de Source aunque la extensión sea .json; el selector documental Markdown y StructureSelection None siguen decisiones independientes. --language sólo vence el resaltado Source y no gobierna fences. Se requieren tres aserciones separadas en pruebas, no una frase ambigua «noJSON».

## D3-R02 cerrado — SourcePreparationLimit

§7 reconoce que Source Arc íntegro no implica SourceModel completo presentable. Conserva máximos actuales visible256MiB y un millón de líneas, sin clon original ni elevación de límites. Tab/o prepara candidata sin destruir Documento/current/Anchor/historia; Ready válido hace commit, Limit/Failed conserva vista existente y diagnostica sin publicar prefijo ni afirmar transición. Causa determinista conocida no se reintenta en redraw, nueva preparación exige cambio de identidad o causa recuperable explícita.

La carga inicial sin Documento ni Source presentable tiene outcome PreparationFailed1 tras restore, conservando causas cerradas; q0/CtrlC130 registrados prevalecen ante fallo tardío. Source fallback inline permanece streaming con salida256MiB y no hereda el límite de líneas TUI. Autoridad del controller/preparer y Arc se retienen hasta cierre físico o Pending; conservar el Arc no se etiqueta como presentación completa exitosa. Esto cierra el contraejemplo lógico R1 sin inventar Source streaming TUI ni reducir capacidad de lectura del parser.

## D3-R03 cerrado — Loading pump y owner de preparación

§8 elimina preparación síncrona del loop/UI: instala guard+KeyboardReader+Loading, bombea teclado/preparación/render breve; hash, proyección, índices y ancho Source pertenecen a SourcePreparationOwner supervisado. Operaciones largas D2 iniciales también requieren owner/pump, no un constructor bloqueante precedido de reader. Una operación y un resultado candidato, canal acotado1, ownership de JoinHandle/Input/token/localstop, fallo Builder.spawn sin handle ficticio y commit sólo de Completion válido/identidad vigente son explícitos.

q se atiende como intención Normal0, activa stops locales y cancel del controller sin convertir el global cancel en CtrlC. CtrlC registra130 y los mismos stops; Ready tardío no inicia trabajo ni publica. Error lector tiene Failed1 y cancela owners. La preparación no tiene que concluir para restaurar terminal; guard se restaura explícitamente antes de la espera de reap, con lector detenido según contrato actual. Sólo join terminado acredita cierre de ese thread; panic queda como causa. Deadline cleanup2s vencido retiene owner completo en slot Pending acotado y bloquea nueva Source/hashprep hasta join físico, separado de PendingDocumentController. Un ticket/sendfailure no libera el slot ni certifica cierre Child/Job.

No promete que allocation sea interrumpible ni que timeout fuerce terminación del thread. Precisamente conserva autoridad si no termina. Arc alias no duplica allocation; resultados/copias/capacidades vivas sí se contabilizan y el preparer no construye dos Source visibles a la vez. Wrapper legacy Source ya preparado conserva semántica y la nueva ruta Markdown no lo exige antes de Loading.

## Condiciones de implementación y aceptación posterior

El cierre depende de implementar las reglas expresas, no sólo nombres de tipos. Revisar funciones nuevas from_shared/owner/Completion y su integración D2c para comprobar localstop/globalcancel, monotonic tokens, canales y capacidad old+new, guards y recepción/join antes del commit. La preadmisión/copia final inline debe respetar256MiB con capacidad efectiva y crecimiento antiguo+nuevo; la estructura Flow/coverage y cuotas D2 no se presumen implementadas. D4 seguirá una revisión separada para color opcional.

Oráculos nuevos están definidos antes de outputs: ancho/gutter/CRLF, SourceLanguage explícito, Tab SourceLimit conserva current/historia/Arc, fallo inicial doble1, barreras de hash/Sourceprep con q0/130, lateReady no Init/commit, Pending bloquea segundo prep y restore antes reap. Pruebas deterministas primero y ejecución funcional Windows sólo sobre bin congelado D3c. No repetir invariantes A–C o corpus gigante por selectores/ayuda; Source fallido no revoca sus gates históricos. El texto previo de §2 que promete fallback Fuente completa se interpreta bajo la excepción explícita de §7; §6–8 declaran sustituir promesas abiertas/conflictivas.

Fuentes reales contrastadas en R1 siguen siendo referencias de adaptación, no implementación ya equivalente: inline::render sin ancho, SafeTextModel constructor limitado y KeyboardReader que sólo latcha CtrlC. Ninguna de estas APIs recibe crédito automático de la revisión de este plan. No fallos nuevos concretos identificados en el alcance contractual R2.
