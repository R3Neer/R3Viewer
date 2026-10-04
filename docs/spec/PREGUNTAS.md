# Viewer: batería de preguntas

Fecha: 2026-10-04. Q01–Q22 respondidas por el usuario; Q03 tiene una condición por cerrar y Q11 abre Q23. Se conservan las alternativas originales como historial.

Relacionado: [requisitos](REQUISITOS.md), [decisiones](DECISIONES.md).

Se presentan en el chat de una en una, en este orden. Las recomendaciones son propuestas; no cuentan como respuesta del usuario. Se admite una respuesta propia o una combinación razonada. Los nombres de flags de los ejemplos son ilustrativos mientras no se apruebe la interfaz.

## Respuestas registradas

Q01 A (--inline/-i); Q02 B (retirar open --pretty); Q03 A condicionada; Q04–Q15 A; Q16 B (entradas finitas); Q17 B (un archivo); Q18 A (investigar Show-Tree); Q19 B (distribución completa, posible reconsideración); Q20–Q22 A.

El detalle y las sustituciones están en DECISIONES.md. El usuario respondió toda la batería por iniciativa propia; no volver a preguntar Q01–Q22 salvo sus aclaraciones pendientes.

## Q01. Qué ocurre al ejecutar viewer archivo

Ejemplo: `viewer apuntes.md`.

- **A. Interactivo por defecto (recomendación):** abre una pantalla para desplazarte, buscar, seguir enlaces y volver atrás. `viewer --inline apuntes.md` imprime y termina.
- **B. Normal por defecto:** imprime en el historial y termina. `viewer --interactive apuntes.md` abre el lector.

Impacto: qué acción cotidiana exige un flag. Las dos vistas seguirían existiendo en ambos casos.

## Q02. Conservar open --pretty

Ejemplo: si viewer ya muestra Markdown, ¿seguimos admitiendo `open -p apuntes.md`?

- **A. Sí:** integración opcional que usa el mismo motor; open sin -p sigue igual. Mantener incompatibilidad pretty/raw. Propuesta: open -p utiliza la vista normal.
- **B. No (recomendación por simplicidad):** viewer concentra la visualización y no oculta open.

Impacto: comodidad de dos entradas frente a instalación y mantenimiento de una adaptación del comando incorporado. La recomendación cambia respecto a la interfaz inicial al existir viewer; no invalida la petición anterior sin respuesta.

## Q03. Distinguir rutas y texto de tubería

Ejemplos: `viewer apuntes.md` frente a `"apuntes.md" | viewer`.

- **A. Argumentos como rutas, tubería como valores (recomendación):** el segundo ejemplo muestra literalmente apuntes.md. Abrir una ruta recibida por tubería requiere una opción explícita.
- **B. Reconocer rutas existentes en cadenas:** el segundo abre el archivo si existe, pero la misma cadena puede cambiar de significado según la carpeta.

Impacto: previsibilidad del contrato. Las tablas y listas siguen siendo valores, no colecciones de archivos por inferencia.

## Q04. Salida hacia otra herramienta o archivo

Ejemplo: `viewer --inline apuntes.md | save salida.txt`.

- **A. Texto legible sin gráficos ni escapes por defecto (recomendación):** conservar etiquetas y referencias de los gráficos en el texto. La exportación visual será una operación explícita futura.
- **B. Rechazar la redirección de contenido gráfico:** pedir un modo textual explícito para evitar confundir una vista visual con una exportación.

Impacto: automatización y expectativas. Un PDF extraído a texto no se presentará como copia visual fiel. El modo interactivo sin terminal debe dar un error explicativo.

## Q05. Terminal sin soporte de imágenes

Ejemplo: abrir una unidad con una diapositiva incrustada en una terminal que solo muestra texto.

- **A. Lectura parcial identificada (recomendación):** mostrar el texto y marcadores de imágenes; fórmulas/diagramas textuales cuando el motor los soporte. No perder recursos silenciosamente.
- **B. Exigir capacidades gráficas:** explicar la limitación y terminar, para garantizar siempre la presentación completa.

