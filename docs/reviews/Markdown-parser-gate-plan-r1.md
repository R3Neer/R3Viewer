# Markdown parser gate — revisión adversaria R1

2026-10-05. **SIN_CRITICAS_PENDIENTES, acotado al diseño del ensayo privado.** Autor de esta revisión distinto del coordinador autor de la propuesta. No implementación, descarga, compilación ni outputs del candidato observados.

Propuesta verificada `Markdown-parser-gate-proposal-r1.md` SHA256 **D5707477DFF3BEA99447BB7CECFA563FAB2C68CD223344B2B9807D74C2859476**. No modificación de propuesta/producción. Contraste con docs/spec/REQUISITOS.md, CONTRATOS.md y DECISIONES.md: Markdown gráfico completo y Obsidian amplio siguen obligatorios; este ensayo no los declara satisfechos ni abre S04.

## Contraste de APIs y procedencia

El [código lib.rs del tag0.13.4](https://raw.githubusercontent.com/pulldown-cmark/pulldown-cmark/v0.13.4/pulldown-cmark/src/lib.rs) permite consumir eventos sin HTML. Expone tipos de bloque/inline, Link/Image, código, matemática y metadata; Text consecutivos son legales. Es correcto exigir balance/UTF8/rangos dentro de fuente sin exigir disjunción universal de rangos. No existe TableBody: la propuesta lo contempla. La fuente/licencia consultada confirma MIT, pero build aún debe bloquear fuentes/lock/features y conservar avisos.

En [parse.rs del tag0.13.4](https://raw.githubusercontent.com/pulldown-cmark/pulldown-cmark/v0.13.4/pulldown-cmark/src/parse.rs), new_ext pasa a new_with_broken_link_callback y ejecuta run_first_pass durante construcción; into_offset_iter entrega parejas Event/Range. Por eso pull/4096eventos no demuestra cancelación de constructor/next ni limita todas sus asignaciones internas. La propuesta reconoce esta limitación y separa proceso/deadline de contención de producto. Se preserva relación token-source y texto decoded/normalizado, evitando fingir mapping por scalar o reconstruir raw desde eventos.

[Options0.13.4](https://docs.rs/pulldown-cmark/0.13.4/pulldown_cmark/struct.Options.html) distingue GFM, wikilinks, math, metadata y otras extensiones: GFM no implica todo Obsidian. La propuesta exige opciones/parámetros exactos y expectativas previas. Al congelar fixtures debe registrarse bitset y nombres de flags por perfil antes de outputs; no interpretar cobertura obtenida con Options::all como perfil de producto. Es una concreción de la trazabilidad ya exigida, no un defecto nuevo del diseño.

El [Cargo.toml exacto](https://raw.githubusercontent.com/pulldown-cmark/pulldown-cmark/v0.13.4/pulldown-cmark/Cargo.toml) será fuente del feature graph: verificar defaults efectivos, no sólo flags solicitados. La propuesta ya evita activar HTML/SIMD/serde gratuitamente. API y consulta web no sustituyen hash/crate/Cargo.lock ni ejecución.

## Condiciones para aceptar la implementación del gate

Estas son condiciones ya derivadas de la propuesta y su corte; no resultados ejecutados ni críticas que obliguen otra ronda documental:

- Oracle anterior al output para cada fixture, incluyendo offsets seleccionados en Unicode, wiki/alias/embed y texto decoded frente a raw. Tolerar patrones reales de múltiples Text, pero no sustituir las expectativas semánticas por los eventos que salgan.
- Distinguir inline HTML de bloques HTML: no asumir que todo lo situado entre etiquetas inline es un único payload HTML protegido. El corpus contextual de código/HTML deberá describir la clasificación esperada, sin regex global posterior. Callout personalizado, tags/bloques/anclas no reconocidos conservan su fuente y limitación explícita; no significan soporte completo de Obsidian.
- Límite32KiB por entrada y4096eventos son del ensayo; exacto/+1 debe clasificar fallo/limit. Cap stdout1MiB debe verificarse antes de acumular más bytes, con stderr acotado y diagnóstico de overflow; un prefix JSON/events truncado no es documento completo. No imprimir ESC/OSC de payload a consola: salida tipada serializada/escapada a evidencia.
- Supervisor posee Popen inmediatamente, cleanup aun si falla setup/captura/logging, deadline único incluye construcción/iteración, wait y cierre propios. Cancel/panic/exit/output incompleto no verde. El ensayo no certifica Job/Unix/scheduler del producto y no debe etiquetarlos como tales.
- Freeze incluye source, opciones, fixture bytes/hashes, expected previos, lock/licencias, bin/runner y resultados raw; AE posterior sobre identidad exacta. Recursos/enlaces quedan inertes, sin IO/callback de resolución ni ejecución de Mermaid/math/HTML.

## Dictamen y límites

No detecté contradicción accionable que impida ejecutar este gate. Su matriz y separación SourceSnapshot/arena/ResourceSpec son compatibles con explorar un modelo común sin convertir datos Nu universalmente ni perder raw. Eso no aprueba todavía el modelo final: coalescing/provenance detallada, adaptación Obsidian y resolución compartida, budgets de documentos grandes, navegación/layout/render gráfico requerirán su refinamiento y validación específicos. Sin aprobación de runtime/cancelación/memoria/terminal/Unix/QA visual ni cierre RV-301/302/303/305.
