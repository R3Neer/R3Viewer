# Gate privado Rust/PDFium — estado R4

R4 recibe revisión de [código](../reviews/PDF-Rust-code-r4.md) y [aceptación](../reviews/PDF-Rust-aceptacion-r4.md) independientes sin críticas pendientes en el delta de lifecycle/I/O. MANIFEST283D501428075005723D6476D646D18571260B1BBFC78986CF27E941D72B6381,949entradas; binarioC8C6717A2161E23ED1A7B0D548C6008CB07DEC80F36D28FFC1A46B0E4C7D7B73. Rust/ABI/DLL sin cambio respecto a R2/R3, snapshots privados preservados.

R1/R2/R3 rechazados permanecen conservados. R4 cierra PRUST-ROOT02: errores/overflow de lectores y errores del writer, incluido cierre, se comprueban después de joins antes de publicar. AE reproduce8receipts tardíos y39regresiones; invocaciones fallidas siguen fallidas, Pending retiene autoridad y bloquea nuevo actor hasta recuperar por el mismo helper. Controles sin rescate externo. El informe conserva los errores propios de investigación separados de fallos del candidato.

No hay aceptación semántica completa: PDFium7881 puede devolver huecos para astrales y el fixture de cadenaTj larga conserva la divergencia32767/65536. GetTextUTF16 de contraste tiene carry corregido, pero no acredita automáticamente mapping o texto completo. Las capas mantienen semantic_state Unverified; original FAIL no se convierte en PASS por aprobar limpieza del supervisor. Investigación de causa/alternativas sigue abierta.

Falta integrar manejo PDF lazy/páginas/texto/búsqueda/navegación/recursos y validar límites reales, Unix y presentación. [Contrato del gate](../refinement/PDF-RUST-GATE.md) y [PDFG0](../refinement/PDF-G0.md). RV-204/G1 y producto siguen abiertos.
