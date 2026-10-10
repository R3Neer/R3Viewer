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


PDF UTF16 privado R1 congelado1076registros: [estado y evidencia](research/PDF_UTF16_GATE.md). Revisión de código rechazada por PDF-U16-ROOT01: callback marcaEOF antesclose real y acepta Failure prematura, lanzaV y publicaContentFailure. Reproducción propia + corroboración independiente, sinrescate; R1/oráculos/FAILSourceUnverified intactos. Corrección R2 separada requerida; AE R1 final NO APROBADO por PDF-U16-AE-EOF01, 53 evidencias verificadas por raíz. No cierre RV-204/G1 ni adopción.

Refinamiento 2026-10-06: adaptación contextual [Obsidian R2](refinement/OBSIDIAN-TABLE-ADAPTER-R2.md) y spike [PluginCommand HTML R2](refinement/HTML-CONVERSOR-NU-R2.md), ambos con revisión independiente sin críticas pendientes de diseño. No cierran OBS-PARSE01/RV-301/304/305 ni RV-404. HTML inicia implementación privada de registro; documento enriquecido se integrará después de la frontera Unicode. Código, runtime y aceptación pendientes.

Unicode integrado R1: [estado y evidencia](research/S03_UNICODE_INTEGRACION.md), candidato502 registros. UIR-01 reproducido independientemente en YAML/XML: filas cortas pierden primer grafema de fila siguiente (seis FAIL, dos controles PASS); impide aprobar R1. Source/RI/controles de raíz tienen cobertura positiva parcial, no aceptación completa. Corregir y revisar R2; S03-UNICODE01/RV y QA permanecen abiertos.

PDF UTF16 R2 final: [estado y evidencia](research/PDF_UTF16_GATE.md), revisión de código y AE independientes sin críticas pendientes dentro del delta EOF/observación/retención. ROOT01/AE-EOF01 resueltos en helper privado1112 registros; doce familias AE y19 evidencias verificadas. R1 y fallos intermedios preservados. No cierre RV-204/G1 ni garantía SourceComplete; integración, gráficos y QA pendientes.

RV-404: spike PluginCommand R1 [rechazado](research/HTML_PARSER_API.md) por HNU-R01: driver acepta PASS con Nu exit7/9 porque sólo comprueba exit del bootstrap C. Dos FAIL propios y control positivo preservados, cleanup acreditado. Corrección R2 y AE pendientes; no cambiar el requisito de árbol HTML ni afirmar conversor entregado.

Unicode integración R2: [revisión final](reviews/S03-unicode-integracion-r2.md), 24/24 controles propios cierra UIR-01 en snapshot528 registros. Raíz verifica receipt/ocho evidencias y22 de investigación focalizada: XML32MB/YAML16MB aislados no reproducen hora;3695.62s/WorkerExited originales siguen sin causa probada. [Estado vigente e historia](research/S03_UNICODE_INTEGRACION.md). S03-UNICODE01/RV/QA permanecen abiertos; no declarar rendimiento global/adopción completa.

RV-404: [spike PluginCommand R2](research/HTML_PARSER_API.md) con revisión de código y AE independientes sin críticas para registro nativo. HNU-R01 cerrado (Nu exit!=0 rechazado, incluso JSON válido);15 expectativas AE y tres controles originales de raíz, evidencias verificadas. Companion opcional directo viable en Nu0.115.1 Windows, raw intacto; parser/DOM/modelo/origen/128MiB/Unix y conversor productivo siguen pendientes, no cierre RV-404.

Markdown documental: [plan operacional de implementación R2](refinement/MARKDOWN-DOCUMENTO-IMPLEMENTACION.md) con [revisión adversaria final sin críticas pendientes](reviews/Markdown-document-implementation-plan-r2.md). Implementación A–C iniciada en copia aislada: modelo/validator/codec reales, parser→arena con Obsidian contextual y worker/controller R3MD verificado. D adopción TUI/inline posterior permanece explícita; no facade JSON ni cierre RV-301/304/305. Raíz preparó20 oráculos semánticos propios antes outputs (ROOT-EXPECTED-before-output-r2.json8FEF4E9E0BAAEE544BD4C15688BD5DDE6BD526B8C1F0D65957F8E464C80F7FDB); preparación no es prueba de funcionamiento.

Markdown worker: helper ChildJob::attach_verified R2 con [revisión y aceptación independientes](reviews/Markdown-worker-job-r2.md),12 controlesPASS; JOBV-01/02 cerrados. Readback512MiB/flags exactos/sinbreakaway, membresía exacta y API conserva autoridad Job en fallo posterior a creación. Raíz verificó cuatro evidencias/receipt; candidato privado listo para adoptar en controllerC, no integrado en repo todavía. Cierre fallible/Pending/START/wire/lifecycle/Unix no acreditados por este helper.

Unicode integrado: aceptación R2 NO APROBADO por S03-AE-NUKEY01, claveNu16KiB+1 atribuida a Source con span exacto peseFuente vacía. [Evidencia y estado](research/S03_UNICODE_INTEGRACION.md),101 registros verificados; UIR01 ya corregido no se reabre. Raíz entrega R3 privada (un brazo de dominio), propios controles/fmt/Clippy/buildPASS; revisión y AE originales pendientes. Cancelación/recuperación/procedenciaYAML/quota positivas no cierran ConPTY/runtime/Unix/performance ni S03-UNICODE01/RV.

NuKeyR3 [revisado y aceptado](reviews/S03-unicode-clave-nu-r3.md), tres controles originales/relevantesPASS,69 evidencias verificadas por raíz. S03-AE-NUKEY01 cerrado y único guard adoptado en fuente compartida tras descartar solapamiento. Incremento Unicode completo aún sin commit/aprobación general: ConPTY/runtime/visual/Unix/rendimiento histórico permanecen pendientes.

2026-10-06: cierre verificable del Job privado aceptado en [revision independiente](reviews/Markdown-worker-job-close-r1.md): retencion del mismo owner tras error inyectado, reintento nativo y ausencia de doble cierre incluso con reutilizacion del slot. Root verifico 9 registros de freeze y 24 de evidencia. No adopcion compartida ni aceptacion controller; query ActiveProcesses/TerminateJobObject R2 en revision separada.

2026-10-06: [aceptacion ConPTY Unicode R3](reviews/S03-unicode-conpty-r3.md), CP01-06 funcionales: Reference a vista ordinaria con End, salida q0, Ctrl-C130 sobre fuente32MiB tras frame, modos restaurados y siguiente invocacion correcta; setup/overflow rechazos sticky con cleanup. Root verifico 25 registros y cuatro grupos con oraculo previo intacto. Primer fallo de matcher R1 preservado y reparacion CSI1C demostrada. No acredita cancelacion dentro prepare, QA visual, Unix ni rendimiento historico; RV/S03 global siguen abiertos.

