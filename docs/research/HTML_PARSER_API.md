# HTML: inspección de API del parser

2026-10-05. Investigación documental para RV-402/404, sin dependencia instalada, código ni pruebas HTML ejecutadas. Se consultó documentación primaria que identifica html5ever0.40.1; la versión es candidata hasta adquirir y fijar Cargo.lock, fuentes/licencias y corpus. Las rutas docs.rs con versión explícita no fueron accesibles mediante el navegador de investigación; las páginas latest sí. No confundir esta consulta con una descarga verificable del paquete.

## Qué proporciona y qué tenemos que conservar

html5ever implementa parsing HTML5 y permite suministrar un DOM mediante callbacks; no trae una representación única de árbol que debamos adoptar. El repositorio publica licencias MIT/Apache2. Puede reutilizarse sin escribir un parser propio. [Repositorio primario](https://github.com/servo/html5ever).

TreeSink requiere crear nodos/atributos, concatenar texto adyacente, insertar antes de hermanos, separar nodos y mover hijos, además de manejar contenidos de template. Por eso una lista append-only de eventos no sustituye al árbol reparado: primero debe estabilizarse el DOM, después se proyecta al documento común y al árbol Nu. Los métodos de creación retornan handles, sin Result de presupuesto; gestionar límites/fallo sin publicar un árbol parcial necesita gate específico. [Contrato TreeSink](https://docs.rs/html5ever/latest/html5ever/tree_builder/trait.TreeSink.html).

Inferencia de esa API: las notificaciones proporcionan nombres/texto y una línea opcional, pero no garantizan rangos de bytes originales para cada nodo. Entidades, nodos implícitos, concatenación y reparentado impiden atribuir spans exactos por posición del callback. La fuente original permanece aparte; el texto DOM tiene identidad propia. No acreditar origen exacto ni reconstruir raw serializando el DOM.

parse_document recibe un sink y opciones; acepta entrada Unicode incremental. Esto permite alimentación acotada y conservar estado del parser entre bloques, pero no acredita por sí solo cancelación dentro de una llamada ni memoria interna máxima. [parse_document](https://docs.rs/html5ever/latest/html5ever/driver/fn.parse_document.html).

Parser::from_utf8 usa decodificación con sustitución de UTF8 inválido. La política de codificación debe anteceder al parser: no usar esta comodidad para afirmar fuente conservada o conversión estricta. Un stringNu ya es Unicode; bytes locales/HTTP necesitan registrar codificación/errores por separado, sin confundir offsets originales y texto decodificado. [Parser y from_utf8](https://docs.rs/html5ever/latest/html5ever/driver/struct.Parser.html).

scripting_enabled cambia el parsing de noscript aunque no exista motor JavaScript. Para un lector estático, fijar explícitamente esa opción y probar noscript; no asumir que omitir ejecución de scripts equivale a elegir la semántica de parsing correcta. drop_doctype es otra opción: el árbol destinado a from html no debe perderlo por configuración accidental. [TreeBuilderOpts](https://docs.rs/html5ever/latest/html5ever/tree_builder/struct.TreeBuilderOpts.html).

## Próximos gates del adaptador

1. Adquirir fuentes/lock y comparar reparación de texto mixto, tablas, formato mal anidado, namespaces y template con expectativas anteriores a outputs.
2. Definir representación DOM ordenada, identidad/origen y presupuesto efectivo, incluidos callbacks no fallibles y allocations de biblioteca; probar rechazo completo/cancelación.
3. Adaptar DOM al documento común sin una cadena Markdown intermedia. El perfil/wire Markdown cerrado sólo cubre sus perfiles actuales; HTML requiere delta revisado para sus atributos/anclas/controles.
4. Comprobar from html opt-in en Nu0.115.1: open/http get nativos y raw; ningún cambio de perfiles. El conversor y el visor comparten parser, sin confundir DOM con vista documental.

HTTP, bases/redirecciones, cobertura visual/formularios pasivos, navegación, integración, ejecución multiplataforma y RV-402/404 permanecen abiertos. Estos gates refinan el alcance confirmado, no incorporan MathML, JavaScript/CSS ni modo artículo por inferencia.


## Probe nativo de descubrimiento Nu

Nu0.115.1 ejecutado con -n y módulo dummy temporal privado, sin instalación/perfiles ni parser HTML. open sample.html sin import devuelve string exacto; tras importar from html devuelve el record del dummy; open --raw conserva string exacto. sample.htm sigue string: no se deduce alias from html por esa extensión. Invocación directa del conversor temporal también conserva entrada. Para cobertura .htm del conversor opcional habrá que exportar alias explícito y comprobarlo.

El primer control de ausencia era inválido: use en el mismo scope se resuelve al parsear y afectaba la expresión anterior. Se conservó intento1 y se corrigió sólo el control con una invocación fresca independiente sin import; no se atribuye ese resultado a fallo de Nushell/producto. Evidencia privada html-nu-converter-probe-r1/PREOUTPUT-MANIFEST.json, OWN-OUTPUT-attempt1.log, OWN-ABSENT-attempt2.log y OWN-RECEIPT.json, entradas verificadas antes/después. No prueba HTTP, metadatos, árbol HTML, integración del companion, visual ni RV-404 completo.


## Adquisición exacta de fuentes, sin integración

Se adquirió `html5ever=0.40.1` con Cargo en la caché privada del proyecto. Archivo publicado de 86,221bytes, SHA256 `456A1A377E608E555D22DDAB27AC0114BC7A7B4199078108E34C2AEAE6C9B130`, coincide con el checksum del registro crates.io y no figura yanked en el registro observado. Sus 35 archivos originales, incluidas licencias MIT/Apache2, se extrajeron sin modificaciones a `html-primary-source-0.40.1-r1`; manifiesto SHA256 `2FDBDF2834EC03DEA2D5C3B039F4989CB49944BECEAC299102B79DCD1DA48D7A`. El manifiesto publicado declara Rust1.85 y el identificador VCS `d7232d746d3112f08926d169591576bd12fe2fdf`; ese identificador no reemplaza el hash del archivo verificado. [Paquete publicado](https://crates.io/crates/html5ever/0.40.1).

El Cargo.lock publicado dentro de ese paquete referencia `markup5ever=0.40.0`. Se adquirió también ese archivo y se verificó su SHA256 `0AB3DC68AC4A0F5719E560136778C1EE716E296030D75DBD4484E37E39E3A842` contra registro y lock publicado; 12 archivos/65,448bytes de fuente en `html-primary-markup-0.40.0-r1`, manifiesto SHA256 `6EB30CD5685BD99D9A42444F659E648153CD2B6326983A9A024D140C916E87F2`. El lock upstream es evidencia de procedencia de esta inspección, no lock de R3Viewer. Ninguna adquisición cambió Cargo.toml/Cargo.lock del producto ni compiló un adaptador HTML. [Paquete markup5ever](https://crates.io/crates/markup5ever/0.40.0).

La lectura de `driver.rs` confirma `from_utf8` como `Utf8LossyDecoder`, y que `process` ejecuta `loop_until_done` antes de volver. Alimentar por bloques no ofrece una interrupción comprobada dentro de esa llamada. `TreeBuilderOpts` tiene `scripting_enabled=true` y `drop_doctype=false` por defecto: el lector estático debe cambiar explícitamente la primera opción. Estas afirmaciones ya tienen contraste en fuentes fijadas, además de la documentación latest inicial.

El contrato TreeSink está definido en `markup5ever/interface/tree_builder.rs`, no sólo en el paquete html5ever. Incluye dos fronteras adicionales para el gate: `attach_declarative_shadow` devuelve false por defecto y `maybe_clone_an_option_into_selectedcontent` es un no-op cuya documentación advierte que deja un DOM ligeramente incorrecto. Debe decidirse y probarse su política antes de afirmar fidelidad del árbol `from html`; representar controles en solo lectura no elimina esta diferencia del parser/sink. No se habilita una función nueva ni se da por aprobado el DOM mediante estos defaults.


## HTTP nativo: conversor custom incompatible en la versión probada

Prueba propia en Nu0.115.1 Windows, seis casos con expectativas anteriores a outputs y servidor HTTP loopback cerrado/join al terminar. Sin módulo, `http get` conserva la fuente; con un `def "from html"` temporal, `http get` para `text/html` y `text/html; charset=UTF-8` falla con `Internal error: can't run custom command with 'run', use block_id`. Los dos controles raw conservan el texto exacto. Un `def "from htm"` delegado al dummy funciona con `open sample.htm`. Resultado: cuatro PASS y dos FAIL originales conservados; no es aceptación de un parser ni del producto.

Evidencia privada `html-nu-http-discovery-r1/PREOUTPUT-MANIFEST.json`, SHA256 `A9080DEAD6C1E4676E638EA2814C73ED5F9292CF903FC82A54E9D614F5B1B8E4`; `OWN-RESULTS-attempt1.json`, SHA256 `F19C85A7AD230E794662AF9D9D555A2F3D3CAAF5C6AC3BCFEED164610436038B`. Los ocho archivos de entrada/script/expectativas siguen iguales antes/después. No se modificaron perfiles ni comandos nativos.

Se adquirieron archivos publicados `nu-command=0.115.1` y `nu-protocol=0.115.1`, con checksums de registro verificados. Fuente HTTP SHA256 `7EAF6A7EB71A24B2CEE57656FB377A2B427BCDA95DD562DE5413DBE08BF72B0D`: líneas1026–1032 invocan directamente `Command.run`. En cambio open.rs217–226 usa `eval_call`. signature.rs993–1003 del comando custom devuelve precisamente ese error; SHA256 `2F91D4370159D77C03BA0F4E15009299FE53D5C252ADCB21B0B19ED7BDE9F9E5`. Ambos paquetes identifican VCS `798c55d19505fd52f205d7eb32a571b9d06ec9e6`; esto explica la ruta observada, sin atestar el build del ejecutable. [HTTP fijado](https://github.com/nushell/nushell/blob/798c55d19505fd52f205d7eb32a571b9d06ec9e6/crates/nu-command/src/network/http/client.rs), [open fijado](https://github.com/nushell/nushell/blob/798c55d19505fd52f205d7eb32a571b9d06ec9e6/crates/nu-command/src/filesystem/open.rs).

Conclusión: un wrapper custom escrito en Nu no cumple por sí solo Q25 para HTTP en esta versión. Se propone contrastar un companion opcional con comandos PluginCommand directos y firma estable, separado del companion tipado actual; todavía no implementado, revisado ni aceptado. El parser debe seguir compartido con viewer y los comandos nativos permanecer intactos. Registrar un dummy PluginCommand sólo demostraría invocación: activación opt-in, árbol/encoding/metadata, cuotas, cleanup y RV-404 requieren implementación y aceptación posteriores. No se cambia el requisito para evitar los dos FAIL.

## Spike de registro refinado

2026-10-06: [plan R2 del companion opcional](../refinement/HTML-CONVERSOR-NU-R2.md) y [revisión independiente](../reviews/HTML-conversor-nu-plan-r2.md) SIN_CRITICAS_PENDIENTES para diseño del spike. La propuesta conserva menciones históricas de revisión pendiente; dictamen posterior prevalece. Fuente de fixture25B exacta, SDK upstream0.115.1 y grafo H/C/Nu/plugin en Job propio antes del lanzamiento. Implementación privada iniciada tras esta revisión, todavía sin candidato/runtime aceptado. Sentinel sólo prueba registro: DOM, parser, integración y RV-404 siguen pendientes.

## Spike PluginCommand R1 rechazado por comprobación de éxito

2026-10-06. Candidato privado html-nu-plugin-spike-r1 MANIFEST69EACB781B29635D12FAB4CA80403A2E3A6FBE7E35FF5F4C773C478DF3D6BAA7,14583 registros originales verificados por raíz. Ocho resultados funcionales del autor no equivalen a aceptación; revisión de raíz **NO APROBADO, HNU-R01**.

El bootstrap C comunica el exit de Nu en C_receipt.exit, pero el driver sólo exige exit0 de C y cleanup. Dos controles propios anteriores a outputs (`exit 7`, `print broken; exit 9`) reciben Nu7/9 y driverPASS=true: FAIL. Control positivo Nu0 PASS. Todos Reaped/wait/handles/streams/joins/JobActive0, Pendingfalse y un hilo; no rescate. Cleanup correcto no convierte un comando Nu fallido en éxito.

Informe privado HTML-NU-root-review-r1.md SHA9B8F35DCFCC4AC3BA2C3CBC46C8E724E477C109B57E7AF87E6F2702B919C89E8; oracle027151F196FF13D269A3FA231DE54CAD69ABAD2F6C32F732854772D8F23F2828, resultados2BA58590704F731675F7B0590B5B8C9E3DEA3A2B872911A9C911E0CDB94F0945. Nueve archivos runtime/fuente copiados byteexactos antes/después, no se afirma copia completa del SDK. R1/oráculos/FAIL permanecen intactos.

Corrección R2 debe exigir exit0 de Nu además de todos recibos actuales, incluso con stdout válido seguido de error. Repetir oráculos sin cambiarlos y los positivos de registro, revisar delta antes de recomendar adopción. SDK/bin/parser no se modifican por este error del helper. RV-404, conversión DOM, producto y QA siguen pendientes.
