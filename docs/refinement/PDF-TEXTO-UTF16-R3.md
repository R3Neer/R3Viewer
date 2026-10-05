# PDF UTF16/mapping — contrato operacional R3

Autor unix_r3_review; **pendiente adversario root antes de código/runtime**. Responde U16-P01/P02 de revisión R2 EE589FDEF8C5C62140386788FAFA7B372038BE6EB941B49C12580C20DFABCF50. R2 183973E0AC067139DCA85A1398E5B6CF8D4296D321074002AA4AEDFE8DE40D95 y matriz18IDs 32BDE8DF418AC12A7B963C183E1E0C6AC3D8F7DC22B0112BFCE5810E15BAE5FD se conservan intactos. No cambia PDF Rust R4, source fixtures ni FAIL históricos. No completa G0/G1/RV ni integra producto.

## Fuente y adquisición

PDFium source commit91b9d569b34be4f38eed7b3c49b227356c3aadad (chromium/7881), archivos/hashes en PINNED-ACQUISITION.json. GetPageText:591–637/GetText:341–372; mapasCPDF_TextPage:410–431; ToUnicode/ProcessTextObject/GetUnicode causas ya evidenciadas. DLL privada R4 x64 SHA79D4676B656CFB1ABCEA88F9ADE3B4B0826C5200382DB5F4EC72A636C598C118. Receipt estático B3FB47E79CCD97F2FCCA8C85F7950CB4E0BC3F2F193748A93F2A9A9514BE009A confirma exports, no ejecución/ABI: FPDFText_LoadPage/ClosePage/CountChars/GetText/GetUnicode/GetCharIndexFromTextIndex/GetTextIndexFromCharIndex/GetCharBox/IsGenerated/HasUnicodeMapError; FPDF_GetPageBoundingBox y FPDFPage_GetRotation. No nuevo downloader/floating library ni DllMain. Export no acredita procedencia del build; esto sigue pendiente separado.

Entrada source≤1MiB, documentPages1..256, request página física1-based válida y countN≤65536. Count -1/error oN>límite es Failure antes reserve/GetText. Una llamada GetText(0,N) paraN>0, bufferu16[N+1], máximo131074B inicializados tras preadmisión; r debe1..N+1 ybuffer[r-1]==0. U=r-1. Payload sólo[0,U); NUL interno NO terminador. N=0 omiteGetText y tiene r=0/U0. Page/textpage/document/source-lifetime y serializaciónPDFium segúnRAII previo; sourceborrow hasta documentdrop. Cancel externo conserva control de llamada bloqueada. No llamadas GetText nativas porchunk, por riesgo solapamiento al saltar no imprimibles. Rust decode enbloques≤1024 unidades/carry1, cancelantesbloque y enloops mapping/cajas. No `char::from_u32(GetUnicode(i))` como decodificador scalar; dato raw sóloevidencia.

## Dominios y tipos

N nativeindices, U unidades UTF16extraídas, S scalars, B bytesUTF8. U≤N, S≤U yB≤3U. Raw i32 se serializa conbitsu32 LE, se reinterpreta i32 antes comparación; nunca castnegativo a índice. NONE=0xffffffff para IDs ausentes. NativeIDs0..N-1/unitIDs0..U-1/scalarIDs0..S-1 densos. UTF16span yUTF8span half-open; índices físicos no son intervalo ni offsets source. Todo éxito ydiagnóstico lleva `semanticState=0 Unverified`, único valoradmitido. No SourceComplete/SourceNoText en wire.

### Tabla Native, stride64B

Prefix8u32 LE (32B): `id,rawUnicode,rawTextIndexBits,rawGeneratedBits,rawMapErrorBits,boxState,nativeRelation,reserved0`; luego cuatro f64LE `left,right,bottom,top`(32B). No padding ni Rust enum layout. Arrays residentes dewords32 ybox64 porrecord tienenstride64/alignment8; serializerLE explícito. `rawUnicode` puedeu32arbitrario (inclsurrogate), no se exige scalar porque expresaGetUnicode. Generated/MapError rawint válidos0/1; -1 APIerror yotros invalidreturn se conservan, noTRUEporcast.