2026-10-06: [revision independiente del Job R2](reviews/Markdown-worker-job-active-r2.md) acepta exclusivamente active_processes y terminate, complementando checkedclose R1. Dos procesos asociados propios acreditan Active2 a1 a0 tras EOF/wait, y TerminateJob137 seguido de ambos wait137 y Active0. Root verifico los10 registros congelados y31 registros de evidencia; cero discrepancias. El implementador puede consumir estas APIs en C. No acredita descendants arbitrarios, controller ni Unix; produccion compartida permanece sin ese delta hasta integracion de sus callers.

2026-10-06: incremento Unicode R2 y correccion NuKey R3 incorporados al codigo compartido, con las dos regresiones de claves. [Registro de adopcion](reviews/S03-unicode-code-adoption-r3.md) y [disposicion independiente](reviews/S03-unicode-commit-disposition-r3.md):31identidades finales verificadas; cinco tests focales frescos PASS,fmtcheck yClippyworkspacealltargets0. El marcador de cache .cargo-ok queda excluido de Git, sin alterar fuente upstream. No cierra S03/RV, QA visual, Unix, distribucion, SLA ni investigacion de rendimiento historico; Markdown continua en copia privada.

2026-10-06: [plan consumidor Markdown R3](refinement/MARKDOWN-CONSUMIDOR-DOCUMENTO.md) con [revisión adversaria final sin críticas pendientes](reviews/Markdown-document-consumer-plan-r3.md). D1–D3: texto lógico multirun/Unicode, Flow y navegación con endpoints globales cobrados, Documento inicial para Markdown reconocido y salida inline completa/fallback de Fuente. Críticas DC-R01/02/03 cerradas; DC-R04 fija D4 obligatorio: reutilizar Syntect5.3 en bloques de código, estado aislado y scheduler/leases acotados, preservando texto íntegro. D4 necesita refinamiento específico revisado antes código; no nuevo resaltador por lenguaje. A–C está en implementación/aceptación privada, consumidor sin implementar; no cierre RV/Markdown/Obsidian, gráficos, QA visual ni Unix.

2026-10-06: A–C documental R1 permanece NO APROBADO en aceptación dirigida por MD-AE-UNC01 (alias MD-UNC01): fixture UNC CRLF válida conserva dos barras iniciales en Source, pulldown devuelve una y el adaptador rechaza InvalidContext. [Plan de corrección acotada](refinement/MARKDOWN-UNC-CORRECCION.md) con [revisión final sin críticas de diseño](reviews/Markdown-UNC-correction-plan-r1.md), implementación R2 privada iniciada; no normalizar rutas ni cambiar el oráculo original. AE parcial acredita41 propiedades semánticas,14 mutantes y fronteras/controller concretos; una fixture tight tenía error de oráculo y los gates Source128MiB/plazo/error compuesto/revisión exhaustiva siguen pendientes. Root verificó45 registros de evidencia; no aceptación general ni adopción compartida/RV. MD-PACK01 del patch quedó reparado: aplicación Git propia36 archivos byteexactos; MD-CONTEXT01 precisado/corregido con17resources y237vendor de baseline acreditados, build independiente posterior PASS. La sospecha inicial de vendor.rs ausente fue retirada; fallos originales conservados. D1 continúa sobre dependencia provisional en copia propia; D4 queda en borrador durante esta corrección.

2026-10-06: UNC R2 congelado37 registros, tres fuentes modificadas; predicciones24/24 del parser original,6 unitarias y7 pipeline propias PASS, revisión/AE del delta pendientes. Segundo fallo distinto MD-CALLOUT-EMPTY01 confirmado independientemente: títulos de callout con enlace vacío (también anidado en Strong) producen SpanLink con cero Runs y domainNONE, panic en compact. Quote normal controla el caso vacío correctamente. [Plan R3](refinement/MARKDOWN-CALLOUT-VACIO.md) con [revisión final de diseño](reviews/Markdown-callout-empty-correction-plan-r1.md): conservar Title/body owners cuando Spans los requieren, sin syntheticRuns ni cambios wire/validator/cuotas. Código R3 privado iniciado, no adopción. Gates independientes adicionales de A–C R1 pasan en su alcance: constructor/codec Source128MiB exacto/+1 con helper60s sin parser ni SLA10s; plazo absoluto incluye espera10.1s antes spawn; errores compuestos y threadstart conservan causa/leases y cleanup. Estos resultados no cierran A–C ni RV; R1 sigue rechazado y ambos freezes/fallos permanecen intactos.

2026-10-06: UNC R2 pasa [revisión estática raíz](reviews/Markdown-UNC-code-r2.md) y [aceptación independiente](reviews/Markdown-UNC-acceptance-r2.md) en el delta acotado:33 controles, incluida fixture original,24 variantes,5 mutaciones rechazadas y3 controles inertes. Raíz verifica37 registros del candidato y17 de evidencia, sin discrepancias. MD-AE-UNC01/MD-UNC01 queda resuelto para ese candidato; R1/fallos intactos. No adopción A–C: MD-CALLOUT-EMPTY01 está en revisión de R3 y la fixture UNC usa ruta temporal absoluta que debe convertirse en recurso relativo reproducible antes de incorporar tests. No cierre RV/Markdown/Obsidian ni QA/Unix/SLA; no se ejecutó el worker exacto R2 desde esta AE pura.

2026-10-06: aceptación independiente R3 confirma MD-CALLOUT-EMPTY01 corregido en sus tres entradas originales y owners Title/Body separados, pero R3 queda NO APROBADO por MD-CALLOUT-BODY01: body-only `> [!note]\n> [](y)\n` y NOTE asignan el enlace vacío al título al faltar Runs/SoftBreak del marcador consumido por GFM. Raíz verificó24 registros AE; Source12..17 y oráculo original intactos. [Plan de corrección R4](refinement/MARKDOWN-CALLOUT-CUERPO-VACIO.md) con [revisión adversaria sin críticas de diseño](reviews/Markdown-callout-body-plan-r1.md): usar Origin Source para partición del span vacío y la misma decisión en owner/remapeo, sin nuevos Runs ni cambios validator/wire/cuotas. Implementación privada autorizada; código/aceptación/adopción pendientes. La corrección de portabilidad de la fixture UNC está congelada por separado, sin incorporar R3 rechazado. D1 congelado para revisión/AE; D2 refinamiento en curso. No cierre A–C/RV/QA/Unix/SLA.

