# Biblioteca inline Markdown R3: integración, 2026-10-06

Se incorpora el emisor completo, roles compartidos y proveedor TextRef real sobre las APIs visuales R3 aceptadas. No se incorpora todavía el selector/FormatMarkdown ni el caller público: la aceptación funcional de `viewer -i archivo.md` requiere el siguiente incremento.

## Revisión y composición

Aceptaciones independientes finales: [inline R3](Markdown-inline-library-acceptance-r3.md) y [RefID/roles](Markdown-ref-roles-acceptance-r1.md). El proveedor combinado conserva las tres APIs visuales R3; el delta añade selección real de TextRef sin Run/Domain sintéticos y guardias que impiden reinterpretar referencias como coordenadas de Runs. El mismo Consumer y kernel Unicode cubren títulos, controles, ALT y destinos.

Fuentes congeladas incorporadas byteexactas: document_inline.rs SHA3A3BFE0304202B2520C7B88598E327D3E88F4F4FE59BB5B4B30C4701D9CC8BD3; document_roles.rs SHA81EB49B3D028637D5738859876A64523F62E2841F69532BEF09B9BAEADEF9A8F; proveedor R3+Ref document_text.rs SHA918F93EE74A2858C786FB8B21FCA89D92C0EDBBB839A7FDAA181B61FF2C42B52; tests Ref SHA7EAA3F2CA59E6E81AA3D647B67EA835B85B1CA61A9ED5422563F41242442503C. Base9e629a7. lib.rs conserva el contenido anterior y añade únicamente módulos privados.

La salida se prepara completa mediante dos pasadas, conserva clusters Unicode y proyecta controles; no abre destinos ni ejecuta HTML/Mermaid. El fallback conserva la fuente completa y el presupuesto de salida/cancelación impide devolver una cadena parcial. Esto acepta biblioteca, no integración CLI, imágenes, matemáticas, enlaces activos ni QA visual.

## Verificación compartida

53 pruebas pertinentes PASS:16 inline,4Ref,17document_view y16document_code, ejecutadas serialmente sobre el árbol compartido. Formato PASS. Primer Clippy detectó atributo allow(dead_code) duplicado entre lib y roles; se retiró únicamente la anotación añadida en lib, sin alterar las fuentes congeladas. Segundo Clippy todos los targets con -D warnings PASS. La compilación/pruebas no acreditan Unix ni visuales.

La integración inicial se detuvo antes de escribir por una ruta de informe incorrecta REPORT.md; se corrigió a REVIEW.md y se volvieron a verificar los hashes. Evidencia privada: inline-library-adoption-readback.json y inline-library-shared-clippy-attempt2.log en el directorio de trabajo del chat. No se añaden binarios/cachés/documentos privados al repositorio.

## Pendiente

Conectar worker real, selector y comando no interactivo; acreditar cancelación/cierre físico antes de publicar salida. Renderer interactivo, presupuesto práctico de documentos densos, activación de syntax y protocolos gráficos permanecen pendientes. No se cierra RV-301 por este incremento.