Impacto: alcance práctico de la portabilidad. El arte con caracteres no se considera equivalente a una diapositiva legible.

## Q06. Profundidad de la navegación interactiva

Ejemplo: desde una nota abrir otra, entrar en un PDF, ampliar y regresar.

- **A. Lector completo (recomendación):** atrás/adelante, posición conservada, índice, búsqueda y apertura de enlaces.
- **B. Primera versión con navegación básica:** desplazamiento y búsqueda; los documentos relacionados se abren en sesiones separadas.

Impacto: experiencia integrada y tamaño de la primera entrega; no decide el objetivo final de los gráficos.

## Q07. Tamaño de gráficos dentro de Markdown

Ejemplo: una diapositiva horizontal entre dos párrafos.

- **A. Vista compacta y ampliable (recomendación):** cabe en el ancho disponible y tiene un límite de altura; la vista interactiva permite ampliarla.
- **B. Priorizar tamaño y legibilidad inmediata:** ocupar tanto espacio como necesite el gráfico y desplazar para leerlo.

Impacto: equilibrio entre recorrido del texto y lectura del material gráfico. Se mantienen proporciones en ambos casos.

## Q08. PDF sin página especificada

Ejemplo: `viewer libro.pdf` o `![[libro.pdf]]`.

- **A. Primera página, con navegación en el lector (recomendación):** la incrustación comienza en la primera página; el enlace permite abrir el documento. No renderizar el libro entero por defecto.
- **B. Recordar última página en aperturas directas:** las incrustaciones sin página mantienen un inicio estable, pero abrir el PDF directamente recupera la última posición.

Impacto: reproducibilidad y persistencia de historial. Si existe #page=N, siempre se respeta N.

## Q09. Compatibilidad inicial de Obsidian

Ejemplos: `[[Nota]]`, `[[Nota#Apartado]]`, `![[PDF.pdf#page=25]]`, `![[Nota#^bloque]]`.

- **A. Cobertura amplia (recomendación):** notas, etiquetas, encabezados, bloques, imágenes, PDF, callouts e incrustaciones de notas, con detección de ciclos.
- **B. Priorizar lo observado en Clases:** notas, etiquetas, encabezados, callouts, imágenes y páginas PDF; bloques e incrustación de notas después.

Impacto: primera cobertura publicada y carga del parser/resolvedor. No significa reproducir todos los plugins de Obsidian.

## Q10. Destinos ambiguos

Ejemplo: dos carpetas contienen Tema1.md y la nota tiene `[[Tema1]]`.

- **A. No elegir por aproximación (recomendación):** mostrar candidatos en modo interactivo; en modo normal marcar la ambigüedad. Las rutas explícitas no son ambiguas.
- **B. Aplicar preferencias de cercanía:** preferir archivos próximos siguiendo reglas documentadas; avisar de ambigüedades restantes.

Impacto: facilidad de uso frente a riesgo de mostrar una fuente distinta. Las reglas deben contrastarse con Obsidian, no inventarse como equivalentes.

## Q11. Recursos de Internet

Ejemplo: `![Diagrama](https://ejemplo.org/imagen.png)` o un enlace a un PDF remoto.

- **A. Abrirlos solo mediante acción explícita (recomendación):** lectura local inicial; el lector permite cargar el recurso y explica errores de red.
- **B. Cargar automáticamente recursos visuales remotos:** comportamiento parecido a un navegador, con caché y límites definidos.

Impacto: funcionamiento sin conexión, tiempos de apertura y tráfico de red. No se refiere a la investigación web de este chat.

## Q12. Fórmulas gráficas en línea

Ejemplo: `La energía es $E=mc^2$ y aumenta con la masa.`

- **A. Gráficos también en línea (recomendación para la opción compleja):** conservar la relación con la frase y resolver altura/alineación.
- **B. Gráficos para bloques y Unicode para expresiones en línea:** integración más simple; mantener visible la diferencia de representación.

