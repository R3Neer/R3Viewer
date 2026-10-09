# Adopción del renderer documental Markdown b2 R1

2026-10-09. Incorporación del delta aprobado de biblioteca textual sobre HEADac4f44e. [Aceptación independiente](Markdown-D2b2-renderer-acceptance-r1.md): SIN_CRITICAS_PENDIENTES limitado al corte congelado; no cierre de RV-301/RV-306.

## Cambio

Presenta títulos, citas, callouts, listas/tareas, tablas completas con continuaciones por celda y alternativa vertical, código/frontmatter literal, HTML inerte y notas al pie. Image/Math/Wiki/Mermaid permanecen PendingRequired seguros, con campos Input/Alt/Title/Kind/Info/Label distintos. No se cargan recursos ni se acredita representación gráfica.

Role/Ref/Resource/Table tienen anclas derivadas del modelo y proveedores reales; Role conserva el Enter real y TableResume mantiene At/Done por celda ante resize. La reserva de layout/draw conserva DocumentWork8MiB, Window128KiB, Frame256KiB, Consumer1MiB, Framework2MiB y checkpoints4096. Dense120x40 usa7417231 Work inicial y7422031 en redraw; no se suma held a spent. Geometría e índices se preparan en fase separada cancelable, no se acreditan como SAMEWork de draw.

## Evidencia y composición

Freeze FREEZE-b2-r1.json SHA256E55ED3D66A52791ADF3A9DDFE2DEBB4384F135FFAAB07B2ADDD4CF9C5DCCEBFD. Root verificó724 fuentes,29 deltas,2 paquetes y40 evidencias. Base del autor7a7b9a7/código353418c; se conservan búsqueda y documentos canónicos posteriores. Se copian28 deltas byteexactos y se añaden los dos módulos a lib.rs sin retirar document_search. Las16 fixtures tienen atributo -text y conservan los bytes revisados; Rust mantiene su política LF. SourceLegacy, Cargo y vendor intactos.

Autor:119 pruebas focales únicas PASS, fmt/Clippy PASS. Adversario:cuatro grupos propios PASS, primero0.06s y suplemento prospectivo0.36s, sin adaptación API ni relajación de oráculos. Comprueba tablas/resize, referencias largas, roles/folds, Unicode, límites de clases y símbolos16KiB/+1, cancelación, WholeReference→Text y geometría extrema sin IO. Informe privado SHA2567DFB9BB758B4DA9A43807426D353E27845E83F94084E01F129A7EFD0A69EA4F5; evidencia3E814F5953CFCA972B9DD11BC7576ACBCA304540003FF0C41CFBB183A5724D0F, artefactos verificados. Bin propio suplementado congelado antes runtime1E48AB07B93A08DF0AB9ABB97B31EB156B192429E5A57B097E479E9359C2A42A.

Root:composición privada y después bin canónico pasan133 regresiones focales (14 búsqueda+119 vistas/renderer/owner/storage/Flow/Text/Ref/Code/roles). Bin canónico preservadoF630656BF51593FE1991A5D8E568E1308608B1683222C64ECF61AB9E10F273FE. cargo fmt --all -- --check y cargo clippy --offline --locked --workspace --all-targets -- -D warnings PASS en canónico. Sólo se omiten dc04_32m_borrowed_and_cancelled_no_replay, rf04_whole_giant_tail y d2a07_giant_first_terminal_no_root, corpus gigantes previamente acreditados; no se elimina/ignora su código. El caso9K sí está incluido. Fallos previos y oráculos iniciales permanecen intactos fuera de Git.

Errata del documento privado congelado SURFACES:Operation::Presentation es variante SIN payload; DocumentInput captura ancla real desde ventana/current. Fuente y API revisadas correctas; no se reescribe el artefacto congelado.

## Pendiente

Conexión viewer_session/main, Loading/chrome/teclado/pump, inspector/acciones/SourceTransition/historial, fences y gráficos. Markdown interactivo sigue mostrando Fuente con el aviso vigente. No acredita ConPTY de sesión documental, QA visual WindowsTerminal/WezTerm ni ejecución nativa Unix. La revisión D2c está en curso; no se anuncia producto completo por este incremento.
