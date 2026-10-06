# D2b R3 — revisión adversaria final de diseño

2026-10-06. Propuesta BA4D78A983B4763239FD45FAAAB6418C73C8D5BF97E5FA1CD048352FEE139938. Adversario root, autor unix_r3_review; agentes distintos. Revisión de diseño, sin atribuir implementación ni ejecución a la propuesta.

D2B-R01 cerrada: búsqueda semántica y expansión transaccional de todos los ancestros que ocultan el hit; una nueva FlowIdentity, remap y commit único, preservando FoundPending y estado antiguo ante cuota/cancel/stale. Expectativas nested previas conservadas.

D2B-R02 cerrada: UI solo entrega peticiones, preparaciones largas en propietario de fondo, resultado y cuotas retenidos hasta cierre físico; request/cancel/stale verificadas antes publicar. El pump y restauración de terminal siguen exigiendo pruebas funcionales posteriores.

D2B-R03 cerrada: b0 implementa primero una biblioteca común real y compilable, kernel genérico con Source/hash reales sobre DocumentSource::new y SafeTextModel::from_shared. b1 instancia la misma autoridad para preparación Document real. D3c reutiliza la biblioteca; no existe dependencia circular ni SourceReady ficticio. El resultado viaja como propietario y FinalReceipt; la recepción de un mensaje sola no autoriza commit ni libera Pending.

## Dictamen

SIN_CRITICAS_PENDIENTES de diseño para D2b R3. Autoriza el siguiente corte b0 y luego b1–b3 según las dependencias y aceptación ya fijadas. Cada entrega de código requiere revisión/aceptación independiente antes integración. La implementación deberá acreditar las capacidades reales y el almacenamiento del propietario Pending entre invocaciones; este dictamen no acredita esos hechos por intención o nombres de tipos.

D2a R2 ya está integrado en 9f577508a391ebb23886878554a0693b914a568c, sobre Source APIs adee007. Conservar los límites/rangos/oráculos previos y adaptar solo la base de trabajo a ese commit. No cierre de RV-301, TUI, routing/inline, Syntect, gráficos, Unix, QA visual ni producto completo.
