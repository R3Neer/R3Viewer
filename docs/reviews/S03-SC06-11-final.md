# SC06–11: revisión focal final SC4 y AE propia

2026-10-10. SIN_CRITICAS_PENDIENTES para este cierre focal Windows: CodeAEinstrumentación SC4 + propias tres familias afectadas y reuse explícito de evidencia anterior suficiente. No S03 completo, QA visual/Unix ni adopción hooks.

## Identidad y ejecución propias

Snapshot fuente1034/1034 CodeAE0DAF3D085BCCF7FA465BBA07EF516604FF79DEF0B4422C1CB1D39E0CE385B776; dosPython deltas R3, Rust/bin/DLL/fixtures byteexactos. Reader copiadoD59169A25C237414A68D79CFBDCEEAEE24EC755E241070C3F1D4C824DF9C3659; pluginC586146A6BFCFB5A4FAFC3B3E10A28D41729F70F1E12D401680174F2379CE623.15harnessfilescopiados y runner/oracles/freezeMF/Nu19identidades fijadasANTESoutput en MANIFEST-before-output.json5066B766615B261F71CB882C8A6BC1DC76F1339E9B8480AD9D0B3FFF672E787E;19/19exactospost. Runner propio sólo paths/output propios,3selectorsclosed/no segundoattempt; misma driver/source y30s exterior. No recompilación propia.

Tres ejecuciones secuenciales únicas: Busyexit0 2.203603s; Startupexit0 0.582693s; Transferexit0 0.652737s. Datosmedidos sin SLA. Cada driver PASS/Reaped/waitclosedjoined/JobActive0; host EOF/HPCONreturned/readerwriterjoined/cleanupComplete, actor6cleanupentrieswaitoriginal/observerjoin/originalsclosed/handle.closed/probeclosed, forcedfalse/pendingfalse. SourceSHA beforeafter iguales.0Pendingmarkers, actorThreadCount1, ningún owner propio retenido; ventana liberada a raíz traswait/cierre. Reap no inferido de Drop/PID solamente.

Busy: Nu0→Tiny0. A gen1/request1→scroll req2→resize100x24req3rows22; CURRENTStylesreq3 con ModelPtr/SourcePtr/len iguales a entradaA. Roots gens2/4/6 punteros/len/request0; Bgen3/Aagain5 fuentes/markersASCII auténticos. Workerwitness28024 FILETIME134361103089475631 antescontrol259→waitexit75→checkedclose.106samples max1;3spawns3reaps; inocuous sibling stdinclosed/waitexit0/handleclosed. Nuevogen sólo cambio pane; root/numeric/key semántica UIoriginal reuse abajo.

Startup: app130→Tiny0 causaltrue. HeldPID48432/FILETIME134361103377228122/phase1/workerSequence2 positivo físicolive259 antesETX y luegoexit75waitclose. Sampler5loops max0 SECONSERVA; no se fabricó sample1: testigo directo original diferente del sampler. Transfer: app130→Tiny0causaltrue; held44312/134361103617646812/phase2/sequence4/liveness259→exit75waitclose;7samplesmax1. En ambasnonce/mapping/phasecontrollerworker/callbackoriginal y heldPIDreal concordanhost/actor. Modos503/7restaurados, cursor25visibletrueantesdespués; clockrestoreComplete<=joinStart<=joinComplete, success1;1spawn1reap. NoCancelhostflags ni globalprocesssignals; ETXreal exclusivoownconsole.

## Matriz original y reuse delimitado

| Criterio original | Evidencia y disposición |
|---|---|
| SC06/C05 | Own Busy: dosStrings A/B/A, scroll+resize SAMEModel/Source/gen y reqCURRENT, rootslimpios. Numeric/key sóloString y searchsameleaf gen estable: dos tests UIexactos R3autor1+1PASS, CodeAE fuente; no nuevo runtimepropio de esos tests. |
| SC07/C05–07 | TestUIexactos arbitration/currentPlain/oldrequest/mismagen/othergenPlain y CodeAE aceptados; Own Busy roots/selecciones sinrevival. No todasintercalaciones. |
| SC08/C06 | Own Startup y Transfer causalETX130restore antesjoin/secondTiny0. View y All parserpasotrasEngineStep/Progress: AUTHOR-runtime-r3 C31BA6776D15603FADC67EF438B5A26495221792949D7F4239A36FE17D3E54AB39/39 físicos revisados, PASSexactsourcebin mismasR4; reuse2casesautor explícito, no ownruntime. AllnoGuard/outputprefix, Viewrestorebeforejoin. No cancelación dentroOnig nativo ni granularidad natural del park. |
| SC09/C06 | Own Busy coalescing requestmientrasheld sin UIwait, qexit0/secondTiny0, maxobservado1/3spawnsreaps/innocuoussiblingalive; original HANDLEcreation/wait/IOjoin. Samplingfinito no garantíatodosinterleavings/OShandlesglobales. |
| SC10/C07 | L3 adjudicación/reuse lowercore tests y evidencia histórica65536LFeligible/65537Plain/nonASCIIbytebound y total/OLDstyles/sourcepreservados, código sin delta; no nuevoreplay/universalcorpus. |
| SC11/C07 | L3 reuse startups2s/Line500ms/View2s/All5s límites originales con ACK/progresssinrenovación, fallbackPlaincorrecto: READSET históricoenfreeze; evidenciaAll5sRequestTimeout separada de100msviewport. No nuevo deadline/SLA ni claim1msrestauración. |

AUTHOR-runtime-r4manifest25DB744627EB3939983112BC6080E4DC3D425AAE140D4A89E1AD8C4DC22B0D8118/18 revisado como evidencia adicional, no sustitutoown3. R3VIEW/ALL/UI/inventory/L3 reuse permitido poradjudicación original: SC4introduce2Pythonobservación, no cambios producto. Inventory1024/align8/mappings/bins anteriores fueraowntime. Normalcfg y hooksprivados no se adoptan por este reporte.

## Fallos y límites conservados

R1/R2CodeAErechazados (pluginquotedlist/statusmultiwriter/ownerpin-beforewrite/handlefalseclose) y R3Busy/startup/transferFAIL inmutables. Copia física39salidas+manifestR3 en historical-r3-runtime. BusyEnterKey/cota21observador incompatibles original; StartupTransfer assert agrupado perdiódiagnóstico, no overlapproductivo demostrado. R4corrige sinclocknuevo ni retryverde; own3sonnuevas corridas prospectivas, jamás relabelR3FAIL. Runtime originalauthor UIViewAllPASS separado propio.

Preparación own intentó filename run.py dentrofreeze que noexistía; se leyó runner de área author y derivó wrapper privado cerrado antesoutput. Python instalado emitió aviso de localización en algunas lecturas auxiliares; no cambió resultado/exit/recibos físicos. No producción/Cargo/registry/perfiles/instalación ni filesystem ajeno alterado. No universal interleavings/CJK/AltGr/Unix/QA/S03closure. Obligaciones de otros núcleos W1/W2/W3 y C01 siguen independientes.