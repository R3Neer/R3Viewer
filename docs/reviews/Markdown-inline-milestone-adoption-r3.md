# Markdown inline: integración del hito, 2026-10-06

Se incorpora `viewer -i` para Markdown: selección automática de archivos reales .md/.markdown y `--format markdown` para texto literal/de pipeline. Los valores Nu tipados conservan su inspección; pedir Markdown para un registro produce error2 sin stdout. La salida redirigida es plana. El documento se prepara completo antes de escribir; la limpieza pendiente del worker impide publicar Documento o Fuente.

[Revisión y aceptación independientes](Markdown-inline-caller-acceptance-r3.md): SIN_CRITICAS_PENDIENTES en caller y routing inline. Siete invocaciones CLI y cinco grupos propios de caller/nativo PASS; cancel130, causa sticky/retención física, Source completa sólo tras cierre y ausencia de reap/retry implícito. Interactivo sólo tiene revisión estática de su aviso, no QA terminal.

## Composición final

Basee054595; se conserva el algoritmo Unicode ResolvedUnit y todas las dependencias/cuotas compartidas. Se añaden selector, caller, registro de módulos, ruta main y utilidad Source fallback crate-visible; cambios del controller son observación/inyección cfgtest y tests de ciclo de vida, sin alterar cierre productivo. Lib actual se conserva y sólo incorpora los dos módulos, ordenados por fmt.

Root verificó687 registros R1,15 overlayR2,8 receiptR3 y34 de aceptación independiente. La evidencia R2 contenía un Diff antiguo en el log fmt debido a Tee-Object sin salida; fuente/binario ya estaban formateados. R3 e independiente confirman exit0 con log vacío; fallo documental preservado. La primera guardia de integración Cargo byteexacta se detuvo antes de escribir por LF de git archive frente CRLF de working; contenido normalizado idéntico, archivos de dependencias sin modificación.

Composición final privada de fuentes que se incorporan:38 pruebas PASS (8caller,9lifecycle nativas,5selector,16inline), fmt y Clippy todos los targets -DwarningsPASS, buildPASS. Paquete privado dentro del repo: `.tmp/markdown-inline-milestone`, con ejecutable y companion; cinco invocaciones reales de Nushell0.115.1 con `-n --plugins` temporal PASS: archivoMarkdown, String explícitaMarkdown, Stringauto literal, registroauto y registroMarkdown rechazado2/stdout0. No instalación, registro de plugins ni edición de perfiles.

## Límites

Hito textual inline. Recursos gráficos, Mermaid, matemáticas y resolución completa de enlaces Obsidian siguen pendientes. Fences aún no tienen el renderer Syntect de Documento. Markdown interactivo muestra Fuente con diagnóstico explícito; renderer/pump documental y QA visual/Unix no están aceptados. RV-301 y el objetivo completo continúan pendientes. El hito permite pausar el trabajo a petición del usuario sin declarar terminado el producto.