2026-10-06: D1 provider/index congelado376 registros pasa [revisión y aceptación independiente raíz](reviews/Markdown-D1-acceptance-r1.md),3 grupos propios focales PASS y copia318 archivos byteexactos+1prefijo test intacto. Unicode multirun/estilos/CRLF, procedencia Source/control/decoded, cuotas old+new/identidades, Work sinavanceparcial y cancel tras8192B reales con cargos restaurados acreditados. No SourceparserA–C/TUI/inline/SLA/Unix ni adopción combinada; dependencia R1 sigue provisional. [Refinamiento D2](refinement/MARKDOWN-D2-FLOW-LAYOUT.md) tiene [revisión adversaria de diseño sin críticas](reviews/Markdown-D2-plan-r1.md): único ConsumerBudget compartido, slots adaptativos, Flow/End sinRootdraw, layout/search/historia/Sourceactions y branchTUI en cortesD2a/b/c. Extensión de perfiles D1 declarada, implementación espera base A–C corregida/aceptada y revisión de combinación. D3routing/inline y D4Syntectfences explícitos posteriores; recursos gráficos/resolver/RV/QA permanecen abiertos.

2026-10-06: [núcleo documental A–C R4 y D1 integrado](reviews/Markdown-AC-D1-code-adoption-r1.md), tras [disposición independiente completa A–C](reviews/Markdown-AC-disposition-r4.md), aceptaciónD1 y portabilidad separada.42 rutas revisadas copiadas; rootGit37BYTEEXACT y4 grupos combinados privadosPASS. Checks compartidos frescos:12pipeline conworkerreal+4Unicode/parserSource+7presentationPASS, fmt/Clippyworkspacealltargets0. Fuente/dependencias/avisos/fixture relativa incluidos; no harnesses privados ni apuntes. viewer.md aúnFuente, D2layout/navegación yD3routing/inline pendientes; D4Syntectfences en propuesta. No cierre RV/QA/Unix/gráficos/Obsidianresolver/SLA/distribución. Fallos R1–R3 siguen preservados fuera Git.

2026-10-06: fixture UNC sintética preservada con atributo -text acotado; índice Git y checkout privado confirman57B/CRLF y SHA revisado. Evita que la normalización altere offsets de origen al clonar. No amplía aceptación a ejecución Unix.

2026-10-06: [refinamiento D3 routing/inline R2](refinement/MARKDOWN-D3-ROUTING-INLINE.md) con [revisión adversaria sin críticas de diseño](reviews/Markdown-D3-plan-r2.md). Geometría y gutter inline, SourcePreparationLimit sin falsa Fuente completa y Loading pump q0/CtrlC130 cerrados en propuesta; dictamen posterior autoriza implementar por cortes D3a/b/c. R1 e informes intermedios intactos fuera Git. No runtime ni adopción de selector CLI aún; D2/D4 dependencias explícitas, RV/QA/Unix/gráficos abiertos.

2026-10-06: [D4 Syntect fences R4](refinement/MARKDOWN-D4-SYNTECT.md) con [revisión adversaria final sin críticas de diseño](reviews/Markdown-D4-plan-r4.md). Cierres R01–04: texto completo antes overlay, deadline absoluto del color, permisos cerrados de trabajo/render y autoridad textual independiente del timeout opcional. Dictamen posterior autoriza implementación privada D4a/b/c; perfiles/hooks D2 todavía en desarrollo, oráculos previos19/8/13 y addenda preservados fuera Git. No código/render/runtime aprobado, no cierre RV/QA/Unix/gráficos.

2026-10-06: D3a candidato selección/sharedSource aceptado independientemente6controlesPASS. [Adopción acotada](reviews/Markdown-D3a-source-api-adoption-r1.md) sólo2SourceAPIs para compartir Arc original; selector/CLI/language todavía privados hastaD3c. No anunciar vistaMarkdown ni cerrar RV/TUI/inline/QA/Unix.

2026-10-06: [D2a R2 integrado](reviews/Markdown-D2a-code-adoption-r2.md), con [aceptación independiente](reviews/Markdown-D2a-acceptance-r2.md) de siete grupos propios. Flow/perfiles/navegación lógica y ownership de cursores aceptados; combinación compartida: 15 D2a pequeños, 4 D1 y 7 Source, fmt/Clippy aprobados. No presentación Markdown/TUI, routing/inline, color, gráficos ni cierre RV-301. D2b sigue en refinamiento adversario.

2026-10-06: [D2b R3, presentación y búsqueda](refinement/MARKDOWN-D2B-LAYOUT-SEARCH.md) con [revisión adversaria final sin críticas de diseño](reviews/Markdown-D2b-plan-r3.md). Cierra ancestros colapsados, preparación fuera de UI y biblioteca común previa sin dependencia circular. Autoriza implementar b0 (propietarios Source/hash reales) y b1–b3 sobre D2a integrado; cada código requiere aceptación propia. No runtime de esas partes ni cierre RV/TUI/QA.

2026-10-06: [continuidad HASH a SOURCE de D2b0 R2](refinement/MARKDOWN-D2B0-HASH-SOURCE.md) con [revision independiente sin criticas de diseno](reviews/Markdown-D2b0-continuity-plan-r2.md). El contexto cerrado conserva Arc original, inputIdentity y digest real tras join; getter inmutable de ArcSource desde Document autorizado. Intento b0 R1 preservado sin aprobacion por ausencia de autoridad tipada, visibilidad externa y continuidad incompleta. Correccion R2 en implementacion privada; no adopcion de codigo, pump/ConPTY, b1 ni cierre RV. Oraculos y fallos anteriores intactos.

2026-10-06: [scheduler CodeText D4b R3](refinement/MARKDOWN-D4B-SCHEDULER.md) con [revision adversaria final sin criticas de diseno](reviews/Markdown-D4b-plan-r3.md). Cache semantica y activacion actual separadas; cobertura completa y nueva conexion contextual tras reap incluso con mismo bloque. R01/R02 cerradas en propuesta, oraculos previos intactos. Implementacion depende de D4a R2 aceptada y biblioteca b0/b1 reales para background; retirement/render D4c posteriores. No codigo D4b aprobado ni cierre RV/color/TUI/Unix/QA.

2026-10-06: [D4a R2 integrado](reviews/Markdown-D4a-code-adoption-r2.md), con [aceptacion independiente cerrada](reviews/Markdown-D4a-acceptance-r2.md). OWN01 resuelto mediante receipt opaco; seis grupos puros y Source a Document nativo finalbin pasan, sin relajar plazos. Combinacion compartida25tests/fmt/ClippyPASS; fixtures13 protegidas de normalizacion CRLF. Timeouts anteriores intactos y sin causa historica atribuida. Scheduler D4b, overlay D4c, Markdown visible, TUI/Unix/QA/RV siguen pendientes.

2026-10-06: [D2b0 R2 integrado](reviews/Markdown-D2b0-code-adoption-r2.md), con [aceptación independiente](reviews/Markdown-D2b0-acceptance-r2.md) sin críticas pendientes de biblioteca. Doce grupos propios y combinación compartida40 tests/fmt/Clippy PASS. HASH/SOURCE conserva contexto y original; cierre físico y Pending bloquean nuevas preparaciones. Sin layout b1, pump/ConPTY, Markdown visible, Unix/QA ni cierre RV. R1 rechazado permanece intacto.

