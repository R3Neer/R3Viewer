# Regresión Windows para cierre de S03

2026-10-10. Evidencia del coordinador sobre el checkout canónico; no aceptación independiente, QA visual, ejecución Unix ni cierre completo de S03. Relacionada con la [auditoría de pendientes](../reviews/S03-closure-audit-2026-10-10.md).

## Candidato y resultados

Se copiaron492 archivos fuente antes de build/test; manifest023CEB04CC99FA6E6C443134016A6161714B223E38478B9DAE5DC9D60F2B1433. Cargo build --locked --offline --workspace terminó exit0. Worker privado antes de test: r3viewer.exe2095247F636325DB524651BF1A8520AF745A3C29AC55579A22307B9898FF3F0E; companion4EB4CCE0EFE1B99197C8DC2CA4EF1BED1030375A704A5A528547D3E115AB06C2. Fuente y ejecutables permanecieron iguales después. Los test harness de Cargo no tienen un freeze físico previo propio en este ensayo; no se presenta como candidato completo nuevo para AE.

La primera ejecución sin worker/config explícitos terminó exit101,417PASS/90FAIL/10ignored. Las trazas identifican ausencia de esas entradas obligatorias y fallos derivados; se conservan. No se corregió producción para convertirlo en verde. La segunda proporcionó worker/config privados y ejecutó en serie, según los controles con estado global:

    cargo test --locked --offline --workspace -- --test-threads=1

Resultado:578PASS,0FAIL,10ignored,exit0, incluyendo doctests. LogSHA978680AC257CDD749E28A12A3ACB0F594F47A22CE2E5F477AE9047118CDCAABD. Los ignored no se cuentan como PASS.

Después se ejecutaron explícitamente seis controles de sintaxis omitidos que requieren un worker nativo construido:

    cargo test --locked --offline --lib syntax -- --ignored --test-threads=1 --nocapture

Resultado:6PASS,0FAIL,exit0. Cubren los escenarios ya definidos de propietario original, rechazo de retry ajeno, recuperación/reap y fallos de adquisición. No prueban toda la matriz de obsolescencia, cache/IPC/timeouts ni cancelación durante prepare. Los otros cuatro tests ignorados de XML no se ejecutaron en este ensayo. Artefactos, logs, fallo inicial y recibos privados conservados bajo .tmp/s03-closure-2026-10-10.

## Reproducción del entorno explícito

Desde el repositorio, cargar scripts/dev.ps1 y construir workspace. R3VIEWER_DOCUMENT_TEST_EXE y R3VIEWER_SYNTAX_TEST_EXE deben indicar el ejecutable construido; R3VIEWER_DOCUMENT_TEST_CONFIG debe indicar un TOML regular propio (aquí vacío). Son variables del proceso de pruebas, sin modificar perfiles/configuración personal. Conservar configuración, fuentes y worker usados; los tests no habilitan ningún ignore por sí solos.

Continúa el control causal de cancelación durante preparación, con criterios prospectivos independientes. El resto de la matriz Windows, apariencia real y contención/runtime Unix mantienen sus pendientes originales. Ninguna suma de tests sustituye esas comprobaciones.
