# Investigación técnica inicial

Registro inicial histórico: 2026-10-04, preparación de arquitectura antes de implementar. Sus snapshots de entorno y afirmaciones de ausencia de código/toolchain no son el estado vigente. Implementación y validación posteriores se registran en ../VALIDACION.md y ../sprints; gráficos y certificación visual siguen pendientes.

## Resumen de resultados

| Cuestión | Resultado y decisión de trabajo | Evidencia y límite |
|---|---|---|
| Lenguaje | Rust para núcleo, render y plugin Nu; Node privado para Mermaid/MathJax | Rust permite compartir nu-protocol; motores Node evitados al leer texto. Elección técnica de análisis, pendiente de spike antes de desarrollo amplio. |
| Tipos de Nushell | Plugin pequeño de conversión tipada + módulo Nu que lanza visor externo | Plugin ocupa canal de protocolo: no usar su stdout para TUI. Versionar el plugin junto a la versión Nu soportada. |
| Teclado con pipeline | Canal de datos por stdin; terminal de control independiente | Crossterm Unix usa TTY; Windows permite CONIN$/CONOUT$ aun con streams redirigidos. Viabilidad documental, no prueba de Nu↔PTY. |
| Gráficos | Ratatui + Crossterm + ratatui-image; backend inline dedicado | Soporta Sixel/Kitty/iTerm2 y fallback por bloques. No asumir que widget TUI resuelve scrollback. |
| PDF | Priorizar PDFium en worker aislado sobre MuPDF | PDFium aporta render, texto y enlaces; necesita binario nativo distribuible. MuPDF AGPL/comercial añade revisión de licencia. |
| HTML | html5ever + árbol para núcleo y from html | HTML5 tolerante; DOM separado del documento de lectura. No Chromium para HTML ordinario. |
| Markdown | pulldown-cmark + extensión Obsidian en AST | Composición propia para gráficos; no convertir documentos a texto pintado antes de resolver recursos. |
| Mermaid | CLI oficial, worker Node/Puppeteer | Mayor cobertura que alternativas parciales; pesa por navegador. No prometer todos los dialectos/versiones sin corpus. |
| Matemáticas | MathJax a SVG, resvg a raster | Fórmulas, no documento LaTeX arbitrario; guardar métricas de baseline/altura, no solo PNG. |
| SVG/imágenes/código | resvg, image, syntect | Evita procesos bat/pager y dependencias gráficas duplicadas por defecto; revisar recursos SVG y fuentes. |
| Distribución | Paquete completo por SO/arquitectura con avisos y hashes | Binarios, Node, Chromium y PDFium privados. Instalación futura sin descargas al abrir. |
| R3CLI/Show-Tree | Roles discretos y adaptación de datos | No importar sus renderizados como modelo ni ejecutar recorrido recursivo obligatorio. |

## Entorno local inspeccionado

Git 2.45.1.windows.1 y Node 24.16.0 encontrados en PATH. nu, rustc y cargo no se resolvieron mediante Get-Command; tampoco se localizaron en las rutas convencionales inspeccionadas. Esto no demuestra que no estén instalados en otra ruta.

env.nu existente declara que fue generado por Nushell 0.114.1: indicio histórico, no verificación de binario activo. config.nu carga módulos Show-Tree y ModpackTools; no se han editado perfiles. No se pudieron determinar versiones activas de Windows Terminal y WezTerm desde esta sesión. No se cambió configuración global ni se instaló nada.

Objetivo inicial de integración: Nu 0.114.1 como candidato, sujeto a confirmar ejecutable activo antes del spike. Código del tag 0.114.1 confirma búsqueda de from <ext> en http get; no depender solamente de main para esta conclusión.

## Puente Nushell y alternativas

1. JSON simple: se descarta como transporte universal, pierde distinción de filesize, duration, binary/date y valores especiales salvo etiquetado.
2. NUON: alternativa viable para muchos tipos; --serialize admite representaciones que no se pueden deserializar. No prometer roundtrip universal ni recoger streams gigantes en una única cadena. Sigue necesitando parser y versión compatibles.
3. Plugin con render directo: no elegido. stdin/stdout pueden transportar protocolo; escapes del visor contaminarían comunicación y captura por Nu. Soporte por sockets no elimina el problema de coordinar la UI.
4. Plugin tipado + proceso externo: elegido como base de diseño. Plugin produce frames de un transporte propio etiquetado; módulo Nu conecta al ejecutable. El ejecutable posee terminal y render. Costo: un binario compañero vinculado a Nu y validación de señales/metadata.

