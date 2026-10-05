# R3Viewer — contrato observable gráfico R2 (propuesta temporal)

Fecha: 2026-10-05. Base leída: HEAD 23722241092e0e94b1feeb4a1d1aa6ece8cdc674, árbol limpio en la inspección. No hay implementación, nuevo sprint, instalación, commit ni QA gráfico en este encargo. S03 permanece activo con XML/lifecycle/visual/Unix pendientes; el refinamiento paralelo no sustituye ese objetivo. Las 39 tareas RV y soporte gráfico completo permanecen vigentes.

## Resultado propuesto del siguiente corte vertical

Abrir una imagen PNG local estática con `viewer dibujo.png`, verla proporcionada en TUI, ampliarla y volver; `viewer -i dibujo.png` imprime una representación gráfica que permanece en scrollback cuando esa capacidad está acreditada. Redirigir cualquiera produce exclusivamente referencia textual legible. Terminal sin capacidad o geometría fiable muestra recurso identificado y explicación de lectura parcial. Es un corte de RV-201/202/205 con dependencia observable RV-203 (zoom/composición), no cierre de esas tareas completas ni de E04.

El corte prueba PNG estático primero. JPEG/WebP/GIF, SVG, PDF, Markdown/math/Mermaid y HTML siguen en sus tareas; animaciones mantienen primera imagen/frame, sin bucle automático. Ninguna descarga ni apertura remota ahora: RV-401 conservará URL/base, límites y autorización explícita; el modelo gráfico podrá recibir posteriormente recursos ya autorizados. PDF y Markdown reutilizarán identidad, dimensiones, alt/reference, estado y presentación del recurso; no se diseña aquí su arquitectura ni AST.

Entrada: formato explícito prevalece sobre auto. Sólo un origen auténticamente local, abierto por argumento o `--path` tipado Nu, permite inferir PNG por extensión `.png` sin distinguir mayúsculas; `--format png` selecciona PNG local independientemente de extensión. Firma/IHDR y límites se validan según la tabla de modalidad de esta R2, nunca por mera extensión. `--format text|code|toml|json|yaml` conserva su lector textual actual aunque la ruta acabe en .png: no reinterpretar un selector explícito ni rescatar automáticamente un fallo UTF-8 como imagen. Un PNG binario real con text/code falla1 por entrada UTF-8 inválida, como el loader vigente; un archivo textual con nombre .png y formato text/code conserva texto y reglas actuales. El pipeline Nu conserva su semántica: string/string_stream/literal no infieren PNG por contenido, extensión ni metadata; binary/unknown conservan el inspector tipado. `--format png` sobre literal/pipeline sin modalidad de ruta es2 explicado en este primer corte, sin ampliar ni eliminar futura cobertura de binary PNG. La validación que requiere conocer un `--path` de Nu ocurre tras leer su envelope tipado, antes de abrir el archivo: pre-IO del recurso no significa no consumir protocolo Nu.

Config/CLI inválidos son2; fallo de apertura/lectura o límite global de adquisición es1; corrupción o límite local de representación conserva recurso original identificado, diagnóstico y0, sin imagen parcial ni pixels inventados. q/Escape normal0 y Ctrl-C130. El origen original permanece intacto; una referencia recuperable no declara éxito visual ni integridad completa. La tabla siguiente reemplaza la generalización R1 «PNG corrupto fatal1».

### Modalidad, validación y errores (GC-02)

R = auto/force/symbols que llega realmente a rasterizar; N = `--graphics none`; D = stdout redirigido; F = fallback por capacidad/geometría/backend/política efectiva de color antes de rasterizar. D aplica recorrido N; F aplica recorrido N. Force sólo es R si reúne sus precondiciones. En todos los modos se abre archivo regular de sólo lectura y se comprueba longitud de entrada declarada (hasta64MiB para este candidato PNG); el modo sin decode no acredita integridad de bytes que no lee. No volver a abrir para cambiar el snapshot durante rasterización.

