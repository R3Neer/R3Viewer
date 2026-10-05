# Preparación técnica de gráficos — investigación, no implementación

2026-10-05. Trabajo paralelo al corte TOML; no cambia su alcance ni acredita imágenes visibles.

Paquete oficial inspeccionado: ratatui-image 11.1.0, MIT, crate SHA256 9593D10C61EC2007F429D1FAFC4F4A19FECDBAE4D06B531A2D505227FB82F5EA, VCS d8f08379465ed8e174c1d25cd60cbcfcbcbb39f1. Fuente local `.tmp/graphics-research/ratatui-image-11.1.0` (carpeta temporal no distribuida). Manifest compatible con Ratatui ^0.30.1/Crossterm0.29, pero falta compilar en conjunto: no fijar dependencia de producción todavía. Defaults incluyen chafa-dyn; elegir default-features=false y codecs explícitos para bundle nativo sin dependencia accidental. Crate estable publicado11.1.0; master12rc no autoridad de release.

## Hallazgos de API

- `Picker::from_query_stdio` escribe stdout y LEE stdin global; el viewer recibe documentos por stdin y teclado por CONIN$/dev/tty. No llamarlo directamente desde la sesión: consumiría transporte y competiría con el lector de eventos.
- `query_with_timeout` crea thread detached; Busy reinicia recv_timeout y la lectura stdin puede quedar bloqueada. No ofrece deadline total, cancelación, join ni devolución de teclas ajenas. Integrar detección en el propietario único del terminal, con cola de respuestas/teclas y límite total, o elegir override/fallback sin probe. El parser público cap_parser es reutilizable tras revisar su acotación.
- `font_size_fallback` Windows devuelve None. Incluso si hay hint WezTerm, sin tamaño de celda confirmado Picker vuelve a Halfblocks. Tener Sixel y tener geometría de píxel son condiciones diferentes. Necesitamos prueba real WT/WezTerm y override explícito de geometría, sin inventar font-size fiable.
- `detect_tmux_and_outer_protocol_from_env` ejecuta `tmux set -p allow-passthrough on` y espera hijo. Incluso `Picker::halfblocks` lo llama. Evitar estas rutas para no cambiar configuración del terminal como efecto de abrir un documento. Configurar protocolos mediante constructores propios o adaptación mínima revisada.
- `Sixel::new`, `Iterm2::new` producen data String y size; la codificación/rasterización va fuera UI y debe tener presupuesto y cancelación. Sixel.clear_area y iTerm2.doNotMoveCursor=1 son comportamiento TUI, no contrato scrollback inline. No imprimir su data ciegamente como renderer inline.
- `Protocol` e Image permiten render ya codificado; `StatefulImage` puede resize/encode al dibujar. Preferir resultados preparados para evitar CPU en UI. `ThreadProtocol` separa resize/encode, pero Sender no acota por sí mismo cola ni caché: coalescing/generaciones siguen responsabilidad nuestra.
- Kitty usa IDs aleatorios/virtual placement Unicode; borrado del placeholder no demuestra liberar bytes de imagen en el terminal. Gestión y limpieza por protocolo debe verificarse, no asumir equivalencia con Sixel/iTerm2.

## Siguiente gate concreto

Propuesta de RV201/202: diagnóstico explícito de capacidades; pequeño corpus PNG con transparencia y patrón de proporción; prueba de raster+codificación acotada y cancelación; TUI resize/move/retiro y renderer inline separados; redirección solo referencia textual. Verificar WT1.24 y WezTerm por host real. ConPTY puede comprobar bytes/lifecycle pero NO certificar píxeles. QA visual sigue pendiente por las limitaciones de herramientas actuales; no instalar ni cambiar perfiles de usuario.

Fuentes primarias: https://docs.rs/crate/ratatui-image/11.1.0 ; https://docs.rs/crate/ratatui-image/11.1.0/source/src/picker.rs ; https://docs.rs/crate/ratatui-image/11.1.0/source/src/protocol/sixel.rs ; https://docs.rs/crate/ratatui-image/11.1.0/source/src/protocol/iterm2.rs . Inspección local de fuente exacta, no ejecución de terminales ni aprobación del diseño.
