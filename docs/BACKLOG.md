# R3Viewer: Product Backlog

Fecha inicial: 2026-10-04. Implementación completa y commits locales autorizados; Scrum por objetivos sin cadencia fija. Los estados conservan aceptación real, sin cerrar QA ausente. Los estados precisos se registran al final. Dependencias y aceptación propuestas, revisables con respuestas nuevas.

## Cómo leerlo

Un producto: lector, integración Nushell y distribución conjunta. Las diez épicas y asignaciones principales de las 39 tareas están en [arquitectura general](architecture/ARQUITECTURA.md). H0 contiene decisiones/investigaciones; H1–H5 conservan agrupaciones históricas de alcance, no sprints ni fases obligatorias. Terminar un hito no completa todo el producto. Un elemento condicionado no se implementa sin cerrar la decisión indicada. Publicar o instalar exige autorización de destino independiente.

Estados: pendiente, en análisis, listo, bloqueado por decisión, en implementación, en validación, terminado. «Terminado» requiere evidencia de sus criterios, incluidos QA visual cuando corresponda. Estado actual: contratos/documentación inicial preparados; algunos elementos H0 pendientes de validación runtime. Véase tabla de estados al final. Los análisis documentales son insumos, no prototipos probados.

## H0 — Contratos y viabilidad técnica

| ID | Trabajo | Dependencia | Aceptación |
|---|---|---|---|
| RV-001 | Cerrar entrada y CLI | Q03 | Ejemplos de ruta, texto, URL y pipeline inequívocos; combinaciones incompatibles y ausencia de entrada documentadas. |
| RV-002 | Consolidar alcance HTML | Q24/Q25 cerradas | Controles solo lectura y from html opcional incluidos; resolver política de recursos secundarios y cobertura restante sin ampliar a navegador completo. |
| RV-003 | Evaluar puente Nu y acceso al terminal | RV-001 | Comparar plugin y puente externo: tipos, TTY al recibir pipeline, señales y versiones; evidencia en Windows y plan de contraste macOS/Linux. |
| RV-004 | Seleccionar arquitectura y motores | RV-001; RV-003 solo para puente Nu | Decisión argumentada por portabilidad, licencias, cobertura, composición y distribución completa; fallback y versiones definidos. |
| RV-005 | Definir modelo documento/datos/origen | RV-001/004 | Contratos de bloques e inline, tipos Nu, recursos, posiciones y bases; demostrar en diseño cómo preservan texto mixto HTML y fórmulas inline. |
| RV-006 | Fijar compatibilidad y criterios medibles | RV-004 | Versiones mínimas/arquitecturas propuestas, corpus representativo y métricas de primera vista, memoria, caché y cancelación; sin garantías inventadas. |
| RV-007 | Especificar errores, configuración y diseño | RV-001/005 | stdout/stderr, salida sin TTY, códigos, errores locales/fatales, tema Nu y roles R3CLI descritos con ejemplos. |

## H1 — Núcleo usable para texto y datos

| ID | Trabajo | Dependencia | Aceptación |
|---|---|---|---|
| RV-101 | Resolver entrada y seleccionar adaptador | RV-001/005/007 | Ruta y pipeline según contrato; errores de entrada claros; no reabrir datos transformados; formato explícito para ambigüedad. |
| RV-102 | Integración Nushell tipada | RV-003/101 | Tablas/listas/registros anidados y tipos acordados sobreviven al puente; null, ausencia y texto vacío distinguibles. |
| RV-103 | Presentación inline y redirección | RV-005/007/101 | Orden, ancho y saltos correctos; -i equivalente a --inline; redirección sin escapes, referencias de gráficos pendientes. |
| RV-104 | Sesión interactiva y ciclo terminal | RV-005/007/101; RV-003 para transporte Nu | Abrir, desplazarse, redimensionar, salir y cancelar; restaurar cursor/pantalla/entrada tras fallos; teclado suficiente. |
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

Ordenar el Product Backlog por valor y riesgo; las épicas no son sprints fijos. E01 permite texto/archivos; E02 habilita valores tipados de Nu sin bloquear lectura independiente. Gráficos y sus spikes bloquean solo historias dependientes. Markdown/HTML textual puede avanzar antes de gráficos. Directorios/binarios pueden adelantarse según prioridad. Calidad, documentación y distribución son transversales desde los primeros incrementos. Refinar historias, protocolos y tareas específicas durante preparación y planificación de cada sprint.

## Fuera del alcance actual

