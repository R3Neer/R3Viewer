# S03-UNICODE01 — revisión adversaria de diseño R2

2026-10-05. **SIN_CRITICAS_PENDIENTES**, acotado al diseño seleccionado y su matriz previa. No implementación, resolución runtime del hallazgo, UI/visual, SLA ni aceptación de nuevos números por medición inexistente. PDF/IPC son gates independientes.

Propuesta exacta SHA256 `BD65C25E0066D6CCB4B3D000A5EC6D2787732C1A88E72A1A82810DDB4B5121A7`; matriz S03-text-grapheme-oracle-r1.json SHA256 `DB5302A8417E3400EFABEE5614A2CC51DC0D814C762FAB6E275E3CAE0D49EA92`. Verificadas antes/después. R1 y review R1 intactas: `DAFBCF2266CA3E85DB2CF4591436012EE29538198C6229062863E1A0C6A6C628` / `0E9CA1813ED7BA582C4694E0F4B4F71B0BFD74AFC9A8A9873CBE28C3AB471248`.

## TG-01 cerrada en diseño

R2 incluye explícitamente yaml_view/xml_view Pane::prepare/boundary/walk/position/window y sus consumidores viewer_session. Conserva Text/Piece prestados y fronteras actuales entre piezas; no materializa wholeleaf en SafeTextModel ni amplía SESSION1MiB/WINDOW128KiB. CLUSTER_BYTES deja de abortar preparación por mera elegibilidad de dibujo: prepara dominio y excepciones cancelablemente, con reserva real de metadatos; una cuota de índice real sigue pudiendo fallar con su diagnóstico explícito.

Preflight tipado Text(plan,lease)/Reference(info,lease) precede copia de Cell.text/Span. Contraste correcto: gigante fuera del viewport permite Text, dentro Reference y salir Text, sujeto a presupuestos vigentes. Nu no inventa SourceSpan; BorrowedDomain distingue raw/visible/piezas y SourceRelation exacta o contenedora sólo por getters acreditados; entidades/escapes no se presentan como offsets serializados equivalentes. Fuente/o/historial/n/N preservan dominio y hit, sin recursión al inspector bloqueado. Resuelve el contraejemplo de preparación global rechazada y el mapping ambiguo.

La matriz TG-B01..06 fija fuera/dentro, YAML/XML decoded/metadata, entidades/escapes, Nu, piezas y search. TG-C02 carga snapshots/excepciones/old+new y exige registrar insuficiencia de arena sin ampliación automática. Estos son gates de implementación; no hay prueba de que todos los índices ya quepan.

## TG-02 cerrada en diseño

Checkpoint ahora conserva Clone del GraphemeCursor real tras frontera resuelta, identidad/generación/len/pieza y columna. Se prohíbe new/set_cursor distante en consultas; reanudación forward reutiliza estado y checkpoints anteriores/posteriores de excepciones. Es compatible con la fuente bloqueada1.13.3 (Clone incluye ris_count/contexto), sin reinventar Unicode. La identidad impide usar estado de otra pieza o generación; offset0 sólo en entrada secuencial legítima.

R2 no da por probado que Clone elimine todo contexto: cuenta preparación, consulta y Frame separadamente, con WorkBudget que carga relecturas/contexto/ancho/proyección antes de operación chunked. Si excede trabajo local, devuelve Reference tipada, sin parcial ni pase a Ratatui. Para RI8B la matriz TG-A03/04 y B05 exige coste no proporcional al prefijo y no vuelta a0 al reanudar, incluidas consultas/repetidosresize/scroll. Contextos Indic/Prepend/ZWJ y cancel se mantienen. Esto aporta mecanismo y condición verificable que faltaban en R1.

## Revisión del contrato/matriz restante

TG-A02 offsets0/3/14/15 y columnas interiores corresponden al recipe e+U0301/mujerZWJlaptop/x. RI pares/impares y el recipe gigante32,000,001bytes tienen fronteras matemáticas independientes. TG-A09 distingue cluster válido de runASCII; exact/+1 aplica al perfil de símbolos y a agregado, sin introducir cap global de fuente. TG-A10 incluye width0/columnas iguales: no pasan símbolos excepcionales al tercero por ancho cero ni se pierde identidad buscada.

La adaptación width conserva rfold reverse, WidthInfo/tablas y wrapping_add_signed0.2.2; no suma chunks y separa acumulación columnas/tabs. Vendor/hash/licencias y gate de viabilidad siguen previos a adopción. Ratatui-core0.1.2 segmenta/mide/copia/apende, por lo que preflight anterior a toda ruta sigue obligatorio; R2 incluye tablas, keys, rows, nombres, paths, help y diagnósticos seguros bounded. Trabajo256KiB y symbols256KiB son contadores distintos, con leases/arena adicionales; metadata también se carga. Reference del viewport completo evita presentar mezcla parcial como documento.

Safe projection/controles/CRLF y roles por primer scalar original de grafema completo conservan comparación explícita con rutas actuales, incluyendo borrowedPiece y placeholders ASCII. Inline no hereda cap frame: entrega buffer completo admitido, cancela durante push/provenance y descarta construcción parcial. Search conserva raw/decoded, no busca texto del panel. Revisión de campos/dominos, tests ordinarios y rutas legacy continúa necesaria; no inferir QAvisual o SLA250ms de libraryprobe.

## Siguiente gate

Umbrales16KiB/256KiB/4096 son candidatos técnicos que deberán medirse y congelarse con viabilidad/counters/reservas. Si cambian, preservar oracle actual y justificar nueva versión previa a outputs; nunca hacer golden desde candidato. Fuentes, bytecounts/ledger, implementación exacta, controles propios y AE independientes deben demostrar los mecanismos, incluidas fallas de reserva/cancel/stale y lifecycle real. El cierre documental no elimina S03-UNICODE01 de producto por sí solo.