2026-10-06: [alias inmutable FlowHandle para b1](refinement/MARKDOWN-D2B1-FLOW-HANDLE.md), con [revisión de diseño sin críticas pendientes](reviews/Markdown-D2b1-FlowHandle-plan-r1.md). Comparte FlowIndex sin copiar checkpoints, cobra metadata/aliases en el mismo Consumer y conserva leases hasta último Drop. Diseño documentado; b1 en implementación privada sobre c423421, sin aceptación de código/layout/renderer/QA.

2026-10-06: [constructor CodeText D4b R2 integrado](reviews/Markdown-D4b-constructor-code-adoption-r2.md), con [aceptación independiente sin críticas pendientes](reviews/Markdown-D4b-constructor-acceptance-r2.md). Dos pasadas y empty temprano acreditados; 8 controles independientes focales y45pruebas compartidas/fmt/ClippyPASS. Fixtures9 conservadas byteexactas por atributo. Scheduler/cache/activación/overlay y Markdown visible permanecen pendientes; no cierre D4b/RV. R1 rechazado intacto.

2026-10-06: [helper de Fuente completa inline integrado](reviews/Markdown-inline-source-code-adoption-r1.md), con [aceptación independiente sin críticas](reviews/Markdown-inline-source-acceptance-r1.md). Cuatro controles propios, nueve pruebas compartidas/fmt/ClippyPASS; proyección completa sin índice TUI y límite/cancelación transaccionales. Caller Markdown, gutter/color y CLI siguen pendientes; no cierre D3/RV-103/RV-301.

2026-10-06: [predicados D4b R1 integrados](reviews/Markdown-D4b-predicates-code-adoption-r1.md), con [aceptación independiente](reviews/Markdown-D4b-predicates-acceptance-r1.md) sin críticas para cobertura, caché vacía y stop local prestado. Readback25/33 sin discrepancias; combinación compartida23pruebas/fmt/ClippyPASS. No productor de estilos, scheduler, activación, renderer ni cierre D4b/RV/QA.

2026-10-06: [D2b1 R1 rechazado](reviews/Markdown-D2b1-acceptance-r1.md) por MD-B1-ANCHOR01: width0 permite commit de rawbyte fuera de Run al saltarse validación en Reference Geometry. Diez grupos independientes positivos no compensan ese FAIL;42 evidencias verificadas. Sin adopción del candidato; corrección R2 en marcha. Emisor inline completo en copia privada: ocho controles propios iniciales PASS, todavía sin roles/resources compartidos ni aceptación/CLI. No cierre RV-301.

2026-10-06: [D2b1 R2 rechazado](reviews/Markdown-D2b1-acceptance-r2.md) por MD-B1-C1-PROGRESS01: wrap dentro de un control C1 proyectado pierde la continuación visual, Line repite ventana y no alcanza texto posterior. MD-B1-ANCHOR01 cerrado en este candidato y nueve grupos independientes positivos;36 evidencias verificadas. Corrección R3 requerida, sin adopción. Medición inicial b2: dobleBuffer Ratatui120x40 usa460800B (Cell48B), supera framework256KiB; se investiga capacidad práctica antes de aceptar renderer, sin relajación silenciosa ni Reference permanente como solución final.

2026-10-06: [D2b1 R3 incorporado](reviews/Markdown-D2b1-code-adoption-r3.md), con [aceptación independiente sin críticas pendientes](reviews/Markdown-D2b1-acceptance-r3.md) en biblioteca. MD-B1-ANCHOR01 y MD-B1-C1-PROGRESS01 resueltos por reproducciones originales;39 evidencias verificadas y68 pruebas de composición PASS. Formato y Clippy de todos los targets PASS. Renderer/pump/CLI, factor RefID combinado y diagnóstico de coste práctico siguen pendientes; sin cierre RV-301 ni QA/Unix.

2026-10-06: [biblioteca inline Markdown R3 incorporada](reviews/Markdown-inline-library-adoption-r3.md), con aceptaciones independientes [inline](reviews/Markdown-inline-library-acceptance-r3.md) y [RefID/roles](reviews/Markdown-ref-roles-acceptance-r1.md). Emisor completo y proveedor R3+Ref byteexactos,53 pruebas compartidas PASS,fmt/Clippy todos los targets PASS tras retirar atributo duplicado en lib. Caller/selector/CLI y renderer interactivo siguen pendientes; no cierre RV-301 ni QA/Unix. Implementación privada del caller real en curso.

2026-10-06: [ResolvedUnitR1 incorporado](reviews/Markdown-resolved-unit-adoption-r1.md), con [aceptación independiente](reviews/Markdown-resolved-unit-acceptance-r1.md) sin críticas para token opaco/atomicidad/Unicode/layout. PRODUCT4 verificado contra base y variantehint4;39+22 evidencias y69 pruebas compartidas PASS,fmt/ClippyPASS. Cuotas compartidas sin cambio; calibración Work2M y Framework2M siguen privadas. Caller/renderer/pump/QA pendientes; no cierre RV-301.

2026-10-06: hito cercano solicitado antes de pausar: [Markdown inline integrado](reviews/Markdown-inline-milestone-adoption-r3.md) y [aceptado independientemente](reviews/Markdown-inline-caller-acceptance-r3.md). Autoarchivo .md/.markdown, selector explícito para texto y conservación de datos Nu;38 pruebas de composición y5 comandos Nushell reales PASS,fmt/build/ClippyPASS. Interactivo permanece Fuente con aviso; gráficos/fences/Obsidian/Unix pendientes. No cierre RV-301 ni del objetivo completo. El resto del backlog conserva su alcance y se reanuda sólo por petición posterior del usuario.

2026-10-09: objetivo reactivado tras la pausa y HEAD0cb0fe6 comprobado limpio. [Plan del primer renderer real](refinement/MARKDOWN-RENDERER-REANUDACION-R2.md) con [revisión adversaria sin críticas de diseño](reviews/Markdown-renderer-plan-reanudacion-r2.md): owner Ratatui, almacenamiento compacto, estilos de spans, limpieza de continuaciones y preadmisión compartida de layout/diff/escritura. Perfil DocumentWork4MiB candidato explícito; límites Window/Frame/Consumer conservados y Legacy intacto. Implementación privada iniciada, pendiente medición completa y aceptación de código. No cierre RV-301, pump, roles/recursos completos, QA visual ni Unix.


