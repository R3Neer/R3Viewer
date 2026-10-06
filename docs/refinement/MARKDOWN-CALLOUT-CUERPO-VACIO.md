# MD-CALLOUT-BODY01: propuesta R4 congelada para revisión

Identidad AE final verificada antes de congelar esta propuesta: `markdown-callout-independent-ae-r3/Markdown-callout-independent-acceptance-r3.md` SHA175CA044FC2163F55BAC2724B0746C642FDCAF937F5421583961C7895E97BED4; OWN-EVIDENCE-MANIFEST SHA3C974999858447E11431408B22F2EA9AE64287532D6665BADC71763B3BEEF89E,24registros todos byteexactos. Expected previo SHA2CBAB3C39E0B4BCB921FF1381C638EAE9B51D4940A3FD8A026422534AF7FFA50. Sin código ni nuevo output candidato. Base: R3callout MANIFEST B075214C669F6251120D64DBD1B375C3D157EC9C3A48FB6A90650FE40DB025B5, y portable separado CB744E3C… no adoptado. Los tres freezes y portabilidad quedan intactos.

Fallo AE final (dictamen NO APROBADO; 11PASS+2MD-CALLOUT-BODY01+1HarnessError marker-onlylower, todos resultados intactos): `> [!note]\n> [](y)\n` y uppercaseNOTE contienen Link whole Source12..17, ningún Text/SoftBreak upstream. HeaderCount=bodyFirst=0, emptySpan0..0 satisface end<=headerCount y R3 inventa TitleDomain mientras debería mantenerse sólo ParagraphDomain del cuerpo. validate pasa porque la topología es válida: defecto semántico productor, no razón para aflojar validator.

Primaria instalada pulldown-cmark0.13.4/src/firstpass.rs230–267 (SHA E66397B39AB0996CA6197D9476EB5DB833F92672ACDBDB855241F33346048917): Options ENABLE_GFM reconoce BlockQuote(kind), consume hasta fin del markerline y rescanea contenedores del siguiente párrafo imaginario. Source y rangos permiten distinguir header/body aunque se omitan los runs y el ordinal se reduzca a0.

Cambio mínimo propuesto sólo document_parser.rs+document_pipeline.rs (respectoR3; sin wire/validator/Cargo/controller/Job/UI/quota): factorizar la decisión header/body de Span en un helper privado sin asignación y llamarlo IGUAL en el pase de necesidad owner y pase de remap. Guardas ordinales y de crossing previas se ejecutan antes de modificación.

Para span.start_ordinal<end_ordinal permanece exactamente criterio ordinal anterior end<=header_count Header, restoBody. Para EMPTY start==end, tomar Origin source probado del Span (raw whole token de inline, nunca targetDecoded): Header si start>=marker.title_start y end<=marker.line_end; Body si hay terminador de esa línea en Source y start>=next_line_start. next_line_start se obtiene leyendo como máximo2bytes en marker.line_end para LF/CRLF/CR; EOF no tiene body. Si no pertenece entero a ninguna zona (newline/crossing/originantesTitle), retener Quote original mediante salida antes de mutar. Un anchor vacío exactamente line_end pertenece a Header por límites cerrados y no inventa un body alEOF; los Source whole tokens de enlaces ajenos son no vacíos, por lo que no necesitan inferencia por anchors. En cualquier caso conservar guards anteriores para marcador/ordinal y comprobar bounds/UTF8 ya existentes, no aceptar Origin ajeno más amplio porque el ordinalsea0.

