# Oráculos prospectivos de HTML R2

Importado del candidato privado `ORACULOS-HTML-R2.md` SHA256 `617767C3263AEDE31FB036ACF9643C5260937C9C1E8FDB4105474C432D3A1F6C`, normalizado a LF. Fixtures sintéticas byteexactas en `html-static-fixtures/`. Son expectativas previas, sin runtime ni resultados PASS. [Contrato](HTML-STATIC-DOM.md).

# Oráculos HTML estático R2, anteriores a outputs

2026-10-10.20fixtures propias guardadas antes de cualquier runtime HTML. No parser/conversor ejecutado, ningún modelo derivado del candidato se usa como golden. Estos son ejemplos del contrato propuesto para revisión, no aceptación ni pruebas implementadas. EXPECTACIONES dependen de aprobar perfil/salida R2; cualquier cambio posterior conserva originales y agrega delta.

## Árbol reparado, texto mixto, attrs y raw

H01 mixed.html: bajo html/body implícitos, p[data-v=1] children Text(a), b{Text(b)}, Text(c), en ese orden. head también presente. View texto abc con estilo b sólo segunda letra. source.raw28B exactos, no newline añadido; cada node owner distinto, sin sourceRange exacto inventado.

H02 attributes.html: elemento HTML div, attr id=a primero y title=A&B segundo. id=b duplicado no reemplaza primero; raw conserva ambas grafías y entity literal. Namespace HTML. raw42B, attrs con orden DOM, no promesa de reproducer spelling literal.

H03 table-repair.html: texto x fostered antes de table; tbody implícito/tr/td(Text(y)); z después de table. View orden x,tabla y,z. raw26B intacto; reparaciones no desplazan raw ni generan spans inexistentes.

H04 format-repair.html: p contiene b{Text(a),i{Text(b)}}, i{Text(c)}, Text(d). Los dos i son nodos distintos en mismo owner; fuente25B no equivale a serializar ese árbol reparado. View abcd con estilos finales coherentes.

H05 template.html: template almacena DocumentFragment separado con p/Text(x), no children ordinarios copiados a body; siguiente p/Text(y) visible. Fragment parentDOM=None con asociación única template_contents al template real. Export conserva x; lectura muestra y y referencia estructural del template. Todos edges incluidos en ledger/quarantine.

H06 namespaces.html: svg/a/text tienen namespaceSVG; xlink:href conserva namespaceXLink y local href/prefixxlink; p/z namespaceHTML. Attr r y text t intactos. No activar SVG links/remotos ni prometer MathML/JS por conservar namespace. raw55B.

H07 noscript.html: scripting=false: noscript/p contiene visible y aparece en lectura; script text throw"never" y style text p{display:none} se conservan en DOM/raw, no prosa ni ejecución. p/end sigue visible (CSS no aplicado). No procesos JS/CSS adicionales.

H08 shadow-inert.html: perfil allowDeclarativeShadowRoots=false; template shadowrootmode=open y fragment p/shadow íntegros, light como texto del div; ningún shadow attach/slotting. Lectura identifica contenido template disponible en estructura. No se declara conformance al composed tree de navegador.

H09 selected-explicit.html / selected-implicit.html: originales123B/105B intactos. RESULTADO FIJO en ambas: Document -> html{head{},body{select{button{selectedcontent{Text(B)}},option[selected=""]{Text(A)},option[selected=""]{Text(B)}}}}. Elementos/attrs sin prefijo; elementos namespaceHTML, attr selected sin namespace. Orden tal como escrito; no doctype/comentarios/fragmentos ni espacios adicionales. Ambas attrs selected se conservan, optionA effectiveSelected=false, optionB=true; select simple muestra resumen readonly B. selectedcontent enabled contiene exactamente un Text(B) NUEVO, distinto del Text(B) original bajo optionB y de Text(A); los doce nodos finales son identidades distintas en el mismo dueño, sin IDs literales prescritos entre parses. Clone sólo hijos: selectedcontent NO contiene wrapper option ni attr selected. Su derivación referencia Text(B) original, optionB y selectedcontent reales; ningún span sourceExact. El árbol original reparado tiene once nodos antes de ese clone; profundidad máxima final6 con Document0. Los dos parses producen isomorfismo de tipos/attrs/text/estado/derivaciones, no igualdad numérica de IDs entre snapshots. Raw y lexema de cierres siguen distintos. Ningún resultado A, vacío, A+B, alias del texto original o normalización que borre selected es aceptable. Todo antes de outputs/parser; API warning sólo orienta reproducción posterior, no cambia obligación.