boxState0 NoBox: GetCharBoxretfalse, no leer outparams, wirecuatro+0bitsexactos. 1 BoxValid: rettrue con4valoresfinitos/ordenados left≤right/bottom≤top. 2 BoxInvalid: rettrue yalmenosno-finito odesordenado; conservarbitsraw, nunca usarpara geometry. No BoxInvalid convalidnumbers. CajaOriginal es user-space PDF, no crop/rotation aplicado. Generated puede tener rawboxválida, pero no se acredita glyph físico por eso.

nativeRelation0 Exact: rawTextIndex t≥0<U yUnit[t].rawIndex=id yUnit[t].relation=Exact. 1 Excluded: rawTextIndex=-1 yninguna Unit conasociación Exact refiereid; no se genera scalar paraesteíndice. 2 Invalid: otra combinación/rango/error/collision. Excluded registra cobertura de capa, no afirma razón semántica de fuente ni UnicodeMapErrorfalse. Relaciones se verifican bidireccionales alfinal.

### Tabla Unit, stride16B

`id,rawUnit,rawIndexBits,relation` cuatrou32. rawUnit≤65535, fiel alpayloadGetText; rawIndex deGetCharIndexFromTextIndex(t). relation enum0 Exact,1 Missing,2 OutOfRange,3 RoundTripMismatch,4 DuplicateIndex,5 NativeCallError. Prioridadnormativa: si rawIndex=-1→Missing; otro<0 o≥N→OutOfRange; si Generated/MapError deltarget retornanerror/enumdesconocido→NativeCallError; si másdeunaunit rawIndexrefieremismoiválido→DuplicateIndex (ambasunidades); siNative[i].rawTextIndex!=t→RoundTripMismatch; enotrocasoExact. Duplicate yvueltasevalidanconcountsN arrayscharged. rawMapError=1 no invalida la ubicación pero marca calidaddesconocida delUnicode; relaciónpuedeExact con flagMapError enScalar ysummaryIncomplete.

Unit exacto define posiciónactual, no automáticamenteglyph legítimo. Nunca insertar índiceguessed siMissing. No se omitenunidadesfallidas para dar apariencia deComplete.

### Tabla Scalar, stride40B

Diezu32: `id,unitStart,unitCount,utf8Start,utf8End,native0,native1,mappingState,geometryState,flags`. unitCount1(BMP no-surrogate) o2(parhigh+lowvalidado). Native0/1 son **conjunto ordenado según posición de unidades**: paraExact Unit deeseoffset, su rawIndex; paraotrarelation,NONE. ConunitCount1,native1NONE. Para parunitCount2, se permiten5/7 o7/5 si lasdosvueltas APIseacreditan; NO se incluye6 ni se llama span[5,8). Duplicado mismoíndice nuncaExact; rawindicesseconservan enUnits aunque asociación scalar invalide. Par conunidadMissingmantienesólo asociación acreditada, no substituye una conotra.

mappingState0 Exact (todasUnitsExact);1 Incomplete (algunaMissing yningunaerrorotra);2 Invalid (algunaUnit2..5). flagsbit0HasMapError=1 si algúntargetExact rawMapError1; bit1HasGenerated=2 si algúntargetExact rawGenerated1; otrosbits0. Estados derivadosporhost, no confianza en flagsdelworker. Source alwaysUnverified aunquemappingExact.

geometryState0 Available: mappingExact, flags0, todascajas asociadasBoxValid. 1 Partial: mappingExact/flags0 ymezclaBoxValid yNoBox/Invalid. 2 Unavailable: resto (inclnoasociaciones/MapError/invalidmapping), salvo3 Generated si flagsHasGenerated. Precedencia Generated→Unavailable(errorflags/mapping)→Available/Partial/Unavailable. Partial nohabilita highlightscalar exacto; sólocajasindividuales comoevidencia. No derivedbbox duplicada: vista futura puede calcular envolvente sólo deconjunto completoAvailable, etiquetada derived/enclosing; no inventarmediacaja ni copiarOriginalBoxaíndiceexcluded. Caja false/Invalid se conserva comoNoGeometry, no aborta textoUTF16válido.

