# Admisión de proveedores b3: cierre de diseño R2

2026-10-09. [Contrato R2](../refinement/MARKDOWN-PROVIDER-ADMISSION.md) copiado byteexact SHA256 `5543377D1D72678949AFB5C5D4C065F589D40F4D28FB0457240D77E9D58C24A7`. Informe independiente original SHA256 `48092AE31028FF99E120B18CF30064907A07CD155DD1A457426329371D8E176D`, reproducido abajo normalizando sólo saltos de línea. Base42409ad: código idéntico a3db799c; avance previo documental.

IP-P01/IP-P02 cerrados en diseño; código y aceptación física pendientes. El contrato D2c anterior conserva el estado histórico abierto de esa dependencia; este cierre posterior permite implementar la biblioteca común, sin aceptar el futuro pump. No cierre de inspector/RV-306/QA ni modificación de cuotas D1 o de INLINE.

# Revisión independiente: provider ledger inspector R2

2026-10-09. Disposición: **SIN_CRITICAS_PENDIENTES de diseño R2 en esta frontera local de provider ownership**. IP-P01/IP-P02 quedan cerrados en diseño. Permite implementar el delta coherente gate/adapter/scoped factories con inspector; no acepta código,19tests del autor ni runtime. No depende de pump futuro ni autoriza contador globalG.

Propuesta PROPOSAL-PROVIDER-LEDGER-R2.md SHA256 5543377D1D72678949AFB5C5D4C065F589D40F4D28FB0457240D77E9D58C24A7 verificada. Criterios previos5FA1386628E86AD08C370562C6B4381FF436E0ECC63641954506A88EB865A6DA e informeR1 122049FE3B60EABDB16AD1EB920C1B770BDD78B5C6C9F83D9C727C8D79ED69A6 siguen intactos. Lecturas de interfaces canónicas reales: b2 window/slots/anchors/inputs, search factories/best/heading y PreparationAuthority genérica. HEAD observado42409add3643862ef9c24f1734710450baac3fc5 limpio; git diff desde base3db799c no registra cambios en src/crates/vendor/Cargo: avance documental únicamente.

## IP-P01 cerrado: adapter ejecutable antes de freeze

R2 nombra document_provider_gate.rs y document_inspection_preparation.rs y una rama inspectable_initial_input sobre el helper real que crea ConsumerBudget/source/context una vez. El gate existe antes del primer layout provider y sigue con View/input; una viewLegacy no se relabela inspectable, entrega ScopePending. Las sumas DI/DO concretas cubren Viewport/Search/Inspect/RetireAndInspect y especializan UNA PreparationAuthority genérica existente, sin nueva autoridad/kernel/sesión. start_document_owner/OwnedPreparation y take_validated con join son APIs reales comprobadas en source, no firmas del pump.

Constructor/capture reciben view/anchor/hit reales y actual scope, financian metadata antes copiar, y publicación conserva Binding/query/cancel/stale. La entrega del adapter se exige ANTES del freeze afectado del inspector; su TUI consumidor queda posterior. Esta secuencia elimina la dependencia circular indicada enR1. Las sumas y nuevos getters/factories aún no existen: su materialización real debe entregarse y probarse, no dar por válidos tokens manuales de un harness.

## IP-P02 cerrado: gates por provider real y scoped search completo

R2 inventaría cada entrada real de Search provider/retry/positioning, SequenceSlot Domain/Ref y sus call sites, Inspector y Heading.visit_title. Cada construcción scoped consume permiso previo; GatedText→GatedIndex mueve el MISMO proveedor/permiso y GatedRefIndex abarca su índice interno, sin doble slot. Los wrappers privados no dejan extraer un índice libre ni un permiso independiente; Value se destruye antes del permit. El objeto que aliasan slot/hit/cursor/page mantiene el wrapper íntegro hastaúltimoDrop.

Scoped backward walk usa BestDescriptor cerrado de secuencia/range/raw-start/owner/snapshot entre secuencias. Recorre/cubre la secuencia actual, financia descriptor y deja caer su provider antes de la siguiente factory; al final reconstruye sólo ganador bajo un nuevo permiso y trabajo real. El caso B pinned+bestRef+nextRef deja de necesitar3 índices: next no se construye mientras best siga vivo. Forward earlystop puede mover el actual, sin reprepare gratis. Busy/quota/posición final queda FoundPending con match acreditado; cancel no publica parcial y retry es explícito. Este es un delta scoped explícito del core, con regresiones reales antes adopción; el coreLegacy queda intacto fuera de scope.

No se atribuye count cap a llamadas D1/INLINE/legacy externas. Inspector exige view/gate acreditados; hitungated no se importa como provider invisible. Si una factory nueva aparece en implementación (incluidos samples), debe entrar a la misma frontera o no construir un provider nuevo; no puede quedar escondida bajo el permiso de la operación exterior. Este requisito es revisión del código a entregar, no un límite adicional.

