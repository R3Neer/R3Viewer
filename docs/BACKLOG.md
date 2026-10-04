# R3Viewer: backlog inicial

Fecha: 2026-10-04. Todos los elementos pendientes. Planificación autorizada; ejecución no autorizada. Dependencias y aceptación propuestas, revisables con respuestas nuevas.

## Cómo leerlo

Un producto, dos entregables: lector y puente Nushell. H0 contiene decisiones/investigaciones; H1–H5 son entregas futuras verificables. Terminar un hito no completa todo el producto. Un elemento condicionado no se implementa sin cerrar la decisión indicada. Publicar o instalar exige autorización de destino independiente.

Estados: pendiente, en análisis, listo, bloqueado por decisión, en implementación, en validación, terminado. «Terminado» requiere evidencia de sus criterios, incluidos QA visual cuando corresponda. Estado actual global: pendiente; los análisis documentales existentes son insumos, no prototipos probados.

## H0 — Contratos y viabilidad técnica

| ID | Trabajo | Dependencia | Aceptación |
|---|---|---|---|
| RV-001 | Cerrar entrada y CLI | Q03 | Ejemplos de ruta, texto, URL y pipeline inequívocos; combinaciones incompatibles y ausencia de entrada documentadas. |
| RV-002 | Consolidar alcance HTML | Q24/Q25 cerradas | Controles solo lectura y from html opcional incluidos; resolver política de recursos secundarios y cobertura restante sin ampliar a navegador completo. |
| RV-003 | Evaluar puente Nu y acceso al terminal | RV-001 | Comparar plugin y puente externo: tipos, TTY al recibir pipeline, señales y versiones; evidencia en Windows y plan de contraste macOS/Linux. |
| RV-004 | Seleccionar arquitectura y motores | RV-003 | Decisión argumentada por portabilidad, licencias, cobertura, composición y distribución completa; fallback y versiones definidos. |
| RV-005 | Definir modelo documento/datos/origen | RV-001/004 | Contratos de bloques e inline, tipos Nu, recursos, posiciones y bases; demostrar en diseño cómo preservan texto mixto HTML y fórmulas inline. |
| RV-006 | Fijar compatibilidad y criterios medibles | RV-004 | Versiones mínimas/arquitecturas propuestas, corpus representativo y métricas de primera vista, memoria, caché y cancelación; sin garantías inventadas. |
| RV-007 | Especificar errores, configuración y diseño | RV-001/005 | stdout/stderr, salida sin TTY, códigos, errores locales/fatales, tema Nu y roles R3CLI descritos con ejemplos. |

## H1 — Núcleo usable para texto y datos

| ID | Trabajo | Dependencia | Aceptación |
|---|---|---|---|
| RV-101 | Resolver entrada y seleccionar adaptador | RV-001/005/007 | Ruta y pipeline según contrato; errores de entrada claros; no reabrir datos transformados; formato explícito para ambigüedad. |
| RV-102 | Integración Nushell tipada | RV-003/101 | Tablas/listas/registros anidados y tipos acordados sobreviven al puente; null, ausencia y texto vacío distinguibles. |
| RV-103 | Presentación inline y redirección | RV-005/007/101 | Orden, ancho y saltos correctos; -i equivalente a --inline; redirección sin escapes, referencias de gráficos pendientes. |
| RV-104 | Sesión interactiva y ciclo terminal | RV-003/005/007/101 | Abrir, desplazarse, redimensionar, salir y cancelar; restaurar cursor/pantalla/entrada tras fallos; teclado suficiente. |
| RV-105 | Texto y código | RV-103/104 | Fuente preservada, resaltado legible, líneas largas navegables; no ejecución ni reforma del archivo. |
| RV-106 | Datos tipados y archivos de datos | RV-102/104/105 | Navegar registros/listas/tablas, buscar; archivo abre fuente y permite estructura; pipeline conserva su transformación. |
| RV-107 | Configuración y coherencia visual | RV-007/103/104 | Tema claro/oscuro, límites visuales y fallbacks; integración discreta con Nu, sin banners; R3CLI solo si evaluación lo justifica. |

