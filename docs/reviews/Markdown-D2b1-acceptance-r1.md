# D2b1 R1 — aceptación funcional independiente

Dictamen: NO APROBADO por MD-B1-ANCHOR01. Diez grupos positivos propios PASS; un grupo negativo encuentra aceptación incorrecta de raw anchor cuando el resultado es Reference geométrico. No es aprobación de renderer/CLI ni del producto completo.

## Identidad y método

Freeze MANIFEST AECD2CFEA4D1673B560CBB81D1EF60E14DA237D05BD51692C9E38A98201C52D9:627 registros antes/después sin discrepancias; copia583 fuentes/recursos exactos. Basec423421612936758d3efb32b362d89036cf244e9, nueve deltas contrastados con blobsGit y hashes baseline. Nunca se aplica patch de adopción; su reparación CRLF/LF es un gate separado del producto congelado. Módulos de base posteriores/D4/fallback no se sobreescribieron en shared.

Sólo document_view.rs recibe sufijo cfg(test) en copia privada, own-b1.rs propio; READBACK-after acredita prefijo byteexacto de583 archivos. No funciones candidatas, constructor Document ni proof se modificaron. Parser/project/validate/quarantine reales; PreparationAuthority<DocumentPreparationInput,PreparedState> real; SourceFactory real. No DI/DO unit exitoso ni facade. Preoutputs20grupos FD2BB3EC59E2AFCEC4245F4F2FE657FBDC68A89E9CB141C58B7921BCF551B6B2 preservados; addenda preceden cada output nuevo.

## Hallazgo MD-B1-ANCHOR01

Fixture propia SourceLiteral `abc`, Document validado, Block1/Run0, texto len3. Request SeekKnown RawAnchor{block1,run0,byte4}. Con width40, prepare pasa por locate_raw y commit devuelve Err(Stale), current byte0 intacto. Con width0, WindowBuilder falla Geometry y prepare selecciona Reference sin ejecutar sequence/locate_raw; commit devuelve Ok y current pasa a rawbyte4 fuera del Run. TEST-attempt2-anchor.log exit101 conserva ambos resultados. Expectativa ANCHOR-ADDENDUM-before-output exige rechazo/preservación independiente de geometría, conforme D2b§3 (anchor acreditado y Geometry0 mantiene anchor).

Causa estática: document_view.rs remap líneas363–368 llama State::prepared_run; document_navigation.rs líneas266–278 copia raw_byte sin range/UTF8. prepare combina selección y builderError mediante Reference Geometry líneas510–529, evitando el único validador actual del byte. Navigator.commit_prepared comprueba sólo FlowLocation antes sustituir selección. Esto no es fallo de render, Source ni sistema operativo; es frontera de aceptación de la API crate-private comprometida. Source hashes: document_view F5EE1DEAF047C35ECE20657DFCBA9F85600148E7969F518FEA5B8ACEAF42CA94, navigation4B7CD4D4FEFA7F2118AEACAA74CB963DAF72091BDB00F636493891D3F2603AD8.

Corrección requerida: validar anchor de texto/Run/raw/owner antes de conceder selección incluso en ramas Reference/Pending y sin depender de dimensión/color/layout. Mantener Source/raw y normalización de cluster mediante APIs acreditadas; no clamp silencioso de byte fuera de Run para aprobar. Revisar asimismo continuidad UTF8 y reciprocidad Block/Run del anchor como frontera del mismo remap; no se atribuye a runtime propio aún. Repetir fixture original inmutable más controles válidos y negativos en nuevo freeze. No modificar R1/oráculos.

## Evidencia propia positiva

TEST-attempt1 siete grupos PASS0.03s bin BD3BAE5A00FD3CF222C7A1377A2AB7743756B14A079479EDEFDF7B370BC0F63D. Construcción privada añadió después otros cuatro tests; no se afirma sietePASS contra bin posterior sin ejecutarlos.

- Unicode a+combining cruzando Strong: un glyph ancho1, OR Strong2, primer componente Run de a/raw0; b separado. Resizewidth1 conserva anchor, no historia, mismo frame dentroWork262144 conReference8192 consumido; glyphmetadata/relación de C1 seguras.
- Oversize18001B: initial wholeReference Symbol sin prefijo BEFORE; End recupera texto AFTER íntegro. Leading mark devuelve WidthZero; geometrywidth0/height0 ReferenceGeometry. Estas geometrías se aceptan cuando anchors proceden de targets válidos, lo que no resuelve negativo anterior.
- Tab real8 local y 界 ancho2; hscroll code leftdentro界 excluye glyph entero y deja B en x1. Fixture tab-wide original contiene literal backslash-t: permanece intacta y no se le atribuye TAB real; source del controlnuevo fijado antesoutput.
- FlowHandle comparte mismo FlowIndex por préstamo, cobra sizeof metadata por alias, no Clone público. Aliasdenegado con Consumerfull. Drop Navigator conserva allocation/lease hasta últimoHandle; finaldrop libera used a baseline y checkpoint0.
- Fold muestra TITLE/AFTER colapsado, BODY oculto; expand cambia Flowidentity, rechaza localizaciónvieja; Resize conservaFlow y rawanchor. Candidate stale superseded rechazado y actualsymbols/current intactos. RequestConsumerfull conserva actual.
- Typed autoridad real alterna Document→SourcePrepared::Model→Document, joinedreceipt exigido; sourceinput/context exacto y Source line0/ original conservados. No runtime TUI.
- Gate de prueba explícito retiene input/DO reales: beforefinished takeValidatedNone, cleanupdeadlineNow→Pending, otra Document factoryBusy/Pending, DropAuthority mueve globalPending y HASH rechazado; release propio seguido retry_pendingjoined libera slot y próximo DocumentReady. GuardDrop siempre libera barrera en fallo; no watchdog/proceso externo ni contabilidad ficticia de reap.

