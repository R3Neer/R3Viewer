# Parada acordada tras S03

2026-10-10. El usuario pide «Para cuando completes este sprint» y confirma «Parar tras cerrar todo S03» al distinguirlo del incremento Markdown interactivo. La parada es condicional al cierre completo de S03; no es una pausa inmediata ni autorización para declarar terminado un corte parcial.

## Límite y procedimiento

El objetivo de producto completo permanece intacto. Prioridad actual: completar y contrastar todos los criterios del [sprint S03](../sprints/S03.md), incluidas validaciones pendientes de inspección, búsqueda, presentación, Unicode, límites, ciclo de vida, QA visual y matriz de plataformas en su alcance. No sustituir ejecución por compilación o ConPTY por apariencia. El auditor independiente produce matriz requisito→evidencia→pendiente antes de decidir cierre.

PDF, Markdown enriquecido y MathJax no se convierten en condiciones nuevas de cierre S03 por estar en preparación. Los agentes de esos incrementos deben dejar un checkpoint coherente de sus operaciones ya iniciadas, recolectar únicamente sus procesos propios y conservar fuentes, fallos y pendientes. No comenzar nuevas historias posteriores durante esta concentración en S03.

Al verificar cierre real de S03: conservar evidencia final, actualizar documentación/backlog y crear commit local coherente; detener agentes y procesos de desarrollo de forma segura, pausar el objetivo por petición explícita del usuario y comunicar el punto de reanudación. No iniciar sprint siguiente, publicar ni modificar perfiles. Si un criterio requiere un entorno no disponible, mantenerlo abierto y exponer la dependencia concreta: no fingir cierre para ejecutar la parada.

## Situación al fijar la parada

S03 conserva aceptación funcional Windows parcial e incrementos integrados. QA visual y ejecución/contención Unix figuran pendientes en sus registros canónicos. Actualmente sólo se dispone de Windows, según respuesta del usuario; CI futura no está configurada ni autorizada por esta petición como publicación. El siguiente paso es auditar evidencia y completar trabajo ejecutable aquí antes de identificar cualquier dependencia externa imprescindible.

## Resolución de preparación externa (2026-10-10)

Ante la falta de hosts Unix y QA visual permitida, el usuario elige «Preparar una guía de QA manual y una matriz de CI local para configurar después». Se preparan y revisan ambos artefactos; esta decisión difiere su configuración/ejecución, sin sustituir pruebas por documentación ni declarar cerrado S03. Se completa el trabajo funcional ejecutable en Windows mientras V/U conservan dependencia explícita. No avanzar al siguiente sprint para ocultar esa dependencia.

## Dependencia restante al terminar el trabajo local

La [matriz final independiente](../reviews/S03-cierre-windows-final.md) conserva19filas y no exige más trabajo funcional Windows concreto. W2 y composición C01/W3 aceptadas; documentación y backlog reconciliados, fallos históricos y causas UNKNOWN preservados. La continuación de S03 depende de ejecutar la QA visual y U-G1–5 que el usuario difirió. Se entrega el punto de reanudación y no se empieza otro sprint. Esto registra una dependencia externa, no declara cumplida la condición de parada tras el cierre completo de S03.