Tipos iniciales del transporte: nothing, bool, int, float (incluidos no finitos mediante representación etiquetada), string, binary, date con offset, filesize en bytes, duration en nanosegundos, listas y registros ordenados, range y cell-path como descriptores sin evaluar. Error como nodo diagnóstico. Custom values: intentar representación base permitida por Nu o descriptor opaco explícito; no fingir pérdida cero para objetos de otros plugins. Closures y valores ejecutables se describen sin ejecución. El documento no reabre archivos por metadatos después de transformar valores.

Entrada finita con frames y backpressure: evita obligar al puente a acumular todo antes de mostrar primera vista. No convierte producto en monitor de streams infinitos. Un frame contiene versión/esquema y registros limitados; cancelación y límites abortan productor con diagnóstico. Protocolo interno especificado en arquitectura; selección final MessagePack/framing se valida en el spike.

## Terminal y composición

Usar TTY de control para eventos/probes, nunca leer bytes del payload como respuesta del terminal. stdout redirigido produce solo texto; aunque exista CONOUT$, no saltarse redirección para dibujar gráficos. Sin terminal de control no activar raw-mode y producir salida textual con diagnóstico útil si fue requerida interacción.

Probes con timeout y una sola cola de eventos: respuestas del terminal no deben perder pulsaciones de teclado ni bloquear arranque. `Picker::from_query_stdio` no es seguro como llamada ciega si stdin es un pipe; adaptar a handles correctos o detección explícita/fallback. Crossterm, ratatui-image y Nu requieren comprobación conjunta.

Windows Terminal: usar Sixel cuando la versión y probe lo permitan; no garantizar gráficos por nombre. WezTerm: protocolos iTerm2 y Sixel/otros según negociación. Kitty opcional donde se compruebe. Halfblocks o referencias textuales identificadas si no hay gráficos reales.

Ratatui-image contempla área de celdas, tamaño de fuente y evitar que el compositor borre la imagen. Sixel puede requerir redibujado completo al desplazar; los protocolos tienen estado y eliminación diferentes. Inline necesita colocación y scrollback propios; no reutilizar blindly la alternate screen. Pantalla completa y scrollback serán QA separados.

## Render de formatos

- HTML5: html5ever requiere representación de árbol (rcdom o equivalente), no proporciona un DOM de aplicación completo por sí solo. from html conserva source origin y nodos; viewer adapta al documento semántico. Parser compartido evita duplicar reglas.
- Markdown: parser estructural con spans y enlaces; extension Obsidian reconoce wikilinks, embeds, callouts, bloques y headings sin alterar código/fórmulas. Resolver en bóveda indexada; namespaces y referencias relativas antes de render.
- Mermaid: worker con versiones fijadas de CLI, Puppeteer y navegador distribuido. PNG puede ser preferible para diagramas HTML/foreignObject que resvg no cubra; SVG solo cuando el corpus confirme representación estática. Limitar red de worker y no cargar referencias remotas como efecto lateral del diagrama.
- MathJax: SVG con métricas de ancho, altura y profundidad; rasterización conservando baseline para inline. Fuente/glyph cache debe producir resultados autosuficientes para resvg. Corpus prueba fracciones, matrices, sumas, símbolos y texto próximo. Render gráfico inline puede ocupar más de una fila y recomponer párrafo.
- PDFium: librería privada de ABI/build fijados, no buscar indiscriminadamente DLL en cwd/PATH. Un worker de proceso controla render/text extraction, páginas físicas y enlaces; no activar JavaScript ni formularios PDF. La biblioteca no garantiza paralelismo seguro por threads: serializar llamadas dentro de cada worker o usar procesos separados. Buscar texto donde exista capa de texto y saltar a página/posición; sin OCR ni promesa de selección textual avanzada en v1.
- Imágenes: JPEG/PNG/WebP/GIF estático inicialmente, primera imagen/frame por defecto; no prometer animación automática. SVG estático y recursos locales controlados; fallback ante funciones no cubiertas. Otros codecs según inventario confirmado, sin introducirlos solo por default features.
- Código: syntect en proceso; preservar fuente y sangría, números y wrap configurables. No ejecutar paginadores externos ni código.

## Recursos y seguridad de la lectura

