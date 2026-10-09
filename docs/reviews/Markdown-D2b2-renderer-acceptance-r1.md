# b2: aceptación adversaria del freeze R1

2026-10-09. Dictamen: **SIN_CRITICAS_PENDIENTES dentro del corte b2 de biblioteca textual, roles, referencias/recursos seguros, tablas y renderer revisado**. Los cuatro grupos propios pasan, incluidas las fronteras previamente previstas. No acredita pump, b3/actions/history/search compuesto, gráficos, D4, Unix ni QA visual/nativa; esas obligaciones del producto continúan vigentes.

## Candidato y procedencia

Freeze oficial FREEZE-b2-r1.json SHA256 E55ED3D66A52791ADF3A9DDFE2DEBB4384F135FFAAB07B2ADDD4CF9C5DCCEBFD; B2-SOURCE-freeze-r1.zip SHA256 BF75A395756370EFB59B26941D3B78FA123BBE8BCFFA456E3C126087DFBA67F9. Base privada7a7b9a76e322edc1cf852614c9bf9755ac795953, código353418c19f2152614d5650d605aa05df72a1cbca; composición posterior del canónico corresponde a root, no se sustituyen módulos ajenos por este snapshot.

Extracción propia product-freeze-r1 verificada contra los724 registros, cero diferencias. Después de AE los723 ficheros sin modificación siguen byteexactos; document_renderer.rs conserva prefijo original SHA18BF804B73C7419FCB756917CB7764B7F6CC5EE394ED4E572CE0A67B8C6BF5BE. Sólo inclusión cfg-test añadida al final y nuevo src/adversary_b2_tests.rs. Código productivo no modificado. SourceLegacy text_engine.rs SHAE0D3F08E7A386FD66EC39D941C9355E0997C47BE70CA1660EA2E69EB9EF59B3C coincide.

Oráculos originales A01/A02 EB3E3FCE0C8E9FADFD1A9E4A998A114C2CF60DB47881C87254E3DF8B7D76B9BA, harness previo44EB10D0095A0865A0A684F77162E83ECD27E7A0FDB891AF057298AC98AC64F2 y EXPECTED-own-b2-before-output-r2.json SHA1434D0D86FA3E78859D32E535045999C843003599C4024F72A55B4F7760F3DBD permanecen intactos. Ese JSON registra estado PREVIO a compilación/runtime: no se reescribió retrospectivamente. La ejecución completada se registra separadamente en EVIDENCE-own-b2-final-r1.json SHA3E814F5953CFCA972B9DD11BC7576ACBCA304540003FF0C41CFBB183A5724D0F.

## Runtime propio

Primera compilación offline/locked --lib --no-run,37.56s exit0. Sin adaptación API ni cambio de expectativas: ejecutable copiado y hasheado ANTES del runtime, bin/own-b2-attempt1.exe SHA581E2888BCC821237C5BD56CB518CC37532A9CB3D1317B88F0B90D4C3412714B. Filtro exclusivo document_renderer::adversary_b2::, --test-threads=1 --nocapture:4PASS/0FAIL/429filtered,0.06s,exit0. Bin previo del autor preservado antes de reutilizar caché; no se usa para atribuir nuestros resultados.

Suplemento focal autorizado de fronteras, fijado antes de su output, conserva los mismos cuatro grupos: EXPECTED-own-b2-supplement-before-output-r2.json SHA52020F55F844AB4109B86EBE77C84DE2CF48A89384ED9A0B91885CF98583C489; harness E3058A3122E24FB7BABDDDA232AD54F07AA57B6FAD261B71B5C4439020DEE1AD. Segunda compilación12.77s exit0; bin separado y SHA antes de runtime bin/own-b2-attempt2.exe SHA1E48AB07B93A08DF0AB9ABB97B31EB156B192429E5A57B097E479E9359C2A42A.4PASS/0FAIL/429filtered,0.36s,exit0. Ambos binaries/logs/manifiestos previos preservados y hashes releídos después.

Fuentes sintéticas propias, parser/validate/quarantine reales y factory con join físico. Backend propio intercepta tuples y entrega el iterator al CrosstermBackend nativo; captura/log acotados son memoria exclusiva del harness, no cargos de producto ni evidencia ConPTY. TERM=xterm sólo proceso hijo. No suites generales ni casos32MiB nuevos. Los119 controles del autor y133 de composición root son evidencia separada, no nuestros tests.

## Resultados funcionales

