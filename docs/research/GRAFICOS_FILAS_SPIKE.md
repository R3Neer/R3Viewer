# Experimento gráfico R2: corrección de integridad

Autor R2: coordinador raíz, distinto del autor R1. Candidato privado D:\Tools\R3Viewer\.tmp\graphics-row-gate-r2; R1 se conserva intacto. Pendiente revisión y aceptación independientes. No modificación de producción ni cierre de G2/RV.

Responde G2ROW-01/02/03 de GRAFICOS-row-gate-review-r1.md SHA A46FFACF060D34E4EA12F00596D5FA113D29B945534793CE4770C6D1F66A8FFC.

## Cambios

- G2ROW-01: pase zlib streaming con miniz_oxide=0.8.9, ya presente en lock R1 y avisos originales. Cuenta longitud exacta de scanlines serializadas, incluidos filtros, packing y siete pasadas Adam7 no vacías. Input y output por iteración <=4096 bytes, estado inflater en Box, sin copiar IDAT ni reservar el raster completo. StreamEnd requerido; checks de cancel/plazo en cada avance. Bytes comprimidos finales permitidos por PNG11.2.3 permanecen ignorables, pero bytes descomprimidos extras dejan reference.
- G2ROW-02: scanner verifica PLTE única, no vacía, longitud múltiplo de tres, máximo 256 entradas/2^depth para indexed, prohibida en grayscale/gray-alpha y antes de cualquier IDAT. Indexed requiere PLTE anterior. IDAT contiguo y al menos uno. Conserva CRC/IEND/EOF del R1.
- G2ROW-03: para indexed, pase adicional del codec con Transformations::IDENTITY inspecciona cada índice nativo empaquetado, incluso fuera del crop y en Adam7. Sólo luego EXPAND genera RGBA visible. No se acepta el negro sintetizado por el codec para un índice fuera de PLTE. La fila de este pase es <=fila original y no coexiste con los buffers del pase RGBA; usa los mismos límites codec y allocator global.

El plan previo de filas permanece antes de estos pases; scratch fijo4KiB y estado inflater pequeño quedan dentro del margen32MiB ya admitido y del hard cap de requested Rust heap256MiB. Esto no demuestra auditoría total de stacks/DLL/OS ni memoria agregada padre+hijo. La expansión PNG hacia el viewport y los emisores del R1 permanecen iguales.

## Verificación de autor

8 unitarios PASS; fmt y Clippy alltargets-Dwarnings PASS; build offline fijado PASS. 204 invocaciones de autor PASS (26 en RESULTS.json y178 en LARGE.json), con serializer/oracle matemático Python conservado y transporte decodificado byte por byte en el corpus pequeño. Son comprobaciones de autor R2, no AE independiente R2.

Incluyen los 162 pequeños originales estáticos/Adam7 (1..9 por dimensión), cinco filtros/IDAT fragmentado/crop/mate, grayscale packed+tRNS, los contraejemplos row/byte extra, paleta crítica fuera de orden/color y índice inválido. Control positivo de padding comprimido permitido. Paleta inválida fuera del crop y paletas Adam7 de1/2/4/8bits válidas/invalidas; tres cancelaciones cooperativas internas. Los casos grandes usan las fuentes selladas del R1 y sus CRC oracle sin alterar goldens.

16M noisy64011493B→256²: requested heap pico Kitty64702704B / iTerm289795927B; transportes350331B/351141B. Última observación debug1297/1433ms, sin SLA. No Job/commit nuevo ejecutado por este autor: las medidas R1 no se presentan como aprobación automática de R2.

Durante desarrollo falló primero el manejo de MZError::Buf en un input segmentado grande: miniz puede escribir bytes pendientes y pedir más input. Se corrigió sólo la condición de avance para aceptar Buf con estado NeedsMoreInput y segmento consumido; estados de corrupción/falta de progreso siguen diagnosticados. Se reejecutó todo el corpus final sin cambiar expectativas. Un aviso Clippy sobre is_multiple_of se corrigió. Se preservan scripts de preparación y patch para inspección, no son editores de producción.

## Evidencia y límites

MANIFEST.json del R2 conserva Cargo/src/exe/licencias/scripts/corpus/evidencia de esta entrega. R1 original MANIFEST C935F522E077B008DECF6AF0E99FCECA658362D1C11C5EE42758CB099E8EBFB2 y sus251 entradas se revalidan antes de freeze; las fuentes grandes reutilizadas permanecen referenciadas allí. Reproducir corpus fuera del snapshot: los runners escriben evidencia.

[PNG3, §11.2.2 y §11.2.3](https://www.w3.org/TR/png-3/#11PLTE) y fuente local fijada png0.18.1/miniz_oxide0.8.9 son referencias de los fixes. R2 no convierte el perfil en validador universal de toda semántica ancillary/ICC.

Sixel, IPC completo real y descriptor/hash completo, cache, memoria agregada, snapshots Unix, geometría/zoom/terminal lifecycle/inline, deadline UI y QA visual/nativa conservan todas las brechas del informe R1. Sin adopción ni aprobación de producción; se pide revisión independiente exacta y aceptación diferente del autor antes de integrar cualquier parte.

## Estado canónico de revisión

[Adversario R2](../reviews/GRAFICOS-filas-r2.md): SIN_CRITICAS_PENDIENTES para el corte exacto, tras tres fallos encontrados en R1 (raster descomprimido sobrante, paleta crítica inválida e índices inexistentes). Sus 250 casos propios y ocho unitarios pasan; aceptación independiente R2 en ejecución. No adopción de producción ni cierre de G2/RV. El informe de autor anterior y críticas intermedias permanecen temporales.

[Diseño G2](../refinement/GRAFICOS-G2.md) y [contrato PNG](../refinement/GRAFICOS-PNG.md) conservan todas las brechas no entregadas: Sixel, IPC real, caché, memoria agregada, terminal/inline y QA visual/nativa. Fuente y exe del experimento R2 sellados por MANIFEST SHA EE62D71FE9879B6AAF085960EF18A062F77D362CCE356DDBA0B44B136396E132; revisión original SHA B17475A66A612BF34ACAA4C6DE46BB4C76907C34801E8F15E7E63C68E8AAC372. Los SHA originales preceden normalización Git.

Aceptación R2 posterior: [AE independiente](../reviews/GRAFICOS-filas-aceptacion-r2.md) ejecuta171casos de corpus, ocho tests, tres cancelaciones cooperativas y cuatro Jobs Windows propios. Manifest122/exe/source intactos; committed pico observado66,080,768B Kitty y91,189,248B iTerm2; kill-on-close/reap y readers correctos. SIN_CRITICAS_PENDIENTES dentro de las pruebas ejecutadas. Estos Jobs renuevan evidencia para R2; no acreditan memoria agregada ni terminal/visual ni G2 entero. Informe original SHA9468E0131A85FEABB5878C0BBEBEB1091AAA043477D576711323DAEC44343852; AE manifest EF77A45E5D20630F3CEAB77A44035D1BB80922C3DF7C2334DED654C14D99A777.
