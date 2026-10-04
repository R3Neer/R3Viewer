# R3Viewer: contratos para preparar el backlog

Fecha: 2026-10-04. Especificación de análisis, sin implementación autorizada.

## Estado y autoridad

REQUISITOS.md y DECISIONES.md recogen decisiones confirmadas. Este documento concreta recomendaciones operativas y señala lo pendiente. El backlog no autoriza implementar, instalar, publicar ni elegir implícitamente una opción pendiente. Crear Git y commits locales fue autorizado por encargo posterior.

## Entrada: Q03 confirmada

Confirmado por el usuario: argumento posicional local abre archivo/carpeta; URL HTTP(S) explícita abre recurso remoto; pipeline contiene un valor. --path/-p fuerza ruta local; --value/-v fuerza valor literal. Flags incompatibles. Propuestas de validación: un solo origen, rechazar argumento más pipeline no vacío y múltiples argumentos. Una tabla/lista es un valor, no varios archivos. No detectar rutas examinando si una cadena de pipeline coincide con un archivo existente.

Sin entrada: diagnóstico y ejemplo de uso; no abrir cwd implícitamente. Formato explícito debe poder resolver texto ambiguo; --format es nombre propuesto. Una URL de pipeline es texto salvo modalidad explícita de origen remoto, cuyo contrato está pendiente. No ampliar --path a URLs sin cambiar su definición.

## Salida y presentaciones

Confirmado: interactivo por defecto; --inline/-i imprime y termina. Redirección produce texto legible sin escapes ni gráficos. Propuesta: redirección de la salida del modo por defecto activa salida textual, comunicando diagnóstico solo por stderr cuando sea necesario; no intentar TUI sin terminal utilizable. La falta de entrada TTY con stdout TTY requiere una solución de acceso al terminal investigada antes de fijar arquitectura.

Separar stdout de contenido y stderr de diagnósticos. Salida normal por teclado y cancelación no deben devolver selecciones implícitas al pipeline. No emitir éxito decorativo. Códigos propuestos: 0 salida normal; distinto de 0 para fallo fatal de entrada/configuración; cancelación y documento parcialmente legible requieren contrato final documentado. No convertir un error local de diagrama en fallo total por defecto.

## Documento común

Modelo semántico propuesto: bloques e inline, estilos/roles, tablas, código, enlaces, identificadores y referencias a recursos gráficos. Mantener origen, base de resolución y posición navegable sin mezclar secuencias ANSI con contenido. Los datos Nushell requieren modelo tipado propio; no convertirlos universalmente en Markdown ni JSON sin estudiar pérdida de tipos.

Fuente HTML, árbol HTML y documento visual son representaciones distintas. Q25 confirma incluir from html como utilidad opcional. Su modelo propuesto conserva nodos de texto, orden y atributos; el lector filtra lo no visual. La fuente original nunca se reconstruye desde el DOM para presentarla como raw.

## Navegación y recursos

Confirmado: búsqueda, índice, historial con posición, ampliación y páginas PDF bajo demanda. Propuesta: grafo de historial conserva documento, destino y posición; resolver destino antes de sustituir vista. Si falla, conservar la vista de origen y explicar el fallo. PDF #page=N significa página física 1-based; rechazar fuera de rango sin abrir otra silenciosamente.

Obsidian: raíz detectada o explícita; distinguir nombre de nota, alias mostrado, encabezado y bloque. Indexar nombres y presentar candidatos ante ambigüedad. Los ciclos de incrustación se cortan con referencia visible. La profundidad máxima y límites de caché quedan como parámetros técnicos a medir.

Confirmado: acceso remoto explícito. Propuesta: URL como argumento autoriza obtener ese documento; recursos secundarios no se descargan automáticamente. Interactivo ofrece acción de carga; inline muestra referencias de recursos pendientes. Flag de carga agrupada y persistencia por dominio no definidos. Mantener base de URL final y respetar base HTML al resolver enlaces. No ejecutar scripts, archivos ni comandos al seguir enlaces.

## Controles HTML: Q24 confirmada

Confirmado: mostrar estado de casillas y controles, sin modificarlos ni enviar formularios. Navegación por enlaces y expansión de details son propuestas sobre la lectura. Marcas temporales locales y envío de formularios quedan fuera del alcance inicial.

## Datos, directorios y presentación

Confirmado: archivo de datos abre original resaltado, con estructura opcional; valor de pipeline abre estructura recibida. No reabrir origen después de una transformación. Inspección y búsqueda, sin duplicar lenguaje de filtros Nushell.

