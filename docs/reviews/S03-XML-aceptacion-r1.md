# XML R1 — aceptación independiente funcional Windows

Dictamen: PASS en la cobertura ejecutada y revisada aquí. No cierra S03/RV-106, QA visual, contención/runtime Unix ni todo el backlog. No se encontraron fallos de producto en estos ensayos. Se preservó un error previo de guarda del arnés, separado del resultado final.

## Identidad exacta

Snapshot Cviz/r3viewer-s03-xml-review/R1, base49535cc: MANIFEST D4F6ABDEA5F9A59B79FB5FC8E920A360DB4603061C65987AC7F1839DBBAB8270,355entradas verificadas antes y después; app18927840E727B1778AB1BBA2808603551E48AB9822513C2E0A7512D17C6846AE, companionE885DE7F826DA9520EA7E6A11139F57F966A9BDB4E9CFEB5653A6BFDB6D5567E, módulo38CF6B9C06CACA991CAD43DD51E02834270713D2E40AFCE7D6E385E2F247768C. Sin modificación de producción, perfiles, plugin register ni GUI.

Oráculos originales:31fixtures/22rawspans; B5fixtures/9casos/6rawspans, ORACLE B A67DE394F0A0F0FF9C5FAF8CDEF603B476DFF7576A7E3F47D75CE71ED401F0D5. Todos sus bytes/hashes/rangos fronterasUTF8 y queries cooked fueron comprobados antes del producto; no se alteró ningún oracle ni se ajustaron categorías a resultados del candidato. Expatsmallreadys sólo se usó como corroboración normativa independiente de5fixtures sin DTD/resources.

## CLI, Nu y presentación segura

cli_nu.py:89invocaciones propias PASS.31fuentes default y31estructuras explícitas con estado/código y fallback entero; suplementos C1/error tardío/metadata/extensiones/UTF8/200k; selectores CLI local/literal/stdin, rutas reales.xml/.XML frente.svg/.html, errores2 de opciones/tipos y loaderUTF8error1; Nu privado String/Binary/Int/ByteStreamBinary/Unknown y ruta, más Readertypedwire String/Binary/Unknown. Ready exige xml_document; los31states/reasons y detalle semántico se cotejan adicionalmente mediante API real abajo.

Fuentes default y fallidas se comparan contra SafeText independiente de TODO el snapshot: CRLF/tab/newline preservados, C0/DEL visibles y C1→cadena segura. C1 legal anterior a error tardío no escapa como control activo, no se pierde el tail ni se publica prefijo estructural. Raw/spans siguen separados y conservan bytes originales.

ByteStreamString Nu REAL: repetición independiente del productor SDK privado de coordinador, source8175CA3341D2CD36635D33C712858108F945CA1D87BAB18E2C43B48B507C31C4, binCF595987E83295E4E55C96E6F6B3526A9B50F94BE038A7D70E6AF9C6ECE5078F, Cargo.lock idéntico R1 y fuentes originales companion cotejadas. Nu0.115.1 -n con dos plugins explícitos privados; nunca instalación/profile. Capture R3V1 Start decodificado por builtin frommsgpack acredita originbyte_stream string/binary/unknown, sin inferirlo de describeString. Payload propio `<e a="&amp;">stream SAFE</e>`: String→XMLReady0 y decoded&amp→&; Binary/Unknown→uso2 sin estructura.6comandos capture+viewer,3decodificaciones Start y archivos wire/stdout/stderr separados en nu-stream-independent. Productor fue escrito por root; lectura y ejecución presente son independientes. Typedwire aislado y builtin describe no sustituyen esta evidencia.

## Modelo exacto y28rangos originales

4tests NUEVOS independientes ejecutados contra la lib de fuente R1 congelada en copia privada, no contra prototipo ni oracles construidos por parser. Log independent-model-test.log,4passed0ignored:

1.31fixtures originales Ready/failure; reasons exactos InvalidXml, UnsupportedXmlEntity/Version/Encoding, y has_doctype para2ReadyWarning.
2.22spans originales por kind+UTF8span exactos y rawliteral: BOMdeclaration+element+atributo/refUnicode, quote/equal/EOL, CDATA/comment/PI/DTD completo, scope y emptyfrentepaired.
3.6spans B y cooked correspondientes: C1numref frenteplaceholderliteral, CDATAnoexpand, CRLFliteral→space en atributo y numericCRconservado, Text/CDATACRLF→LF.
4.Orden físico mixed2atributos+7contenidos con kinds/valores exactos y emptymark; namespaceQName/local/URI, atributo sin namespace, declarationQNamexmlns:p/bindingprefix y namespaceNone, rebinding/restauración de hermanos;5unidades no concatenadas del fixture no-cross.

Los tests usan exported xml_worker::project y publicnode/metadata/textref, generation17 y comparación source Arc exacta. Sólo wrapper privado recibió cfgtest independent_model; source/lock/modules originales permanecen byte-exactos. No valida por sí solo hostileIPC ni todo allocator; AE del exe ejercita controller/receiver vía CLI y ConPTY.

## Ocho sesiones Nu/ConPTY propias

7sesiones de conpty.py,44observaciones registradas, y1sesión adicional source_search.py; salidaq0/retornoNu/alt-screenleave confirmados y sin forcedcleanup final:

