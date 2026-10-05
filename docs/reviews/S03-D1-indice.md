# S03 D1 — gate independiente de índice físico y profundidad

2026-10-05. **SIN_CRITICAS_PENDIENTES** para la viabilidad de proyección/enlace demostrada por este spike dirigido. No aceptación del worker productivo, lifecycle, límites de memoria, rendimiento o QA.

Identidad verificada: EVIDENCE-R1.md52EA1A2D82E3F219C562C3E70FCBCD461FA76E40E282C9C34515C78CE2D7F0A9 y src/main.rsF0CFFABF3A008164C739C084D368325B389B59549E71819BAF6768DFD9CC8AE3. Lectura completa de ambos. Document parse válido precede al lexer público; Key::parse/FromStr evita decodificador TOML propio. Resolución de componente usa contexto actual y ordinal AoT seleccionado; valor se coteja contra span DOM exacto. No se obtiene orden físico por DFS ni se inventa contigüidad de tabla.

Reproducción independiente offline/locked/serial con harness/target privados Cviz/s03-d1-index-adversary: **15 PASS, 0 FAIL**. Once pruebas originales conservadas más cuatro propias; log tests-r1.log y fuente src/main.rs. No se modificó el spike autor ni producción.

Casos propios:

- Header quoted con escape1.1 x y componente vacío, asignación Unicode escapada dotted y padre declarado tarde: cuatro declaraciones, componentes cooked/rangos originales y destino exacto.
- AoT anidados tres niveles: nuevo padre reinicia correctamente la selección de hijos por identidad del nodo, no por nombre global; siete headers comprobados hasta segundo ordinal de nieto del segundo padre.
- Cadenas multiline de cuatro/cinco comillas con brackets/braces/coma/hash dentro y escape e, clave x quoted, containers vacíos y array heterogeneous: solo cinco declaraciones reales; protección léxica y rangos exactos.
- Mappings inline intercalados y padre tardío: orden lógico v/late/middle, dentro v a/b y dentro a x/y, separado del índice físico y conservando IDs.

Se reprodujeron también los límites lógico/sintáctico64/65, header vacío frente a hoja65, arrays e inline tables, declaraciones de tablas vacías/dispersas, dotted quoted, AoT repetidos y multiline CRLF. El hallazgo Document.iter reordena un padre tardío está bien corregido mediante primera aparición física: ordenar entradas de mapping, sin mover IDs de nodos ni elementos Sequence/AoT. El índice físico guarda headers y asignaciones inline, no solo hojas finales. Una tabla convencional/AoT no adquiere un span global falso.

No se encontró un caso dirigido que exija cambiar parser o rebajar D06/D08. La implementación debe comprobar completitud/enlace y degradar todo ante discrepancia, preservar esta separación de órdenes y repetir corpus contra el modelo/IPC final. Este spike usa recursión, Vec/HashMap y pathsString sin límites, reconocidos expresamente; no se autoriza copiarlo literalmente como worker seguro. Controles previos a recursión/asignación, cancelación y presupuestos siguen en el plan core y se revisarán en código exacto.

Adenda documental comprobada: docs/sprints/S03-D1-core.md tiene el mismo SHA del plan core aprobado; docs/reviews/S03-D1-core.md, docs/research/S03_TOML_SPIKE.md y docs/reviews/S03-TOML-gramatica.md coinciden byte a byte con sus informes temporales revisados. S03-D1 distingue plan/asset aceptados de implementación en curso y gate físico de proyección pendiente de esta revisión; D01–14 no se altera. Documento del recurso conserva el momento histórico de entrega para revisión y el informe independiente separado registra cierre. No crítica documental accionable.

No se cierra D1, S03 ni RV106 por este resultado. JSON/YAML/XML, protocolo/cap/lifecycle, bundle integrado, aceptación independiente y QA visual/matriz siguen sus gates propios.
