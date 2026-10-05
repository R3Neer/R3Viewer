# Documento Markdown — arena y wire R2

Autor refinador: unix_r3_review; adversario designado root, distinto del autor. **No aprobado ni ejecutado.** Sustituye exclusivamente el borrador R1 SHA94E645C678AD073B7FADE11B9627C7ADB3986054CA51D08A6AFCB7C5A08D0B3F, intacto. Plan macro canónico `docs/refinement/MARKDOWN-DOCUMENTO.md` y gate pulldown0.13.4 R3 aceptado permanecen. Sin código/Cargo/lock, producción, S04, perfiles ni corpusClases. Unicode activo no se toca. Wire apto para implementación sólo DESPUÉS de resolver la revisión de este contrato; no es aprobación de contención/Unix/layout/gráficos.

## 1. Invariantes, fuente y cantidades

IDs y offsets de tablas son u32 LE; NONE=0xffffffff; IDs densos desde0, el campo id debe igualar posición. Offset de source/decoded y tamaños se computan checked en u64/usize antes de cast; ninguna cantidad puede alcanzar NONE. Rangos son half-open. Empty text válido start=end, distinguible de NONE. Strings son UTF8, permiten controles/NUL en modelo; jamás se emiten crudos al terminal. No IO/callback/HTML/código/renderer durante parse o validación.

Profiles cerrados:0 CommonMark optionsbits0;1 Extendido optionsbits35998 exactamente, como gate. Otros rechazados. No habilitar features nuevas por un discriminante del modelo. Extendido habilita YAMLmetadata, GFM callouts, math/wiki/tasks/table/footnote/strike; plusmetadata/headingattributes/definitionlists/super/subscript no forman parte de estos perfiles. FrontmatterPlus se reserva como capacidad de modelo, se rechaza en wirev2 actual. No parser serde/html/SIMD implícito. Adaptaciones Obsidian contextualizadas sólo sobre eventos/fuente, no regex global.

Nblocks B, Ndomains D, Nitems I, Nruns R, Nspans S, Nlinks L, Nresources G, Ntextrefs T, Norigins O, Nattributes A, decodedbytes C. K=B+D+I+R+S+L+G+T+O+A ≤1,000,000 (decimal), B≥1. Profundidad bloques y spans combinada contextual≤64 (Rootdepth0); maxDepth declarado≤64, host recalcula. C≤64MiB y también entra ledger. Fuente lógica≤128MiB, completa siempre disponible aunque Document falle. Números son candidatos de ingeniería para este incremento, no límites nuevos confirmados al usuario ni recorte de Markdown complejo. Exceso devuelve LimitDocument manteniendo Source íntegra, no prefijo.

## 2. Tablas de datos cerradas

Todos los campos siguientes son u32 (wire+resident array de words u32, alignment4); no enums Rust/String/Vec por record. Reserved siempre0. `size_of` residente debe coincidir y se exige comprobación compile-time; serializer escribe LE explícito, no memcpy de structs. No padding/booleans implícitos. TablaKind/stride:1 Block64,2 Domain24,3 Item16,4 Run64,5 Span40,6 Link64,7 Resource80,8 TextRef16,9 Origin16,10 Attribute24. Formatos completos:

| Tabla | Campos en orden (cantidad u32) |
|---|---|
| Block64 | id,parent,firstItem,itemCount,domain,kind,flags,aux0,aux1,aux2,aux3,origin,reserved[4] (16) |
| Domain24 | id,kind,owner,firstRun,runCount,reserved (6) |
| Item16 | parentBlock,ordinal,kind,target (4) |
| Run64 | id,domain,ordinal,kind,styles,text,origin,link,resource,aux0,aux1,reserved[5] (16) |
| Span40 | id,domain,parentSpan,startOrdinal,endOrdinal,kind,link,origin,reserved[2] (10) |
| Link64 | id,kind,flags,targetDecoded,rawToken,titleDecoded,referenceLabel,labelDomain,labelStart,labelCount,origin,targetOrigin,reserved[4] (16) |
| Resource80 | id,kind,flags,inputDecoded,rawToken,titleDecoded,origin,altDomain,widthKind,widthValue,heightKind,heightValue,intent,ownerRun,aux0,aux1,reserved[4] (20) |
| TextRef16 | kind,start,end,origin (4) |
| Origin16 | relation,start,end,reserved (4) |
| Attribute24 | ownerBlock,ordinal,kind,key,value,origin (6) |