Propuesta: carpetas cargan nivel actual, no calculan tamaños recursivos por defecto. Reconocer salida Show-Tree solo mediante contrato de datos/metadatos, sin analizar su árbol dibujado. Snapshots no reescanean archivos para reemplazar el contenido recibido. Binarios tienen offsets y bytes sin fingir comprensión.

Diseño confirmado: discreto y coherente con Nushell. Propuesta R3CLI: roles de estado y textos auxiliares, sin banners ni marcos ornamentales; mapping explícito de color_config y fallback. El compositor TUI mantiene control del dibujado.

## Instalación, compatibilidad y límites

Confirmado: distribución completa, Windows nativo, Windows Terminal en Windows y WezTerm en los tres sistemas. Publicar versiones/arquitecturas verificadas, todavía pendientes. No asumir compatibilidad universal por detectar un protocolo.

Recomendado: dependencias internas sustituibles y versionadas, perfil completo por defecto; texto no necesita inicializar navegador ni motores gráficos. No instalar dependencias durante una apertura. Inventario de licencias, tamaños y actualizaciones antes de empaquetar.

Entradas finitas inicialmente; no prometer detectar infinitud. Cancelación disponible durante lectura/renderizado. Elegir límites de bytes/nodos/páginas, memoria y caché después de pruebas representativas; truncamiento siempre visible. Cache fuera de documentos y bóvedas, con limpieza y clave por contenido/opciones/versiones.

## Pendientes que no se trasladan al usuario como elecciones de tecnología

Lenguaje, protocolo Nu, acceso al TTY, APIs, bibliotecas, licencias, versiones, métricas y códigos exactos: decisiones técnicas con investigación y alternativas justificadas. Preguntar al usuario cuando cambien instalación, cobertura, comportamiento o experiencia prometida.

## Resolución técnica en preparación del repositorio

Fecha: 2026-10-04. Véanse ../architecture/ARQUITECTURA.md y ../research/INVESTIGACION_TECNICA.md. Se adoptan como decisiones de trabajo: --format, origen único, ausencia de entrada como error, salida textual automática cuando stdout no es terminal; códigos 0 normal, 1 fatal IO/render, 2 CLI/config y 130 cancelación (propagación por Nu por validar). Documento parcialmente legible mantiene 0 con diagnóstico stderr; limitación que impide completar entrada es fatal, nunca truncamiento oculto.

Navegación por URL de pipeline: sigue siendo texto. Para abrirla, pasar explícitamente como argumento, por ejemplo viewer ($url); --path se mantiene local. --value interpreta la cadena como contenido y --format indica HTML/Markdown si hace falta. El comando no adivina que una cadena parece URL o existe como ruta al recibirla por pipe.

Datos finitos se transportan por frames limitados, lectura cancelable. Los límites alcanzados no eligen datos silenciosamente: diagnóstico y cancelación/error, o selección de ventana explícita. Cancelar durante una carga conserva vista anterior cuando exista.

from html se importa desde módulo opcional; la instalación del encoder no exporta ese conversor silenciosamente. Su proyección --flat es una capacidad posterior opcional, no necesaria para devolver árbol completo. El parser comparte representación y origen con adaptador HTML; metadatos que no sobreviven en pipeline requieren base explícita --base, propuesta de nombre.

PDF con texto permite búsqueda y salto; highlight exacto depende de validación de bounding boxes. PDF escaneado sigue visible, búsqueda comunica ausencia de texto. Selección/copia exacta y OCR no prometidos. SVG/imágenes animadas muestran representación estática inicial. No ejecutar acciones de PDF, scripts HTML o código de Markdown.

Búsqueda v1: texto literal con siguiente/anterior y estado sin coincidencias; contenido textual del documento completo dentro de límites; PDF indexado por página bajo demanda. No prometer regex, OCR ni búsqueda de imágenes. Búsqueda profunda en datos tiene límites visibles y no evalúa valores ejecutables.

Configuración v1: archivo TOML por usuario en ruta estándar del SO; flags prevalecen sobre config, luego entorno y detección. NO_COLOR se respeta salvo override explícito; --graphics auto|sixel|kitty|iterm2|symbols|none propuesto. Teclado funcional sin ratón y ayuda de teclas en sesión; no vincular todas las acciones al ratón.

## Refinamiento por sprint

La arquitectura general vigente es ../architecture/ARQUITECTURA.md. Las decisiones funcionales confirmadas se mantienen. Nombres/UX/API/límites técnicos propuestos en este documento se refinan por sprint y no se consideran contratos implementados por aprobar el plan macro. Transporte, recursos, caché y terminal conservan validaciones por capacidad.