| Escenario | R (raster) | N, D y F (referencia) |
|---|---|---|
| Archivo no existe/inaccesible, no regular, error de apertura/metadatos necesarios |1 con diagnóstico; no imagen ni referencia de éxito |1 con diagnóstico; no fingir recurso adquirido |
| Tamaño del archivo excede límite global64MiB, o lectura efectuada excede ese límite |1; no truncar ni rasterizar prefijo |1; misma política de adquisición, aun sin full decode |
| Error IO leyendo bytes requeridos por el modo |1; retirar representación incompleta |1 si ocurre en la cabecera; no buscar ni afirmar errores en cuerpo no leído |
| Firma/IHDR ausente, truncado por EOF, CRC IHDR inválido, dimensiones0/desbordantes |0; referencia original con «cabecera PNG inválida», diagnóstico; no decode/imagen |0; misma referencia, dimensiones desconocidas o explícitamente no confiables |
| IHDR válido pero pixels/RGBA exceden presupuesto gráfico |0; referencia con dimensiones declaradas y límite local, sin reservar raster |0; referencia con dimensiones declaradas/límite, sin reservar raster |
| Cabecera válida, IDAT/cuerpo corrupto detectable sólo al decodificar |0 al detectarlo; conservar referencia y diagnosticar «no se pudo representar PNG» |0; leer sólo cabecera33bytes máxima, no body/decode; referencia «integridad completa no validada», no declarar PNG válido |
| Decode/encode/worker falla o excede límite local de tiempo/memoria/payload |0 con original/referencia, diagnóstico y cleanup; sin imagen parcial |No se inicia esa operación;0 de referencia, sin proclamar ausencia de corrupción |
| PNG completamente decodificado y representación emitida |0; estado decodificado, visual sólo si QA lo acredita |0; cabecera identificada e integridad completa no validada |

Cabecera acotada: como máximo33bytes (firma y primer IHDR con13bytes de datos y CRC), comprobando longitud/tipo y sin perseguir chunks o reservas según datos hostiles. Un error de lectura es IO1; EOF de cabecera es contenido incompleto recuperable0. Metadatos de longitud no sustituyen contabilidad real de bytes leídos en R ni verificación de archivo mutado durante snapshot. El gate técnico deberá acreditar rechazo claro si cambia la entrada, sin mezclar dos versiones como PNG válido. N/D/F no cargan cuerpo ni codec y distinguen dimensiones declaradas de decodificación completa. Fallo de representación no reemplaza el recurso con «imagen vacía»: sólo referencia segura. En futuro embed mantiene la misma clasificación local0 y documento origen legible; IO fatal del documento principal sigue1.

### Opciones de formato y presentación existentes (GC-04)

| Entrada/opciones | Selección y comportamiento |
|---|---|
| Ruta local .png, auto, sin opciones textuales incompatibles |PNG; recorrido R/N/D/F anterior |
| Ruta local cualquiera, `--format png` |PNG; no inferencia alternativa por extensión |
| PNG explícito o auto local .png con `--structure` |Error2 «estructura requiere fuente de datos, no PNG», antes de abrir recurso; incluso none/redirección no neutraliza incompatibilidad |
| PNG explícito o auto local .png con `--language rs` u otro lenguaje |Error2 «--language selecciona resaltado textual y no se aplica a PNG», antes de abrir recurso; no cambia formato. La validación vacía/tamaño actual sigue vigente |
| PNG con `--line-numbers` o config `line_numbers=true` |Opción sin efecto sobre raster y referencia; no inventar filas/líneas. No invalida perfil global ni produce banner. `--graphics-info` puede informar inaplicabilidad |
| PNG con `--no-line-numbers` |Sin efecto sobre raster/referencia; coexistencia con --line-numbers conserva error2 actual |
| Ruta .png con `--format text|code` |Mantener loader/UTF-8 y selección textual actual; PNG binario real da1, no fallback PNG. Text con --language/--structure mantiene2 antesIO |
| Ruta .png con formato de datos explícito |Mantener loader textual y parser seleccionado, no PNG; UTF-8 inválido1; parseo textual fallido conserva Fuente con diagnóstico0 vigente |
| String/string_stream/literal auto cuyo contenido parece ruta/PNG |Texto recibido sin abrir ni inferir imagen; --language y --structure siguen contratos actuales de texto |
| Binary/unknown/registro/lista Nu auto con --language |Conserva tipo/inspección actual: la gramática no convierte datos en texto ni abre PNG; sin nueva prohibición global de --language |
| Valores Nu no textuales con selectores existentes text/code/datos |Conservar validación y render actuales tal cual, sin imponer nueva regla PNG a esos adaptadores; selectores de datos mantienen requisito textual vigente |
| Literal o binary/string Nu con --format png, sin --path |2: PNG local explícito requiere origen local en este corte; futura capacidad sobre bytes sigue backlog |

