# Dictamen final del diseño HTML R2

Informe original privado `REVIEW-HTML-DESIGN-R2.md`, SHA256 `7155350A710D7B0A9BE6538B8CCBF45D83A359CDDB5F2EEB13F7386A35FD3665`, importado normalizado a LF. [Contrato canónico](../refinement/HTML-STATIC-DOM.md); [adopción y límites](HTML-static-dom-adoption-r2.md).

# Revisión adversaria independiente HTML estático R2

2026-10-10. **SIN_CRITICAS_PENDIENTES**, limitado al DISEÑO exacto R2 de representación HTML estática, políticas y gates prospectivos. **HD-R1-01 resuelto en diseño.** No parser/runtime/prueba HTML, implementación, versión fijada/adquisición nueva, Nu companion productivo, QA ni cierre RV401–404. No se reabre spike PluginCommand R2 ya cerrado para registro pequeño.

## Identidad y criterios

Propuesta SHA256 A3F0C5D87CD674DE4BD5B5C8D0A508C41718B1DAB2F7C9EF083C7A93DC8E913D; oráculos617767C3263AEDE31FB036ACF9643C5260937C9C1E8FDB4105474C432D3A1F6C; manifestB7D16F52B724609504E61B284520C8A0B0F4F0C50AA87522C4E9EBF27FD80973; seal162E1C2AC44DD5E44A6C90B057A83F9009797E0DAFE12EB30FAE44D8771ACF25. Todos archivos del manifest hash/bytes PASS y las20fixturesR1 byteexactas; tres nuevas prospectivas. INTEGRIDAD-R2.json contiene resultados individuales. R1 informe/rechazo y oráculos permanecen intactos; no se interpretan hashes como AE.

HD01–16 propios originales intactos: CRITERIOS-HTML-PREPROPUESTA.md SHA22D963D1785200AF430F054ED0FEEFE0678B625EFC767F9EB68FF9D9F4A61544, fijados antes propuesta R1. R2 leído tras verificar identidad. Revisión completa de propuesta/oráculos y delta, no sólo respuesta persuasiva del autor.

## HD-R1-01: cierre preciso

H09 ahora fija para ambas fixtures original123B/105B árbol final isomorfo con12nodos y profundidad6 (Document0); selectedcontent contiene un Text(B) NUEVO, optionB mantiene Text(B) original, optionA/optionB conservan selected attrs pero sidecar false/true. Ningún vacío/A/A+B/alias/borrado deattrs satisface el oráculo. NamespaceHTML, attr sinnamespace, identidad/proveniencia derivada y raw distintos se fijan antesoutputs. Once nodos reparados previos +un clone es conteo semántico prospectivo, no tamaño hardcode para sustituir sizeof real.

H18 fija checkbox efectivo=true aunque checked="false"; radio1=false/radio2=true con attrschecked intactos, password sólo oculto en lectura. Suplementos fijan selección fallback primera habilitada/simple, size2 sin selección, multiple con ambas selected y agrupación porformowner/tree/name real, incluido form explícito y nombres vacíos. DeltaControlState enlaza NodeIDreal/option/source-state con export id/provenance; numbers en envelope Provided no mint autoridad.