`unitStart/count` particionan[0,U) sin gaps/overlap cuandoValid; `utf8Start/end` particionan[0,B), decodeUTF16 ycodepointUTF8exactamenteigual enambosdominios. NUL scalar esU+0000/unbyte0, se preserva. TextSearchhitsutf8 sólo boundariesvalidadas yscalars completos, devuelve orderednativeIDs yGeometryState/page. No offsetPDFsource ni mappingescalaresinventado.

## Resultados y errores

Begin.resultStatus0 ReadableLayer: U>0/decodeválido/S>0/B>0, inclusoMappingIncomplete/Invalid oGeometryNone; host publica texto legible con capacidadeslimitadas ydiagnóstico tipado. No palabraComplete sincalificador yno highlight sinAvailable. Status1 NativeEmpty: U=0/S=B=0; Npuedeser0 o>0 conNativeaudit/exclusiones; label explícito capa extraída vacía, NO certificarPDFsintexto. Status2 InvalidUnicodeDiagnostic: UTF16malformado, U>0; S=B=0, firstInvalidUnit válido ytodoNative/Unitrawseconserva. Es diagnóstico completo delensayo, **no ArtifactReadable ni prefijode texto**. Sourcefallbackpermanece. rCount/returnedNULinválido no produceStatus2: FailureContract.

Begin.mappingSummary0 Validated: todasUnitExact, todosNativeRelationExact/Excluded, Generated/MapErrorrawretválidos yMapError0. 1 Incomplete: existeMissing oMapError1 sinningúninvalid;2 Invalid: cualquierrelation2..5/nativeInvalid/APIreturninvalid. EnU0 mappedSummary seclasificaconNative, no por vacuidadciega. GeometrySummary0 AllEligibleAvailable (S>0, todosScalarAvailable);1 Partial (algunoAvailable/Partial yotrosfallos);2 None (resto;inclGeneratedsolo); hostderiva. NativeAudit0 AllClassified sóloNativeExact/Excluded yrawGenerated/MapErrorválidos/mapError0 yningunaUnitinvalida/missing;1 Issues enotrocaso. Es auditarrecordsdeestaAPI, no completitudsemántica de fuente ni prueba contra truncadoparser.

Clasificaciones mapping/box fallidas **no rechazan por sí mismas wire fiel**: puedenReadableLayercondiagnóstico, conservartexto válido ydeshabilitarnavegaciónespacialnoacreditada. Host rechaza protocolo/counts/ranges/relaciones/summarymentirosos, UTF8noequivalente aUnits, estadoInvalidUnicodeconprefixS>0, NaNconBoxValid, boxstateNoBoxconoutparamsbasura, percent/sourcefabricada oSemanticState!=Unverified. Errornativefatal, load/page/textpageerror,Count<0,GetTextretornoimposible/finalNULmissing, input/resourcecap/reservefail, timeout/cancel/stale oIOreceiptfailure producen Failure/hostError, nuncaArtifactvalidado.

### Page metadata40B

Dosu32 `rotationBits,pageBoxState` +fourf64 `left,right,bottom,top`. rotation native rawFPDFPage_GetRotationdebe0..3 parausartransform, otroinvalid metadata deshabilitavisualmapping; no reemplazarpor0silenciosamente. PageBox usaFPDF_GetPageBoundingBox, FS_RECTForiginalf32promovido exactamente af64 yreordenadocanónico; boxstate0FalseReturnzeros/1Validfiniteordered/2Invalid. CajaOriginaldelNative sigueunrotateduser-space, no dividirporviewport; Cropoutside no excluye text/searchsilenciosamente. PageMetasummary no se mezcla con geometryAvailableoriginalBoxes: consumer transform puedeUnavailableporrot/cropmetadataaunquecajasoriginalesValid. Testsrotcrop mantienen expectativasgeométricasprevias.

