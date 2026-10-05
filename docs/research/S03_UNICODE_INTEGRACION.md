# Integración Unicode: candidato R1 sin aprobar

2026-10-06. Contrato [operacional](../refinement/S03-UNICODE-INTEGRACION.md) y [grafemas](../refinement/S03-GRAFEMAS.md). El gate mecánico privado aceptado no acredita los consumidores del producto. Candidato integrado R1 congelado y revisión/aceptación independientes en curso; **UIR-01 reproducido impide aprobarlo**. S03-UNICODE01 y tareas principales siguen abiertas.

## Identidad y evidencia

Snapshot privado unicode-integrated-r1, MANIFEST36786926EED660B7644CD0344D69CA9BA023EAFEE8A2F11099B5B066D18F4AC3,502 registros. Base documental d1399f33ab6fa4afbf37952f384e22ee4b137747. Raíz verificó original/copia antes y después: sólo instrumentación de pruebas añadida a lib.rs en copia propia, prefijo original byteexacto. Fuente productiva compartida permanece sin commit hasta aceptación. Bin87393097D78FAB01D8756D03DEED04129DF217209A571828EEABB298A59320F0; companionFE4F04247990C473C05D49741E40F83E1FCE6BCAE155EA3E78F7B1CAB2101A87.

## UIR-01: filas cortas pierden el primer grafema siguiente

Adversario unix_r3_review, distinto del implementador, ejecutó ocho casos propios con expectativas derivadas de la fuente antes de outputs: seis FAIL y dos controles PASS (filas de80 columnas). En YAML y XML Pane.windows, ab\ncd\nef devuelve [ab,d,f]; \nx\ny devuelve tres filas vacías; a\nćd\n🐱x devuelve [a,d,x]. No es un defecto atribuido a la terminal: las ventanas de texto ya contienen la pérdida antes de Ratatui.

walk avanza el cursor antes del callback. Cuando el callback encuentra unit.line mayor que la fila actual, ese primer grafema ya fue consumido; sequential reutiliza el cursor posterior para la fila siguiente. Corregir preflight y materialización conservando cursor resuelto, identidad, cuotas y replay; comprobar filas vacías/cortas, Unicode, CRLF, clipping y fronteras de piezas. No ocultar la pérdida eliminando contenido o cambiando expectativas. Freeze R1, seis FAIL y controles quedan intactos; corrección R2 y nueva aceptación pendientes.

Evidencia temporal unicode-review-r1/INDEPENDENT-PREOUTPUT-MULTIROW.json 5A7D4BD1F92428756E3682581B9DEEBBD05C0DEF31001457581F08501AD61826 y INDEPENDENT-MULTIROW-attempt1.log AB18BCA191F89B61CF767CE9A549AA87D7C65A54B68E3DBFD098BE04AD2E590B. Informe final de revisión todavía en curso; conservará otros hallazgos si los hay.

## Cobertura positiva delimitada y pendientes

Raíz ejecutó pruebas propias Source32MB: referencia del grafema completo antes de materializar, recuperación de líneas ordinarias y búsqueda original. Regional Indicators mantienen pares/checkpoints y rechazan cursor de otro owner. Controles ESC/C1, CRLF y grafemas completos pasan tras corregir una consulta mal escrita del harness (dos backslashes frente a uno en fixture); ambos FAIL originales y diagnóstico conservados, sin cambio de producto ni oráculo semántico. Informe parcial privado S03-unicode-root-acceptance-partial-r1.md F59DB4EB804BA404ABDADBCE320786F9DB54191FD6CAC1DC3A12A757C30A74D5, siete evidencias. Estas pruebas no aprueban consumidores YAML/XML ni el candidato completo.

YAML32MB falla en la cuota existente del modelo antes de Pane; Fuente sigue íntegra. No es un PASS borrowed:16MB admitido es otra prueba. Cancelación durante rutas integradas, historial/procedencia, resto de aceptación, ConPTY funcional y QA visual/Unix siguen pendientes. No cerrar RV ni declarar producto completo por conteos de tests, compilación cruzada o SHA.
