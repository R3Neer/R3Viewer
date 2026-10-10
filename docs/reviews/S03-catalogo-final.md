# Revisión independiente catálogo S03 R3
SIN_CRITICAS_PENDIENTES en el alcance focal catálogo/selección/CLI help/version Windows.

## Identidad y pertinencia
Manifest R3 original 4AB08F1997F4794A56DD4DAAA095C5C0D712CAEAE452CD807D75421B7C485F8B:22/22 entradas bytes/SHA conformes, incluidos libr3viewer.rlib, selector/worker/build/CLI, recursos fijos y binarios. Criterios CA01–06 propios fijados antes de leer probe/output.
Manifest propio BEFORE output70546B4964A20479E0AD1923A689B8801865FDC0FA8FB2969CD7444E7906656D. Observador propio compilado rustc exit0 E868ABF2C9F41AA2645F0D88D896221F73D77627D97E9712284C5E8DEFD54D09; enlaza copia física libr3viewer congelada D0192D3A07FA4339B233DAC341C452B7ED0A0F3DBB5FA847586F638910160CCE. Esto es compilación propia del observador, NO recompilación independiente de la biblioteca/producto.
Replay propio sobre copia probe-r3 5C71CDF6102F2834E170B467742641E88FE1A83FFDA64880A147CE578A32487A. Help/version sobre copia app2063423E082E7791EF788FB98EDC991AC27CDBF87CB7F6E5A8CBB57A2C8F301A. Hashes pre/post iguales.

## Matriz
| Criterio | Evidencia propia | Disposición |
|---|---|---|
| CA01 identidad/previos |22 checks input PASS; propios prehash/posthash conformes; logs/oráculos R1/R2 físicos preservados | PASS |
| CA02 catálogo real | Replay carga bundled_syntaxes(),78 entradas y244 tokens; build genera catálogo y pack del mismo SyntaxSet ordenado | PASS focal |
| CA03 prioridad original |78 nombres/244tokens y archivos Auto del replay;322 tokens propios a través SyntaxSelection en Literal cotejados contra pack; resolución reverse extensión antes forward nombre | PASS |
| CA04 origen/alias/fallback | Own explicit .Rs con trim, pwsh/PWSH→PowerShell, Auto.pwsh→Plain, Literal→Plain, desconocido Plain+diagnóstico, Plain Text plano, rs explícito sobre archivo.py, Text+language validate error | PASS |
| CA05 CLI | Help/version exit0, versión0.1.0, sin documento/input abierto ni worker invocado por observadores | PASS focal |
| CA06 límites | No fuente arbitraria, corpus universal, render, QA o Unix ejecutados | Pendientes externos conservados |

Todos los cuatro procesos observados retornan exit0: own probe, replay, help, version. No propio test de representación visual o resaltado real de cada gramática; el pack se deserializa y se comprueba selección, no ParseState por lenguaje.

## Semántica para aclaración documental
Extensión exacta primero, luego sin punto opcional, ambas en orden inverso; después nombre case-insensitive normal. Auto sólo Path::extension final de LocalFile real. No igualdad entre token explícito compuesto css.erb y extensión final erb de ruta fixture.css.erb; no cambiar producto a reconocimiento de sufijos compuestos.
Colisiones reales de token/nombre: ASP selecciona HTML(ASP), TeX selecciona LaTeX por prioridad de extensión; no implica catálogo corrupto. Alias pwsh es explícito y no añade extensión automática.pwsh.
Los FAIL previos R1/R2 resultan de expectativas del observador que imponían prioridad de nombre o sufijo compuesto; permanecen rechazados. No se reportan como defecto de producto ni como outputs PASS retroactivos.
Se puede documentar esta prioridad y las colisiones existentes con alcance preciso. No cierre S03 integral, catálogo universal, rendimiento de gramáticas, QA visual/Unix ni suficiencia de licencias.
