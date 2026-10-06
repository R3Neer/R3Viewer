# RefID/roles R1 — revisión y aceptación independientes del factor

Dictamen: **SIN CRÍTICAS PENDIENTES dentro del factor biblioteca RefID/roles**. No aprobar b1 R2 ni su navegación C1 rechazada; no framework, base completa, renderer, pump, CLI, integración inline ni Unix. Adopción/combinación requiere mantener esos gates separados.

## Identidades y método

Freeze MANIFEST 38D10592F554973234B4D6C51CBBAEA00B85D698621E125A5FB83771C4BE08B2,18 registros; CONTEXT D8A8752490DAE2E729D097A180FB4A3E3AABBBD00D6004C0D7347B975CFFCB7D,606 registros. Readbacks propios antes/después sin discrepancias. Tres fuentes exactas del delta: document_text.rs BFA652D6599F442383299574BECB46546BC8BB8BD23B5B11C1DDBEBBC228B200; roles81EB49B3D028637D5738859876A64523F62E2841F69532BEF09B9BAEADEF9A8F; tests7EAA3F2CA59E6E81AA3D647B67EA835B85B1CA61A9ED5422563F41242442503C. Contexto compilable copiado registro a registro a copia nueva; no funciones modificadas, sólo sufijo cfgtest/harness propio. Ninguna mutación de freeze o shared.

EXPECTED-before-output propio B3130D26FE2AE3DE74693BC16AF7CD29A642FBF65A95583F96164C1CA5D21CD1 precede compilación y runtime. Outputs autor no usados como goldens. BUILD-attempt1 exit101 por helper de test asumió accessor TextRef.id(); corregido sólo selección por índice real, antes de cualquier runtime, conservando HARNESS-attempt1 y HARNESS-CORRECTION-before-runtime. Expectativas intactas. BUILD2 terminal0 1m25s, target propio offline/locked/escalado. Bin propio60A24E6BC4287B15003F61A6115B3FD31D573E3BA91C75307770F11706A5394C; runtime sandbox Windows.

## Resultados propios

TEST-attempt1.log: cinco grupos nuevos PASS 0.04s.

- Source literal R + U+0085 + ESC + q. Payload seguro `R\u{0085}␛q`; cuatro coberturas originales local0..1,1..3,3..4,4..5. ExactSource/ControlProjection/ControlProjection/ExactSource; rangos Source físicos suman inicio del TextRef, id real conserva procedencia. No raw byte proyectado inventado.
- Decoded título con entity numérica y ZWJ: b+marca, espacio, mujer-ZWJ-ordenador. Tres clusters de anchuras1/1/2; seis scalars semánticos tienen EnclosingToken del Origin, no ExactSource fabricado. RefIndex rechaza raw_anchor y RunCoverage antes de callback. TextRefView préstamo coincide con pointer del payload real; intervalo UTF8 inválido rechazado.
- Consumer exacto: foreign Document/budget y ID inexistente rechazan sin cargos retenidos; cancel previo rechaza igualmente. Index mantiene Arc exacto del Document y cargos del presupuesto; drop de Point e Index devuelve used a baseline. Consumer completamente ocupado rechaza nueva admisión sin éxito parcial. step/coverage con cancel previo fallan; no se prueba aquí carrera de cancel en allocator ni restauración OS.
- Título con b+9000 marcas: unit de18001 B oversized completa pese hint4; siguiente Q accesible, EOF real; misma instancia Work cobra más de18001, ningún recorte o reinicio para green.
- Roles reales de Header nivel2, lista ordenada inicio5, task unchecked y callout inicialmente cerrado; Image y MathInline son PendingRequired. Descriptor input/raw/ALT/title y Origin se conservan separados, Origin es préstamo del registro real. TextRefView Source conserva pointer y Exact(range) del texto, no IO ni color/render fabricado. IDs inexistentes rechazados.

El factor modifica payload/backprecontext de Runs también: se fijó suplemento RUN-REGRESSION-before-output antes de ejecutar tres tests D1 existentes y pequeños (autoría previa, ejecución independiente nueva): dc01_a_accent,dc01_zwj,dc03_actual_hardbreak_is_boundary. Todos PASS con exact filters, un test por log; Source multirun conserva glyph entero y cobertura Run; HardBreak Synthetic/sourceNone y marca después en línea1. No ejecutar dc04_32m, suite21 ni matriz gigante. Son tres regresiones ejecutadas, no tres tests nuevos propios ni aceptación global del motor.

## Revisión estática completa del delta

PayloadSelection::TextRef(id) es selección real de TextRef validado. Ordinal interno0 es cursor de pieza; no se construye Run/Domain ni se publica como BodyFlow. prepare_selection retiene Document Arc inmutable, Identity propia y cargos scratch/index en Consumer que belongs_to exige exactDocument. TextRefIndex cobra sizeofSelf, lookup real y perfil checked antes de preparar. Caché Work carga sizeofTextRef al entrar y1 cada acceso ulterior; no devolución de Work. Runs mantienen su caché sizeofRun y cargo TextRef al entrar. Cambios payload y precontext recorridos: el motor mantiene proyección, GraphemeCursor y sparse index, synthetic Break/Resource semánticas originales, cancel admission existente.

RefCoverage valida rangos proyectados y UTF8 de partes, describe local semántico original; Source usa inicio+bytes del scalar físico con checkedadd. Decoded alwaysEnclosingToken obtiene Origin real. ControlProjection sólo cuando expansión segura cambia longitud, flag separado bajo Decoded; no usar semantic_text crudo para backend. Guards explícitos de selección cierran locate_raw/RunCoverage/raw_anchor para Ref; own negativos acreditan ese camino. Identity de Point sigue motor real, no wrapper fake con campos Source reetiquetados.

Roles enum cerrado se deriva de registros validados con cargos de Work por estructura. Descriptor y TextRefView lifetime enlazados a Document prestado; sin String flatten/copia de Domain/Resource, sin hacer petición de destinos. Resource estado único PendingRequired; raw/input/ALT/title/dimension intent separados, getters no afirman gráfica. relation comprueba orden/límite y char_boundary antes de sumar ExactSource o devolver Origin envolvente. Cancel/errores y consultas cargadas mantienen los contratos existentes del Work de biblioteca.

Las medidas del autor de Buffer/Ratatui no se cuentan como admisión framework. Este factor no materializa buffers ni ofrece Activation/Styles/OriginalColorProof ni renderer completo. Consumer cargos comprobados son requested/known capacities; no RSS/allocator global/SLA. No nuevos helpers OS/Job/nativefault. Readback posterior intacto y ningún handle de sesión activo.