2026-10-09: [primer renderer textual con estilos integrado](reviews/Markdown-renderer-text-adoption-r2.md), tras [aceptación independiente sin críticas pendientes](reviews/Markdown-renderer-text-acceptance-r2.md) para este corte. Owner Ratatui, compactación, preclears, cargos y cancelación incluidos; cuatro grupos propios adversarios PASS y 51 pruebas de composición, fmt/Clippy PASS. DocumentWork8MiB explícito sustituye candidato4MiB insuficiente, con fallo anterior conservado y límites de memoria acotados. No conexión al pump/comando interactivo, roles/recursos completos, fences, gráficos ni cierre RV-301/QA/Unix. [Pausa solicitada tras el hito](process/PAUSA-2026-10-09.md); no comenzar siguiente incremento hasta nueva petición del usuario.


2026-10-09, reanudación posterior: el usuario revoca la pausa y autoriza continuar el producto completo sin pausas voluntarias hasta nueva indicación. HEAD353418c limpio y goal activo comprobados. Refinamiento b2 de roles/recursos del renderer con adversario independiente en curso; especificaciones b3 de búsqueda/acciones se preparan en paralelo. Ningún cierre adicional de RV ni ampliación de la aceptación del corte previo.


2026-10-09: [contrato b3 de búsqueda y acciones R2](refinement/MARKDOWN-D2B3-SEARCH-ACTIONS.md) con [revisión adversaria sin críticas pendientes de diseño](reviews/Markdown-D2b3-plan-r2.md). SourceHit absoluto separado para marcado/targets sin Run y headings ocultos mediante expansión conjunta/remap en Flow candidato cierran B3-R01/02. Biblioteca e historial/Source ownership todavía no implementados; dependen de anchors b2 congelados. Refinamiento b2 de bloques/recursos continúa en revisión; no cierre RV-301/306, pump, gráficos, QA ni Unix.


2026-10-09: [presentación b2 R2 de bloques y recursos seguros](refinement/MARKDOWN-D2B2-PRESENTATION.md) con [revisión de diseño sin críticas pendientes](reviews/Markdown-D2b2-plan-r2.md). A01/A02 resueltos mediante posiciones lógicas por celda y propietarios TextRef de BlockRole/Resource; reserva cerrada del renderer conserva el mismo techo DocumentWork8MiB, con cargos efectivos pendientes de medición. Implementación privada b2 iniciada; núcleo independiente b3 de walker/búsqueda/headings también en código sobre modelo/proveedores reales, sin modificar layout/historia/owners. Versiones intermedias y oráculos previos intactos fuera Git. Ningún runtime de estos cortes aceptado ni cierre RV-301/306, gráficos, pump, QA o Unix.

2026-10-09: [delta revisado de historial atrás/adelante](refinement/MARKDOWN-HISTORY-FORWARD.md) incorporado al contrato b3. Timeline con cursor y cuota conjunta128; Back preserva futuro, una nueva transición sólo lo trunca al confirmarse. Siete oráculos previos fijados; no código de history aceptado ni cierre RV-306. El núcleo de búsqueda continúa independiente de este almacenamiento.

2026-10-09: [núcleo semántico de búsqueda b3 R2 integrado](reviews/Markdown-D2b3-coresearch-adoption-r2.md), tras [aceptación independiente sin críticas](reviews/Markdown-D2b3-coresearch-acceptance-r2.md). OWNER01 corregido con ResourceID consumible en Hit/Pending/retry;15 grupos propios y65pruebas de composición/fmt/ClippyPASS. R1/evidencias intactos; fixtures byteexactos. Headings paginados y búsqueda original separados de SourceReady; inspector/acciones/history/pump/renderer b2 siguen pendientes. No cierre RV-301/306 ni QA/Unix.

2026-10-09: [almacenamiento/commit de history b3 R2](refinement/MARKDOWN-HISTORY-STORAGE.md), con [revisión adversaria sin críticas de diseño](reviews/Markdown-D2b3-history-plan-r2.md). H01 cerrado mediante anchors completos Ref/Resource/Table At-Done; quota128 conjunta y retirement preadmitido retienen cargos reales hasta Drop fuera de publicación. Implementación depende del freeze b2 y adaptadores cerrados/pane guards reales; no tipos numéricos inventados ni código/RV-306 aceptados. Inspector en refinamiento focal independiente, core de búsqueda ya integrado.

2026-10-09: [inspector/procedencia R2+I-A01](refinement/MARKDOWN-INSPECTOR.md) con [revisión de diseño sin críticas pendientes](reviews/Markdown-D2b3-inspector-plan-r2.md). I-R01/I-R02 cerrados mediante owner/leases/cargos reales y summary sólo tras EndOfCoverage; desde Hit se inspecciona unidad del anchor, no todo highlight como glyph. Incluye entrada sin query, ResourceOwner/field, links múltiples, páginas64+1 y SourceAction tipada. Código pendiente de cursores D1 y adaptadores b2 reales; no cierre inspector/RV-306/SourceReady/pump/QA.


2026-10-09: [renderer documental b2 R1 integrado](reviews/Markdown-D2b2-renderer-adoption-r1.md), con [aceptación independiente sin críticas pendientes](reviews/Markdown-D2b2-renderer-acceptance-r1.md). Bloques/roles/recursos seguros y tablas con continuaciones At/Done; cuatro grupos propios y133 regresiones de composición privada/canónica, fmt/ClippyPASS. SourceLegacy/Cargo/vendor intactos; oráculos/fallos previos conservados. No conexión al comando interactivo ni cierre RV-301/306, gráficos, fences, ConPTY documental, QA visual o Unix. Inspector en implementación privada; D2c sesión/pump en revisión de diseño.

2026-10-09: [sesión/pump Markdown D2c, contrato final R1+R2+R04](refinement/MARKDOWN-SESSION-PUMP.md), con [revisión independiente y composición sin críticas pendientes](reviews/Markdown-D2c-session-plan-r2.md). Una autoridad de preparación, frame completo con chrome y cargos reales, Fuente compartida sin clonar modelo, selector de dominios y restore antes cleanup. Ambos oráculos previos normativos se conservan canónicos; CF-01 documental cerrada. Source común e inspector en implementación privada; IP-P01/IP-P02 de admisión de proveedores siguen abiertos. No código/pump/ConPTY aceptados ni cierre RV-104/301/306, routing D3c, fences D4, gráficos, QA visual o Unix.

2026-10-09: [admisión de proveedores b3 R2](refinement/MARKDOWN-PROVIDER-ADMISSION.md), con [revisión independiente sin críticas de diseño](reviews/Markdown-D2b3-provider-plan-r2.md). IP-P01/IP-P02 cerrados en diseño: adapter real sobre la misma autoridad, gate antes de cada índice, permisos/Weak y cargos hasta últimoDrop, búsqueda scoped sin retener bestIndex entre secuencias y retiro físico de caché. P01–P13 prospectivos conservados; implementación privada coherente con inspector autorizada. La dependencia abierta histórica del contrato D2c pasa a implementación, todavía sin aceptación física/código ni cierre RV-306/pump/QA; D1 e INLINE legacy no reciben cuota global nueva.


