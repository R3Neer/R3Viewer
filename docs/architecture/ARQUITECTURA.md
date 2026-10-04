# R3Viewer: plan general de arquitectura y épicas

Fecha: 2026-10-04. Plan de organización general con revisión adversaria; el cierre se registra por separado. Alcance: organizar producto completo y trabajo Scrum; protocolos, APIs, versiones, límites y diseño de historias se refinan por sprint. MIT confirmada. No hay autorización de implementar/instalar/publicar. Esta organización general es la referencia para refinar por sprint; los ADRs de trabajo previos y la investigación se conservan como antecedentes técnicos revisables.

## 1. Objetivo y alcance

Un visor general que se integre visual y funcionalmente con Nushell, tanto para archivos/documentos como para valores tipados. Interactivo por defecto; --inline/-i imprime; redirección textual sin escapes. Argumentos=archivo/carpeta/URL, pipeline=valor, --path/-p y --value/-v incompatibles. Nu open y explore intactos; from html opcional. Markdown enriquecido, Obsidian amplio, PDF/imágenes/SVG, Mermaid y matemáticas gráficas, código/archivos de datos, valores Nu, carpetas/binarios y web HTML estático con controles solo lectura.

Distribución completa, Windows nativo más macOS/Linux, Windows Terminal y WezTerm según matriz verificable. Diseño discreto con roles R3CLI donde aporten coherencia. No introducir edición, OCR, navegador JS/CSS completo, formularios activos ni monitor infinito por inferencia.

## 2. Un producto y tres piezas entregables

1. **Núcleo lector y ejecutable**: contenido, navegación, presentaciones y coordinación; puede leer archivos sin necesitar Nu para interpretarlos.
2. **Integración Nushell**: comando público viewer, valores tipados y utilidad from html opt-in; companion privado cuando el transporte de tipos lo requiera.
3. **Distribución completa**: runtimes/motores gráficos privados, configuración e instalación por plataforma, avisos y compatibilidad.

Tres piezas de una misma versión/producto, no tres productos independientes. Bibliotecas y motores externos son dependencias, no nuevos productos del proyecto. El usuario invoca viewer, sin tener que elegir un motor por formato durante la lectura.

## 3. Dominios y responsabilidades

| Dominio | Responsabilidad | Frontera |
|---|---|---|
| Entrada e integración | Distinguir origen/valor, preservar tipos y metadatos, validar CLI | No dibuja ni vuelve a abrir origen de valores transformados. |
| Contenido y adaptadores | Interpretar formatos, conservar fuente/estructura y generar modelo | No descarga recursos ni ejecuta código como efecto lateral del parsing. |
| Recursos y resolución | Archivos/red, enlaces, raíces de bóveda, ambigüedad y permisos | Operaciones remotas explícitas y fuente en solo lectura. |
| Sesión y navegación | Estado del documento, búsqueda, índice, historial y retorno de posición | No depende de coordenadas de pantalla como única identidad del destino. |
| Composición y presentaciones | Mezclar texto/gráficos, layout y presentar TUI/inline/texto | Un modelo compartido, tres comportamientos de salida; no convertir datos tipados universalmente a Markdown. |
| Servicios de render | Generar imágenes/PDF/math/Mermaid y controlar tareas/caché | Trabajos largos fuera del ciclo UI; aislamiento de fallos no equivale a sandbox. |
| Plataforma y distribución | Terminal, configuración, runtimes, instalación y compatibilidad | Restaurar sesión; no saltarse redirección; no descargar motores al abrir. |

El núcleo contiene modelos comunes neutrales y contratos; no depende del protocolo Nu, de UI ni de motores específicos. App coordina dominios. Los adaptadores producen contenido, el resolutor proporciona recursos y el render produce representaciones gráficas para las presentaciones. No fijar aún número de crates/directorios por dominio.

Relación principal:

```text
Archivo/URL ────────┐
                   ├→ entrada → adaptadores/modelos → sesión → composición → TUI/inline/texto
Valor Nu → puente ─┘                 │                            ↑
                          resolución/recursos → servicios de render

Plataforma/config/distribución soportan el conjunto.
```

