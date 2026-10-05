# PDF G1: ABI y carga privada Windows, propuesta R2

Corte experimental previo a integrar; PDF G0 R2 vigente. No nuevo sprint ni implementación de producto. Aún no se ha cargado la DLL. Preparación de adquisición propia en D:/Tools/R3Viewer/.tmp/pdf-native-research-r1, sin modificar Cargo raíz.

## Artefactos exactos

PDFium chromium/7881, paquete pdfium-win-x64.tgz 3733154 bytes, SHA256 73CC0DE638AC2095E7445BF56A38200A5B7C7CA0E9F4BA144598F2457377AC08, contrastado con digest publicado por GitHub. DLL SHA256 79D4676B656CFB1ABCEA88F9ADE3B4B0826C5200382DB5F4EC72A636C598C118. VERSION declara 151.0.7881.0; args.gn declara x64/win release, pdf_enable_v8=false y pdf_enable_xfa=false. Esto es evidencia del paquete declarado, no build reproducido ni prueba de ejecución de acciones.

La versión latest consultada era chromium/8086; no elegir latest flotante. Wrapper candidato pdfium-render=0.9.4 (checksum crates.io 8948A803616A9E936B15A6637AF2CD48C5FB8AE0FCDEB3C32EAC3A540E255A19), features candidatos default-features=false y pdfium_7881. El paquete del wrapper fija ese selector. MIT OR Apache-2.0; elegir MIT en incorporación futura. No adoptar aún: bind_to_library usa libloading Library::new y exige revisar la búsqueda de dependencias Windows antes de integrarlo.