2026-10-10: B2-RESOURCE-OWNER01 confirmado sobre código canónico3db799c sin gate ni Source común: una preparación real con tres imágenes y dos bloques termina en Panic al interpretar ResourceAlt.owner (ResourceID) como BlockID. La aceptación anterior de b2 no cubrió este caso; se conserva su evidencia histórica, pero el defecto queda abierto. Corrección mínima de clasificación por bloque propietario real y regresiones independientes en curso, incluidas coincidencias numéricas con Code/FrontMatter. Source común R1 tampoco se adopta: revisión independiente SC-W01 exige contabilizar aperturas de índices/posición/procedencia en el mismo Work; corrección aditiva R2 en curso. No cierre adicional de RV, inspector, pump ni QA.


2026-10-10: [calibración Source Common512KiB](refinement/MARKDOWN-SOURCE-COMMON-WORK.md), con [revisión independiente sin críticas de diseño](reviews/Markdown-Source-common-work-plan-r2.md). El diagnóstico completo de aperturas/posición/procedencia mide300398 de Work para120x40 y conserva el FAIL previo256K. Cambia sólo la política de preparación Common, sin eliminar cargos ni cambiar Source Legacy, Frame, símbolos, Consumer, Actor o DocumentWork; complementa explícitamente D2c256K histórico. Código R2 y aceptación independiente pendientes; SC-W01 no cerrado ni Source/pump/QA/RV aceptados.


2026-10-10: [B2-RESOURCE-OWNER01 corregido e integrado](reviews/Markdown-B2-resource-owner-adoption-r1.md), tras [aceptación independiente final](reviews/Markdown-B2-resource-owner-acceptance-r1.md). Clasificación por bloque contenedor real evita Panic y texto alternativo oculto por coincidencias con Code/FrontMatter; cargo de 64 bytes por secuencia explícito, límites intactos. Cinco AE propias / 45 regresiones independientes y 77 pruebas de composición canónica, fmt y Clippy PASS. Baseline fallido preservado; cierre sólo del defecto, sin cerrar RV-301/306, inspector, Source, sesión/pump, gráficos, QA visual o Unix.

2026-10-10: [Source Common R2 integrado](reviews/Markdown-Source-common-adoption-r2.md), tras [aceptación independiente sin críticas pendientes de biblioteca](reviews/Markdown-Source-common-acceptance-r2.md). SC-W01 cerrado: aperturas/posición/procedencia medidas con mismo Work, modelo y contexto compartidos sin clonar; Common512KiB explícito, Legacy y memoria intactos. Ocho AE propias en ejecuciones preservadas y132 controles distintos de composición/fmt/Clippy PASS. Fix B2 mantenido. Hash común/Loading, dispatcher inspector, pump/frame nativo, history/routing, gráficos, QA/Unix y RV permanecen pendientes.

2026-10-10: [plan concreto de ejecución D2c](refinement/MARKDOWN-SESSION-EXECUTION.md), con [revisión independiente sin críticas de diseño](reviews/Markdown-D2c-session-execution-plan-r1.md). Implementación privada de sesión completa iniciada: Hash financiado, Loading, frame mixto, publicación conjunta, reader restore/Pending, navegación e historial. SourceR2 conservado; inspector/gate2 todavía requiere freeze y AE antes de composición. Dos oráculos previos y suplemento prospectivo intactos. No código/pump/ConPTY aceptados, ni cierre RV/QA/Unix; D3c routing y D4 fences posteriores obligatorios.

2026-10-10: inspector/gate R1 no adoptado por CGR1-W01, confirmado mediante AE independiente sobre el freeze exacto: captura hit/anchor/cursor admite respectivamente176/144/184 de Work frente a Scope144 + cuerpo112; proof hit/anchor admite32/0 frente a cuerpo64. Consumer exact-/1 y rollback físico pasan, pero no compensan la omisión de Work antes de construir. Corrección focal R2 de los cuatro constructores en copia privada, sin cambiar cuotas ni oráculos; R1, binarios y fallos conservados fuera de Git. La revisión de las30familias sigue abierta; no aceptación de inspector/gate, pump, RV o QA.

2026-10-10: [inspector/gate R2 integrado](reviews/Markdown-inspector-gate-adoption-r2.md), tras [aceptación independiente sin críticas pendientes de biblioteca](reviews/Markdown-inspector-gate-acceptance-r2.md). CGR1-W01 cerrado: cuerpos seed/proof admitidos antes construir, exact-/1 y rollback físico comprobados; límites intactos. Matriz30familias,25controles propios y39secundarios R2; composición canónica219pruebas distintas/fmt/Clippy PASS. Source/kernel/Owner01 preservados, fixtures byteexactos. Work total de operación, SourceReady/acciones, sesión/pump/history/routing/fences, gráficos y QA/Unix siguen pendientes; no cierre RV-301/306.


2026-10-10: [diseño HTML estático R2](refinement/HTML-STATIC-DOM.md) y [oráculos prospectivos](refinement/HTML-STATIC-DOM-ORACULOS.md) incorporados tras [revisión independiente sin críticas pendientes de diseño](reviews/HTML-static-dom-plan-r2.md). HD-R1-01 cerrado para contrato de estado readonly/clonado; 20 fixtures originales y tres suplementos conservados. DOM/raw/lectura separados, companion opt-in y recursos explícitos. ResponseLease concreto, lock/decoder/sink/adapter/HTTP/history/runtime/QA/Unix siguen pendientes; spike registro Nu cerrado no se repite. RV-401/402/403/404 abiertas.


2026-10-10: [diseño de integración PDF R1](refinement/PDF-PRODUCT-INTEGRATION.md) con [revisión independiente sin críticas pendientes de diseño](reviews/PDF-product-integration-plan-r1.md). Reutilización RustR4/UTF16R2 delimitada; supervisor Rust, snapshot binario, raster/text separados, lazy/zoom/búsqueda de ocurrencias/links/bookmarks/cache/history/inline conservan alcance completo.18 criterios/26 familias prospectivas, identidades contrastadas; freeze UTF16 recuperado1112 entradas sin nuevo runtime. P1 necesita wire/ledger técnico y fixtures exactos antes código; ningún RV/QA/Unix cerrado.

2026-10-10: [SYN-CLEANUP-SAMECLIENT01 observado y abierto](reviews/Syntax-same-client-cleanup-observation.md). En D4b, Pending físico bloquea nuevos workers pero el controller termina y el mismo SyntaxClient no procesa retry_cleanup; un FAIL primario y cuatro posteriores por retención global, ejecución serial. Corrección privada en curso, sin ampliar cuotas/plazos ni rescatar con otro cliente. Requiere revisión del delta D4a afectado y AE física de recuperación en el mismo cliente; producer-only PASS no cierra D4b ni RV/QA.