open --pretty, sustitución de explore/open, navegador CSS/JS completo, envío de formularios, gestor de archivos con escritura, monitor de streams infinitos, editor de datos, compilación LaTeX de documentos completos y OCR implícito. El encargo posterior autorizó crear el repositorio y commits locales. Este backlog no autoriza publicación; la implementación quedó autorizada después, según el apartado vigente.

## Actualización de preparación del repositorio

Repositorio y commits locales autorizados y documentación migrada. Fuente canónica docs/spec. Investigación de T01–T04 documentada en [investigación](research/INVESTIGACION_TECNICA.md) y [arquitectura](architecture/ARQUITECTURA.md); los puntos que necesitan ejecución siguen abiertos como gates, no como preguntas de producto sin investigar.

| Elemento | Estado a 2026-10-04 | Evidencia y siguiente paso |
|---|---|---|
| RV-001 | Terminado: especificación | Entrada, flags, validación de un origen y ausencia de entrada en CONTRATOS. Implementación RV-101. |
| RV-002 | Terminado: especificación | HTML estático, controles solo lectura, from html opt-in; recursos secundarios explícitos. |
| RV-003 | En validación técnica | Comparación de puentes y TTY cerrada documentalmente; requiere spike RV-008. |
| RV-004 | En validación técnica | ADR/motores definidos; bundle/build y licencias concretas pendientes RV-008/501. |
| RV-005 | Terminado: diseño inicial | Modelos y transporte etiquetado en ADR; detalles binarios/build se validan en RV-008/102. |
| RV-006 | En validación técnica | Matriz/corpus/límites candidatos definidos; versiones reales y métricas requieren ejecución. |
| RV-007 | Terminado: especificación | stdout/stderr, códigos, configuración y presentación documentados; validar propagación en Nu. |
| RV-008 | Pendiente; requiere implementación/prototipo autorizado | Gate de puente tipado/TTY/señales, gráficos TUI+inline, PDFium ABI y worker Mermaid/math frío. Depende RV-003/004/005/006; criterios en VALIDACION.md. |
| H1–H5 | Pendiente | No hay código ejecutable ni QA runtime. |

RV-008 coordina spikes por riesgo dentro de E10, con referencias a E02/E04/E05: no es bloqueo global de H1–H5. Cada validación antecede solo a las historias que dependen de ella; prototipos e implementación requieren encargo futuro. Los sprints buscan cortes verticales útiles, no completar toda una capa horizontal. No se altera el objetivo final ni se presentan garantías multiplataforma antes de medirlas. El backlog contiene 39 tareas únicas: 38 originales y el gate RV-008 añadido. El recuento anterior de 39 antes del gate era incorrecto.
## Organización Scrum histórica antes de la adaptación

Plan general revisado por adversario independiente: dos propuestas generales y dos informes temporales; cierre SIN_CRITICAS_PENDIENTES. Épicas E01–E10 y propietarios principales en architecture/ARQUITECTURA.md, única tabla canónica de asignación. Las referencias secundarias no duplican tareas.

Registro histórico de preparación, sustituido por la metodología por objetivos: antes del primer sprint se acordarían responsables reales de Product Owner, Scrum Master y Developers, duración, Definition of Done y realización de eventos. No hay Sprint Backlog, fechas, estimaciones ni capacidad comprometidos todavía. Planning selecciona resultado/objetivo; Daily inspecciona/adapta; Review recoge feedback del incremento; Retrospective mejora el proceso. El adversario revisa diseño, no sustituye eventos Scrum. La preparación actual no autoriza implementar el producto.

Los detalles de transporte, APIs, locks, límites y UX se refinan con cada historia. La revisión macro no cierra RV-003/004/006/008 runtime ni modifica las decisiones funcionales confirmadas.
## Autoridad y proceso vigentes

Implementación completa autorizada el 2026-10-04. Scrum por objetivos sin cadencia fija según process/METODOLOGIA.md. Estados anteriores conservan evidencia histórica; el corte activo se indica en el seguimiento posterior. No cerrar RV por una entrega parcial ni sustituir QA pendiente por documentación. El backlog sigue abierto hasta satisfacer toda su aceptación.

## Seguimiento de desarrollo por objetivos

| Corte | Estado | Aceptación/evidencia |
|---|---|---|
| S01-A entrada textual | Comprobado Windows | Archivo/literal/stdin finito; Unicode/vacíos/conflictos/límites. RV-101 parcial. |
| S01-B salida textual | Comprobado Windows | -i y redirección, controles visibles, canales, cancelación de escritura. RV-103 parcial. |
| S01-C sesión | En validación | ConPTY funcional, teclado por pipe y retorno; QA visual/matriz pendiente. RV-104 parcial. |
| S02 puente Nu | Comprobado funcionalmente Windows | 48pruebas, revisión integrada R1 sin críticas, AE y ConPTY Ctrl-C/reset/máscaras. QA visual/matriz y distribución pendientes. RV-102 parcial; sprints/S02.md. |

