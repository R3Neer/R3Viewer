# Migración a Nushell 0.116.1: revisión adversaria final

Fecha: 2026-10-12. Revisor independiente: nu116_adversary. Base de revisión: da9df0d. Resultado focal: **SIN_CRITICAS_PENDIENTES**.

## SDK y procedencia

El manifest final `.tmp/nu116-sdk/MANIFEST-sdk-candidate.json` contiene 35 archivos: tamaños y SHA256 verificados por el revisor sin diferencias. SHA256 del manifest: `7E438CF99FA234165AD4028DC7E5B6FCC7DB8E8EE86D2AA376A6919093BAD281`. Se verificaron las dos archives oficiales descargadas y todos los originales registrados en `vendor/nushell/UPSTREAM.json` contra su contenido. Los únicos deltas locales frente a upstream son el README normalizado y cinco archivos Rust de la adaptación ya existente. Estos cinco archivos conservan exactamente el contenido de da9df0d tras normalización LF.

Las dependencias de la integración fijan nu-plugin y nu-protocol a 0.116.1; el lockfile usa la misma versión del stack Nu. Se conservan los cambios oficiales de manifiestos y sockets Unix. No se introduce un fork local de nu-protocol ni una modificación nueva del protocolo. Se revisaron recepción con polling, ACK/drop, señales por invocación y Reset global latched. No se detectaron modificaciones adicionales de esos mecanismos durante la migración.

Los logs del autor acreditan check del companion y pruebas aisladas del SDK: 43 unitarias plugin, 78 core y 17 ejemplos de documentación, sin fallos ni ignoradas. El revisor comprobó los resultados por lectura; no volvió a ejecutar esa suite.

## Runners y reproducción independiente

La revisión inicial pidió dos correcciones: un plazo externo finito para una posible regresión de bloqueo en abandono/reuse, y cobertura de entrada ByteStream real y transformación de datos procedentes de un archivo. Ambas se aplicaron y revisaron. El runner usa ProcessStartInfo con ArgumentList, plazo 30 segundos, terminación exclusivamente del árbol del proceso que creó y resultado de timeout como fallo. Guarda la identidad de Nu, binaries, módulo, suite, configuración y fixtures antes de ejecutar. Los reemplazos finales de operadores inexistentes por `take 8` y `char --integer 27` conservan los controles previstos.

El autor ejecutó la suite final en `.tmp/nu116-integration/package-run-r3`; el revisor la reprodujo una única vez en `.tmp/nu116-integration/package-adversary`, bajo el contenedor externo de Job con plazo 60 segundos. Ambos result.json indican exit_code=0, timed_out=false y failure=null. Cada manifest de entrada contiene ocho archivos cuyos tamaños y hashes se verificaron de nuevo después: sin diferencias.

Se comprobaron texto Unicode, registro con tipos, Binary value, ListStream, nothing frente a texto vacío, ByteStream real mediante open --raw/into string, CSV transformado de 42 a 43 sin reabrir su origen, cinco abandonos parciales de salida del plugin seguidos de reutilización en la misma sesión, redirección con paleta sin ESC y argumento literal. La salida completa esperada de cada caso coincide. No se inferió QA gráfica de estas pruebas redirigidas.

Identidades de ambas reproducciones:

| Entrada | SHA256 |
|---|---|
| Nu 0.116.1, D:\Program Files\nu\bin\nu.exe | E37DFF301CF52A49EB82A90E8B4634328379FFFA3B244B2172A1C307E03E5D69 |
| r3viewer.exe | 76154360FF0F5194E6FAAAE7747F871B6500255006AD22CD520DB6426FEF5131 |
| nu_plugin_r3viewer.exe | 8FFF8566E2E4A6CDDB0E37FEB0A471AB709882A9110088173A3B158EB972D031 |
| nu/r3viewer.nu | 511BE720C899118B963C0C8941E0EF917B59AF470736DCA1A9B99878DA09B435 |
| Suite Nu final | 899CC3FF6C24E038FC9A9311C6D92096899A34A3554936BEBB43A575C58FA8B5 |
| Runner PowerShell final | 2DD88B55282E5AF5F2048A9AD39264ADB4BDA4683AF6415CB84F3DE9A953E3AE |

La reproducción propia produjo `.tmp/nu116-integration/adversary-receipt.json`, SHA256 `36D44A6EC4753817A630129F085FF3DF4D0FFA39C4852675F1FD0C97274E4E7B`: espera, EOF, joins y cierres comprobados; Job con cero procesos activos antes/después de limpieza, Reaped, sin errores, cleanup_only ni propietario retenido. No queda proceso de esa ejecución vivo. El recibo de autor registra el mismo resultado físico. Los dos fallos anteriores del nuevo runner se conservaron; no fueron defectos de producción ni se contaron como aceptación.

## Límites

Esta aceptación corresponde a la migración del SDK y a la integración funcional Windows con el ejecutable identificado. No certifica ejecución nativa Linux/macOS, los nuevos tests cfg(unix), apariencia en terminales, todas las intercalaciones de señales ni la totalidad de S03. No modifica perfiles personales ni acredita publicación. Los controles de W3 y calidad ejecutados por el coordinador se registran por separado.
## Cotejo final de integración y calidad del coordinador

El revisor cotejó además los dos controles nuevos de W3 ejecutados por el coordinador con el host 0.116.1 y el companion identificado arriba: `.tmp/nu116-w3/harness/AUTHOR-redirect-outer.json` y `AUTHOR-terminal-outer.json`, sus recibos actor/host y los resultados semánticos. Ambos son PASS, exit0, esperas/joins/cierres acreditados, sin force ni Pending y Job final vacío. En redirección stdout contiene 298899 bytes y stderr está vacío. ConPTY captura 309023 bytes; el cuerpo decodificado de 298899 bytes coincide íntegramente con expected/golden, SHA256 `50AF0AB33B9E14BD075BB6508E3BF4AF23AEAE9C7E0526EEBEDC1588A5FED3AF`, incluidos 10000 tokens y 70000 bytes semánticos de sus hojas. Esto verifica correspondencia funcional; no apariencia. Son ejecuciones del coordinador, no reproducciones nativas adicionales del adversario.

Se cotejaron los logs finales de calidad del coordinador: `.tmp/nu116-integration/workspace-tests.log` suma 583 pruebas/ejemplos aprobados, cero fallos y 10 ignoradas separadas (512 aprobadas corresponden a la biblioteca del lector). Clippy release workspace/all-targets con -D warnings termina correctamente; el coordinador registra además build release, fmt y nu-check del módulo satisfactorios. No se atribuyen esos checks a una nueva ejecución del revisor. Investigación y VALIDACION distinguen correctamente estos resultados de las dependencias Unix/visuales pendientes. README prepara un bundle nuevo para el ejemplo 0.116.1 y evita reutilizar el paquete histórico 0.115.1.

El dictamen focal continúa **SIN_CRITICAS_PENDIENTES**; los límites anteriores se mantienen.