Resolver rutas según origen, URL final y base HTML. URL posicional descarga ese documento; propuesta --load-remote concede carga de sus recursos secundarios durante esa invocación, no permisos permanentes por dominio. En TUI acción individual carga recurso; inline sin flag muestra referencias. Limitar esquemas HTTP(S)/archivo local según origen; HTML remoto no abre automáticamente archivos locales. No disparar procesos por javascript:, data ejecutable u otros esquemas.

Cache fuera del repo/documentos en directorios de plataforma, clave de contenido/opciones/versión del motor, LRU y límites. Memoria temporal de recursos remotos y cache persistente se diferencian: URLs autenticadas no persisten cuerpo ni tokens por defecto. Log sin cabeceras sensibles. No crear archivos dentro de Clases.

Valores de trabajo propuestos para spike: caché 512 MiB; cuerpo remoto 64 MiB; pipeline tipado 128 MiB; bitmap máximo 16 millones de píxeles; embeds profundidad 8; workers gráficos 2; timeout remoto 30 s; probe terminal 250 ms. Son configuración inicial revisable, no métricas acreditadas. Archivos grandes pueden inspeccionarse por ventanas sin límite global de tamaño arbitrario.

Metas experimentales de cancelación: UI atiende en 250 ms y worker termina en 1 s tras cancelación; primera vista de corpus pequeño en 1 s excluyendo inicio frío de Chromium. Si no se cumplen se ajusta diseño o se publica límite; no afirmar que hoy se cumplen.

## Compatibilidad y distribución

Matriz candidata, no soporte probado: Windows 10/11 x86_64 con Windows Terminal estable que soporte Sixel y WezTerm; macOS arm64 con WezTerm; Linux x86_64 con WezTerm. No se reduce la intención Windows/macOS/Linux: versiones exactas, macOS x86_64 y Windows/Linux ARM64 quedan como extensión de matriz tras verificar bundles. Windows Terminal se especifica por release concreta y no por la antigua preview de introducción de Sixel.

No fijar MSRV/crates a latest/main sin compilar conjunto. El main consultado de ratatui-image era 12.0.0-rc.0 y activa chafa-dyn por defecto: elegir release estable y desactivar defaults no necesarios para evitar dependencia nativa accidental. Lockfile y manifest de distribución se crean con la implementación, no como archivos ficticios de esta fase.

Completo por defecto no significa arquitectura monolítica. El usuario no necesita instalar Python/Rust/Node global para usar el paquete final. Rust será requisito de desarrollo; runtime Node/Chromium privado solo se inicia para capacidad que lo necesita. Instalador futuro Nu usa manifiesto de archivos/registro, integración reversible y perfiles preservados.

Licencias: proyecto MIT confirmado por el usuario. Los candidatos y dependencias conservan sus licencias. PDFium tiene licencia BSD-style y dependencias propias; pdfium-render requiere su propio aviso. MuPDF se documenta como AGPL/comercial, por eso no se adopta automáticamente. Chafa no se vincula por defecto. Mermaid/MathJax, Chromium, fuentes y codecs se inventarían con su versión exacta antes de distribuir, incluyendo notices y obligaciones de binarios. Esta revisión de candidatos no acredita legalmente un bundle que aún no existe.

## Evidencia que falta ejecutar, trasladada al backlog

Spikes por capacidad: Nu↔encoder↔stdin↔TTY, cancelación y restauración, imágenes TUI/inline en Windows Terminal/WezTerm, PDFium ABI/render/text y Mermaid/math cache en paquete frío. No hay runtime QA disponible en esta fase; RV-008 coordina estas validaciones antes de ampliar cada capacidad dependiente, sin gate global para todo el lector.

## Fuentes primarias consultadas