Impacto: fidelidad visual, densidad de lectura y complejidad del compositor.

## Q13. Error dentro de un bloque

Ejemplo: una fórmula o Mermaid incorrectos dentro de una nota de diez apartados.

- **A. Continuar y mostrar original + diagnóstico (recomendación):** el resto del documento sigue visible; el fallo queda señalado.
- **B. Detener la apertura:** exigir corregir el documento antes de mostrarlo.

Impacto: utilidad como lector y comportamiento de errores; se definirán códigos de salida sin confundir lectura parcial con éxito completo.

## Q14. Archivos de datos: texto o estructura

Ejemplo: `viewer config.yaml`, con comentarios, frente a `open config.yaml | viewer`.

- **A. Respetar la entrada (recomendación):** el archivo comienza como texto resaltado, con opción de vista estructurada; la tubería comienza como datos.
- **B. Estructura por defecto también al abrir el archivo:** comentarios y disposición originales quedan en una vista secundaria.

Impacto: lectura de configuración frente a exploración de datos. Cambiar vista no modifica el archivo.

## Q15. Funciones sobre tablas y registros

Ejemplo: `ls | viewer`, localizar archivos grandes y entrar en sus propiedades.

- **A. Inspección con búsqueda y navegación (recomendación inicial):** explorar anidamiento y contenido; filtros y cálculos se hacen en Nushell antes de viewer.
- **B. Añadir operaciones de consulta:** ordenar, filtrar y seleccionar dentro del visor, con posibilidad de devolver una selección mediante un modo explícito.

Impacto: cercanía a explore y tamaño del producto. No implica editar datos ni ejecutar comandos arbitrarios por defecto.

## Q16. Datos grandes o flujos sin final

Ejemplo: millones de filas o un comando que produce eventos continuamente.

- **A. Soporte incremental con límites visibles (recomendación):** mostrar mientras llegan, poder pausar/cancelar y comunicar límites de historial.
- **B. Solo entradas finitas en la primera versión:** no prometer un monitor en vivo; detectar o limitar la espera con claridad.

Impacto: memoria, búsquedas globales y tiempo hasta la primera pantalla. Nunca omitir filas sin indicarlo.

## Q17. Varios archivos

Ejemplo: `viewer uno.md dos.pdf tres.png`.

- **A. Una sesión con selector de documentos (recomendación):** cada archivo conserva su estado; en modo normal se imprimen separados y etiquetados.
- **B. Un archivo por invocación:** simplificar el contrato inicial y rechazar varios argumentos con explicación.

Impacto: comodidad al comparar y complejidad de navegación.

## Q18. Carpetas y formatos desconocidos

Ejemplo: `viewer carpeta` o `viewer archivo.bin`.

- **A. Inspección general básica (recomendación):** listado de carpeta con apertura de archivos y bytes/metadatos para binarios desconocidos.
- **B. Solo formatos y datos declarados:** error claro para carpetas o formatos no reconocidos.

Impacto: significado concreto de «visor de cosas»; no se pretende un gestor de archivos con operaciones de escritura.

## Q19. Dependencias e instalación

Ejemplo: quien solo necesita leer código no utiliza Mermaid ni PDF.

- **A. Base ligera y capacidades opcionales (recomendación):** instalación modular y diagnóstico de lo que falta para un formato.
- **B. Distribución completa:** instalar todos los motores para una experiencia inicial uniforme, a costa de tamaño y mantenimiento.

Impacto: facilidad de instalación frente a peso del conjunto. No habrá instalaciones automáticas al abrir un documento sin un contrato específico.

## Q20. Terminales garantizados

Ejemplo: Windows Terminal en Windows y WezTerm en los tres sistemas.

- **A. Garantizar varias terminales (recomendación):** Windows Terminal para Windows, WezTerm para Windows/macOS/Linux; otras por mejor esfuerzo y protocolo.
- **B. Garantizar inicialmente WezTerm en los tres:** una base común; ampliar la matriz después.

