# Conversor HTML opcional: propuesta de registro Nu R2

Autor root. Propuesta privada de viabilidad de registro, pendiente revisión independiente; no abre sprint ni implementa el parser/DOM. Base producto a948cfe, Nu nativo0.115.1. No tocar Cargo/lock/código del incremento Unicode ni instalar perfiles/publicar.

## Evidencia y problema

Q25 exige from html opt-in, open y http get nativos con raw intacto. Probe anterior confirma que open usa def from html; nueva prueba propia HTTP loopback con el mismo módulo falla para text/html y text/html;charset=UTF-8: Internal error can't run custom command with run, use block_id. Raw conserva la fuente; el proceso sin import y la apertura .htm con delegación funcionan. Se conservan seis expectativas previas, cuatro PASS y dos FAIL, no se adaptan a aprobar una integración incompleta.

Fuentes exactas nu-command0.115.1: open.rs217–226 eval_call; http/client.rs1026–1032 invoca directamente Command.run. nu-protocol0.115.1 signature.rs993–1003 devuelve ese error en el comando custom. Archivos publicados/checksums verificados; no atestación del build de nu.exe. Evidencia html-nu-http-discovery-r1, manifiesto de fuentes43E7D78B36187BF709DFAFB977745D50A51123FFD6FFFEB2AB6C31BF70B6ACEE y resultadosF19C85A7AD230E794662AF9D9D555A2F3D3CAAF5C6AC3BCFEED164610436038B.

## Hipótesis de solución a contrastar

Un companion opcional separado y con firma estable, nu_plugin_r3viewer_html, registra comandos PluginCommand from html y from htm directamente. El companion tipado actual conserva sus comandos; cargar viewer no activa la conversión HTML nativa. El bundle completo incluye el companion opcional; activación explícita se documenta posteriormente y debe comprobarse sin depender de wrappers custom para el converter descubierto por http get. No cambiar open/http get ni elegir un fork de Nu.

La arquitectura futura comparte parser y DOM entre visor y conversor, pero este spike sólo demuestra la invocación de PluginCommand, no parser correcto, árbol, lectura grande, budgets, IO containment ni frontend HTML. Registrar un plugin que devuelve un sentinel no equivale a entregar from html.

## Spike privado propuesto

1. Paquete Rust standalone en directorio privado, sólo PluginCommand from html/from htm, SDK0.115.1 fijado y sus fuentes actuales registradas. Produce sentinel y fuente exacta de la fixture propia25B (24HTML+LF); no código de producción, heurísticas ni red/file secundaria.
2. Fixture y matriz anteriores a outputs: HTTP text/html y charset, open .html/.htm, raw y control sin plugin. También invocación directa. Cada Nu nuevo con -n; --plugins temporal activa el companion explícitamente, sin registro/perfiles. Conserva módulo custom y sus FAIL como control, sin sobrescribir.
3. El servidor es propio, HTTP loopback puerto efímero; se cierra y join al terminar. Cada proceso Nu termina normalmente, queda reaped; no rescate se interpreta PASS. Errores/setup/timeouts se conservan y distinguen de fallo del contrato.
4. Las únicas entradas del spike son pequeñas y sintéticas; si se usa into_value para reunirlas, se declara esa limitación y no se anuncia una implementación acotada de parsing por ello. Ningún resultado sirve para aceptar input128MiB o bytes inválidos.
5. Freeze de fuentes/lock/binario antes de aceptación independiente; comparar el mismo cuerpo, no un árbol propio usado como oracle. Revisión del candidato y prueba de registro por agente distinto del autor antes de recomendar adopción.

## Salidas posibles

Si PluginCommand también falla, mantener RV404 abierto e investigar eval_call/plugin interface; no sustituir http get. Si pasa, demuestra una vía de registro nativo. Quedan por cerrar activación opcional dentro del paquete, convivencia con plugins de terceros, versión de Nu, parser/DOM/procedencia y cuotas, encoding/metadata/base y cleanup real. No se modifica el requisito de árbol ni se acredita HTTP completo de viewer.