1. Orden Item0 A/sublistaB/textoC/Item1D, Heading, Quote, Callout abierto/cerrado, Rule, Footnote unused, Code y HTML inerte conservado. Fold real por ID del Document hace visible HIDDEN y cambia signo; cierre vuelve a ocultarlo. Heading usa bold+RoleHeading, marcador Secondary, ColourNever mantiene caracteres sin modifier/color. Hscroll left1 excluye CJK completo y conservaAB; End conservaTAIL.
2. Tabla tres columnas/filasH1H2H3/ABC/DEF completa en80 y18. Celdas desiguales52/2/20 navegan ventanas cortas hastaTAIL conservando cada payload real una vez. TableResume en width30 conserva byte8 I y Done de uv; width66 empieza porI y89 del tercer campo, no byte20. Cambio a vertical18 conservaI. Cierra B2-A01 en código/runtime.
3. FootnoteLABEL52 pasa todas sus ventanas hastaBODY/TAIL. Ref glyphs no fabrican FirstComponent; etiqueta se conserva íntegra y resize intermedio retiene posición. Cierra B2-A02 junto constructor RefOwner de bloques/recursos y validación provider real inspeccionados.
4. Pending Image/MathInline/MathBlock/WikiEmbed/Mermaid completos en orden, alt a+combining multiestilo atómicoBold, entity del title segura y ESC original ausente de symbols. Mermaid mantiene además código original navegable; no carga de recursos ni éxito gráfico. Draw real Textab→WholeReferenceWidthZero→EndTAIL funciona. Cancel final antesIO deja captura/updates vacíos y owner vigente; Geometry0 conserva Reference tipada sin BackendIO.
5.220x alternando estilos produce >176Runs genuinos, sigueText y draw real bajo cuotas. Cluster exactamente16384B produce un glyph entero, heap real16384 y Consumer vuelve al baseline tras Drop; cluster16385B es WholeReferenceSymbol y después EndTAIL recuperaText.
6. Dense4920B produce1440X/4800X en80x24/120x40. Costes de etapas comprobados mediante fórmula independiente de los cargos declarados; held no sumado a spent. Exact effective Work acepta, exact−1 rechaza sin metadata retenida. Redraw cobra oldpayload adicional en diff y no vuelve a emitirX.

|Geometría|Layout spent|Held draw|Spent inicial|Spent redraw|
|---|---:|---:|---:|---:|
|80×24|417972|2936064|2661268|2662708|
|120×40|1373775|6553344|7417231|7422031|

Geometry65535×65535 entrega WholeReferenceWindow acotada. Framework prepare rechaza antes de asignar grids; el renderer/draw rechaza sin BackendIO y conserva capacities4×2 anteriores. Ningún cargo se redujo ni cuota elevó para PASS.

## Revisión estática focal y límites

Inspeccionados FrameWorkReceipt/LayoutWork, renderer/owner, nuevo Flow context/copy_current, WindowBuilder/growth/sidecars/anchors, table local, Ref/Resource/roles, View requests/validation/commit y extensiones TextRef provider. Roles/NoLiteral se derivan de eventos Flow reales y phases, sin Source/Run ficticios; RefOwner valida campo/ID/UTF8/affinity antes Geometry. CellAnchor usa provider local/projectionpart y sidecar≤32B, no offsets de symbol como Source. ALT conserva Domain real. Table At/Done queda local/charged, no subfila reinterpretada ni scan global de filas. Sólo current+candidato de provider; copies/capacidades reales con cargos old+new y Work antescopy.

Fallback tras fallo layout conserva mismo spent y reserva retenida; cancel/invalid no se convierten en Reference tardía. Renderer consume ventana admitida, no traversal/provider; receipt valida coste real<=held y total8MiB. PreviousFrame cerrado y readbacks reales, rollback current-only y poison Backend permanecen. La preparación de geometría/índices sigue fase separada expresamente declarada: no se acredita como SAMEWork layout+draw.

Errata documental congelada: SURFACES-b2-r1.md escribe Operation::Presentation(PresentationAnchor), pero la API real es Operation::Presentation SIN payload; DocumentInput captura la ancla real de ventana/current. Se contrastó en source y runtime pasó sin adaptación. No se corrigió el archivo frozen ni se añadió fábrica arbitraria.

No quedan críticas accionables dentro de este corte. La aceptación permite considerar el delta b2 en integración local conservando el canónico posterior; root resuelve composición/adopción. No se anuncia RV-301 ni producto completo por esta aceptación.