H10 control.html: bytes13 incluyendo ESC0x1B; entity &amp; se decodifica DOM&, ESC queda carácter/diagnóstico según HTML5, raw literal intacto. View usa proyección segura ␛ (no envía escape terminal). DOM text/source igualdad aparente no acredita ExactSource; inspección sin literal exacto. Si parser aplica replacement normativo, registrar diferencia DOM vs raw, no reescribir fixture para ocultarla.

H11 original.html: raw25B contiene CRLF real y &amp; inconcluso p; source raw byte/string exacto. DOM texto& y nodos reparados, HTML input preprocessing puede normalizar saltos. No raw=DOM serialization; Source decodificado se identifica como tal y raw bytes disponibles separados.

## Encoding y procedencia

H12 windows1252.bin: byteE9 literal, meta windows-1252 dentro prefijo; decoder elegido windows-1252 produce é. source.raw37B exactos binary; encoding/decision registrada; DOMtextoé sin byteSpan original inventado. Si HTTPcharsetUTF8 explícita acreditada prevalece, E9 inválido produce replacement+diagnóstico conservando E9, no é por meta.

H13 utf8-bom.bin: BOMUTF8 prevalece sobre meta windows-1252; DOMé correcta, raw41B incluyeBOM; Unicode derivado/procesado no se anuncia como mismos bytes. from html string yaUnicode no se transcodifica por esa meta.

H14 utf8-invalid.bin: BOMUTF8 + FF inválido. raw11B exactos; decoder produce U+FFFD y cuenta reemplazo/error; no éxito de fuente UTF8 estricta ni pérdida silenciosa de FF. original bytes inspeccionables.

H15 base.html con fallback Observed=https://final.example/dir/page.html: efectiva https://final.example/assets/, target x#part=https://final.example/assets/x#part; segunda base ignorada. Requested=https://request.example/ distinta se conserva, no se usa en lugar de final. `--base https://provided.example/dir/page.html` sin origenObserved produce Provided y resultado provided.example/assets/x#part, no inventa redirección.

H16 invalid-first-base.html con fallbackhttps://final.example/dir/page.html: primera base javascript:never no se ejecuta y retorna fallback; segunda no se adopta; x=https://final.example/dir/x. Sin fallback relativa Unresolved. base absoluta segura puede servir sin finalURL, pero no acredita finalURL.

H17 metadata Nu http_response{status,headers,urls} sólo: guardar bytes/metadatos recibidos; finalURL=Unknown hasta fuente/campo/API acreditada. No confiar en último urls como final por intuición. Raw open/http nativos siguen exactos; full converter debe comprobar metadata sobreviviente sin repetir sentinelspike. FilePath es evidencia Provided de origen, nunca se reabre para cambiar un valor transformado.

## Lectura pasiva y navegación

