# D4b R4: comprobación raíz del candidato final

Fecha: 2026-10-10. Alcance: integridad y ejecución del binario congelado del autor; revisión independiente de código y compilación propia todavía pendiente. No adopción de producción.

Raíz cotejó los 958 archivos del manifest `E4FDB2B4D43AEB2805A7A60FAA8B5105976AD560C63BAC1DF1841BB89EE23FC0`: todos los tamaños y SHA-256 coinciden. El candidato contiene 840 fuentes, ocho cambios respecto a R3 y 46 rutas de código/fixtures respecto al origen canónico; incluye dependencias D2c provisionales.

Se ejecutó `REPLAY.ps1` en proceso independiente sobre los binarios congelados, con salida y código de terminación conservados en `.tmp/root-r4-final-runtime-2026-10-10`. Resultado: exit0; 17 controles físicos del scheduler, 25 controles normales y cuatro controles del lector pasan. Los 17 controles físicos ignorados por la pasada normal se ejecutaron expresamente en la primera pasada; no se interpretan como pruebas omitidas aceptadas.

Identidades pre/post:

| Artefacto | SHA-256 |
|---|---|
| Host de pruebas | `60E87FA5A3C96FDD56F39C231C49C6E1F81737D5C40B2072317117CBF9170382` |
| Worker | `1DBC55436015D731115900B6EB7B7061CD2F5A8093A45FBF52288CC1490C5128` |
| Fixture TOML original | `7B0F804648294BF03962981A297919630DA419EA092F87415D8ADD9C7C91F7C7` |
| Log de ejecución raíz | `056E025DAC0931A68D2FBE35110F51C8EFAC053671C12B02214897C9D09CD1B3` |

El host incorpora la ruta de compilación del autor para el fixture TOML; se verificó ese archivo antes y después. Por ello no se presenta como ejecución hermética ni compilación propia. Se conserva el perfil cache256, Work8MiB y plazo2s. Los nuevos controles separan retención de evento durante fase ocupada de supersede tras Styles con el mismo proceso/contexto; no sustituyen la matriz original DS/O/DF/DM.

El formato global sigue fallando (exit1 del autor), aunque pasan las siete rutas focales. La pérdida declarada del ejecutable del primer intento fallido permanece como limitación histórica. I01/P01 requieren dictamen independiente sobre el candidato exacto; integración completa D2c/D4c, provider inline, gráficos, QA y ejecución Unix siguen pendientes. Esta comprobación no cierra tareas RV.
