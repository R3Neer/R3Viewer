# Inventario local de avisos Cargo para Windows

2026-10-10. Investigación de RV-501, sin aceptación de distribución. Base343c5aa. No descarga, build, instalación, modificación de dependencias ni publicación.

Se inspeccionaron los275paquetes del metadata Cargo filtrado para x86_64-pc-windows-msvc, cuyo hash sigue1CADEE2EBB241718D9FCDC27CFF3760D655FDC5CE557FE1446E5C9F6FA917925; Cargo.lock conserva7CCFFA215C44A6F04930706FAE6B131FBA3F080619C768566B52B09C05F407F7. El grafo incluye tipos build/dev/runtime y el companion Nushell: no equivale al contenido de un paquete final ni a cobertura de Linux/macOS.

El inventario físico registra458archivos por nombre LICENSE/LICENCE/NOTICE/COPYING/COPYRIGHT y license_file declarado, con ruta relativa, bytes y SHA256. Recorre cada raíz de paquete; para el paquete raíz R3Viewer usa sólo archivos directos para no atribuirle .tools/.tmp ni dependencias de otros paquetes. Una primera ejecución se detuvo al detectar ese recorrido demasiado amplio; no produjo inventario aceptado y su script está preservado. La segunda terminó0 con regla corregida. No interpreta expresiones SPDX, elige licencias alternativas ni certifica derechos sobre assets; tampoco detectar un nombre acredita contenido correcto.

Ocho paquetes no tienen un archivo detectado por esa regla:

| Paquete | Licencia declarada | Trabajo pendiente |
| --- | --- | --- |
| alloc-stdlib0.2.4 | BSD-3-Clause | Revisar aviso original y procedencia exacta. |
| buf-trait0.4.1 | Apache-2.0 | Revisar texto original, notices y procedencia exacta. |
| byteyarn0.5.1 | Apache-2.0 | Revisar texto original, notices y procedencia exacta. |
| nu-config0.115.1 | MIT | Contrastar licencia y copyright del origen Nushell fijado. |
| nu-experimental0.115.1 | MIT | Contrastar licencia y copyright del origen Nushell fijado. |
| omnipath0.1.6 | MIT OR Apache-2.0 | Revisar texto original y elección documentada al empaquetar. |
| r3viewer-nu-plugin0.1.0 | MIT | Verificar inclusión de la licencia del proyecto en el paquete conjunto. |
| r3viewer-wire0.1.0 | MIT | Verificar inclusión de la licencia del proyecto en el paquete conjunto. |

La ausencia de nombre no demuestra ausencia de licencia: puede estar embebida, fuera de la raíz de la crate o requerir documentación del origen. No sustituir avisos upstream por la licencia MIT de R3Viewer. Revisar también assets de sintaxis generados durante build, parches locales y su procedencia antes de decidir qué avisos se distribuyen.

Evidencia privada en .tmp/root-dependency-inventory-2026-10-10/notices-r1: PHYSICAL-NOTICES.json SHA82F43345C388250A3F6487DC0D4BCDFA654CB7B0BE3C49092201E2816BF79316; SUMMARY.json SHA1350CF2DF7728258EBF8DE300528907258F3639F565B420DA4F8EBDF16FA9BB0; script ejecutado SHA40035EC5CC2568E3234DDD9E6543E5C81529601643A266B9F680923703FF543C. No se incorporan rutas de caché ni copias de avisos a la distribución por esta investigación.

RV-501 sigue abierto: faltan atribuciones comprobadas por artefacto, textos/avisos finales, bundle reproducible, motores gráficos/PDF/HTML y sus fuentes/codecs, así como matriz completa de sistemas. El metadata global offline anterior falló por r-efi5.3.0 ausente; no se descargó ni se acredita cierre multiplataforma.

## Origen de los avisos no incluidos en las crates

2026-10-10. Captura primaria de los seis paquetes upstream pendientes, usando repository y el commit exacto de su .cargo_vcs_info.json local. Se consultaron árboles GitHub no truncados y sólo archivos de licencia/aviso en la ruta de la crate o sus ancestros. Siete capturas comprobadas por SHA, sin errores HTTP. No se usa main ni una licencia de una versión distinta como sustituto.