H18 forms.html: Document/html/head implícito/body/form, con hijos en orden checkbox,radio1,radio2,password,textarea,button. Checkbox checked="false" conserva ese valor de attr y effectiveChecked=true. Radio1 y radio2 comparten owner form real/name r/tree; ambos conservan attr checked="", pero effectiveChecked radio1=false y radio2=true. Password conserva value=secret en DOM/raw, se oculta en prosa. Textarea contiene Text(a&b); button Text(go), ambos namespaceHTML. Input attrs source-order: checkbox(type,checked), radios(type,name,checked), password(type,value); ningún readonly attr añadido. Todos controles se muestran readonly por política de vista, independientemente de attr readonly. No submit/click/value mutation/autofocus/request. Estado y árbol se publican juntos y conversor incluye sidecar acreditado, sin convertirlo en attrs. Ninguno de ambos radios true simultáneamente, primero true o ambos false satisface fixture. Raw211B exacto, sin newline añadido.

H19 resources.html: img muestra alt a&b y srcx.png pendiente; iframe indicaother.html y alternativa fallback; hrefjavascript permanece dato/inactivo. Parser/conversor/inline realizan CERO fetch secundarios y CERO ejecución. Tras acción explícita sobre x.png sólo ese recurso tiene autorización; falla/cancel conserva vista y fuente previas.

H20 ids.html: dos nodes idpart distintos y dos anchors acreditados. Fragment presenta dos candidatos explícitos paginados, no escanea source buscando primer string ni pierde posiciónBack. Navegar al seleccionado prepara destino antes commit y conserva futuro si no commit, según historial común cuando adoptado.

## Fronteras prospectivas derivadas, sin fixtures gigantes ni número fake de owner

H21 source: contabilizar len+capacity real y Raw/Unicode derivados con recibos independientes. Perfil reducido prospectivo en reader real: cap exacto admite última unidad en etapa lectura; siguiente byte rechaza SourceLimit antes callback/DOM. Repetir boundary product128MiB sólo cuando necesario acreditar límite. Un input dentro sourcecap puede fallar DOMarena/deadline: no anunciar aceptación universal de128MiB.

H22 arena/registros/depth: parser+sink reales con límites reducidos admiten exactamente el coste compilado del último nodo/attr/text/edge/diagnostic/repair; menos1 dispara abort tipado antes alloc/mutación, cero árbol parcial publicado. Realloc mantiene old+new physical charges hasta oldDrop; fragment/template/implicit cuentan. depthDocument0->child1 y templateedge cuenta;64exact/65reject según perfil. Cap1M registros no evita arena64MiB: ambos aplican. Cancel tras reserva y antes mutation libera cargos candidato, conserva actual.

H23 Work: total admitido SAMEHTMLWork de source/decoder/capture/DOM/copies/finalizer/quarantine/export no menor que cuerpos/copias reales. límite derivado sizeof+copias exacto pasa y exacto-1 falla antes copiar/crear/Box con mismas identidades/budget/gate; no reiniciar contador por chunk/page. HTMLWork512Miunidades candidato no cambia DocumentWork8Mi/D1/Layout. No confundir receiptwrapper con CPU total tokenizer ni capmemoria.

H24 cancel/worker: cancelar antesSTART no parser; durantefeed entrecallbacks abort/killcontrol conservaoldview y raw. Timeout/callbacklimit/errorIPC/panic no produce Ready parcial. Probar wait/close/join/JobActive0; fallo de cierre deja Pending y siguiente spawn rechazado sobre MISMOowner, no nuevoactor que oculta leak. Windows limit committed512Mi/proceso probado conreadback, noRSS/aggregate. Unix permanece no acreditado.

H25 export: Raw se mueve una vez al campo source.raw, no copia pornode; DOMpool+Value nuevo coexisten bajo64Mi construcción (capacity/header/capture reales). ResponseLease del mismo CallId físico se retiene mientras SDK write_response/flush tieneValue; última liberación real devuelve recibos, no al retornar command.run. Segunda llamadaHTML mientras actual/resultado pendiente no inicia otroworker; error/cancel preserva primera. Probar quotaexact-/1 en guard/captura/Value y ledger actual+candidate. Si writer bloquea, Pending explícito sin matar Nu ajeno ni falso cleanup; casoNoGuardLegacyencoder conserva perfil actual.

