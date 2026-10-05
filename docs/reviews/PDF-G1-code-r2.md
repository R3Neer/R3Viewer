# PDF G1: revisión independiente de código R2

2026-10-05. Adversario distinto del autor raíz. Revisión de fuentes congeladas y ejecución únicamente en copia privada propia; sin editar candidato/producción, sin Rust ni wrapper adoptado.

## Identidad y dictamen

Candidato `pdf-g1-candidate-r2/MANIFEST.json` SHA256 `F5D894E7C3C0D78F423E1087D77806C18A298FCE409600646C88FDE8DEDD6125`: **816 tamaños/hashes verificados antes y después, cero discrepancias**. Delivery R2 `FBB833B06FC419E540CF6A32A3C5F19554826477EBB6CAAA7257FA9947F3C085`, leído junto al delivery R1 para conocer ABI y límites preservados. Worker `C08D319441B83234D46F65412767AB6C3678CD142BB7C5A7C9175B00CD5F83E1`; DLL `79D4676B656CFB1ABCEA88F9ADE3B4B0826C5200382DB5F4EC72A636C598C118`.

**SIN_CRITICAS_PENDIENTES en el candidato privado G1 R2 para el corte Windows x64 ctypes/fixtures pequeñas/controles ejecutados. AE-PDF-01 cerrado por código y repro.** No aprobación del gate PDF completo, integración, wrapper, distribuciones, corpus general ni QA visual.

Copia ejecutada: `pdf-g1-adversary-r2`. Evidencia propia `EVIDENCE-MANIFEST.json` SHA256 `4A7E133DDD90416BBF2E8503B0F13200B67D5385B5369310ED782CA246D1B5EA`. Conserva scripts, resultados y RAW de los controles; los ficheros author-results en esta copia son resultados de mi repetición, sin modificar los del snapshot.

## Evidencia ejecutada

**26 controles PASS:** ocho casos funcionales repetidos (válido, cero, corrupto, DLL inexistente/hash cambiado, cancelREADY/cancelLOADED/timeoutLOADED); diez fallos de setup/logging repetidos; ocho controles propios adicionales. Scripts relativos ejecutados con Python `C:/Python314/python.exe` en la copia. No rescate del test ni enumeración de procesos por nombre.

Las ocho adiciones propias son fallo de read1 en stdout/stderr, fallo de IsProcessInJob, fallo del readback final, marcador bootstrap incorrecto, longitud1MiB+1, fuente truncada y byte extra tras fuente. Las cuatro solicitudes malformadas dan exit2, diagnóstico ASCII escapado y ausencia de JSON exitoso. Los cuatro fallos del controlador conservan excepción y dejan el único Popen esperado, handle cerrado, streams cerrados y lectores recogidos/sin iniciar. `adversary-controls.py` y `ADVERSARY-CONTROLS.json` documentan cada punto; no reutilizan validate para decidir estas expectativas.

El válido confirma páginas físicas0/1/2, dimensiones200×200, textos `alpha alpha`, `beta beta`, vacío, chars11/9/0 y retorno12/10/0. Cero abre con count0 y LoadPage0 inválido; corrupto no abre documento. Los bitmaps externos tienen16384 bytes, stride256 y guardas intactas. Los hashes de bitmap se registran como observación, **no oracle de fidelidad**, ni comparación Poppler. No reproduje la inspección visual previa de fixtures.

Job readback observa flags8448/0x2100, límite536870912bytes y pertenencia del handle propio antes de bootstrap. Pico de process committed normal18,673,664/18,309,120/18,546,688bytes en válido/cero/corrupto: observaciones pequeñas, no RSS/AS, estrés512MiB ni agregado. Cancelación por cierre Job devolvió0 en estas sesiones; se acredita por fase viva anterior, cierre/espera y falta de JSON final, sin convertir exit0 en éxito. Holds son previos a Init: no prueba de interrupción dentro de una llamada PDFium bloqueada.

## Código y fronteras revisadas

AE-PDF-01: try/finally comienza inmediatamente tras Popen; objetos opcionales están preparados antes. Capture conserva objeto antes de start. Cleanup intenta cierre Job, kill si vivo, wait, join de threads iniciados, cierre streams y handle confirmado antes del logging RAW. El fallo de logging no salta esos pasos; errores de cleanup no se convierten en PASS. Constructor/start parcial y errores posteriores quedan cubiertos por los controles repetidos y propios.

Loader usa ruta absoluta y hash fijado antes de carga; constantes0x100|0x800=0x900 correctas, sin DEFAULT_DIRS. Inventario de módulos del hijo antes/después: PDFium desde la DLL de la copia y único módulo adicional en las sesiones normales. Esto es observación de ese proceso, no garantía ante atacante concurrente, DLLs previamente cargadas arbitrarias o I/O de producto. [LoadLibraryExW, fuente primaria](https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw).

Firmas contrastadas con headers7881: LoadMemDocument64 size_t/handle opaco, Width/Height double, APIs void None, CountChars/GetText int y unsigned short*, FillRect BOOL en7881. Config completo size48/alignment8/offsets0,8,16,24,32,40,44; version2, resto cero. ctypes.WinDLL necesita restype explícito porque su default es int; el worker lo declara para las APIs PDFium usadas. [ctypes, fuente primaria](https://docs.python.org/3/library/ctypes.html#ctypes.WinDLL).

Fuente del hijo limitada1MiB antes de leerla; count páginas<=3 y CountChars<=4096 antes de buffers count+1; retorno y terminador comprobados, decode UTF16 estricto. Captures64KiB por canal, resultado ASCII JSON escapado<=64KiB; controles/C1 dentro de strings JSON no se emiten como terminal commands. El padre sólo trabaja con fixtures privadas fijadas: su read_bytes anterior al assert no constituye adquisición bounded de documento arbitrario de producto. Asimismo hashread de DLL es lectura del asset privado fijo, no API de archivo general.

Buffer fuente permanece vivo hasta CloseDocument; raw externo hasta DestroyBitmap. Cierre normal bitmap/text-page/page/document precede DestroyLibrary, en un solo thread. Una terminación OS cancela sin ejecutar finalmente los destructores del hijo: no se presenta como cleanup normal. InitLibraryWithConfig no devuelve estado; la evidencia funcional proviene de las operaciones posteriores. No se interpreta build declarado sinV8/XFA como certificación universal de inercia.

## Límites del cierre

No se ejercitó error artificial en cada API nativa ni se certificó comportamiento bajo bloqueo dentro de ellas. Fixtures/ASCII/bitmap64×64 no acreditan Unicode completo, surrogates/mappings, labels API, links, permisos/cifrado, rotación/CropBox, search, raster de producto, estrés o fuentes grandes. Wrapper0.9.4 sólo inspeccionado: ctypes no acredita su libloading, lifecycle ni ABI integrado. Dependencias/licencias fijadas no equivalen a aprobación de distribución.

IPC del producto, agregado padre+hijo, cachés, presupuesto final PDF, workerRust, adquisición mutante, Unix, terminal/inline y QA visual siguen pendientes. S03/RV204 y PDFG2-G5 permanecen abiertos. La aceptación del candidato exige también AE distinto del autor/revisor; este informe es la revisión independiente de código y controles acotados, no sustitución de esa AE.
