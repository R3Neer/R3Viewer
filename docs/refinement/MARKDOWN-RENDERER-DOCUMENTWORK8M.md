# Delta acotado: DocumentFrameWork8MiB candidato
Fecha2026-10-09; base0cb0fe6, propuestaOwnerRendererR2 cerrada. Sólo sustituye importe candidato4MiB por8MiB en DocumentWork.frame/bounded. Source text_engine intacto.
Oráculo original EXPECTED-before-output.json SHA7d6e12c4... y fallo TEST-Work4M-attempt1.log/binaryC709907C... se preservan; no editar sus expectativas.
|Fase|80×24|120×40|
|---|---:|---:|
|Layout ya realizado|404164|1327023|
|Leer packed20+class40+reconstruirGlyph72+metadata16|213120|710400|
|Materialización y payload|214560|715200|
|Dos readbacks ambos grids48B|368640|921600|
|Diff parejaCell y payload|185760|465600|
|Reset ambas grids para éxito/abort|184320|460800|
|Formatter ≤2updates/cell*256B +payload+fixed256|984736|2462656|
|TOTAL|2555300|7063279|

Importe8MiB=8388608 deja1325329 para este primer120×40; redraw incluye previouspayload real. No promesa de todas fuentes/estilos; overflowchecked, quotaReference/error explícito antesbackend.
READBACK y formatter son guard conservador del camino real pinned, no RSS ni bytesruntime exactos. Capturewriter test64KiB fallible/fijo está identificado como harness, no producciónmetadata8192.
Cambio técnico mínimo: DOCUMENT_FRAME_WORK=8MiB; Work::permit existente; DocumentWork.bounded rechaza >8MiB. No deadline/hash/Source/Workpreparation cambiado.
Controles antes nuevosoutputs: fixture4920B conserva1440/4800glyphs y24/40rows; ambosdrawsPASS,2frames+resize; oldnewFrame256K y heapConsumer/Framework.
Controles negativos: old4MiB retornaWork(required,limit,parts) antesoutput; exacttotal/exact-1 y8MiB+1; cancel presente y callback último rechazan; stale/foreigngeometry/identity no commit.
Perfil8MiB no elimina checksCancel por operación/admisión y antesIO; no añade segundoallowance. No modificar goldenold para conseguirPASS.
Plan/privatecódigo por revisar independientemente; no adopción de perfil o renderer por esta medición del autor.