Arenabytes wire/residente de tablas W=64B+24D+16I+64R+40S+64L+80G+16T+16O+24A+C, checked. No otra tabla variable escondida. No parent/siblings copiadas extra; items son los edges ordenados. Aux none usa NONE sólo para campos ID opcionales especificados; campos numéricos/enums ausentes0. Un ID heterogéneo sin diccionario se rechaza.

### 2.1 TextRef y Origin

TextRef.kind=0 Source,1 Decoded; start≤end≤sourceLength/C respectivamente y límites UTF8. Source debe representar EXACTAMENTE el texto semántico: worker sólo lo escoge tras igualdad con substring source, host valida rango y relación con Origin; hash prueba snapshot, no prueba que un parser hostil interpreta bien. FFI/worker no es autoridad semántica: fixtures/AE cubren elección de texto.

Origin.relation=0 ExactToken,1 EnclosingToken; start≤end≤sourceLength, UTF8. ExactToken significa ubicación exacta del token completo original, NO equivalencia decoded/scalar. EnclosingToken significa contenedor acreditado para payload con subrango exacto no disponible. TextRef.Source exige ExactToken con el mismo rango y substring probado igual; TextRef.Decoded puede tener ambas relaciones a token distinto, sin mapping escalar a source. Todos los TextRefs tienen origin válido, incluso vacío; recursos/links tienen token original completo Source por `rawToken`, con Origin.ExactToken. No proporciones. Un decoded entity `&` apunta al token `&amp;`, no a supuesto byte& original.

Si se necesita representación sintética sin source exacta (espacio de layout), NO agregar TextRef inventado al documento: es fragmento de layout futuro, fuera wire. Soft/HardBreak conservan sus tokens y no un TextRef. Source sigue dominio separado de selección/search documental.

### 2.2 BlockKind y auxiliares

kind0 Root;1 Paragraph;2 Heading;3 Quote;4 Callout;5 List;6 Item;7 Table;8 Row;9 Cell;10 Code;11 Rule;12 Footnote;13 HtmlInert;14 Frontmatter. `flags`0 para todos en esta versión; estado se codifica sólo en aux. OriginRoot=[0,sourceLength); parentRoot=NONE/id0. Otro parent válido y único. `domain` sólo según tabla; cada Block que admite runs tiene exactamente un DomainBlock aunque esté vacío. Los contenedores sin runs tienen domainNONE.

| Kind | aux0,aux1,aux2,aux3 | Hijos de bloque / runs directos |
|---|---|---|
| Root | 0,0,0,0 | todos salvoRoot/Item/Row/Cell; no runs |
| Paragraph | 0,0,0,0 | ninguno; runs |
| Heading | level1..6,explicitIdText oNONE,0,0 | ninguno; runs; atributos sólo explícitos acreditados, no slug |
| Quote | 0,0,0,0 | bloques de flujo; no runs |
| Callout | kindText,titleDomain oNONE,fold0Always/1Open/2Closed,markerOrigin | bloques de flujo; no runs; kindText no vacío |
| List | ordered0/1,startu32,tight0/1,0 | Item; no runs; unordered start0, ordered conserva0 también válido |
| Item | task0Absent/1Unchecked/2Checked,0,0,0 | bloques de flujo y runs intercalados; task state readonly |
| Table | columns≥1,0,0,0 | Row; no runs |
| Row | header0/1,0,0,0 | Cell; no runs; exactamente columns del Table; sólo primera fila header1, exactamente una header |
| Cell | align0None/1Left/2Center/3Right,0,0,0 | ninguno; runs |
| Code | style0Indented/1Fenced,infoText oNONE,languageText oNONE,mermaidResource oNONE | ninguno; runsCodeText; Mermaid es resource opcional, código original sigue íntegro |
| Rule | 0,0,0,0 | ninguno; sin runs |
| Footnote | labelText,0,0,0 | bloques de flujo; no runs; no generar ancla/resolver aún |
| HtmlInert | 0,0,0,0 | ninguno; runsHtmlText; no ejecutar/reparsear como Markdown |
| Frontmatter | syntax0Yaml/1Plus,0,0,0 | ninguno; runsCodeText; Plus prohibido perfiles actuales |