## Wire fijo y publicación

Header32B LE:magic`R3UT`4/versionu16=3/kindu16/generationu64/requestu64/bodylenu32/reservedu32=0. sourcehash yIDs comparadosconrequestHpropio; enum/lenboundsantesreserve, noJSON/Debug/terminalliteral. BootstrapownchildJobbeforeSTARTsegúninvariantesR4, inputactualsourcebytesno path/callback. START/control1byte separado no implicaGIPCactor extra.

| Kind | Body |
|---|---|
| 1 Init H→D | profileu32=1,flagsu32=0,pageu32,maxPagesu32=256,sourceLengthu64,SHA256source32,reserved8 :64B |
| 2 SourceChunk H→D | offsetu64 +bytes1..65536; contiguous :9..65544B |
| 3 SourceEnd H→D | lengthu64+SHA256source32 :40B |
| 10 Boot D→H | wireVersionu32=3,profileu32=1 :8B |
| 11 Begin D→H | words[16]u32 +sourceLengthu64+SHA256source32+totalWireu64+reserved16+PageMetadata40 :168B |
| 12 Records D→H | tableu32,firstu32,countu32,strideu32 +count*stride :16+recordbytes |
| 13 Utf8Chunk D→H | offsetu64+bytes1..65536 contiguous :9..65544B |
| 14 End D→H | N/U/S/B cuatrou32,resultStatusu32,flagsu32=0,totalWireu64,SHA256payload32 :64B |
| 15 Failure D→H | codeu32,stageu32,indexu32,rawReturnBitsu32 :16B |

Beginwordsorden: profile,page,documentPages,N,U,S,B,resultStatus,mappingSummary,geometrySummary,nativeAudit,semanticState,getTextRetBits,firstInvalidUnit,reserved0,reserved0. N=0 getTextRet0; N>0getTextRet=U+1. firstInvalidUnitNONEsalvoInvalidUnicodeDiagnostic, entoncesexactfirstfaileddecodeposition (lowaislado; high+nonlowposhigh;highEOFposhigh). DecodeValidnoStandaloneSurrogate. SourceLengthhashpointeridentitydocPAGEtodoscoincidenhostsnapshot/request. reservedbyteblock16todo0.

Tablas1Native64,2Unit16,3Scalar40; countsbeginderivanN/U/S. Records canonicalbatches1024salvoúltimo, sincount0, firstcontiguo ytableorden1,2,3; tablascount0sinframe. Utf8chunks65536salvoúltimo yB0nochunks. Sourcechunks65536salvoúltimo y source0noChunk (PDFvacío → loadFailure). Worker Boot es primero en output. Begin sólo tras Init, SourceEnd, stdin EOF y hash comprobados. La fuente es bytes PDF arbitrarios; NO se impone validación UTF8 al archivo PDF. SourceEnd length/hashcalculado porD. Extracción requierepage/invocaciónsintrailingrequest.

RespuestaNormal=Boot+Begin+Tables+Utf8Chunks+End+stdoutEOF+exit0; Failure=Boot+Failure+EOF+exit0 sóloANTESBegin. DespuésBegin unaexcepción/allocfail terminaexitno0/protocolfallo, no mezclarFailureconparcial. Host quarantineaStatus2comodiagnóstico yStatus1empty; ninguna evidencia deReady/CompleteSource. PayloadhashincrementalSHA256 deBodiesBegin+Records+Utf8Chunk, noheaders/Boot/End. Endcounts/status/total/sourceIds/hashexactantesEOF, trailing1B/extraEndinvalid. RecibosIOstickydewriterwrite/flush/close/panic yreadcapture/error/overflow se muestrean DESPUÉSjoins; waitexit0/ownhandleclosed+3streamsclosed/readerswriterterminados antesúnicocommitfinalH. Cancel/stalelatchalcommit, no reset trasEOF. Job/Pendingleasesretainedblocknewspawn; retrycleanup mantieneerroreshistóricos, neverrescue forPASS.

