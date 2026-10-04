# Metodología de R3Viewer

Fecha: 2026-10-04. Decisión del usuario: Scrum por objetivos, sin cadencia temporal fija. Adaptación inspirada en Scrum, no Scrum íntegro. Se autoriza desarrollar todo el producto y continuar hasta completar el backlog con evidencia; no publicar ni instalar en perfiles del usuario.

## Objetivos y autoridad

Un Product Goal, un Product Backlog y un sprint activo para el producto. Cada sprint termina al cumplir un objetivo funcional concreto y su Definition of Done, sin plazo fijo. Si el objetivo deja de ser viable, el coordinador registra el impedimento y propone resolución; no recorta requisitos ni cierra tareas para agotar una lista. El usuario conserva prioridades y decisiones de producto. El coordinador tiene autoridad operativa para decisiones técnicas compatibles con el alcance acordado y commits locales.

Las épicas agrupan resultados, no equipos permanentes ni sprints obligatorios. Especificaciones se refinan por épica e incremento cuando se aproxima el trabajo. Los cambios de comportamiento y cobertura se contrastan con las decisiones del usuario.

## Responsabilidades

- Coordinador: estado común, asignación, dependencias, integración, trazabilidad y comunicación con el usuario.
- Especificaciones/aceptación: detallar comportamientos, ejemplos, errores y criterios; proponer cortes de sprint y comprobar el producto ejecutándolo. No ser el único juez de sus propias especificaciones.
- Arquitecto transversal: contratos y fronteras compartidas, riesgos y decisiones de impacto; participar según necesidad, sin rehacer arquitectura por cambios locales.
- Implementador: código, pruebas pertinentes, documentación y correcciones; no aprobar su propia entrega de forma exclusiva.
- Adversario independiente: contrastar requisitos originales, diseño, código, pruebas y evidencias. No corregir producción durante la revisión. Sus hallazgos son refutables con evidencia.

Estos son encargos técnicos para agentes, no una asignación ficticia de responsabilidades humanas oficiales de Scrum. El coordinador facilita planificación, inspección de progreso, revisión y retrospectiva; el usuario conserva autoridad sobre producto. La aceptación técnica no exige esperar permiso por cada decisión rutinaria.

## Ciclo por objetivo

1. Refinar contrato observable: objetivo, alcance, exclusiones, criterios, riesgos y comprobaciones.
2. Proponer arquitectura específica solo donde el incremento la necesita.
3. Revisar propuesta con adversario, responder y repetir hasta no tener hallazgos accionables pendientes.
4. Implementar y verificar por el autor.
5. Especificaciones prueba aceptación; adversario revisa candidato exacto y realiza exploración independiente según riesgo.
6. Corregir/refutar y repetir revisión hasta cierre explícito. Integrar, comprobar combinación final y mostrar evidencia.
7. Actualizar estados y decisiones, commit coherente, breve retrospectiva y siguiente objetivo.

La inspección de progreso ocurre al completar trabajo relevante o surgir impedimentos; no requiere reuniones simuladas diarias entre agentes. La revisión presenta el incremento y sus límites; la retrospectiva adapta el proceso. El refinamiento del siguiente objetivo puede avanzar durante el actual.

## Revisión iterativa

Cada revisión identifica base Git, commit o snapshot exacto, alcance y criterios. Cada hallazgo tiene ID, escenario, evidencia, consecuencia y criterio de resolución; distinguir fallo, riesgo y preferencia. El autor puede resolver o refutar; el adversario debe retirar falsos positivos comprobados. El veredicto SIN_CRITICAS_PENDIENTES se limita al alcance/candidato revisado y no acredita QA ausente.

No cambiar aceptación para obtener aprobación. Una integración posterior invalida la aprobación de las partes afectadas y exige comprobaciones pertinentes del conjunto. Tras tres rondas sin progreso, escalar discrepancia concreta al usuario con alternativas; no aprobar automáticamente y continuar trabajo independiente. Propuestas e informes intermedios temporales fuera de Git; conservar versión final y resumen de cierre/evidencia.

## Definition of Done

Criterios del incremento comprobados, código integrado y revisado, pruebas pertinentes aprobadas, documentación y ayuda coherentes, límites visibles, ausencia de regresiones conocidas dentro del alcance y veredicto adversario cerrado. Cambios visuales requieren QA visual adecuado, no solo tests. Compatibilidad solo se anuncia para plataformas/versiones con evidencia. Capacidades no verificadas permanecen pendientes; no vaciar backlog eliminando tareas o convirtiendo aceptación en documentación.

Una historia parcial puede terminar dentro de un sprint, pero su RV principal solo termina al completar toda su aceptación. Defectos nuevos se registran y priorizan, sin ocultarlos para lograr cierre. Investigación o CI configurada no equivalen a ejecución multiplataforma.

## Paralelismo y contexto

Un incremento en integración; inicialmente un implementador y como máximo dos cuando contratos y archivos permiten independencia. Worktrees no evitan incompatibilidades semánticas: interfaces, dependencias, lockfiles y empaquetado tienen coordinación explícita. No crear un equipo completo permanente por épica.

Se puede refinar el siguiente objetivo y revisar un candidato actual congelado en paralelo. Si el anterior no cumple aceptación sigue pendiente. Si una revisión cambia un contrato, actualizar trabajo posterior antes de construir sobre él. Dar a cada agente requisitos relevantes, contratos, versión exacta y evidencias, evitando copiar todo el chat. El adversario conoce criterios comunes sin heredar la justificación persuasiva del autor.

## Modelos y coste

Perfil inicial propuesto y autorizado como política operativa: coordinación/specs GPT-6.1 Sol medium; arquitectura y adversario Sol high; implementación Sol medium o high para lógica difícil; Luna high para extracción y tareas pequeñas delimitadas; Astra medium o superior para decisiones especialmente difíciles y desacuerdos persistentes. Revisiones importantes usan capacidad comparable al autor; independencia significa agente/contexto distinto, no necesariamente modelo distinto.

Los modelos son hipótesis de eficiencia, no garantía. Medir coste por trabajo aceptado, defectos posteriores, falsos positivos y rondas; ajustar al menor coste que mantenga calidad. No usar max/ultra ni múltiples revisiones redundantes por defecto. La disponibilidad real del cliente prevalece y se registra cualquier sustitución.

## Referencias

- [Scrum Guide](https://scrumguides.org/scrum-guide.html): referencia del marco original; nuestra cadencia por objetivos es una adaptación explícita.
- [Harness de desarrollo prolongado](https://www.anthropic.com/engineering/harness-design-long-running-apps): contratos comprobables, evaluador independiente y reducción de sobrecoste; evidencia externa, no prueba de R3Viewer.
- [Subagentes OpenAI](https://learn.chatgpt.com/docs/agent-configuration/subagents) y [selección de modelos](https://developers.openai.com/api/docs/guides/model-selection): funciones acotadas, costes y evaluación de perfiles.
- [Revisión de Google](https://google.github.io/eng-practices/review/reviewer/standard.html): evidencia, preferencias y resolución de desacuerdos.

## Aplicación inicial

S01 propone un corte de lectura textual usable con CLI, redirección y sesión segura. Puente Nu tipado y gráficos se validan por capacidad sin bloquear lectura de archivos. Las diez épicas y 39 RV conservan su aceptación. El plan exacto de S01 se refina y revisa antes de implementar.

## Cierre de revisión inicial

Metodología revisada por architecture_adversary sobre archivo exacto y README coherente: SIN_CRITICAS_PENDIENTES. M-01 corregida (estado inicial obsoleto de README). Cierre de proceso, no evidencia runtime.
