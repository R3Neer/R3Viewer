# S03 D3 YAML — candidato integrado R3

2026-10-05. Revisión adversaria SIN_CRITICAS_PENDIENTES y [aceptación independiente](S03-YAML-aceptacion-r3.md) funcional parcial Windows. No completa S03 ni RV-106.

Candidato congelado R3, base c2dc9b4: 335fuentes y2binarios, MANIFEST472C490C51152D80A6FF76393A46BFD2D4BCE3DC41ABD10AEDCB46E4093EFBF3; app29EBF22FF82B03FDBF039B46FBD086E766EB3F67BB4B0945E796F0E3C5F6B15A, companionE885DE7F826DA9520EA7E6A11139F57F966A9BDB4E9CFEB5653A6BFDB6D5567E. Informe adversario temporal R3-adversarial-review.md SHA1D3C0CAED3A39224990D1CFD52BDC84897240ADE4C0C74E25D2212CFD0D8580A; AE original REPORT-R3.md SHA3A1506825EE2917BA5DCB921460893A63B5D7DEA161129C5A9C4BEFF48331A68.

R1 mostró dos fallos de búsqueda: comparación sobre placeholders y unidades extra del inspector. R2 compara contenido original, separa unidades ScalarDecoded/TagUri/AliasDescriptor, rangos originales/visibles y orden metadata→decoded. R2 aún proyectaba el prefijo por cada coincidencia backward; R3 proyecta únicamente el elegido. Adversario confirma exactamente1proyección sobre200000coincidencias, controles/glifos distintos, wrap, alias sin expansión y metadatos limitados. Sospecha de límite Small128 refutada mediante grapheme201bytes.

Coordinador:188tests workspace,fmt --check,Clippy all-targets -D warnings ybuild PASS. Crosscheck Linux x86_64 workspace PASS; macOS arm64 lector+wire PASS. Crosscheck companion macOS desde Windows falla en build.rs de libproc0.14.11: sus main se condicionan al SO host macOS/Linux, sin Windows. No se parcheó dependencia; requiere comprobación nativa/CI. No afirmar ejecución Unix.

AE:101fixtures evaluados con fuentes/códigos/estado preservados,95rangos originales ejecutados y12sesionesConPTY correctas incluyendo coordenadas,resize,retornos128,alias,flowvacíos y200kbackward. Reutilización R1 de202CLI se justifica por hashes del parser/modelo/protocolo/output; no se atribuye aR3 una reejecución inexistente.

El corpus antiguo tenía15expectativas de categorías contradictorias con el coreR2 aprobado: custom true y14URI explícitas desconocidas exigían Boolean/String. Adjudicaciones independientes y suplementos separados corrigen solo categoría aTaggedScalar; originales inmutables, decoded/URI/raw/estado/spans intactos. Primer suplementoSHA044C3ACC4B1760604D343422F15CFB502D03815375C95AA844238B07545D7CA6; segundoSHAEAAE7DD1F0F7D60D8F016442ED169770BC556F9CC74CD9EBEEE1BD040B19DE22, adjudicación14SHABACC4F7B61990868A9C30965741A4562BE6AA24B3A2E2C804BC359C23B4C6477. No es cambio de contrato para obtener aprobación.

Pendientes explícitos: presupuestos máximos y estrés independiente completo, lifecycle/deadlines/Job/reap/reintento para YAML, visual WT/WezTerm, contención/runtimeUnix, distribución y XML. La revisión no exime estas comprobaciones ni vacía el backlog.

Proveniencia comprobada por coordinador:39archivos originales,2sources derivados,manifest de empaquetado ylicenciaArrayDeque coinciden con sus hashes;45archivos vendorizados/licencia/patch verifican bytes iguales entre índice Git ycheckout. El check de whitespace propio pasa; se preservan líneas de contexto de parser.patch/scanner.patch yla línea final vacía de tests/spec_test.rs.inc originales, en vez de alterar su evidencia byte-exacta.