Failure.code cerrado: 1 InputLimit, 2 DocumentPageLimit, 3 PageOutOfRange, 4 LoadFailure, 5 NativeAPI, 6 GetTextContract, 7 Allocation, 8 OutputLimit, 9 InternalInvariant. Failure.stage cerrado: 1 Input, 2 LoadDocument, 3 LoadPage, 4 LoadTextPage, 5 CountChars, 6 GetText, 7 Decode, 8 Mapping, 9 Geometry, 10 Serialize, 11 Reserve. Code1 sólo stage1; code2/3 stage2/3; code4 stage2; code5 stages2..6; code6 stage6; code7 stage11; code8 stage10; code9 stage1..11. Index es índice native/unit sólo si la etapa8/9/7 lo acredita; en otras etapas NONE. rawReturnBits sólo retorno i32 registrado en etapas2 (CountPages), 5/6 o un valor raw fallido en7..9; en las restantes0, nunca direcciones de pointers. StatusInvalidUnicodeDiagnostic ya describe UTF16 malformado: no codificarlo como Failure con prefijo. Fallos de mapping/caja declarados en los records son qualified status, no Failure. Cualquier discriminante/combinación incompatible se rechaza antes de publicar. Failure diagnosticado requiere input válido y receipts finales; stderr no sustituye código tipado.

## Fórmulas exactas y ledger — respuesta U16-P01

M=64N+16U+40S+B. Admitir0≤N≤65536,0≤U≤N,0≤S≤U,0≤B≤3U antesreserve; Status2exigeS=B0. M≤123*65536=**8060928 bytes**. No arraysassociation/cajasadicionalocultos: Scalarcontienehasta2IDs, cajasúnicasNative; Unitsrawincluidos. Cadaintegerproducto/sumchecked, comprobarcountscontraexpected beforealloc.

qN=ceil(N/1024),qU=ceil(U/1024),qS=ceil(S/1024),qB=ceil(B/65536). **OutputTotal=336+M+48*(qN+qU+qS)+40*qB**. Fixed=Boot40+Begin200+End96. Máximo(qN+qU+qS)=192,qB=3; upperbound **8070600B<8388608B(8MiB)**, margen318008B. No quotas rebajadas yno cajasomitidas parafit. Begin.total/End.total exact derivado counts antesreserve; actualframingcontador+hash confirma, unknownstride/batches fail. Input≤source1MiB+40*16+Init96+SourceEnd72=1049384B, START1byteafuera. No reserve8MiBporBegin.

Controlledrequestedheap agregadoH+D≤64MiB, partición fija **H40MiB+D24MiB** antesrequest; sourceperactor1MiB incluida, sourceHpreservada yDborrowhastaRAIIdrop. FuenteprivadaArc<Vec>readonlyevitafullcopyinHwriter; DunaVecsource. Dpartición: 2M≤16121856 para worstpeakarraygrowthold/new (suma2tables noallcopiesthird), rawbuffer2*(N+1)≤131074, mappingcounts/work≤5*N=327680, source1MiB, control1MiB. Suma **18677762B<25165824B**; vectorovershoots/transientheadroom6488062B cobrado antesreserva, no gratis.

Hpartición: previousArtifact≤Mmax +newCandidate≤Mmax (old+new), optionalwirecapture≤8MiB, validationscratch≤16*(N+U+S)+4096≤3149824, source1MiB, control1MiB. Máxima **29757440B<41943040B**, headroom12185600B paracapacityovershoot/newallocationtransient cobrado. No proof-tablecopyfueraestareserva; scratchbitmap/rawmapcounts remap dentroarraycharged, notHashMap/Stringperindice. SioldArtifactmayor queMmaxnoadmitiresteperfilsinrevisar; no dropoldpara fingirReady. SiwirecapturaevidenceStreamfileenvezmemoria, stillretaindeclaredcapbutno doublereadmodel.