## Registry, handoff y cargos consistentes

Weak apunta al pequeño PermitAllocation privado. El recibo de su body+Arc headers permanece en PermitEntry hasta purge/ledgerDrop; fixedGate/Mutex/entries/Weak y nuevos parentenums/handles/sumas llevan size/capacities reales. Strong permit sólo existe dentro del wrapper real; aliases del wrapper mantienen ocupado exactamente ese provider. Purge de Weak muerto y receipt ocurre antes de reservar/asignar candidato, sin lock durante D1/drop y sin Drop que reentre mutex. No ciclo strong Gate→Permit→Gate. Así no se conserva el gran índice muerto bajo un Weak con cargos liberados antes de tiempo.

retire_cached_provider es captureO(1) admitido sobre View real: mueve slot a retirement financiado sin últimoDrop en publicación. El job RetireAndInspect de la MISMA autoridad lo destruye antes de acquireC. input.old_slot/page/getteralias que retenga A mantiene su wrapper/permit: retirar el slot de View no promete libertad ni resta occupancy ficticia. Con otros aliases C devuelve ProviderBusy/Pending; después últimoDrop el purge libera capacidad real. A=B comparte1 y no necesita handoff innecesario.

Los datos b2 materializados conservan glyph/symbol/paint/OriginMeta/Packed.local, presentation_row/next/cell_anchor, TableAtDone, Source/Flow y receipt después slotNone; no se exige nueva copia de índice para eso. Navegación/inspection que requiera reprepare pasa nuevamente gate/Consumer/Work y puede quedar Pending con dos inspectores pinned. FalloC conserva oldInspectorB/page/continuación y frame; cacheA no se promete reconstruida. History retiene bookmarks/metadata completos, no wrapper/permit/provider incidental.

## Próxima aceptación de código

P01–P07 preservados y P08–P13 añaden los testigos que faltaban: backward Titles propietarios reales, Busy antes factory, Ref interno cuenta1, Weak tombstone hasta purge, aliases old_slot, cacheNone con anchors/draw iguales, un kernel/jobs/join y parentmetadata nearquota. Son obligaciones prospectivas, no evidencia runtime. Scope/source/budget identidad reales y queryRevision independiente de viewBinding deben comprobarse en capture/publish.

No nuevos límites ni cambios de cuotas/perfiles globales. Las reservas/fórmulas deben medirse en compiled layouts, conservando old+new y cargos hastaúltimoDrop. Todo el delta debe congelarse conjuntamente o como dependencias implementadas anteriores al inspectorfreeze, con hashes/binary y AE propia sobre view/parser/factories reales. Sólo entonces puede cerrarse aceptación física2; la presente disposición aprueba el diseño implementable, no su cumplimiento.

## Criterios prospectivos P01–P07 preservados

Se reproducen sólo los oráculos de R1 SHA256 `65E1EE65358A5C3F0F46EDCAE0403753A8397E4FD68C7B1F0AA34838D40BDEC4`; su propuesta intermedia queda privada. Las reglas R2 prevalecen: en P06 el slot ya está ocupado antes de cada factory D1, nunca se registra por primera vez después de construir el índice. P08–P13 y los detalles ejecutables están en el contrato R2. Estos oráculos siguen siendo criterios previos, sin PASS de implementación.

P01 actual view cache A + old inspector B + requested mismatched C: cap-only
admission returns typed Pending with old page/continuation/source binding intact.
No new Index is prepared. Scope and byte charges unchanged after failed permit.
P02 same setup with A's ONLY actual alias in materialized view: closed cache
handoff frees A physically in existing owner; B survives; C admitted; activeB+C2.
Failed/cancelled C preserves oldB/page/continuation and current frame/anchors,
while cacheA is explicitly empty. New actual permit/operation invalidates stale
candidate, never old accepted page through a reused numeric revision.
P03 A shares B's actual provider: view+hit/page aliases add charges but occupancy1;
newC occupies2, no needless eviction/copy/index clone.
P04 A detached but actual retired/other page still aliases A: weak liveness keeps
A counted, C remains Pending; dropping unrelated registration cannot free A.
P05 two inspectors B/C retained after old view cache removal: thirdD Pending;
explicit last aliasDrop ofB frees exactly one slot, D then admissible. History
bookmark/reference metadata alone has zero physical-provider occupancy.
P06 permit cancellation before Ready releases candidate slot/charges only; no
old page mutation. Ready consumes the actual reserved slot then registers exact
allocation; arbitrary budget/provider/key/same bytes newDoc tokens rejected.
P07 scope guarantee only combined owner's producers: direct accepted D1 profiles
outside this owner do not gain a new global cap or changed accepted behavior.
