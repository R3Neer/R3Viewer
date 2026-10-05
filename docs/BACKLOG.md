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