Flujo=Paragraph/Heading/Quote/Callout/List/Table/Code/Rule/Footnote/HtmlInert/Frontmatter; no Root/Item/Row/Cell fuera padre permitido. Frontmatter sólo hijoRoot primero, fuente marcador acreditado. Un callout sólo deriva del primer contenido de una Quote bajo perfil/caso contextual confirmado; no interpretar secuencias `[!...]` en Code/Html o párrafo ordinario. `markerOrigin` delimita marcador completo, títulos son DomainCalloutTitle, opcionales. Marker reconocido se sustituye por rol estructural, rawSource siempre conserva sintaxis; desconocido/no aceptado sigue Quote/Paragraph literal íntegro. Fold es intención inicial no una modificación fuente.

Atributos kind0ExplicitId,1Class,2KeyValue; propietario Heading, ordinal desde0, key/value TextRef válidos (keyNONE en0/1); perfil actual prohíbe A>0 porque no habilita headingattributes. Permite wire/modelo cerrado para perfil futuro, pero habilitarlo requiere nueva revisión/perfil, no aceptación automática. Sin diccionario operativo de HTML/settings.

### 2.3 Orden mixto, domains, runs y spans

Domain.kind0Block→ownerBlock.domain reciproco;1ResourceAlt→Resource.altDomain;2CalloutTitle→Block.aux1. Domain.firstRun/runCount particionan TODOS runs exactamente una vez, en orden de Domain.id; run.domain/ordinal coherentes. DomainBlock sólo para blocks indicados. Cada Block.firstItem/itemCount particiona TODOS items por Block.id. Item.kind0ChildBlock o1Run; ordinal0..count-1. TargetRun debe pertenecer a DomainBlock de parent. Cada runBlock aparece en un Item exactamente una vez y orden creciente por Run.ordinal; cada block noRoot aparece ChildBlock exactamente una vez, parent reciproco. Así `a` + lista anidada + `b` conserva orden sin “hijos primero/runs después”. Alt/title no son items visibles del Block ni copias de texto principal.

Run.kind0Text,1InlineCode,2SoftBreak,3HardBreak,4FootnoteRef,5Resource,6CodeText,7HtmlText. styles bits0Emphasis=1,1Strong=2,2Strike=4,3Code=8; otros rechazados. Text/InlineCode/CodeText/HtmlText/FootnoteRef requieren TextRef; Soft/HardBreak y Resource textNONE. InlineCode styles incluye8, CodeText8, HtmlText0. FootnoteRef text=label, sin resolver/ancla fabricada. `origin` requerido. Aux0/1=0 siempre; task es BlockItem. Resource run usa resourceid, y Resource.ownerRun=id; otros resourceNONE. `link`=linkid sólo si run está en su SpanLink, y todos runs cubiertos deben coincidir; no LinkMarker duplicado. Link/Image no disparan IO.

Span.kind0Emphasis,1Strong,2Strike,3Link. Span.start/end son ordinales half-open de un Domain; 0≤start≤end≤runCount. parentSpanNONE o padre mismoDomain que contiene estrictamente la secuencia/nesting; iguales rangos permitidos para nesting real, parentid menor para evitar ciclos. Spans de hermanos disjuntos o adyacentes, nunca cruzados. Profundidad de InlineCode no se interpreta como style parseable; enclosingSpan de énfasis alrededor de code sí legal. Stylebits derivados OR de spans activos+Codekind, host recalcula (no inferir nesting sólo con bits). SpanLink.link requerido; otro linkNONE; no links anidados. EmptySpan legal si token acreditado (`[](x)`), mantiene enlace vacío sin run visible ni fake label. Orden determinista: Span por Domain,start ascendente,end descendente, profundidad ascendente; ties por token source start/end y orden de evento. Si coincidencia exacta sin orden distinguible, conservar orden del parser con parentid menor.

DomainAlt/title admiten Text/InlineCode/SoftBreak/HardBreak/FootnoteRef; prohíben RunResource para evitar recursion de alt. Eventos nested-image que parser emita se convierten a texto alt conservado con Origin.EnclosingToken de su token, no pérdida de label; expectativa previa debe probar esa proyección. Spans estilos admitidos; SpanLink en alt/title sí conserva destino legible pero no IO. Todo runResource aparece sólo DomainBlock. Code/Html no tienen spans/recurso inline; mermaid bloque se asocia sólo a BlockCode. Todos Spans/Links/Resources alcanzables desde ownership; no extras libres.

### 2.4 Links y recursos

