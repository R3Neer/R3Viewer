# MD-UNC01 — corrección operacional A–C R2, propuesta R1

2026-10-06. Autor graphics_gate_experiment, adversario root. **Sólo propuesta, no implementación/adopción.** R1 real/source/bin/manifest84 siguen intactos; falla independiente UNC reconocida, no se altera su fixture/golden para obtenerPASS. D4 queda interrumpido con borrador preservado Markdown-code-Syntect-D4-WIP-before-UNC01.md.

## Defecto y autoridad

Fixture independiente wiki-unc-crlf.md: rawToken18..53 conserva DOSbackslashes iniciales; destination pulldown conservaUNO. expected(Some(original)) de R1 exige igualdad literal prefix/original, por eso devuelveInvalidContext y no publica modelo. No es corrección de oráculo ni falta de capacidad filesystem; el Source original sigue inmutable.

Fuente primaria instalada pulldown-cmark0.13.4, opciones35998exactas: firstpass.rs877..916 reconoce `backslash + ASCII punctuation`. Para pareja inicial backslash/backslash fija begin_text=ix+1/backslash_escaped=true; el primer nodo comienza en el SEGUNDO byte de pareja. parse.rs handle_wikilink887 toma firstbody_node.item.start, y scanners.rs935 scan_wikilink_pipe devuelve una slice RAW desdeese start hasta primerpipe. No aplica unescape genérico al resto del target. Es una pérdida de UNbyte inicial por origen del primer nodo, distinta del escape Obsidian TableCell antespipe. Con3/4backslashes iniciales el mismo primer nodo sigueenindex1; no colapsar todas parejas ni paresimpares. No usar identificación de UNC porhostname/extensión, ni interpretar targets comoIO.

Hashes primarios exactos: parse.rs 76FCA0F94731D660CF022E948FF119DAED1BDF446E813E4A86FAA742700483A5; firstpass.rs E66397B39AB0996CA6197D9476EB5DB833F92672ACDBDB855241F33346048917; scanners.rs 2CDCB3CC76BC3D6947EEC066EEFE4582B660DD52DA71D048F6DC515665291840. PerfilCore no produceeventosWiki y no admiteestaadaptación.

## Contrato pequeño nuevo solicitado para revisión

Sólo sobre evento REAL TagLink/TagImage cuyo LinkType es WikiLink, ProfileExtended, token/rangoUTF8checked y delimitadores whole [[...]]/![[...]]. Derivar primerpipe literal y prefix crudo; validarflagpothole iguala rawpipe paraLink; paraEmbed conservarreglaexistingraw. Validarowner/context anteseligibilidad, igualqueR1.

Clasificación estricta antescomparación:

1. Si ProfileExtended y prefix empieza exactamente con dosbytesbackslash (`0x5c,0x5c`), clasificar **FirstBodyBackslashLoss** del upstream fijado. Destination ORIGINAL debe ser byteexactamente prefix[1..]. No aceptar prefix[2..], alteracióninterior, normalización, suffixdistinto ni cualquiermismatch. Para esa clase tambiénrechazar original==prefix si contradice la dependencia fijada. Si enuncontexto concreto el upstream no cumplepredicción, la fixture quedaFAIL/investigación antesampliarclase, no se modifica regla porapariencia.
2. Para todootro prefix/perfil el originaldebeigualprefix entero byteexacto (reglaR1). Escapedpunctuation distinta, entidades, `%`,slashesinteriores/UNCcomponentes no habilitan normalizaciones generales. Coreleadingpair no entraFirstBodyBackslashLoss.
3. Sourceprefix COMPLETO, contodosbackslashes iniciales, es la autoridad semántica trasdemostrarla pérdida concreta. **No reconstruir strings por añadirbackslashes**: slice prestada del Sourceoriginal, cotaschecked.
4. Aplicar DESPUÉS la adaptación OBS-PARSE01 existente: sólo TableCell real+pothole y exactamenteunbackslash consecutivoantesprimerpipe elimina UNbyteFINAL delprefix. FueraTable se conserva tambiénese trailingbackslash. Dos/tres/cuatrotrailingbackslashes no seeliminan. Si elparser no generaeventoWiki porsemánticatable, no sintetizaruno ni recorrerregexglobal.