S01 código revisado iterativamente hasta R2 SIN_CRITICAS_PENDIENTES; aceptación independiente y15pruebas. Evidencia y límites en sprints/S01.md. RV-003/004/006/008 y H1–H5 conservan pendientes completos; no borrar tareas ni marcar RV terminado por subhistorias comprobadas.


Registro histórico R2/R3, sustituido por el estado R4 posterior: S03 activo con A/B y configuración en validación; resaltado aislado en integración. R2A/B pasa86pruebas y revisión adversaria; aceptación independiente detecta AltGr/Alt-code Windows y override NO_COLOR. R3 se corrige y vuelve a revisión/AE. RV-105/106/107 siguen abiertos, incluyendo estructura de archivos de datos, paleta Nu condicionada y QA visual/matriz. No se cancela el objetivo por terminar un corte.


Registro histórico A/B R4, ampliado por C R1 posterior: aceptación funcional Windows y revisión sin críticas; entonces quedaban resaltado C04–07, estructuras de archivos de datos y QA visual/matriz. Las correcciones de las revisiones R1–R3 se conservan como evidencia en sprints/S03.md. Este resultado parcial no cierra RV-105/106/107 ni el objetivo S03 completo.

## Estado vigente tras C R1

S03 C04–07 implementado/revisado sin críticas con AE funcional parcial Windows;114pruebas del workspace, CLI/Nu/ConPTY y fallbacks reales WorkerExit/RequestTimeout/SpanBudget. Linux x86_64 y macOS arm64 tienen crosscheck del lector, sin ejecución nativa ni contención sintáctica Unix. Evidencias y pendientes en sprints/S03.md y reviews/S03-sintaxis-aceptacion-r1.md. RV105/106/107 siguen abiertos: estructuras de JSON/YAML/TOML/XML, cobertura de gramáticas (defaults no TOML/Nu/PowerShell), paleta Nu condicionada, criterios de estrés/presupuestos independientes y QA visual/matriz. No cerrar sprints/tareas por esta aceptación parcial; el siguiente refinamiento trabaja estructuras de archivos conservando fuente original.


Corte siguiente en refinamiento: [S03 D1 TOML](sprints/S03-D1.md), plan revisado sin críticas pendientes. Spike de parser/gramática y diseño técnico son gates; sin implementación/aceptación acreditada todavía. RV-106 sigue abierto con los cuatro formatos JSON/YAML/TOML/XML.


Actualización D1: diseño core y recurso TOML-R4 aprobados por adversario, implementación A/B iniciada. Gate de viabilidad del índice físico/profundidad cerrado por15pruebas independientes; aceptación y candidato de producto pendientes. Plan/recurso aprobados no cierran RV-106 ni QA.


Refinamiento paralelo preparado: [delta JSON D2](sprints/S03-D2-JSON.md) y [revisión](reviews/S03-D2-plan.md) sin críticas pendientes del contrato observable. Elección de parser y candidato de producto siguen abiertos; TOML D1 permanece en implementación. [Inspección de API gráfica](research/GRAFICOS_API_PREPARACION.md) identifica adaptación de stdin/teclado, geometría y ausencia de efectos de configuración; no es soporte gráfico implementado ni probado visualmente.


Estado D1 TOML R1: candidato integrado en validación,145tests/build/fmt/Clippy y crosschecks Linux/macOS pasan. Adversario y AE independientes ejecutan el mismo snapshot; el ciclo de vida8casos del autor es evidencia separada. Ningún cierre de D1/S03/RV106 por estas pruebas; visual, contención Unix, resto de formatos y distribución siguen pendientes.


Estado D1 actualizado: revisiónR1 detectó dosP2 (presupuesto previo agregado e IPC trasSourceEnd), corregidos en candidatoR2 con147tests/build/Clippy. Revisión y AE del delta pendientes. AE independienteR1:69CLI/Nu+26semánticas y controles/cancel/reintento ConPTY pasan; límites exhaustivos y visual/matriz permanecen. Véase S03-D1.md. No cerrar RV106 ni eliminar JSON/YAML/XML ni los otros formatos del backlog.

D1R2 integrado y revisado sin críticas, [cierre de candidato](reviews/S03-TOML-producto.md). AEfuncional69+26+2casos y147tests pasan. TOML mantiene pendientes exhaustivos/visual/matriz; RV106 sigue abierto con JSON/YAML/XML. JSON detalle técnico en revisión sobre este núcleo, no producciónJSON todavía.