- [Plugins Nu](https://www.nushell.sh/book/plugins.html), [nu-plugin](https://docs.rs/nu-plugin/latest/nu_plugin/), [nu-protocol](https://docs.rs/nu-protocol/latest/nu_protocol/), [NUON](https://www.nushell.sh/commands/docs/to_nuon.html).
- [HTTP conversión Nu 0.114.1](https://github.com/nushell/nushell/blob/0.114.1/crates/nu-command/src/network/http/client.rs).
- [Crossterm](https://docs.rs/crossterm/latest/crossterm/), [TTY Unix](https://github.com/crossterm-rs/crossterm/blob/master/src/event/source/unix/tty.rs), [Console Handles Microsoft](https://learn.microsoft.com/en-us/windows/console/console-handles).
- [Ratatui](https://docs.rs/ratatui/latest/ratatui/), [ratatui-image](https://github.com/ratatui/ratatui-image), [manifest](https://github.com/ratatui/ratatui-image/blob/master/Cargo.toml).
- [Terminal releases](https://github.com/microsoft/terminal/releases), [WezTerm protocolos](https://wezterm.org/escape-sequences.html).
- [PDFium wrapper](https://github.com/ajrcarey/pdfium-render), [PDFium licencia](https://pdfium.googlesource.com/pdfium/+/refs/heads/main/LICENSE), [MuPDF licencia](https://mupdf.readthedocs.io/en/latest/license.html).
- [html5ever](https://github.com/servo/html5ever), [pulldown-cmark](https://docs.rs/pulldown-cmark/latest/pulldown_cmark/), [syntect](https://github.com/trishume/syntect).
- [resvg](https://github.com/linebender/resvg), [Mermaid CLI](https://github.com/mermaid-js/mermaid-cli), [MathJax Node](https://docs.mathjax.org/en/latest/server/direct.html).

Inspección local adicional: README/diseño/licencia de D:\Tools\R3CLI; dependencies.json de Show-Tree. Fuente se verificó localmente; no se copiaron dependencias al nuevo repositorio.


## Estado tras la revisión de arquitectura general

Este documento conserva investigación y candidatos técnicos, no impone detalles a todos los sprints. La referencia vigente es ../architecture/ARQUITECTURA.md. RV-008 coordina spikes por capacidad; APIs/versiones/límites/seguridad detallada se refinan con el trabajo dependiente. La revisión general no acredita runtime.

## Actualización técnica S03 C, 2026-10-05

Syntect5.3.0 default-syntaxes+regex-onig, Onig6.5.3/onig_sys69.9.3, sin temas ni YAML de usuario. Worker recibe snapshot/idioma integrado; fuente/caches/operaciones/tiempos bajo presupuesto y Job Windows committed512MiB. Catálogo real no incluye TOML/Nu/PowerShell, por lo que cobertura adicional permanece abierta. Revisión, AE funcional y fronteras en ../sprints/S03.md. Las licencias MIT de wrappers no sustituyen el COPYING de Oniguruma; avisos de esta incorporación en ../../third-party/README.md, sin cerrar revisión transitiva de distribución.

[Linux setrlimit](https://www.man7.org/linux/man-pages/man2/getrlimit.2.html) limita espacio virtual y mantiene límites entre fork/exec; no es RSS/commit. [XNU kern_resource](https://github.com/apple-oss-distributions/xnu/blob/xnu-12377.121.6/bsd/kern/kern_resource.c) delega RLIMIT_AS en vm_map_set_size_limit y rechaza un límite por debajo del mapa actual; regiones compartidas grandes en ARM64 impiden asumir512MiB absolutos. Candidato macOS requiere probe en host real y semántica declarada, no portar el número Windows por analogía. Gate Unix provisional se mantiene; crosscheck Linux/macOS no lo valida.

[Syntect5.3 regex.rs](https://docs.rs/syntect/5.3.0/src/syntect/parsing/regex.rs.html) convierte errores internos de búsqueda Onig en no-match. GrammarError cubre errores expuestos por parser/scope; no prometer detección de cada fallo interno. Investigación pendiente: corpus retry-limit antes del watchdog y hook de propagación controlado; Fancy no elimina por sí solo esta limitación. Observación documental, no defecto reproducido ni reducción del alcance.


2026-10-10: [fuentes publicadas MathJax/Mermaid contrastadas](MOTORES_FUENTES_PUBLICADAS.md) con revisión independiente de tarballs, selección, métricas y caminos de IO. MM-PATH01 confirmado en predicado sintético; frontera propia de recursos/red obligatoria antes integración. Investigación aceptada, motores/runtime/distribución/RV302/303/501 pendientes.

2026-10-10: [NewCM publicado y avisos embebidos](NEWCM_FUENTES_AVISOS.md) con revisión independiente:105 WOFF2 contienen remisión GUST en copyright, frente a declaración Apache del paquete; ambas identidades separadas. Referencias primarias conservadas, origen Git exacto404 no verificado. Investigación cerrada en alcance limitado, diseño/motores/cobertura/distribución pendientes.