Control1MiB poractor cubre encode/decodeframe≤65552B+header32, read4096,stderr65536,queues≤16slots+262144Bpayloadsum, hash/bootstrap/error/smallstack≤65536, restoheadroomprotocolo. Nadieusa1MiBframesarbitrarios: bodyallowedmax65552, Nativebatch65536+16=65552; UTF8Chunk65544. Scratchdecodecarry1sinfullUTF8clone. Arrayresidentstrideigualwirevalidationcompiler; `try_reserve_exact` antesresize, capacity*stride real se cobra; crecimiento old+new debeadmitirseantesreserve. Unexpectedreportedcapacityovershoot consumeheadroomsiadmitida; sinheadroomdrop/rechazo yregisterobservedpeakallocationfailure, jamásbudgetPASSporpostdrop. RequestedallocationRustnoesallocatorusable-size/RSS; defaultallocatorunknownnativecommitseobservaporJob512MiBporproceso, noagregado64MiB. Allocations internas de PDFium se registranExternalNativeUnknown bajoJob/cancel; no reclamar64MiBtotalRAM. Peak/tables/source/IO logs ownledgerproveweights vsactualcapacityparaesteperfilantesaceptación.

Deadlineparser/FFI/globalrequest10s, no renovar con progreso. Cleanup intentosindependientes kill/wait/joins/closes conplazo2s; sinproofPendingretainsauthorityyblocknewrequest. Read/write/captureoverflowfatal, no silentdropgreen. Errorloggingaftercleanupbest-effortbounded. ContenciónWindowsreadback/bootstrapasig Job512 siguecheckreal conownhandles, no certificadoheredadoporusarR4; nativeblockedstub no pruebaGetTextPDFiumlatency. Unix/runtimevisual/rendererproductonoautorizadosporpropuesta.

## Matriz preservada y añadidos preoutputs

18IDs R2 intactos, ningúnoutputnuevo. Añadir **U16R3-A** modelo Unit[t0]→5,Unit[t1]→7;Native5.textIndex0,6=-1,7.textIndex1 con rawpairD83D/DE00: Scalar asociacionesordered[5,7], nunca[5,8); OriginalBoxes5/7preservadas, 6excludednohighlight. **B** pair[7,5]siinverseconsistentordered[7,5], mismoUnicode. **C** segundoMissing→native1NONE/MappingIncomplete/GeometryUnavailable, validtext😀seconserva. **D** duplicaterawindex5ambos→DuplicateIndex/ScalarInvalid/noimaginedspan. **E** mappingAPIout-of-range yGenerated/MapErrorraw-1/2 explícitos yNativeIssues. **F** statusInvalidUnicodeconS/Bprefixrechazado,diagnósticoN/UcompletoS=B0aceptadocomodiagnósticonolegible. **G** StatusReadabletextcorrecto+NoBox/InvalidBBox/maperror mantienevalidtextperoShapeCapabilitiesdeshabilitados. **H** quotasformula8070600/8388608+1 headerlying/reserveactualcapacity/oldnewfail/scratchlimit. **I** UnitNULinteriorylastNULgatecontraste r4vs3/finalterminator0missing, ningúnestrlen. **J** CountsS>U/U>N/B>3U/tablasfaltantes/flagsmentirosos/Unknownstate/noendEOF/IOlate/cancelbeforecommit rechazan.

Oracleoriginal yfixturesPDForiginales no se regeneran parareclasificarsingleTJFAIL oastralesR4. NuevaAEdeberáfreezeinputs/PDF/sourceexpected/typedstubcases antesoutputs. SiPDFium no secomporta comomodeloderivadosources,registrarFAIL/refutacióntécnica, noactualizargoldenparaaprobar. U16R3stubcasesnoafirmanmappingsnonadjacentPDFiumrealhastaejecutarse.

Root revisaR3 antesautorprivatebuild; AE distinta ysource/binsmanifest después. Estecontratocierra layouts/strides/relations ytiposdeerrorpropuestos, noautocierre deU16-P01/P02. SourceUnverified yFAILs originales permanecen.