Las reglas específicas PNG sólo operan después de selección válida de origen/formato; no hacen inválida globalmente estructura de datos Nu ni --language de otros adaptadores. Puede validarse pre-IO la combinación con PNG explícito y auto de ruta conocida. Si el origen sólo se conoce al recibir Nu, resolver semántica primero y rechazar antes de IO del archivo. No usar metadatos de string/unknown como origen local. Argumento+pipeline presentes, -p/-v, config inválida y demás validaciones actuales siguen en su orden vigente; estos ejemplos no autorizan eludirlas.

## Opciones únicas para CLI y Nushell

Son nombres propuestos, todavía ausentes del producto. Deben implementarse simultáneamente en `r3viewer` y `viewer` Nu, con idénticos valores, validación y códigos; sin aliases extra ni nuevas abreviaturas.

| Opción | Contrato |
|---|---|
| `--graphics auto\|sixel\|kitty\|iterm2\|symbols\|none` | Auto negocia según presentación; protocolo explícito solicita ese backend, sin afirmar que el terminal lo soporte. Symbols es alternativa de bloques identificada; none imprime referencia. |
| `--graphics-cell-size WIDTHxHEIGHT` | Enteros decimales de píxeles por celda con dígitos ASCII, ambos1..1024, separador `x` ASCII minúscula, sin espacios/signos ni ceros iniciales. El dígito0 interno es válido: `10x20` y `1024x1` válidos; `0x20`, `010x20`, `10X20`, `10x020` y dígitos Unicode inválidos2; la misma gramática rige flag/config/Nu. Override de geometría, independiente de protocolo. No se deduce de tamaño de fuente en puntos/DPI. |
| `--graphics-info` | Durante la invocación informa por stderr modo, capacidad/protocolo, procedencia de geometría, fallback y motivos. No agrega escapes a stdout redirigido ni ofrece éxito decorativo. |

Claves TOML de nivel raíz: `graphics = "auto"`, `graphics_cell_size = "10x20"`; `graphics_info` no se guarda, es diagnóstico por invocación. Heredar lectura/config/path estándar y límite64KiB existentes, deny_unknown_fields y error2 antes de IO de documento. No crear configuración implícita. El ejemplo10x20 es elección explícita, no medida universal. El módulo Nu reenvía flags conservando encoder como primer comando, para no perder PipelineData; no poner un valor auto por defecto en Nu que tape la config del ejecutable.

