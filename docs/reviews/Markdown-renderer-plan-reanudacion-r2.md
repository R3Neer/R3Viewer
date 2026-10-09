# Revisión adversaria owner + primer renderer R2

2026-10-09. Base0cb0fe65363dae463d739502d5d3058cc740909c. Propuesta exacta OWNER-RENDERER-PROPOSAL-R2.md SHA886AA43DB160EE314988EA64E05C9A05BDBA2B56CD029A64969807D33CA182B4 verificada y leída completa. R1/review preservados. Alcance primer renderer texto/spanstyles, owner+packing+perfilWork explícito; no pump/b2 completo. Revisión de diseño, sin runtime nuevo ni código productivo cambiado.

## MD-RENDER-R01 cerrado en diseño

R2 abandona confianza en workaround automático de ForcedWidth. r3_apply_buffer_admitted amplía diff_iter prestado: para updatewide emite referencias blank+stylebase de cada continuación ANTES del glyphbase. No nueva segmentación/ancho, Vec o clone. Skip permanece sólo en diffnativo; preclears se emiten directamente a Backend.draw y por ello no pasan por is_skip(next[j]) que invalidaba una reparación limitada a TrailingState.

Fuente CrosstermBackend0.1.2 lib.rs232..274 imprime symbols yStyle/MoveTo de cada tuple; no reinterpreta diff_option. Iteratorprestado con coordenadas controladas y referencias de currentgrid es viable. Límite2*nCellschecked y fila/inbounds cerrados cubren expansión sin ilimitados updates. El costeextra de visita/copias/formatter/preclear se reserva en MISMO Work antes de operar. R04 incorpora ab->VS16 y inversas/CJK/ZWJ/styledwide/narrow/blanks.

El mantenimiento de lastknowncursor_pos debe observar los updates del iteratorAMPLIADO, como parte de preservar flush/lifecycleoriginal indicado por§3; confirmar esto sobre código candidato. Dejar apply original de Legacy intacto es compatible con el hook estrecho. No aprobación del comportamiento de terminal por esta lectura; el control con writerVT/ConPTY y trailingprevio obligatorio comprobará la reparación.

## MD-RENDER-R02 cerrado en diseño

R2 fija1italic/2bold/4crossout, links>0=16underline,8code/linkRoleAccent y otrotextoRoleValue; Palette vía config.style. Colournever Styledefault sinmodifiers. El ejemplo a**U+0301** fija clusterenteroORstyles2/bold sin SGR interno. Mapping coherente con document_inline actual y separa stylebitsdeRolepalette. Roles estructurales posteriores no se atribuyen al primer renderer.

Se mantendrán oráculosstyle pequeños del informeR1 al aceptar candidato, junto a ASCII dense: roundtrip dePackedClass no basta para aprobar estilo efectivamente emitido. Esto forma parte de criteriosya resueltos, sin gate global nuevo.

## Perfil y nativepaths

DocumentWork4MiB candidato explícito es aceptable como hipótesis medible. Dosreadbacks9600Cell a48B921600 +layout120x40 histórico1327023 ya rebasan2MiB; no hay fundamento para exigir2MiB sin ocultar costo. R2 conserva techo único/SAMEWork, preadmisión, gasto no reembolsado, historiafallos Work256K y memoriaWindow128K/Frame256K/Consumer1M/arena64M/símbolo16K/checkpoints4096. Source text_engine/Workpreparation/Legacy no se elevan.

Fórmula4188079 frente4MiB4194304 deja6225B-equivalenteWork: margen estrecho claramente etiquetado predicción. No se valida como cota final ni garantía Normal. Gate exacto debe incluir todas visitas, reconstruccionesGlyph/rangos/copy/stylechecks/readbacks/diff/reset/formatter/preclear/cancelchecks y metadata. Si no alcanza, bloquear adopción y medir/refinar importe explícito con revisión; jamás refund o allowance nuevo para cerrar verde.

get_frame/apply APIs públicas pinned siguen utilizables dentro ownercerrado; no necesita r3_try_draw_checked nuevo. HookApplyadmitted debe preservar cursor/swap/reset/backendflush/CompletedFrame/count y failure-after-swapPoison; error IO después salida no garantiza oldframephysical intacto. Resizes/reservasfallibles conservan capacidad1 y oldgeo antesfailure2; receiptphysicalincluye symbols/cargos hasta Drop. Código exacto deberá demostrar esto.

## Dictamen

SIN_CRITICAS_PENDIENTES DE DISEÑO para SHA886AA43D, limitado al primer corte descrito. Autoriza preparar candidato privado compuesto de HEAD con deltas mínimos, no adopción/cierre runtime todavía. Aceptación independiente del código/camino factory->owner->diff->Crossterm real80x24/120x40, controlesstyle/preclear, cuotasoldnew y mediciónWork completos permanece necesaria. Pump/D2c, b2roles/resources completos, búsqueda/history/sourceacciones, QAvisual yUnix/RV301 continúan pendientes explícitos.

No pruebas enormes lanzadas, sin publicación/perfiles/corpusprivado, sin nuevas demandas para completar restoproducto antes revisar este corte.
