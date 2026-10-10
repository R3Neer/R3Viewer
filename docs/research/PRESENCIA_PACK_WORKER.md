# Presencia física del pack de gramáticas en el worker R5

2026-10-10. [Revisión independiente](../reviews/Gramaticas-pack-presencia-worker-r1.md) sin críticas pendientes para la presencia física y lectura separada del código de carga.

El worker congelado R5, SHA-256 B68C86DF5CACD47EA66D36AB8A80E48F1BBCA47FA48499C241C4D21550BB52B3, contiene íntegramente el pack de 355.889 bytes, SHA-256 301A65C46F0B86764BBF557674616EA899CBE67C402E48751EB3AEA6B7B62C51. Hay una única coincidencia, en el offset 8.027.880, totalmente contenida en la sección PE .rdata. La comparación usa todos los bytes; no una muestra, prefijo ni normalización.

El adversario usó su propia reconstrucción previamente aceptada y un scanner propio fijado antes de producir resultados. Comprobó tamaños y hashes del worker y del pack, recorrió las secciones con límites y extrajo el intervalo completo para comparar nuevamente. Leyó por separado build.rs y syntax_worker.rs: el primero genera el pack; bundled_syntaxes usa include_bytes en Windows y run_worker llama esa función. La fuente canónica y la fuente congelada del worker no son idénticas y se registraron separadamente.

Raíz leyó el informe, el scanner y el resultado y cotejó las nueve evidencias independientes sin discrepancias. Manifest independiente 3E187FDD1CBBE1582037F081C9694805DC815AE6E6337E438D30A0D3CE011312; informe 1E5F10CA7282DD71E0835B524B04FC470D89029E66FF2595FE5B1A5EEB7E4FAB. Evidencia privada: .tmp/d4b-scheduler-adversary-2026-10-10/pack-presence-own-r1. Entrega raíz: .tmp/root-syntax-asset-provenance-2026-10-10/worker-pack-presence-r1, manifest CA760D9BFA8EFFA4A8407B28D9A5906993D52AFED3294ED0A509D3D2AD014195.

Este experimento no ejecutó el worker. Acredita presencia física en ese candidato y lectura de la ruta de carga; el uso dinámico de esos bytes, el embedding del entregable final, la distribución, plataformas adicionales y la suficiencia jurídica permanecen pendientes. No cierra RV-501 ni amplía la [reproducibilidad del pack local](REPRODUCIBILIDAD_PACK_LOCAL.md) más allá de su evidencia.
