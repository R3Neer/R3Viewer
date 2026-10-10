# S03: conservar el timeout de validación XML

2026-10-10. Corrección integrada de clasificación de errores, con [revisión y aceptación focal independientes](../reviews/S03-XML-timeout-final.md). S03 y RV-106 siguen abiertos.

## Problema y cambio

La construcción XML vuelve a validar su fuente antes de finalizar el modelo. Si esa validación agotaba su plazo, transformaba Timeout en ModelError::Cancelled; la proyección lo presentaba como StructureFailure::WorkerExited. Un fallo de ordenación también perdía su categoría. En una llamada local sin proceso hijo, ese resultado no acreditaba muerte de ningún worker.

ModelError conserva ahora Timeout separado de Cancelled. Parse y ordenación comparten una conversión que preserva cancelación, timeout, memoria y profundidad. Los mappers de estructuras y JSON/TOML/YAML conservan la nueva variante. El protocolo, las cuotas y el plazo original de cinco segundos no cambian.

Las pruebas usan XML válido pequeño y un deadline ya vencido para comprobar el mecanismo sin sleeps. La ordenación se ejecuta realmente con tres relaciones extraídas del modelo; se verifican éxito, timeout y prioridad de cancelación. No hay flags ni configuración nuevos para manipular el plazo productivo.

## Validación e identidad

Candidato basado en308ab023608bf34e8737362a1175aed43d756928, seis fuentes modificadas. Manifest privado previo a outputs3AB7AED889B26D7D9C349D33AA827ADD312E2E8089E81D0D16AA475F093384B0;493 entradas exactas. El primer compile detectó una comparación NodeId sin PartialOrd en un test; se corrigió la aserción usando sus enteros, sin cambiar el tipo productivo. Compilación posterior y nueve pruebas focales PASS; el agente independiente ejecutó esas nueve y comprobó identidad antes/después. Revisión y AE: SIN_CRITICAS_PENDIENTES exclusivamente en este alcance.

Integración: build workspace, cargo fmt --all -- --check y Clippy workspace/all-targets con -D warnings exit0. Harness, worker, companion y configuración vacía congelados antes de ejecutar regresión serial de biblioteca:509PASS/0FAIL/10ignored, exit0,102.61s. Los diez omitidos no se cuentan como aprobados. Manifest integrado8088AC7E3A6A621F83C0FAB9A288605AE34171AED6A425317293FF97B95D0D8F; log48E5F9F2D8F6A868FF1A7FAD9E7AFFFDE121892B826682F455D32E087531BDE5. Sus diez identidades permanecieron exactas al terminar; los seis hashes canónicos coinciden con el candidato revisado. Artefactos privados en .tmp/s03-xml-timeout-2026-10-10.

## Pendientes que conserva

La cadena estática demuestra el defecto de clasificación y las pruebas verifican su reparación. No determina retrospectivamente la causa del antiguo fallo paralelo ni de los3695.62s. Rendimiento causal, fronteras materiales, cancelación durante preparación, QA visual y runtime Unix conservan su aceptación original en la matriz de cierre S03.