Link.kind0Inline,1Reference,2ReferenceUnknown,3Collapsed,4CollapsedUnknown,5Shortcut,6ShortcutUnknown,7Autolink,8Email,9Wiki; `flags` bit0WikiPothole sóloWiki, otros0. Map exacto de LinkType de pulldown; tipos Unknown conservan información, no IO. targetDecoded TextRef literal parser (SIN percent-decode extra/normalización), rawToken es Source token completo link; titleDecoded/referenceLabel opcionales. labelDomain/start/count exacto de SpanLink y viceversa; no duplicar label en blob. Link.origin tokenSource; targetOrigin puedeNONE si sólo se acredita contenedor del uso; referencia resuelta usa LinkDef.span (fuente original definición completa) EnclosingToken para el destino. `targetDecoded` con Origin.EnclosingToken de definición cuando no se obtiene subtoken exacto; raw target está preservado dentro raw fuente de ese contenedor, NO afirmar que rawToken del uso es destino literal. RefDefs fuente0.13.4 expone dest/title/span; validarlo contra source y no pedir callbackbrokenlink. Wiki mantiene `#page=`, alias y destino exacto; no interpreta permiso ni rutas aquí.

Resource.kind0Image,1Mermaid,2MathInline,3MathBlock,4WikiEmbed. flags0. InputDecoded requerido (target parser paraImage/WikiEmbed, código/math payload paraotros); rawToken Source token completo; titleDecoded opcional sóloImage/WikiEmbed. origin tokenSource. altDomain requerido Image/WikiEmbed aunque vacío, NONEotros. Width/height.kind0Unspecified o1Pixels; Unspecifiedvalue0, Pixels1..u32MAX-1 preservado como intención NO permiso para allocrenderer. Dimensión wiki sólo se reconoce en alias formato decimal `W` o `WxH`, ASCII completo checked; otroalias se conserva alt sin dimensiones, no autodetectar a partir de texto cualquiera. No porcentaje/unidad nueva en v2. Mermaid/math widthheightUnspecified.

intent0Inline,1Block; MathInline0,MathBlock1,Mermaid1; Image/WikiEmbed0 inicialmente según evento inline (un párrafo compuesto sólo imagen sigue imageninline dentro párrafo; layout futuro puede ampliar por rol sin cambiar source). Mermaid.ownerRunNONE y BlockCode.aux3 reciproco, exactamente un blockCodeFenced y lenguaje acreditado `mermaid` ASCII case-sensitive; inputDecoded todo payload código original semántico, rawToken cerca completa source. Otras Resources.ownerRun válido RunResource reciproco. Aux0/1=0todos. Raw destinations/fragments no se normalizan; clasificación PDF/nota y página aún resolverposterior. Source completa; rendererfuturo recibe identidad completa generación/origen y RenderArtifact, no wireANSI.

## 3. Construcción y validación acotadas

Worker convierte events a records; perfiles/spanbalance antes serialization. No copies/String por record; CowStr parser prestado o copiado a decodedpool tras admisión. Mantener eventos Text consecutivos; no coalescer por coincidencia de largo. IDs/runs reorganizados al final por Domain.id con remap explícito charged, no sort que cambie orden inline; model semántico antescount no puede usar buffers invisibles.

Host valida primero tamaños, luego ranges/enums, luego ownership/orden/grafo y spans por sweep iterativo. DFS bloques stackmáximo65 y visited2bits/block; un contador de indegree por block se reutiliza después DFS, no HashMap1M. Run-use bitset, domainownership y reciprocidad recorren arrays sin estructura anidada. Scratch máximo calculado conservador **16*K+4096 bytes**, además de model; semántica/counts ya garantizanK≤1M. Se pueden usar arraysu32 de longitudK particionados y bitsets dentroesa reserva; si algoritmo necesita más, diseño incompatible, no allocateextra. Registro de errores no lista1Mstrings: código cerrado+tabla/id/offsetu32 de primer error. Sweep inline stack65, expectedStyles/setLinks valida spans laminares por domain en O(K) sin comparar cada par. Relaciones Source/Decoded UTF8 se validan límites y cada campo; una tabla de UTF8 boundaries1bit/byte NO se permite fuera64M: usar `is_char_boundary` O(1) sobre&str validado, no materialización source.

Rechazar falta/multiplicidadparent, selfcycle, orphans, wrongdomain, itemtarget repetido, depth65, linkSpan discrepante, resources huérfanos, attrs en perfilactual, IDsNONEdondeobligatorios. Referencias hacia atrás/adelante lícitas entre tablas, pero sólo validaciónfinal admite Artifact. SourceSpan/decoded token containers pueden solaparse; NO imponer partición de Origin/TextRef.

