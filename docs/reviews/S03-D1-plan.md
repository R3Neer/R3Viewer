# S03 D1 — revisión adversaria de propuesta R2

Fecha: 2026-10-05. Alcance: contrato y organización de D1 TOML antes del candidato de producción. Sin editar producción, ejecutar el spike ni acreditar QA visual/Unix.

## Estado

**SIN_CRITICAS_PENDIENTES** dentro del alcance de propuesta D1. No se aprueba aún la implementación ni se certifican los resultados preliminares del spike. S03 y RV-106 permanecen abiertos.

Documentos revisados completos: r3viewer-s03-d1-proposal-r2.md y r3viewer-s03-d1-acceptance-r1.md. R1 de RV106 queda investigación para cuatro formatos, no autoridad para implementar JSON/YAML/XML sin refinamiento propio. También verificada directamente la corrección README:88: «Un fallo detectado o presupuesto excedido del motor deja el documento entero plano». Cierra la crítica documental C sobre errores internos de regex no expuestos por Syntect.

## Cierre de hallazgos anteriores

- RV106-P1, P2: selector y origen precisados. `--format toml` es intérprete explícito de entrada textual; Nu no textual es error2. `--structure` en Nu ya tipado conserva inspección recibida. Auto/Code solo infieren de extensión local real; Text desactiva interpretación y Text+Structure da error2; Language solo controla sintaxis. No se abre metadata ni se adivina por contenido.
- RV106-P2, P2: búsqueda de fuente segura y búsqueda de claves/hojas interpretadas tienen dominios y estados separados. El salto usa el span original completo validado; ningún offset cooked se suma a un rango del archivo. Sin span contiguo, diagnóstico sin cambio de vista.
- RV106-P3, P2: límites sintáctico64 y lógico64 independientes, incluidos dotted keys y headers. Se comprueban antes de publicar y al navegar/historial, no se confía en wire Nu para limitar el árbol de archivo.

## Revisión integral

Fuente Arc inmutable y arena final propia con ordinales/aristas constituyen un propietario coherente. Adaptar Nu por referencia evita una segunda copia universal. El DOM nativo solo existe en parsing aislado; la representación final preserva tipos, lexemas y rangos sin confiar en f64 o precisión temporal nativa como autoridad. Tab conserva ambas vistas; `o` fuera de editor/ayuda realiza el salto explícito al origen y no sustituye la consulta ni último resultado de búsqueda de Fuente. La búsqueda de cooked no requiere un mapa por carácter no demostrado. No se encontró binding previo para `o` en viewer_session actual.

D02/D07 exigen publicación completa o fuente con diagnóstico, nunca un árbol recuperado parcial presentado como éxito. D06/D08 separan jerarquía por primera declaración de orden físico y admiten None para tablas implícitas/dispersas. Esto permite cumplir TOML sin inventar continuidad de rangos.

D11/D12 y R2 separan memoria persistente padre y pico hijo. Parsing monolítico se lleva a proceso terminable si no cumple cancelación cooperativa real. La elección preliminar ya observada no basta para cerrar lifecycle: protocolo, ownership, transferencia, cleanup y mediciones quedan exigidos antes de producción. Sintaxis se suspende/recoge durante construcción para evitar dos presupuestos independientes confundidos con un límite agregado. Fuente sigue navegable, publicación obsoleta se rechaza y UI no hace parsing ni IO/join bloqueantes. Unavailable permanece latched por revisión y Tab no reinicia indefinidamente el parser.

No se detecta recorte del alcance completo de RV106 ni una promesa de multiplataforma fundada en crossbuild. Gramática TOML auténtica, licencia y pin verificable son gates; Nu/PowerShell siguen abiertos.

## Precisiones que deben trasladarse a la aceptación del candidato

Son verificaciones ya derivadas del contrato, no nuevas críticas de propuesta:

1. El resaltado temporal del salto `o` debe convivir con el estado de búsqueda de Fuente: `n/N` después del salto sigue usando la consulta/último resultado conservados. Probar salto desde clave y hoja cuyo cooked difiere de raw, retorno Tab y destino sin span.
2. Además del corpus general, la gramática y el intérprete TOML1.1 deben ejercitar formas nuevas pertinentes: escapes `\e`/`\xHH`, tiempos con segundos omitidos y inline tables multilínea con coma final. No certificar cobertura por cargar una gramática histórica. La especificación normativa es [TOML1.1.0](https://toml.io/en/v1.1.0); la política de vistas, orden visual y presupuestos pertenece a R3Viewer.
3. El futuro protocolo debe validar profundidad/nodos/rangos/totales durante recepción y antes de construir/publicar la arena; medir capacidades persistentes reales y picos separados, también en salida inline. Un cap del proceso hijo no sustituye al presupuesto de representación del padre.
4. La revisión del candidato debe comprobar que Source/Structure conservan sus estados también tras cancelación, fallback y suspensión de sintaxis; q/Ctrl-C restaura TTY antes de la recogida final y no mata llamadas/procesos ajenos.

No hay hallazgos accionables adicionales bloqueantes o no bloqueantes del plan. Estas verificaciones no acreditan pruebas realizadas ni permiten cerrar D01–14 por lectura.
