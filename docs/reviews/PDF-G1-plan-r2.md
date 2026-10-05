# PDF G1: revisión independiente del delta R2

2026-10-05. Sólo diseño; sin cargar DLL, ejecutar PDFium ni modificar producción.

Propuesta exacta `PDF-G1-proposal-r2.md`, SHA256 `599089C855B49A0CEE5C60F8C7B7A8603C0F0D5ACBF04E7025F3EA1C0D1F4F62`, leída completa y comprobada antes/después. Informe R1 conservado: `AEF45FFD7A3EA354A51AAEE77B00BB1B9DA47689EDC668499C0D2CEB231BEC7A`.

**SIN_CRITICAS_PENDIENTES para el diseño G1 R2. PDFG1-01 cerrado.** No equivale a aceptación de la implementación o ejecución del gate.

El texto corrige la combinación del loader a constantes nombradas DLL_LOAD_DIR=0x100 y SYSTEM32=0x800, OR=0x900, con assert anterior a carga. Excluye explícitamente DEFAULT_DIRS=0x1000 y conserva el rechazo de fallback/cwd/PATH, inspección de módulos propios y hashes previos. Esto resuelve la contradicción numérica R1 frente a la [documentación primaria de LoadLibraryExW](https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw), consultada en esta misma revisión.

El delta explicita count+1 y la comprobación del retorno de GetText dentro de su capacidad. Se mantiene la necesidad de preadmitir los bytes y count contra un máximo pequeño de estas fixtures antes de reservar; R2 no convierte corpus ASCII en certificación general UTF16/mapping. La fuente binaria debe permanecer viva hasta cerrar documento y el buffer externo hasta destruir bitmap, como exige la vida útil de las APIs revisadas en R1.

La evidencia previa de fixtures se identifica con manifest y expectativas anteriores a PDFium, sin reclamar equivalencia de raster. He revisado esa afirmación como parte del diseño; no reproduje Poppler ni verifiqué el contenido de esas fixtures en esta revisión delta. Los 774 hashes de adquisición y la DLL fueron comprobados durante R1; aquí no se presenta esa comprobación previa como nueva ejecución ni resistencia frente a mutación concurrente.

Siguen pendientes las firmas completas del harness, offsets/alineación, guardas de buffers, rechazo anterior a Init de DLL ausente/hash cambiado, barrera Job efectiva antes de PDFium/documento, deadline único, cleanup/reap/lectores en errores y controles negativos, y el manifest exacto del candidato ejecutado para revisor/AE distintos del autor. ctypes no acredita el loader de pdfium-render0.9.4. No se adoptan ABI/wrapper, licencias de distribución, G2-G5, producto PDF completo, Unix ni QA visual mediante este cierre de diseño.