Resolución por campo: flag explícito > config explícita > entorno como indicio > detección/default. No añadir variables R3VIEWER_* en este corte. `TERM_PROGRAM`, WEZTERM_EXECUTABLE y demás hints nunca certifican protocolo/cleanup; `TERM=dumb` restringe auto a referencia. Geometría: flag > config > medida actual válida; desconocida sigue desconocida. Override explícito puede intentar capacidad no inferida; sin un backend construido que cumpla esta presentación, conserva referencia y diagnostica. Fallar la capacidad de terminal es lectura parcial con0, no error de entrada. Sólo sintaxis/valor de opción inválido es2. El protocolo explícito es force de selección, no force de seguridad: puede omitir negociación de capacidad, pero exige geometría y backend de esa presentación con lifecycle acreditado. Si esas precondiciones fallan, referencia y0. Cuando un protocolo no proporciona ACK fiable, la emisión forzada se registra como solicitada/no confirmada; nunca inferir pixels visibles del éxito de write. La referencia identificable permanece accesible y el gate visual decide compatibilidad; no inventar --strict ni fallo fatal por falta de soporte del terminal. `--graphics none` no prueba ni decodifica pixels: conserva lectura de cabecera acotada para dimensiones cuando sea segura. En imagen standalone none sigue referencia textual y tabla de errores anterior, no se convierte a inspección binaria ni se diagnostica como entrada sin lector.0 es referencia de entrada adquirida, no excusa para ocultar IO1.

Redirección prevalece sobre todo, incluyendo protocolo y geometría forzados: no raw, alternate screen, probes ni CONOUT$ alternativo para eludir stdout. `--graphics-info` allí explica “salida textual por redirección”. Color efectivo conserva exactamente S03: `--colour` explícito > `colour` de config > auto por defecto. Always permite ANSI de texto/symbols con stdoutTTY incluso si NO_COLOR o TERM=dumb; Never lo prohíbe; Auto lo prohíbe con NO_COLOR o TERM=dumb. Redirección siempre plana, incluso Always. Symbols pasa a referencia0 únicamente cuando ese color efectivo está desactivado, no por mera existencia de NO_COLOR si Always lo ha sustituido. Raster Sixel/Kitty/iTerm2 es independiente de esta política y puede transmitir pixels de color con Never/NO_COLOR; `--graphics none` es la opción para omitir imágenes. No añadir una nueva precedencia ni cambiar AE vigente.

| stdoutTTY y NO_COLOR presente; graphics symbols con geometría válida | Resultado efectivo Nu y CLI |
|---|---|
| `--colour always` (incluso config never/auto) |Symbols coloreados permitidos |
| Sin flag, config `colour="always"` |Symbols coloreados permitidos |
| `--colour auto` (incluso config always) |Referencia0, sin ANSI de color |
| Sin flag/config o config auto |Referencia0, sin ANSI de color |
| `--colour never` (incluso config always) |Referencia0, sin ANSI de color |
| Cualquiera de esos casos con stdout redirigido |Referencia textual, sin escapes y sin decode/probe/raw |

Permitir color no valida geometría ni capacidad/lifecycle: las otras precondiciones siguen rigiendo. `--graphics-info` es diagnóstico solicitado por invocación, no banner habitual.

Ejemplos futuros compartidos Nu/CLI:

```text
viewer dibujo.png
viewer -i dibujo.png --graphics iterm2 --graphics-cell-size 10x20
viewer dibujo.png --graphics sixel --graphics-info
viewer dibujo.png --graphics none
viewer dibujo.png --graphics kitty > referencia.txt
```

## Capacidad, geometría y fallback

Capacidad separada por presentación, no un bool global. Publicar protocolo seleccionado, dimensiones de recurso, geometría medida/override/desconocida y motivo de fallback en ayuda/diagnóstico. Tener Sixel no implica conocer tamaño de celda; soporte TUI no implica scrollback inline ni borrado de datos.

Auto: sólo elegir un backend con emisión, clipping/placement y limpieza acreditados para modo/terminal concreto; preferencias candidatas WT=Sixel, WezTerm=iTerm2, Kitty cuando esté acreditado, no selección universal por marca. Negociación puede probar soporte y usar una tabla de compatibilidad verificada por versiones, manteniendo mejor esfuerzo para otros terminales. Sin geometría pixel fiable usar alternativa que acredite conservación de proporción sin esa métrica; si no existe, referencia. No inventar10x20 automáticamente. Symbols sólo conserva proporción física con geometría explícita/medida: desconocida implica referencia con explicación, no deformación silenciosa.