## 4. Ledger de memoria, fuente y biblioteca

**64MiB por actor de peakallocation controlada de arena**, no total conjuntohost+worker ni RSS. W+scratch+builder/remap/transient buffers+capacityovershoot+old/new≤67108864 en host y enworker separadamente. Host Begin preadmite W+16*K+4096 más recepción/queues/scratch antes cualquierreserve. Sólo tablascontiguas de stride declarado; `try_reserve_exact` no garantiza capacityexacta: tras reserve medir capacity*size, cobrar diferencia y rechazar/drop antes llenar si supera. No coexistir stagingframe table completa yVecclone; framefixedsize buffers cargados. Growth decoded/workerarrays: reservar peakold+new antes reserve, devolverold sólo tras proofdealloc; no `Vec::shrink_to_fit` para fingir pico menor. FinalcountsW solo es límite inferior, no prueba allocation real.

Reserva fija transporte poractor **3MiB** dentro64MiB: scratchencode/decode1MiB, boundedqueue256KiB, readbuf≤64KiB, stderr64KiB, header+bootstrap/hash/control≤64KiB, metadata/cancel/smallstack≤64KiB, resto margen paraunframeenproceso+backendRust. Cadaqueuepayload se cobra porcapacity junto slots≤16; bytesacumulados≤262144; put no puede ocultarfull. Si el frame de1MiB se reserva juntootro1MiB utiliza reserva, no4MiB presumidos. Fórmula de admisión mínima Begin: **W+16*K+4096+3145728≤67108864**; no implica que1Mrecords64Btodosadmitidos: límitebytesmanda. Bigranges decode no duplicarC entero; pools a partir dechunks.

SourceSnapshot host preferido `Arc<Vec<u8>>` wrapped privado inmutable conUTF8validado (sin APIsmut/&mut/Arc::get_mut alconsumer), avoids Vec→Arc<[u8]> fullbytescopy. Si integraciónactual usaotroSourcecontainer, transiciónbytesold/new se cobra enledgerSource aparte y se prueba; no prometer zero-copy. Límite128MiB es logicalinput; sourcecapacityreal y old/newdelinputloader se registran fueraarena64M. Worker mantiene una solaVec sourceUTF8borrow, sinArc extra ni clones. Es **otra copia**, sourcehost+sourceworker hasta256MiB payload simultáneo; no64M ni128Mmemoriaagregada. Source writer Arc lease se libera tras close/join (tambiénerror), sourceDoc host permanece; SourceEnd escrito solo NO prueba drop. Parser/borrow se destruyeantes workerSource; code no permite referencesdolifetime másallá.

**Allocations internas de pulldown NO quedan acreditadas por ledgerarena**: constructor/syntax trees/refdefs/CowStr propiedadbiblioteca pueden reservar sin estecontador. Job512MiB committed por proceso worker validadoefectivamenteantesSTART es protección externa distinta; allocfallo/abort fuentegrande→WorkerFailure/sourcefallback, noCompletedocument. Registrar memoriaJob/requestedknown y allocationlibraryunknown separado. Para afirmar64M totalworkerRust habría queinstrumentarallocatorglobal/fijarbibliotecabudgetaware enotrogate; esteplan NO lo afirma ni omiteeseunknown. Parserallocations prestadas que persistencuandoarena existe siguen bajoJob; drop parser tanprontolaadaptacióntermineantes serialize. El1M/depth64pools no protegeconstructor, sóloJob/cancelexterno.

Viejo Document+newCandidate+validate dentro mismo64Mhost (old+new). Si no caben: mantenerSource/oldview, diagnosticarLimitDocument; nocerraroldanteshaberCandidatevalidatedparaevitarfalsoéxito. Historial/cache extra necesitará ownconsumerbudget antesretenermúltiplesmodelos; no seautorizaexclusióninformal. Controlpeak512Job puede fallarindependientemente64arenaknown.

## 5. Wire v2 completo

Header32B: magicASCII`R3MD`(4),versionu16=2,kindu16,generationu64,requestu64,bodyLengthu32,reservedu32=0. LE; headerIDs debencoincidirconinvocación; request/generationcheckedincrementnooverflowreuse. Framebody≤1MiB, allowedlengthantesalloc. Headerlongitudexacta; truncación/wrongversion/unknownkind/reservedtrailinginvalid. Input/outputdirectionsdiferentesenumkinds; unconstrainedmessagesnuncaintercalan.

