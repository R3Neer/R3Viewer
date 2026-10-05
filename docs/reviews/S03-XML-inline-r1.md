# Addendum AE XML R1: evidencia externa inline

Inspección independiente de evidencia ejecutada por el coordinador; no se repite ni se atribuye ejecución AE propia. Siete entradas de INLINE-ARTIFACTS.json verificadas por bytes y SHA256, manifiesto FC5E4651DF6F84FDAEF4BC17C33D410D53C8729E4E381B81B454255F792636F2 y REPORT-inline.md 9DA8AD814615C158ABC2FC40E97F9DC0845A329FF8597C6C894F635C0BD15C68. El manifiesto anterior conserva 1BCA2AFAA27B09BE9B98A14D7CC9BA8B8E9A1525A0953FE2E3519966FBF9039A.

El harness lanza el binario R1 identificado por hash con Popen propio, selecciona descendiente parent=appPID, verifica ruta/comando --structure-worker y creationFILETIME antes de suspender por handle. La guardia ancestry y la guardia creation/command reducen el riesgo de PID reutilizado; no observa contenido de procesos ajenos. El handle mantenido del worker debe estar señalado al finalizar. Cleanup kill/resume se restringe al proceso Popen propio o a identidad worker guardada. Las funciones Job presentes no se invocan en este ensayo: no deriva prueba de Job de ellas.

inline.json registra app12032, worker44988, creation134356796063250240; exit0, diagnóstico de plazo estructural, stdout7320008bytes igual al fixture, SHA96A51B0F16E1BE33CC8351A3EE97E90889CF66FB216C189B79FE604EE46B8DD4. Worker señalado, residuos propios vacíos, cleanup no forzado. Se acepta como ampliación externa verificada de fallback inline íntegro y reap bajo suspensión. Los5.156810s desde suspensión incluyen escritura/drenaje; no prueban el instante exacto del timeout5s ni un SLA. No afirma RSS/límites máximos, Unix ni presentación visual.

Dictamen: sin crítica pendiente dentro de este alcance; REPORT-R1 y su manifiesto no se modifican.
