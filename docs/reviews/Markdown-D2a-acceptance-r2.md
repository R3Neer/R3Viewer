# D2a R2 — revisión y aceptación independiente delimitada

2026-10-06. Autor unix_r3_review; revisor graphics_ipc_acceptance distinto. Freeze MANIFEST SHA256 `73C19AE35C494248FFC94AFCE7139C1EA95CCDB8AE8A70BFFCAB2CF7ABAFBC84`,415registros; CONTEXT-MANIFEST365 SHA `CD73345DCED222C7C53390938EF5881CD92182CCF2F432DBCDCE589FB684C684`. Delivery exacto SHA `2CBC8DF93478230A9FA90CD3A43D77A3F0FE0299D1519CA6346F34A773ACBDEE`. R1 no aprobado y su fallo de Clone/ownership se conservan; no se revisó ni ejecutó WIP posterior R2.

**Dictamen: SIN_CRITICAS_PENDIENTES dentro del código D2a R2 y la aceptación pura descrita.** Aceptable como incremento Flow/perfiles/navegación lógica. No es aceptación D2b/c, layout/TUI/backend, Anchor de glyph/afinidad visual, search/sourceactions, routing/inline D3, color D4, Unix, SLA, QA visual o producto/RV. Ningún código compartido adoptado por este revisor.

## Identidad y alcance fuente

Verifiqué415registros antes y después; copia privada365archivos exacta antes de runtime. Después364 siguen byteexactos y lib.rs conserva el prefijo completo candidato más newline/sufijo cfg(test) propio identificado; no cambio de producción. No se usó bin autor como nuestros controles ni PRODUCT.patch para ejecutar. Cargo/lock/dependencias/resources/model/parser/protocol/controller/validator están en el contexto exacto; nuevos módulos Flow/navigation/profile/test y deltas ConsumerBudget/DocumentWork/lib fueron leídos. Se contrastó el diff D1 del perfil/default; algoritmos de segmentación/proyección no se reescriben.

CursorState es privado; FlowCursor no Clone/Copy, campos cerrados y Charge que conserva ConsumerBudget/Document. start falible reserva size_of real y1slot ANTES de materializar. Target endpoints sólo prestados, sin Clone público ni escape de su cursor; índice cobra su objeto/cursors/scratch y2endpoints, puntos/folds ChargedVec cobran capacidad efectiva. reserve_points no pierde cargo si falla el límite de puntos; Drop devuelve bytes/slots. El cierre no es disclaimer de futuros consumidores: existe ownership real para cursor exportado.

Flow itera Items con stage/depth65, incrementos checked, no ID-preorder. Preparación en dos pasadas usa mismo Work de preparación, cobra copia/records/Targets/checkpoints antes del paso y polls cancel vía admit. Resolve privado coteja identidad, usa endpoints retenidos o binary checkpoint/replay con mismo Work; next_event copia candidata sólo tras cargo y commit sólo si next termina. Fallo no muta cursor. locate_run usa scratch propio cobrado y Work cancelable; no ejecuta parser/IO/layout. Fold overrides sortedunique/callout válido se cobran y producen identidad nueva, filtran Items del body preservando descriptor/Title y Source inmutable.

start no recibe Work/cancel porque crea un único owner constante y no hace traversal/texto/IO. No se exige que todo constructor falle por cancel ni se atribuye coste gratis a futuras consultas dentro draw. D2b/c deberán preadmitir las copias/operaciones del frame y no ejecutar starts repetidos dentro render como Work gratuito. Aquí start demuestra admisión física, no SLA ni permiso para un loop ilimitado de preparación en UI.

Navigator cobra objeto/current/scratch e historial128 antes de densidad Flow. Movimientos home/end/seek no push; jump sólo si anchor cambia, pending→ready no duplica retorno. Push fallido conserva current y back consume entrada. States tienen campos cerrados; selected_run se obtiene por traversal y DomainBody válido; foreign FlowLocation rechaza. D2a guarda posición lógica raw UTF8-start/end acreditada, no hace todavía resolución de cluster/afinidad visual. Profiles planificados con tamaños checked son estimadores: alloc real debe seguir obteniendo cargo, no permisos gratuitos ni desalojo de old/current. DEFAULT mantiene1536/1536/512; perfil menor no trunca excepciones como índice completo, falla Budget.

## Controles propios previos y finales

MANIFEST preoutput propio `6270B54CA32D1566F1F9FC27D96951A635F9F9986A44512BB8FDBEC646667A2C`:5fixtures,15grupos anteriores a candidato. API-EXPECTATIONS-before-output aclara, antes de runtime, Work0 de cursor frente anchor seleccionado Pending y roles vacíos sin marcador Heading ficticio. No se cambió ese manifiesto ni se derivaron goldens del candidato. Expectativas raíz conservan su autoría; no son ejecución nuestra.

