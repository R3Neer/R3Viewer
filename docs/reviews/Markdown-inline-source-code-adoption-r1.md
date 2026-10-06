# Source fallback inline: adopción del helper R1

Preparación completa de Fuente segura incorporada como módulo privado para el futuro emitter D3. Cuenta proyección antes de reservar una única String, conserva CRLF/tabs/Unicode, hace visibles C0/DEL/C1 y sólo devuelve la candidata completa. No construye SafeTextModel ni índices, no aplica el millón de líneas de Source TUI, no emite ANSI ni escribe stdout.

## Identidad y aceptación

Freeze 2A08EA31BF23DBCCCBFC1E0B56239DD2A3C3A5B8F5559B67EC629E3B005FF5D1, nueve registros; fuente 09B8ABD9C91C803686ED68B44BCA497129315FEE57D60CE410587BED59B0C679. La proyección escalar coincide exactamente con text_projection compartido. Informe independiente FF1280966BB4B6800D4CA4FEF83BB05C4C4E2525BB9857E3ECE400E7202DD390 y manifiesto16 133C1690DEAE9EC273B8CCBA4F781EB7C920CAAD164F07D4973E9D912086945A verificados sin discrepancias. SIN_CRITICAS_PENDIENTES limitado al helper.

Cuatro controles propios: toda la proyección C0/DEL/C1 y Unicode con expected470B antes runtime; límite470 admite una reserva,469 falla antes allocation; vacío/cancel inicial no reservan; un millón LF produce salida completa con una reserva; cancel inyectado al reservar libera18B candidata sin devolver prefijo. Tracker del allocator de prueba no es medición RSS ni garantía de allocator peak. Clamp256MiB y guard de capacityovershoot se verificaron estáticamente; no matriz256MiB ni fallo del allocator real acreditados. Los intentos propios de setup y compilación fallidos del adversario se conservan, sin atribuirles ejecución funcional.

## Combinación compartida

Sesión raíz45188 terminal0: tres pruebas del helper y seis de inline existente pasan; fmt y Clippy workspace/alltargets terminan0. Advertencia upstream proc-macro-error2 de compatibilidad futura preservada. Logs Markdown-inline-source-shared-*-r1.log en evidencia del chat. El source copiado es byteexacto al freeze; lib sólo registra el módulo interno con allow(dead_code) explícito hasta el caller D3.

Formato inicial del autor falló bajo sandbox por canonicalización/acceso antes de compilar. Retry con skip_children y toolchain autorizado terminó0; no modificó text_projection compartido. Primer freeze intentó copiar logs vacíos que Tee no había creado, falló antes del manifiesto; receipt conserva el diagnóstico y los logs se registraron vacíos de acuerdo con salida terminal0. Estas reparaciones de preparación no cambian código ni oráculos.

## Límites de incorporación

No conexión CLI, gutter, color, geometría, representación estructural de Markdown, diagnóstico de fallback, transición TUI ni aceptación D3 completo. El futuro caller destruye la candidata de Documento antes de invocar este helper y escribe únicamente el resultado completo; esa integración todavía no está probada. Presupuesto de salida es distinto de Consumer/Actor, capacidad observada distinta de RSS. No publicación, perfiles, Unix/QA, distribución ni cierre RV-103/RV-301.
