# D4 R4 — cierre adversario focal del diseño

2026-10-06. Propuesta exacta `Markdown-Syntect-D4-implementation-plan-r4.md`, SHA256 `E40CEE4D45BA351033B132F1343F91F527301ACED4AFCCEA1CE2B8BB6D64A53B`, rehash propio correcto. Autor y adversario distintos; R1–R3 y sus informes se conservan. Revisión focal de contrato, sin implementación, compilación, runtime, procesos auxiliares, cambios compartidos ni repetición de matrices A–C.

**Dictamen: SIN_CRITICAS_PENDIENTES en el diseño D4 R4. D4-R04 cerrado; D4-R01–03 conservan sus cierres de diseño.** No equivale a aceptación de código Syntax/D4, integración D2/D3, backend efectivo, terminal funcional, NativeUnix, SLA, QA visual ni producto completo.

## D4-R04 cerrado: Plain conserva autoridad textual independiente

§9 diferencia FrameAuthority obligatorio de ColorOperationContext opcional; §8 actualiza expresamente los checks del RenderPlan para evitar la ambigüedad anterior. FrameAuthority contiene identidad/perfil/frame/geometría/viewRequest/frameOperationId, cancelación obligatoria y únicamente un mandatoryTextDeadline absoluto si el contrato textual ya lo proporciona. D4 no inventa ese plazo ni adopta Syntect5s como plazo para todo contenido.

FrameRenderPermit se admite y consume con FrameAuthority; ColorWorkPermit/CompleteOverlay conservan ColorBinding enlazado al frame y colorOperationId/colorDeadline originales. ColorTimeout o cancel exclusivamente Syntax no activa FrameCancel, no cambia autoridad textual ni devuelve Work. Cancel global o identidad stale sí invalidan ambos caminos. Correlacionar con ids separados no permite aplicar overlay de otra ventana/bloque.

La API commit_frame revalida primero FrameAuthority, después acepta o descarta overlay según su propia autoridad/plazo. Overlay tardío produce Plain íntegro con stickyColorCause, sin renovar colorDeadline ni invalidar T/FrameRenderPermit. Con overlay None el commit no exige ColorContext ni puede recuperar estilos históricos. Un overlay calculado antes pero publicado después del plazo también se descarta. MandatoryFrameRejected conserva reglas textuales cuando existe cancel/stale o deadline textual normativo vencido; no se omiten esos checks para dar Plain.

Inline mantiene el mismo plazo5s opcional por todos los fences/preparación/cola/cleanup; al vencer no inicia nuevos envíos/estilos y conserva texto D3 ya admitido para el commit propio. Pending/reap del worker y stickyTimeout siguen autoridad separada; ninguna espera de limpieza adquiere plazo nuevo de color ni autoriza un leak. Las fronteras obligatorias de limpieza D3, si existen, permanecen y no se reinterpretan como fallo de lectura. No se promete interrumpibilidad de syscall ni SLA.

## Oráculos y evidencia de revisión

Verificado addendum `markdown-d4-oracles-r04-addendum-r1/MANIFEST.json`, SHA256 `EAFF908C9FF0E7612866E1164DFF4D8A2B270EBA9FD9AB0886C51F820EE5C863`, y su único registro EXPECTED.json985B concuerda en bytes/hash. Cuatro expectativas PREOUTPUT_NOT_EXECUTED: colorTimeout con autoridad textual válida publica Plain+sticky sin estilos/envíos/refund/renew; cancel/stale rechaza; overlay4999/commit5001 se descarta; deadline textual propio vencido sigue resultado textual normativo. Son expectativas nuevas separadas, no cuatro PASS ni reemplazo de19/8/13/R03.

La estructura R4 conserva T completa y RenderPermit obligatorio antes de C opcional, permisos privados no duplicables con cargo único y spend local, memoria/copias/backend concretos y deadline original de color por operación. No otra crítica nueva identificada en este alcance. La implementación deberá demostrar esos tipos/counters/leases/commit y adaptación efectiva del backend; nombres de permisos y oráculos sin ejecutar no acreditan automáticamente esa prueba.
