# Adopción del núcleo semántico de búsqueda Markdown b3 R2

2026-10-09. Código base353418c; HEAD anterior28f1556 sólo añadía documentación. El núcleo privado R2 se incorpora después de revisión de código y aceptación propia del adversario sin críticas pendientes. No cambia CLI, sesión, layout, kernel de preparación, Cargo ni wire.

## Cambio

Walker sobre Items validados, incluido contenido oculto por folds; búsqueda KMP por proveedores D1 compatibles, query4096bytes, solapamiento y una vuelta, dominios MainBody/CalloutTitle/ResourceAlt/ResourceTitle/SourceOriginal. DocumentHit conserva cobertura y asociación real; SourceHit usa bytes absolutos originales sin inventar Run/Domain. Resultados Pending/Failed/Cancelled permanecen distintos de NoMatch. Headings se preparan en páginas de64 objetivos lógicos, incluidos los ocultos.

R1 perdió ResourceID al compartir títulos TextRef; B3-CS-OWNER01 bloqueó adopción pese a13+1 grupos propios positivos. R2 retiene ResourceID sellado y copiado desde el record validado en Sequence/Hit/Pending/retry. La reproducción original distingue owners0/1 y mantiene asociación al reintentar. R1, informe y binarios permanecen intactos en área privada.

## Evidencia

- [Aceptación adversaria R2](Markdown-D2b3-coresearch-acceptance-r2.md):15 grupos propios PASS; dictamen SIN_CRITICAS_PENDIENTES. Informe privado SHA256B980E3A70CE15397A0013CFC2EED77EAD58105D373081433C7E47A59284A3252; bin adversarioEFEA6F7C2FAFE859062C8EF299C38018A761F00986EB6DB3324B2416901209A0.
- Freeze final13artifacts verificado, manifiesto7A0E795560E0A84FF89B88BB827AA96A9DE3FDCBC26158717258E42CF6957579. FuenteAC413B53C7BE38247FCA832479B9722815D8F3C42E45AA2AC2297CF7275D596E, tests809ADE5DB4E1A0D057410C9503EFC14B8CCCD48D541E54D7E752531C85FD5F3D, comparados byteexactos en workingtree antes del commit. Git mantiene su política LF para Rust; fixtures conservados byte a byte mediante atributo -text.
- Composición canónica:14 búsqueda +15 Flow +17 vistas +11 renderer +8 propietario/almacenamiento =65 pruebas PASS. Se omite únicamente d2a07_giant_first_terminal_no_root, corpus32MiB previamente acreditado y sin cambio aquí.
- Bin de composición preservado antes de reutilizar target:CA05A0FD9F31540BAF5C2E28AC3AB680D114CD5C3E25C9D321CDE758E8AD4E62.
- cargo fmt --all -- --check y cargo clippy --offline --locked --workspace --all-targets -- -D warnings PASS; diff --check PASS.

## Límites de aceptación

Preparación D1 tiene su propio Work cancelable; find_with_work inyecta recorrido/matcher/cobertura, no limita toda CPU del provider. FoundPending con highlight ausente no acredita posición visual final. SourceMatched sólo acredita coincidencia original, no modelo SourceReady ni transición publicable.

Inspector/chooser, expansión y posicionamiento de objetivos, SourceTransition, historial atrás/adelante, pump, teclas, publicación de frames, fences/gráficos, QA visual y plataformas nativas Unix siguen pendientes. No cierre de b3 completo ni RV-301/RV-306. El comando Markdown interactivo conserva su comportamiento anterior hasta la integración de sesión.
