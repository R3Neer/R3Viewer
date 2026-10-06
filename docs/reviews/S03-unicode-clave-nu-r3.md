# Revisión y aceptación independiente NuKey R3

**SIN_CRITICAS_PENDIENTES para el delta NuKey; S03-AE-NUKEY01 resuelto.** Este dictamen es específico de la corrección y no cierra integración completa.

Identidad compuesta: parent Unicode R2 MANIFEST56EA09960104D701CB906E752CD403A7BA400FA80481EA8B75EFEBDBD8886F95,528 registros + DELTA-MANIFEST2D524BDCC68B7AE59C461CB6151917FC970B2F1AF321E91A09D8C2A442FE40A7,10 entradas; todos verificados antes/después. Sólo productionOverlay declarada: repo/src/viewer_session.rs→PRODUCT-viewer_session.rs SHA09D3324D63775E6E844FFE23145F60A28E33190AF7C40702144CA77F0F848413 y bin/r3viewer.exe SHA66D017DC93E09C4142DD9D316E9B0C4C3E3A3C4D51947148C5B9168860600582. Archivo repo/src del delta con tests root no usado como producción.

Revisión propia verificó byte-exactitud: producción R3 equivale a producción R2 con una sola rama `None if pane.key => Domain::NuValue { key: true }` anterior a None=>Source. Los tres callers de open_key pertenecen al Navigator Nushell; pane.key=true sin node_identity identifica esa clave. Fuente ordinaria con key=false permanece Source. Se conserva SourceRelation None porque Reference::text sólo asigna SourceRelation Exact/ControlProjection al Domain Source. No altera rangos de clave/identidad/owner, fuente/proyección ni cuotas.

Copia limpia R2 byteverificada + PRODUCT overlay declarada. Sufijos propios de tests agregados en R2 transferidos byte-exactos; oráculos originales y adicional también byte-exactos. Código de producción completo anterior al cierre del módulo de tests intacto. Compilación propia no cambia binarios originales; artefacto testexe enlaza funciones congeladas y tests propios. No se ejecutó ni confunde con bin/r3viewer.exe original.

Tres controles relevantes PASS, con sus aserciones intactas:

- own_s03ae_nu_key_no_source: original FAIL R2 ahora PASS; TestBackend120x10 muestra `dominio NuValue { key: true }; rango0..16385; visible0..16385; sin span literal de Fuente`, distribuido en dos líneas. Source vacía, file=None y clave real original conservados después de o.
- own_s03ae_nu_key_relation: original FAIL adicional R2 ahora PASS. Objeto pane.key=true,node_identity=None,SourceLen0; backend300x8 muestra misma relación correcta completa. Rangos reales0..16385 y acción o conservados.
- own_s03ae_source_frame_not_partial: control de rama Fuente key=false PASS: Reference íntegra antes materializar, ningún BEFORE parcial y recuperación hacia AFTER, Source original completa.

Runner H/C propio acotado copiado explícitamente de AE R2, sin heredar aprobación: tres Jobs/procesos propios naturales con exits0, wait/closed/joined, Active0 antescleanup, Reaped, sin teardown externo ni rescate para PASS. Pendingfalse y sólo thread1 después. Registro nu_pid denomina aquí al testexe, no afirma proceso Nushell real.

Evidencias anteriores FAIL R2 y oráculo de cuota inválido siguen conservados: esta revisión no los reescribe ni ejecuta otra batería. No se repiten los24 controles UIR01, Source32M/RI/control ni serial20. Queda pendiente ConPTY/restauración/cancel130, runtime externo integrado y visual/manual/Unix. Originales3695.62s y WorkerExited con causa desconocida permanecen abiertos; probe aislada YAML32M previa no explica esos eventos. SIN_CRITICAS_PENDIENTES se limita a NuKey y a la exactitud de su delta/controles.
