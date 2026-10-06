# D2b0 R2: adopción de código

Preparación HASH/SOURCE y autoridad tipada incorporadas tras aceptación independiente. El código conserva el original compartido, sólo publica resultados después del cierre del hilo y bloquea nuevas preparaciones mientras queda un propietario pendiente.

## Identidad y aceptación

Freeze A1D17B65686630EA352A737914A79212CC1E6E7DA17B65B201B8C9BFBC1C66CD, 602 registros; contexto 558 archivos. Informe independiente 3F245B01D8CF779E85E819829C16FDA570B05A3383F49A4E35722262A48A6A9A y manifiesto de evidencia 06EDBC3DF34B560A3CEBD1D4F0BBF490EF21546976DE7661219E454B0203B892, 23 registros, verificados sin discrepancias. Doce grupos independientes pasan con expectativas anteriores a las salidas: continuidad real HASH a SOURCE, identidades, cierre y candidato tardío, errores y panic, canales llenos/desconectados, Pending local/global, fallo de spawn inyectado, Document validado y límites de líneas. R1 continúa rechazado y sus evidencias permanecen intactas.

Las seis fuentes se contrastaron con el freeze, permitiendo únicamente normalización CRLF a LF de Rust conforme a atributos existentes. No se incorporan harnesses privados, binarios, SDK ni corpus universitario. El getter Document::source_handle conserva exactamente la fuente validada; los nuevos módulos siguen privados hasta su integración de sesión.

## Combinación compartida

Sesión raíz 32824 terminó 0: 19 pruebas preparation_owner, 7 presentation y 14 syntax pasan; tres pruebas nativas de syntax quedan explícitamente ignored en esta ejecución. fmt y Clippy workspace/alltargets terminan 0. Se conserva la advertencia upstream proc-macro-error2 sobre compatibilidad futura. Logs independientes por suite Markdown-D2b0-shared-*-r2.log en el directorio de evidencia del chat.

La aceptación acredita biblioteca y ownership; no acredita Loading, teclado, ConPTY, restauración, layout b1, renderer, Markdown visible ni runtimes Unix. Las capacidades medidas no equivalen a RSS; no se repiten matrices gigantes ajenas al delta. D2b1 y la integración D2c/D3c siguen pendientes. No publicación, perfiles, instalación ni cierre RV-301.
