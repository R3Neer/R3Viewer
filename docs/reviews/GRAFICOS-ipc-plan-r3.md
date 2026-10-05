# Revisión adversaria de diseño IPC R3

2026-10-05. **SIN_CRITICAS_PENDIENTES**, acotado a este diseño. Sin ejecución, implementación, aceptación visual, Unix ni cierre G2.

Propuesta exacta GRAFICOS-IPC-proposal-r3.md SHA256 `5438FBA7523650C082EA33C5B7D296F8191B4E2A26050BDB6951EF5DA6BE2B3E`, comprobada antes y después. R2 conserva SHA256 `6F1372A3E09E73722DDFEAA6C0DCC145C434ECF0E7353D8D2A0896E318C464EB`; la revisión R2 conserva `EE8D218A0AF8453FB75A863B991A3D3575B9B956ECF03B28F2382A956626B8BB`.

## G2IPC-01: resuelta en diseño

R3 elimina la creación remota de handles por C. C comunica el valor de su handle fuente retenido; H realiza DuplicateHandle desde su handle propio de C hacia su tabla y lo convierte inmediatamente en OwnedHandle antes de validación o logging. No herencia, derechos explícitos y sin DUPLICATE_CLOSE_SOURCE. Este protocolo elimina el contraejemplo R2: C muerto antes de comunicar o antes del pull no deja un duplicado desconocido en H; después del pull H ya conoce y posee el recurso, incluso con ACK fallido.

Se comprobaron los escenarios de handle inexistente, tipo incorrecto, C/H, proceso fuera del Job, mensaje repetido, muerte C antes/después del pull y antes de ACK. El contrato rechaza/limpia sin START; la fuente permanece retenida evitando reutilización; nonce/canal exclusivo impiden confundir datos worker con control. ActiveProcessLimit=2, no breakaway y D bloqueado en bootstrap acotan miembros C+D. El handle de C con PROCESS_DUP_HANDLE existe desde spawn y H no entrega processhandleH a C.

Job exterior retenido más ActiveProcesses=0 cubre D desconocido cuando C muere; no pretende inventar exit0. Ready requiere handle D transferido, espera individual y exit0. Errores de query/wait/cierre quedan irreversibles y bloquean siguiente parser. La muerte de H no cuenta como receipt. Se satisfacen los criterios pendientes de ownership y cierre sin rescate.

## G2IPC-02: cierre conservado

Cuarentena, receipts Process/IO de C y H, stderrEOF/cuota/errores, escritor terminado y joins continúan siendo previos al único commit H. La región serial comparte cancel y generación; End/stdoutEOF/exit0 sin ReceiptIO no publica. Los ensayos de barrera y stderr tardío conservan el contraejemplo independiente previo.

## Límites del dictamen

La coherencia documental no sustituye prueba de derechos Windows, herencia real, Job readback, fallos de API, leak counts, tiempos ni recepción. El ledger agregado numérico y margen runtime/native deben concretarse y revisarse antes de ejecución, tal como exige el propio diseño. Código congelado y AE independientes siguen necesarios. Sixel sólo se adopta después de su gate independiente verde; no se aprueban framing terminal, cache o integración PNG de producción. El diseño de graphemes se revisa aparte.
