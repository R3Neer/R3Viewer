# Revisión adversaria b2 R2 — diseño

2026-10-09. Disposición: SIN_CRÍTICAS_PENDIENTES en diseño b2 R2. Permite iniciar implementación privada conforme la propuesta exacta. No acredita runtime, candidato de código, composición b3, pump ni recursos gráficos.

Base código353418c19f2152614d5650d605aa05df72a1cbca; documentos posteriores7a7b9a7 conservan ese código. Propuesta B2-PRESENTATION-PROPOSAL-R2.md SHA256 783C801B3608765B775E48A033C15832B57875E4EE88C1F0FFA24A667C144340; EXPECTED-b2-before-output-r2.json SHA256 7051A5F0B1D3FF11A5E5371C8F7DD25522D8C23888BD5614B15557752A62A416. Ambos hashes verificados directamente. Fixtures table-resize.md SHA9A6B0E5E828A0A8D40A58CCFE074BF1871427A20AE2F58D920BA80B5935289C3 y role-ref-resume.md SHAEC105F5290EBD89ED1ACA20DD95196C3BE6F62F35DE2BC7E1FBE139F7CB409CD verificadas.

R1 y su informe A8B20E8B64A2EF34F90AA9CEEE711B6BA045116CD3175703169BA17DAB91E096 quedan preservados. Criterios propios FD014C77316F2BCB6067132AD47D0FA193419DE1F56DFF131AC9557270AC98DD y oráculos propios de anclajes EB3E3FCE0C8E9FADFD1A9E4A998A114C2CF60DB47881C87254E3DF8B7D76B9BA anteriores a implementación/runtime siguen vigentes; no se usan outputs del autor como goldens.

## Cierre B2-A01

R2§4 reemplaza Table(row,cell,subfila) por TableResume: seleccionado y posiciones Domain/LayoutAnchor acreditadas, progreso At/Done por cada celda local. La subfila queda metadata geométrica y no identidad de texto. Resize vuelve a posiciones lógicas y no transforma byte8 en byte20 por cambiar ancho8→20; los Done no reaparecen. El progreso usa ChargedVec local, único slot candidato y SAMEWork; no índice global ni Points por celda. Validación Row/Cell/domain/Flow precede Geometry; mínimos insuficientes producen WholeReference explícita.

El testigo nuevo fija dos posiciones byte8 distintas, otra celda Done, cambio30→66 y continuación a row2/TAIL. Los criterios propios añaden cambio columns↔vertical y ventanas cortas. El algoritmo final y sus cargos se comprobarán sobre freeze, pero no queda crítica de diseño por identidad geométrica.

## Cierre B2-A02

R2§4 incorpora RefAnchor con RefOwner::BlockRole(block,role,field) o Resource(resource,field), ID real y posición local/projectionpart acreditada. Constructor comprueba aux/campo del modelo; no fabrica Resource/Domain/Run. KIND/INFO/LANGUAGE/LABEL tienen lanes y posiciones propias; slot Domain/TextRef respeta dos slots. cell_anchor no infiere Source de bytes de símbolos; RefText no fabrica FirstCodeComponent. Continue/Line/Resize conservan lane y posición hasta sufijo y TAIL.

El testigo nuevo contiene calloutKind, fenceInfo y footnoteLabel largos, con negativos de RefID ajeno, UTF8 interior, projectionpart, stale y cancel. El oráculo propio de footnoteLabel52 mantiene procedencia separada de decoración. No queda crítica de diseño por referencias no-resource.

## Admisión restante

No se encontró incoherencia funcional nueva en el delta focal. Se conservan orden Items, cobertura completa de familias/recursos, separación PendingRequired/WholeReference, width0, provider/provenance, old+new y rollback del owner ya aceptado. En tabulación columnas↔vertical se deberá comprobar en código la inicialización/reconstrucción del progreso local desde IDs/posiciones reales, sin convertir geometría en fuente ni perder celdas posteriores.

Envelope n*1256+524544, saldo layout, transferencia opaca noClone y distinción held/spent continúan sin objeción adicional de diseño. Suficiencia real, metadata sidecar≤32B, capacidades físicas, crecimiento fallible y Frame/Consumer/cancel typed requieren comprobación del freeze exacto. Rechazo guard>held preserva old antesIO y nunca reduce cargos/rellena saldo para pasar.

No pruebas ni compilación ejecutadas en esta revisión. Siguiente fase: implementación privada, oráculos previos preservados, tests focales del autor y freeze; después revisión estática y AE propia de código. B3/pump/D4/gráficos/Unix/QA visual permanecen requisitos posteriores del producto, sin exigirse como condición de este diseño b2.
