# Revisión del candidato TOML D1

2026-10-05. Veredicto independiente final: SIN_CRITICAS_PENDIENTES sobre candidato R2. Alcance: implementación TOML, arena/IPC/proceso, CLI/UI/salida y delta de corrección; no cierre S03/RV106 ni visual/matriz.

Snapshot210archivos: manifiesto299BCB2A61FFBC3E1281F7DA01A109CB4D57FBE59142BA74ACD9CDB27AB76154; lector7BBC1B0C2B002A7CCEF66C2845577674DEAC168FC5E970C2D1CFE08CE36B1CEB; companion0B9B04E6B7D1AC4ABCC871D99576BDEA5805AA4E0759619CA3E61E17BF488184. Todos hashes verificados por adversario y fuente de producto contrastada antes de commit. Informe temporal final REVIEW-R2.md SHA36E6972C26C1552D322286E1B4D34645688C3F7BE6A52E0FACD4559564F53CBD.

R1 tuvo dos P2 confirmados. D1-R1-01 cerrado: capacidad de scratch vivo reduce presupuesto antes de reservar; buffers e índices del escáner y migración se cargan mientras viven; sort in-place evita scratch estable no declarado. Harness independiente comprueba bloqueo antes de crecer y recuperación al liberar scratch,6/6 pruebas. D1-R1-02 cerrado: reader de lease rechaza frames trasSourceEnd; repro tardío termina86 sinEnd, controlválido0.14probes funcionales independientes adicionales pasan. No quedan críticas accionables del candidato en ese alcance.

Root:147tests workspace, fmt, Clippy all-targets estricto y build pasan. AE independienteR2:69/69CLI/Nu/redirección,26/26semánticas y2/2 casos nuevos anidados pasan. Building cancela visible27.31ms, sinReady tardío tras300ms y reintento manual terminaReady; qBuilding0 y CtrlCBuilding130 restauran y recogen hijos propios, máximo1worker muestreado. Exceso1M conserva fuente completa0+diagnóstico en580.44ms. Mediciones concretas, no SLA. Evidencia ampliada en S03-D1.md y ciclo de vida del autor separado.

Pendientes: presupuesto/entrada/deadlines exhaustivos, spans/índice internos completos, todas fases y carreras de obsolescencia, correspondencia inline larga bajo decoderVT, Repeat/AltGr nativo, visual WindowsTerminal/WezTerm, contención y ejecución Linux/macOS. CrosschecksR1 no son ejecución ni certificado deR2. TOML no sustituye JSON/YAML/XML restantes. Aceptación parcial no reduce backlog ni cambia Definition of Done.
