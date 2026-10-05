# S03 D4 XML — cierre de arquitectura del núcleo

2026-10-05. Propuesta canónica: [núcleo XML](../sprints/S03-D4-core.md), SHA256 original 168F1D53C754399C72E2A3F5ABA2F56E03F379579CD1EAD3316A96C09EAB53B3. Revisión independiente xml_core_adversary: SIN_CRITICAS_PENDIENTES sobre esa propuesta. Informe temporal r3viewer-xml-core-r1-adversary.md, SHA256 AA0CC73758514D2A0A1E3B3D6A6122AC980BF61237FCFC0A17F1AB552D32F605.

La revisión contrastó BOM/rangos, validación propia del perfil, namespaces normalizados, DOCTYPE inerte, presupuestos/contención, IPC y modelo mixto contra W3C y el spike local. No encontró objeciones materiales pendientes. El detalle conserva siete gates experimentales: este cierre no acepta el parser ni el producto, no acredita runtime ni completa RV-106.

El siguiente trabajo ejecuta los gates previos al núcleo; los experimentos pueden exigir una nueva propuesta. El contrato observable de [XML](../sprints/S03-D4-XML.md) permanece vigente.


Gates experimentales G1–G4 cerrados para el prototipo MANIFEST94869AFD7ACD4AA787AABFD54B2C4F4D10FB506325B75127F30F1CA17FDA685C:198checks autor y47independientes; [observación de recursos](../research/S03_XML_OBSERVACION_RECURSOS.md) y [revisión G4](S03-XML-gate-recursos.md) sin críticas pendientes para criterio existente. Monitor NT con control positivo y siete negativos; no ETW kernel ni garantía universal. Portado A iniciado; G5/G6/G7 y producto siguen pendientes.