## Estado vigente JSON D2 R2 y refinamiento YAML

JSON integrado sobre TOML,159tests workspace/fmt/Clippy/build PASS. R1 revisiónadversaria sincríticas pero AE encontró fallo de identidad glifo/ESC; R2 corregido, revisión cerrada y aceptación delta funcionalWindows PASS con trazabilidad de corpusR1. [Cierre del candidato](reviews/S03-JSON-producto.md). No hay aceptación visual de algunas filas UnicodeConPTY (decoder/presentación indeterminado), ni QA/contenciónUnix/presupuestos exhaustivos. RV-106 sigue en implementación/validación parcial; YAML/XML y resto deaceptación no se eliminan.

[YAML D3](sprints/S03-D3-YAML.md) contrato observable cerrado en dosrondas adversarias; [gate experimental](research/S03_YAML_PARSER_SPIKE.md) revisado sin críticas; detalle de integración revisado e implementación iniciada. [XML D4](sprints/S03-D4-XML.md) tiene contrato observable revisado y spike preliminar, sin gate técnico ni producto. El corte activoS03 continúa; no abrir otro sprint ni completarproducto por este resultado.


XML D4 tiene [detalle técnico del núcleo](sprints/S03-D4-core.md) revisado independientemente sin críticas pendientes; [trazabilidad](reviews/S03-D4-core.md). G1–G7 siguen abiertos: no hay implementación XML ni aceptación del parser/producto. YAML D3 está en revisión de candidato R3 tras corregir identidad de controles, unidades de metadatos y proyección repetida en búsqueda hacia atrás; aceptación y cierre de integración pendientes. RV-106 sigue abierto.


YAML D3 R3 integrado y revisado sin críticas con aceptación funcional parcial Windows:188tests y95spans comprobados. [Evidencia](reviews/S03-YAML-producto.md). Límites máximos/lifecycle/visual/runtimeUnix pendientes; XML en gates experimentales. No completar RV-106 ni S03 ni retirar tareas por este corte.


YAML Y08 amplía aceptación independiente con cinco casos lifecycle,depth64/65 yfuente128MiB±1; [límites exactos de evidencia](reviews/S03-YAML-lifecycle-y08.md). Timeout controlado/exclusión sintaxis/Job512MiB/arena64MiB/items1M siguen pendientes, ademásvisual/runtimeUnix. XML G1–G3 ydelimitaciónG4 pasan experimentos yrevisión, pero G4 mantiene observaciónOS archivos/red exigida pendiente; no portarlo ni cambiar ese criterio para aprobar.


Refinamiento gráfico PNG: [contrato R2](refinement/GRAFICOS-PNG.md) y [revisión final](reviews/GRAFICOS-contrato-r2.md) sin críticas pendientes, G0 cerrado. G1–G7 y producción permanecen abiertos; no es nuevo sprint ni cierre de RV-201/202/203/205. S03 continúa activo.


XML G4 observación de recursos: [evidencia NT acotada](research/S03_XML_OBSERVACION_RECURSOS.md) y [cierre independiente](reviews/S03-XML-gate-recursos.md) cumplen el criterio original para prototipo exacto. G1–G4 experimentales cerrados, integración núcleo A iniciada; G5–G7 y aceptación producto pendientes. No es ETW/kernel ni garantiza ausencia universal de I/O. RV-106 y S03 continúan abiertos.


YAML Y09: [aceptación deadline/exclusión/Job](reviews/S03-YAML-lifecycle-y09.md) cierra los casos pendientes de timeoutTUI y exclusión real en dos transiciones bajo fallo. Job observado por handle propio confirma configuración512MiB por proceso; aplicación por estrés/arena total64MiB/items1M/visual/runtimeUnix siguen pendientes. No cerrar RV-106 ni S03.


XML [vista B R2](sprints/S03-D4-vista.md) revisada independientemente sin críticas; [informe](reviews/S03-D4-vista.md) fija unidades buscables por tipo y conserva SourceSafeText. Implementación A/B en paralelo con ownership coordinado; G5/G6/G7/candidato/AE aún pendientes. No cerrar RV-106/S03.


Gráficos [experimento de backends](research/GRAFICOS_BACKENDS_SPIKE.md) y [revisión parcial](reviews/GRAFICOS-backends-spike.md) sin críticas pendientes: cuatro constructores sin Picker, PNG/cabecera/alpha y cancelación propios. Kitty16M excede presupuesto; cuotas preencode/snapshot/inline/visual/cache yG1–G7 permanecen abiertos según evidencia. No adopción de APIs ni nuevo sprint; RV-201/202/203/205 intactos.


