# W3: correspondencia completa de salida inline larga

2026-10-10. [Revisión final independiente](../reviews/S03-W3-final.md): SIN_CRITICAS_PENDIENTES focal. Un pipeline Nu real declara texto con `open --raw fixture.json | into string | viewer --inline --structure --format json --colour never`. No hay cambio productivo; input declarado String conserva metadata y stream, no reinterpretación automática de Unknown/Binary.

Fixture propio10000strings únicos de7bytes, primera cadena escapada;100007bytes fuente y70000bytes totales de contenido UTF8 decodificado. Esta última cifra es contenido semántico, no memoria/capacidad del arena cooked. Expected prospectivo298899bytes/10002líneas cortas sinwrap cruza tramos64KiB y batches2048. El oráculo compara cuerpo completo, orden, tipos, índices, comas, espacios y LF; no sólo tokens/counts. La hoja individual grande y memoria se comprueban en W1/W2, no se inventa un nuevo gate de wrapping aquí.

Autor y adversario ejecutan cada uno una redirección y una ConPTY. Ambos fijan su GOLDEN real antes de terminal y contrastan después contra expected prospectivo. Resultado propio: redirect y terminal exit0; GOLDEN/decoded298899bytes exactos, raw309023bytes bajo1MiB. Decoder acepta whitelist cerrada antes de strip, conserva espacios/LF y rechaza relocation/unknown; no simulación visual ni normalización arbitraria de whitespace.

Nu propio15008 redirect y45320 terminal terminan0; Jobs internos baseline2→2/4→4. Wait/EOF/joins/cierres físicos/HPCON y JobActive0 externos, sin fuerza, cleanup_only o Pending.19prehases y18identidades posteriores intactos. FAIL inicial preservado: `open --raw` sin declaración textual entregaba un tipo no aceptado por --format json, sin salida/golden/terminal. R2 añadió exclusivamente `into string` tras adjudicación prospectiva; no defecto de truncación ni cambio de cuotas/oráculo.

Raíz verificó45/45 del final propio C2DB2A05C2B91287A9A9392E3889158F1B105754E39F3C2C8253220F79BD1B67; informe0CADDAA4454AE058C8A6DE9CD8CF20368C7E778D11C1D0E08AC8849EC288E6BE. Freeze autor91entradas47832B8AECC91460524729F109CEDF27B59CF3E562A174FD1BEAF72485D9B01B. Binarios normales baseline29993a0 y módulo anterior separados de C01; reproducción de copias exactas sin build propio. Cotejo/smoke del build canónico combinado pendiente y se registra aparte.

La prueba acredita correspondencia textual bajo esas condiciones, no apariencia, fuentes/DPI, wraps, toda salida Unicode o Unix. No cierre S03/RV ni adopción de arneses.