Fallback explícito: un protocolo forzado no cambia silenciosamente a otro protocolo raster. Si no sirve para ese modo, muestra referencia e identifica intento fallido; auto puede usar siguiente backend acreditado o symbols y etiqueta la degradación. Symbols/none son alternativas útiles, no solución final ni acreditación de gráficos completos. La elección del fallback no reduce Windows/macOS/Linux ni formatos pendientes.

Probes: propietario único de terminal con deadline total inicial250ms, volumen y respuestas acotados; stdin documental nunca se lee como respuesta. Pulsaciones ordinarias conservan orden, Ctrl-C no se pierde y overflow visible sigue el contrato actual; parser reconoce sólo respuestas esperadas dentro de su ventana y no interpreta escapes arbitrarios como capacidad. Respuestas tardías no alteran documento ni quedan como teclas tras cerrar la ventana. Inline no toma raw ni lector de teclado: hasta demostrar negociación sin alterar ese contrato usa override/hints acreditados o referencia. Cancelar/timeout no deja thread detached leyendo terminal ni payload. No modificar tmux, perfiles ni settings como efecto de abrir imagen.

La API11.1 no puede usarse ciegamente: `Picker::from_query_stdio` lee stdin global; `query_with_timeout` lanza thread sin join y Busy renueva timeout; `font_size_fallback` Windows devuelveNone; `Picker::halfblocks` y from_fontsize invocan detección que ejecuta `tmux set -p allow-passthrough on`. Usar alternativa/adaptación sólo tras gate experimental específico sin esos efectos; esta propuesta no fija clases, procesos o crates.

## Representación y navegación observables

Referencia segura: `[imagen PNG: dibujo.png; 640x480; representación no disponible]`, con origen identificable y motivo discreto. Sanitizar nombres/alt como el texto vigente; nunca imprimir escapes del archivo. Si dimensiones aún no son fiables usar “dimensiones desconocidas”. Si sólo se validó cabecera: `[recurso PNG: dibujo.png; dimensiones declaradas 640x480; integridad completa no validada; representación omitida]`. Si cabecera/codec falló: `[recurso solicitado como PNG: dibujo.png; no se pudo representar; original conservado]`.0 nunca afirma imagen visible, PNG íntegro ni bytes del cuerpo no inspeccionados. Redirección genera esa referencia en stdout y diagnósticos stderr, sin ANSI/OSC/DCS/APC, incluso bajo flags always. No inventar un alt descriptivo del contenido visual.

Vista compacta: ajustar dentro de viewport útil, preservar proporción en píxeles y no ampliar automáticamente una imagen pequeña; metadata/estado no tapa la imagen. Alpha se conserva en recurso. Para protocolos sin alpha/composición fiable, componer sobre color de fondo explícitamente conocido de tema oscuro/claro; con tema auto no medido, usar fondo mate documentado y visible en estado. No volver transparente el negro ni fingir alpha nativo. Corpus con transparencias parciales confirma sin halos ni pérdida accidental; fondo mate es alternativa identificada, no defecto escondido.

TUI: Enter abre ampliación; `+`/`-` cambian escala, `0` vuelve a ajuste, flechas/hjkl desplazan cuando no cabe. Backspace sale de ampliación y restaura posición y escala compacta; q/Escape mantienen salida normal de la sesión, sin reinterpretar el contrato S03. Ayuda enumera teclas y escala; límites impiden zoom/memoria infinitos. Cursor/foco no desaparece. Resize recalcula área visible sin deformar, conserva centro lógico o ancla identificable; resultados de generaciones viejas no reemplazan el tamaño actual. No asignar aquí porcentaje/límites de escala exactos antes de gate de geometría; el gate debe publicar rango finito y prueba de sus extremos antes de candidatura.

