# Plan de validación y preparación

Fecha: 2026-10-04. Ninguna prueba runtime de R3Viewer ejecutada todavía. Documentación y repositorio se verifican ahora; pruebas siguientes requieren implementación/prototipo autorizado.

## Gate RV-008 antes de desarrollo amplio

| Caso | Evidencia necesaria |
|---|---|
| Nu activo y companion | Ejecutar version del binario real, registrar ruta, build plugin compatible; incompatibilidad debe diagnosticarse. |
| stdin ocupado | ls y valores anidados pasan por encoder; teclado y probes leen terminal separado, salida final y Ctrl-C restauran sesión. |
| tipos | Corpus nothing/bool/int/float/date/filesize/duration/binary/list/record/range/cellpath/error/custom; sin coerción silenciosa; no ejecuta closures. |
| TUI e inline gráfico | Misma imagen/fórmula en Windows Terminal y WezTerm; scroll/resize/zoom/cleanup visual; protocolo, release y DPI registrados. |
| PDFium | ABI privado carga, páginas 1/N y texto; proceso cancelado sin cerrar lector; fuentes/notices inventariados. |
| Mermaid/math | Runtime privado frío, PNG/SVG autosuficiente, baseline inline, errores locales y cancelación; no iniciar browser para texto. |
| HTML converter opt-in | open y http get convierten con módulo activado; -r conserva fuente; sin activación mantienen comportamiento nativo. |

Un fallo de gate cambia arquitectura/alcance documentado antes de seguir. No marcar runtime soportado por revisión de fuente solamente.

## Corpus propio

Generar fixtures sintéticos futuros: Markdown con tablas, bloques código, mermaid y math; vault pequeño con basename duplicado, headings, blocks, aliases y ciclo; PDF con texto, escaneado y páginas de distinto tamaño; HTML mal formado, entidades, enlaces relativos/base y controles; tabla Nu anidada con clave ausente frente a nothing; binario grande y directorio con acceso denegado. No incorporar material académico privado al repo.

Caso externo Clases se valida posteriormente de solo lectura bajo sus instrucciones, sin usar cambios no confirmados como baseline ni copiar PDFs a Git.

## Matriz candidata

Windows x86_64 Windows Terminal + WezTerm; macOS arm64 WezTerm; Linux x86_64 WezTerm. Releases exactas se fijan al ejecutar QA. El target no equivale a una certificación actual. Otras arquitecturas deben revisar PDFium, Chromium y toolchain antes de ofrecer soporte. Multiplexores/SSH por mejor esfuerzo inicialmente.

## Checks pertinentes por cambio

- Documentación: diff --check, vínculos relativos, coherencia de decisiones/estados y ausencia de rutas temporales como destino canónico.
- Código futuro: tests del comportamiento observable y límites; no espejar implementación. Rust fmt/clippy/test y pruebas Nu en versión objetivo cuando existan fuentes reales.
- Gráficos: capturas y QA manual, no solo snapshots de texto.
- Packaging: instalación limpia por sistema, notices/hashes, entorno sin Node/Rust global, actualizaciones/reversión de integración y uninstall propio.

## Criterio de preparación inicial

Repositorio local inicializado, archivos canónicos migrados, política de commits vigente, contratos de entrada y casos de error definidos, decisiones técnicas motivadas, backlog con dependencias y gates de ejecución, y licencia registrada o explícitamente pendiente. Esto no declara terminados H0 runtime ni H1–H5.
