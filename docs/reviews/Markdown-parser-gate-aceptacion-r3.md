# Markdown parser gate R3: revisión de código y aceptación independiente

**SIN_CRITICAS_PENDIENTES dentro del gate privado acotado.** MD-AE01 y MD-AE02 se cierran sobre esta candidata exacta. No acredita integración, soporte completo Markdown/Obsidian ni QA visual.

Snapshot `markdown-parser-candidate-r3/MANIFEST.json` SHA256 `367B4BF11E313185963EABD4EBD1B48193B13DE7E879DD0702F50E1A04F87686`,702registros verificados antes/después, sinmutaciones. Exe `470B8355A63713315B51BC23A4C65A567C602C6E568B5D2ACCADB4B3B1C0A4C3`. DeliveryE719F239… y R2 rechazado permanecen intactos. Fuente/bin/oráculos/lock se conservaron byte-exactos en copia propia markdown-ae-work-r3; observer scripts sólo cambian sus paths ROOT/root, como documenta RUNNER-ROOT-DELTA.diff. No recompilación ni producción.

## Cierre MD-AE01

Inspección: invoke incluye ahora cualquier stream abierto en la condición que retiene Pending. reap_pending mantiene el ticket si cualquier paso arroja o faltan receipts de cierre; sólo lo retira tras streams/handle cerrados. La invocación fallida no se convierte retroactivamente en éxito.

Controles propios, distintos de los de autor: fallos persistentes de cierre stdout, stderr y ambos a la vez. En los tres, excepción original/childreaped/Pending1 conservado; siguiente spawn rechazado con contadorPopen1. Reintentohelper todavía bajo fallo persistentemente conserva el ticket. Tras restaurar únicamente los wrappers close inyectados, recuperación por **el mismo helper y autoridad** deja Pending0 y todos streams/handle cerrados. Cero rescates externos; los controles esperan fallos, no Ready. `OWN-HELPER-CONTROLS.json` conserva receipts/errores/PIDs.

Error tardío propio en Capture.read justo tras EOF y antes de terminar thread se recoge tras join en cleanup_errors para ambos canales. Caller debe rechazarlo. Este control también pasa; no demuestra todos los fallos físicos ni autoridad de un writer inexistente (stdinDEVNULL).

## Cierre MD-AE02

Inspección: lectura read_to_end de bytes bajo take32769, comprobación bytes.len>32768 **antes** de String::from_utf8. No lee fuente completa fuera de cuota.

Corpus propio conservado deR2 y extendido antes de observarR3: fuenteexactaASCII32768,ASCII+1,UTF8exact32768 (`a`×32766+é),UTF8+1 yUTF8+2 (`a`×32767/32768+é). Exactos admitidos completos; los tres excedentes devuelven source_limit/exit2 sin documento parcial. InválidoUTF8 dentro de cuota (`a`+FF) devuelve diagnóstico UTF8/exit2, distinto de source_limit. Todos pasan. `source-utf8-over-limit` es+2, corrigiendo imprecisión de mensaje preliminarR2.

## Regresiones y provenance

16casos independientes:16pasan sus aserciones. Incluyen wikiUnicode/fulltokentrange[3,16],corewiki literal,escapado,code/fencedmermaid inerte,HTMLblock inerte,entidad decoded& cuyo source-token es&amp;,mathcore/extended,eventquota ycuotas. Profilesbits0/35998 comprobados en11casos exitosos, rangosUTF8válidos.

OracleR3 SHA256 `5E14326DFCFE9376682A88A7023302CB4BA7485B2391F330292E4B25D8858A57` escrito antes de outputsR3. Reutiliza13inputs/oráculos propios congeladosR2 (sus paths siguen en markdown-ae-evidence-r2) y agrega3inputspropios. Corrección DebugWikiLink se auditó sobre enumoriginal **después deR2output y antes deR3output**, explicitada en provenance; no se vuelve a presentar como oracleprevioR2. R2oracle/results/fallo propio se preservan sin reescritura. ContextosHTMLinline/Obsidian completo no se infieren de casosbloque/code.

Replay propio de21casosautor cero discrepancias;5siguen exploratorios. Replay propio de7controlesautor cero rescates, con waitpersistent rechazado/Pending/block y recuperaciónhelper. No se ejecutó fix-controls-r3.py porque regenera su oracle; controles propios exact/+1/+2 y cierre ejercitan sus correcciones conservando el oracle autor. Scriptsautor cambian sólo pathencopia. Fallos propios de preparación (schemaentriesvsrecords y root/minúscula) sucedieron antes de ejecutar candidato; fuenteattempt1/traza conservadas y adaptación mecánica corregida.

Evidencia propia `markdown-ae-evidence-r3/MANIFEST.json` SHA256 `D6D1BE12B7544664B0CC2CF91BEF00FF06399C51EAC68E35287868E96A396075`,36registros. Scripts/oracle/raws/results/receipts/checks/delta incluidos; baselineinputsR2 permanecen en su evidencia congelada44registros SHA55DFCDD… referenciada por paths/sha256 del oracleR3. Snapshot702antes/después probado; bin/source/oráculos protegidos iguales.

## Límite del cierre

Parsing/offsets/profile y correcciones de harness privadoWindows quedan sin críticas pendientes en este corte. No adopción del parser ni soporte gráfico,Obsidianresolution/customcalloutadapter,navegación/modelocomún,HTMLrenderer,Unixnative,TUI/visual,docsgrandes,scheduler/Job/memoriaagregada ni cancelacióninternaparser. Deadline10s es observación del ensayo; receiptscleanup se comprueban aparte. No promesaSLA/vida frente a todo fallo imaginable. R2 errores/rescate propio se conservan y no se convierten en PASS por R3. La aceptación es independiente del autorcoordinador, limitada a esta candidata y corpus.
