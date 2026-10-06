# Markdown D2b1 — aceptación independiente R2

Dictamen: **NO APROBADO**. MD-B1-ANCHOR01 queda corregido dentro de este candidato; permanece el bloqueo nuevo **MD-B1-C1-PROGRESS01**. No se atribuye aceptación del renderer, CLI, pump, ConPTY, b2/b3, Unix ni producto global.

## Identidad y método

Candidato markdown-d2b1-candidate-r2, MANIFEST 0CCDC741DB79D04D250B9FF06C326D1A755575CB57D12EC4977E6218BEFA58A9. Readback propio anterior y posterior: 639 registros sin discrepancias, 583 archivos repo exactos salvo sufijo cfg(test) de 54 B en document_view.rs. Ninguna función candidata modificada. El harness original R1 es prefijo byteexacto del nuevo; R1/rechazo/FAIL no se alteran. Seis deltas respecto R1: cinco módulos productivos y tests del autor. Leídos completos los deltas de flow/navigation/text/layout/view y contrapuestos a los sitios efectivos del camino request→prepare→commit; tests autor identificados como evidencia distinta.

Propios antes de outputs: originales preoutput/ANCHOR-original-before-output.json; R2-ADDENDUM-before-output.json; ALT-C1-before-output.json SHA 0223B45D61F462BD45B6625C54CB4BF176FC139562E56D921240137E985FFD7B. ALT supplement quedó fijado antes de su primera ejecución; C1 fijado antes de nuevo build/runtime. No goldens derivados de outputs. Bin propio 1 BCE3CF84EFC9E888B5F404CCC31EC1B8F06A0D958814D581E37A6B88F0E6EA34; nuevo bin con test C1 A4124C79ADAF4076A76CECA1E267A5943A7ED98882B71F781D0B9FF0F8818F57. BUILD1 terminal0 (2m00s dependencias), BUILD2 terminal0 (8.10s incremental). Perfil privado Windows offline: build escalado, runtime propio ejecutado en sandbox; Python aviso localización preservado. No procesos auxiliares de sistema, ningún rescate o sesiones activas al cierre.

## Resultados funcionales

- Original MD-B1-ANCHOR01, fuente abc, Run0 byte4 inválido, width40/0: 1 test PASS 0.01s, Err(Stale) y current previo intacto. Test original byteexacto.
- TEST-attempt2-r2-negatives: tres grupos PASS 0.01s. Dieciocho escenarios negativos: UTF8 continuación de é; Block/Run ajenos; Role con byte1; HardBreak byte1; Resource byte1; ALT Resource Domain cuyo owner coincide numéricamente con BlockID. Cada caso en geometría normal, width0 y cuota ventana 1000x1000; rechazo conserva current. Controles válidos raw0/2 de é, Role0 y Reference de geometría/cuota; constructor State valida límites y UTF8. HardBreak A seguido B normaliza siguiente cursor a Text B byte0 y Line±1 mantiene primera scalar/historial0.
- Cinco regresiones originales focales PASS: crossrun_geometry_controls; reference_and_end (18001 B original ya existente, no nuevo corpus gigante); fold_resize_stale_preserve; line_prepared_rows; window_exact_admission. Cada log exacto separado y REGRESSION-RESULTS.json. Se repitieron por impacto directo en ancla/canonicalización/rechazo Reference, no kernels/source/ownership sin cambios.
- TEST-attempt3-C1: 1 FAIL, exit101, 0.01s. Ningún ajuste del oracle posterior al fallo.

Nueve grupos propios PASS y un grupo FAIL no equivalen a todos los veinte criterios previos; evidencia R1 no afectada permanece separada. No repetir ni atribuir tests del autor como aceptación propia.

## MD-B1-C1-PROGRESS01 — Line queda en bucle dentro de control proyectado

Fuente exacta `A` + U+0085 + `Z` (UTF8 41 C2 85 5A), literal, Extended. Geometry width4/height1/left0. Preparación inicial muestra `A\u{` y next RawAnchor(block1,run0,byte1). Tras Line+1 muestra `\u{0`, current y next son RawAnchor(block1,run0,byte1). Ocho avances repiten ese mismo contenido/ancla; Z nunca aparece. Las peticiones completan y hacen commit, por tanto no es timeout de worker ni bloqueo de proceso. Log preserva cada transición y el assert final. El criterio era progreso finito a texto siguiente sin convertir offset proyectado en byte de fuente; ocho pasos bastan holgadamente para spelling de ocho caracteres más A/Z con ancho cuatro.

Atribución concreta (archivos congelados): document_text.rs:160 retiene Position.part; character:383–409 usa y avanza esa parte para expandir el scalar. raw_anchor_with_work:920–950 devuelve sólo raw.run/raw.byte, perdiendo part. document_layout.rs:397–401 guarda next mediante ese raw sin continuación visible. document_view.rs:268–277 convierte Line positivo desde next en Seek(raw); el nuevo prepare vuelve a locate_raw desde principio del mismo scalar. El descriptor completo de la continuación no está en el RawAnchor ni en State. La validación nueva de rango de raw es correcta pero no puede recuperar esa información descartada.

El contrato exige Source/raw fiel y controlproyección seguro/navegable; el README incluye Line desde anchor/fila de ventana actual. Esto es un fallo de progreso del comportamiento implementado b1, no una futura demanda de renderer responsive. End/Home u otra operación no hacen que Line+1 cumpla su contrato. No exigir rawbyte proyectado ni reinterpretar C1 como salto de línea. Una corrección debe conservar identidad/fuente/run/byte y una continuación o afinidad validada de la vista con cargos/cuotas/Work/cancel existentes. No prescribe nueva API pública ni debilitar proof.

## Revisión estática del delta y límites

validate_raw_anchor se ejecuta antes de WindowBuilder/profile, incluso geometry/cuota Reference; carga Block/Run/Domain/TextRef en mismo DocumentWork. Role NONE sólo byte0; synthetic Break/Resource sólo byte0; textos exigen len y char_boundary. Reciprocidad/Flow membership cerrada por remap y constructor locate_run (el negativo ALT confirma camino real, sin aceptar owner Resource como Block). State::prepared_run valida antes de crear cursor. Canonicalización lazyRun tras Break sólo en frontera part0 y previo agotado, sin consumir siguiente scalar; regresión original demuestra B0.

No hay cambios en preparación owner/kernel/source/protocolo/pools/parser ni cierre físico. SharedFlow/Consumer/slots, rechazo stale/fullquota/current, y scopes R1 válidos permanecen evidencia anterior donde no fueron reejecutados. Este dictamen no cierra b1 mientras C1 siga fallando, ni gates b2/b3/renderer/pump/keyboard/CLI/Unix. No atribuir SLA ni RSS a tiempos/cargos propios. Candidato permanece intacto antes/después; source de funciones sin instrumentación productiva.
