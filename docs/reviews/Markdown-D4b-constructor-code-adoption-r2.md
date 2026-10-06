# D4b constructor R2: adopción de código

Constructor de CodeText y selección compartida incorporados tras aceptación independiente. Conserva bytes semánticos, mapa run/local, clave por documento/bloque, Consumer y contexto original; la primera pasada reúne mapa y conteos y la segunda copia y calcula SHA. El bloque vacío natural sale antes de reservar metadata/input.

## Aceptación y combinación

Freeze 490D1D62ED44B091A252F525736DA29015B8F91D6067746ECA85673CC9165D91, 45 registros verificados; contexto 365 archivos. Informe independiente 8A4CB659E8683CB6A6CC63BE29F20B8DCAC47ECEF92D37F88D3FE4450F3419D8 y manifiesto 9FC854CB088755E2FDD48942748DB60DAFE4C5238CEBF6FF8BB77AFAF6709A47, 33 registros verificados. SIN_CRITICAS_PENDIENTES en constructor puro combinado R1+delta R2. D4B-CONSTRUCT-R01 cerrado con tres controles nuevos y cinco regresiones afectadas propios; los controles R1 de código inalterado permanecen trazables, sin repetirlos ni borrar su rechazo anterior.

El adversario acredita dos pasadas 5/5/1, vacío natural sin incremento de pico libre o con Consumer lleno, y record Decoded vacío con mapa temporal pero sin input/snapshot. Cuota/aliases, SHA y líneas, multirun/Decoded, cancelación por chunk y límite de línea comprobados en el delta. No se atribuyen las pruebas del autor al adversario.

En shared se copiaron 13 rutas productivas byteexactas y se añadió sólo el registro privado document_code a lib.rs, conservando los módulos b0. Las dos fuentes modificadas coincidían con la base 98b1d37; no solapamiento semántico con b0. Primer precheck de lib falló porque su expectativa omitía la línea en blanco añadida; diff exacto inspeccionado y expectativa corregida antes de copiar. No modificación de algoritmo por ese diagnóstico.

Sesión raíz 38868 terminal0: 10 constructor, 7 retained, 9 selector y 19 preparation_owner pasan (45 en total); fmt y Clippy workspace/alltargets terminan0. Advertencia upstream proc-macro-error2 de compatibilidad futura preservada. Logs Markdown-D4b-shared-*-r2.log en evidencia del chat. Sin prueba nativa del worker en este corte de biblioteca.

## Packaging y límites

Atributo -text acotado a tests/fixtures/markdown-code preserva nueve fixtures y sus offsets CRLF. Fuentes Rust se normalizan a LF conforme a política; fixture C09 conserva sus bytes. Índice y checkout privado se contrastan antes del commit.

No scheduler, cache/CodeActivation, owner de fondo b1, overlay, prioridad de frame, terminal, inline ni Markdown visible acreditados por este constructor. No D4b completo, QA visual, Unix, distribución, publicación, perfiles o cierre RV-301. Los límites y fallos R1 permanecen históricos; la corrección no cambia plazos o presupuestos para obtener aprobación.