Bootstrap fuera wire: workerpropio nace bloqueado1byteSTART encontrolcanalown; supervisorownedChilddesdeantescualquierfallo, verificaJob configuración/asignación/readback antesSTART, enUnixUnsupported hasta ejecuciónnativa. ElSource aúnhost; nodatadelfile/redworker. START no consume stdin sourceframes ni terminalteclado. Setupfailurelimpiaownhandles/streams/process sin liberarparser. ProductoChildJob actual no acredita readback porreusarlo.

| Kind / dirección | Body exacto |
|---|---|
| 1 Init H→D | profileu32,flagsu32=0,sourceLengthu64,SHA256source[32],reservedu64=0 :56B |
| 2 SourceChunk H→D | offsetu64+bytes1..65536; canon65536salvoúltimo; offsetcontiguo :9..65544B |
| 3 SourceEnd H→D | sourceLengthu64,SHA256source[32] :40B |
| 10 Boot D→H | profileCapabilitiesu32=3,wireVersionu32=2 :8B |
| 11 Begin D→H | profileu32,flagsu32=0,sourceLengthu64,SHA256source[32],Counts[11]u32,maxDepthu32,totalWireu64,reservedu64=0 :112B |
| 12 Records D→H | tableKindu32,firstu32,countu32,strideu32 + count*stridebytes :16+n*s |
| 13 DecodedChunk D→H | offsetu64+bytes1..65536 :9..65544B |
| 14 End D→H | Counts[11]u32,maxDepthu32,totalWireu64,SHA256payload[32] :88B |
| 15 Failure D→H | codeu32,tableKindu32,idu32,sourceOffsetu32 :16B |

Countsorder B,D,I,R,S,L,G,T,O,A,C. maxDepth exact recomputed. Failurecodes1Parse,2ArenaLimit,3DepthLimit,4RecordLimit,5InvalidContext,6Allocation,7Internal; sourceOffsetNONE si noacreditado, table0/idNONE si noapply; nofreeformtext. Invalid inputproto causes proceso fail/exitno0, stderrboundedcode; noreplytrustedFailuretrasinputinválido. Runtimecancel/timeout hostclasifica distinto sinrequerirFailurebody.

Sequence input Init,Chunksoptional,SourceEnd,stdinEOFsinextra; workercompruebaUTF8/hash/EOF antesparse. SourceLength=0nochunks. Boot sólopostSTART, primerooutput; nooutBeginantesinputEOF. Response: Boot+Begin, Recordsde tablas1..10 ascendentes (counts0noRecords), DecodedChunks,End,stdoutEOF. Recordscount1..2048 ybodyfits1MiB; firstcontiguo, neverduplicate/gap, tableexactstride yrecordcountDeclaradosantesfill. Batchcanonical2048salvoúltimo; maxstride80 ⇒163856Bbody. No1MiBbatchporrecordtable; decodedchunk64K. Alternativefailure Boot+Failure+EOFexit0, sinBegin/payload; failedafterBegin noFailureframe: terminateexitno0/candidatoquarantineinvalid. Publication neveraBegin.

SHA256payload sobreconcatenaciónBodiesBegin+todosRecords+DecodedChunk (sinheaders, sinEnd/Boot); incluyeoffset/count/stride ydecodedchunks. InputhashtripleInit/End/computedigual. OutputtotalWirecontabiliza **todos bytes outputinclheaderBoot/Begin/End**. FinalCRCno; SHAprocedencia noespruebasemántica. Parentrecomputaincremental mientrasvalidarelframe, no copiafullwire. Endcounts/id/hash/total exactoantesEOF, cualquierbytepostEndinvalid.

Sea n_j counttabla y s_j stride, batchesq_j=ceil(n_j/2048), decodedchunksq_c=ceil(C/65536). **OutputTotal=40+144+120+Σ(n_j*s_j+48*q_j)+C+40*q_c=304+W+48*Σq_j+40*q_c**. Fixed Begin112/header32=144;End88/header32=120;Boot8/header32=40. Checked≤128MiB=134217728 antes Begin/reservas; Failurewire=40+48=88. R1 contaba input+output en128M, imposibleadmitirsource128Mconheaders: R2 separaoutputtransport128M e **InputTotal=sourceLength+40*ceil(sourceLength/65536)+160**, máximosource128M ⇒134299808B; Init88+SourceEnd72=160. ControlSTART1secuentafija aparte contabilizado. Fuente128Mseguiráaceptada porinputwire, aunqueparse/modelbajoreservaspuedediagnosticarlímite. Noproductmaxnew128M-headersinferido.

