# Unicode integrado R2 — revisión independiente del delta UIR-01

2026-10-06 Europe/Madrid. Autor graphics_gate_experiment; revisor unix_r3_review. **SIN_CRITICAS_PENDIENTES para el delta funcional UIR-01 y la combinación afectada examinada.** UIR-01 cerrado en este snapshot. Riesgo operacional/rendimiento pendiente, separado y explícito abajo; no aprobación general de producto ni cierre S03/QA/RV.

## Identidad y preservación

`unicode-integrated-r2/MANIFEST.json` SHA256 `56EA09960104D701CB906E752CD403A7BA400FA80481EA8B75EFEBDBD8886F95`,528registros; todos hashes/tamaños correctos antes de copiar y después de los controles, cero discrepancias. Delivery `S03-unicode-integrated-delivery-r2.md` SHA `6EACE3042A68F06972ACEF538F56D4DABAA5E99CCE7206975A8368985E5DC532`; rootbin `3B84A4BCD21F2AE542C4E7137E5826598C66CFD31733F0101ED3CDE50DA43AC0` intacto.

Copia propia `unicode-review-r2/repo`; target propio nuevo `unicode-review-target-r2`. Offline/locked, sin modificar workspace compartido, freeze, Cargo/lock, perfiles ni producción. Sólo se añadieron tests en la copia. Los siete archivos de evidencia original R1 siguen iguales al receipt histórico; los ocho tests de Pane se copiaron byte-exactos y los dos de controller conservan su texto exacto. R1 permanece FAIL y su informe no se reescribe. Los catorce nuevos fixtures se instanciaron directamente desde PREOUTPUT-ORACLES.json SHA `981040DDDDA3433BD680F68EABE4FCF050A5AA67F592DA6E5B3B8AA71592E53D`, fijado antes del freeze/salidas R2. Fuente, valor, parámetros y expectativas no se adaptaron al output.

## Código y resolución

Comparación completa repoR1→repoR2: sólo yaml_view.rs y xml_view.rs cambian código; otras diferencias son documentación. En ambos, callback de preflight y materialización corta al encontrar newline de la fila actual, además de columna>=right/line>fila. El Point resuelto tras consumir newline queda antes del primer grafema siguiente. Se evita el lookahead destructivo de R1, con corte simétrico para que replay/materialización cuenten lo mismo. Si termina antes por recorte horizontal, el siguiente walk puede atravesar el resto de la fila anterior hasta llegar a su fila; ya no pierde su primera unidad. No se reinicia cursor distante, no nuevos snapshots, identidad ni cuotas modificadas. Index/Work/provenance/width/inline/session y resto de mecánica son byte-idénticos a R1; no se recrea revisión universal ni batería completa.

Los diez casos originales pasan sin modificar expectativas: filas cortas ab/cd/ef, vacías, compuestas y controles80columnas en cada formato, más find_yaml/find_xml→apply_search→render TestBackend80×8. Las dos sesiones ahora muestran exactamente ab/cd/ef.

Los catorce adicionales previos pasan: en ambos formatos, primer cluster de fila siguiente16385B/width1 o16386B/width0 exige Error::Reference con rango/visible exactos; cinco combining ordinarios width0 no publican símbolo pero no omiten z/q; top1,left1 con borde lejano o width3; recorte de wide cluster preserva espacio de celda parcial y ASCII siguientes; piezas reales Metadata YAML/Inspector XML mantienen labels/valor y Text::new.count>1. Todos comprueban que used del lease vuelve al baseline tras soltar Windows/error. Esto cubre la combinación afectada por continuidad/corte/preadmisión, no todos los posibles documentos.

## Ejecución y evidencia

Receta: `cargo test --offline --locked --lib independent_ -- --nocapture --test-threads=1` en copia/target propios. Exit0. **24/24 controles propios PASS**:10históricos+14previos nuevos. El filtro de nombre también incluyó3tests existentes del autor; log total27PASS/0FAIL, no se cuentan como AE propio. Cargo reporta57.91s compilación y0.08s ejecución de esta matriz pequeña; no se extrapola SLA.

`unicode-review-r2/INDEPENDENT-24-attempt1.log` SHA `33BD9CC344D7EC6F84EA38A27F53CC723AACA2839DCFB648BB23DBEACE2007ED`. Receipt `INDEPENDENT-RECEIPT.json` SHA `8135E2B255E8E5D3614AAE482237F4127B74C3E5916C0F31BE0ACEE1E6E6D320`, identidades y hashes de precheck/tests/parches/log. Parches propios comparados contra freezeR2 guardados. Primera tentativa de montaje falló por marcador CRLF/LF no encontrado antes de escribir; ajuste exclusivamente de extracción de módulos, sin expectativa/código candidato cambiado. No procesos ni sesiones de checks propios vivos.

## Riesgos y límites que continúan

Evidencia del autor R2 preserva regresión serial20PASS con3695.62s de tests y8.09s compilación; duración externa aproximada3705s coherente con timestamps, sin marcadores individuales. Previo intento paralelo17PASS/1FAIL WorkerExited al proyectar XML32MB antes de Pane, causa desconocida. **Ambos hechos siguen pendientes de investigación focalizada; ni este PASS pequeño ni rerun serial los explican o cierran rendimiento/SLA/estabilidad.** No atribución a concurrencia/setup/Unicode particular sin evidencia. Este dictamen funcional de UIR-01 no autoriza usar esos20PASS como aceptación operacional global.

Mantener YAML32,000,001B ArenaBudget previo a Pane/fallback Fuente completo y contraste16,000,001B separado; no borrowedPASS inventado. No cambio de presupuestos source128MiB/visible256MiB/frame256KiB/symbol16KiB ni borrowed1MiB/window128KiB. Root AE independiente y demás gates permanecen separados. Sin nativeUnix, ConPTY/restore/Ctrl-C, QA visual, distribución, aceptación universal o cierre RV/S03. No commit/adopción realizados por el revisor.
