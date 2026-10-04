# Política de commits y colaboración

## Alcance

Un commit representa un cambio coherente y revisable. Documentación, investigación, prototipos e implementación tienen evidencia distinta; un commit de documentación no acredita funcionamiento. El backlog no es autorización permanente para implementar o publicar.

## Mensajes

Conventional Commits con descripción en español, verbo concreto y sin punto final:

```text
tipo(ámbito): descripción

Motivo y efectos relevantes, si no resultan evidentes.
Validación realizada y límites, cuando corresponda.

Backlog: RV-003, RV-004
```

Tipos: feat, fix, docs, refactor, perf, test, build, ci, chore. Ámbitos estables: repo, spec, cli, nu, core, tui, inline, graphics, pdf, markdown, obsidian, html, data, packaging. Breaking changes usan `!` y un pie `BREAKING CHANGE:` que explica migración. No añadir resultados de validación que no se hayan ejecutado.

Ejemplos: `docs(spec): fijar entrada de archivos y valores`; `fix(tui): restaurar el cursor tras cancelar`; `build(packaging): fijar el motor PDF para Windows`.

## Preparación y verificación

1. Leer estado, rama y diff antes de modificar; conservar trabajo ajeno.
2. Separar cambios independientes y evitar commits masivos de fin de sesión.
3. Preparar rutas explícitas; prohibido git add . y equivalentes globales.
4. Revisar diff --cached, ejecutar checks pertinentes y comprobar que no hay cachés, binarios generados, secretos ni documentos privados.
5. Crear commit y revisar estado y log. Los fallos de hooks o firma se investigan; no desactivarlos sin autorización.

Commits de investigación describen fuentes y distinguen documentación, inspección local, prototipo y QA visual. No crear tests que solamente espejen cambios documentales.

## Ramas e historial

Rama base local: main. Desarrollo posterior: codex/<tema> para agentes, sin cambiar de rama con trabajo ajeno incompatible. La preparación inicial puede crear commits directamente en main del repositorio nuevo. No forzar push, reescribir commits compartidos, descartar cambios ni alterar configuración global por iniciativa propia. No añadir remoto, publicar, etiquetar release o instalar sin autorización de destino.

No se impone firma nueva ni hooks todavía. Se respeta la identidad y firma configuradas. La validación automática del formato de mensajes se añadirá solo si su utilidad justifica el mantenimiento.

## Documentación y cierre

Mantener referencias relativas válidas. Los requisitos confirmados se separan de propuestas, y una decisión técnica indica fecha, evidencia y efectos. Un elemento del backlog se cierra solo cuando cumple su aceptación; investigación cerrada no significa QA runtime terminada.

Licencia del proyecto pendiente; revisión de avisos y licencias de dependencias obligatoria antes de distribuir. No incorporar apuntes privados al repositorio: fixtures propios/sintéticos o autorizados.