2026-10-10: [ResponseLease HTML R3 incorporado](reviews/HTML-response-lease-adoption-r3.md), tras [dictamen final independiente de diseño](reviews/HTML-response-lease-plan-r3.md). RL-R1-01 y RL-R2-02 resueltas: owner conjunto worker/reply con Pending hasta cierre físico; callback nominal consumible Rust estable y preadmisión de cuerpos/capturas/returns. Doce familias propias y29oráculos prospectivos; integridad24controles adversario/28raíz PASS, cero runtime. SDK/companion/decoder/parser/export/HTTP/QA/Unix siguen pendientes; registro Nu cerrado intacto y RV401–404 abiertas.

2026-10-10: SYN-CLEANUP-SAMECLIENT01 sigue abierto. El candidato privado R1 supera controles físicos focales, pero revisión independiente confirma SC-R1-01: retry desde otro Shared puede recoger el owner global y dejar bloqueado al propietario original vivo. [Evidencia y criterio](reviews/Syntax-same-client-cleanup-observation.md). Corrección de ownership y regresiones del siguiente freeze pendientes; ningún código, D4b, RV ni QA aceptados.

2026-10-10: [SYN-CLEANUP-SAMECLIENT01 y SC-R1-01 corregidos e integrados R4](reviews/Syntax-same-client-cleanup-adoption-r4.md), tras aceptación independiente de código y worker real. Controller original conserva retry; propietario extranjero vivo rechazado y orphan actualiza sólo Shared original. Cuotas5040 exact/menos uno y cargos hasta último Drop. Composición canónica:14 controles normales y4 nativos PASS, fmt/Clippy workspace/all-targets PASS. Fallos previos preservados; arranque frío no garantizado. Cierre sólo de estos defectos; D4b, sesión, RV y QA abiertos.

2026-10-10: [diseño PDF P1 actor R2 incorporado](reviews/PDF-P1-ACTOR-adoption-r2.md), con revisión independiente sin críticas de diseño. Préstamo físico con guard, control público separado, registry linealizado y D0–D3 resuelven cinco hallazgos R1; mismo cliente Syntax requiere reap/nuevo Init al cambiar input/contexto. D1 compilado, wire R4, D2/AE, P1 productivo y P2–P5 pendientes; ledger OPEN_COMPILE, oráculos NOT_RUN, RV204/QA/Unix abiertos.

2026-10-10: [diseño wire PDF R5 incorporado](reviews/PDF-P1-WIRE-adoption-r5.md), revisión independiente D0 sin críticas,15 identidades físicas PASS. Framing/receipts/metadata/geometría y alfa straight BGRA4 fijados; PDF-WIRE-ALPHA01 resuelto documentalmente, no runtime. Actor R2+wire R5 habilitan D1 real/inventario antes candidato D2/AE y D3; P1 productivo, P2–P5, RV204/QA/Unix pendientes.

2026-10-10: candidato privado D4b producer R1 no adoptado. AE independiente confirma D4B-R1-W01 (Styles del worker real permiten CachedCodeData/proof con Work0) y D4B-R1-A01 (pump con autoridad ajena Idle borra correlación de la preparación original todavía físicamente Running). Dos FAIL conservados; el segundo no demuestra Reaped ni liberación del kernel. Freeze E889392CCFA8AE1B54C926762DF37FFF31A703358CB281DF260128D0932C7D5E verificado916/916 por raíz; log independiente4D4AED4FBFB4AB6075D53C77D4A1CA619D235922897016273D2309A037230F93 e informe inicial44C8D3224E6AF4B91725D1C8340D8C4EE450CEF00EE7DAD2E6389C89D429E083 permanecen privados en .tmp/d4b-scheduler-adversary-2026-10-10. Corrección R2 y revisión restantes pendientes: preadmisión SAME Work antes metadatos/cache/proof y guard de autoridad física original antes mutaciones. No ampliar cuotas/plazos ni cambiar oráculos. Sintaxis focal3c7eabc permanece aceptada por su alcance; D4b/D2c/D4c, RV y QA abiertos.

2026-10-10: suplemento independiente D4B-R1-T01 confirmado para el mismo candidato privado R1. La factory de color acepta Document/operación sin una ColorPreparationAdmission privada derivada de T/ventana/render y cargos mandatory efectivos; FrameSeal posterior sólo protege activación. CodeAE más alcanzabilidad de Init/Styles ya observada, no tercer assert nativo nuevo. DS05/O02/O08/O19 eran criterios anteriores al candidato; suplemento6C3C086473F8E8BFF896B4DC3C09F020EE7A45B49A2ADCD2A6A7DD5A50011B39 permanece privado. Corrección coordinada con D2c: permiso real ligado a Document/Consumer/operation/binding antes reserva/scan/spawn; absent/stale/foreign devuelve AwaitText o causa equivalente y conserva T. Builder puro separado no acredita UIpriority. W01/A01/T01 siguen abiertos hasta freeze y aceptación R2; sin cuotas/plazos nuevos ni cierre RV/QA.

2026-10-10: suplemento independiente D4B-R1-C01 confirmado por CodeAE del freeze producer R1: falta contexto host cerrado Inline/Tui con comienzo y plazo original acotados antes demandas/preparación. Conservar OperationContext D4a no impone por sí solo la política D4b; los positivos voluntarios de5s no prueban el rechazo de plazos arbitrarios. DS08/O11/O17 anteriores al candidato; suplemento privado639492616C23992982DEB874A422DD225FE0EBF02AF5B185FB86C377BD8A263A. Corrección R2 requiere wrapper checked que conserve SAMEid/cancel/Instant y modo en binding, correlación, cache y activación; supersede/resize no renuevan el plazo. No reabrir D4a ni diferir el contrato a D4c. W01/A01/T01/C01 abiertos, sin nuevo runtime para este hallazgo, adopción ni cierre RV/QA.

2026-10-10: [inventario físico de avisos Cargo Windows](research/DEPENDENCIAS_AVISOS_WINDOWS.md):275paquetes/458archivos identificados, ocho paquetes requieren revisar avisos u origen de la licencia. Metadata/lock byteexactos; sin descarga ni cambios de dependencias. Investigación de archivos por nombre, sin acreditar atribuciones finales, artefactos distribuidos, assets de motores, Unix ni cierre RV501.

2026-10-10: [revisión independiente del inventario de avisos Windows](reviews/Dependencias-avisos-Windows-r1.md) sin críticas pendientes de exactitud/procedencia:275paquetes/458archivos/ocho ausencias reconstruidos y7capturas validadas contra commits/árboles/blobs Git vivos. Fuente original de6upstream identificada; mcy y omnipath conservan pendientes de copyright nominal sin completar por inferencia. Cierre de investigación limitada, no bundle/avisos finales, Unix ni RV501.