Nu integra valores antes de la presentación. from html comparte parser con adaptador HTML pero devuelve estructura, no documento pintado. R3CLI aporta roles/recursos y diagnósticos discretos; no controla el compositor de documentos ni altera tipos. Show-Tree se integra por contrato de datos si aporta valor, sin depender de su árbol dibujado ni escanear de nuevo snapshots.

## 4. Representaciones que debemos separar

- **Origen/fuente y procedencia**: archivo, URL final/base, literal o pipeline; nunca perder base de enlaces ni confundir valor con ruta.
- **Modelo documental**: títulos, texto, tablas, enlaces y recursos incrustados para Markdown/HTML/documentos.
- **Modelo de datos**: valores tipados, anidamiento y orden para Nu; fuente resaltada y estructura de archivos se conservan como vistas diferentes.
- **Recursos gráficos y páginas PDF**: generación bajo demanda, métricas y posiciones para componer y ampliar.
- **Estado de sesión**: búsqueda, destino y posición estables, independiente del tamaño del terminal.

Estos conceptos pueden compartir tipos internos donde tenga sentido; no obligan a cinco bases de datos o procesos. Cambiar ancho o protocolo gráfico no cambia interpretación del documento. Los errores locales dejan recurso/fuente y diagnóstico legible, sin abortar todo por defecto.

## 5. Decisiones y cuestiones que refinan los sprints

Base de trabajo: núcleo Rust, integración Nu separada del proceso de terminal; Ratatui/Crossterm y backend gráfico candidato; PDFium preferido para PDF; Mermaid oficial/MathJax para cobertura gráfica; parsers estructurales Markdown/HTML y resvg para SVG. Estas son hipótesis técnicas respaldadas por fuentes y sujetas a spikes, no versiones bloqueadas ni garantías actuales.

Riesgos de arquitectura que necesitan validación observable antes de apoyarse en ellos:

| Riesgo | Spikes/aceptación de alto nivel | Épica propietaria |
|---|---|---|
| Pipeline y teclado compiten por stdin; plugin lifecycle | Valor tipado pasa al lector, teclado funciona, cancelar devuelve control y siguiente invocación funciona; stdout del plugin no mezcla UI/protocolo. Si streaming falla, considerar entrada finita limitada antes de abrir sin perder tipos/cancelación. | E02 |
| Protocolos y composición de gráficos | Misma mezcla texto/imagen/fórmula inline funciona en vistas y terminales objetivo; scroll/resize/exit sin residuos; negociar sin consumir teclas. | E04/E01 |
| Motores/runtimes y fuentes | PDF página+texto y Mermaid/math funcionan con bundle privado, incluyendo errores/cancelación; browser no se inicia para texto. | E04/E05/E10 |
| Obsidian y procedencia de enlaces | Nota/heading/block/embed PDF, basename ambiguo y ciclo; volver a posición y no reabrir origen de valor transformado. | E06/E07 |
| Web/recursos y render no confiable | Cargar explícitamente, relativos/base correctos, secundario no autorizado permanece referencia; workers no ejecutan acciones del documento ni obtienen permisos por simple embed. | E08/E04 |
| Soporte multiplataforma y dependencias | Build/instalación/QA real por target y versiones, avisos/licencias y entorno sin runtimes globales. | E10 |

Planificar cada spike dentro de un sprint con pregunta concreta, criterio de decisión y resultado documentado. Su aprobación de diseño no acredita runtime. Si una hipótesis falla, revisar elección técnica y backlog antes de seguir usando esa dependencia. No exigir terminar todos los spikes ni todos los formatos para entregar una lectura textual útil.

Detalles reservados al refinamiento: API y protocolo de transporte, versionado, clases/tipos finales, formato caché y presupuestos, atajos y UX de errores, workers y permisos por SO, locks y arquitectura de paquetes. No convertirlos en requisitos del usuario sin evaluar impacto.

## 6. Épicas del Product Backlog

Épicas agrupan objetivos coherentes; no son sprints obligatorios ni deben implementarse enteras de una vez. RV-* ya existente conserva IDs; se mapea a épicas y se descompone después en historias con aceptación.

