# Revisión adversaria focal de diseño history R2

2026-10-09. Revisor markdown_actions_adversary; autor markdown_search_implementation. Alcance: almacenamiento timeline y preparación/commit de bookmarks antes código. No implementación, nuevos tests, publicación, perfil ni edición canónica. No reapertura del core search R2 adoptado.

## Snapshot y autoridad

HEAD leído12d28a48df7701bea5cf2d1e60b66c9674d2ef41. PROPOSAL-HISTORY-R2.md SHA33E0CB4243D5E5194F4A4A18A86F5F4093FD6E2CBBD950B75652BF356B288E94, junto a los apartados no sustituidos de R1 SHA040DB973BEE763F27D029F977209505680E1AE0B3D58E08DE7DE1D84036F2699. Coordinación H01 ROOT-REVIEW-HISTORY-R1.md SHAB2CDDDDA4906684E709634EEFED865B79846EA50238CF14B2AD0D1C4D6443C2A. Hashes verificados.

Contrastados REQUISITOS113/RV-306, canon b3§3, MARKDOWN-HISTORY-FORWARD.md SHAB5A5000CC16384A39B7830A535DDB1D723D24AAE74A96C4EF33B319B3E78DAF6 y b2R2 PresentationAnchor SHA4D379672543404918C1793632180013B4B892286F36DF616BAADB605B80F42CA. Readback focal actual DocumentView::commit/check, PreparationAuthority::take_validated y PreparedSourceContext/PreparedSourceModel. b2 aún requiere freeze propio: estas firmas/representaciones propuestas no se fingen APIs integradas.

## H01 — RESUELTA en diseño

R2 sustituye explícitamente las variantes incompletas Copy de R1 por PresentationBookmark cerrado que conserva todos los componentes acreditados del anchor b2. Ref retiene owner Resource o BlockRole, field, TextRefID, local byte y projection_part; Resource conserva ownerRun/resource/lane/posición; Domain conserva LayoutAnchor. Tabla conserva RowID/CellID seleccionado, Domain/LayoutAnchor y progresos de TODAS las celdas locales At(anchor) o Done. Sólo caches geométricas/ordinales Flow se eliminan. No se reinicia un título largo ni una celda ya terminada al volver/resize. ResourceOwner adquirido en coresearch se preserva aun con TextRef común.

TableBookmark admite representación owned o alias inmutable charged, elegida sobre el b2 freeze real. El diseño ya no cobra una representación Copy reducida ni promete128 filas/tablas gratis. Reserva fixed metadata y cada payload/alias/capacity efectivo con old+new vivos; límites físicos pueden rechazar candidato sin mover cursor/descartar oldest. Gates añadidos contrastan KIND/INFO/LABEL/title largo, propietarios distintos de mismoRef, mezcla At/Done y cambio vertical/columnas/suffix+TAIL. La crítica original H01 queda cerrada por estos cambios.

## Timeline, retirados y commit

Sin nueva crítica. Cuota semántica conjunta0<=k<=m<=128 y129 estados incluidos current; Back/Forward preservan rama y sólo mueven cursor tras remap válido. None actualiza current/captured anchor y revisión checked, sin destruir futuro. Append tras Back trunca futuro exclusivamente al commit; al extremo128 rechaza129. Los siete oráculos canónicos no se cambian.

Retired container se reserva/cobra ANTES de Ready/preflight. Su bound129 entries es consistente: el peor Append desde k0,m128 retira128future +current1; MoveBack/Forward reemplaza current+target (2), None sólocurrent. No se confunde ese bound de cantidad con bytes ni con table payload: los bookmarks retirados llevan sus Charges/leases y el receipt los retiene hasta Drop fuera de publicación. No last-owner Drop de anchor/model durante truncado. Copiar/aliasar candidatos exige cargo antes, y abandonar un candidato mantiene su lease hasta Drop. No se usa el descarte futuro como crédito anticipado para una allocation todavía coexistente.

El mismo protocolo requiere prevalidar ambos lados y adquirir pane/history guards antes de cualquier reemplazo, con todos los índices/capacidades/revisiones preparados. Ambos apply son infalibles; no validación/cancelación posterior a truncar. El adaptador real del pane debe cumplir esa misma frontera, conservando hasta retirement lo que su reemplazo desplace; este informe no afirma que DocumentView::commit actual ya proporciona esos guards. Freeze b2 y accessors/prevalidation cerrados son dependencias expresas antes código/composición.

## Autoridad y Fuente

R1/R2 proponen wrapper privado que llama por sí mismo a take_validated de la autoridad EXISTENTE tras join físico y sólo entonces produce JoinedHistoryReady. No raw OwnerReceipt(joined:true), flags caller, State arbitrario, PositionPending, SourceMatched o heading sin remap. Prueba real del pane valida key/snapshot/oldFlow/candidate selection; Source e inspector consumen sus propietarios/proveedores reales. Ningún kernel/worker/proof booleano nuevo. Identidad/revision/m/k exactos invalidan candidato tras un None posterior, evitando ABA.

Fuente conserva único Box<PreparedSourceModel>, selección preparada y receipt Source real; history almacena bookmark, no visible/model/índices clonados. HistoryScope captura UNA vez el binding/contexto real y pointers Document/Source. Aplicar la instrucción explícita de ROOT-REVIEW-HISTORY-R1: obtener el PreparedSourceContext original ya retenido por DocumentView mediante accessor cerrado; no llamar from_document de nuevo por acción, porque crea otra input identity. Los modelos/Arc/cargos Source permanecen en su ledger separado; el bookmark y alias metadata se cobran donde realmente viven. Fuente no preparada conserva pending y timeline original.

## Dictamen

SIN_CRITICAS_PENDIENTES de DISEÑO focal para R2 + apartados R1 conservados. H01 resuelta; no nuevos hallazgos accionables. Autoriza concretar tipos/adaptadores/cargos sólo tras freeze b2 real, conforme dependencia declarada, y después implementar/revisar. No exige tipos ni cifras físicas inventadas antes de ese freeze. No acredita código history, SourceReady, pump/session, Inspector, QA, Unix ni RV-306/producto completos. Esta revisión no invalida la aprobación independiente del core search.
