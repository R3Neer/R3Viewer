# R3Viewer

Visor de archivos, documentos y valores de Nushell, con lectura interactiva y salida inline (`--inline/-i`). Windows nativo, macOS y Linux son objetivos de soporte.

Estado: desarrollo autorizado; lectura textual e integración Nu tipada comprobadas funcionalmente en Windows. Todavía no hay una entrega certificada. Implementación, revisión iterativa y commits locales autorizados; publicación e instalación en perfiles personales fuera del encargo.

## Documentación

- [Requisitos](docs/spec/REQUISITOS.md), [decisiones](docs/spec/DECISIONES.md) y [preguntas](docs/spec/PREGUNTAS.md).
- [Contratos](docs/spec/CONTRATOS.md), [HTML/web](docs/spec/WEB_HTML.md) y [estudio previo](docs/spec/ESTUDIO_ADICIONAL.md).
- [Product Backlog](docs/BACKLOG.md): 39 tareas, dependencias y aceptación; diez épicas en el plan general.
- [Investigación técnica](docs/research/INVESTIGACION_TECNICA.md): fuentes, conclusiones y límites de evidencia.
- [Arquitectura general y épicas](docs/architecture/ARQUITECTURA.md), revisada iterativamente con adversario independiente hasta cierre sin críticas, y [validación por capacidad](docs/VALIDACION.md).
- [Política de commits](CONTRIBUTING.md).

## Contrato objetivo; cobertura actual indicada más abajo

```nu
viewer notas.md
viewer -i notas.md
viewer https://ejemplo.org
"notas.md" | viewer
ls | viewer
viewer --value "# Un título"
"notas.md" | viewer --path
```

Argumentos abren rutas o URLs; el pipeline contiene valores. `--path/-p` y `--value/-v` fuerzan la interpretación y no coexisten. Se mantienen `open` y `explore` nativos; `from html` será una utilidad opcional.

## Ubicación canónica

Repositorio local: `D:\Tools\R3Viewer`. Los documentos iniciales de este chat se copiaron conservando el historial como primer commit. A partir de la migración las modificaciones se hacen aquí; las copias del chat son una referencia histórica.

No hay remoto configurado. Licencia del proyecto: [MIT](LICENSE), confirmada el 2026-10-04. Las dependencias conservan sus licencias y avisos; revisarlos antes de distribuir.

## Forma de trabajo

Adaptación de Scrum por objetivos: un Product Backlog, un sprint activo sin cadencia temporal fija y detalles refinados según prioridad/riesgo. Responsabilidades, aceptación y Definition of Done en docs/process/METODOLOGIA.md. Revisiones iterativas independientes de propuestas y código; informes intermedios temporales, versiones finales y evidencia canónicas.

## Desarrollo vigente

El usuario autoriza implementar el producto completo siguiendo [Scrum por objetivos y revisión adversaria](docs/process/METODOLOGIA.md), sin cadencia fija. La descripción de preparación anterior es histórica. Sin publicación ni modificación de perfiles personales.

## Incremento textual disponible para desarrollo

S01 contiene lector UTF-8 local/literal/stdin externo, salida textual y sesión de terminal. S02 incorpora el módulo público `viewer` y companion privado `nu_plugin_r3viewer`; transporta valores tipados de Nushell al ejecutable `r3viewer`. S03 añade navegación estructurada, búsqueda y presentación configurable; su candidato está en validación independiente. Los formatos enriquecidos y la distribución final siguen pendientes.

```powershell
. ./scripts/dev.ps1 # si se usa el toolchain privado .tools
cargo test --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo run -- --value "Texto literal"
cargo run -- -i archivo.txt
```

Para otro entorno de desarrollo se necesita Rust 1.99.0 conforme a rust-toolchain.toml y compilador/enlazador de su plataforma. La configuración privada de esta sesión no cambia PATH global ni perfiles. Cargo.lock fija dependencias. El paquete final no requerirá Rust al usuario.

Límites S01: solo texto UTF-8, hasta 128MiB por entrada; URL, formatos enriquecidos y --path desde pipe textual diagnosticados como pendientes. Pipeline externo textual no preserva tipos Nu. -p y -v incompatibles; stdin vacío válido, sin entrada con TTY error2. ESC/OSC y controles peligrosos se muestran visiblemente. Cancelación devuelve130; BrokenPipe sin cancelación es salida normal. Sesión: flechas/hjkl, páginas, Home/End, ?, q/Escape y Ctrl-C.

Validación actual: 15 pruebas, fmt y Clippy Windows; compilación/Clippy de partes Linux sin ejecutar Linux. Revisión adversaria de código R2 sin críticas pendientes; aceptación textual independiente. ConPTY funcional comprobado; QA visual Windows Terminal/WezTerm y resto de matriz pendiente. Detalle: docs/sprints/S01.md. Script opcional de reproducción de señales Windows: `python scripts/test-cancellation-windows.py target/debug/r3viewer.exe` (binario debug, grupos hijos aislados; no prueba visual y falla como inconcluso si el proceso acaba antes de señal).

## Prueba de desarrollo con Nushell

Construir workspace y preparar directorios privados bin/ y nu/ con los dos ejecutables y nu/r3viewer.nu. Cargar companion temporalmente en Nu0.115.1 con --plugins y usar módulo, sin registrar plugins ni cambiar perfiles. En esta sesión existe .tmp/package para reproducir:

