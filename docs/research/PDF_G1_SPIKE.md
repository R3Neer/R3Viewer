# PDF G1 R2: cleanup de setup parcial

Autor raíz; nueva revisión y AE pendientes. Responde a AE-PDF-01 de informe4AC2D2E8B39939ED990EF7884E724AD98A03150B0D27C603E8696DB29371CFEF. El PASS de ocho casos R1 no aprobaba el gate. R1 y su fallo están conservados.

Snapshot Cviz/pdf-g1-candidate-r2, MANIFEST SHA256 F5D894E7C3C0D78F423E1087D77806C18A298FCE409600646C88FDE8DEDD6125,816 entradas verificadas. R1 MANIFEST1BE346… y sus805 entradas comprobadas intactas antes de freeze. Worker c08d319441b83234d46f65412767ab6c3678cd142bb7c5a7c9175b00cd5f83e1 idéntico, DLL/headers/fixtures/expectations tampoco cambian. Sólo delta del controlador y controles propios; sin producción ni wrapper adoptado.

Harness da37e3cb0d9cd9b949a02fb92cb7f216b4c8973bc415b2d132dfccb3efbe6f1b pone try/finally inmediatamente después de Popen exitoso. out/err/job/writer opcionales están preparados antes de crear hijo. Capture constructor no lanza thread; se retiene el objeto y luego se hace start. Cada paso de cleanup se intenta aunque falle otro: cerrar Job, terminar hijo propio si sigue vivo, wait handle, join threads que llegaron a iniciar, cierre de stdin/stdout/stderr y cierre del handle Popen cuando wait ya confirmó salida. Logging RAW sólo después de todos los intentos; un error de evidencia no puede impedir los pasos anteriores. Errores de cleanup se conservan y no se convierten en PASS.

Controles autor, no dictamen independiente: constructor primer/segundo Capture, start primer/segundo/tercer thread (tercero escritor), creación/configuración/asignación/readback Job y logging final. Cada uno eleva el fallo inyectado, termina y espera al único hijo Popen retenido, cierra su handle/streams y deja threads terminados o sin iniciar, **sin rescate del test**. Control script33047F666796A6B404306AC851FBD61D29C32A39D0DE8BED9FCDC753185244E6; AUTHOR-FAULT-CONTROLS7549377861353C13FDA5FE8DC974F54856014E6E44996C07FC99CC7D1A0E469D. Inyección desde Windows propio, no PID por nombre/lista, señales ajenas ni cambios del sistema.

Los ocho casos PDF/loader/cancel/timeout previos pasan otra vez sobre R2. KillOnClose sigue devolviendo0 en estos controles y se clasifica por fase, wait y falta de JSON final, sin usarlo como éxito de aplicación. Ninguna llamada nativa bloqueada acreditada: holds antes de Init. Reproducir en copia escribible del snapshot y ejecutar pdf-g1-harness.py + pdf-g1-r2-controls.py allí, sin alterar snapshot. Controls/root por defecto relativos al script copiado; override del harness sólo explícito para directorio de evidencia propio.

Se conserva delivery R1 como descripción del ABI/corpus y sus límites. R2 no amplía textoUTF16/mapping/rotación/CropBox/links/crypto/stress/workerRust/rasterG2/Unix/terminal/QA visual. Ningún RV/G1 entero aprobado desde resultados de autor.

## Estado independiente final

[Revisión de código R2](../reviews/PDF-G1-code-r2.md) y [aceptación R2](../reviews/PDF-G1-aceptacion-r2.md): SIN_CRITICAS_PENDIENTES acotado al candidato privado Windows ctypes y corpus pequeño. AE-PDF-01 cerrado. El adversario ejecutó26controles propios (incluye repeticiones del autor); AE8casos+7faults, sin rescate. No sumar estos corpora como casos únicos. Rust/wrapper, corpus amplio, cuotas máximas, llamada nativa bloqueada, raster/IPC/cache/producto/terminal/visual/Unix siguen pendientes; no cerrar G1 entero/RV204/S03.
