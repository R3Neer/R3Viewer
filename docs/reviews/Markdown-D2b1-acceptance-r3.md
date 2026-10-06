# Markdown D2b1 R3 — aceptación independiente focal y disposición de biblioteca

Dictamen: **SIN CRÍTICAS PENDIENTES dentro del corte b1 biblioteca revisado**. MD-B1-ANCHOR01 y MD-B1-C1-PROGRESS01 resueltos por reproducciones originales. R1/R2 y sus FAIL quedan históricos intactos. No implica aprobación de renderer/pump/CLI/b2/b3/ConPTY/Unix, combinación con RefID/roles o gráficos/Syntect todavía sin conectar.

## Identidad/método

Freeze MANIFEST444363CFD182D478AB7C8965D7E048A90AB3B665F73C980898FE804160B9D571,624 registros y583 archivos repo. Readback propio antes/después cero diferencias. Delta respecto R2: cinco módulos productivos flow/layout/navigation/text/view y tests autor. Deltas completos archivados y leídos; otras577 fuentes exactas. Funciones candidatas byteexactas; únicamente sufijo cfgtest54B en document_view.rs y harness propio. Copia/target nuevos, build offline/locked privado escalado terminal0 2m05s, sesión3209 completada. Runtime propio sandbox Windows; bin031C1B6C01AE735F55DBCF7BBA714561FDB8EDFFA108BF41626B4E0D092E99D0. Sin helpers OS ni rescates, sesiones activas0.

Preoutputs originales copiados sin mutación y suplemento R3-ADDENDUM-before-output.json FC062B7803082EFA70BA186E6B76A33E30CF00E1C2E2ED77B0E2BBFF5D04CEAB previo al build/runtime. Frames/partes del suplemento se deducen de la proyección compartida de8 caracteres y geometry4/height1, no goldens del autor. Cuerpos completos de los dos tests críticos originales byteexactos: comprobados explícitamente en READBACK-after; no cambiar criterio de rechazo o progreso. HARNESS-original-parent-r2.rs conserva fichero original. Única adaptación mecánica del control HardBreak previo: next.raw.run/byte y comparar next.raw porque next pasa a LayoutAnchor; mismas expectativas, archivada antes de runtime. No fallos de build/runtime nuevos; aviso Python ubicación preservado como diagnóstico herramienta.

## Evidencia funcional propia

TEST-attempt1-newparts: dos grupos PASS0.02s.

1. Source literal A+U+0085+Z, geometry4x1. Inicial A\u{; siguiente raw(byte1)/part3. Line+1 muestra0085/currentpart3 y nextmismoRaw/part7. Resize3 muestra008 y Resize4 muestra0085, ambos conservan topLayoutAnchorpart3. Reference geometrywidth0 y quota1000x1000 conservan esa ancla; volver4 reanuda0085. Glyphs de esa ventana apuntan local físico1 y ControlProjection; jamás projectedpart como rawbyte. Line+1 llega}Z/currentpart7; historial0.
2. Veintiún negativos (7casos×3geometrías normal/zero/quota): C1part8/255, rawUTF8interiorC1, ASCIIpart1, RoleNONEpart1, HardBreakpart1, Resourcepart1. Cada intento Continue rechazado y currentLayoutAnchor previo íntegro. Validación debe suceder antes de Reference, no éxito por geometry/quota.

Diez controles previos repetidos por cambios reales, un test exact por log y todos PASS:

- Original ANCHOR01: abc rawbyte4, width40/0 Err(Stale), currentrawbyte0 conserva; expectativa original intacta.
- Original C1-PROGRESS01: secuenciaA\u{→0085→}Z, alcanza Z en dos avances y mantiene rawbyte1 separadamente de partes3/7. Original test sólo pedía progreso finito a Z; no se relajó después de FAILR2.
- MatrizR2 negativa UTF8/BlockRun/Role/Break/Resource/ALTResourceDomain noBody para normal/zero/quota.
- R2 validReference/rangoUTF8/constructor State y HardBreak canonical Bbyte0.
- Window exact/exact-1 y agregación old+new262144/+1, usando sizeof actuales de TextWindow/Glyph/Row, Reference Window real.
- Historial400 movimientos ordinarios sin gasto,128 saltos reales, sameanchor gratis,129reject preserva current; State creció pero cargos por tamaño actual siguen válidos.
- Fold/ResizeFlowidentity, stale/fullConsumer conservan current; Line±1 fila preparada; crossrunUnicode/estilos/controlprojection y límitesWork/checkpoint/window.

REGRESSION-RESULTS contiene los diez outputs exactos; cada log confirma running1/passed1, no filtro cero. Total12 grupos propios PASS en esta ejecución, sin adoptar17 autor como evidencia propia ni afirmar cobertura de los20 criterios generales originales. No repetir dc04_32m/factorRef/megacorpus/KernelSourceJob sin cambios.

## Revisión estática del delta completo

RawAnchor físico no cambia; LayoutAnchor añade u8projection_part separada. State incluye parte en igualdad; constructors Home/End/Role/ordinaryTargetpart0, prepared_visual validaRunByte+parte antes de Flowlocation/State. Continue en Operation/DocumentKey/DI/current/next/row conserva la parte: Line positivo desde window se transforma en Continue, Resize usa LayoutAnchorState y remap no la desecha. Factory OwnedPreparation y binding/physicaljoin no cambiaron.

prepare valida RawAnchor actual y parte antes de WindowBuilder/profile/Reference. RoleNONE no permite parte distinta0. validate_visual_part sólo admite nonzero para scalar real C1 en Run textual, con source-local UTF8boundary y part<8/UTF8boundary; syntheticBreak/Resource/noC1/EOF rechazan. Part0 conserva rutas anteriores y cancel admission. No projectionoffset convertido a Source/code coordinate.

SegmentedText::locate_visual añade parte validada al offsetproyectado interno de locate_raw; Point.position.part/GraphemeCursor existentes continúan. visual_anchor_with_work conserva raw normalizado y u8part, comprobando identidad interna mediante raw_anchor y validando el scalar. sequence usa esa posición para preparar y guarda next/row/glyphpart; shift_line toma y vuelve a Stateprepared_visual; cruce entre eventos Flow vuelve a part0. Canonicalización HardBreak de R2 permanece intacta. No nuevo Segmentado/Reopen/textcopy ni prueba fabricada.

Metadatos/State/history/Window/DI ya reservan sizeof de tipos reales, capacities efectivas mediante ChargedVec y mismos Consumer/Work/slots. Cambiar layout de estructuras no reutiliza tamaños constantes de R2; control exactAdmission acredita ruta actual, no RSS/framebackend. No renovación/reembolso por partevisual; stale/cancel/quota mantienen rutas existentes. Source/controller/parser/wire/pools/preparationOwner son577fuentes sin delta; gatesR1/R2 anteriores no afectados se mantienen por identidad, no se reenuncian como runtimeR3.

## Disposición y pendientes reales

Se puede adoptar este corte biblioteca con los controles indicados y la evidencia anterior no afectada. Cumple alcance anunciado de FlowHandle/typed Source|Document authority/viewport-fold-preparation/windowUnicode-controls/Line-Resize/Reference-admission, sin los dos defectos críticos anteriores. No afirmar b2renderer completo, navegación responsive universal, chooser/search/backb3, main/CLI/integration routing, SourceFallbackUI, gráfica, códigos Syntect conectados o Unix/ConPTY. Combinación con otros factores exige preservar sus fuentes/APIs y control correspondiente; este freeze no los incluye. Tampoco SLA/RSS/globalproductacceptance por conteo de tests.