TUI clipping/retirada: scroll fuera de viewport, vuelta, zoom, resize repetido y cierre dejan sólo imagen vigente; no pintar fuera del área reservada ni borrar texto ajeno. Cambiar protocolo/tema invalida representación previa y limpia placements/datos propios. El retorno y nueva sesión no dejan residuos. Dejar alternate screen no sustituye cleanup de recursos del terminal (Kitty distingue retirar placements y liberar datos).

Inline: impresión única en orden, sin raw/alternate screen ni sesión de teclas; reserva filas físicas suficientes y el prompt queda después de la imagen, fuera de sus píxeles. Debe sobrevivir scroll arriba/abajo de historial en terminal acreditado. Resize posterior es del terminal, no un proceso lector persistente: no prometer recomposición interactiva tras exit. No imprimir sin adaptar `Sixel.data` o `Iterm2.data`: clear_area y doNotMoveCursor de TUI no acreditan scrollback. Éxito inline no borra la imagen emitida al terminar; liberar recursos locales/hijos es distinto de eliminar su representación persistente. Cancelación a mitad de transmisión termina secuencia de modo acotado, devuelve prompt limpio y130; un pipe roto mantiene contrato0 sin excepción decorativa.

## Trabajo, memoria y latencia: gates medibles

Todas las cifras siguientes son presupuesto inicial de experimento, no resultado ni garantía del producto. Deben fijarse y medirse antes de implementar candidato, revisándose por evidencia sin declarar PASS por omisión.

- Entrada PNG hasta64MiB, dimensiones y multiplicación comprobadas antes de reserva; máximo16millones de píxeles, RGBA decodificado hasta64MiB, cero dimensión inválida. Codificación/transmisión con bytes y tiempo propios acotados; candidato inicial payload codificado máximo32MiB. No suponer que PNG comprimido pequeño consume poca memoria.
- Un trabajo gráfico activo y una solicitud siguiente reemplazable; resize/zoom agrupan estados, no cola ilimitada. Priorizar generación vigente; un resultado obsoleto no pinta ni gana caché sin comprobar presupuesto. Cualquier implementación de canales/threads debe acreditar tamaño real de cola y recolección.
- Cache memoria inicial64MiB con LRU y contabilización de bytes reales; límite separado para decoded, encoded y trabajo temporal. Caché de proceso no escribe junto al documento ni persiste datos por defecto. Presupuesto pico adicional objetivo256MiB sobre lector base, incluyendo copias IPC/rasters/payload/cache/hijo; publicar medición agregada y hard limits efectivos por SO, sin equiparar RSS/commit/VA. Si no puede contenerse o cancelarse el codec, no adopción hasta gate de aislamiento.
- Claves por bytes/contenido del snapshot+codec/version+escala/geometry/protocolo/presentación+alpha/fondo; no por ruta/mtime solamente. Cache hit/miss, cambio de archivo mismo nombre, cambio de fondo y eviction deben ser reproducibles. Verificar propio recurso del terminal: caché app limitada no demuestra caché terminal limitada. IDs/placings propios con límite, liberación y evidencia; nunca purge global que borre imágenes ajenas.
- Persistencia posterior de RV-202 mantendrá cache externa al origen, cuota/limpieza/versionado; primer corte sin persistencia no cierra ese criterio completo. No elegir formato de disco ni daemon ahora.
- Metas iniciales: UI reconoce cancelación en250ms; reap/retirada de tareas propias en1s; primera imagen de corpus pequeño en1s, distinguiendo lectura/decode/encode/emisión y inicio frío. Medir salida de usuario y señal por separado. Si falla, corregir o publicar límite revisado antes de aceptación, conservando criterio y objetivo completo pendientes.

## Gates de aceptación y evidencia

