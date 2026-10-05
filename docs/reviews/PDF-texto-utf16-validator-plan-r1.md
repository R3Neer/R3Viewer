# PDF UTF16 — revisión del modo validador secuencial privado

Adversario root; autor unix_r3_review. Candidato SHA256 6C85BE376BD1C595B9760096DC9DD88BA72CB1DB8D1A7CFAD432284397CDA544.

SIN_CRITICAS_PENDIENTES para este delta de gate privado, no aceptación del código/runtime ni integración del producto. V no carga PDFium ni abre PDF; consume descriptor esperado H y wire nativo streaming. Hash del wire incluye framing y H lo contrasta con captura inmutable; receipt final96B y ContentFailure3 no equivalen a texto ni completitud. Reaped del primer hijo no publica respuesta; segunda etapa y sus IO/Pending/latches también obligatorios, deadline único10s y cleanup separado acotado.

Cota conocida H: 2*8388608+8060928+3149824+3*1048576 =31133696 <41943040. Buffers Python conocidos/overheads/capacity y transientes se cargan; memoria interna del intérprete/native desconocida no se presenta como RSS/heap total bajo40MiB. El límite solicitado agregado y Job512MiB son pruebas distintas. No copias completas wire durante polling/feed/JSON ni artefactos anteriores descartados para fingir admisión.

Controles previos V01..08 se deben ejecutar sobre freeze: no DLL en V, descriptor/wire/receipt falsos, segundo cleanup/lateIO, deadline compartido, ledger real y Failure temprano. R3+deltaR4,18casos originales, suplementos y fallos históricos siguen vigentes. Implementación privada autorizada; RV-204/G1 permanecen abiertos.
