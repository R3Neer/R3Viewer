# S03 B07: preparación, búsqueda y navegación

2026-10-10. [Aceptación independiente focal R3](../reviews/S03-B07-final.md) sin críticas pendientes. Completa la medición de fuente/índice y la cancelación tardía del matcher sobre el código real, junto a la aceptación previa de preparación cancelable y consulta acotada. S03 permanece abierto.

Fuente sintética de128MiB,16385líneas, controles y Unicode; snapshot/hash original conservados. Una ejecución release del autor y otra independiente pasan. El adversario reproduce el binario congelado sin compilar ni repetir suite: un test PASS, exit0, wait físico y lectores recogidos, sin limpieza forzada;521 inputs intactos antes/después. La raíz coteja527 entradas del manifest final.

| Medida | Autor | Reproducción independiente |
|---|---:|---:|
| Preparación de fuente e índice |5,2927909s|4,793487s|
| Búsqueda completa sin coincidencia |58,5644ms|57,3281ms|
| Navegación y dibujo de viewport |478,2us|677,5us|

Son tiempos observados de cada ejecución, sin SLA ni medida del productor Nu/encoder/IPC. Las capacidades conocidas del modelo suman413401144B; original134217728B y visible268435456B, además de checkpoints/índice. El recibo detalla crecimiento y cabeceras; no es RSS, memoria global, coste del allocator/canales/stacks ni pico simultáneo de todas las fases.

El observador privado espera progreso real de65536bytes antes de llamar SearchWorker.cancel. El matcher devuelve Cancelled y no publica resultado obsoleto; una nueva búsqueda NotFound termina en la misma fuente, generaciones1/3, con snapshot y posición invariantes. Drop recorre el join original, cuyo error se descarta: no se afirma una prueba adicional de errores de join. TestBackend y este API no acreditan apariencia ni Ctrl-C nativo.

R1 fue rechazado por posible doble panic con mutex envenenado antes de runtime. La primera compilación fallida y R2 runtime fallido se preservan. R2 usaba cotas9/2 propias de tabla; Source permite10/3 filas en80x12/30x5. R3 corrige sólo esas expectativas tras errata prospectiva; fuente, matcher, clocks y cuotas productivos no cambian. Los hooks cfg(test) siguen en copia privada, sin incorporación al producto.

Bin SHA23D3733F552E75822A8439573F2B1185B5B60A322F4E1E81E4A0E240693467DD; fixtureCAAF9A24DA0FEEA85F55AA28E50FC6600A8A520D97DCAABA05A90BC195127A0B. Manifest finalF8E8480241C5A63EB5C1B70366C883A0AF8609CA85D34780FEE3CC0FE83E88FE; informe9FCA52F9DDF88EBCD7EA19EA7166272988649D09F3F0582C83CBF63D0137D8C1. Artefactos privados en `.tmp/s03-closure-adversary-2026-10-10/b07-own-ae-r3`.

QA visual y Unix continúan pendientes. Los tiempos/fallos XML históricos conservan causa desconocida; esta medición no los explica.
