# Job attach_verified R2 — revisión delta y aceptación independiente

2026-10-06 Europe/Madrid. Autor root; adversario unix_r3_review. **SIN_CRITICAS_PENDIENTES en el helper/configuración/readback y API de retención examinados. JOBV-01/02 cerrados en R2.** No prueba de CloseHandlefailure, controller/C/STARTwire, writerlifecycle/JobPending ni Unix; caller sigue responsable de cleanup/retención.

Freeze document-job-verified-root-r2/MANIFEST.json SHA `1CDCB729E8DD80406E606BA0A7CF7F05648AB66C4335C6FD0E914D7B2E113B7F`,12registros hash/tamaño correctos antes/después,0discrepancias. PRODUCT-worker_process.rs SHA `D5844D51FF5C8E37C52A5C920DD075964A49747B547D9713E663F73ED6928452`. Copia document-job-independent-r2 y target propio document-job-independent-target-r2; smallcrate offline/locked Windows-sys0.61.2/Windows-link0.2.1, sin SDK/workspacebuild/Git/perfiles/producción. R1 MANIFEST y sus11records revalidados intactos; sus FAIL/mutantes/reportes no se reescriben.

## Resolución de código

JOBV-01: flags readback ahora `LimitFlags == required` exacto0x2100, manteniendo512MiB, Query bytesreturned exactos, GetHandleInformation noinheritable e IsProcessInJob del mismo handlechild/Job. Mis controles BREAKAWAY_OK0x2900/SILENT_BREAKAWAY_OK0x3100 con mismas expectativas InvalidData se rechazan en R2. Sin descendientes escapados ni garantía de ataque natural OS; acreditada corrección del verifier ante configuración equivocada.

JOBV-02: attach_verified ya no delega setup al attach histórico que sólo devuelve ioError. Crea Job y lo envuelve inmediatamente en Self antes de Set/Assign/Query; creaciónfallida devuelve AttachFailure{error,job:None}. Todo setup/readback posterior devuelve su ioError hacia finish_attach, sin mover Job mientras closure lo presta; luego finish_attach devuelve el mismo Job con Some en error. Campo error conserva diagnóstico, debug sólo marca retained_job; caller puede retener/limpiar esa autoridad. No destruct/drop anticipado del lease en postcreatefailure. nonWindows delega Unsupported yNone; sólo lectura cfg, no ejecución Unix.

Attach histórico no cambia texto normalizado ni lógica. BASE-RAW mantieneLF90/2883B, BASE CRLF90/2973B con SHA distinto; no igualdadbyteexactfalsa. PRODUCT R2 LF200 es prefijo byte-exacto del harness antes del módulo de tests. No se usa la diferencia EOL como patchfuncional ni como prueba de código no revisado.

## Controles propios y reproducción

Antesoutputs: AE-REPLAY-EXPECTED-original.json es copia intacta del oráculo R1 corregido únicamente en su suposición no normativa de exitcode, y cuatro tests propios R1 son prefijo byte-exacto del nuevo archivo de tests. Se mantienen rechazo de ambos flags, rechazo PROCESS_MEMORYausente y closeJob→reap de peer todavía gated, sin exigir exitstatusno0. FalloR1/primerharnessnonzero sigue preservado, no retorno a golden del output.

Dos controles nuevos fijados antesruntime en AE-OWNERSHIP-EXPECTED-before-output.json SHA `5F37EB1736FC6EA70B6E1D330F21E7829E717D70B53461B421B0BDC4F14EB1F2`:

- Error setup sintético pasado a finish_attach: SomeJob/rawhandle exactamente mismo, ioKind/message intactos, GetHandleInformation válido y propiochild aún gated; sólo después propietario close+wait, sin rescue. Acredita composición/move/retención, no fallo natural de Set ni Close.
- Child propio terminado/waited antes de attach_verified: expectativa previa condiciona observación OS, no obliga a inventar un error si Windows acepta; **se observó errorWin32 5 Acceso denegado** y API devolvió SomeJob con handle válido. Ese actor estaba ya reaped, cierre posterior explícito del Job. Así hay evidencia nativa de la rama real postcreate Assignfallido; no se simula OpenProcess/childajeno.

Receta cargo test --offline --locked -- --nocapture --test-threads=1 en copia/target propio: **12PASS/0FAIL**,6propios (4intactos+2nuevos) y6testsautor reproducidos (5históricos+1retentiontwoflags). Exit0;2.76s compilación/.34s tests segúnCargo, no SLA. Propios cierran/reap/wait antes asserts críticos; noFALLBACK/rescue logs. Los actores sólo propios, sin START liberado en controles de cierre; exit0 obtenido porcloseJob no se interpreta respuesta válida ni publicación. Sesión final terminada sin checks/procesos propios vivos.

Log AE-NATIVE-r2-attempt1.log SHA `C5E9F150E18F471B756902BE1B89F2E17607B6D3B49D316EF6177CD11D41EBAF`. Receipt AE-RECEIPT-r2.json SHA `C3DC350C98F147ACDB9E1A8F0C3AD1F8D3B6A802A9F69FF340D5F7D0ADFEC9C4`, identidades antes/después, oráculos/harness/log y alcance.

## Límite y entrega

API permite retención útil para C; no obliga a caller a usarla correctamente ni demuestra retrycleanup, falloCloseHandle, stderr/writer/joins/finalreceipts/Active0 de un controller. OwnedHandleDrop/close histórico sigue sin receipt de CloseHandleerror; este incremento no declara ese receipt ni adopta C por herencia. Integración deberá probar su ownPending y graph START/source/worker antes aceptar. No existencia de contrato helper verde como sustituto del gate entero, Unix/runtimeproducto/visual/RV/S03 pendientes. Sin otros hallazgos concretos en el delta revisado; R2 apto para integrar esta API bajo coordinaciónroot y revisiónC futura.