XML D4 R1 tiene [revisión y aceptación funcional parcial Windows](reviews/S03-XML-producto.md): 213 tests/fmt/Clippy/build, revisión independiente sin críticas, 89 CLI/Nu, 28 spans, ocho sesiones ConPTY y StringStream SDK real. Fuente original/visible y cancelación Unicode YAML también revisadas. Nueva observación G4 de producción y addendum independiente sin críticas incorporados, dentro del alcance NT observado. Estrés exhaustivo, inline deadline real, visual/Unix/gramáticas/distribución y formatos enriquecidos siguen abiertos. No cerrar RV-105/106/107 ni S03 por este incremento.


Gráficos: [propuesta G2 R2](refinement/GRAFICOS-G2.md) revisada sin críticas de diseño tras dos rondas. Experimento privado de raster visible/encoders acotados iniciado; implementación/producción, G1–G7 y RV-201/202/203/205 continúan pendientes. No nuevo sprint ni reducción del alcance final de imágenes/animación.


XML R1: [addendum inline](reviews/S03-XML-inline-r1.md) aporta timeout del controlador real bajo suspensión propia y Fuente completa, exit0 y reap/residuos correctos, con inspección independiente. Tiempo final observado incluye escritura/drenaje; medición aislada, carreras adicionales/estrés y visual/Unix siguen pendientes. No cierre de RV ni modificación de código por esta evidencia.


Contención [Unix R2](refinement/CONTENCION-UNIX.md) revisada sin críticas de diseño: supervisor macOS externo y nacimiento/configuración cgroup delimitados, AS distinto de committed/RSS. Probes privados preparados antes de adopción; ejecución Linux/macOS pendiente y fallback transitorio no satisface el objetivo final. No habilitación de producción ni cierre de S03/matriz/RV por esta revisión.

Gramáticas shell: [diseño R2](refinement/GRAMATICAS-SHELL.md) revisado sin críticas pendientes tras resolver retokenización y regex incompatibles; inventario exacto y review conservados. Prototipo privado y oracle independiente en preparación. No soporte Nu/PowerShell acreditado todavía ni cambios de producción; RV-105/106/107 y S03 continúan abiertos.

Gráficos: [raster por filas R2](research/GRAFICOS_FILAS_SPIKE.md) revisado independientemente sin críticas dentro del corte, tras corregir tres errores PNG de R1. 204 invocaciones de autor y250 del adversario (no corpus único sumado); AE R2 independiente completa171casos y cuatro Jobs Windows, dentro del corte. PNG16M→viewport evita RGBA completo; Sixel/IPC/cache/agregado/terminal/inline/visual/nativos siguen abiertos, sin adopción ni cierre de RV/G2/S03.

PDF: [contrato G0 R2](refinement/PDF-G0.md) revisado sin críticas de diseño; gate binario/ABI/corpus, implementación y aceptación siguen pendientes. No cerrar RV-204 ni demás capacidades. Unix: [harness R4](research/S03_UNIX_PROBES.md) revisado sin críticas en preparación, corrige UP-04; no ejecución nativa. Solo Windows disponible confirmado por el usuario; futura matriz GitHub Actions pendiente. No cierre U-G1–5/S03 ni soporte Linux/macOS por este resultado.


Sixel: [diseño experimental G2](refinement/GRAFICOS-SIXEL-G2.md) revisado sin críticas pendientes. Cuerpo acotado RGB332, dos pases y validador independiente por implementar/probar; no soporte de terminal ni cierre de G2/RV por revisión de diseño.

Gramáticas shell: [prototipo R2](research/S03_SHELL_GRAMMAR_SPIKE.md) con revisión y AE independientes sin críticas acotadas,269intervalos+16estados finales por motor. [Integración propuesta](refinement/GRAMATICAS-SHELL-INTEGRACION.md) revisada; implementación/candidato worker real por completar. No aceptación visual, runtimeUnix ni cierre RV/S03 desde runner privado.

PDF G1: [diseño de carga/ABI Windows](refinement/PDF-G1.md) revisado tras corregir flags a0x900. Candidato ctypes privado ejecutado por autor; revisión y AE independientes pendientes. No adopción del wrapper ni PDF de producto; mantiene RV204/matriz abiertos.

PDF: [candidato privado G1 R2](research/PDF_G1_SPIKE.md) con revisión de código y AE independientes sin críticas en ABI/carga/fixtures pequeñas/cleanup. AE R1 encontró orphan en fallo de setup, corregido y controlado en R2; evidencia previa preservada. Sin adopción Rust/wrapper ni producto/QA; nativecallblocked/UTF16exhaustivo/budgets/Unix y RV204/G1 completo permanecen abiertos.


