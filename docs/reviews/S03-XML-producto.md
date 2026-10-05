# S03 D4 — XML integrado, candidato R1

2026-10-05. [Revisión adversaria de código](S03-XML-code-r1.md) **SIN_CRITICAS_PENDIENTES** y [aceptación independiente](S03-XML-aceptacion-r1.md) **PASS** en la cobertura funcional Windows ejecutada. No completa S03, RV-106 ni el producto.

## Comportamiento disponible

Un archivo real `.xml` conserva Fuente inicial; Tab/BackTab ofrece Estructura. `--format xml` permite interpretar literal, stdin y String/StringStream Nu; `--language xml` sólo selecciona resaltado. `--structure` solicita estructura inicialmente. Binary/Unknown y demás valores Nu mantienen su contrato tipado: no se adivinan como texto ni se reabre un archivo transformado. SVG y HTML no seleccionan automáticamente este inspector.

Inspector XML 1.0 y Namespaces 1.0, UTF-8/BOM, una raíz. Mantiene atributos/declaraciones namespace en orden físico, texto mixto, referencias, CDATA, comentarios, PI, declaración y DOCTYPE. Los valores siguen siendo texto. DOCTYPE es opaco e inerte, sin validación DTD, atributos por defecto ni expansión; usar una entidad personalizada conserva Fuente completa con diagnóstico. Ningún URI de namespace, XInclude, stylesheet, SYSTEM o xml:base se usa como permiso para cargar recursos.

Búsqueda separa QName, nombre local, URI, valor interpretado y metadatos según el tipo de nodo; las etiquetas de interfaz no crean coincidencias. No concatena nodos o dominios. `o` selecciona el token/atributo/elemento original completo. Fuente y Estructura guardan consulta y posición propias. `K` abre QName cuando existe, `m` inspecciona metadatos, Backspace restaura el contexto. Fuente busca bytes originales y proyecta sólo la coincidencia elegida a SafeText, sin emitir controles activos.

## Identidad y comprobaciones

Freeze `r3viewer-s03-xml-review/R1`, base `49535cc1a7c623d840e111f0ef092073e9cda684`: 355 entradas de MANIFEST verificadas por ambos revisores. Manifest SHA256 `D4F6ABDEA5F9A59B79FB5FC8E920A360DB4603061C65987AC7F1839DBBAB8270`; app `18927840E727B1778AB1BBA2808603551E48AB9822513C2E0A7512D17C6846AE`; companion `E885DE7F826DA9520EA7E6A11139F57F966A9BDB4E9CFEB5653A6BFDB6D5567E`; módulo Nu `38CF6B9C06CACA991CAD43DD51E02834270713D2E40AFCE7D6E385E2F247768C`.

Coordinador: 213 pruebas del workspace (157 lib + 23 CLI + 5 SDK + 12 mapper + 16 wire), formato, Clippy all-targets y build pasan; cuatro pruebas ignoradas explícitas del autor se distinguen de esa cifra. Adversario: 66 casos CLI propios, 47 comprobaciones API propias, reejecución de 157 tests y tres gates grandes/nativos. Aceptación: 89 CLI/Nu, cuatro tests de modelo con 31 estados y 28 rangos originales, ocho sesiones Nu/ConPTY y productor SDK real de ByteStream String/Binary/Unknown.

Los gates grandes comprobaron campos de 1 MiB, rechazo completo por arena en 200000 atributos/declaraciones namespace, profundidad 64/65 y fuente 128 MiB ±1. A 128 MiB el parser puede terminar por Timeout a cinco segundos; admitir tamaño no promete Ready. La cuota de un millón de items se comprueba por admisión/IPC; no se afirma materializar un modelo de ese tamaño cuando la arena lo rechaza antes. Job Windows observado configura 512 MiB por proceso y kill-on-close, separado de memoria total y de causalidad exclusiva de un rechazo de reserva.

Cuatro sesiones adicionales del coordinador, con evidencia inspeccionada por adversario y aceptación, prueban deadline TUI bajo suspensión de worker propio, exclusión sintaxis/estructura por FILETIME/handles, q, Ctrl-C130 y cancelar/reintentar con identidad nueva. Sin procesos propios residuales ni limpieza forzada. ConPTY prueba funcionamiento y restitución de pantalla; no es QA visual de Windows Terminal/WezTerm.

## Cambios compartidos revisados

`reserve` admite el pico de capacidades vieja+nueva antes de crecer, también en TOML/JSON/YAML; el rechazo conserva contenido. Puede rechazar antes una construcción que sólo cabría tras liberar el buffer viejo. Fuente original/visible separada afecta las vistas de texto fuente, sin cambiar la navegación de valores Nu tipados. XBCANCEL01 corrige en XML y YAML la comprobación de cancelación sobre caracteres UTF-8: umbrales cruzados en vez de posiciones exactas. Dos pruebas activan cancelación dentro del recorrido, y la regresión heredada pasa.

## Evidencia y pendientes

Informes originales: código SHA256 `4E23F1878179001EAE9092FEEBB076C33B968841F1DC2F4848F9F5D670F57FB0`; aceptación `D1BA38D543DB8DF7413C0D53969E9E87513245BE95C44FF8CE72FDF98FC3AA0D`; artefactos AE `23D1ED34F3BE05ADEEABE2B29D93A14A0A7A760AC47F6E824B3A608B43C6A185`. Fuente de evidencia privada: `C:/Users/Usuario/.codex/visualizations/2026/10/04/01a10823-9b5a-78d3-8c6a-c64ff5709d5f`. Los informes canónicos son copias de los finales; corpus, ejecutables y transcripciones privados no se incorporan a Git.

AE renovó G4 con el parser de producción exacto: siete casos y controles positivos propios antes/después. Observa intentos mediante APIs NT y connect del wrapper propio, junto con el callgraph puro; no equivale a trazado kernel/ETW ni garantiza ausencia universal de I/O. El [addendum adversario](S03-XML-g4-produccion-r1.md), SHA256 `44D59B0C2E592A240D6BF733AE17522B174FDE85A48D132AFD1C19CECFEA176D`, cierra sin críticas el alcance G4 observado de producción, verificando 62 artefactos y la copia de las 355 entradas. Se conserva la distinción entre monitor acotado, callgraph e hipótesis universales; no se extiende automáticamente la aprobación del prototipo.

Permanecen pendientes estrés independiente exhaustivo de presupuestos/carreras/IPC/lifecycle no cubiertos, deadline inline del controlador real, contención y ejecución Unix, QA visual, gramáticas adicionales y distribución. Compilación cruzada del lector/wire Linux/macOS pasa; ejecución nativa no comprobada. La integración funcional parcial no cierra RV-105/106/107 ni reduce requisitos de formatos enriquecidos.
