# Gate físico/profundidad D1 — evidencia R1

2026-10-05, Windows actual, Rust 1.99 privado. Spike exclusivamente temporal; no código producción modificado. Target aislado `.tmp/s03-d1-index-spike/target`.

## Resultado

11/11 pruebas PASS y ejecución del corpus completa (exit 0). La API pública `Source::lex()` iterada de toml_parser 1.1.3+spec-1.1.0, junto con `Document<Shared(Arc<String>)>` de toml_edit 0.25.15+spec-1.1.0, permite recuperar y enlazar declaraciones originales completas a IDs del árbol, sin depender del orden DFS ni asumir que un span de tabla sea todo su contenido. `Key::parse`/FromStr decodifican componentes citados con la misma biblioteca, no un decodificador TOML propio. Esto acredita la viabilidad de la proyección propuesta para el corpus dirigido; no acredita aún implementación productiva ni presupuestos/protocolo/lifecycle del nuevo worker.

Cada declaración contiene kind, rango completo, rango de clave, rango de valor opcional, ID destino y componentes {decoded, rawspan, nodeID}. Hay índice físico ordenado por inicio de declaración, incluidos headers vacíos y asignaciones inline. El scanner recorre la estructura léxica de valores ya semánticamente validados; tokens multiline protegen sintaxis aparente en strings. Leaf spans y arrays/inline tables se cotejan exactamente contra los spans del DOM; cada rango publicado en el spike valida longitud y límites UTF-8. Tablas ordinarias/implícitas/AoT tienen ubicación de container None. Los headers son declaraciones propias, no spans inventados de todo el container.

## Corpus acreditado

- Headers `[a.b]`, `[empty]`, `[a]` intercalados, tablas vacías e implícitas/dispersas: seis declaraciones exactas, destinos separados.
- Dotted keys intercaladas `a.x`, `b."y.z"`, `a.'界'`; clave vacía y clave escapada Unicode: cinco declaraciones exactas, puntos dentro de quoted keys no se confunden con componentes.
- AoT repetidos y anidados: dos `fruit`, tres instancias `variety` bajo sus padres respectivos, headers `[fruit.physical]`: catorce declaraciones y selección ordinal exacta por cada header físico.
- AoT vacíos repetidos, padre declarado después y subtabla del último elemento: seis declaraciones exactas en prueba dedicada.
- Arrays/inline tables anidados, dotted keys inline intercaladas, tabla vacía inline, trailing commas y multiline inline TOML 1.1: nueve declaraciones incluyendo los pares inline, rangos completos; mapas dotted inline mantienen span None.
- Prueba exacta adicional de índices inline: asignación externa, cinco declaraciones de pares/header, orden por byte original, destino de cada par comprobado.
- Multiline basic/literal con falsos headers/asignaciones dentro, continuación backslash+espacios+CRLF, Unicode y ausencia de LF final: tres declaraciones reales; span conserva CRLF y todo el lexema multiline.
- Comentarios, whitespace y datetime con separador espacio y fracción larga: no se incorporan comentarios al rango ni se recorta el lexema datetime.

## Profundidad y orden lógico

Límite lógico: raíz 0, cada componente y cada índice de array aumenta 1. Dotted key de 64 componentes y header vacío de 64 componentes tienen profundidad lógica 64 y pasan; sus variantes 65 se rechazan. Una hoja dentro de header de 64 componentes alcanza 65 y se rechaza aunque cada clave/header individual tenga como máximo 64. Inline table externa más 64 componentes alcanza 65, también se rechaza.

Conteo sintáctico independiente de brackets/braces en valores: arrays 63 → lógica64/sintaxis63 PASS; arrays64 → lógica65/sintaxis64 FAIL por límite lógico; arrays65 → lógica66/sintaxis65 FAIL. Inline tables anidadas 64 alcanzan lógica65/sintaxis64. La biblioteca admite estas entradas; su límite interno no sustituye los dos límites de producto.

Hallazgo relevante: el DOM reordena un padre declarado tarde. Para `root=1; [a.b]; x=2; [empty]; [a]; y=3`, `Document.iter()` da root,empty,a, aunque la primera declaración de a antecede empty. El spike deriva primer byte de aparición de cada entrada desde los componentes/declaraciones y reordena SOLO ordinales de mapping a root,a,empty, conservando IDs y orden de arrays/AoT. Prueba específica acredita ambos órdenes y estabilidad del enlace. Producción debe hacer este paso; usar iter() tal cual incumpliría orden por primera declaración.

## Reproducción

PowerShell:

    . D:/Tools/R3Viewer/scripts/dev.ps1
    cargo test --manifest-path D:/Tools/R3Viewer/.tmp/s03-d1-index-spike/Cargo.toml --target-dir D:/Tools/R3Viewer/.tmp/s03-d1-index-spike/target --locked --offline
    cargo run --manifest-path D:/Tools/R3Viewer/.tmp/s03-d1-index-spike/Cargo.toml --target-dir D:/Tools/R3Viewer/.tmp/s03-d1-index-spike/target --locked --offline

Dependencias exactas: Cargo.toml `toml_edit = =0.25.15` parse-only y `toml_parser = =1.1.3`. Cargo.lock resuelve variantes +spec-1.1.0.

SHA256:

- src/main.rs: F0CFFABF3A008164C739C084D368325B389B59549E71819BAF6768DFD9CC8AE3
- Cargo.toml: 3B80BF38F9A1D14085D8DDC9901CEDD5B477A441C37FD013C242AA204088D6EB
- Cargo.lock: C8E3E4C77B27B6502BF9A7CD6BD3D4B928894C4E2FA16EBC2035CA6EEFE43072
- output-r1.txt: 732D6BECEC4253F5CE8C80F6141FD0628CBE92C822DD69ACDF15FB60AD2C7F28
- test-r1.txt: CF769D5E294C4A4DEC9E4490EC64C25073FBAEB7F5FC3454465E04E16863171C

Evidencia intermedia preservada source-r0.rs/output-r0.txt (antes de ordenar mappings por primera aparición). Primer test tuvo una expectativa de conteo errónea: el corpus inline contiene nueve declaraciones, se esperaba diez; rangos verificados y expectativa corregida. No se ocultó un fallo de biblioteca mediante omitir declaración.

## Consecuencia para el plan

No requiere alternativa de parser ni cambio del contrato core R1: la API pública basta para este enlace dirigido. El gate de viabilidad física/profundidad tiene evidencia positiva y queda disponible para revisión independiente exacta. Producción debe mantener ids estables, orden lógico por primera aparición, índice físico separado y rechazo completo si el enlace/validación falla. La proyección productiva será iterativa o limitada antes de recursión, cancelable, con reserves/budgets; este pequeño spike usa Vec/HashMap/recursión sin presupuestos y NO se traslada directamente como código seguro del worker. No hubo nuevo protocolo ni mediciones de rendimiento/lifecycle. Memoria, cancelación visible, containment, Unix y QA visual siguen pendientes de sus pruebas propias.