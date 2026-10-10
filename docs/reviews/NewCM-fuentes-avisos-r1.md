# Dictamen independiente NewCM publicado y avisos embebidos

2026-10-10. **SIN_CRITICAS_PENDIENTES para la exactitud y alcance de esta investigación**, sobre las tres entregas exactas siguientes. No aceptación de diseño/motor/instalación/licencia global/distribución ni cierre RV-302/303/501.

- Captura publicada MANIFEST1A07DC4D7EFBC0FFF90D68EC63AFF86CA713E8FD6C00A929D680A9BB251B3573:22/22 archivos físicos propios PASS. FINDINGS E3574288FCA02F586F7FFFD26E3157BBFD37E297AE6B4F6EE4DA25BFB7F2CB6D.
- WOFF2 nombres MANIFEST E611701D8BFFECED1057500A69853F22BD31856C4C10824169C8A5F3BEF93CE1:228/228 físicos propios PASS.
- Referencias primarias MANIFEST D26DCD932E0D61D5A02CF9C1B4CCB347D2F79D02D15EB26D3A0317C94F0B1B71:11/11 físicos propios PASS; cuatro cuerpos/status/URLs/hash del recibo coinciden.

Criterios NC01–06 fijados antes salidas: CRITERIOS-before-candidate.md SHA7EB60114B70ED4F35BB32082336BD0D97A186AB0C0D286C5E97D692909FEEBC9. Todas salidas propias están aquí; no se modificaron entregas congeladas ni producción.

## Procedencia y corpus

Own Python verifica SHA1 e integritySHA512 contra metadata original registrado para @mathjax/mathjax-newcm-font4.1.3. Tarball13872892B SHA25687D7B869C6A2A6169D9A53ACC4EAB6C846A9CBE11752738226461BB5070C8B88. No verificamos npm firmas/attestations ni sustituyamos metadata por consulta latest. Inventario propio completo1124 archivos regulares/49247851B: path/type/bytes/SHA de todos los miembros igual;2html/392js/8json/308map/309ts/105woff2. package.json y cuatro selecciones mjs/svg.js, mjs/chtml.js, svg.js, def/mathjax-newcm.ts byteexactos con archivo original; no igualdad inferida de un repositorio entero.

Paquete declara Apache-2.0. Búsqueda propia de rutas LICENSE/LICENCE/NOTICE/COPYING/COPYRIGHT/OFL/GUST produce cero; este resultado sólo describe nombres. La receta publicada referencia Font.load de NewCM Math/10/sans/mono y addCopyright2024MathJax. Se leen imports de SvgFontData, dynamicPrefix y fontURL/font-face de CHTML; son referencias de código, no reproducción de fuentes originales, contornos, resolución offline o loader ejecutado. La declaración anterior @mathjax/src4.1.3 de esta dependencia permanece evidencia previa, no se ejecutó ahora.

Intento de origen exacto conservado: script pide MathJax-fonts tree para gitHead b3b77f655ed2d9009078301626e27b56dcfa90d9 y falla HTTP404 antes de TREE/RESULT. No evidencia de árbol original verificado, ni reemplazo por main. Se preservan seis archivos de ese intento en origin-attempt-preserved; se inspecciona el intento registrado, no se afirma nueva consulta GitHub propia ni inexistencia global del commit.

## Reproducción independiente de los nombres