Todos están ABIERTOS: esta propuesta no ejecuta ninguno ni abre el siguiente sprint. Primero revisión adversaria del contrato observable; luego detalle técnico únicamente para gates cerrados o preguntas experimentales concretas, no arquitectura gratuita de todo E04.

| Gate | Evidencia obligatoria y condición de aceptación |
|---|---|
| G0 Contrato | Revisión independiente sin críticas accionables; mismo nombre/valor/precedencia Nu+CLI, referencia/códigos/modos coherentes. |
| G1 Terminal seguro | Probe con stdin ocupado por Nu/texto, respuestas ausentes/tardías/malformadas y teclas concurrentes; total deadline/colas; Ctrl-C/restauración/reap y segunda invocación. Sin mutation tmux/perfiles; inline/redirección sin raw. |
| G2 Raster acotado | Corpus sintético PNG válido/corrupto/extremo/alpha; límites antes de reserva; cancelación real durante decode y encode; memoria total/bytes/tiempo/retry medidos. API y features compiladas juntas sólo en futuro gate autorizado. |
| G3 TUI visual | Proporción cuadrado/círculo/regla, alpha completo/parcial, fondos, clipping/mover/resize/zoom/volver, q/Ctrl-C/fallo y nueva sesión sin residuos; escala extrema finita publicada. |
| G4 Inline visual | Texto marcador antes/después y prompt íntegros, imagen permanece scrollback; salida normal/cancelación y nueva invocación; resize terminal antes/después, sin borrar imagen válida al exit. |
| G5 Fallback/config | Auto/protocolos forzados/symbols/none; geometría desconocida vs override; NO_COLOR/colour/TERM=dumb; config inválida antes IO; wrapper Nu no tapa config ni pierde pipeline. Redirección byte-safe y jamás abre CONOUT para dibujar. |
| G6 Cache/lifecycle | Miss/hit/cambio snapshot/options/version/eviction; zoom+resize tormenta mantiene cola/memoria acotadas, no stale paints; límites de imágenes/placements propios terminal y liberación; fallo/timeout/cancel sin huérfanos. |
| G7 Matriz nativa | Windows x86_64: Windows Terminal release exacta candidata1.24 y WezTerm; macOS arm64 WezTerm; Linux x86_64 WezTerm. SO/CPU/terminal/build, font/DPI, geometría y protocolo registrados, ejecución real y QA visual en cada combinación declarada. Otras arquitecturas siguen pendientes, no eliminadas. |

Corpus futuro propio: PNG cuadrados y rectángulos vertical/horizontal, círculo+regla de píxeles, bordes con alpha0/128/255, muy pequeño,16M justo/límite excedido, dimensión desbordante y CRC/inflate corrupto; un nombre con controles y ruta Unicode. No copiar material privado de Clases. Extremos/decode-hostil sólo bajo presupuestos efectivos ya demostrados.

Separar evidencia de autor, AE independiente, adversario, bytes/controlTTY/ConPTY y visual real. ConPTY puede acreditar secuencias/modos/teclas/hijos y códigos, nunca píxeles, alpha, proporción, scrollback ni DPI. Build/crosscheck no es ejecución Windows/macOS/Linux. No automatizar GUI/tecleo de terminal por herramientas de escritorio; QA visual exige procedimiento manual o evidencia real aportada por canal autorizado. Mientras no exista, G3/G4/G7 pendientes explícitos, sin inventar capturas ni certificar soporte.

Cierre del corte requiere gates pertinentes completos para combinación declarada y documentación de pendientes; RV-201/202/203/205 conservan resto de matriz/formatos/persistencia/composición mixta pendiente. El objetivo final incluye gráficos raster útiles en terminales nativos objetivo, no Halfblocks como solución definitiva. Restricciones de acceso a QA son límites de evidencia, no permiso para reducir aceptación.

## Fuentes y grado de evidencia