```powershell
& 'C:/Users/Usuario/AppData/Local/Programs/nu/bin/nu.exe' -n --plugins '[D:/Tools/R3Viewer/.tmp/package/bin/nu_plugin_r3viewer.exe]' -c 'use D:/Tools/R3Viewer/.tmp/package/nu/r3viewer.nu *; {nombre: "Ana", puntos: 42} | viewer -i'
```

Validación S02: 48pruebas del workspace, Clippy/fmt, revisión independiente R1 sin críticas y aceptación de tipos, errores, límites y abandono. ConPTY propia confirma Ctrl-C130, modos restaurados y siguiente invocación funcional; no acredita QA visual. [Detalle y límites](docs/sprints/S02.md). Custom ajenos no son universalmente transportables: Nu los rechaza por propietario. No se convierten ejecutando callbacks.

## Candidato S03 en validación

La TUI permite Enter para inspeccionar una celda/hoja, r para la fila completa, K para una clave completa y Backspace para volver conservando selección. / abre búsqueda literal sensible a mayúsculas; Enter confirma, n/N recorren y vuelven por los extremos. Escape cancela el editor; fuera del editor sale. L alterna números de línea. Consulta máxima4096bytesUTF-8. Nothing, ausencia y string vacío son estados distintos; la búsqueda incluye claves y hojas colapsadas completas.

Opciones de presentación: --colour auto|always|never, --theme auto|dark|light, --line-numbers/-n y --no-line-numbers. --config indica TOML de lectura, máximo64KiB. Flags prevalecen sobre configuración; auto respeta NO_COLOR/TERM=dumb. Redirección siempre plana. Esquema y rutas estándar en [S03](docs/sprints/S03.md). No crear perfiles ni archivos de configuración implícitamente.

A/B tiene aceptación funcional Windows en R4. C añade resaltado en TUI e inline con 114 pruebas, revisión independiente sin críticas y aceptación funcional parcial Windows. Código, lectura y datos son de solo lectura; los formatos enriquecidos siguen pendientes. Compilación cruzada del lector comprobada para Linux x86_64 y macOS arm64; ejecución en ambos y QA visual todavía pendientes.

Resaltado: --format auto|text|code y --language por nombre/extensión integrada. Auto deduce únicamente la extensión del archivo realmente abierto; pipeline/literal necesita --language. Text con --language es error2; lenguaje desconocido o code sin lenguaje deducible conserva fuente completa plana con diagnóstico. Plain Text, colour never y redirección evitan iniciar el motor. String y string_stream Nu explícitos son textuales; Unknown/Binary mantienen su tipo. No cargar gramáticas arbitrarias ni reabrir metadata.

```nu
viewer archivo.rs
"let saludo = \"hola\";" | viewer --language rs
viewer -i -n archivo.rs
```

Un fallo detectado o presupuesto excedido del motor deja el documento entero plano; no bloquea navegación/búsqueda. Inline espera el resultado completo antes de imprimir. El aislamiento de sintaxis está validado funcionalmente en Windows; en Unix el lector compila como candidato con sintaxis plana y diagnóstico hasta validar su contención y ejecución. Catálogo real en src/language_catalog.rs: incluye TOML1.1 con recurso versionado; Nushell y PowerShell siguen pendientes; la cobertura adicional sigue abierta. Detalles y límites en [S03](docs/sprints/S03.md), [revisión](docs/reviews/S03-sintaxis-integracion-r1.md) y [aceptación](docs/reviews/S03-sintaxis-aceptacion-r1.md).


## Estructura opcional de archivos de datos

TOML, JSON y YAML conservan Fuente como vista inicial. Para un archivo real .toml/.json/.yaml/.yml, Tab/BackTab cambia a Estructura y vuelve manteniendo posición/búsquedas; `--structure` pide Estructura inicialmente. `--format toml|json|yaml` selecciona parser para literal/stdin/string de Nu. `--language` solo selecciona resaltado, no parser; `--format text --structure` es error2.

```nu
viewer config.toml
viewer datos.json --structure
'{"a":1,"a":2}' | viewer --format json --structure -i
```

Estructura conserva orden y lexemas originales, incluidas todas las ocurrencias JSON con claves repetidas y una advertencia; no convierte números enormes a float. Buscar Estructura compara contenido decodificado, con identidad separada de placeholders de controles. `o` salta al span original completo acreditado; si no existe, conserva la vista y diagnostica. Fallo de parseo/límite mantiene Fuente completa con diagnóstico y código0; cancelar devuelve130. Datos Nu ya tipados mantienen navegación y no se reabren ni se convierten a JSON/TOML/YAML.

YAML conserva documentos, claves complejas, anchors y alias por identidad. Enter sigue alias explícitamente; Backspace restaura hasta128retornos sin expansión automática. Tags desconocidos quedan inertes; versiones distintas de1.2 dejan Fuente completa. Búsqueda distingue contenido decoded, URI y descriptor alias de los placeholders visibles. No aplica merge YAML1.1 ni ejecuta constructores.

[Estado JSON y rondas](docs/reviews/S03-JSON-producto.md), [TOML](docs/reviews/S03-TOML-producto.md), [YAML R3](docs/reviews/S03-YAML-producto.md). Aceptación funcional parcial Windows; no certifica visual/matriz, presupuestos completos ni RV-106. [XML](docs/sprints/S03-D4-XML.md) tiene contrato y arquitectura revisados; gates experimentales pendientes, sin implementación integrada.