TEST-attempt3-window un grupo PASS0.00s bin DDD298EB3825C730399952E46EB9961076098AFE7CA120209C3224244A63633B (mismo bin de anchorFAIL). WindowBuilder quotas calculadas con sizeof/capacidad: exact aceita, exact-1 rechaza/libera cargos; sumaold+newFrame262144exact admite y +1rechaza. Geometría1000x1000 prepara wholeReferenceWindow. No se afirma igualdad RSS ni límites de buffer/framework no implementado.

TEST-attempt4-history y TEST-attempt5-line dos grupos PASS0.01s cada uno bin1A16929D5017B08B6DF5614154839341439352B3413D44F40F2E17B963129B38. Nuevo SharedFlow conserva semántica Navigator:400Home/End sin historia,128 saltos reales, mismoanchor no añade,129rechaza concurrent preservado. Line+1/-1 sobrecode3líneas y viewportheight1 prepara one→two→one, conservaFlow y no historia. No claim Page/Back remap b3.

## Revisión estática del corte

Nueve deltas revisados completos en alcance productivo: Flow/newSharedFlow, Navigation, SegmentedText nuevos métodos, Layout, View, Consumer getters, owner sizegetters ylibregistración; tests de autor identificados como evidenciaajena, no contados. SourcePreparation/kernel previos byteexactos salvo getters de tamaño sin cambio de lifecycle, aceptación b0 conserva alcance.

Flow visible separado del BodyFlow mantiene contratos previos; TitleDomain se incorpora antesbody/colapsado sin duplicar árbol. Aliasown reserve antesArcclone, FlowAllocation sobrecargo antesArc materializado, budgetcheckpoint4096 retiene cursores/points. Mandatory View/scratch8192/roles65/Reference4096x2 y Navigation128 se reservan antesperfil; mismosConsumer/ActorArena, Source capacidad separada.

Layout usa única Work::frame con8192 previamenteadmitido y WholeReference al fallo de símbolo/Work/cuota. Window capacities físicas comprobadas antes publicar; raw glyph primeraCoverage/ORstyle/links/relation, visit_coverage una vezporUnit, sin String por hoja ni resegmentador backend. Preparación extra locate/índice usa Workpreparation fueraDraw, no renueva frame. Slots oldcurrent/candidate: al nuevoDomain no reutilizable builder.slot se poneNone antes SequenceSlot::prepare, local visitas anteriores terminan antes siguiente allocation; no se conserva tercerSlot. Esto es revisión estática, no contador runtime globalmaxslots.

locate_raw valida range/UTF8 y binarysearch cobra comparaciones; nuevos seek_before/line/cell cobran antesclonarPoint. Excepciones step cobran comparaciones antes inspección. FalloAnchor deriva precisamente de no invocar validación en todasramas, no de resegmentación Unicode.

Request captura Arcsource/Document/keygeometry/foldrevision/operation y Flowalias; current/historia no van mutable a worker. Commit coteja keyexpected/Document/source/Flow, navegación primero sinfallo posterior ylayoutgeometry/fold juntos; cancel/stalekernel no publicaReady antesjoin. Binderrevision checked sinhash truncado. ErrorPositionPending conserva current y noNoMatch. Aliascurrent/oldslot/outputcharges persisten. Límites y backends futuros no se declararon implementados.

## Fallos de arnés/entorno preservados

BUILD-attempt1 exit101 getter raw_text inexistente. HARNESS-attempt1 conserva también dos precisiones previas: Strong es2 según parserprimario, C1 se escribe literal backslash-u{0085} según text_projection; corregidas ANTESruntime, criterios ORStrong/proyecciónsegura no cambiaron. HARNESS-attempt2-before-runtime y HARNESS-precision conservan esas correcciones. BUILD-attempt2/3/4/5 terminal0. Lectura safe_text.rs inexistente fue error de ruta reviewer; módulo real presentation. Primer gitblob lookup sandbox falló Permissiondenied y scriptassert, preservado como diagnóstico; repetición escalada readonly verifica9baselines, no sourceedit. Python diagnosticlocation sandbox no se interpreta como fallo producto. Manifestreadback final sin cambios.

## Pendientes y alcance

MD-B1-ANCHOR01 bloquea aceptación/adopción. Los20grupos previos no son veinte testsPASS:10 positivos propios combinan varios criterios,1 negativoFAIL; slotcap/copiaframework/proof/publicPoint frontera se revisa estática sólo y no cierra toda geometría. No ejecución nueva de kernelPANIC/sendfailure b0 porque nueva interacción mantiene kernelbyteexacto y heredamos gate acotado, no claim nuevo test. Nestedhit expansión completa, search/chooser/Backremap/tabla/alt/resourcegráfica/copies/renderer son b2/b3 o integración posterior segúncontrato; pendientes, no falsa cobertura. B1 no equivale aCLI/pump/ConPTY/Unix/productocompleto. Ningún proceso/session propio activo al emitir. Fuente/productivosfreeze intactos.