Autoridad local leída: AGENTS, README, CONTRIBUTING, spec/REQUISITOS+CONTRATOS+DECISIONES, architecture/ARQUITECTURA, BACKLOG, VALIDACION, research/INVESTIGACION_TECNICA y GRAFICOS_API_PREPARACION. Fuente exacta picker11.1 inspeccionada localmente en `.tmp/graphics-research`; crate SHA256 registrado por investigación9593D10C61EC2007F429D1FAFC4F4A19FECDBAE4D06B531A2D505227FB82F5EA (no recalculado aquí). No dependencia fijada por este contrato.

Fuentes primarias consultadas2026-10-05; documentan protocolos, no funcionamiento de R3Viewer:

- [ratatui-image11.1 publicado](https://docs.rs/crate/ratatui-image/11.1.0): release/API candidato; la consulta web de picker no fue accesible, la evidencia concreta es fuente local exacta.
- [WezTerm escapes](https://wezterm.org/escape-sequences.html): iTerm2 y Sixel documentados; Sixel descrito preliminar/incompleto, por tanto protocolo disponible no acredita todo lifecycle.
- [iTerm2 imágenes](https://iterm2.com/documentation-images.html): anchura/altura y preserveAspectRatio; no atribuir a WezTerm todas las garantías de iTerm2 por compartir secuencia.
- [Kitty graphics](https://sw.kovidgoyal.net/kitty/graphics-protocol/): placements/datos/borrado distintos, relevante para cleanup propio; no borrar recursos ajenos.
- [Windows Terminal releases](https://github.com/microsoft/terminal/releases): release concreta debe registrarse en QA;1.24 es candidato, no medición local de versión instalada.

Alternativas técnicas propuestas para gate, sin consulta rutinaria al humano: adaptar negociación al propietario de terminal con parser acotado o comenzar por overrides+tabla acreditada sin probe; adaptar constructores de backend sin efectos tmux o sustituir backend si no se logra; inline dedicado por protocolo o referencia temporal mientras se acredita scrollback. Elegir entre ellas exige evidencia y revisión, no trasladar nombres de librería al estudiante. Ninguna alternativa es aprobación de producción ni recorta el soporte final.
## Respuesta a adversario R1

Informe revisado: r3viewer-graphics-contract-review-r1.md, SHA256 ACAEE57EFD985091ED5426E42AE4AD3505DC054A6856B623B76E986B8BF41FD5. R2 propone resolver GC-01 con gramática decimal ASCII sin ceros iniciales; GC-02 con tabla R/N/D/F, IO global1 vs representación local0, cabecera-only marcada y sin falso éxito; GC-03 con política de color efectiva exacta y casos Always/Auto/Never/config/redirección; GC-04 con selección explícita prioritaria y tabla de opciones sólo PNG, preservando tipos Nu y perfiles de números de línea. Se releyeron cli.rs/input.rs/config.rs/syntax_selection.rs/main.rs/nu_input.rs y contratos canónicos. Esta respuesta no retira hallazgos por sí misma: G0 continúa abierto hasta revisión independiente de R2. Todos los gates de ejecución/QA, S03, resto de formatos y las39tareas permanecen abiertos según su evidencia real.

## Conservación canónica y cierre G0

Contrato exacto R2 revisado independientemente sin críticas pendientes. Original SHA256 F444CAEC38A7744F6BF65BB83235BDC852BA31FD4CEF1651AFA67357F4A3AD0D; informe SHA256 1C27171F3E5494FD02CB795DBAA20659C8B87A0F69D7C0DB9A377E9B584E26C9. GC-01–04 cerrados y G0 cerrado; las expresiones anteriores de propuesta temporal/gates todos abiertos describen el estado de redacción anterior a esa revisión. [Informe final](../reviews/GRAFICOS-contrato-r2.md). G1–G7 siguen abiertos. Refinamiento paralelo: no abre otro sprint ni acredita producción/QA visual.