Preallocreceive sólofixedheader32, bodylengthbeforeallowlist, count/stridecheckedbeforebatchfill. No reserveDeclaredtransport128M niJSONfull. Queuesbothdirections≤16slots+256KiBcapacity sum; readers/escritordeadline global candidato de 10s, sin renovarlo con progreso; stderr64KiBtotal/sanitizeddiagnosticlocal, chunkread≤4096,linecap4096 sólosi newlineparsernecesario; wirebinaryreader neverreadline. IO receiptssticky: capture/write/flush/close/panic/overflow/finaldecodeerrors alfinalDESPUÉSjoins, deadline de parse/transporte 10s; después, cleanup tiene plazo separado de 2s para intentar kill/wait/joins y clasificar Pending si falta prueba. No publicar éxito por expirar cleanup ni confundir esos 2s con SLA UI. Cancel/stale latch before/end/EOF/reap yúnicocommithost, noreviveconprogress/emptyerrors. HiloJobobservernochildforeignPID; sóloownhandles. Jobclose/kill/wait/joins/streamclose seintentanTODOSsinloggingomitcleanup; Pendingleases/source/child/streamsretainedyblockspawnnext hastaRetrycleanup proof. Reaped requierewait+exit+IOproof, no pollsolo niEOFsolo. Rescateexterno registraFAIL, nuncaPASS.

## 6. Casos/aceptación antes de código

EsquemaJSON/constantes/testlayout derivadosdeestecontrato, oráculossemánticos independientes deencoderdecoder. Freeze expectativasantesoutputs. Repetir21gatecasos+16AEpreviossinconvertirDebuggolden; map actualrecordexpectedSource/Decoded/Origins/graph/domainstyles/resources. Casos nuevosprioritarios:

- ItemmezclaText-subList-Text, paragraphs+quotes; unrunprincipalunaaparición yaltlabel noimpresoextra. Emptylink, emptyimagealt, nestedemph/strongmismosrangos, crossingSpan/linknestedinvalid. Styledlinklabel/imagealt yaltwikialiasvsdimensiónmalformada.
- Texto&entity/escape/CRLF/code normalization, fuenteexactinclusoNUL/ESCsourcepreservada; declararSourceconmismolargoperootrosbytesfalla; ExactToken!=mappingescalares. ReferenceLinkDef.spanorigenfuerauso permitido yUTF8checked.
- Calloutcustomtitlefoldprimercontexto, code/HTML/no-markerfallback; mathinline/displayraw/fencedmermaidretenciónsource/resources; WikiPDFpageoriginal+ambiguitypending. No metadataasIOpermission.
- CountsK1M±1,W+scratch+transport64M±1 ycapacityovershoot, growtholdnew, source128Mexact/inputheaderscálculo,scratch16K+4096exact. Batchcount/strideunknown/overflow/offsetgaps/tableorder/decodedtrailing/hashEndwrong,EndsinEOF/exitno0/latewriterclose/stale/cancelPendingnextspawnblock.
- Blocksgraphcycle/depth65/unreachable, duplicateitemtargets/disjointpartition, resourcealtlinkownerwrong/refNONE/attrsprofileforbidden. Max0/countscastnegative senderbytes u32max siempreboundsbeforealloc.
- Blockingparserconstructor/next con cancel externo; proveJobown readback/assignfailure/testsimpostores, nativeUnixnoPASSdesdeWindows. MediciónmemoriaKnownArena vs source vs libraryunknown y512Job, no asumir64Mtotal.

Riesgos retenidos: allocatorlibraryunknown/Jobfail grandesource; Sourcecustomadaptaciónconactualloader yperfilcalloutadversario; restricciónauxaltprojectiondebeprobadaantesclaim; renderedheadingindex/Obsidianresolver/HTMLDOM/layout/cache/graphicsprocessor quedan etapasmacro, noTODOdelwire. Ninguna capacidad enriquecida se cierra por wirevalidado. Root puede criticar estecontrato completo antesautorproductivo; ningún campo nuevo queda delegado a “TODO implementación”.
