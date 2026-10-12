# Migración de integración a Nushell 0.116.1

2026-10-12. Host seleccionado por el usuario: `D:\Program Files\nu\bin\nu.exe`, versión real0.116.1, SHA256E37DFF301CF52A49EB82A90E8B4634328379FFFA3B244B2172A1C307E03E5D69. Base limpia da9df0d. nu-plugin, nu-plugin-core, nu-protocol y el stack Nu del lockfile quedan fijados a0.116.1. La [regla oficial de compatibilidad de plugins](https://www.nushell.sh/contributor-book/plugins.html) exige la misma versión menor antes de1.0; el runtime probado aquí es0.116.1 Windows, sin certificar otras versiones/plataformas.

Los dos crates vendorizados proceden de archives oficiales, con sus MIT y hashes raw/LF en UPSTREAM.json. Los cinco archivos upstream afectados por nuestros parches de polling/señales eran idénticos entre0.115.1 y0.116.1; las modificaciones locales se reaplicaron sin ajustes API ni cambio de protocolo propio. Cambios oficiales de manifests y sockets Unix conservados, sin validar tests cfg(unix) aquí. [Revisión independiente](../reviews/NU116-migracion-final.md) verifica35/35 identidades y deltas; SIN_CRITICAS_PENDIENTES focal.

Se actualizaron README, módulo Nu y guías vigentes a la instalación de D:. Nuevo runner PowerShell configurable por NuExe/BuildDirectory, con plazo30s y registro de timeout como fallo; crea paquete/config/fixtures privados nuevos y carga el companion sólo en esa invocación. Prueba tipos, null/vacío, Binary/ListStream/ByteStream real, CSV transformado42→43 sin reabrir origen, cinco abandonos y reutilización en la misma sesión, paleta redirigida y argumento literal. Autor y adversario pasan, con Job externo60s, waits/EOF/joins/cierres y cero procesos activos. No perfiles ni registro permanente de plugins.

| Verificación nueva | Resultado |
|---|---|
| Companion cargo check | PASS |
| SDK aislado |43plugin +78core +17ejemplos PASS, sin ignored |
| Build release workspace |PASS |
| Formato y nu-check del módulo |PASS |
| Regresión release workspace serial con workers/config privados |583pruebas y ejemplos PASS;10ignored separados |
| Clippy release workspace/all-targets -D warnings |PASS |
| Integración real Nu0.116.1, autor y adversario |PASS |
| Salida larga redirigida y ConPTY funcional |298899bytes íntegros iguales a expected/golden; raw309023bytes; PASS |

Reader release conserva SHA76154360FF0F5194E6FAAAE7747F871B6500255006AD22CD520DB6426FEF5131; companion nuevo8FFF8566E2E4A6CDDB0E37FEB0A471AB709882A9110088173A3B158EB972D031. Evidencia privada nueva `.tmp/nu116-sdk`, `.tmp/nu116-integration` y `.tmp/nu116-w3`, sin sobrescribir freezes/runners/resultados históricos0.115.1. Los nuevos intentos fallidos del runner (`bytes take` y `char escape` inexistentes) se conservan; corregidos a `take 8` y `char --integer 27`, sin defecto productivo ni cambio de oráculo. La compilación fría observada y los timings de pruebas no son SLA.

Esta migración no cierra QA visual, Unix/U-G1–5, S03 ni otras historias. ConPTY acredita correspondencia funcional, no apariencia. No instalación, perfiles, remoto ni publicación.
