# Viewer: aclaraciones de diseño y reutilización

Fecha: 2026-10-04. Análisis documental y lectura de repositorios; sin implementación, instalaciones ni pruebas de integración.

Relacionado: [requisitos](REQUISITOS.md), [decisiones](DECISIONES.md).

## Entrada en Nushell

Nushell no impone que una cadena de argumento y una cadena de tubería deban significar lo mismo. El contrato pertenece al comando:

- open interpreta como nombre de archivo tanto su argumento como la entrada de tubería: `'myfile.txt' | open`.
- get recibe una estructura por tubería y una ruta de campo por argumento: `ls | get name`.
- Los comandos personalizados separan parámetros de entrada de pipeline.

Fuentes: [open](https://www.nushell.sh/commands/docs/open.html), [get](https://www.nushell.sh/commands/docs/get.html), [comandos personalizados](https://www.nushell.sh/book/custom_commands.html).

Conclusión: Q03 A es un contrato razonable para un visor que combina archivos y valores, pero no una ley universal de Nushell. Evita abrir accidentalmente un archivo cuando se desea inspeccionar una cadena.

Propuesta de flags: --path/-p y --value/-v, mutuamente excluyentes. path incluye archivos y directorios; value incluye texto y datos estructurados. No usar --raw para esta distinción, porque texto fuente y entrada por ruta son conceptos diferentes. El formato de interpretación se indicaría aparte cuando la detección no sea suficiente.

Ejemplos de diseño, no comandos implementados:

```nu
viewer apuntes.md                      # archivo
viewer -i apuntes.md                   # archivo en vista normal
"apuntes.md" | viewer                  # cadena literal
"apuntes.md" | viewer --path           # ruta recibida por tubería
viewer --value "# Un título"           # contenido literal, no archivo
open config.yaml | viewer              # resultado estructurado de open
ls | where size > 1mb | viewer          # tabla transformada recibida
```

No admitir simultáneamente argumento de contenido y entrada de tubería sin definir una combinación explícita; propuesta: error por dos fuentes. --path requiere una sola ruta textual en la primera versión. Una lista de cadenas sin --path permanece una lista de datos.

Actualización: Q03 cerrada posteriormente con --path/-p y --value/-v. El estado técnico vigente está en CONTRATOS.md y ../architecture/ARQUITECTURA.md; este estudio conserva antecedentes de la investigación.

## URLs y páginas web

Q11 A exige acción explícita para recursos remotos, pero no determina soporte HTML. Distinguir:

1. Documento remoto conocido (PDF, imagen o Markdown).
2. Lector HTML estático: texto, enlaces, imágenes y tablas, con límites CSS/JavaScript.
3. Navegador completo: JavaScript, sesiones, formularios y aplicaciones dinámicas.

Propuesta Q23 A: lector estático, sin prometer sustituir el navegador. La apertura de una URL indicada directamente por el usuario cuenta como acción explícita. Mostrar una nota con enlaces remotos no inicia por sí solo esas cargas. La interfaz de URLs necesita diseñarse sin confundir ruta local, cadena y URL.

No se ha investigado ni elegido todavía un motor HTML. Esto amplía el alcance y no es una capacidad automática de Chafa, MuPDF o el compositor Markdown.

## Show-Tree

Lectura actual de D:\Tools\Show-Tree\README.md y src/nushell/show-tree.nu/show-tree-display.nu.

La salida pública es una fila por nodo con name, type, size (filesize), children y path; la jerarquía efectiva usa también metadatos privados. Los resultados filtrados no deben reconstruir entradas eliminadas. Los snapshots .showtree no vuelven a explorar el disco al abrirse.

Reutilización propuesta:

- Aceptar `show-tree carpeta | viewer` como entrada nativa de datos, con reconocimiento explícito del contrato de árbol cuando estén disponibles sus metadatos.
- Evaluar soporte de snapshots .showtree en el adaptador Nushell.
- Reutilizar reglas y APIs estables, no parsear el dibujo de caracteres ni copiar APIs internas como contrato permanente.

Límite importante: Show-Tree hace recorrido recursivo y calcula tamaños. Un visor de carpeta debería listar primero el nivel actual y cargar descendientes al abrirlos, sin bloquear cada navegación para calcular todo el árbol. Show-Tree puede aportar una vista enriquecida solicitada, pero no se fija como backend obligatorio del explorador.

Portabilidad completa de su instalación no acreditada aquí: su README declara requisitos Windows para la instalación de perfiles y ciertos wrappers. No trasladar esos requisitos automáticamente a Viewer.

## R3CLI: evaluación

El usuario solicita uso discreto, como Show-Tree, y coherencia con el ecosistema visual de Nushell. Esa preferencia queda confirmada; usar R3CLI es una recomendación de este análisis, no una integración ejecutada.

Evidencia actual:

- R3CLI es una capa de presentación semántica para Python, PowerShell y Nushell; no un motor de documentos ni de navegación.
- docs/design-language.md prescribe empezar por el resultado y evitar banners que repitan el nombre del comando. Tablas sin bordes ornamentales salvo utilidad real.
- El adaptador Nu tiene roles, configuración de color/ASCII/ancho y sink explícito. Sus operaciones imprimen presentación humana sin convertirla en datos estructurados de la tubería.
- El adaptador empaquetado no requiere Python ni PowerShell durante su ejecución.
- Show-Tree usa r3cli line con segmentos: carpeta/heading, prefijo/secondary, tamaño/value, archivo/accent. No necesita banner en la vista del árbol.
- Show-Tree importa una copia privada bajo vendor/R3CLI y fija revisión/hashes en dependencies.json. Esa copia declara 0.4.1; el checkout fuente R3CLI consultado incluye documentación de versiones posteriores. Elegir y comprobar revisión compatible, no asumir identidad.

### Recomendación de responsabilidades

| Superficie | Uso recomendado |
|---|---|
| Ayuda, diagnósticos y estado | Primitivas y catálogo R3CLI. Mensajes escasos; sin confirmación de éxito por cada apertura. |
| Árboles, rutas y metadatos inline | Roles discretos, como Show-Tree; coherencia de símbolos y ancho. |
| Tablas de valores Nushell | Conservar tipos, convenciones y configuración de Nushell cuando haya equivalencia; no sustituir su aspecto por tablas decorativas R3CLI de forma global. |
| Texto del documento | Motor propio: encabezados del autor, tablas Markdown, callouts y fórmulas no son encabezados de ayuda CLI. |
| Visor interactivo | Consumir semántica/paleta en su propio renderizador; no emitir prints R3CLI sin control dentro del bucle de pantalla. |
| Gráficos y layout | Compositor/visor y adaptadores gráficos; R3CLI no gestiona posiciones de imagen, recorte, zoom ni protocolos gráficos. |

### Contrato visual propuesto

- Documento o datos como protagonistas; sin banner R3Viewer al abrir normalmente.
- Fondo del terminal, texto normal y márgenes moderados; no imponer una estética de dashboard.
- Secundarios discretos; acento para enlaces/acciones, significado accesible sin color.
- Barra interactiva mínima: documento, ubicación/página y controles necesarios.
- Teclado coherente con usos de terminal; errores fuera de los datos redirigidos.
- Honrar NO_COLOR y política de Unicode/ASCII. La ausencia de color no debe borrar contenido.
- Configuración visual Nu como entrada cuando la integración la proporcione; mapa explícito de roles y fallback para colores complejos. No asumir que R3CLI importa automáticamente $env.config.color_config.
- Fuera de Nushell, utilizar defaults equivalentes sin depender de su configuración.
- No cambiar globalmente los hooks o estilos de Nushell para conseguir esta coherencia.

### Coste y límite

Ventaja: reutilizar lenguaje, ayuda y diagnósticos, distribuir el adaptador privado y evitar otro sistema visual paralelo. No limita el formato de los datos.

Riesgo: el motor interactivo o un núcleo en otro lenguaje no dispone automáticamente del adaptador R3CLI correspondiente. La arquitectura debe reutilizar recursos semánticos mediante un puente definido, evitando clonar manualmente la paleta o forzar un lenguaje por una biblioteca de presentación.

Decisión recomendada: R3CLI para la capa CLI y como base semántica de la interfaz, con adaptación discreta al tema de Nushell. No convertirlo en el motor de documentos/TUI.

Fuentes locales: D:\Tools\R3CLI\README.md, docs/design-language.md, docs/nushell.md, nushell/r3cli/mod.nu; D:\Tools\Show-Tree\src\nushell\show-tree-display.nu y dependencies.json. Lectura estática: no acredita el resultado visual de Viewer.

## Distribución completa y modularidad

Q19 B permanece elegida. Ventajas prácticas de una variante A: menor descarga para quien no usa motores pesados, actualización aislada de motores y evitar que un fallo en la instalación de Mermaid impida leer código. Coste: combinaciones extra que diagnosticar y probar, y capacidades ausentes al abrir documentos.

Compromiso recomendado, todavía propuesta: arquitectura modular interna y un instalador con perfil completo por defecto. En el futuro podrían añadirse perfiles ligeros instalables con una sola operación. Así se cumple B sin obligar al usuario a montar manualmente motores. No llamar «distribución completa» a un instalador que deja incompletas capacidades prometidas.

## Nombre de repositorio

Recomendación: **R3Viewer**, ruta futura D:\Tools\R3Viewer y comando viewer. Abarca archivos, documentos y valores sin limitarse a Markdown; encaja con R3CLI y la identidad de las herramientas del usuario.

Alternativas: TerminalViewer (más descriptivo y genérico) o NuViewer (vincula el nombre a Nushell incluso si el núcleo termina siendo reutilizable fuera de él).

No se ha creado el repositorio, instalado nada ni publicado en GitHub.