## Integración shell vigente

Gramáticas Nushell y PowerShell integradas con revisión de código y aceptación independientes sin críticas pendientes en el alcance funcional Windows: 215 pruebas workspace (cuatro ignores separados), 269 intervalos y 16 estados finales sobre 84 documentos, controlador real con Job y límites, 18 casos CLI/Nu y ConPTY funcional. Windows usa Onig; fancy-regex construye el pack. Auto .nu/.ps1 y alias explícito pwsh; .pwsh continúa texto plano. [Entrega y evidencia](research/S03_SHELL_INTEGRACION.md). QA visual, contención/runtime Unix y cierre RV-105/106/107/S03 permanecen pendientes.


Sixel: [candidato privado R2 final](research/GRAFICOS_SIXEL_SPIKE.md) con revisión independiente y AE240invocaciones sobre oráculos previos. R1 tenía dos fallos de ownership/captura del helper, corregidos con encoder intacto; snapshots/controles negativos conservados. Sin críticas pendientes en el gate probado. Snapshot/IPC/agregado/cache/framing/terminal/inline/visual/Unix y G2 completo permanecen abiertos, sin adopción ni cierre RV/S03.

S03-UNICODE01 abierto: [reproducción privada de cancelación](research/S03_UNICODE_CANCELACION.md) detecta demora en un único grafema de32MiB. Fuente intacta; no corrección ni límite nuevo aprobado. Refinamiento independiente de segmentación/ancho/render cancelables en curso; conservar original y revisar todas las rutas antes de cerrar RV-105/504.


Gráficos: [diseño IPC R3](refinement/GRAFICOS-IPC.md) revisado sin críticas pendientes tras cerrar ownership externo/duplicación segura y receiptIO final antes de Ready. Wire/subreservas/gate privado e integración por implementar; no cierre de cache/terminal/visual/G2/RV/S03.

PDF Rust privado R1: implementación entregada para revisión y aceptación independientes; PRUST-ROOT01 detectado y reproducido por el coordinador. Una excepción en `child.poll()` dentro del finally del supervisor omite kill/wait/joins/cierres y deja vivo el hijo retenido; el ensayo propio requirió rescate. Candidato congelado SHA256 1E4195C719DECF9F37C7D630F2C9A31A526034327A39FF1008BF67B3D76D387C conservado como rechazado; corrección R2 separada en preparación. Las observaciones de cobertura nativa (astrales y texto truncado por PDFium) mantienen la extracción semántica sin acreditar; no cerrar gate Rust, G1 ni RV-204. Informe intermedio y controles propios fuera de Git, conforme a metodología.
S03-UNICODE01: [diseño de grafemas R2](refinement/S03-GRAFEMAS.md) y matriz previa de20escenarios revisados [sin críticas pendientes de diseño](reviews/S03-grafemas-plan-r2.md). Incluye panes YAML/XML prestados y procedencia por dominio, snapshots reales del cursor, trabajo por consulta/frame y admisión antes de Ratatui. Fuente/búsqueda/inline completas; cuotas de dibujo candidatas no reducen entrada. Viabilidad del adaptador de anchura, implementación, pruebas/AE y QA siguen pendientes; el bug continúa abierto y S03/RV-105/504 no se cierran.
Markdown: [gate de parser/procedencia](refinement/MARKDOWN-PARSER-GATE.md) revisado [sin críticas de diseño](reviews/Markdown-parser-gate-plan-r1.md); pulldown-cmark0.13.4 candidato, events/offsets y perfiles de extensiones explícitos. Paquete privado preparado y compilado con14dependencias fijadas, fuentes/licencias congeladas antes del build; fixtures/oráculos, ejecución y aceptación independiente pendientes. Sin cambio de producción ni nuevo sprint; no acredita composición gráfica, resolución Obsidian ni RV-301/302/303/305. La elección Markdown compleja completa permanece obligatoria.
Gate Unicode R1 rechazado en revisión/AE independientes: UROOT-01 reproduce fallo de cleanup del runner con rescate propio; UG-AE01 detecta comprobación del presupuesto de anchura después del escaneo; UG-AE02 registra divergencias RI con bloques4/5/7. Los corpus reales32/4096 contrastados no mostraron esa divergencia, sin extrapolarlo a corrección del producto. Candidato R2 separado en preparación, con fuente/expectativas previas preservadas; S03-UNICODE01/RV-105/504 continúan abiertos.

