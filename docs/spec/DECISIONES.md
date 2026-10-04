# Viewer: decisiones

Fecha de inicio: 2026-10-04. Fase: análisis inicial.

Relacionado: [requisitos](REQUISITOS.md), [preguntas](PREGUNTAS.md).

## Confirmadas por el usuario

| ID | Decisión | Procedencia |
|---|---|---|
| D01 | Mantener el trabajo en análisis; no implementar. | Peticiones reiteradas del usuario. |
| D02 | Priorizar portabilidad, incluyendo Windows. | Petición de solución multiplataforma. |
| D03 | Incluir los formatos de la investigación inicial. | «me gustan todos tus formatos». |
| D04 | Markdown enriquecido con imágenes, Mermaid y matemáticas gráficas. | Elección explícita de la opción compleja. |
| D05 | Incluir archivos y valores Nushell: tablas, listas, registros y texto. | Respuesta a la pregunta de alcance. |
| D06 | Guardar requisitos y decisiones en Markdown de este chat. | Solicitud directa de documentación persistente. |
| D07 | Presentar preguntas con ejemplos y resolverlas de una en una. | Solicitud directa del usuario. |
| D08 | Evaluar R3CLI con uso discreto, como Show-Tree, y coherencia con el lenguaje visual de Nushell. | Petición adicional explícita; integrar R3CLI sigue siendo recomendación técnica. |

## Respuestas del usuario a Q01–Q22

Registradas el 2026-10-04 en una única respuesta. A/B remiten a las alternativas históricas de PREGUNTAS.md.

| Pregunta | Respuesta | Decisión y matices |
|---|---|---|
| Q01 | A | Lector interactivo por defecto. Vista normal mediante --inline, también -i. |
| Q02 | B | Retirar todo open --pretty. open queda intacto; la visualización se concentra en viewer. |
| Q03 | A, cerrada | Confirmado posteriormente: argumento abre ruta local o URL HTTP(S); tubería contiene valor. --path/-p fuerza ruta local y --value/-v fuerza contenido literal; incompatibles. |
| Q04 | A | Salida redirigida textual legible, sin gráficos ni escapes y con referencias a recursos. |
| Q05 | A | Sin gráficos: lectura parcial identificada y alternativas textuales disponibles. |
| Q06 | A | Navegación completa con búsqueda, índice, enlaces e historial con posición conservada. |
| Q07 | A | Gráficos compactos, ampliables en la vista interactiva. |
| Q08 | A | PDF sin página: primera página y navegación bajo demanda. |
| Q09 | A | Obsidian amplio: notas, etiquetas, encabezados, bloques, callouts e incrustaciones; detección de ciclos. |
| Q10 | A | No elegir por aproximación entre destinos ambiguos. |
| Q11 | A | Recursos remotos solo mediante acción explícita. Pregunta adicional: abrir páginas web; cobertura HTML pendiente. |
| Q12 | A | Matemáticas gráficas también en línea, resolviendo altura y alineación. |
| Q13 | A | Continuar ante errores locales conservando original y diagnóstico. |
| Q14 | A | Archivo de datos: original resaltado por defecto, estructura opcional. Tubería: datos recibidos. |
| Q15 | A | Inspección, búsqueda y navegación; filtrado/cálculo en Nushell antes de viewer. |
| Q16 | B | Entradas finitas inicialmente; no monitor de flujos infinitos. Lectura incremental interna de entradas grandes sigue siendo posible. |
| Q17 | B | Un archivo por invocación. Una lista/tabla de entrada sigue siendo un valor estructurado, no varios archivos. |
| Q18 | A | Carpetas y binarios desconocidos con inspección básica. Investigar Show-Tree. |
| Q19 | B | Distribución completa. Abierto a reconsiderar A si la instalación modular es sencilla y sus ventajas convincentes; B vigente. |
| Q20 | A | Windows Terminal en Windows y WezTerm en Windows/macOS/Linux; otras por mejor esfuerzo. Versiones/arquitecturas pendientes. |
| Q21 | A | Detección y configuración sencillas, teclado completo y ratón opcional. |
| Q22 | A | Entregas verificables con límites publicados; un hito parcial no es producto completo. No autoriza implementar/publicar ahora. |

## Repositorio y publicación

- Destino futuro: repositorio Git bajo D:\Tools.
- Nombre aprobado el 2026-10-04: D:\Tools\R3Viewer; comando viewer.
- Intención futura: subir a GitHub. No autoriza publicación ahora.
- Repositorio creado en D:\Tools\R3Viewer el 2026-10-04 por autorización posterior; documentación canónica migrada y commits locales autorizados. Sin remoto ni publicación.

## Sustituciones

Q02 retira el contrato anterior de open -p/--pretty y su incompatibilidad con --raw. Esa validación deja de ser requisito. Los nuevos flags ruta/contenido sí deben excluirse mutuamente.

## Pendientes tras las respuestas

- Q03 cerrada; casos de error y combinaciones de entradas se detallan como propuestas en CONTRATOS.md.
- Q23: orientación confirmada hacia lectura de HTML con convenciones visuales Markdown, semejante a una página sin CSS. Alcance concreto de controles, recursos y extracción pendiente; véase WEB_HTML.md.
- Posible reconsideración de Q19; B sigue vigente.
- Reutilización Show-Tree/R3CLI y arquitectura/motores definitivos.
- Acceso al TTY, compatibilidad Nushell, límites de entradas finitas, versiones/arquitecturas y rendimiento.

## HTML y web: análisis del 2026-10-04

- El usuario propone leer páginas HTML con estilo Markdown: títulos, listas, checkboxes y contenido semántico. Se incorpora esta orientación, sin dar por aprobados todos los detalles de WEB_HTML.md.
- Q25 confirmada: incluir conversor opcional from html aprovechando open y http get nativos, sin wrapper de open; verificar comportamiento en versión objetivo.
- open de URLs sería una ampliación distinta y sigue sin aprobarse. Se recomienda mantener http get para transporte remoto.
- La petición mantiene la fase de análisis: no se implementa conversor, visor ni repositorio.
- Q24 confirmada: controles HTML solo muestran estado; no modificar casillas ni enviar formularios.

## Preparación autorizada y decisiones técnicas de trabajo

Encargo posterior autoriza migrar documentación, git init, política de commits y commits locales. Continúa sin autorizar implementar el producto ni instalar/publicar. README y CONTRIBUTING son entradas canónicas; las copias del chat quedan históricas.

ADRs registrados: núcleo Rust, encoder compañero Nu + módulo + proceso de terminal externo; DOM/documento/datos separados; Ratatui/Crossterm/ratatui-image con renderer inline específico; PDFium privado preferido a MuPDF; Mermaid oficial y MathJax workers; html5ever/pulldown-cmark/resvg/image/syntect como base de diseño. No son integraciones probadas ni versiones bloqueadas.

Revisión documental inicial cerrada en docs/research/INVESTIGACION_TECNICA.md. RV-008 valida las decisiones antes de ampliar desarrollo; versiones/arquitecturas y garantías quedan candidatas hasta QA. Licencia de proyecto pendiente de respuesta, MIT recomendada.