## Delta HNU-P01/02/03 (2026-10-06)

HNU-P01: conservar exactamente sample.htm existente25B, <p>Hello &amp; world</p> más LF final. Sourcehash y bytecount se fijan antes runtime; sintrim/newlineadaptation. Cada sentinel devuelve fuenteigual byteporbyte. La cifra24 deR1 describía sólo HTMLsinLF y era errónea para fixture; no cambia los oráculos originales.

HNU-P02: spike utiliza SDK upstream original nu-plugin/nu-plugin-core/nu-protocol0.115.1, sin patch local ni copiar configuración Cargo del producto. Workspace standalone; resolución efectiva Cargo.lock/features/manifests/VCS/archivechecksums/sources/licencias congelados antesbuild. Si falta fuente, adquisición privada registrada; nunca resolver silenciosamente a fork de misma versión. Buildprofile/Rust/version y hashbin posteriores registrados. El sentinel no evalúa callbacks ni solicitudesEngine; no demuestra cancelación/flowcontrol del encoder actual ni parsing128MiB/DOM/typedbridge. Si resulta necesario el fork, detener esa elección y producir delta explícito, no substitución inadvertida.

HNU-P03: driver de proceso completo, no sólo waitNu. H crea bootstrapPythonC propio que bloquea START antes de lanzar Nu. Retiene PopenC inmediatamente y capture/writerowners antesstartthreads; crea/asigna JobWindowspropioKillOnClose+ProcessMemoryLimit512MiB, sinbreakaway, conreadback/membership antesSTART. C entonces lanza Nu, que crea el companion: todos heredan J. Máximo esperado tres procesos C/Nu/plugin; no buscar porPID/nombre ni OpenProcess. C conserva PopenNu antes de setupfallible, verifica wait/cierreHandle y erroressticky;stdout/stderrNu propios se encaminan a captures acotadas con ownership explícito, no acumular read_text/completions sinquota. H valida C receipt/wait/joins/cierres y JobActive0 antes de éxito; no inferir ausencia deplugin desde exitNu. Si J no se vacía normalmente, registrar fallo y cleanup propio, no promoverlo a PASS por kill. Registrar teardown delplugin/C/Nu dentroJob conterminación/waitActive0 y cierreJob; error mantiene autoridadPending y bloquea siguienteinvocación.

Plazo único10s de invocación, cleanup2s y capsstdout256KiB/stderr64KiB porcapture, leyendo4096. Setup/start/capture/poll/wait/close/reader/writer/servidor exceptions intentan todas obligaciones. Portloopback propio efímero; server sólo fixture25B, shutdown/join acotados ysocketcerrado trasfinalmente. Nu -n --plugins sólo temporal, no registro/perfiles. Prueba de fallo propio deNu/script que mantienecompanion o servidor activo debe acreditar cleanup de todoJ ydelserver; fallo persistente dewait/close clasificadoPending y próximoC/Nu/pluginnoSpawn. No reintentar como nuevoactor ante observationtimeout. H/C retienenestado sólo de sus procesos/Job/handles.

Este grafo es contrato de ownership delspike; implementarlectura/cierre y validar controles antes aceptar registro. No se afirma memoriaaggregate512MiB ni sandboxfilesystem/network. Todos cuerpos/SDK/bin/scripts/fixtures/expected congelados antesAE; root autor no será único juez. C añadida como bootstrap no cambia el CLI público ni producto, sólo garantiza JobantesNu/plugin. No gategráfico adoptado por referencia automática: helpers propios se revisan y prueban paraeste grafo.

PlanR1 188C17DD193DBD990EF17039145B03487B57ABB77195397CF043CE49BF49FABF y crítica9C8174B49BF6532DC691858D5AE18DE63193C44F22DD4DA233D2F12F42756471 intactos. R2 aún pendiente adversario. Matriz original mantiene HTTP/charset/openHTML/HTM/raw/noplugin/direct más failurecleanup; no códigos deDOM oquotasproducto acreditados.