PDF Rust R2 rechazado por PRUST-AE01: una espera transitoria deja sin reap/handle cerrado al hijo y requiere rescate. Poll y carryUTF16 pasan controles acotados; cobertura semántica continúa sin acreditar. Candidato R3 con reintentos y autoridad Pending entregado para nueva revisión/AE; no adopción ni cierre de RV-204.

Markdown privado R2 ejecutó21casos sobre expectativas anteriores a outputs y7controles del supervisor sin rescate externo, con fmt/clippy/build; cinco casos mantienen observaciones exploratorias explícitas. Parser reconoce wikiembedPDF y conserva pagefragment, pero resolución/render/composición no están implementados. Freeze691entradas entregado para revisión/AE distintas del coordinador autor; no aceptación automática, producción ni cierre de RV-301/302/303/305.

Unicode gate privado R2: [viabilidad](research/S03_UNICODE_GATE.md) con revisión de código y AE independientes sin críticas pendientes. Quedan cerrados los fallos del experimento UROOT01/UG-AE01/02; S03-UNICODE01 sigue abierto hasta integración y aceptación en todas las rutas reales. Sin cierre RV-105/504/S03.

Markdown R2 rechazado por MD-AE01 (stream.close fallido pierde autoridad Pending) y MD-AE02 (cuota de bytes corta UTF8 y presenta error equivocado). R3 separado corrige ambos, repite21casos y7controles anteriores, añade4controles y está congelado para revisión/AE independientes; sin adopción ni cierre de formatos.

PDF Rust R3 rechazado por PRUST-ROOT02: error del reader al EOF posterior a comprobación inicial podía publicar validated_response tras join. R4 separado mueve validación IO después de joins e incorpora errores de cierre del writer; candidato pendiente de revisión/AE. Cobertura semántica nativa continúa sin acreditar, RV-204 abierto.


Markdown gate privado R3: [evidencia final](research/MARKDOWN_PARSER_GATE.md) y revisión/AE independientes sin críticas pendientes; MD-AE01/02 cerrados para el experimento. Parser/procedencia viable, pendiente integración de documento enriquecido y todas sus capacidades gráficas/Obsidian. No cierre RV-301/302/303/305 ni recorte de alcance.


PDF Rust R4: [estado y evidencia](research/PDF_RUST_GATE.md), revisión y AE independientes sin críticas pendientes en lifecycle/I/O; PRUST-ROOT02 cerrado para este corte. Extracción semántica nativa permanece Unverified con fallos originales conservados; investigar causa/alternativas e integrar producto antes de cerrar RV-204/G1.


S03-UNICODE01: [plan operacional](refinement/S03-UNICODE-INTEGRACION.md) revisado [sin críticas pendientes](reviews/S03-unicode-integration-plan-r1.md) sobre rutas actuales; implementación conjunta Source/Nu/inline/legacy/borrowedYAMLXML/preflight autorizada en el incremento activo. Gate verde no sustituye revisión/AE del código integrado; defecto y RV-105/504 permanecen abiertos.


Markdown: [plan operacional de documento enriquecido](refinement/MARKDOWN-DOCUMENTO.md) revisado [sin críticas pendientes a nivel general](reviews/Markdown-document-integration-plan-r1.md). Mantiene imágenes/Mermaid/mathinline+block/Obsidian y separar datosNu/DOM. Wire/ledger/contención efectiva/baseline/resolver/servicios se refinan antes de implementación; no abre otro sprint en paralelo a integraciónUnicode ni cierra prestaciones.


PDF texto: [investigación de primarias fijadas](research/PDF_TEXTO_INVESTIGACION.md) identifica truncado32767 en parser antes de extracción y unidadesUTF16 separadas detrás de GetUnicode; [seguimiento](research/PDF_TEXTO_SEGUIMIENTO.md) señala solapamiento posible al concatenar GetText por ventanas nativas. PropuestaUTF16R2 elige una llamada acotada por página y decodeRust cancelable; revisión exige cerrar esquema/strides/ledger y asociaciones exactas de índices (U16-P01/02) antes de construir el nuevo candidato. Investigación no cambia FAIL originales ni acredita SourceComplete, geometría completa o RV-204.


Markdown documento: [arena/wire R2](refinement/MARKDOWN-DOCUMENTO-WIRE.md) y tamaños tienen [revisión independiente sin críticas pendientes de contrato](reviews/Markdown-document-wire-plan-r2.md). Cierra esquemas/ownership/orden/procedencia y framing para implementación después de frontera de integraciónUnicode; source128MiB se conserva, memoria de fuente/biblioteca/arena se distingue explícitamente. Contención efectiva, código/AE y capacidades enriquecidas completas permanecen abiertos.