Ambos paquetes descargados y extraídos privadamente tras hash, validación de cada ruta dentro del destino y rechazo de symlinks/hardlinks/dispositivos. ACQUISITION-MANIFEST.json SHA256 DEF624DAB5C05BF4B7A741F2B7C7B91F77A334357CCB17BD7C99F9AA744E090D enumera todos los archivos extraídos, incluidos headers y licencias. LICENSE más licenses/* de PDFium se conservan íntegros. No concluir revisión completa de distribución desde una lista de avisos.

## Experimento delimitado

Harness Python ctypes propio, no wrapper adoptado, declara firmas stdcall de los headers del mismo paquete; Windows x64 exclusivamente. WinDLL con ruta absoluta verificada y winmode LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_SYSTEM32 (0x900), sin fallback cwd/PATH ni cambios globales. Biblioteca y dependencias cargadas se inspeccionan por handles del hijo propio, distinguiendo PDFium de DLL del sistema. Fijar hash antes de cada ejecución. Cambios de archivo invalidan el resultado; no se promete resistencia a un atacante concurrente que controla el directorio privado.

Proceso hijo bloqueado en bootstrap antes de leer fixtures, cargar PDFium o inicializarlo. Padre crea Job propio KillOnJobClose y ProcessMemoryLimit512MiB, lo asigna por handle de Popen, consulta configuración efectiva y sólo después libera bootstrap. Deadline total y lectores acotados supervisados externamente. Fallo de asignación no permite carga. Al cancelar o fallar: terminar sólo este Job/hijo y esperar el handle; no enumerar por nombres ni tocar procesos ajenos. La inicialización del intérprete anterior al bootstrap no se renombra como contención desde nacimiento; este gate acredita contención antes de PDFium/documento, no producto completo.

FPDF_InitLibraryWithConfig usa estructura completa del header con version=2 y campos posteriores cero; validar ctypes.sizeof/alineación/offsets según ABI x64. No V8 ni XFA ni APIs de acciones/formularios. Thread único serial, sin callbacks de usuario. Cerrar bitmap/text-page/page/document y DestroyLibrary mediante finally; mantener buffer original vivo hasta cerrar documento. No llamadas concurrentes ni rescate de una llamada nativa bloqueada desde otro thread.

Corpus pequeño sintético propio construido en el padre: PDF válido de tres páginas con texto ASCII repetido, etiquetas distintas de índices y página final sin texto; corrupto, página fuera de rango y documento con cero páginas. No corpus Clases ni material de terceros. Binario de fuente <=1MiB para este gate pequeño, sin cambiar la cuota final pendiente. El hijo recibe bytes del snapshot del padre por canal acotado, no reabre ruta de documento. Primera prueba sólo LoadMemDocument64, GetPageCount, LoadPage, GetPageWidth/Height, CountChars/GetText y bitmap64x64 BGRx de buffer externo con stride checked; no texto Unicode exhaustivo ni render a resolución natural.

Expectativas escritas antes de ejecutar: índices físicos 0..2 corresponden al contrato externo 1..3; cada texto conocido se encuentra y repeticiones se preservan; página sin texto sólo tras extracción exitosa; bitmap buffer de 16384 bytes no rebasa guardas; corrupto no da documento válido, rango no genera página válida y cero páginas nunca produce primera página. El modo API binario fuera de rango no sustituye validación futura del padre. Guardas no certifican memoria interna; registrar commit Windows propio y distinguirlo de AS/RSS/agregado. Salida JSON <=64KiB, sin bytes de terminal ni texto PDF sin sanitización en pantalla.

Control negativo: DLL inexistente/hash cambiado se rechaza antes de Init; bootstrap no liberado no carga; timeout y cierre Job dejan hijo esperado/recolectado y lectores terminados. Error de fixture no oculta fallo de carga o cleanup. Congelar source, fixtures, headers/licencias, DLL y resultados con manifest antes de adversario/AE independientes. El reviewer puede reproducir dentro del mismo corte. No cerrar PDF G2-G5, RV-204, texto UTF16/mapping/rotación/CropBox/encriptado/search, raster G2 ni native Unix/QA visual.

## Fuentes primarias

[Release exacto](https://github.com/bblanchon/pdfium-binaries/releases/tag/chromium/7881), API GitHub conservada release-7881.json; headers/version/args/licencias extraídos del asset fijado. [Wrapper](https://github.com/ajrcarey/pdfium-render) y paquete crates.io 0.9.4 exacto, src/pdfium.rs bind_to_library. [WinDLL](https://docs.python.org/3/library/ctypes.html#ctypes.WinDLL) y [LoadLibraryExW](https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw) justifican loader local; deben comprobarse al revisar el diseño. Consulta 2026-10-05. No API main se confunde con ABI del paquete.

## Corrección tras revisión R1

PDFG1-01: LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR=0x100 y LOAD_LIBRARY_SEARCH_SYSTEM32=0x800; OR efectivo=0x900. Usar constantes nombradas y assert efectivo antes de carga. DEFAULT_DIRS=0x1000 queda excluido; el valor erróneo0x1100 anterior no se ejecutó. R1/informe preservados. Revisión R1 SHA256 AEF45FFD7A3EA354A51AAEE77B00BB1B9DA47689EDC668499C0D2CEB231BEC7A. No cargar DLL hasta revisión de este delta.

FPDFText_GetText usa count+1 unidades UTF16 y comprueba retorno dentro de esa capacidad; el corpus ASCII de este gate no acredita todos los mappings UTF16. Fixtures previos en fixtures/MANIFEST.json SHA917A66379629AD156A537936A6D4308C26F7AFE3C98DC79166C3A4B385424B3F; expectations fijadas antes de creación y de cualquier ejecución PDFium. Tres páginas inspeccionadas con Poppler, sin equivalencia pixel anunciada. No cambia alcance G1.


## Estado final del diseño

Revisión independiente final ../reviews/PDF-G1-plan-r2.md: SIN_CRITICAS_PENDIENTES en diseño, PDFG1-01 cerrado. El experimento de implementación y su revisión/AE son evidencias separadas y siguen pendientes de aceptación. No adopción de wrapper ni soporte PDF de producto.
