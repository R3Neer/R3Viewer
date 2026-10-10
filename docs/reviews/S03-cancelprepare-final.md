# Dictamen final focal cancelprepare S03

SIN_CRITICAS_PENDIENTES para la evidencia focal original de cinco familias, con los límites de observación de este informe. No implementación adoptada ni cierre S03. CodeAE independiente de fuente/protocolo y replay propio prepare-P; I/early/late/wrongnonce son ejecuciones del autor cuyos recibos se han cotejado, no runtimes propios. Normalcfg igualmente es runtime del autor, con exclusión binaria revisada directamente.

## Identidad y contraste

FreezeR1 MANIFEST0A55EED8D5F4289B81FF7CCD5085C540C694E17176B30BE83577058A07DD667B1043entries y causalR2 aftermanifestD7A7B2D18841FA38C6DE2BDB24AD4C150D89B413987120288D2E41707283165927entries:1070/1070 bytes/SHA/pathcontained PASS, recibo INTEGRITY-CP-R1-R2.json C032B7E6008B72769FD4C00EB5D5DC2D60EABA233C9BE1D62BA5C7512474A6C7. App5764A475DD225F2959B569F0528DDC18CF26930BD34C346D8A85D8AC5AA62E78 y DLLAFBE165EC59A5630A06B52DE62C4E1F6433144FA8337B10B113E0C4EB69F6F61, fuentes previas a outputs exactas.

Seis rutas originales completas contrastadas contra canónico actual: build.rs sólo declara check-cfg; lib.rs registra módulo cfg; main.rs observa callback/ViewerRun; presentation.rs scope/progress/checkCancel; session.rs observa entrada TerminalGuard real; text_engine.rs añade aliasIndexScope cfg y observación tras operaciones/check original. Ningún cambio de límites, condiciones de cancelación, loops, pasos de GraphemeCursor o resultado fuera cfg. Nuevo s03_prepare_probe.rs sólo compilado all(windows,s03_prepare_probe); build.rs no activa cfg. Fuente actual relevante igual al baseline308ab fuera hooks; XMLtimeout posterior es otra ruta y no forma parte de estos controles.

## Mecanismo implementado

Record512B align8/64u64 estático y DLL/application mismos load LOCK CMPXCHG/store XCHG con barrera completa, sin nonatomic refs al payload ni exportKernel32 supuesto. Header immutable tras publicación inicial contiene magic/version/size/nonce128/selector/inputlen/hash/PIDactor; app valida nonce/layout/selector, begin verifica inputlen. Source no valida por sí solo hashcontent: identidad completa de fixture se sostiene con freeze y actor que genera header, no adversarial handoff universal.

Main sólo observa su slot16; callback thread ctrlc serial es único escritor slot32; host único escritor slot48. Secuencia impar/even antes/después rechaza snapshot durante escritura; cada slot coherente, no snapshot simultáneo artificial de los tres. Pares phaseword se contrastan tras proceso esperado físicamente. Callback original conserva el mismo AtomicBool Relaxed: lee phaseBefore, hace storetrue original y phaseAfter; no pausa/barrier/ack. CheckCancelled se escribe sólo tras loadtrue en el checkpoint original. Progresos observan trabajo realmente ejecutado antes de Active; host envía ETX, nunca muta cancel app ni espera congelando worker.

Codificación real phase(kind,state)=(kind<<16)|(kind<<8)|state: el campo alto es etiqueta de la única fase/kind en esta invocación, no contador de epochs de sesiones múltiples. P1 Active65794→Cancelled65795, I2 Active131586→Cancelled131587. Causal exige before/check Active2 mismo prefijo selector; after admite2/3 mismo prefijo, signal/done/ETX1, PIDapp, sinerror/ViewerRun/Guard. OwnP before/after/check fueron65794 y final65795: misma fase identificada, distinto estado final. No igualdad numérica de estados ni cobertura de preparación repetida en un proceso.