| Épica | Resultado para usuario/arquitectura | Cobertura backlog inicial |
|---|---|---|
| E01. Base de lectura y sesión | Entradas claras, dos vistas, restauración/cancelación, navegación estable, configuración discreta | RV-001, RV-005, RV-007, RV-101, RV-103, RV-104, RV-107, RV-306 |
| E02. Integración Nushell | viewer idiomático, valores tipados transformados y compatibilidad plugin; instalación integración | RV-003, RV-102, RV-502; referencia secundaria RV-008 |
| E03. Texto, código y datos | Fuente resaltada, tablas/registros/listas navegables, sin operaciones de cálculo/editor | RV-105, RV-106 |
| E04. Infraestructura gráfica y render | Gráficos en su posición, fallback, zoom, trabajos cancelables y caché | RV-004, RV-201, RV-202, RV-203, RV-205; referencia secundaria RV-008 |
| E05. PDF | Páginas físicas lazy, búsqueda sobre texto disponible y enlaces/navegación | RV-204 |
| E06. Markdown enriquecido | Lectura Markdown con Mermaid y matemáticas en línea/bloque | RV-301, RV-302, RV-303 |
| E07. Obsidian | Enlaces, encabezados/bloques, callouts, embeds, ambigüedad y ciclos | RV-304, RV-305, RV-307 |
| E08. HTML y recursos remotos | Página estática legible, navegación y carga explícita; from html opt-in | RV-002, RV-401, RV-402, RV-403, RV-404 |
| E09. Inspección general | Directorios y binarios desconocidos, integración de datos Show-Tree por evaluar | RV-405, RV-406 |
| E10. Calidad, distribución y compatibilidad | Distribución completa, plataforma publicada, documentación y evidencia de soporte | RV-006, RV-008, RV-501, RV-503, RV-504, RV-505, RV-506 |

RV-008 pertenece principalmente a E10 como coordinación de validación; sus spikes por riesgo sirven E02/E04/E05, sin volverse épica extra ni gate global que impida trabajar en texto. La tabla asigna las 39 tareas RV a una épica principal, con referencias secundarias explícitas. A cada historia se asigna un responsable de decisión y una épica principal; referencias secundarias no duplican trabajo. RV-306 base de navegación nace en E01 y se amplía al incorporar cada adaptador.

Dependencias por capacidad: E01 habilita lectura de archivos/texto/código de E03; E02 habilita valores tipados/pipeline de E03, sin bloquear lectura independiente de archivos. E04 habilita aspectos gráficos de E05/E06/E07/E08; el Markdown textual de E06 y HTML textual de E08 pueden avanzar con E01 antes de gráficos. E06 documental habilita E07; E01 navegación y E08 red permiten seguir destinos remotos de cualquier formato. E09 puede progresar con E01/E03. Los spikes bloquean solo las historias que dependen de su riesgo, no épicas completas por asociación. E10 es transversal desde el primer incremento, no fase final de calidad. Cada épica implementa ambos modos pertinentes y fallback, no una vista completa y otra indefinidamente pendiente.

## 7. Organización Scrum

Un Product Backlog para todo R3Viewer, un Product Goal orientado al lector general con integración Nu y soporte acordado. El usuario fija prioridades/producto; personas que hagan desarrollo refinan arquitectura y tareas. No inventar roles de Scrum Master ni composición de equipo por disponibilidad de agentes. Revisor adversario es práctica de revisión de diseño, no rol oficial Scrum ni sustituto de Sprint Review.

Antes de iniciar el primer sprint se acordarán las responsabilidades reales de Product Owner, Scrum Master y Developers, la duración del sprint, la Definition of Done y la forma de realizar los eventos. No se asignan personas ni se inventa un equipo en esta fase. Si se decidiera omitir elementos de Scrum, se describirá como adaptación inspirada en Scrum, sin presentarla como Scrum completo. No fijar ahora duración/fechas, capacidad o Sprint Backlog sin el equipo y prioridades. Propuesta de dirección, no calendario: primero un incremento pequeño que lea texto/valor Nu y salga correctamente; luego cortes verticales de gráficos/PDF y Markdown enriquecido; Obsidian/HTML/otros formatos se integran por prioridad con base suficiente. La distribución y QA empiezan pronto con lo implementado. Los hitos H0–H5 anteriores son agrupaciones de alcance, no sprints fijos ni fases waterfall.