Own verify-own.py SHA D4B58394A2D57E58540077E4B395977C4B1E2A349F93DF4A4FE5E7E438EA8245 usa Python estándar; NO importa FontTools ni MathJax. Parser manual según [WOFF2 W3C](https://www.w3.org/TR/WOFF2/) interpreta header, tabla de63tags, UIntBase128 y longitudes transformadas; rechaza colecciones/duplicados/versiones no manejadas/name transformado y acota entradas/directorios/streams. Reconstruye105 bloques Brotli desde tarball exactamente iguales a los capturados, con offsets de name propios. No reconstruye tablas glyf ni SFNT ni certifica conformidad integral del font.

Own decompress-own.ps1 usa BrotliStream .NET en proceso privado,32KiB por lectura,16MiB por fuente/256MiB acumulado, tamaño exacto y EOF. Produce105 streams/4196758B byteexactos con los streams capturados. El parser manual de [OpenType name Microsoft](https://learn.microsoft.com/en-us/typography/opentype/spec/name) verifica header versión0, tabla/offsets/longitudes y cada string. Plataformas0/encoding4 y3/encoding1 UTF16BE,1/encoding0 MacRoman;1785registros de cada par. 5355 registros completos, raw_hex e identificadores/texto EXACTOS frente al candidato, cero error de decodificación. No se normalizan strings ni completan desde summaries.

Resultado propio:105files con GUST en algún nameID0;21 añaden MathJax2024. nameID13/14 contienen sólo espacio,315registros de cada ID; resumen completo de avisos calculado propio coincide exactamente. Algunos registros nameID0 de otras plataformas también son espacios; la afirmación es por archivo/al menos un registro, no todos los registros copyright.

OWN-NAMES.json SHA4B45BEADBB4E53CE0F2BE7D09ADFCF94F8863D1409BB6F58A6A69C3EBFE1A31E. NAMES-RESULT.json SHAAE5AD362DEF8360823CAC5BD478DEDC8923FFAD26A4668A39DABB50D1B0E5835. Scripts y expectativas/prechecks fijados antes outputs propios; prepare2/decompress/names/supplement exits0.

## Referencias y límites

Las cuatro respuestas conservadas son HTTP200 de captura urllib registrada, no ejecución de motor. [GUST e-foundry](https://www.gust.org.pl/projects/e-foundry/licenses/GUST-FONT-LICENSE.txt) y [TUG textual](https://tug.org/fonts/licenses/GUST-FONT-LICENSE.txt) se identifican GFL1.0,22Jun2009 y remiten LPPL1.3c o posterior;1417B/1377B y hashes son distintos. Ambos originales permanecen, sin igualdad normalizada supuesta. [CTAN GFL](https://ctan.org/license/gfl) es referencia HTML y [LPPL1.3c](https://www.latex-project.org/lppl/lppl-1-3c.txt) texto capturado separado. El Web403 previo no se relabela; el suplemento acota ese fallo a herramienta/consulta y aporta nuevos cuerpos200. No elegir licencia global ni inferir compatibilidad/cumplimiento por estas referencias o por license de npm; diferenciar paquete, WOFF y contornos SVG/JS.

## Matriz y errores preservados

NC01 PASS físico/tarball/inventario/selecciones. NC02 PASS separación de procedencia declarada, publicado y gitHead NO_VERIFICADO. NC03 PASS nombre/avisos/rawrecords propios. NC04 PASS extracción acotada independiente para name, no validación glyph integral. NC05 PASS límites: originales/glyphs/derivaciones/closurefonts/loader/runtime/offline/fallback/QA/distribución/legal exhaustivo pendientes. NC06 PASS preSHA/scripts/results propios y fuentes congeladas intactas.

Autor directories.py attempt1 falla contrato sstruct antes miembros; fuente/log/exit preservados y R2 selector string/sfntVersion separado. Own prepare attempt1 KeyError: supuso entries cuando segundo manifest usa files. Verifier-attempt1/log/exit/prehash conservados; cambio propio sólo selector de esquema no vacío y containment para rutas absolutas; prepare2 verifica22+228 antes parse. Ningún FAIL convertido en PASS ni criterio ajustado a texto del autor.

No instalación, font registration, reconstrucción de glifos, MathJax import/generador/engine, browser, rendering, cobertura matemática ni QA ejecutados. No cierre RV ni aprobación de artefacto distribuible. Antes distribución sigue necesario asociar cada activo elegido a originales/avisos/transformaciones aplicables; esta investigación conserva evidencia para ese trabajo futuro.