H26 pipeline envelope transformado: `--format html` explicit consume árbol validado recibido y raw separado. No reabre FilePath ni vuelve a parsear raw para sustituir valores transformados. IDs de record crean autoridad nueva validada, no pueden fabricar scoped owner o ExactSource; cycles/foreigntemplate/mismatchedsourceHash se rechazan o quedan Provided/no-exact según schema aprobado. Sin formato explícito record sigue datosNu tipados, no autodetecciónporstringschema.

Todos oráculos son prospectivos; no H01-H26PASS inventados. EXPECTACIONES de parsing primarias/adaptación pueden ser criticadas antes código. R2 fija HD-R1-01 antes de todo runtime; no convertir callbackwarn en defecto confirmado ni cambiar estas expectativas por outputs.
## Suplemento R2 prospectivo de H09/H18 (precedencia sobre R1)

Este archivo conserva H01–H26 salvo corrección explícita H09/H18 y aclara clones/estado. R1 sigue intacto. No HxxPASS, ningún árbol candidato ejecutado.

H09-CARGO: comprobar sink real en ambas fixtures, reserva compiled de un clone Text(B), edge, texto y receipt de derivación más ControlState/export/stack reales; no hardcode size de nodo para aparentar metering. Exacto admite publicación completa; exacto−1/cancel entre reserva y clone rechaza sin árbol parcial/estadoB publicable, conserva owner/current/raw y devuelve charges físicos al último Drop. Retener temporal old content si existe hasta reemplazo completo, sin liberar charge sólo por cambiar un índice. Fragmentos de clones profundos cuentan aun si temporales. R1caps intactos.

H09-CLONE-DEEP (fixture nueva selected-rich.html, fija antes runtime): mismo select/button/selectedcontent vacío, una option selected con b[data-v=1]{Text(B)}. Final selectedcontent contiene b[data-v=1]{Text(B)} clonado y option mantiene b original: dos b y dos Text(B) de IDs distintas, namespaceHTML/local b, attr data-v value1 sin namespace. Children order idéntico, no wrapper option clonado. Cada clone referencia su original y la misma pareja option/selectedcontent. Template/namespaces adicionales se comprueban con árboles HTML admitidos pequeños si se necesita cubrir camino: deep clone preserva template_contents y namespace cualificado con fragmento nuevo, no aplanado/alias; ninguna cuota especial. Este texto no convierte árbol falso fabricado en AE parser.

H18-SELECT (fixture nueva select-states.html): tres select independientes. s1 sin multiple/size tiene option A disabled no selected y option B habilitada sin selected -> A=false,B=true por fallback. s2 size=2 sin selected -> ninguna marcada. s3 multiple con A selected y B selected -> ambas true. Attrs se conservan; ninguno de estos select envía nada ni hereda estado de otro. No first-disabled fallback, no fallback en multiple/size2.

H18-GROUP (fixture nueva radio-groups.html): al final, radio interior f1=false, radio interior f2=true, los dos radios exteriores name=""=true cada uno y el último exterior form=f1 name=r=true. El último se asocia al owner f1 y desmarca el interior f1; f2 no cambia por owner distinto y los name vacíos no se agrupan. Defaultschecked y attrs source-order intactos; form asociación basada en form real, nunca igualdad numérica de IDs de dos snapshot. No JS ni llamada submit. Grupos sin checked quedan sin marca y disabled no excluye pertenencia (regla contract R2; runtime propio futuro cuando pertinente).

selectedcontent disable/multiple boundaries permanecen contrato fijo: disabled por ancestro option/selectedcontent/segundo select no actualiza; multiple no actualiza contenido autoral. Sin opción efectiva, selectedcontent conectado/habilitado/simple queda vacío. No es permiso para ignorar H09 enabled/simple. Estas fronteras se podrán convertir en fixtures propias antes de la runtime que las ejercite; no elegir expected después de observar parser.