Durante refinamiento: elegir resultados de usuario, dividir RV grandes en historias/spikes, documentar aceptación, dependencia y límites. Sprint Planning selecciona Sprint Goal y trabajo viable; ejecución produce un incremento usable del alcance seleccionado; Daily Scrum inspecciona el avance hacia el Sprint Goal y permite adaptar el plan diario; Sprint Review recoge feedback y ajusta backlog, Retrospective mejora forma de trabajar. El revisor adversario no sustituye responsabilidades ni eventos Scrum. No planear que cada sprint sea únicamente un componente horizontal completo. Un spike puede ir junto a una historia viable, sin presentar investigación sola como producto entregado.

Definition of Done inicial a acordar al iniciar desarrollo: comportamiento y límites documentados, criterios de historia comprobados, código/revisión pertinente, QA visual si cambia presentación, cambios seguros de lectura/cancelación y documentación coherente; solo anunciar plataforma si se ejecutó validación allí. Alcance no implementado sigue backlog. Checklist de refinamiento opcional para evitar historias ambiguas, no gate formal adicional a Scrum.

No estimar velocidad, fechas de entrega o story points sin información de equipo. El plan general describe fronteras y riesgos; el Sprint Backlog contendrá decisiones específicas y cómo producir el incremento.

## 8. Cierre de revisión y mantenimiento

Agente adversario independiente revisa cada Rn general, emite críticas temporales con IDs y evidencia; propuesta nueva responde y vuelve a revisión completa. Fin solo con informe explícito sin críticas accionables pendientes dentro de alcance. Plan final, referencias y breve registro del resultado quedan canónicos; propuestas/informes intermedios fuera Git. MIT en LICENSE; no instalación ni publicación ni código de producto por este encargo.

Finalización de este plan deja preparada la organización general. No cierra RV-003/004/006/008 runtime, no pretende arquitectura inmutable y permite revisar decisiones en sprints ante evidencia sin recortar requisitos silenciosamente.

## 9. Referencias de diseño

Consultadas el 2026-10-04. Fuentes primarias; evidencian posibilidades/reglas, no integraciones probadas ni versiones seleccionadas.

- [Scrum Guide](https://scrumguides.org/scrum-guide.html): backlog único, objetivos e incrementos; épicas son agrupación interna, no artefacto obligatorio de Scrum.
- [Plugins Nu](https://www.nushell.sh/contributor-book/plugins.html): canales plugin, compatibilidad y streaming; separar integración y presentación.
- [Metadata Nu](https://www.nushell.sh/book/metadata.html): procedencia y efectos de transformación.
- [Crossterm events](https://docs.rs/crossterm/0.29.0/crossterm/event/index.html): único propietario de lectura.
- [Ratatui](https://docs.rs/ratatui/latest/ratatui/) y [ratatui-image](https://github.com/ratatui/ratatui-image): base TUI y límites de gráficos, no composición documental completa.
- [PDFium renderer](https://github.com/ajrcarey/pdfium-render): páginas/texto y distribución nativa; no asumir threads seguro.
- [html5ever](https://github.com/servo/html5ever), [pulldown-cmark](https://docs.rs/pulldown-cmark/latest/pulldown_cmark/): parsing estructural separado de presentación.
- [Obsidian links](https://help.obsidian.md/links) y [embeds](https://help.obsidian.md/embeds): semántica de destinos y páginas PDF.
- [Mermaid CLI](https://github.com/mermaid-js/mermaid-cli), [Mermaid securityLevel](https://mermaid.js.org/config/schema-docs/config-properties-securitylevel.html), [secure config](https://mermaid.js.org/config/schema-docs/config-properties-secure.html): motor y configuración controlada; no equivalen a sandbox SO.
- [MathJax Node](https://docs.mathjax.org/en/latest/server/direct.html) y [resvg](https://github.com/linebender/resvg): render matemático y SVG; separar métricas/composición del motor.
- R3CLI/Show-Tree inspeccionados previamente en D:\Tools: recursos/diseño discreto y datos estructurados; integración concreta para refinamiento.

## Actualización de proceso

El encargo posterior autoriza implementación completa y sustituye restricciones históricas de solo análisis. El usuario confirma Scrum por objetivos sin cadencia temporal fija, adaptación explícita descrita en ../process/METODOLOGIA.md. Arquitectura transversal y revisiones iterativas independientes por incremento; aceptación y QA siguen obligatorias.