Impacto: alcance de QA y soporte. Se fijarán versiones y arquitecturas tras comprobar las herramientas, no por suposición.

## Q21. Configuración visual y controles

Ejemplo: fondo claro u oscuro, tamaño de gráficos y lector con teclado o ratón.

- **A. Detección con configuración sencilla (recomendación):** tema claro/oscuro, ancho, tamaño de gráficos, teclado completo y ratón opcional; valores previsibles.
- **B. Personalización amplia desde la primera versión:** estilos por elemento, perfiles de teclas y configuración detallada.

Impacto: coherencia y tamaño de la interfaz de configuración. Ninguna elección elimina el acceso por teclado.

## Q22. Primera entrega y objetivo final

Se mantiene como objetivo todo el alcance elegido. La pregunta es cómo dividir la implementación futura.

- **A. Entregas verificables (recomendación):** motor y entrada; vistas de datos/texto; gráficos/PDF; Markdown enriquecido y Obsidian; cada entrega publica sus límites y no se presenta como producto completo.
- **B. Primera entrega pública solo con toda la cobertura acordada:** hitos internos, sin publicar una versión parcial del producto.

Impacto: cuándo empieza a ser utilizable y qué se considera una versión terminada. Esta respuesta no autoriza implementar ni publicar ahora.

## Decisiones técnicas posteriores

Después de las respuestas se investigarán y propondrán: lenguaje, API del modelo, protocolo de entrada Nushell, acceso al TTY, empaquetado, motores definitivos, caché y métricas. No se pedirán elecciones técnicas al usuario sin explicar sus consecuencias prácticas. El nombre definitivo también podrá cambiar antes de fijar la CLI.

## Q23. Qué significa abrir una página web

Derivada de Q11. Cargar explícitamente una imagen/PDF remoto y representar HTML son capacidades diferentes.

- **A. Lector web estático (recomendación):** cargar HTML mediante acción explícita y representar texto, enlaces, imágenes y tablas; seguir enlaces. No garantizar JavaScript ni fidelidad CSS. Ejemplo: leer documentación o un artículo desde una URL.
- **B. Solo documentos/recursos remotos:** imágenes, PDF y Markdown desde URLs; enlaces a páginas HTML se abren en el navegador externo al seleccionarlos.
- **C. Navegador completo en terminal:** JavaScript, sesiones y páginas dinámicas mediante un motor de navegador. Mayor tamaño y complejidad; formularios y autenticación necesitan requisitos propios.

Ninguna opción carga recursos remotos al leer una nota sin acción explícita. Una captura de página no equivale a navegación web completa.

## Cierre posterior y nuevas preguntas del backlog

Q03 cerrada: argumento como ruta local o URL HTTP(S), pipeline como valor; --path/-p fuerza ruta local, --value/-v fuerza contenido literal; incompatibles. Q23 orientada a A por aclaración sobre HTML con estilo Markdown; detalles en WEB_HTML.md.

### Q24. Controles HTML

Ejemplo: una página contiene [x] Estudiar y un formulario de búsqueda.

- A. Solo mostrar estado (recomendación inicial): sin cambiar casillas ni enviar formularios.
- B. Marcas temporales locales: permitir cambiar casillas durante la sesión sin guardar ni enviar. Se debe indicar que son cambios solo de lectura local.

Respuesta confirmada: A, solo mostrar el estado, sin modificar ni enviar formularios.

### Q25. Conversor HTML para Nushell

Ejemplo: open pagina.html devuelve árbol HTML; open -r pagina.html muestra fuente. http get puede utilizar el mismo conversor según el tipo de contenido.

- A. Incluir from html como utilidad opcional de la integración (recomendación): comparte parser y no sustituye open/http get.
- B. Mantener HTML solo dentro de viewer inicialmente: inspección estructural por herramientas existentes o ampliación futura.

Respuesta confirmada: A, incluir from html como utilidad opcional. El árbol estructural conserva información que el lector no muestra.