Resultado Examples: `\\server\share\Nota\|Alias` enTable→targetdosleading +resto `server\share\Nota` (sólotrailingúnico eliminado); mismoTokenfueraTable→targetdosleading +resto `server\share\Nota\`. Conleading3/4 conservar3/4 completos, aunqueoriginaldestpredichoquite1. No alias→sintrailingadapt; targetcompletoSourceprefix. Core/noevent/code/HTML noadaptación.

DatoDecoded del targetrestaurado usa OriginEnclosingToken delrawToken COMPLETO, no ExactTokenni offsetSourcefalso. RawToken Source/Exact, OriginLink/Resourcewhole, sourcehash/bytes, label/alt/styles/ordinals y snapshotidentity permanecen intactos. No corregirdisplayalias ni otrasEventText: estaadaptación tiene autoridad sólo sobre target. WikiEmbed width/height aliascontinúa reglaactual, no participar enreconstrucción. No cambiodiscriminantes/table/wire/flags.

## Delta concretamente implementable después dictamen

Crear copia privada R2 desde R1 verificado con baseline/resourcescontext completos; no mutableworkspace/freezeR1. Archivos productivos previstos:

- document_obsidian.rs: refactor expected en prefijo/FirstBodyLossproof→sourceauthority→CellPipeRemoval, todascomparaciones porbloques≤1024 y cancel/deadline; sinallocation detoken/destination. Target añade clasificación local no wire (o bool interno agregado) paraindicarOriginEnclosingrequerido porrestauración/sourcecorrection. `original:Some` esla frontera producer; `None` sólohost derivación de autoridad, nuncaexención devalidaroriginal enproducer.
- document_parser.rs: consumir resultado prestado nuevo para LinkyWikiImage antes b.text; forzarDecoded+OriginEnclosing delwholeToken cuando FirstBodyLoss oPipeRemoval. No tocarlosEventText/Spans/alias ni reparseFromDoc.
- validate_wiki en document_obsidian.rs/validatecaller: hostreconstruye target DESDESource raw+graph, noconsultarparsernioriginaldestomitido; exigeigualdad deldecodedtarget conprefixfullsource menosúnicotail elegible. ExigeDecoded/EnclosingWholeToken cuando rawclasificaciónFirstBodyLoss oPipeRemoval lo requiera, rechazando target1leading,origensubrange/Exactforged. Alias/dimensión yfullraw validatorsR1siguen. No disminuirhostvalidation paraaceptarUntrustedCandidates.
- Testspropios unitproducerproof/hostmutants + tests/document_pipeline.rs doscasosrealescontrasteLink/Embed table/parágrafo, fuentesprevias yactualsourceexpected. Cargo/lock/lib/main/worker/controller/Job sin cambios. Dependencia/upstream sin patch; se corrigeadaptación delproducto que conservaSource.

`.adapted` deR1 significa sóloPipeRemoval: no reutilizar su nombre para ocultarambos motivos; nombrar `requires_enclosing`/`source_corrections` tipo local cerrado (FirstBodyLoss/PipeRemoval), o equivalenteexplicado. HostNone noafirma que observó originalparserdest: calcula laexpectativa semántica y condición deorigen desdeSource/pinnedprofile; la aceptación originalproducer sucedeantesmodelo.

## Oráculos previos y pruebas pertinentes

24fixtures fuente propias, antes R2outputs, en markdown-unc-r2-source-oracles: leading2/3/4 ×Link/Embed ×Table/paragraph ×alias/nopipe, conUTF8 Álias/名, CRLF, sourceSHA/tokenrango ytarget fuente. EXPECTED-before-R2-output.json SHA256 **FA6B9845E558D87389BE8E9E797A77BD164DCFCBCE025297CA6DFA485BC761DD**. La columna originalparserdest se etiqueta **predicción source-derived**, no output candidato/oráculo universal delparser. El reportedeAEoriginal yfixture original permanecen independientes e inalterados.

| Familia | Expectativa fijada |
|---|---|
| UNC01 | ReprorealLinkTable leading2/trailing1/Strongalias: modelo ytargetdosleadingpublicables, alias/style/rangobytesorig/hashintactos. |
| UNC02 | MismoLink fueraTable: restauradosdosleadingpero trailing1 íntegro. |
| UNC03 | EmbedTable/paragraph: dosleadingrestaurados, wholeEnclosing, alias100x145/dimensiones/Runsaltintactos. |
| UNC04 | Leading3/4 y sinpothole: demostraroriginalminusONEinitialbyte, conservar3/4 completos; no paircollapse/parity regla. Si upstream discrepa, deteneresaexpectativa fuente antes ampliarcontrato. |
| UNC05 | Leading1/interior\/percent/entity/malformed: reglaR1 originalexact salvoclase específica; originalmismatch arbitrariorechazado. |
| UNC06 | Producer mutantsdrop2/changeinterior/truncsuffix/rawequalscontradictory: InvalidContext, no parcial. Hostmutants decoded1leading/forgedOrigin/subrange/cellforeign/flagcontradiction rechazados. |
| UNC07 | Trailing2/3/4 nuncaCellPipeRemoval; helperproof aceptaúnicamentedestoriginaldemostrado, snapshotrooteventsi/no conserva semánticaexacta sin sintetizarWiki. Core/code/HTMLinert noResourceWiki/LinkWiki. |
| UNC08 | Targetlargo/byte1024boundary/cancel yarenaoldnew: prestadosscan/comparemáx1024, cuotaadmisiónDecodedantesreserve sinclones. Hostcandidate/decoder mismoslímitesR1 y wirepassthroughsinnewfields. |

La aceptaciónajena vuelvearejecutar fixtureUNC original, no reemplazada por casos máscómodos. Checksautor posteriores sólo deltaObs/parser y regresiones pertinentes +fmt/clippy/buildbinexacto; sin workspacefull/SDKmegafreeze. Entrega fuente/R2bin/patchválido/manifiestodelta, firstFAILR1conservado. A–C no seacepta por autor y D4 no sereanuda mientrasesta críticapendiente.
