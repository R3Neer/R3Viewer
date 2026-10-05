# Gate privado PDF UTF16: R2 revisado; producto pendiente

Estado vigente: R2 final tiene revisión y aceptación independientes sin críticas pendientes **en el delta EOF/observación/retención**. El rechazo R1 siguiente permanece como historia y evidencia. Integración y PDF completo siguen pendientes; no se convierte Unverified en completitud.

2026-10-06. Contrato [R3+deltaR4+V](../refinement/PDF-TEXTO-UTF16.md); candidato privado congelado, no integración ni aceptación. Revisión de código de raíz: **NO APROBADO**, PDF-U16-ROOT01. AE independiente finalizada: **NO APROBADO**, PDF-U16-AE-EOF01 corrobora el fallo con peer propio. Raíz verificó los 53 registros de evidencia sin discrepancias. RV-204/G1 y SourceUnverified permanecen abiertos.

## Identidad y comprobación

Snapshot temporal pdf-utf16-mapping-candidate-r1: MANIFEST79043DF04C64675698CC9499B8E28AA5D1E980E97C302843172CD493F7647D1C,1076 registros. BinB34BD1A5CF394025A1DFAB6BA6E9C73EE2430E11D9D812210E516836C4B9D68B; supervisor7EDA58B4713AC58EDAEAF125E75AEECEFCAFEC54B910E6A7DDA4CA159AE91873. Raíz verificó todos los registros antes/después en original y copia propia; código y helper permanecen intactos. Entrega autor88005E8E00DAA23D6D87D45F004BE4B366F22B2955BE70FF9EFC1EC7BCC401FE. Su cobertura declarada no se presenta como aceptación independiente.

## Defecto reproducido

feed_native marca EOF por callback before_stdin_close antes del close real. Un wrapper propio demora ese cierre y permite observar Failure15 después de SourceEnd mientras stdin sigue abierto. El supervisor exacto lanza V y publica validated=true ContentFailure; debía rechazar la respuesta temprana sin V. El cierre posterior correcto de ambos procesos no valida retrospectivamente ese transporte.

Oracle propio previo53F4C0D1DAFFB86F4500B620228F003AFA9CCBA0BA171BA81870BBAF570C6670; driverF9B41DB030CC6610056774CA39EB4CF3135C790EAF1543BC79BD52A83AD89871; resultadoFAIL9ED8998AD23C6F464F2B005AA4EB29A8140A1B993A52310F74257B5CE6083C2A. Copia temporal pdf-utf16-root-review-r1 contiene inputs/resultados. Dos procesos propios Reaped, handles/streams/readers cerrados, sin Pending ni rescate. Peer ficticio de protocolo, no fallo atribuido al comportamiento PDFium. AE reprodujo con peer nuevo independiente.

Revisión privada de raíz PDF-UTF16-root-review-r1.md EA3325E0B1B0899A08504FA415ED69F698A4A5EA9E7712C97F8322C82CEFA0B6 preserva lectura main/UTF16/wire/supervisor y delta lib respectoR4, crítica y criterio. Informes intermedios permanecen fuera de Git según metodología.

## Siguiente candidato y límites

R2 debe acreditar cierre real de stdin y retener cualquier error/autoridad; Failure observada antes de EOF no admite V/publicación. Afinar ordering sin tratar flag local como prueba de consumo del peer y repetir controles de cierre atrasado/fallido y positivos posteriores a EOF. Candidato separado, nueva revisión y AE de bytes exactos; no modificar R1 ni goldens.

Una llamada GetText, decoding UTF16, asociaciones/geometry/wire y ledger no eliminan los FAIL de ligaduras24B→22B, singleTJ32767 ni CMap0000. Semántica de fuente sigue Unverified, NativeEmpty no significa SourceNoText. Contención nativa/margen externo, producto completo, PDF gráfico, terminal/QA visual, Unix y distribución siguen pendientes.

## Aceptación independiente R1 finalizada

Informe temporal PDF-UTF16-independent-acceptance-r1.md SHA256 7819C0F275F41FBA075B98B2CD1B8AC09CDA6A28E850AC023D5F5B74FEDAE08A; manifiesto OWN-AE-EVIDENCE-MANIFEST.json F491DC6DA0789F0E07BE86A913ABB04E8249D4815517CA8FBCC5C743C03B299C, 53 registros verificados por raíz. Recibo privado PDF-UTF16-root-verification-independent-r1.json conserva el contraste. Informes del candidato rechazado permanecen fuera de Git.

Cobertura positiva delimitada: cuatro PDFs propios, tres rechazos preflight, 23 modelos wire propios (13 aceptados y diez rechazados), máximo lógico sintético V N=U=S=65536 y cuatro contrastes nativos conservando discrepancias. No demuestra máximo agregado real H+D, lifecycle completo, semántica de fuente, integración ni QA visual/Unix. El control de EOF propio produce validated=true y spawn V con stdin todavía abierto: FAIL preservado, sin rescate externo. R2 debe superar código y aceptación independientes antes de adoptar el helper.

## R2 final: cierre del defecto EOF dentro del experimento

Candidato privado pdf-utf16-mapping-candidate-r2-final, MANIFEST E3FA1AED161E589A448E250ED12C01E0952B2B2CA67618353A76824A9DA9D435,1112 entradas; helper BB36465CED2EA1BAADF8D68F85D6BDFBF41347799DEAD61C56A030CB479EBA18. Bin/Rust/DLL/wire conservan identidad R1; sólo helper cambia comportamiento. [Revisión de código](../reviews/PDF-texto-utf16-code-r2.md) y [aceptación independiente](../reviews/PDF-texto-utf16-aceptacion-r2.md) SIN_CRITICAS_PENDIENTES dentro del delta descrito, no del producto PDF completo.

Raíz repitió exactamente su oracle/driver original de EOF: stdin observado abierto con94B/Failure15, ahora validated=false y ceroV, Reaped/streams/handles/joins cerrados, sin Pending/rescate. En la copia se preserva resultado propio separado del baseline del autor; comprobaciones posterior originales/copia1112 sin discrepancias. La primera verificación detectó sólo una colisión de nombre del output, registrada sin alterar código/oracle/candidato.

AE de un tercer agente ejecutó doce familias propias: PDF real/corrupto, peer EOF y fragmentación, READY+wire y primer byte temprano, close sin cerrar/que lanza, físicoClosedInFlight conservador, observer previo al lock, V temprano y tres solicitudes GCdisabled sin captura Native retenida. Fuente/expectativas previas y fallos propios de comprobación se conservan. Todos actores propios Reaped/allclosed, un hilo, cero rescates, Pendingfalse. Raíz verificó los19 registros del manifiesto de evidencia102DB24581BEC898C9809D0F16514989769B8D42BE252A64BD4D45DB2104A996, y doce PASS de resultados7F94836DB658C77EFD312D66DE05CFF819EC2168252675DF51909839DCB61F1A.

PDF-U16-ROOT01/AE-EOF01 quedan resueltos para este helper congelado y casos probados. La aceptación del delta no amplía garantías de Rust ni máximos agregados, transmisión remota, llamada nativa bloqueada, semántica de fuente, geometría, integración, gráficos, Unix o QA visual. SourceUnverified y todos FAIL semánticos se mantienen. RV-204/G1 abiertos hasta aceptación de producto.