Salida H1: lector de texto/datos con dos presentaciones y límites publicados. No se anuncia soporte Markdown gráfico completo.

## H2 — Gráficos, PDF y composición

| ID | Trabajo | Dependencia | Aceptación |
|---|---|---|---|
| RV-201 | Detectar capacidades y emitir imágenes | RV-004/006/103/104 | Imagen de referencia visible en terminales objetivo, proporción/transparencia y alternativa textual; override de detección documentado. |
| RV-202 | Recursos gráficos, caché y trabajos cancelables | RV-005/201 | Cache externa al origen, invalidación por contenido/opciones/motor, límites y limpieza; cancelar sin corromper sesión. |
| RV-203 | Composición mixta y ampliación | RV-104/201/202 | Texto e imagen mantienen orden; clipping/redibujado retiran imagen anterior; ampliar y volver a posición; fallback no oculta recurso. |
| RV-204 | PDF por página | RV-202/203 | Sin destino abre primera página; #page=N físico correcto; páginas lazy, fuera de rango claro; zoom y regreso; PDF escaneado no promete OCR. |
| RV-205 | SVG estático e imágenes | RV-201/202/203 | Formatos/versiones declarados; dimensiones y fuentes fallan de forma identificable; animación tiene política explícita, sin bucle inesperado. |

Salida H2: infraestructura gráfica y PDF navegable; QA visual obligatorio, independiente de tests de código.

## H3 — Markdown enriquecido y Obsidian

| ID | Trabajo | Dependencia | Aceptación |
|---|---|---|---|
| RV-301 | Markdown estructural al modelo común | RV-005/103/104 | Títulos, listas, tablas, citas, código y enlaces preservan orden; índice y destinos estables; no parser basado solo en regex. |
| RV-302 | Mermaid gráfico | RV-004/202/203/301 | Corpus del dialecto declarado renderiza; error conserva bloque y diagnóstico; proceso cancelable y resultado cacheado. |
| RV-303 | Matemáticas inline y bloque | RV-004/202/203/301 | Altura/alineación de fórmula inline junto al texto, bloques legibles; cobertura expresiones publicada; error local no corta documento. |
| RV-304 | Resolver bóveda, nombres y destinos | RV-005/301 | Rutas, nombres abreviados, encabezados/bloques y etiquetas; raíz detectable/expresa; candidatos ante ambigüedad; ausentes identificados. |
| RV-305 | Incrustaciones y callouts Obsidian | RV-203/204/301/304 | Nota, imagen y página PDF en su posición; callouts legibles; ciclos y profundidad limitada señalados; no modificación de bóveda. |
| RV-306 | Navegación, búsqueda e historial común | RV-104/301/304 | Enlaces entre adaptadores, índice, búsqueda y atrás/adelante con posición; fallo de destino conserva origen; PDF↔nota comprobado. |
| RV-307 | Validar corpus Clases | RV-302/303/305/306 | Unidad representativa con Mermaid/matemáticas/PDF abre, amplía y vuelve; inspección y fixtures respetan HEAD y cambios locales; sin reescribir unidades. |

Salida H3: caso de uso central de Clases completo dentro de cobertura publicada.

## H4 — HTML, remoto y visor general

| ID | Trabajo | Dependencia | Aceptación |
|---|---|---|---|
| RV-401 | Transporte remoto y origen | RV-002/005/101/202 | HTTP(S), redirecciones, MIME y codificación; límites/cancelación; URL final/base preservadas; recursos secundarios solo por acción definida. |
| RV-402 | HTML5 al documento visual | RV-002/005/301 | HTML real tolerante, texto mixto, títulos/listas/código/tablas/imágenes; ids, entidades y controles según alcance; CSS/JS sin promesa de ejecución. |
| RV-403 | Navegación web y recursos enlazados | RV-306/401/402 | Resolver relativo/base/fragmentos; enlazar a PDF/imagen/HTML por adaptador; historial preservado; inline identifica destinos y recursos no cargados. |
| RV-404 | from html opcional | RV-003/402 | Árbol con orden/atributos/texto, raw independiente; comprobar open HTML y http get text/html nativos en versión objetivo, sin sustituir comandos; activación opcional documentada. |
| RV-405 | Directorios y evaluación Show-Tree | RV-101/104/106/306 | Listado de nivel actual y apertura; errores permisos/enlaces; sin recursión obligatoria; datos Show-Tree/snapshot no se reemplazan mediante reescaneo. |
| RV-406 | Binarios desconocidos | RV-101/104 | Bytes y offsets, metadatos básicos, búsqueda/lectura limitada; no cargar archivo grande completo para primera vista. |

