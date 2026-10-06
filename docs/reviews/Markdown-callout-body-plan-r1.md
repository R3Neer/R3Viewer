# Revisión adversaria del plan MD-CALLOUT-BODY01 R4

2026-10-06. Propuesta exacta Markdown-callout-body-correction-plan-r1.md SHA256 2CD2FCC0C111C7F42AE2EB3101B4A88741E6860923164C98DD5267995222AAF0. Dictamen: SIN_CRITICAS_PENDIENTES dentro del diseño de esta corrección.

Se contrastaron el código R3 callout/marker, los bytes de las dos fuentes independientes, el informe AE175CA044 y su manifiesto24 registros (rehash propio sin discrepancias), y firstpass0.13.4 líneas230–267. La nueva crítica es semántica: ordinal0..0 puede pertenecer al cuerpo cuando GFM consume el marcador y no emite Run separador. R3 es NO APROBADO pese corregir los tres pánicos anteriores; el informe estático raíz previo no sustituye esta evidencia.

El helper propuesto usa Origin wholeSource solamente para spans vacíos, con zonas Header/Body delimitadas por title_start, line_end y next_line_start. Comparte la decisión entre admisión de owners y remapeo, evitando reglas divergentes. Un span del cuerpo12..17 se mantiene en ParagraphDomain aunque no existan Runs; Title permanece NONE. LF/CRLF/CR se examinan con como máximo2bytes y EOF no crea cuerpo. Orígenes cruzados, en separación o antes del título conservan Quote antes de mutar. Los guards y criterio ordinal de spans no vacíos no se amplían.

Se preservan constructor validado, validator, wire, Cargo, quotas, controller y UI. No Run artificial, target alterado ni origen desplazado. La matriz requiere las dos fuentes originales, título/cuerpo vacíos separados, estilo, CRLF/EOF y controles. Marker-only no reconocido no se fuerza a Callout; el error de expectativa previo queda declarado. Las observaciones upper runtime y lower source/scanner se distinguen explícitamente.

Autor puede implementar R4 en copia privada. La aprobación es del plan: fuente, pruebas propias, congelación y aceptación independiente R4 siguen obligatorias. D1/D2, gráficos, resolver, Unix, QA visual y cierre de RV no se acreditan.
