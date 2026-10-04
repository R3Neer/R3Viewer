# R3Viewer

Visor de archivos, documentos y valores de Nushell, con lectura interactiva y salida inline (`--inline/-i`). Windows nativo, macOS y Linux son objetivos de soporte.

Estado: desarrollo autorizado, primer incremento en preparación. Todavía no hay una entrega certificada. Implementación, revisión iterativa y commits locales autorizados; publicación e instalación en perfiles personales fuera del encargo.

## Documentación

- [Requisitos](docs/spec/REQUISITOS.md), [decisiones](docs/spec/DECISIONES.md) y [preguntas](docs/spec/PREGUNTAS.md).
- [Contratos](docs/spec/CONTRATOS.md), [HTML/web](docs/spec/WEB_HTML.md) y [estudio previo](docs/spec/ESTUDIO_ADICIONAL.md).
- [Product Backlog](docs/BACKLOG.md): 39 tareas, dependencias y aceptación; diez épicas en el plan general.
- [Investigación técnica](docs/research/INVESTIGACION_TECNICA.md): fuentes, conclusiones y límites de evidencia.
- [Arquitectura general y épicas](docs/architecture/ARQUITECTURA.md), revisada iterativamente con adversario independiente hasta cierre sin críticas, y [validación por capacidad](docs/VALIDACION.md).
- [Política de commits](CONTRIBUTING.md).

## Uso previsto, todavía no implementado

```nu
viewer notas.md
viewer -i notas.md
viewer https://ejemplo.org
"notas.md" | viewer
ls | viewer
viewer --value "# Un título"
"notas.md" | viewer --path
```

Argumentos abren rutas o URLs; el pipeline contiene valores. `--path/-p` y `--value/-v` fuerzan la interpretación y no coexisten. Se mantienen `open` y `explore` nativos; `from html` será una utilidad opcional.

## Ubicación canónica

Repositorio local: `D:\Tools\R3Viewer`. Los documentos iniciales de este chat se copiaron conservando el historial como primer commit. A partir de la migración las modificaciones se hacen aquí; las copias del chat son una referencia histórica.

No hay remoto configurado. Licencia del proyecto: [MIT](LICENSE), confirmada el 2026-10-04. Las dependencias conservan sus licencias y avisos; revisarlos antes de distribuir.

## Forma de trabajo

Adaptación de Scrum por objetivos: un Product Backlog, un sprint activo sin cadencia temporal fija y detalles refinados según prioridad/riesgo. Responsabilidades, aceptación y Definition of Done en docs/process/METODOLOGIA.md. Revisiones iterativas independientes de propuestas y código; informes intermedios temporales, versiones finales y evidencia canónicas.

## Desarrollo vigente

El usuario autoriza implementar el producto completo siguiendo [Scrum por objetivos y revisión adversaria](docs/process/METODOLOGIA.md), sin cadencia fija. La descripción de preparación anterior es histórica. Sin publicación ni modificación de perfiles personales.