Bin final propio `bin/d2a-independent.exe`, SHA `2C723D14309E0EB51B2174512673119986671FBB7B04105B403E99DD5C549410`, before/after idénticos. Sufijo test SHA `49B395401BE983EB36D9312B567963FD9F0A3A2959222DB172530F95632F9B02`. TEST-attempt5-final-exact-bin.log: **7 grupos PASS,0FAIL,277filtered,0.03s**, terminal exit0. No nuevos workersOS, Jobs, servidores, perfiles o procesos auxiliares del producto. No sesiones activas quedan:22711/70193 terminaron101, compilaciones posteriores fueron terminales.

Cobertura propia real:

- Items FIRST→ChildCode MIDDLE newline→LAST→TAIL; permutación de bloques de candidato real parser, reescritura parent/Items/DomainBody, reempaquetado ItemPool y quarantine host válido antes de Flow. Orden semántico igual; hash, TextRefs y Origins permanecen iguales. No modelo falso ni proof debilitado.
- Resource entre LEFT y combining/RIGHT conserva descriptor y orden; ALT/TITLE no aparecen como Runs main body. Heading/callout/Code vacíos no fabrican texto; primeros/terminales roles Callout/Rule acreditados, no glyph Heading inventado.
- Fold body elimina BODY, conserva rol callout/Title lógico y TAIL; old location rechaza y ambas Flow simultáneas cargan el mismo budget. No afirma render de Title ni widget fold implementado.
- Work0 next_event rechaza sin avanzar ordinal; event_at endpoint usa trabajo positivo y no Root ni checkpoint replay; consultas sucesivas acumulan MISMO Work. End LAST tiene raw_byte4; todavía no prueba última página/reflow/render.
- start suma size_of FlowCursor y1slot, Drop restaura. Reserva de puntos hasta4096 con API crate-private de admisión, explícitamente test, rechaza otro start sin fuga; no se presenta como4096cursores físicos materializados. Pool real de cursores crece hasta rechazo Consumer/cap, nunca sobre1MiB/4096. Caso adicional Weak<Document>: destruir índice/documento/budget externo mantiene Doc exclusivamente por cursor hasta su Drop.
-170end/home no llenan historial; seleccionado con Work0 permanece conocido/pending sin retorno; jump al mismo anchor pending→ready no añade.128transiciones reales aceptadas;129rechazo conserva current/history y back reduce127.
- Cancel previo prepare/locate rechaza y devuelve cargos; foreign índice sobre mismo Document rechaza location. Profile adaptativo limita dos slots y total de bytes/checkpoints restantes; tercero inválido. Perfil exceptions0 sobre cluster16385B no da índice truncado: Budget.

Los15grupos son criterios de revisión, no15tests PASS independientes. Los puntos de límites/copias/stack/cancel checked y cierre de raw State imposibles de fabricar se combinan con inspección estática. No se ejecutó frontera window131072/frame256KiB (D2b), oversized512 exacto/513 o fixture32MiB nuevamente: invariantes D1 previas y DC17 autor se conservan como evidencia de dependencia, sin atribuirnos esos resultados. No fallback Reference visible ni Source↔Doc de UI acreditado por Navigator puro.

## Fallos e intentos preservados

Attempt1 sesión22711 exit101: harness include_str! con ruta un nivel corta, texts en vez getter text_refs, assertEq FlowLocation sin PartialEq. Se preservan log, test y lib compilada; ajuste sólo API/rutas, ninguna expectativa ni producción.

Attempt2 sesión70193 exit101: linkerLNK1114 error5 al sobrescribir biblioteca en target propio bajo sandbox; no runtime. Repetición escalada autorizada sobre misma copia/target privado, sin rescate de proceso ni target autor. Warnings canonicalización/hardlinks y diagnóstico Python de localización quedan documentados, no se declaran inexistentes.

Attempt3 escalado compila7.79s:6PASS/1FAIL0.02s. La permutación fallaba quarantine ANTES de ejecutar Flow porque no reempaqueté ItemPool: validador exige first_item contiguo en orden BlockID (fuente primaria document_validate). Corregí exclusivamente la mutación de prueba para conservar ese invariant; no relajé validator ni cambió el expected nonpreorder. Primer FAIL y código exacto permanecen.

Attempt4 sólo caso afectado: compile3.94s,1PASS0.00s. Attempt5 ejecutó directamente los7propios sobre exe final copiado/hash antes-después. Esa repetición pequeña se justifica por cerrar identidad binaria: el bin anterior con6PASS no se había preservado/hash antes de recompilar el fixture, por lo que no se presenta como bin exacto final. No se repitió suite autor, gigante32MiB o A–C.

## Disposición

No defecto productivo pendiente identificado en las fuentes de este corte. R2 cierra la fuga de cursor retenido de R1 con autoridad efectiva. La adopción debe conservar identidad de fixtures/oráculos CRLF frente reglas Git y correr checks afectados sobre composición con Source APIs nuevas; es trabajo del integrador, no un resultado inferido de esta copia basada2e89abc. La planificación de perfiles no acredita memoria total del framework/RSS y la futura retención de estados lógicos/Points/ventanas debe seguir cobrada en D2b. Código puro aceptado no anuncia Markdown enriquecido completo ni cierra backlog/RV.
