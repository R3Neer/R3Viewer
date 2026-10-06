# D4b constructor R2 — aceptación independiente del delta

**SIN_CRITICAS_PENDIENTES en el constructor puro combinado R1+deltaR2. D4B-CONSTRUCT-R01 cerrado.** No aprobación de scheduler, background, D4b completo o UI. R1 rechazado e informe E86634187AA81AAF4F53F83AD2F7D8EF8EC16B5198B862B469B23CEB69923B78 y sus33evidencias CBBEB273… permanecen intactos.

Freeze R2 MANIFEST490D1D62ED44B091A252F525736DA29015B8F91D6067746ECA85673CC9165D91,45registros/46archivos físicos. Delivery989B9B372E2A950F26BE19CB030F9F4A8F860A2199727D114602AB3985A42B2D. Se verificaron45antes de copiar y sourcezip365byteexactos. Final readback repite45+365; Product14 comparado contraR1 demuestra sólo document_code/syntax_retained/document_code_tests diferentes; otras11 exactas. Nueva copia/target, sin cambiar freeze ni shared. Base98b1d37 anterior a b0 integrado: lib.rs debe combinar únicamente registro de módulo, no sobrescribir shared.

## Cierre de la crítica y empty

Document_code::build ya no tiene loop de runs. Copy_code_domain preadmite ChargedVec de todos segmentos desde run_count, primera pasada valida Domain/Block/run/ordinal, calcula prefix/total/map y LF/longline mediante chunksUTF8<=4096. Segunda pasada llena único String, SHA en la misma copia y lineStarts. No tercer recorrido ni String plano intermedio. Preadmite metadata exacta de CodeSnapshot sólo después de totalnonzero, antes de crear/copy input; mueve cargo/segments/input/digest al snapshot, mismoConsumer. El bundle interno CodeDomainCopy tiene campos crate-private, pero la única construcción productiva de CodeSnapshot sigue privada y consume directamente el resultado del helper validado; no hay constructor que acepte un bundle/charge externo ni weakening de Document proof/roles.

Natural Domain.run_count0 se detecta después de selección y reciprocidad, antes de cualquier reserva map/snapshot/input/String/lineStarts: PlainNone con Consumer usado y peak inalterados, también fullConsumer. Para nonzero run_count con todos los payloads vacíos, única pasada deja map temporal admisible y lo destruye al devolverNone; no snapshot/input/String/lineStarts/hashfill. No se borra observación R1 baseline64→peak508/fullBudgetPlainSourceBudget. Esa observación era mejora temprano-empty, no defecto de corrupción ni fallo funcional independiente; delta demuestra mejora específica.

## Expectativas y runtime propios

Expectativas R1 FE795…/9fixtures, addendums y DIGEST-before-output copiados byteexactos. Parent harness entero2923D38B2EA4C8697A6982819CCFCA15C66E07787D027A180B57F86B8A890ABE conservado en HARNESS-PARENT-byteexact.rs. Tests originales no se editaron: se añadió import explícito std::mem::size_of porque el módulo padre dejó de importarlo, y tres tests nuevos de delta fijados en EXPECTED-DELTA-before-output.md antes de construir/ejecutar. No ajuste de cuota a success-output ni changinggoldens.

8 controles focales propios pasan individualmente sobre OWN-UNIT-attempt1.exe EC8FB6A7A573C3F5E082BDDAA24033D4C5430AE8D4CB79C2698BEE0916A37985:

1. FiveDecodedruns controlados de R1: stats primera5/segunda5/inputcreated1, payload/map completos. Contadores cfg(test) congelados usados por test independiente y corroborados por revisión de loops exactos; no test autor atribuido como propio ni instrumentalización del bin producto.
2. Naturalempty run_count0, Consumer libre/full: stats0/0/0, PlainNone, no incremento usado/peak.
3. Modelo validado/quarantine propio con un CodeText Decoded vacío y SourceOrigin de record EnclosingToken8..12: stats1/0/0 y PlainNone. Preadmisión temporal sólo map1segment (12B conforme capacity independiente antes llamada); Consumer vuelve baseline. No reinterpretación de SourceOrigin::Literal de input como relación de TextRef.
4. Original own_code_decoded_empty_runs_offsets inalterado: cinco segmentos incluyendo vacíos, rawlocalUTF8/end/foreignrun correctos, Decoded distinto de Source ABCD.
5. Original independent_payload_hash_and_lines inalterado: hashes Python previos para twofences/chunk8197B/Decoded6B y lineStarts terminalempty correctos. No hash Source completo como sustituto.
6. Original exact_quota_and_aliases inalterado: cargo calculado desde sizes/capacities independientes ANTES llamada exact→Ready/+1escasez→Plain y oldnew/aliases hasta últimoDrop. Reordenar map/metadata no cambia total ni libera prematuramente.
7. Original cancel_after_utf8_chunk inalterado: hook cfg(test) después de primer chunk4095B al llegar a é, Atomic original activado→Cancelled y cargos parciales liberados. No wallclock/externalcancel/SLA.
8. Original longline_boundary inalterado:65535ASCII+LF→Ready,65536ASCII+LF→Plain, entero y sin cargos restantes.

Logs TEST-own_code_* completos;0.00–0.01s por control. No suite de nueve repetida ni selector/key/context/CRLF controles inalterados reejecutados. Aceptación combinada referencia explícitamente los nueve grupos funcionales R1 y observación previa; estos ocho son tres nuevos y cinco regresiones afectadas, no diecisiete grupos nuevos independientes. Selected source changes justifican las cinco repeticiones; no matrices gigantes.

## Identidades y límites

BUILD-attempt1 terminal0,35.36s, sesión80109 finalizada antes copy/runtime. Compile offline escalado sólo copia/target nuevos; testexec sandbox WindowsPowerShell. No helpers OS, Jobs, perfiles, rescates ni handles activos. La copia source sólo añade un sufijo cfg(test) en document_code para own-code.rs; productiveprefix byteexacto.365context=364copyexact+1prefixexact. Ningún cambio productivo en funciones/visibilidad ni constructor artificial de proof.

Lectura diagnóstica intentó DELTA.json inexistente y produjo exec1 junto a lectura del build terminado: preservado en DIAGNOSTICS.md como fallo de lookup del revisor, no buildfailure, ausencia de artefacto exigido ni causa de candidata. Manifest era autoridad de archivos, no aquel nombre supuesto. Setup Python imprimió aviso de real location, terminó0 y verificó45/365; resumen explícitamente identificado sin inventar transcript.

Contrato D4b canónico6346…/D4R4E40C… mantiene límites y snapshot entero-or-Plain. No worker native, nuevas gramáticas, SourceLegacy runtime, background/scheduler/cache/Activation/rolecursor/render/Workhook, b1, TUI/inline/TTY/backend/ConPTY/Unix, QAvisual, RSS, aggregatedmemory/SLA o cierreRV. Current UserFrame/demandadmission priority será acreditado al conectar factories reales, no por builder privado. C09CRLF24B debe conservar bytes al adoptar Git, gate packaging raíz distinto de esta AE; no rerun por normalización de módulos hasta probar identidad productiva.
