# Arquitectura de referencia

Fecha: 2026-10-04. Decisiones técnicas de trabajo para implementación futura. El gate RV-008 valida o sustituye las decisiones antes de ampliar el desarrollo. Evidencia en ../research/INVESTIGACION_TECNICA.md.

## ADR-001: núcleo externo Rust y puente tipado Nu

Módulo público Nu `viewer` valida argumentos, recoge configuración visual y metadatos y lanza `r3viewer` externo. Un compañero `nu_plugin_r3viewer` convierte tipos a frames propios; no renderiza ni mezcla protocolo y escapes. El núcleo no necesita Nu para leer rutas/URLs, aunque Nushell es la integración principal.

Contrato transporte v1 propuesto: cabecera magic/version, frame de origen y capacidades, frames de valores etiquetados, fin y errores. MessagePack con longitud prefijada como candidato; límites por frame, límites totales y backpressure. Record conserva orden; int i64, float f64 con representación de no finitos, duration ns y filesize bytes sin pasar por float; binary bytes, date ISO y offset; listas y registros anidados. Ausencia = clave inexistente, nothing = nodo explícito, string vacío = nodo string. No serializar comandos para ejecutarlos. Custom values opacos/base identificados, no coerción silenciosa.

Parser y protocolo versionados separadamente de nu-plugin. El compañero se compila para versión Nu comprobada y diagnostica incompatibilidad antes de abrir TUI. Módulo y binarios se distribuyen juntos. El usuario ve un solo comando viewer; no tiene que invocar encoder manualmente.

Alternativa documentada: NUON reduce puente, pero no resuelve valores especiales universalmente ni los requisitos de streaming. Si el spike muestra que plugin impide cancelación/lifecycle adecuados, revisar ADR sin perder tipos prometidos.

## ADR-002: separar contenido, presentación y terminal

ContentSource: archivo, URL o valor tipado. Provenance: ruta/URL original y final, base, raíz de bóveda y metadatos permitidos. Adapters generan DocumentModel o DataModel. Resolver produce LinkTarget/ResourceRequest. Layout produce regiones de texto/gráfico con métricas y posiciones estables. InlineRenderer imprime scrollback; InteractiveRenderer gestiona ratatui, búsqueda/historial y regiones visibles.

TerminalSession posee teclado, probes y restauración. stdin se reserva a payload si hay pipe. Con stdout redirigido nunca abre CONOUT$/TTY para sortear redirección. Cleanup de raw mode/pantalla/cursor es idempotente y no depende únicamente de salida feliz. Recursos largos operan en workers cancelables fuera del loop de UI.

R3CLI aporta roles/recursos de presentación discreta en Nu y mapping explícito en núcleo; no utilizar print desde componentes del compositor. No sustituir tablas tipadas por cadenas decoradas.

## ADR-003: modelos compartidos sin pérdida de fuente

Documento = bloques, runs inline, tablas y recursos con span/origen. Datos Nu = árbol tipado. HTML DOM = árbol estructural de parser con texto mixto; from html devuelve modelo estructural, viewer construye documento visual. Ninguno reconstruye fuente original para raw. Obsidian extiende AST Markdown antes de resolver y compone embeds sin regex indiscriminadas.

## ADR-004: motores y proceso aislado

Ratatui/Crossterm/ratatui-image para UI y gráficos TUI; inline comparte negociación/encoders cuando sea posible con posicionamiento propio. PDFium privado en worker para render y texto, con ABI fijado; MuPDF alternativa no seleccionada. Mermaid oficial a PNG cuando SVG no sea autocontenido; MathJax SVG con métricas y resvg; image/syntect/html5ever/pulldown-cmark en núcleo. Las versiones exactas se fijan al demostrar build conjunto, no con latest ficticio.

No activar chafa-dyn accidental mediante default features. Chafa puede seguir siendo alternativa si el backend elegido no cumple calidad inline. Ningún gráfico externo pasa al terminal hasta decodificar, limitar tamaño y establecer área.

## ADR-005: remoto y búsqueda PDF

HTTP(S) explícito como origen; tipo real dirige adaptador. Acción individual de recurso en TUI; opción --load-remote propuesta para carga agrupada por invocación; sin ella inline deja referencia. HTTP remoto no accede a archivos locales ni ejecuta scripts. Autenticación/sesiones de navegador fuera de v1; cabeceras manuales aún no son contrato público.

Búsqueda PDF incluida cuando haya texto extraíble: buscar, indicar página y saltar al resultado; highlight si mapping de bounding boxes se verifica. OCR y selección/copia exacta con maquetación quedan fuera de garantía inicial. PDF con password pide mecanismo que no se registre en historial/log; detalle UX antes de RV-204, no omitir silenciosamente documento protegido.

## Organización prevista de implementación

```text
crates/core/           modelos, resolución, adaptadores y layout
crates/viewer/         CLI y presentaciones
crates/nu-plugin/      transporte tipado y conversor from html
crates/render-worker/  aislamiento PDF y operaciones nativas
nushell/              módulo público viewer y utilidades opt-in
workers/              Mermaid y matemáticas
packaging/            bundles/manifiestos por plataforma
tests/fixtures/       corpus sintético no privado
```

Es propuesta, no directorios con código ficticio. from html se exportará de forma opt-in; registrar el plugin encoder no debe cambiar silenciosamente open HTML. Definir command interno de plugin y wrapper `from html` en módulo opcional es la vía preferida.

## Consecuencias

Completo por defecto, múltiples componentes privados; tamaño de Chromium domina paquete, por decisión de cobertura Mermaid. Updates de Nu pueden requerir companion nuevo. Ratatui no convierte por sí mismo explore nativo en lector gráfico. Terminología pública no expone detalles de IPC. QA gráfica y distribución multiplataforma son trabajo sustancial aunque la viabilidad documental sea positiva.
