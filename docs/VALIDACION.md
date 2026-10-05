# Plan de validación y preparación

Fecha inicial: 2026-10-04. Implementación y pruebas privadas autorizadas posteriormente. S01/S02 tienen evidencia funcional Windows; S03 en validación. Esta matriz conserva los criterios pendientes y no anuncia certificación de plataformas por compilar.

## RV-008: validaciones por capacidad dentro de los sprints

| Caso | Evidencia necesaria |
|---|---|
| Nu activo y companion | Ejecutar version del binario real, registrar ruta, build plugin compatible; incompatibilidad debe diagnosticarse. |
| stdin ocupado | ls y valores anidados pasan por encoder; teclado y probes leen terminal separado, salida final y Ctrl-C restauran sesión. |
| tipos | Corpus nothing/bool/int/float/date/filesize/duration/binary/list/record/range/cellpath/error/custom; sin coerción silenciosa; no ejecuta closures. |
| TUI e inline gráfico | Misma imagen/fórmula en Windows Terminal y WezTerm; scroll/resize/zoom/cleanup visual; protocolo, release y DPI registrados. |
| PDFium | ABI privado carga, páginas 1/N y texto; proceso cancelado sin cerrar lector; fuentes/notices inventariados. |
| Mermaid/math | Runtime privado frío, PNG/SVG autosuficiente, baseline inline, errores locales y cancelación; no iniciar browser para texto. |
| HTML converter opt-in | open y http get convierten con módulo activado; -r conserva fuente; sin activación mantienen comportamiento nativo. |

Cada spike se selecciona y refina según las historias que necesitan esa capacidad. No deben terminar todos para leer texto/archivos ni actúan como fase previa global. Un fallo cambia la elección técnica o el trabajo dependiente antes de continuar con esa capacidad; no reduce alcance confirmado sin acuerdo. No marcar runtime soportado por revisión de fuente solamente.

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

## Relación con el plan general

La arquitectura general de architecture/ARQUITECTURA.md fue revisada iterativamente hasta SIN_CRITICAS_PENDIENTES. Su alcance es dominios, relaciones y épicas. Esta validación es un catálogo inicial para refinar por sprint, no APIs finales ni orden completo de implementación.

Puente Nu corresponde a E02; gráficos a E04; PDF a E05; formatos/Obsidian/HTML a E06–E08; compatibilidad/calidad a E10 y a cada historia afectada. Los criterios específicos y versiones se concretan al seleccionar el trabajo. Sin ejecutar QA no anunciar plataforma/capacidad acreditada. Licencia MIT registrada; revisar avisos de dependencias al distribuir.
## Evidencia actual (2026-10-05)

S01/S02: lectura, bridge tipado, cancelación, restitución de modos y siguiente invocación ejecutados en Windows con Nu0.115.1; referencias exactas en sprints/S01.md y S02.md. S03A/B: revisión adversaria R2 sin críticas y86pruebas; AE detecta teclado AltGr/Alt-code y override NO_COLOR, que requieren candidato R3. cargo check -p r3viewer --target x86_64-unknown-linux-gnu --locked pasa para A/B, sin ejecución Linux.

ConPTY propia es prueba funcional de procesos/teclas/modos. No sustituye revisión visual Windows Terminal/WezTerm. Gráficos, PDF, Markdown/HTML/Obsidian, distribución completa y ejecución de la matriz siguen pendientes. Los modelos y snapshots de widgets no certifican fuentes/DPI/protocolos reales.


Actualización R4 A/B: revisión independiente sin críticas y aceptación funcional Windows; 89 pruebas workspace y dos pruebas parser controlado, formato/Clippy/build. Compilación Linux pasa; advertencia cosmética upstream de paréntesis en Crossterm Unix, sin cambio de su código. Las validaciones de apariencia, ejecución macOS/Linux, resaltado, demás adaptadores y distribución conservan estado pendiente.