PDF texto: [contrato UTF16 R3+deltaR4](refinement/PDF-TEXTO-UTF16.md) con [revisión independiente final](reviews/PDF-texto-utf16-plan-r4.md) sin críticas pendientes de diseño. U16-P01/02/03 resueltos: tablas/asociaciones exactas, cotas y rechazo temprano con receipts finales. Gate Rust privado iniciado, sin runtime/aceptación ni adopción; SourceUnverified, FAIL históricos y RV-204/G1 abiertos se conservan.


HTML: [inspección primaria de API](research/HTML_PARSER_API.md) confirma sinkDOM mutable, entrada Unicode incremental y decodificaciónUTF8 lossy de la API de comodidad; identifica gates para identidad/procedencia, presupuestos y configuración noscript. Sólo investigación documental; versión/dependencia/implementación/aceptación de RV-402/404 pendientes.


GIPC privado R1 congelado FCB6AE119CC5CE20ED34FB95BB9C747FF2715B8A403C7BC394320230952F887B: revisión de código y AE finalizadas, ambas NO APROBADO por ROOT01. GIPC-ROOT01 reproducido nativamente: cleanup devuelve Reaped con tres handles de pipes no transferidos aún vivos; GetHandleInformation propio confirma tras wait/Active0/close proceso. Fallo y snapshot278registros preservados fuera de Git, sin rescate de procesos. R1 no aprobado; corregir cierres/autoridad Pending y repetir aceptación. No cierre G2/RV ni adopción.


GIPC R2 privado en corrección de propiedad/cierre: conservar pipes y duplicados hasta cierre probado, distinguir join/receipt/cierre y bloquear siguiente actor ante Pending. AE R1 preserva el primer fallo de lectura bajo sandbox y la corrección de un oráculo Kitty; sus controles persistentes Python no acreditan cierre Rust/Win32. No cierre G2 ni adopción.

PDF texto: [reinspección de fuentes fijadas](research/PDF_TEXTO_INVESTIGACION.md) documenta normalización de ligaduras antes de char_list/text_buf; concordancia entre APIs y flags cero no demuestra texto de fuente exacto. Candidato UTF16 privado sigue en implementación/validación; se conservan oráculos, Unverified y RV-204 abierto.


RV-404: prueba nativa HTTP identifica que Nu0.115.1 Windows llama Command.run y rechaza un conversor custom escrito en Nu, aunque open sí lo ejecuta. Se preservan dos FAIL y cuatro controles PASS; [fuentes y evidencia](research/HTML_PARSER_API.md). Propuesta privada de companion opcional PluginCommand pendiente de revisión/spike; Q25, árbol y comandos nativos intactos siguen obligatorios, sin cierre de RV-404.


GIPC privado R2: [evidencia final](research/GRAFICOS_IPC_GATE.md), [revisión de código](reviews/GRAFICOS-ipc-code-r2.md) y [aceptación independiente](reviews/GRAFICOS-ipc-aceptacion-r2.md) sin críticas pendientes dentro del alcance probado. ROOT01 cerrado para el experimento mediante consulta nativa de tres handles originales;383 registros candidato y63 evidencias AE verificados.149 casos exact-bins y cuatro grupos propios nativos; seis cierres persistentes siguen FailedPending, teardown externo no cuenta como reap. R1/fallos conservados. Pendientes integración, cuotas completas, terminal/visual/Unix/cache/SLA/distribución; sin cierre G2/RV ni adopción en producción.


OBS-PARSE01 abierto: [investigación Obsidian](research/OBSIDIAN_RESOLUCION.md) y cinco fixtures propios previos a output detectan tres FAIL del parser fijado al conservar backslash de alias/tamaño en tablas (nota/PDF/imagen); dos controles PASS. Contexto TableCell/source necesita adaptación antes de resolver. Cinco lifecycle nativos cerrados sin rescate no acreditan semántica; fallo/oráculos conservados, revisión independiente iniciada. No tratar conteos regex del corpus como rutas Windows ni cerrar RV-301/304/305.


PDF UTF16 privado R1 congelado1076registros: [estado y evidencia](research/PDF_UTF16_GATE.md). Revisión de código rechazada por PDF-U16-ROOT01: callback marcaEOF antesclose real y acepta Failure prematura, lanzaV y publicaContentFailure. Reproducción propia + corroboración independiente, sinrescate; R1/oráculos/FAILSourceUnverified intactos. Corrección R2 separada requerida; AE restante en curso. No cierre RV-204/G1 ni adopción.