2026-10-10: candidato privado D4b producer R2 no adoptado. Freeze9CF0765256344B0D23D9AE84028F2AC747FDA291440D4EF1B4939B086196AD3A verificado1009/1009 por raíz;839fuentes y45deltas (9D4b,34dependencias D2c,2Syntax). La base fuente canónica3c7eabc no tiene deriva Rust en930e84d. AE independiente reproduce D4B-R1-W01 residual en el callback Ready→Set/request: con SAME SessionOperation agotada8388608 y snapshot real físicamente joined, prepared devuelveOk, sequence0→1 y WorkerBinding=true sin gasto nuevo. No demuestra CreateProcess/Styles posteriores; shutdown del control pudo impedirlos. LogD7F6026658EF059D0E389AA4ACD42D969E7E7859AB14CB07B0D3E04EAE064D82 y preidentidades propias permanecen privados. Autor confirma omisión de Work, aunque Consumer/D4a financien memoria; corrección R3 separada y aceptación pendientes, sin aumentar cuotas/plazos.

R2 dispone de controles independientes positivos limitados de autoridad ajena, capability CURRENT/origen y modos/plazos, producer/cache DF03 y recepción Styles0/1559/1560. La subfrontera poll_event1560 está acreditada por esos controles, pero no cierra W01 completo ni toda la matriz DS/O. R3 conserva los oráculos de operación agotada y coste exacto/menos uno antes de mutar generación, clonar o iniciar solicitud. Sus controles del autor y Clippy no son aprobación independiente. D4b, composición completa D2c, routing D3c, provider inline real y D4c, gráficos, RV y QA/Unix permanecen abiertos; no adoptar las34dependencias D2c por aceptación aislada del scheduler.

2026-10-10: candidato privado D4b R3 congelado, aún no adoptado. Manifest4D7EF74D7E3385A1D41848979F122CA611B31EFB58F2F0ACCF80CEF8E919D555 verificado1039/1039 por raíz;839fuentes, sólo cuatro cambian frenteR2 (scheduler, bridge, tests e inventario Syntax). Recibo raízB83559DF4A372C6EFCA8AC89AC61D7E7C932C2F245113E180A88877444A46517. AE independiente del callback con construcción/receipt real y SAME SessionOperation pasa0/2375/2376: rechazo conserva completion/generación y gasto;2376 consume el coste compilado exacto y crea WorkerBinding. LogF635F7784D6A89F3F50112BBA260D9633FC1B548CD835E68F3E94021BB0E23C5; cuatro identidades propias de fuente/inventario/executor/worker contrastadas por raíz. No acredita Init/Styles del caso exacto ni cierra W01/matriz completa; bridge y restantes controles siguen en revisión. R1/R2 y primer fallo del harness R3 se conservan sin relabel.

La afirmación global de fmt PASS del README privado R3 era incorrecta: el log muestra diffs en19fuentes heredadas y PowerShell continuó sin capturar ese exit. Autor confirmó el error; errata externa9F1551E00C80E56E5DEE88288D577017285AA9B23468840A55F4C2B5B45F1D8F mantiene freeze intacto. Check de los cuatro paths del delta exit0 no sustituye fmt workspace requerido para integración final. No reformatear/adoptar dependencias D2c para cerrar este gate por asociación; D4b/D2c/D4c, RV y QA/Unix permanecen abiertos.

2026-10-10: revisión independiente final D4b R3 rechazada por D4B-R3-I01/P01. Informe privado65F8B874F3BB39FC761175A0D31BD6381089CB5A5588832FCD0604F680C98DA8 y manifest49A14DD1B0F713AE998AF184D4DAB4E3024628A515842FDDB9E94CD9FBD4F5AB; raíz contrastó90/90 evidencias bytes/SHA. I01: igualdad de batch omite demandas/color y descarta viewport nuevo; reutilización dentro del mismo batch no acredita supersede productivo O06/DS12. P01: factory durante SAMEphase ocupada devuelve OperationBudget antes submit y no retiene nueva intención; coalescing de batches prefinanciados prueba sólo subfamilia O08. Originales DS01–20/O01–20/DF01–08/DM01 y todos los fallos conservados. W01 residual corregido por callback real0/2375/2376 y bridge0; Clippyalltargets propio exit0 y regresión9PASS/25ignored no cierran ingreso completo, fmtglobal, D2c/D4c/RV/QA. R4 debe financiar comparación y supersede en fase original, retener evento nuevo mediante ownership real durante espera, revalidar CURRENT y conservar deadline/cancel; sin adopción R3.

Sesión Markdown privada: panel6 conserva discrepancia de fidelidad tras corregir captura nativa a CHAR_INFO pareados: modelo contiene two.md y consola devuelve twomd. FAIL y forcedcleanup preservados; reducción coldSource positiva no cierra secuencia original Inspector→Source. Recibo raíz44E3F6BFE5D7DF2DE5FEEFA411DE9C257CFEB219C07E0150EEB028855B9FC24E. Bytes reales del writer y backend ratatui-crossterm0.1.2 permiten investigar continuidad de cursor después de símbolo combinado y celda omitida por diff; hipótesis causal, aún sin corrección aceptada ni QA visual. Fuente canónica sin cambios; D2c completo pendiente.

2026-10-10: [diseño focal R4 de ingreso](refinement/MARKDOWN-D4B-INGRESS-R4.md) revisado [sin críticas pendientes de diseño](reviews/Markdown-D4b-ingress-design-r4.md), fuentes documentales exactas2C7167675EFA2FE2CCA9F259D111FDA5D4C4CC7071C97FD7EC4DF36F2017D1B7/335887DB2593C8E613D3051613F2F00F4753F889E75353B78BFF4894B26B2E97. Retiene evento nuevo en lector financiado durante espera, préstamo de latest y leases a través del draw mandatory, identidad original antes control, señal accepted-sequence y ack ordenado, CURRENT posterior a flush e historial genuinos. Handoff lector8/8 exacto y cargo conocido6468, sin ampliar cuotas; bin del autor pertenece a fuente de sesión distinta y no acredita composición focal. Implementación y aceptación productiva pendientes: I01/P01 siguen abiertos; diseño no cierra D2c/D4c/RV ni QA.

2026-10-10: [avisos de gramáticas fijadas](research/AVISOS_GRAMATICAS.md) con [revisión independiente R2](reviews/Gramaticas-avisos-r2.md):74/74 fuentes y2/2 fuentes del loader/generador cotejadas; avisos C#/YAML identificados y falsos positivos clasificados. Investigación documental aceptada en ese alcance, sin ejecución del generador ni cierre del linaje del dump, bundle o RV-501.
