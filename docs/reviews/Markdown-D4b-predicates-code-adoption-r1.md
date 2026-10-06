# Adopción de predicados D4b R1

Fecha: 2026-10-06. Alcance: identidad de bloques, cobertura semántica, infraestructura de caché acotada vacía y cancelación local prestada durante copia. No produce estilos ni activa color.

[La aceptación independiente](Markdown-D4b-predicates-acceptance-r1.md) finaliza sin críticas pendientes para este alcance. Se verificaron 25 registros del candidato y 33 de aceptación, sin discrepancias. Cinco fuentes se incorporan exactamente; lib.rs recibe solamente la declaración privada del módulo de caché, conservando los módulos actuales. La comprobación inicial rechazó una expectativa errónea de línea vacía antes de modificar archivos; la inspección del delta confirmó la declaración contigua.

La combinación compartida pasa 16 pruebas document_code y siete de syntax::retained; cargo fmt y Clippy de workspace/all-targets con warnings denegados terminan con código cero. Se utiliza el toolchain privado y Cargo offline/locked. La nota upstream de compatibilidad futura no constituye validación de versiones futuras.

La cobertura conserva la identidad del input y los rangos reales: no infiere estilos de rangos ni renueva la autoridad de una operación caducada. La caché aún no tiene productor, activación de frame ni presentación. Permanecen pendientes el scheduler real, integración de preparación b1, renderer, CLI, QA visual y ejecución Unix. No se cierra D4b ni RV-301.
