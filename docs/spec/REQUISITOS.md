# Viewer: requisitos y alcance

Fecha: 2026-10-04. Estado: análisis inicial, sin autorización de implementación.

Documentos relacionados: [decisiones](DECISIONES.md), [preguntas](PREGUNTAS.md) y [aclaraciones](ESTUDIO_ADICIONAL.md).

Actualización: Q01–Q22 respondidas y Q03 cerrada posteriormente. Q23 se orienta a lector HTML estático con convenciones Markdown, por aclaración posterior del usuario. Véanse [HTML/web](WEB_HTML.md), [contratos](CONTRATOS.md) y [backlog](BACKLOG.md). Las recomendaciones no se convierten en decisiones confirmadas al incluirlas en el backlog.

Este documento registra el estado de la conversación. «Confirmado» expresa una petición o respuesta del usuario; «propuesto» expresa una recomendación pendiente. Los motores mencionados son candidatos documentales, no una integración probada. Las decisiones posteriores sustituyen las propuestas incompatibles y se registran en DECISIONES.md.

## Alcance confirmado

- El usuario utiliza Nushell.
- La solución debe ser lo más multiplataforma posible, incluyendo Windows; se investiga soporte nativo en Windows, macOS y Linux.
- Se quieren archivos y valores de Nushell: tablas, listas, registros y texto.
- Se desean presentaciones en terminal normal y en un visor interactivo. La integración gráfica dentro del explore incorporado no está acreditada.
- Se incluyen los formatos investigados: Markdown, imágenes, PDF, código, JSON/YAML/TOML/XML y otros archivos de datos, SVG y Mermaid.
- Para Markdown se elige expresamente la variante compleja: imágenes, Mermaid y matemáticas gráficas dentro del documento.
- Compatibilidad Obsidian amplia: notas, encabezados, bloques, etiquetas de presentación, callouts e incrustaciones; destinos ambiguos no se eligen silenciosamente y se detectan ciclos.
- Lectura de HTML local y remoto con convenciones visuales Markdown, semejante a una página sin CSS; detalles en WEB_HTML.md.
- Controles HTML solo muestran estado: no se modifican ni se envían formularios.
- Se retira open --pretty. open y explore permanecen intactos; viewer concentra la visualización.
- Solo análisis y especificación. Se autoriza guardar y mantener estos Markdown del chat; no implementar, instalar ni modificar unidades.

## Organización propuesta del producto

Un producto de usuario, con dos entregables relacionados:

1. **Viewer**: motor de contenido y presentaciones normal e interactiva.
2. **Integración con Nushell**: entrada de valores tipados y comando viewer, sin adaptar open o explore.

Nombre de producto aprobado: R3Viewer. Comando: viewer. view ya es un grupo de comandos de Nushell.

Destino futuro confirmado: repositorio Git D:\Tools\R3Viewer. Publicación futura en GitHub como intención, no autorización actual.

Componentes internos:

- Entrada y modelo: distinguir archivos y valores, conservar estructura y procedencia.
- Adaptadores: interpretar cada formato y seleccionar su representación.
- Resolución: localizar recursos, enlaces, notas y páginas de PDF.
- Presentación normal: imprimir texto y gráficos, finalizar y dejar salida en el historial.
- Presentación interactiva: navegación, búsqueda, historial, ampliación y redibujado.

Los adaptadores y motores externos no son productos independientes nuestros. Se elige distribución completa; reconsiderar modularidad necesita nueva respuesta. Lenguaje, motores y protocolo de integración pendientes.

## Requisitos específicos confirmados por Q01–Q22

- Interactivo por defecto; --inline y -i seleccionan la vista normal.
- Un archivo por invocación. Las tablas/listas de una tubería son datos, no colecciones de archivos.
- Argumento abre ruta local o URL HTTP(S); tubería contiene valor. --path/-p fuerza ruta local y --value/-v fuerza contenido literal; mutuamente excluyentes.
- Redirección: texto legible sin gráficos ni escapes, con referencias a recursos.
- Sin soporte gráfico: lectura parcial identificada y alternativas textuales disponibles.
- Navegación completa; gráficos compactos y ampliables; PDF sin página comienza en la primera.
- Obsidian amplio, sin selección silenciosa entre destinos ambiguos.
- Recursos remotos mediante acción explícita; HTML como lector estático según aclaración de Q23.
- Matemáticas gráficas en línea y bloque; errores locales conservan original y diagnóstico y permiten continuar.
- Archivos de datos: original resaltado por defecto y estructura opcional. Valores Nushell: conservar la estructura recibida sin reabrir su origen.
- Datos: inspección, búsqueda y navegación; filtrado/cálculo en Nushell antes de viewer.
- Primera versión para entradas finitas, sin prometer monitor de flujos infinitos. Definir límites/cancelación sin pretender detectar siempre una entrada infinita.
- Carpetas y binarios desconocidos con inspección básica. Investigar Show-Tree, sin fijarlo como dependencia.
- Distribución completa B; variante modular solo como alternativa por reconsiderar.
- Garantizar Windows Terminal en Windows y WezTerm en Windows/macOS/Linux; versiones/arquitecturas pendientes.
- Configuración sencilla, teclado completo y ratón opcional.
- Entregas verificables; implementación/publicación no autorizadas ahora.
- Preferencia visual confirmada: discreción como Show-Tree e integración con lenguaje visual de Nushell. Evaluación R3CLI documentada en ESTUDIO_ADICIONAL.md; no imponer banners ni perder tipos por motivos de presentación.

