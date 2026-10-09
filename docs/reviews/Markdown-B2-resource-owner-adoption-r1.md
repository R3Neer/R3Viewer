# Corregir el bloque contenedor de recursos Markdown

2026-10-10. Integración sobre `39e4931`; código anterior `3db799c`. B2-RESOURCE-OWNER01 podía terminar la preparación con Panic al presentar tres imágenes: ResourceAlt.owner identifica un recurso y se usaba como índice de bloque. Si ese identificador coincidía con Code o FrontMatter ajenos, el desplazamiento horizontal también podía ocultar el texto alternativo.

El layout valida el bloque contenedor real y calcula su política horizontal una vez por secuencia. La lectura añade sizeof<Block>() al trabajo antes de preparar o publicar; el registro actual tiene 64 bytes. El coste total del caso denso pasa de 7417231 a 7417295 por esa lectura, conservando Work de 8 MiB y el guard exacto. El renderer no cambia; su prueba expresa el incremento mediante el tamaño compilado.

Se adoptaron sólo cuatro rutas Rust y seis fixtures sintéticas del freeze 87847E755ED0425B48514B497E902DE8C6F21FC33488FF16D75E7E03D9104A35, delta C7CE402202F46DDE1302DDDBEBD33D8D6F48BA32E43F0B2CD9C1D6634C94A3F1. La regla específica de `.gitattributes` conserva también sus bytes CRLF y offsets en cada checkout. Verificación del coordinador: 32 comprobaciones de bytes/hashes/rutas sin discrepancias. [Aceptación independiente](Markdown-B2-resource-owner-acceptance-r1.md): cinco casos propios y 45 regresiones PASS, incluyendo coincidencias con Code/FrontMatter, wrapping y Code genuino. El baseline conserva 2 PASS / 3 FAIL y sus binarios/logs.

Composición canónica: 77 pruebas PASS (recursos 4, renderer 13, vista 30, búsqueda 14, inline 16), formato y Clippy del workspace/todos los targets aprobados. Bin canónico preservado B168B0471467F72C799AC364AA26E3E75F94A20996EC1E14F19CCB76AF423068; evidencia privada .tmp/root-b2-owner01-2026-10-10. El trabajo agregado correcto y su rechazo exact−1 se conservan; no se actualizó un golden a partir de la salida.

El hallazgo B2-RESOURCE-OWNER01 queda cerrado en este corte. El renderer sigue siendo biblioteca interna: Markdown interactivo continúa en Fuente con aviso. Esta aceptación no completa RV-301/306, inspector, Source común, pump, fences, gráficos, QA visual o ejecución Unix.