MappingArc mantenido en OnceLock y captura callback vive hasta exit físico del app; no join/unregister de ctrlc ficticio. Alias IndexScope/ModelScope pagados como inventario del observador: Work72 frente normal64, Mapping16/ArcHeader16, scope8/index8/Once16, record512; no se confunde memoria compartida con cuota producto o RSS. Drop Rust no es receipt de cierre kernel: vida static acaba por proceso físico esperado; actor y host sí tienen closechecked/EOF/join propios. DLL actor CtrlC/Break TRUE nativo sin GIL/IO/alloc vive hasta actor exit; no ignoreNULL heredable ni cambio de modes. TerminalGuard real se usa desde viewer_session import session::TerminalGuard/enter2940; hook antes enter cubre intento, no guard falso.

## Cinco familias y regresión normal

| Familia | Evidencia actual y autoría | Resultado observado |
|---|---|---|
| P SourceProjection | Own replay exacto premfB2A394…; ownfinal3A704…29entries/report982687… | Callback/checkActive65794, Cancelled65795, app41056exit130; viewer/guard0; primera reap y Tinyreal/q0 sameConPTY. HReaped/Job0/EOF/join/checkedclose; elapsed4.892s |
| I SourceIndex | Autorfinal frozen3.531s, recibos actor/host/evidence cotejados | before/after/check131586, final131587, causaltrue, exit130→Tiny0; viewer/guard0; amboswait/join/close, Job0/noPending |
| early-P | Autorfinal0.843s | callbackphase0/check0, causalfalse, exit130→Tiny0; viewer/guard0; amboswait/join/close, Job0/noPending |
| late-I | Autorfinal13.549s | preparaciónComplete131588, Viewer1/Guard1; causalfalse; rawCtrlC interactivo exit130→Tiny0, signalprobe0 no se convierte en prueba callback durante preparación; amboswait/join/close Job0 |
| wrongnonce-P | Autorfinal0.855s | setup76 antesSource, causalfalse/counters0→Tiny0; amboswait/join/close, Job0/noPending |
| normalcfg complementario | Autor4.039s, bin markers inspección propia | Source normalframe/End/q0→Tiny/q0, masks503/7 y size25/visibletrue; normal no tresstringsprobe, probe sí; no assertion de equivalencia binaria por sólo strings |

Todos CP-M01–07 conservados: identidad/protocolo, cancel causal real en P/I; controles negativos que no acreditan esa causalidad; salida130 original sin force aceptado; no Viewer/Guard en cancelpre; reap físico y segunda invocación mismaConPTY; límites explícitos. Los cuatro controles del autor son evidencia suficiente cotejada para estas subfamilias; criterio original no exige duplicar cada ejecución ni todas las intercalaciones. OwnP es reproducción independiente, no self-report.

## Fallos y límites preservados

PfinalR1FAIL30s y captura271B376CD913… permanecen. Fuente permite deducir primera iteración validada sin errores/Pending antes OWN-START2, pero no atribuir el bloqueo posterior. R2 añade sólo nueve registros bounded192B/1728B dentroPTY1MiB, no accioneshost nuevas, y observación autor+own pasan sin intervalo bloqueado: causa histórica UNKNOWN. No se declara corregida ni se relabela aquel FAIL como PASS. No retry-to-green/cuotas nuevas. Normalcfg primer esquema fracasado y DLLedition fallida continúan en el freeze.

Los recibos validan masks y cursor size/visibility, no cursorposition universal: texto diagnóstico mueve Tiny0,9→0,15; ello se registra. No SLA de preparación/rendimiento3695s, no evidencia de todos los bytes durante todas carreras, no sesiones simultáneas/ABIotrasarquitecturas/Unix/QAvisual. NeverSpawned se deriva de ruta Source text/colournever sin worker antes ViewerRun, no se afirma inventario universal de descendientes. Excluir cfg y examinar normalbin no convierte el probe en producto ni requiere integrar hooks.