## Requisitos funcionales generales propuestos

| ID | Requisito |
|---|---|
| F01 | Admitir archivos por argumentos y valores mediante tuberías. |
| F02 | Distinguir una ruta de archivo de un valor textual sin adivinación silenciosa. |
| F03 | Detectar una representación apropiada y permitir especificar el formato cuando sea necesario. |
| F04 | Ofrecer las dos presentaciones solicitadas. |
| F05 | Compartir interpretación y resolución entre ambas. |
| F06 | Conservar tipos y estructuras de los valores de Nushell; no recibir solamente su tabla visual. |
| F07 | Comunicar formatos no soportados, dependencias ausentes, enlaces ambiguos y errores de contenido. |
| F08 | Detectar capacidades gráficas y aplicar una política explícita de alternativa o error. |
| F09 | Permitir cancelar carga, renderizado y navegación. |
| F10 | Gestionar contenido grande sin exigir siempre su lectura o renderizado completo. |

## Requisitos no funcionales generales propuestos

| ID | Requisito |
|---|---|
| NF01 | Soporte nativo en Windows, macOS y Linux; Windows no dependerá de WSL. |
| NF02 | Matriz publicada de sistemas, versiones de Nushell, arquitecturas y terminales comprobados. |
| NF03 | Operación de lectura: preservar archivos y valores originales. |
| NF04 | Dependencias por capacidad; leer texto no debería exigir instalar un navegador. |
| NF05 | Límites de memoria y caché y cancelación durante tareas largas. |
| NF06 | Restaurar cursor, pantalla y entrada del terminal al salir o fallar. |
| NF07 | Manejo completo con teclado; soporte de ratón opcional. |
| NF08 | Comportamiento definido sin TTY, al redirigir salida y dentro de tuberías. |
| NF09 | Extensibilidad de formatos sin duplicar presentación y navegación. |
| NF10 | QA visual en los terminales objetivo; los checks de código no acreditan render ni navegación. |

No se han fijado métricas numéricas de rendimiento. Se definirán con documentos y datos representativos antes de convertirlas en garantías.

## Requisitos propuestos por presentación

### Normal

- Imprimir el documento y terminar; no activar una sesión de teclado de pantalla completa.
- Mantener el orden de texto, tablas, fórmulas, diagramas y recursos incrustados.
- Ajustar dimensiones al terminal respetando proporciones y legibilidad.
- No iniciar animaciones indefinidas o paginadores de forma inesperada.
- Conservar destinos identificables; los enlaces no equivalen automáticamente a navegación interna del lector.
- La política para salida redirigida se decide en PREGUNTAS.md.

### Interactiva

- Desplazamiento, búsqueda, selección y apertura de enlaces.
- Historial atrás/adelante, conservando la posición de lectura.
- Índice y saltos a encabezados.
- Navegación por páginas de PDF y ampliación de gráficos.
- Recomponer ante redimensionado; recortar y retirar gráficos al cambiar de vista.
- Salida y cancelación coherentes con la restauración del terminal.

## Requisitos propuestos por contenido

| Parte | Cobertura y límites |
|---|---|
| Markdown | Parser estructural; texto, tablas, código, enlaces y recursos gráficos en su posición. Callouts Obsidian con presentación propia. No asumir que Glow solo resuelve la composición gráfica. |
| Obsidian | Nombres abreviados, rutas, etiquetas de presentación, encabezados y bloques; distinguir enlaces e incrustaciones. Definir reglas para ambigüedad, archivos ausentes, alias y ciclos. Detectar bóveda o admitir raíz explícita. |
| PDF | Página física concreta, navegación y ampliación, render bajo demanda. Diferenciar visualización, extracción de texto y OCR. |
| Imágenes | Proporción, transparencia, dimensiones y ampliación; política explícita para animaciones. |
| SVG | Render estático con cobertura documentada; revisar fuentes y recursos externos. |
| Mermaid | Motor gráfico; límites del dialecto claros. Mantener el bloque original si falla. Alternativa textual opcional. |
| Matemáticas | Fórmulas en bloque y en línea; cobertura de expresiones matemáticas, no compilación de cualquier documento LaTeX. Resolver altura y alineación. |
| Código | Resaltado sin ejecución ni reformateo implícito; sangría, líneas largas y números de línea. |
| Archivos de datos | Diferenciar texto original y vista estructurada. No perder comentarios por una conversión implícita. |
| Valores Nushell | Tablas, registros y listas anidadas, escalares y tipos acordados; distinguir null, ausencia y cadena vacía. Mostrar resultados transformados de la tubería sin reabrir el archivo fuente. |
| Binarios desconocidos | Propuesta: bytes y metadatos básicos; no fingir comprender su formato. |
| Carpetas | Listado navegable de nivel actual y apertura de archivos; integración Show-Tree por evaluar, sin recursión completa obligatoria. |
| HTML/web | Adaptación semántica al compositor común, títulos, listas, enlaces, tablas, imágenes y estados de controles. CSS/JavaScript y envío de formularios fuera de la propuesta inicial. |