- C1atributoA no coincide con la cadena literal\u{009b} de atributoB; match elegidoB,8celdas resaltadas, o rawowner completo; queryneedle pasa A→B→CDATA y N vuelveB; Tab y resize65/110 preservan dirección/valor/domino.
- ab&/&cd/cdef ausentes al cruzar unidades; ab/&/cd/ef presentes en sus hojas.
- QName/LocalName/NamespaceUri separados con misma identidad física; QNamep:e→child, Qnamee→locale, URIroot, o selecciona Elementowner completo y Tab restaura campo.
- Helperlabels NamespaceUri/Element/NodeId/ordinal no generan hits por sí solos; Comment/PI/Decl metadata sí; PI targetgo en QName separado.
-200kcoincidencias de a, backward funcional y tiempos observados; no es prueba independiente exhaustiva de proyecciones ni SLA.
- Error tardío con C1 queda Fuente segura y latch tras4Tab, no estructura parcial.
- NUEVO Fuente raw: fixture `<e>C1real mid \u{009b} tail</e>` muestra dos placeholders visuales pero queryASCIIliteral devuelve sólo tail; n/Nwrap conserva tail, nunca el C1real. Tab→Estructura compara cooked igualmente y o vuelve al tokenText entero (C1real+literal); Tab restaura hit. Acredita matching original distinto de presentación y rawowner, sin escribir C1 como input de teclado.

El primer intento de7sesiones falló en guarda que exigía palabraValueDecoded completa en65columnas; header se recortaba legítimamente aValue. Se preservan pre-resize-ConPTY archivos y forcedcleanup de ese intento; se corrigió sólo la assertion de viewport: identidadpath+contenido en65, dominio completo restaurado en110. No cambio de oracle semántico ni defecto de producto.

## G4 NUEVA observación del parser de producción

g4-prepared/FROZEN-IDENTITY registra fuentes/módulos/lock exactos. Copia privada R1 añade únicamente dos[[bin]]; Cargo.lock y todos355archivos originales salvo Cargo.toml targets-only coinciden. Wrapper propio llama exactamente producción xml_worker::project, Ready0/Failure2, sin otro parser. Build --locked --offline PASS; binwrapper1F997F324E034C76EBF8CC188362CFDFA97804B5578A882D9B9B3E86B26D93AC y positiveFCCE5731F29837D86AE107D87BA790F1E2A8A8E262CD2CF9B85D3CEC6CB77D64. OBSERVATION-WRAPPER-FROZEN.rs conserva fuente observada antes de agregar tests privados; bin observado nunca se reconstruyó tras esos tests.

Owned NT monitor hash1E9811C02973FF545E85BF5122077076F04F50F314538D20381804F603C67A9D, mismo mecanismo acotado conocido. DEBUG_ONLY_THIS_PROCESS, hooks de NT-export file APIs/DeviceIoControl y connect/WSAConnect sólo del hijo propio,20s/256events; sin ETWglobal/driver/UAC/memoria ajena. Positivosantes/después detectan aperturaarchivo propio+loopbackpropio, listener recibeOWNED_POSITIVE_CONTROL.

7casos producción NUEVOS:baseline,DOCTYPEURL,DOCTYPEfile,PIstylesheet,XInclude,namespaceURL,xml:base; todos Readyexit0,0canaryopens,0connects y0AFDopens. Evidencia PRODUCTION-OBSERVATION.json. No se cita monitor del prototipo como cobertura del parser nuevo. Inspección de callgraph exacto project→parse→Reader::from_str/lexical/profile muestra sólo source/modelo, sin resolver/network/FS/unsafe/directsyscall/subprocess en rutas de producción; Command/Windowsunsafe posteriores son tests cfgtest, no llamadas desde project.

Límites: APIs observadas no son trazado kernel/ETW exhaustivo; directsyscalls/gaps por stepping/subprocesses no quedan cubiertos por el monitor. Callgraph y perfil de este wrapper justifican alcance concreto; adversario revisa G4 y sus límites. Esta evidencia renueva observación, no certifica todos los posibles recursos/inputs ni Job/lifecycle del worker real.

## Lifecycle de coordinador verificada, atribuida

Root ejecutó por separado xml-lifecycle-r1 REPORT SHA80ACC4FF22AF00049A610EAAB16B48ECC0C19B02CDC3304CA886FBC627B31612, ARTIFACTS SHA1BCA2AFAA27B09BE9B98A14D7CC9BA8B8E9A1525A0953FE2E3519966FBF9039A. AE verificó16hashes/bytes (incluidos3dependencies propias), guardas PID/parent/command/creation, FILETIME fin antes de creation+handles señalados, flags0x2100/512MiB por proceso y conhost propio; q0/C130/cancel→retry con nueva identidad y sin forcedcleanup/residualr3viewer en logs. No se atribuyen estas ejecuciones aAE ni a consentimiento visual. Timeout2,074s es observado con suspensión del propio worker, no SLA ni512MiBagregados. Adversario también recibe evidencia; root no aprobación exclusiva.

## Pendientes y límites de aceptación

HostileIPC completo e instrumentación/admisión de todos los máximos64MiB/1M/128MiB son evidencia A/adversario, no ejecución independiente de toda frontera en esta AE. No se ejercitó historia128XML en esta ronda ni todos los casos de interrupción dentro de búsquedas/graphemes extremos;tokens1MiB y máximos de arena mantienen el alcance de sus informes respectivos. EOFhostile worker, inline5deadline exacto, enforcement Job512MiB bajo parser real, RuntimeUnix/contención/distribución y QA visual de WT/WezTerm no se acreditan aquí. No igualar fuentescompiladas a píxeles correctos ni declarar producto/backlog terminado. Revisión adversaria global independiente sigue necesaria.
