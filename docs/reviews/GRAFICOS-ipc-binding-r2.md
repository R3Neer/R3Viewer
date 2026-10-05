# Revisión adversaria del binding IPC R2

2026-10-05. **SIN_CRITICAS_PENDIENTES**, acotado a diseño del gate privado. No implementación, runtime, ledger medido, terminal/visual/Unix ni cierre G2.

Binding R2 exacto SHA256 `1EF4852EB3D1A1FD97475DE669BD00B7E7E3C6E3DAC7A19349B38040280924C9`, verificado antes/después. R1 intacto `D8113721F51282E7F7D714ECDBB0C278DB546B6A4F251191FF9C195A6D1D2C2C`. Base propuesta R3 intacta `5438FBA7523650C082EA33C5B7D296F8191B4E2A26050BDB6951EF5DA6BE2B3E`. Dictamen previo binding R1 `35B965F17D6CFD991F8592AAC1BBFD37ADF5D780116EC1EBC2BF7505EC1525F9` conserva críticas e identidad.

## Resolución de críticas

**GIB-01 cerrada.** R2 selecciona normativamente wire sin SourceACK; C ya no espera handshake de recepción fuente ni hay control extra antes de R3GIRES1. ACK H↔C del handleD sigue separado. C termina bytes/End/close sin error, une writer y suelta TODOS los owners Arcsource antes de lectura/admisión/asignación de body. D valida sourceSHA/longitud/End/EOF internamente y puede escribir respuesta bloqueándose temporalmente en stdoutpipe. C no espera exitD para abrir reader: elimina el ciclo potencial. Constructor/start parcial, writererror/cancel, respuesta inmediata y backpressure deben probar la barrera sin reservas/capturas fuera de cuota; no hay relajación de receiptsIO ni único commit.

**GIB-02 cerrada.** Alpha0 ahora preserva RGBA straight sin composición, matte0/0/0 canónico; Kitty/iTerm conservan alpha. Alpha1 usa fórmula entera explícita por canal y alpha255 para todos los backends. Sixel alpha0 exige alpha255 de TODOS los píxeles target seleccionados por crop/nearest, con Reference sin cuerpo ante otro alpha; transparencia no seleccionada fuera del crop no cambia el resultado. Identidad del descriptor ya no permite interpretar «opaco» como borrado de alpha. Controles de alpha0/128/255 y RGB/matte distintos permanecen necesarios contra oracles anteriores del raster.

**GIB-03 cerrada.** Num/den son u32 positivos reducidos; ratio≤8 se comprueba con u64(num)≤8*u64(den), sin limitar ninguno a8. Los máximos u32 caben en ese producto u64; floor de dimensión por numerador también debe usar operación ampliada/checked como exige el contrato general. Fit1/4096 queda expresamente válido. Dimensiones target exactas, gcd, denominador0, ratio8/+1 y redundancia siguen gates separados de transporte/hash.

## Coherencia conservada

Descriptor108, Request116, Responseprefix117, Payloadheader125, End40/48 y Reference167 permanecen. Sin nuevos bytes de control en stdoutD; status/enum/código cerrados, EOF estricto y End/hash no bastan para Ready sin lifecycle aprobado. Framing4096 conservador no reemplaza el límite Sixelbody+5 previo ni acredita terminal.

Partición H40+C80+D136=256MiB requestedRust continúa coherente con picos de fase y las barreras C/source y D/decoder/source antes de encode. Arc<Vec> sin make_mut/copia evita transición fullbytes; old+new, realloc y startup se cargan antes de allocator. Contadores Rust no sustituyen memoria OS/Job ni garantizan recuperabilidad de OOM no fallible. H inflater≤8MiB y buffers exactos de dependencias deben acreditarse con inventario/trazas y fallos, no sólo suma. Un Reference por cuota deja su cobertura funcional pendiente.

Cap visibleRGBA24MiB es explícitamente experimental; no baja producto64MiB ni cierra16M/RGBA64 completo. Perfil/hash/lock/build y fuentes derivadas codec siguen pendientes de congelación. Cuerpo IPC sólo cuarentena; validadores independientes, receiptsProcess/IO, Jobmiembros0/handlesclosed y commit H serializado con cancel/generación conservan R3.

Siguiente gate: implementación privada conforme R3+bindingR2, inventario/ledger y source/bin/manifest congelados, revisión de código y AE distintas. Este cierre documental no convierte Sixel/PDF/graphemes u otros gates en aprobados.
