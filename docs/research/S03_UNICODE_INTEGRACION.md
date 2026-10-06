# Integración Unicode: UIR-01 corregido en R2

Estado actual: integración R2 **no aprobada** por S03-AE-NUKEY01; corrección R3 privada en revisión/aceptación. UIR-01 sigue cerrado en el delta ya revisado.


## Estado vigente R2

2026-10-06. [Revisión independiente final](../reviews/S03-unicode-integracion-r2.md) sin críticas pendientes para el delta UIR-01: 24/24 controles propios pasan (diez originales sin cambio y catorce preparados antes de outputs). Snapshot528 registros, MANIFEST56EA09960104D701CB906E752CD403A7BA400FA80481EA8B75EFEBDBD8886F95. Raíz verificó receipt8135E2B255E8E5D3614AAE482237F4127B74C3E5916C0F31BE0ACEE1E6E6D320 y sus ocho evidencias. UIR-01 cerrado en ese snapshot; R1 y sus FAIL históricos se conservan abajo. No cierra S03-UNICODE01, RV ni adopción operacional completa.

Investigación focalizada del autor, 22 evidencias verificadas por raíz: XML32MB aislado9.39s (modelo2.93s, pane5.78s); YAML16MB aislado2.68s (modelo0.70s, pane1.17s). Ambos mantienen texto/rangos completos y terminan sin procesos pendientes. No reproducen la hora, pero no explican la regresión serial3695.62s ni el WorkerExited previo. El XML de ese test llama project directamente en el proceso de pruebas: WorkerExited no prueba una salida de hijo del SO; existen rutas deadline/cancel que producen esa categoría, sin identificar cuál causó el fallo. Informe temporal S03-unicode-performance-investigation-r1.md SHA7C40C1C878DF823DD8046AC072B43496FFAEC9C318FFAB0FBC29199DB7B385F1, manifiestoE4F0C81B1F16E179C98201C9AF3CB7FC86B4FE48F6C2ECBC90CD84D91189ACE0. Rendimiento/estabilidad originales pendientes, sin SLA ni conclusión por un PASS aislado.

## Historia R1 — rechazada


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


## S03-AE-NUKEY01: clave Nu atribuida a Fuente

Aceptación independiente R2: clave de Record Nushell `a` +8192 U+0301 (16385B), Source vacía, file=None. K crea pane.key=true/node_identity=None; render asigna Source a None sin consultar key y muestra origen exacto0..16385. `o` conserva Fuente vacía, pero referencia/rangos atribuyen un origen inexistente. Repetición TestBackend300x8 confirma objeto y mensaje completo: defecto de procedencia, no truncado ni QA visual. Informe privado S03-unicode-independent-acceptance-r2.md3305E2B9…, manifiesto1BCB03A3…101 evidencias verificadas por raíz, incluyendo reproA6B920FF… y postcheck. R2 no aprueba la integración completa.

Cancelación durante Source.prepare/provenance/inline, recuperación del viewport, YAMLprocedencia/historial y cuota semántica pasan en esa aceptación. El primer oráculo propio de dos planes ASCII3000 era infundado (reservas cabían); FAIL preservado y nueva expectativa previa exact262144/+1 y viewport262145 rechaza completo. No adaptar aceptación al output. YAML32M ArenaBudget más fallbackSource aislado5.948s no explica3695.62s ni WorkerExited históricos. ConPTY, runtime externo congelado, visual/Unix y aceptación general siguen pendientes.

Corrección privada R3 por raíz: único brazo `None if pane.key => NuValue { key:true }` antes del NoneSource. Controles propios preparados antesoutputs: claveRecordK/Enter, claveTableK y Source literal. OriginalSourcePASS/keyFAIL, mismos dos tests despuésPASS (bucle recorre las tres acciones); fmt/Clippyalltargets/build privados0. ParentR2 MANIFEST56EA… + overlay explícito DELTA-MANIFEST2D524BDCC68B7AE59C461CB6151917FC970B2F1AF321E91A09D8C2A442FE40A7; PRODUCTviewersession09D3324D63775E6E844FFE23145F60A28E33190AF7C40702144CA77F0F848413. Oráculos/FAILR2 intactos, código compartido no adoptado. Revisor/AE repetirán casos originales sin cambio; no cierre por tests del autor.
