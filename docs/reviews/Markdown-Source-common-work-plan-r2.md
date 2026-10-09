# Source Common: cierre de diseño del presupuesto de trabajo

2026-10-10. Se conserva la revisión independiente final de la [calibración Common](../refinement/MARKDOWN-SOURCE-COMMON-WORK.md). El contrato final revisado cambia únicamente la preparación SourceViewport Common a 512 KiB de trabajo; Source Legacy permanece en 256 KiB. Conserva Frame256KiB, Symbol16KiB, Consumer1MiB, Actor64MiB y DocumentWork8MiB.

Este complemento sustituye explícitamente el límite Common de 256 KiB del [contrato D2c](../refinement/MARKDOWN-SESSION-PUMP.md), conservado como snapshot histórico. El coste preparado debe incorporarse una vez a la operación completa del futuro pump. No hay todavía aceptación de código R2 ni cierre de SC-W01, Source nativo, inspector, pump o QA.

Propuesta exacta SHA256 BFF4E78A394119A4BFE516049C1B8DEB9DB0224800473D52767F4E4DC896BC7B. Informe independiente original SHA256 3CABCC89A2C91E55A48F45638C150ED0A15FBD7BA9E2A47AB5665D2834499BFB. Se reproduce el texto final con saltos LF; originales, diagnóstico y fracaso previo permanecen en evidencia privada. Errata ya señalada por el adversario: el subtotal histórico corresponde a las primeras cuatro filas de la tabla, no cinco; la suma numérica es correcta.

---

# Revisión focal de diseño SC-W01 / calibración Source Common R2

2026-10-10. **SIN_CRITICAS_PENDIENTES de diseño/calibración**, limitado a propuesta BFF4E78A394119A4BFE516049C1B8DEB9DB0224800473D52767F4E4DC896BC7B. Permite implementar perfil cerrado SourceCommonWork512KiB y delta aditivo metered. **SC-W01 no está cerrado runtime; R1 sigue NO_APROBADO**. No se ha leído código mutable ni ejecutado candidato R2.

## Procedencia y criterio independiente

Propuesta exacta: .tmp/markdown-source-common-2026-10-09/PROPOSAL-SC-W01-CommonWork512K-R2.md, SHA BFF4E78A394119A4BFE516049C1B8DEB9DB0224800473D52767F4E4DC896BC7B.
Suplemento propio guardado ANTES de leer propuesta y antes runtime R2: EXPECTED-own-source-metering-calibration-before-R2-r1.json, SHA0E5D897DDCD7FD702D29F0DC0798143A0DC8E0EDDF128027750DF10AF0975A50. Estado NOT_RUN;9 comprobaciones/fixtures independientes. Declara temporalmente lectura del diagnóstico que motivó calibración, no esperado numérico copiado como medición propia.
CRITERIOS y EXPECTED R1, EXPECTED SC-W01 previo y REVIEW-AE R1 verificados intactos:569EB159… /5B5CF4B7… /5ACBDBB5… /818806D7….
Única evidencia nueva permitida leída fue diagnóstico y su esperado previo: productie256KFAIL preservado; cfg(test) misma factory/model/authority/prepare bajo512K,300398Work/123712Frame. No produce aceptación.

## Evaluación

La suma explícita es241111histórico +59287lookup/metadata real =300398. Componentes presentan header, campos/probes/comparación, recordselección, Point clone144 real, fallbackconstructor144 y provenance72; no pagar un Point entero por lectura prestada. No factura nominal que omita probes ni escaneo de fullcapacity para inflar recibo.
Margen propuesto512KiB−300398=223890; la política permanece finita y no asegura latenciaUI/SLA/RSS/CPUexacta. Mantener256K causa rechazo denso120×40 al meter lookup; omitir/reducir cargos contradice SC-W01. Optimización nueva cache de índices sería otro delta con lifetime/retainedcargos; no es necesaria para adoptar una calibración técnica explícita.
Se conserva Work::frame y FrameBudget::new Legacy256K; Common constructor privado512K no acepta cuota variablecaller ni flag que suprima cargos. Frame256K, Symbol16K, Consumer1M, Actor64M/Transport3M y modeloSource límites originales permanecen. No segundo permiso/reset/fallbackrefill.
La exportación work() deberá añadirse UNA VEZ al total efectivo futura operación Document8M; este corte sólo prepara SourceViewport y no acredita NativeSource/chrome/publicación. No llamar resultado512K permiso nativo nuevo. Los contratos afectados deberán registrar la calibración explícita al adoptar para no dejar la afirmación Source256K universal contradictoria.
APIs metered aditivas pueden cambiar SHA completo text_engine/presentation manteniendo bloques y comportamiento Legacy. Archivoentero byteexact ya no puede afirmarse enR2; inventario debe declarar cambios reales. Root autorizó este delta mínimo, no D1global.
OriginalBox/context/Arc/receipts/scope/keys/metadata/kernel deR1 deben mantenerse enR2; no relajar esos criterios por Worknuevo.

Errata menor no bloqueante: párrafo dice primeras cincofilas histórico241111; son primeras CUATRO, y la quinta es rowheader1040. Tabla y suma correctas. Autor informado para nota de handoff, sin reescribir propuesta congelada.

## Disposición antes de implementación y AE

Ningún nuevo límite de memoria; perfilWork Common explícito permitido. Tabla autor dense80=129078 ydense120=300398 pertenece SUS fixtures, no esperado de misfixtures80×24/120×40 sinLFfinal. AE mide recibo real y contenido exacto con oráculospropios; no cambiar outputesperado para PASS.
Al freeze verificar probes/fields/clones/constructor/admitcancel; aperturas endpoint/provenance/indexposition/cellposition; exact/−1 porAPI y preparador real; lateC1despuéscheckpoint; Symbol16Kexact/+1 y SourceFrame; composición metadata/lastDrop/SAMEauthority y preservarR1logs. La corrección puede requerir ajustar sólo APIs del harness guardando borrador, no semántica.
No repeat de suites verdes sin cambioafectado. No cierre Pump/LoadingHashcomún/SharedDocumentDescriptor dispatcher inspector/QA/RV.
