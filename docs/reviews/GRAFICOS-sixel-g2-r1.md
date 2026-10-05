# Revisión independiente SIXEL G2, propuesta R1

2026-10-05. Rol: adversario del diseño; no autor del encoder. Sólo lectura del candidato y contratos; sin implementación, ejecución Sixel ni emisión a terminal.

## Identidad y dictamen

Propuesta `SIXEL-G2-proposal-r1.md`, SHA256 `425DCC64310A927D852D1EA15AAE39ED43D1D67C6CA6E5E22F80B47C9580F31C`, comprobada al leer y al cerrar esta revisión. Contrastes: `D:/Tools/R3Viewer/docs/refinement/GRAFICOS-G2.md` y `GRAFICOS-PNG.md`; fuente DEC exacta `D:/Tools/R3Viewer/.tmp/sixel-research/vt3xx-chapter14.xhtml`, SHA256 `97BE785D3F9997E33F0F3EE7C2D8CF59B76F4E6B85EA82BBDDF9470E9689DE48`.

**SIN_CRITICAS_PENDIENTES, exclusivamente para este diseño de experimento privado.** No acredita código, payload real, IPC, integración, fidelidad, disponibilidad del backend, límites OS/agregados ni QA de terminal. No cierra G2 entero ni cambia S03.

## Contraste adversario

La propuesta preadmite `256×width` y coexistencia source/codec/raster/scratch/output antes de reservar. El contador previo admite la salida completa contra 32MiB antes de asignar; el segundo Writer está limitado y no publica resultados parciales. La cancelación invalida ambos pases y se conserva el plazo único heredado. La medición de requested heap256MiB y la contención Job512MiB siguen siendo propiedades distintas; no se usa una para certificar la otra.

La cota de bytes por sí sola no limita el efecto de `!Pn`: pocos caracteres pueden ordenar muchos avances. El diseño añade aritmética comprobada, cota acumulada de avance y pintura, límites de dígitos, geometría y comprobaciones por tramos. El validador procesa runs sin expandirlos ni materializar canvas. Los avances de máscaras cero también cuentan, y los bits inexistentes de la última banda se rechazan. Esto cubre la vía de amplificación propuesta sin atribuirle una cota de CPU/SLA aún no medida.

El manual DEC confirma caracteres SIXEL `?` a `~`, repeat decimal, retorno `$`, siguiente banda `-`, atributos raster y colores RGB por porcentaje. Ph/Pv declarados no recortan por sí solos los datos que llegan después: por eso son necesarias las comprobaciones de posición, avance y bits finales que la propuesta exige. Aspecto1:1, RGB y registros0..255 forman un perfil restringido; no se acepta la gramática universal ni HLS. [Fuente primaria DEC, capítulo14](https://vt100.net/docs/vt3xx-gp/chapter14.html).

La frontera de seguridad separa cuerpo ASCII de DCS/ST y comandos del padre. El parser independiente debe rechazar controles, parámetros incompletos o extra, selección no declarada, overflow y coordenadas fuera del descriptor antes de publicar el resultado. Validar sintaxis y geometría no demuestra identidad de píxeles; el oracle separado que decodifica el cuerpo y compara resultados cuantizados es un gate diferente y necesario. Count/Emit pueden compartir el encoder genérico; el validador y el oracle no pueden compartir su interpretación defectuosa como única evidencia.

RGB332 sin dither es una hipótesis explícita. Componer alpha sobre mate conocido antes de cuantizar evita convertir transparencia en colores arbitrarios; mate ausente y alpha parcial conservan referencia. El artefacto de implementación tendrá que fijar la fórmula numérica de cuantización, redondeo de componentes porcentuales y composición antes de generar expectativas. La propuesta exige esa definición, pero no ha aprobado calidad ni un algoritmo concreto de filtrado.

Palette state, registros privados, scrolling y cursor tienen efectos dependientes del terminal. La propuesta conserva estos gates y no cambia modos desde el encoder. Una decodificación independiente del cuerpo no acredita esos efectos. [Fuente primaria Xterm, control sequences](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Sixel-Graphics).

## Evidencia exigida por el propio diseño, todavía pendiente

El candidato debe congelar unidades y máximos de contadores, límites de dígitos y fórmula RGB332; distinguir longitud del cuerpo y presupuesto total reservado para framing; registrar capacidades y coexistencias reales. Son concreciones del gate antes de reservas/oráculos, no permiso para postchecks ni para rebajar cuotas.

Debe acreditar corpus 1..9 y últimas bandas, paleta/alpha/mates, casos originales PNG R2/crop/fit/Adam7, payload32MiB exacto/+1, panoramas y viewport ruidoso, Count/Emit iguales, hostiles de gramática/geometría/trabajo y cancelación durante ambos pases. Las expectativas de píxeles deben calcularse fuera del candidato; el validador debe ser ejercitado también con cuerpos hostiles fabricados independientemente. Revisor y AE distintos del autor, manifest exacto y hashes antes/después.

El Job propio anterior a fuente/encoder, kill/reap/lectores y plazo requieren evidencia del ejecutable congelado. Descriptor/generación en memoria siguen simulación. IPC real, snapshot mutante, caché, agregado padre+hijo, Unix, terminal/inline y QA visual permanecen abiertos. No se ejecutó ninguno de esos gates en esta revisión de diseño.
