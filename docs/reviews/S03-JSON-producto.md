# JSON D2: revisión y aceptación del producto

Candidato R1 base f263d06. Snapshot privado `r3viewer-s03-json-review/R1`,275 archivos; manifest SHA256 2197E023D552E02451D07AE4DC838372E376F623051205FFDCB67B10DE0A4DC5; app106FF1A9FDBCC6322C28AA7AEB81D2466AF4933E65D386F8F79F1E29BAD8C7E6, companionE885DE7F826DA9520EA7E6A11139F57F966A9BDB4E9CFEB5653A6BFDB6D5567E.

## R1: fallo de aceptación

Autor:158tests workspace,fmt,Clippy all-targets estricto,build PASS. Check de compilación para x86_64-unknown-linux-gnu y aarch64-apple-darwin PASS; no ejecución Unix ni certificación containment/terminal.

Adversario independiente:57probesJSON+14regresiónTOML+9modeltests y275hashes PASS; cierre SIN_CRITICAS_PENDIENTES limitado al alcance. Informe temporal REVIEW-R1.md SHA256 ACC0F19E5C4936EA924C55FD6051FA466EE59C4DAC2A5CB117353D65A21D9738. Ese cierre no sustituye aceptación independiente.

Aceptación independiente R1:187CLI/Nu,49checkssemánticos,69CLI+26semánticosTOML pasan. ConPTY TUI15checks:14PASS/1FAIL. Buscar glifo literal␛ en strings-controls.json selecciona$[0] escaped (ESC cooked) en vez de$[1] glyph. Reproducción `/␛`+Enter UTF8bytes2F E2 90 9B 0D. Contrato D1D09/J06 exige distinguir contenido original de placeholder. Candidato R1 rechazado por este defecto aunque revisión adversaria no lo detectó.

Número16MiB exacto0 observado2124ms; arrays combinado999999Ready/1000001fallback; spans Unicode/rawkey/cookedvalue/container, stdinUTF8partido PASS. CancelBuilding31.61ms/manualretry, C130restore3.29ms/Nu48.21ms y q0/reap/modes PASS. Mediciones puntuales, no SLA. Informes/corpus privados conservados en acceptance-json-r1; errores iniciales de oráculo se preservan y explican, sin modificar criterios para aceptar el fallo real.

## R2: corrección y repetición

Buscar Estructura compara texto decodificado original del inspector; las coordenadas de resaltado siguen siendo del texto seguro. Previous hit se traduce display→decoded y nuevo hit decoded→display con provenance. Fuente y datos Nu mantienen contratos vigentes. Test de ESC/C1 frente glifo literal/texto de escape, wrapping adelante/atrás y tests existentes de dominios actualizados para consulta ESC real. No se buscan placeholders como si fueran contenido.

Autor159workspace+fmt+Clippy/build PASS. Snapshot `r3viewer-s03-json-review/R2`,275archivos, manifestDFA9A7A4D1B9465A648A8DA4D9325494C1006EE6099498D173FE7DFBE187DDCE; appAEDFB540522209678889193E79981135B9D6C2FEAF72C7C8288927F3DD3E64CA, companion sin cambios. Revisión adversariaR2: SIN_CRITICAS_PENDIENTES,275hashes y3regresiones propias de controles/glifos/etiquetas, Unicode/checkpoints4KiB/chunk64KiB y precancelado PASS. Informe REVIEW-R2.md SHAB5006085EAA2084419CC7029E21511BEE7B6FC70BB2D0718B21DD21A702791E3.

AEindependienteR2 delta funcional PASS: reproducción original ahora$[1]glyph;10familiasJSON/TOML con56pasos n/N/wrap/ESC/C1/offsets/Unicode/hoja5000,18estados de consultas Fuente/Estructura/salto/retorno y rawkey8cols/value22cols pasan. R1 permanece intacto;187CLI/49semánticos/regresiónTOML69+26/lifecycle pertenecen aR1 y se reutilizan con hashes/delta explícito, no nuevas medicionesR2. Reporte temporal acceptance-json-r2/REPORT.md. No se confirmó nuevo fallo semántico; algunas filas ConPTY CJK/combining presentan restos de causa indeterminada (decoder o presentación), retenidas paraQAvisual. No disfrazar esa incertidumbre comoPASSgráfico.

Root contrastó275archivos previo a integrar: código/binarios coinciden conR2; diferencias únicamente README y contratoJSON, documentación posterior de estado/uso. Commit conserva código exacto revisado salvo normalización Git habitual de líneas de fuente; recursosvendorizados -text conservan bytes de procedencia.

Retrospectiva: la revisión adversaria no sustituyó aceptación. El fallo glifo pasó tests que esperaban placeholders; se reemplaza esa expectativa por semántica contractual y regresión independiente del valor. Candidatos fallidos, oráculos corregidos y captures tempranas permanecen trazables.

## Límites que no se cierran

QA visual, ejecución Unix/containment, Repeat/AltGr físico, decoderVT inline largo y presupuestos/deadlines exhaustivos siguen pendientes. D2 no termina RV-106: YAML/XML y aceptación restante permanecen. No se completa S03 ni producto mediante este informe.