## Integración propuesta

Ejemplos de semántica, todavía no implementados:

```nu
viewer documento.md          # ruta de archivo
"documento.md" | viewer      # valor textual
open datos.json | viewer    # datos ya interpretados
ls | viewer                 # tabla tipada
viewer --inline documento.md # también viewer -i documento.md
```

La integración debe conservar tipos relevantes de Nushell. Una conversión universal a JSON requiere evaluar pérdidas de tipos. Un plugin nativo requiere estudiar compatibilidad de versiones y cómo el visor obtiene acceso al terminal.

open --pretty queda retirado por decisión Q02. No adaptar ni ocultar open o explore.

Conversor from html: utilidad opcional confirmada de la integración Nushell. Aprovecha la búsqueda nativa de conversores de open y http get; no introduce --pretty ni sustituye esos comandos. Su activación y comportamiento se verificarán en versión objetivo. Acceso URL mediante open no se recomienda; viewer acepta URL como origen explícito.

## Evidencia de la bóveda

Inspección de HEAD, sin usar cambios locales de unidades como fuente:

- 41 líneas con incrustaciones PDF #page= en las unidades versionadas inspeccionadas.
- Ejemplo: SO_T01.2_Nucleo_y_soporte_hardware.md incrusta páginas 25 y 27 de SO_T01_Introduccion_a_los_sistemas_operativos.pdf.
- IA_T01.2_Agentes_y_racionalidad.md incrusta la página 19 del PDF complementario de T01.
- Se observan encabezados [[#Apartado]], enlaces a notas con etiquetas, callouts, fórmulas y Mermaid.
- No había basenames duplicados entre los PDF versionados inspeccionados; esto no certifica otros archivos ni futuras versiones.

Criterio de aceptación propuesto: abrir una unidad de Clases, mostrar sus gráficos y página PDF incrustada, ampliarla y volver al mismo punto. Aún no ejecutado.

## Motores candidatos, no seleccionados

- Glow/Glamour: Markdown textual; requieren integración adicional para la composición enriquecida.
- bat: resaltado de código y texto de archivos de datos.
- Chafa: presentación de imágenes con Sixel, Kitty, iTerm2 o símbolos; distribución nativa multiplataforma.
- MuPDF/mutool: render de páginas PDF y extracción separada de texto.
- resvg: SVG estático a imagen.
- Mermaid CLI: motor oficial a PNG/SVG/PDF, con Node y Puppeteer.
- beautiful-mermaid: biblioteca para SVG y diagramas ASCII/Unicode, cobertura parcial.
- MathJax: expresiones matemáticas a SVG usando Node.
- termtex: alternativa matemática Unicode; Windows no acreditado en la investigación realizada.

## Fuentes investigadas

- [Nushell open](https://www.nushell.sh/commands/docs/open.html)
- [Nushell explore](https://www.nushell.sh/book/explore.html)
- [Nushell plugins](https://www.nushell.sh/book/plugins.html)
- [Obsidian enlaces](https://help.obsidian.md/links)
- [Obsidian incrustaciones](https://help.obsidian.md/embeds)
- [Chafa manual](https://hpjansson.org/chafa/man/) y [distribuciones](https://hpjansson.org/chafa/download/)
- [MuPDF draw](https://mupdf.readthedocs.io/en/latest/tools/mutool-draw.html)
- [Glow](https://github.com/charmbracelet/glow), [Glamour](https://github.com/charmbracelet/glamour), [bat](https://github.com/sharkdp/bat)
- [resvg](https://github.com/linebender/resvg)
- [Mermaid CLI](https://github.com/mermaid-js/mermaid-cli) y [beautiful-mermaid](https://github.com/lukilabs/beautiful-mermaid)
- [MathJax Node](https://docs.mathjax.org/en/latest/server/direct.html) y [termtex](https://github.com/doug/termtex)
