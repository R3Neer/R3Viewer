# Adopción del diseño ResponseLease HTML R3

2026-10-10. Base071f6d563332ad51e350a18d0705370a772a7da7. Se incorpora [contrato](../refinement/HTML-RESPONSE-LEASE.md) y [29 oráculos prospectivos](../refinement/HTML-RESPONSE-LEASE-ORACULOS.md) tras [dictamen independiente sin críticas pendientes](HTML-response-lease-plan-r3.md). Ningún SDK, parser, Cargo, conversor, perfil personal ni runtime modificado.

RL-R1-01: el mismo HtmlCallOwned conserva worker y respuesta. Idle exige cierre físico de préstamos/Value/header, worker recogido o realmente no creado y Drop final del bundle fuera del mutex. Un error de respuesta ya enviado no libera el gate mientras haya Child/Job/pipes/threads Pending. Shutdown conserva la misma autoridad; no otro controller ni bool como proof.

RL-R2-02: el callback nominal usa un trait consumible propio, realizable en Rust estable; vtable object-safe y puente Any permanecen separados. Permiso tipado antes construir captura, args, tuple, error-return, bridge o resultado; tamaños y capacidades se inventariarán en código compilado. No Fn manual, nightly, callback auxiliar gratuito ni referencias que escapen del préstamo.

Guard exterior opt-in abarca run, init, error recursivo, write, flush y try_to_report. Value completo y Error únicamente; streams se rechazan para llamadas HTML guardadas y mantienen el comportamiento legacy en otras llamadas. CallKey procede del CallId, flag real y controller de la interfaz SDK; clones no duplican payload ni permiso de respuesta. Work y cargos siguen vivos hasta liberación física, con IO/callback/Drop grande fuera del mutex.

El adversario conserva doce familias propias anteriores a la propuesta y contrasta29 oráculos prospectivos, sin runtime. Verifica24 identidades físicas (4 artefactos,9 fuentes,11 originalesR1/R2). Raíz corrobora esas24 más manifest, sello, dictamen y criterios:28 controles SHA/bytes PASS. R1/R2 y rechazos permanecen temporales; las copias canónicas se normalizan aLF con prefacio, no se anuncian byteidénticas a originales.

Identidades: propuesta E757E265406A5B13A8699063ED07540E4D935635C429673F7E1AE75CD8B6AE4E; oráculos F3C3B07D5FE22B47BD41397E5345620AFCCFD93792962D6B8E244C7DF193EDE7; manifest 9B0071C814E2A4DE0A20A093698EF733E635AC739FF680DF2EABDF6C8E7FFDF3; sello 55BB5CD669A5DA7B3AA8E98C3AFDBF2725B6F09F2EF27F42D84C4D4413315867; dictamen 0F8A4F1A820F8EED83E7C83E91F570EF5A1E07F7AE15A38A951E04E577E26029. La etiqueta historical93658 en prosa no acredita ejecución contra esa base; manifest071f y fuentes físicas fijan el candidato.

Siguiente gate: implementación SDK/HtmlCompanionOwner privada, compile-only e inventario de cuerpos/align/Arc headers/capacidades/capturas/proofs/retiring antes runtime, freeze exacto y CodeAE/AE independiente. Debe realizar las pruebas de preadmisión exact-/1, callbacks once/downcast/compile-fail, error/init/retry/unwind/flush bloqueado, Pending worker con replyDone, shutdown retenido y últimoDrop, además de regresiones encoder legacy/streams. Diseño aceptado no acredita tamaños, memoria medida ni una API existente.

DOM/selectedness/decoder/sink/exporter/controlador/parser/HTTP/origen/navigation/QA y Unix siguen pendientes. Registro companion Nu ya cerrado no se repite. RV401–404 permanecen abiertas y el diseño HTML estático previo conserva sus límites.