Retorno Header/Body de ese helper controla ambos booleanos owner y el mismo checked_sub(marker_count/body_first) posterior. EmptyBody con bodyFirst0 remapea0→0 sobre ParagraphDomain, Title NONE si no hay otros spans/runs reales de header. EmptyHeader de los3PANIC originales sigue TitleDomain0runs. BothEmpty mantiene owners separados y WholeSource origin distintos. Cuerpo sin runs con spanBody retiene ParagraphDomain/block y su Origin anterior, comoR3; no fakeRun ni cambio de slicingBody. No cambio de runs, estilo, linktarget/rawtoken/sourcehash ni constructorDocument. Cada owner real sigue ArenaVec cobrado al límiteactual; helper≤2Sourcebytes+O1 porspan, mismo poll, sin mapa porbyte ni allocations auxiliares.

Matriz de expectativas previa R4: fuentes ajenas lowernote+upperNOTE Link12..17→labelDomainKind0/bodyParentCallout/TitleNONE/runCount0; variantes CRLF link13..18 y EOFsinnewline (deriveSourcebytes); nativeotros markers IMPORTANT/TIP/WARNING/CAUTION, custommarker conservaBody; cuerpo normaltext y enlacenonempty sin Title artificial. Original3PANIC titleEmptyLink+body/headerOnly/StrongemptyLinkTitle→TitleDomain0runs; bothEmpty y CRLFUTF8→title/bodyowners separados, targets/rangos/parentStrong mismos. Quote control sinmarker y crossingStrong/Link permaneceQuote. Marker-only pulldownBlockQuoteSomeNote sinParagraph permaneceQuote, no nueva recognition. Raw delimitadoresvacíos no sintetizan spans.

Mutantes de modelo permanecen rechazo por validator unchanged: domainsNONE/wrongowner/label_count1sobre0runs/origoutSource. Helper privado controlesSourceorigin antesheader, newlinegap, crossing y EOF no se convierten en fauxTitle/Body; se conservaQuote. Comparar frontend events fijados por dependencia original, no derivar expected de salidaR4. Caso source-only que refute clasificación previa debe reportarse antes expandircode.

Checks focalizados: PrimaryOffset de nuevos body-only y marcadorconsumido; unitclasificador+mutantes y parserregresiones13; worker reales body-onlylower/upper+variantes y original3/bothEmpty con SOURCEorigin exacta; fmt/clippy/buildprivadoincremental. No repetir Source128MiB/deadline/OS/contención ni broadworkspace. Mantener FAIL MD-CALLOUT-BODY01 y previos; nuevo snapshot pequeño2sources+bin+evidence/patch válido, noadopción compartida hasta adversario+AE. La propuesta no cierra buggeneralcallout hasta R4observado.

## Identidad de las dos fuentes independientes

[
  {
    "case": "lower",
    "source": "> [!note]\n> [](y)\n",
    "source_size": 18,
    "source_sha256": "A1C9FFD8CFF9422DAD6E0DF79B9CD9BE6D1D5A80342BDA7FDAC16287143F2CD5",
    "raw_range": [
      12,
      17
    ],
    "provenance": "source bytes; lower event equivalence by case insensitive scanner, not separately printed runtime"
  },
  {
    "case": "upper",
    "source": "> [!NOTE]\n> [](y)\n",
    "source_size": 18,
    "source_sha256": "B9E9657FE4D9593F37DB6B59542E71A3DA08495E42B88AEC420D0C6C1169CA15",
    "raw_range": [
      12,
      17
    ],
    "provenance": "runtime UPSTREAM link12..17 targety, no text/softbreak printed"
  }
]

El evento uppercase12..17 sinText/SoftBreak se imprimió enruntime. Lower12..17 proviene de bytes fuente y equivalencia por scanner case-insensitive, no evento impreso aparte; no convertirlo en observaciónruntime inventada. Scanners.rs215–252 SHA2CDCB3CC76BC3D6947EEC066EEFE4582B660DD52DA71D048F6DC515665291840 yfirstpass230–271 SHAarriba verificados. El harness marker-onlylower conserva su FAIL por expectativa impropia, no se contará como nuevo PASSR3 ni nueva prestaciónR4. Diagnóstico añadido al sufijo después delruntime no se ejecutó; esta propuesta no lo trata como prueba.
