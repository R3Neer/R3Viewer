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
