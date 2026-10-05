# PDF G1 R2 — AE independiente

**SIN_CRITICAS_PENDIENTES dentro del gate ABI/corpus/lifecycle acotado ejecutado. AE-PDF-01 cerrado.** No adopción wrapper ni producción/PDFG2-G5/Unix/terminalvisual. Autor root; AE graphics_gate_experiment, distinta del revisor de código en paralelo.

## Corte e integridad

Cviz/pdf-g1-candidate-r2 manifest SHA F5D894E7C3C0D78F423E1087D77806C18A298FCE409600646C88FDE8DEDD6125,816archivos tamaños/hash comprobados antes/después. Delivery SHA FBB833B06FC419E540CF6A32A3C5F19554826477EBB6CAAA7257FA9947F3C085 verificado. Se ejecutó sólo copia privada nueva Cviz/pdf-g1-ae-r2/candidate-copy. R1 original805registros nuevamente íntegros; worker/DLL/valid/zero/corrupt/EXPECTED R1-R2 idénticos por bytes. Adquisición original manifest DEF624DAB5C05BF4B7A741F2B7C7B91F77A334357CCB17BD7C99F9AA744E090D y todas sus entradas verificados al cierre, sin divergencia. Fuentes/snapshots/oracles/producción intactos. AE evidencia manifest SHA 364F9A865B1591001CD5C58A8F18234903306212EFCA2F6993B1FE588BEA3213.

## Regressión propia

execute.py mismo archivo AE R1, reutiliza run candidato real pero no validate/main ni AUTHOR-RESULTS; assertions independientes con oracle previo F635469BEBF051568373750D922974ECBFF25BAA947D7618825A89B5EC5661CC. Ocho casos PASS: valid3page físicos0/1/2 (externos1/2/3),200×200, alpha alpha/beta beta/empty exactos, repeticiones preservadas2/2/0, CountChars11/9/0; copied12/10/0 dentro count+1. Zero count0 sin primera válida; corrupt no documento. Rango LoadPage(count) inválido. BGRx64×64, stride256/16384B y guardas externas intactas; no comparación pixels.

PDFium7881 absoluta/hash79D4676B656CFB1ABCEA88F9ADE3B4B0826C5200382DB5F4EC72A636C598C118, WinDLL0x900 namedflags, módulos propios controlados; Config48 align8 offsets0/8/16/24/32/40/44 version2, firmas header exactas. Job readback512MiB y flags8448 + IsProcessInJob exacto antes bootstrap/source/load/Init, buffer bytes snapshot padre confirmadoSHA; no rutas documento en hijo. committed normal valid/zero/corrupt18255872/18264064/18599936B, no AS/RSS/agregado. DLLmissing/hashchanged rechazan antes LOADED/Init.

CancelREADY/LOADED: hijo propio vivo, CloseJob exitoso y wait/lectores/escritor, ausencia JSONfinal; exit0 clasificado external-cancel, jamás éxito aplicación. Timeout holdbeforeInit: CloseJob/wait y ausencia JSONfinal, exit0 clasificado external-timeout. Ni hold ni marker equivale a nativecallblocked. AE-RESULTS incluye PIDs retenidos y readback/resultados propios.

## Cierre AE-PDF-01 y controles propios

R2 sitúa try/finally inmediatamente tras Popen, Capture constructor no arranca hilo, arranque explícito después asignación objeto opcional. Finalmente cleanup por pasos intentados, fallos recogidos y logging sólo después; cierra handlePopen sólo una vez wait obtuvo returncode.

fault-controls.py independiente, no usa selftests de autor: siete inyecciones reales sobre módulo importado desde copia, sin editar sus archivos. (1) constructorCapture1 (reproduce AE-PDF-01 original), (2) constructorCapture2 tras lectorstdout arrancado, (3–5) Thread.start1/2/3 (lector1/lector2/escritor), (6) AssignProcessToJobObject failure previo release, (7) RAW write failure tras cleanup.

Todos elevan excepción esperada; **antes de cualquier rescate AE** returncode ya obtenido, handle cerrado, tres pipes cerrados y todos los lectores existentes muertos. AErescueNeeded false en7/7. Se retuvieron únicamente Popen propios y finally de seguridad no tuvo que matar ninguno. Logging fallido no saltó recogida. Inyección Jobfalse produjo WinError0 porque control no modifica LastError, no se cuenta como error del candidato; acredita rama rechazada y cleanup, no reproduce un códigoOS concreto. Fallossetup antesbootstrap no ejecutan PDFium; logging caso normal sí termina su ejecución correctamente.

## Reproducir y límites

C:\Python314\python.exe <Cviz/pdf-g1-ae-r2/execute.py> y fault-controls.py; ejecución en copia privada, RAW/AE-RESULTS/FAULT-RESULTS propios preservados. Si se repite, usar copia nueva para conservar evidencia. No controles propios adicionales sobre Kill failure/CloseHandlefailure/lecturaoverflow ni presión512MiB; source estático muestra intento de restantescleanup, no se anuncia exhaustividad. Codeadversary separado aún decide hallazgos de código; AE no reemplaza revisión.

No LoadPage(-1) extra, guardas físicas GetText ni canary tras BitmapDestroy como nuevosPASS; se conservan fronteras R1, ABI x64 no certifica stdcall x86. Sin wrapperRust, nativeblockedcall, PDFs hostiles/largebudget, permisos/cifrado/UTF16mapping exhaustivo/CropBox/rotación/search/labelsAPI/corpusClases/OCR/IPC/workerproducto/rasterG2/cache/distribuciónUnix/QAvisual. Esta conclusión sólo permite avanzar al siguiente gate con esos pendientes explícitos.