Salida H4: formatos restantes y navegación web estática dentro de límites. MathML, modo artículo y controles editables no incluidos automáticamente.

## H5 — Distribución, robustez y documentación

| ID | Trabajo | Dependencia | Aceptación |
|---|---|---|---|
| RV-501 | Empaquetado completo y licencias | RV-004/006/adaptadores incluidos | Instalación reproducible por sistema, versiones fijadas e inventario/licencias; completo por defecto; no descargar motores durante apertura. |
| RV-502 | Integración/install/uninstall Nu | RV-102/501 | Comando disponible tras instalación; configuración existente preservada; desinstalación retira solo artefactos propios; procedimientos revisables antes de ejecutarlos. |
| RV-503 | QA matriz y fallos | RV-006/H1–H4 | Ejecución y QA visual Windows Terminal y WezTerm por SO/arquitectura prometidos; redimensionado, salida, cancelación, fallback y dependencias ausentes. |
| RV-504 | Límites y rendimiento | RV-006/202/H1–H4 | Medir corpus, memoria/cache/primera vista; entrada sin fin cancelable; sin truncamiento silencioso; límites y resultados publicados. |
| RV-505 | Ayuda y documentación | RV-007/H1–H4 | Ejemplos archivos/datos/URLs, compatibilidad, cobertura Mermaid/math/HTML, errores y alternativas; distingue soporte completo y parcial. |
| RV-506 | Preparar entrega revisable | RV-501–505 | Checklist y evidencias por alcance, riesgos restantes explícitos; GitHub, release e instalación del usuario requieren autorización propia. |

## Preguntas emergentes y decisiones diferidas

| ID | Pregunta | Tratamiento |
|---|---|---|
| Q03 | Posicional=origen y pipeline=valor; nombres de flags | Confirmado: --path/-p y --value/-v incompatibles; RV-001 puede completar casos de validación. |
| Q24 | Checkboxes/controles HTML solo lectura o marcas temporales locales | Confirmado: solo lectura; no modificar ni enviar formularios. |
| Q25 | Incluir from html como utilidad opcional del puente Nu | Confirmado: incluir; activación opcional, sin sustituir open/http get. |
| T01 | Versiones/arquitecturas y motores | Investigación RV-003/004/006; preguntar solo si implica recortar cobertura o cambiar instalación. |
| T02 | Gráficos remotos en inline y carga agrupada | Referencias sin descarga por defecto; mecanismo explícito pendiente antes de RV-401. |
| T03 | PDF búsqueda/selección de texto y extracción | Separar render y extracción; búsqueda PDF necesita decidir cobertura en RV-204/306, sin prometer OCR. |
| T04 | Lectura en pantalla completa con stdin ocupado por pipeline | Riesgo de arquitectura crítico para RV-003; validar antes de elegir puente. |

## Orden y paralelismo futuros

H0 antes de implementación. H1 habilita H2 y Markdown textual de H3. HTML parser y from html comparten parser pero conservan modelos distintos. H4 directorios/binarios puede adelantarse tras H1; composición gráfica y caso Clases priorizan el objetivo central. Empaquetado/licencias deben estudiarse en H0 y concretarse durante hitos, aunque entrega final sea H5.

## Fuera del alcance actual

open --pretty, sustitución de explore/open, navegador CSS/JS completo, envío de formularios, gestor de archivos con escritura, monitor de streams infinitos, editor de datos, compilación LaTeX de documentos completos y OCR implícito. Publicación, creación del repositorio e implementación no autorizadas por este backlog.