El comportamiento se contrastó documentalmente con primarias actuales: selectedcontent clona hijos de la opción efectiva y grupos radio se distinguen porformowner/tree/name no vacío; select conserva última selected efectiva y fallback habilitado según display size. [Selectedcontent y select](https://html.spec.whatwg.org/multipage/form-elements.html#the-selectedcontent-element), [radio checkedness](https://html.spec.whatwg.org/multipage/input.html#radio-button-state-(type=radio)). Consulta2026-10-10, HTML Standard actualizado9octubre2026. Esto contrasta el contrato; no demuestra candidata/parser ni ejecución browser.

Queda técnica por resolver contra esas expectativas: recibos de construcción/finalizer o eventual parche mínimo si la candidata pierde estado/callback. Ya no puede rebajarse obligación tras output. Clone profundo conserva nombres/attrs/text/template y fragmentos nuevos; cargos de clone, edge, derivación,ControlState,stack, transitorios/export y old+new bajo los caps existentes y Work previo; rechazo/cancel no publica mitad del árbol/estado. No exigimos parche sin reproducción ni afirmamos bug runtime desde warning API.

## Matriz completa de criterios originales

| ID | Contraste del diseño R2 |
|---|---|
| HD01 | Rawbytes/Unicode exactos, DOMreparado y Documento separados; ParsedUnattributed/DerivativeClone honestos, sin spansExact por callbacks/igualdadtexto. |
| HD02 | ParserHTML5 compartido y DOMmutable, noscriptfalse/doctypepreservado/templates/namespaces, sin cadenaMarkdown. |
| HD03 | Aborttipado/rechazo completo/quarantine antes publicación y sinsentinel; diseño viable, implementacióncallback/unwind será gate. |
| HD04 | Capacidades propias actual+candidate, opacos bajo proceso/controlador; Pendingconservaowner hasta wait/close/join/Active0, no cotaRSS/aggregate ni interrupción porchunks garantizada. |
| HD05 | Decoder explícito previo/BOM/header/meta/fallback/restart y rawintacto/diagnóstico; NuUnicode no transcodifica; decoderlock/recibo/128MiB aún poraceptar. |
| HD06 | Provided/Observed/Unknown, URLfinal porrecibo real, primerbase/fallback/resolverpuro; metadatosNu no autorizan reabrir niinventan finalURL. |
| HD07 | Readonly fijo separadoattrs, scripts/styles/hidden/templates/details/pasivos; sin envío/modificación; HD-R1-01 cerrado. |
| HD08 | Estructura lectura mixed/list/code/table/figure/controls, tablescomplejasdegradaciónexplícita; identidadDOMstable yrecursos pendientes. QA/renderproducto aparte. |
| HD09 | RV401/403transporte/adaptador/history siguen dependencias; carga root explícita no concede secundarios/local a remoto; error/cancel conservaestado. |
| HD10 | Companionopt-in PluginCommand html/htm directo, encoder/importsinactivaciónimplícita y comandosraw/nativosintactos. Spike registro pequeño cerrado sin extrapolación. |
| HD11 | Shadowinertfalse explícito, selectedcontentclone/disabled/multiple fijados y templatefragmentos propios; HD-R1-01 cerrado. |
| HD12 | ExportDOMordenado completo namespaces/attrs/text/raw separado,ControlState/provenance/IDsrevalidados y construcciónold+new; definitivoCodeAE/AE posterior. |
| HD13 | DeltaHTML producer/quarantine/sidecarNodeAnchor/sourcebinding antesimplementar, no relabelMarkdown. Core comúnUnicode/renderer se reutiliza con fronterasrevistas. |
| HD14 | Fuentescandidataslocales verificadas, versiones/lock/licenciasdecoder pendiente; no integración instalada ni certificación por docs. |
| HD15 | Cortes/gatesprospectivos dejan RV401–404/gráficos/platform/history/productopendientes; no recorte deacceptance ni falsaDoD. |
| HD16 | ExpectedH09/H18 y suplementos ya fijos antesoutputs,20originalesconservados+3propiosautorprospectivos. NingúnHxxPASS/runtime inventado. |

## Gates pendientes que este veredicto no cierra

ResponseLease conserva gate focal antes implementación: Valuecompleto, CallIdreal, transferenciadecharges/slot, init_write fallido/recursiónError/flush/error/unwind/clonescontexto, no mutexduranteIO y últimoDrop físico. SDKactual no exponeguard, nuevoSDKdeltarequiere CodeAE/AE y comparaciónencoder sin cambiar spikeR2; no streamwriter aprobado. WriterbloqueadoPending no es cleanup por timeout ni autorizaciónpara matarNuajeno.

También pendientes adquisiciónlock/licenciasdecoder/parsers, sink/finalizer/clone real y tamaños/capacidades compiladas,protocolo/IPC/QuarantineHTML/Source/bytereceipts, URLfinal/HTTP yrecursos explícitos, navegación/historia/sharedpump, rechazo exact-/1/cancel antespublicar, Source128MiB/DOM64MiB/Work512Mi candidatos y contenciónUnix. Rawsource/Unicode derivados preservan cuotas distintas; ninguna cota totaldeNuShell. No se anunciaAceptar cualquierHTML128MiB.

Aceptación posterior debe congelar código/bin/source/config y fijar variantes propias antes runtime, comparar árbol/estado/clones con estos oráculos inmutables, conservar fallos y últimasretenciones físicas. Revisión de diseño general cerrada no permite saltar el refinamiento ResponseLease concreto ni esos gates. RV402/404 producto y QA siguen abiertos; R1rechazo no borrado.