| Paquete | Commit publicado | Archivos primarios |
| --- | --- | --- |
| alloc-stdlib0.2.4 | ae42d22078b98549e987d2f03d12df7b984fde47 | [LICENSE](https://github.com/dropbox/rust-alloc-no-stdlib/blob/ae42d22078b98549e987d2f03d12df7b984fde47/LICENSE), SHA C0C56F26D9C051CAC4D200C34C84E7AE9AAA853E01A982A1DF08B09931E518AE. |
| buf-trait0.4.1 | 82c9cb19de4743cdbdf882134389781a77b25c70 | [LICENSE.md](https://github.com/mcy/strings/blob/82c9cb19de4743cdbdf882134389781a77b25c70/LICENSE.md), SHA CFC7749B96F63BD31C3C42B5C471BF756814053E847C10F3EB003417BC523D30. |
| byteyarn0.5.1 | dde8c1cda1c5608db7029fcf2190b39ee8cea5a6 | [LICENSE.md](https://github.com/mcy/strings/blob/dde8c1cda1c5608db7029fcf2190b39ee8cea5a6/LICENSE.md), mismo hash que buf-trait; versiones y procedencia siguen siendo entradas separadas. |
| nu-config0.115.1 y nu-experimental0.115.1 | 798c55d19505fd52f205d7eb32a571b9d06ec9e6 | [LICENSE](https://github.com/nushell/nushell/blob/798c55d19505fd52f205d7eb32a571b9d06ec9e6/LICENSE), SHA FF5DCAABA6140EC57CAFDD4F707CD0D329E727477CF7D5B9BA7A244EC2678B79. |
| omnipath0.1.6 | 0d316e93dc522093d5686c44686fdf7afa272f56 | [LICENSE-APACHE](https://github.com/ChrisDenton/omnipath/blob/0d316e93dc522093d5686c44686fdf7afa272f56/LICENSE-APACHE), SHA 0D542E0C8804E39AA7F37EB00DA5A762149DC682D7829451287E11B938E94594; [LICENSE-MIT](https://github.com/ChrisDenton/omnipath/blob/0d316e93dc522093d5686c44686fdf7afa272f56/LICENSE-MIT), SHA 748C15C2A285EF74AEC280F2E72BD98482B96BD0E2DA0D8F0FDE934FA88E2E16. |

Capturas, árboles originales y script permanecen privados en .tmp/root-dependency-inventory-2026-10-10/notices-upstream-r1. UPSTREAM-SOURCES.json SHA921862A6583B78E3D39ACBB6B64E4D9A24C985892A20B11DA7E99A52BFAE9BE0 contiene URLs, commit, path_in_vcs, blob Git, bytes y SHA de cada captura. Esto identifica los textos originales faltantes del inventario por nombre; no acredita todavía copia en el bundle ni revisión de todos los avisos embebidos en fuentes/assets. Los dos componentes propios declaran MIT y deben viajar con LICENSE de R3Viewer, actualmente copyright2026R3Neer. La distribución efectiva sigue pendiente RV-501.

## Revisión independiente del inventario

[Revisión final R1](../reviews/Dependencias-avisos-Windows-r1.md): SIN_CRITICAS_PENDIENTES para exactitud y procedencia sobre e960082. Reconstrucción ajena de275/458/ocho ausencias, commits/rutas y7capturas byteexactas contra blobs y árboles vivos. Atribuciones pendientes concretas: mcy incluye plantilla Apache sin copyright nominal; omnipath ofrece permiso MIT sin copyright nominal. No inventar titular/año al preparar avisos; investigación cerrada en este alcance, RV-501 y bundle final abiertos.

## Avisos de gramáticas

[Investigación de fuentes fijadas](AVISOS_GRAMATICAS.md) y [dictamen final R2](../reviews/Gramaticas-avisos-r2.md): 74/74 gramáticas del árbol Packages fijado verificadas por bytes/SHA256/blob Git; fuentes del cargador y generador Syntect2/2 cotejadas. Avisos propios MIT identificados en C# y YAML; cinco coincidencias restantes son tokens de gramática. La revisión independiente se limita a exactitud y clasificación: no acredita ausencia legal exhaustiva, procedencia del dump generado ni avisos finales del bundle. RV-501 sigue pendiente.

La [reconstrucción íntegra del dump base](REPRODUCIBILIDAD_DUMP_GRAMATICAS.md) y la del [pack local con tres adiciones](REPRODUCIBILIDAD_PACK_LOCAL.md) cuentan ahora con compilación y ejecución independientes. Se acredita reproducibilidad actual byteexacta; no autoría histórica, embedding final ni avisos exhaustivos/distribución. Estas comprobaciones amplían la investigación anterior sin cerrar RV-501.

## Ensamblaje privado de avisos R2

2026-10-10. [Revisión independiente del ensamblaje](../reviews/Dependencias-avisos-ensamblaje-Windows-r2.md) sin críticas pendientes para copia mecánica, procedencia registrada y privacidad del índice. Receta propia del adversario: 275 paquetes, 467 avisos y 2.252.777 bytes; comparación íntegra contra origen y candidato congelado. Universo de IDs exactamente igual al metadata, siete hashes de blob Git recalculados, ledger e índice iguales. No nueva validación remota de pertenencia de esos blobs al árbol histórico.

El índice público usa 275 package_ref relativos únicos y referencias a avisos; ninguna ruta local, URL de archivo, ruta UNC/Unix absoluta o carpeta de caché. El ledger de procedencia con rutas originales permanece privado. Las alternativas declaradas y todos los textos se preservan, sin normalizar, deduplicar entre paquetes, elegir licencias ni inventar titulares.

R1 se conserva rechazado para el índice: diez IDs Cargo locales exponían rutas absolutas. El registro notice-staging-r1-index-rejection.json y sus archivos no cambiaron. R2 corrige la construcción del índice y reproduce los mismos bytes y correspondencias. El primer comprobador histórico del adversario usó una clave equivocada y contó cero; su fallo está preservado y la corrección posterior acredita diez, sin cambiar R1 ni R2.

Raíz leyó el dictamen y cotejó 487/487 evidencias independientes. Candidato privado .tmp/root-dependency-inventory-2026-10-10/notice-staging-r2: manifest7B3ED94CE1EAE2A7EA075B3DEF3FDB67345CA103277EC0C728D60DD056C4CA8F. Evidencia independiente .tmp/d4b-scheduler-adversary-2026-10-10/notice-staging-own-r2: manifestF0450D38006A18F34FE113DBDF467236E85A43BCFE45751A5E94A1D22B3B725B; informe4EFD8280D8F7344D59B68BC11E9A21C352DE5CBFE574DA5D2299281ADA5F6C27.

Esta aceptación no adopta los avisos como bundle final. El inventario sigue siendo superset Cargo Windows con build/dev/runtime; recursos de gramáticas/motores, fuentes/codecs, cierre por artefacto/plataforma, derechos exhaustivos y RV-501 siguen pendientes. No se ejecutó, instaló o publicó el producto.
