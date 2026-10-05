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
