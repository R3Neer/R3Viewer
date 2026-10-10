# Adopción del diseño de propietario de proceso PDF D2

2026-10-10. [Propuesta exacta](PDF-D2-PROCESS-OWNER.md) con [revisión independiente sin críticas pendientes](../reviews/PDF-D2-process-owner-design-r1.md). Sustituye la representación std Child exclusivamente del futuro PDF P1 por proceso e hilo primario propios, suspendido→attach/readback SAMEJob→resume→START; Legacy conserva sus firmas/perfiles. No acredita código ni ejecución nativa.

Las obligaciones concretas Win32 del dictamen forman parte del siguiente CodeAE/AE: whitelist de cuatro handles realmente heredables, array estable hasta Delete, flags explícitos/commandline mutable/env Unicode, ResumeThread==1 antes START, captura inmediata de recursos y cleanup del proceso original incluso si attach falla fuera del Job. Cada campo/captura/buffer real invalida su inventario D1 y requiere financiación/medición previa, sin ampliar cuotas o inventar recibos por Drop.

Raíz cotejó 7 entradas físicas del manifest F9332B91D75DAE1A42CF12FE3D0CBE9335180B95EE48EBF5856F1BA028FBBA42 sin discrepancia. Propuesta SHA E5396174CFABE9019DC7244138F60E98306DB0396EE52A58583B362910DBA09E, informe SHA6B40161B65CC13C0C4F96F75E839DB7955BF97F5E7D7F0DC02DE2638F9ACE7A5. Recibo privado .tmp/pdf-product-d2-2026-10-10/PROCESS-DESIGN-ROOT-COTEJO-R1.json.

O3 registry/mintage/hooks/backstop y O7 supervisor/IO/EOF/Job/publicación siguen pendientes del candidato D2 exacto. Guardias sobre cancel/deadline deben preceder mintage/reserve/clone/spawn según actor original; compilar un helper no prueba esas fronteras productivas. QA, Unix y RV204 no reciben cierre por este diseño.
