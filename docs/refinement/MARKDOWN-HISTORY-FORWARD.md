# Decisión focal: history con atrás/adelante

2026-10-09. markdown_actions_adversary, encargo explícito del coordinador. Sólo análisis de almacenamiento futuro; sin código ni reapertura de matcher/provenance. Canon leído `docs/refinement/MARKDOWN-D2B3-SEARCH-ACTIONS.md` SHA256 `85A78F7EAC0BD6FA5DDF0862D89EA7F8AB21216F469009D5AA906A93238D5395`; Navigator actual SHA256 `6ECE4B94C0EE16626ECE2629E491E76A0171E38EF2147359624EBFB62E8B0CFF`.

## Decisión

Timeline cerrado con cursor compatible con el requisito original y los oráculos128/129. Requiere un delta focal explícito del contrato history ANTES de implementarlo: Back consume disponibilidad anterior, pero NO destruye el destino que queda delante. Forward es recorrido del mismo historial; una transición nueva confirmada trunca la rama futura. La elección de almacenamiento sigue abierta en b3R2, de modo que concretarla así no invalida aceptación del core search ni el cierre de B3-R01/B3-R02. Tampoco cierra RV-306.

## Evidencia y relaciones reales

- REQUISITOS.md113 exige historial atrás/adelante con posición; BACKLOG.md58 RV-306 exige esa cobertura común entre adaptadores.
- Canon b3 §3 declara128 anteriores+current, owner único, rechazo129, no eliminación de la más antigua, prevalidación y Back sólo tras remap válido. Su frase 'Sólo Back validado consume entrada' y el nombre candidato PopReturn necesitan la precisión anterior para no prescribir destrucción.
- Navigator::commit (document_navigation.rs114-122) almacena el current anterior al salto en ChargedVec; Navigator::back (236-244) valida la ubicación, hace pop destructivo y sustituye current. No tiene cursor/Forward, por lo que no se puede adoptar tal cual como almacenamiento final.
- NavigationStorage::new (251-262) reserva128 State + current/candidato fijo; ChargedVec::pop realmente elimina (document_consumer_budget.rs161-163). Puede reutilizarse la reserva/política presupuestaria, pero el mecanismo y tamaño de entries b3 necesitan medir capacity/size reales; añadir un Vec Forward128 independiente sería incorrecto.

## Invariantes del delta

Sea m el número de transiciones retenidas y k el cursor: 0<=k<=m<=128; hay m+1 estados incluido current. Disponibles atrás=k; delante=m-k; atrás+delante=m<=128. Una autoridad y quota conjunta para Documento/Fuente/inspector/heading/hit. Es válido implementar129 slots con current dentro, o128 alternativas más current externo, con cargos reales equivalentes. No mantener dos cuotas128 ni un current duplicado sin cargo.

Back preparado valida el estado k-1 y sólo commit mueve k a k-1; Forward hace lo mismo con k+1. No append, no eliminación, ningún cursor se mueve por request/pending/fallo/cancel/stale. Ambos guardan la posición lógica/hit/projection_part actual al abandonarla y restauran el destino por anchor acreditado + remap del Flow vigente; no reutilizan ordinal invalidado. Geometry best effort. Current y candidato/leases/Source originales siguen sus ledgers físicos existentes. Timeline no autoriza128 documentos ni modelos Source gratis.

Nueva transición retornable desde k: prepara candidato, captura la posición de current y prevalida estado/cargo/capacidad; sólo commit descarta estados futuros k+1..m y añade destino, dejando m'=k+1 y k'=m'. Al estar en el extremo m=k=128, intento129 falla sin cambiar current/cursor/ninguna entry. Tras Back con k<128, una rama nueva sí cabe aunque antes m fuera128: reemplaza rama futura, nunca descarta la más antigua para fingir capacidad. Preparación fallida conserva íntegra esa rama. Same-anchor/no-op, viewport/resize/scroll, paginación y actualización de posición pending->ready no truncan futuro ni añaden transición.

Nombres de efecto candidatos deben cubrir None, AppendBranch, MoveBack y MoveForward, o semántica cerrada equivalente. PopReturn sólo podría conservarse si se define explícitamente como cambio de cursor, no ChargedVec::pop. Commit atómico prevalidado: el truncado no ocurre antes de comprobar candidato y expected state/cursor. Guardar/restituir metadata fija preadmitida evita clones de ventanas/Document/índices o trabajo largo durante commit.

## Oráculos focales antes de implementación

1. A->B->C: m2,k2. Back: m2,k1, prev1/future1. Forward restaura C y su hit/posición; m2,k2, sin nuevo retorno. Repetir ciclos no crece.
2. A->B; scroll B a B'; salto C; Back restaura B'; Forward restaura C. Scroll tras Back conserva future; salida de B' actualiza su posición al abandonar.
3.128 transiciones consecutivas: m128,k128 y128 atrás disponibles.129 rechazada intacta.128 Back: m128,k0, forward128;128 Forward: m128,k128. Una quota conjunta, sin129 estados adicionales.
4. Desde m128,k127 (un Back), salto X confirmado: m128,k128, futuro anterior descartado y Forward vacío. Otro salto al extremo falla129. Si salto X falla/stale/cancel/pending, m/k/futuro originales permanecen.
5. Desde A->B->C, Back a B; salto D confirmado: A,B,D. Forward no ofrece C. Salto D fallido conserva C; misma selección B/no-op no borra C.
6. Mezclar Documento->inspector->heading->Fuente en un timeline. Back/Forward conserva pane/domain/anchor/hit/projection_part y modelos/identidades acreditados, incluido control tras resize/fold. Fallo de remap o SourcePreparationLimit no mueve cursor ni destruye futuro.
7.400 viewportmoves sin transición: m0,k0; preparación y paginación no añaden entradas. Una SourceFoundPending inicial no mueve timeline; SourceReady confirmado añade sólo su transición real.

Dictamen focal: COMPATIBLE_CON_DELTA_DE_HISTORY. Acordar esta precisión en el refinamiento canónico y en la interfaz del almacenamiento antes de implementar history. No es crítica del core search aprobado, ni exige ahora pump/teclas/runtime/QA ni ampliar las APIs genéricas Binding.
