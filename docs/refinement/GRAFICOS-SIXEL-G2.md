# G2: siguiente experimento Sixel R1

Diseño de encoder/validador privado revisado independientemente sin críticas pendientes. No sprint nuevo, adopción ni terminal emission. Mantiene diseño GRAFICOS-G2.md y contrato GRAFICOS-PNG.md; raster PNG por filas R2 ya tiene revisión y AE acotadas. IPC real/cache/agregado/terminal/inline/Unix/visual permanecen abiertos.

## Corte

Derivar candidato privado del raster R2 congelado, preservándolo intacto, para cuerpo Sixel de un viewport RGBA8 validado. Sin Picker/threads/stdin probe/tmux/config global/quantizer externo. Usar el mismo allocator/Source64MiB/origen16M/visible/budget32MiB y cancel/plazo; no cambiar cuotas para pasar. Ningún cuerpo ni framing se imprime al terminal: archivos de evidencia y oracle propio.

Color fijo RGB332 sin dither como hipótesis de viabilidad, no calidad visual aprobada. R/G se aproximan a ocho niveles y B a cuatro, redondeo definido y paleta declarada en porcentajes RGB del protocolo. Probar error cuantizado frente al color de origen por fórmula; no afirmar fidelidad fotográfica. Si alpha no es255, componer sobre mate RGB conocido antes de cuantizar; alpha parcial con mate desconocido deja reference. RGBA premultiplicado no se interpreta como straight sin contrato.

Sixel necesita backend/celda/lifecycle conocidos para producto futuro. Este gate no fija detección ni afirma disponibilidad de256 registros en terminales antiguos; perfil RGB332 requiere ese número. Palette state/private registers y restore se acreditan en gate de terminal, nunca se cambia modo global dentro del encoder.

## Encoder acotado

Bandas de seis filas, planos por color realmente usado. Scratch de máscaras por color×ancho (máximo256×width bytes) y arrays pequeños de uso/última posición; preadmitir multiplicación/peak antes de reservar. Dimensión panorámica que excede memoria se diagnostica antes de allocate, sin raster natural completo ni cuota origen menor.

Pintar bits de cada pixel en su plano; filas inexistentes de última banda no se encienden. Emitir sólo hasta último sixel no cero de ese color, retorno de carro entre planos y avance de seis filas sólo entre bandas reales. RLE de caracteres repetidos cuando disminuya la longitud; contador decimal fijo, sin format/String no acotados. Paleta/atributos/planes se producen mediante una única rutina genérica Writer.

Primer pase CountWriter cuenta bytes exactos (incluido overhead futuro DCS/ST reservado), sin cuerpo asignado. Cap32MiB antes de reservar; segundo pase usa Vec try_reserve_exact(count) y Writer con límite que no crece fuera de cuota. Comparar conteo/emisión; error/cancel descarta todo resultado. Coexistencia source/codec/raster/scratch/output se registra y sigue hard requested heap256MiB; no se afirma memoria OS ni agregado padre/cache.

Además de bytes, contar trabajo lógico de avance: sum(run_length) de comandos sixel y bits pintados multiplicados por repeticiones. Proponer para experimento un máximo64Mi avances horizontales y16M pixels pintados, con checks por tramos; evita admitir millones de repeats mediante muy pocos bytes. Son candidatos medibles del encoder, no SLA ni límite final de soporte del formato. Definir unidad exacta en reporte y comparar Count/Emit. Mantener source16M y fallback explícito si su viewport no cabe en recursos, igual que otros emisores.

## Frontera de cuerpo seguro

Worker sólo entrega cuerpo ASCII del perfil, sin ESC/C0/C1 ni DCS/ST/control de cursor. Padre futuro añade framing/IDs y controla modo/cursor; no confiar en String arbitrario del codec. Perfil canónico: atributos raster iniciales con aspect1:1/width/height esperados; definiciones RGB de registros0..255 con componentes0..100; selección de color declarado; sixel '?'..'~', repeat decimal positivo, '$' y '-'. No comandos desconocidos, HLS, números gigantes, prefijos vacíos ni parámetros extra.

Validador streaming independiente del encoder interpreta cada comando con números checked, límites de bytes/dígitos/trabajo, posiciónx/banda y registros definidos. Width/height coinciden con descriptor esperado, avance no supera width, bandas dentro height, máscara última banda sin bits superiores. '$' reiniciax, '-' sólo avanza una banda legal y reiniciax; cero bits no permite avance ilimitado. Pintura acumulada<=16M, avances<=64Mi. No materializa canvas para validación; cobertura/color exactos se comprueban con un oracle independiente en corpus acotado.

## Gates de autor y revisión

1. Corpus previo de1..9 dimensiones y última banda parcial; RGB extremos/grises/gradientes, alpha0/128/255 con mates dark/light y mate ausente; crop/fit/Adam7 de PNG original R2. Oracle decodifica cuerpo por separado y compara pixels cuantizados y dimensiones, no hash obtenido del candidato.
2. Payload exact32MiB/+1 y peak old+new; panorama/large noisy16M→viewport; dos pases cuentan/emiten igual y no publican prefijo. Ningún RLE permite rebasar cuota de trabajo o metadata.
3. Hostiles: ESC/OSC52/C1, atributo/dimensiones/paleta/color/run inválidos, enterooverflow, run0, cursor fuera width, última máscara alta, bandas extra y repeats que caben en bytes pero exceden trabajo. Rechazar antes de cualquier emisión/publicación.
4. Cancel/checkpoints y JobWindows propio antes de fuente/codificación, kill/reap/lectores; descriptor/generación en memoria sigue sólo simulación hasta IPC real. Revisor y AE diferentes del autor, exactmanifest, sin QA visual ni G2 entero.

## Referencias inspeccionadas

[Manual DEC VT330/VT340, cap14](https://vt100.net/docs/vt3xx-gp/chapter14.html), copia XHTML privada SHA97BE785D3F9997E33F0F3EE7C2D8CF59B76F4E6B85EA82BBDDF9470E9689DE48: bandas verticales y parámetros/colores/repeat. [Xterm control sequences](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Sixel-Graphics): DCS del formato y modos de palette/scrolling dependen del terminal. Consultadas2026-10-05; el gate no usa modos ni atribuye sus efectos a pruebas de cuerpo.

## Estado

[Revisión R1](../reviews/GRAFICOS-sixel-g2-r1.md) aprueba el diseño del experimento; no código ni gate ejecutado. Propuesta temporal exacta SHA256 425DCC64310A927D852D1EA15AAE39ED43D1D67C6CA6E5E22F80B47C9580F31C. Esta copia añade estado final. G2 y las capacidades de producto siguen abiertas.
