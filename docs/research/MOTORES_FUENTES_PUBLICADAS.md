# Fuentes publicadas de MathJax y Mermaid: investigación fijada

2026-10-10. [Revisión adversaria final](../reviews/Motores-publicados-fuentes-r2.md) sin críticas pendientes para esta investigación. No selecciona motores de distribución ni acredita su funcionamiento en R3Viewer.

Los tarballs oficiales @mathjax/src4.1.3 y @mermaid-js/mermaid-cli12.0.0 coinciden con SHA1 y digest SHA512 del metadata capturado. El adversario inventarió nuevamente todos sus archivos: 4287 y 23 archivos regulares, respectivamente, con tamaños y hashes completos; las 23 selecciones de fuente fueron comparadas contra archivo y captura. No se instaló o ejecutó ningún módulo. No se verificaron firmas o attestations npm; esta integridad acredita relación con el metadata registrado.

Las comparaciones con los snapshots de desarrollo anteriores son distintas: SVG de MathJax coincide en bytes; package.json de MathJax sólo coincide como objeto JSON; package.json de Mermaid coincide en bytes; index.js de Mermaid difiere. El index publicado, SHA-256 FABF76A7418EEBFF7591D67AE46541593AA582860160A9B735F2401C7DD6A992, coincide con la captura raw del gitHead npm db1ceebbe529d7975474eb0d0e9c23e9dc57cd37. No se infiere igualdad del repositorio o bundle completo ni pertenencia al árbol Git verificada. Una frase ambigua sobre «igualdad de cuatro archivos» se corrigió mediante suplemento R2, preservando la evidencia R1 intacta.

La fuente SVG publicada de MathJax usa w/h/d/pwidth y aplica transformaciones, clamps y alineación; createRoot/createSVG son protected. Su lectura sirve para refinar un adaptador de métricas, pero no garantiza API pública ni alineación con la fuente del terminal. Fuentes NewCM y transitivos, carga offline, SVG autosuficiente, raster, baseline y corpus matemático siguen pendientes. La [documentación oficial de componentes Node](https://docs.mathjax.org/en/latest/server/components.html) consultada se identifica4.0: es referencia sobre carga asíncrona y cierre de threads, sin atribuir ejecución acreditada al paquete4.1.3.

Mermaid publicado abre página, carga HTML/imports/CSS/fuentes y produce PNG mediante geometría del navegador; SVG puede contener foreignObject. page.close y el cierre condicional del browser en run no prueban reclamación del árbol de procesos. Los rangos de Puppeteer y otros paquetes tampoco fijan un runtime reproducible.

## MM-PATH01: frontera propia de IO necesaria

El interceptor publicado comprueba que relative(dir,filePath) sea no vacío y no absoluto; admite resultados '..' y rutas relativas a archivos fuera del directorio en la misma unidad. El adversario reprodujo nueve escenarios con node:path.win32/posix, rutas sintéticas absolutas y relative exacto, sin importar upstream o acceder a esos archivos. Esto confirma el defecto del predicado; no prueba explotación completa del flujo URL/request/browser ni exposición de R3Viewer.

Otros caminos de fuente continúan peticiones no interceptadas y usan fetch para fuentes/licencias. No se observó tráfico generado por el motor. Antes de integrarlo, el diseño deberá aportar una frontera propia de recursos y red, conjunto cerrado de archivos y supervisor físico; no se aceptará el interceptor como garantía de contención. El primer observador Windows de raíz usó escapes incorrectos y rutas relativas a unidad: queda rechazado en ese alcance, con resultados y errata anterior al intento corregido conservados.

Raíz leyó el dictamen y cotejó38/38 evidencias independientes. Evidencia privada R1: .tmp/root-math-mermaid-preparation-2026-10-10/published-source-capture-r1, manifest D8C4B2F3DA2BBE9FD5FA9E0927AAE088A9648A6B3DDF6461B8F1144C56B15329. Suplemento textual R2: published-source-findings-r2, manifest 2F7AD7A0A77BA5261DC1F66A115BC491828AAE0703117074ACED307F529A96FF. Revisión propia: .tmp/d4b-scheduler-adversary-2026-10-10/published-source-own-r1, manifest 0188818D52CA4AF4B0F1B8282C617FB60D8A2041E9E5414AC2447A84153DE27D; informe C18DB1FF9517CB3F12754250E022BFB7046B14030C57FED39A50232908B253B0.

El diseño, implementación, corpus M/D/X, cancelación y limpieza, TUI/inline, QA y validación de plataformas siguen pendientes. No cierra RV-302, RV-303 o RV-501 ni reduce su alcance.
