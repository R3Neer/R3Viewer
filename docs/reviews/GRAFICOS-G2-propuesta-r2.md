# G2 — revisión independiente de propuesta siguiente R2

2026-10-05. Resultado: **SIN_CRITICAS_PENDIENTES** para la propuesta experimental revisada. G2R-01, G2R-02 y G2R-03 quedan cerrados en diseño. No equivale a aprobación de una implementación, adopción de backend ni cierre de G1/G2.

Objeto: `GRAFICOS-G2-next-proposal-r2.md`, SHA256 verificado `BF967BBF213978A61FF821ABF92D3D72E322D48779FE1C1BC133779102459672`. Comparado con contrato canónico `docs/refinement/GRAFICOS-PNG.md` R2, investigación `docs/research/GRAFICOS_BACKENDS_SPIKE.md` y revisión anterior `GRAFICOS-G2-next-proposal-review-r1.md` SHA256 `CE9E96DE298D1CA428C3A3A7D9BC0CF71FDF7A2BFCBFFBBC894FA70B8C933BAD`.

Esta revisión evalúa una nueva propuesta de otro autor. No revalida el spike anterior realizado por este revisor. Sólo inspección de documentos y fuente primaria; no cambios de producción/Cargo/XML, experimentos nuevos, compilación ni QA visual.

## Cierre de críticas R1

| ID | Corrección verificada en R2 | Resultado |
|---|---|---|
| G2R-01 | Perfil experimental PNG estático; detectar acTL/fcTL/fdAT produce referencia0 sin integridad de animación. Scanner de snapshot acotado comprueba todos los CRC e IEND vacío al final exacto; bytes posteriores se rechazan. Checksums y ancillary CRC se activan expresamente antes de read_info. Ready limita su afirmación a framing/CRC y deflate/Adler del único frame, sin certificar semántica universal ancillary. Corpus distingue Adler con CRC recalculado, CRC ancillary, APNG posterior corrupto y trailing bytes. | Cerrado en propuesta. No omisión de frames bajo una promesa de integridad completa. |
| G2R-02 | Worker retorna cuerpos separados de framing. Controller genera comandos e IDs propios. Descriptor canónico completo comparado con request autorizado; caps se derivan antes de leer/reservar cuerpo. End/hash/EOF/exit/reap necesarios antes de Ready; cancelación/stale invalida resultado. Kitty/PNG son datos binarios destinados a base64; Sixel tiene parser de gramática restringida con límites de números/paleta/runs/bandas/dimensiones y sin C0/C1/ESC. Corpus incluye cuerpos hostiles, descriptor incorrecto, exceso, truncado/extra/stale. | Cerrado en propuesta. La igualdad semántica de píxeles queda correctamente en el oracle independiente, separada de la seguridad de framing. |
| G2R-03 | Fit=min(1,Vw/W,Vh/H), viewport vacío sin raster, cuantización mínima explícita. Ampliación manual acotada sin excepción fit>8; resize conserva escala/ancla y ajusta crop. Corpus de imágenes pequeñas/grandes/extremos y endpoints. | Cerrado en propuesta. Compacto/0 no amplía automáticamente. |

## Segunda inspección técnica

Adam7Info::new(pass,line,W), PartialEq/Eq y significado de W como ancho original se verificaron directamente en png0.18.1 `src/adam7.rs:13–65`. La comparación propuesta de secuencia completa, longitudes y número de filas evita usar campos privados/Debug y no requiere fork. Transformaciones grayscale/RGB/RGBA/tRNS y nearest deben seguir demostrándose mediante el oracle; no se presuponen correctas por la API.

DecodeOptions tiene ambos setters indicados en `src/decoder/stream.rs:503–526`; la nueva política elimina la dependencia de defaults permisivos. El scanner complementa las comprobaciones del codec sin adjudicarse conformidad semántica universal. La limitación APNG es experimental y conserva el alcance futuro vigente.

StreamWriter::finish consume/finaliza su stream y Writer::finish es distinto; la propuesta propaga ambos errores y cuenta los tres buffers de fila más chunk/compresor antes del constructor. Las reservas internas restantes continúan sujetas a auditoría/medición efectiva, como exige el plan, no a una garantía inventada de hard cap del crate.

Se revisaron también admisión old+new, coexistencia spool/base64/IPC, raster visible, alpha/mate, clipping, cancelación total/reap, generación y cache/IDs. No se encontraron otros fallos accionables en el nivel de esta propuesta. Las condiciones pendientes están delimitadas: política ICC/fidelidad, contención real de codec/compresor, snapshot Unix, gramática efectiva del emisor Sixel y limpieza/transmisión terminal deberán acreditarse antes de cualquier adopción. Su evidencia está pendiente, no sustituida por este resultado.

Fuentes primarias: [Reader png0.18.1](https://docs.rs/png/0.18.1/png/struct.Reader.html), [Decoder png0.18.1](https://docs.rs/png/0.18.1/png/struct.Decoder.html), [Kitty graphics](https://sw.kovidgoyal.net/kitty/graphics-protocol/), [iTerm2 images](https://iterm2.com/documentation-images.html). Se conservan las fuentes locales versionadas e identificadas en la revisión R1: registry `D:\Tools\R3Viewer\.tools\cargo\registry\src\index.crates.io-1949cf8c6b5b557f\png-0.18.1`, SHA de adam7.rs `6698303F534586F6D2CD952C800D4379F1F73DB872D69C05689AE337FA4DA2F6`, decoder/stream.rs `AF87E912ACCD5057DCB098CA8AA3B6D4A1A1A2ABC2412955409EE8260112E4FF`, encoder.rs `787F6F8922588A74436C76A3B2C2B5689206F7ED13A19A36227B8809710812CE`.

## Alcance del resultado

**SIN_CRITICAS_PENDIENTES** autoriza considerar esta propuesta suficientemente concreta para su siguiente experimento privado, sujeto al encargo del coordinador. No aporta mediciones ni certifica que el experimento vaya a cumplirlas. Mantener ledger/corpus/oracles y adversario independiente del resultado real; ningún gate de ejecución o visual, soporte nativo, requisito final o tarea de backlog se cierra aquí.
