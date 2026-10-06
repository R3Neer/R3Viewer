# D3b R3 — revisión y aceptación independiente de integración inline

Dictamen: **SIN CRÍTICAS PENDIENTES en este corte de biblioteca inline revisado**. Resueltas las dos dependencias declaradas R2: proveedor real TextRef Unicode y roles compartidos. No aprobación de main/CLI/routing público, TUI/pump, renderer b2, Source syntax activation, gráficos ni composición shared final; esos caminos no se ejecutaron en este snapshot.

## Autoridad y método

Delta freeze13 MANIFEST BB808226AD795AA506E2D84059B8587B9D9D280EAF868260D54673C0D3F03093, source inline3A3BFE0304202B2520C7B88598E327D3E88F4F4FE59BB5B4B30C4701D9CC8BD3. Contexto ParentR1 MANIFEST613 78E4A9F719573B5646C66C89C1CD7D6241D1E44A94BC78D0FB03084F52162A74. Readback propio anterior/posterior de ambos sin discrepancias. Copia nueva Parent602 + dos fuentes nuevas, sustituciones sólo las cinco fuentes indicadas: inline/text/ref_tests/roles/lib. Factor tres fuentes son byteexactas al factor previamente aceptado; no se repite su batería ni se hereda toda aceptación baselineb1. R3 SourceEngine todavía se basa en snapshot anterior a b1R3 compartido; no se atribuye composición automática de esos deltas.

Ninguna función candidata modificada. Único sufijo cfgtest/harness propio; seis controles R2 retenidos, sólo ruta de config owned adaptada a la nueva copia. HARNESS-parent-r2.rs conserva original. Nueva expectativa propia EXPECTED-before-output.json 8A47E11EEA4E1046C58812DE30B196BBC20403DA0EE9E8A2A3B2D0683A96718B anterior a compilación y runtime. No usar outputs del autor como goldens, ni omitir ALT para Wiki. Artefactos R1/R2/rechazos originales intactos.

Build91433 propio offline/locked/escalado terminal0 55.07s, target nuevo. Dos grupos warning deadnav de dependencia sin consumidores b1 en baseline preservados; no claim Clippy limpio. Runtime sandboxWindows contra bin4F2824C65EBD52D6040A6D18977356C79284AB1D0E36BC44564905965E6ED334. TEST-attempt1 nueve grupos PASS0.20s. Sesiones activas0, sin helpers OS/rescates. Aviso Python ubicación y búsqueda inicial a ruta repo/src que no existía (delta es src plano) sólo diagnósticos herramienta; no producción ni oráculo cambiados por ellos.

## Controles propios

Seis controles anteriores repetidos por integración real que afecta el mismo walk/domain y color/count: ChildRunChildSourceboundary original, loosemarkers, Unicodebodycluster18001B, cuota/cancel/ordinalgutter, wholeclusterSGR y Sourceoriginalfallback. Todos PASS; Child sigue `- before\ncode\nafter\n`, marcadores no quedan en línea huérfana. No retornar String parcial cuando count/copy/fallback no cabe o cancel previo.

Tres nuevos grupos:

- Image0 de fuente `![ALT](asset "b`+C1+ESC+`&amp;👩‍💻")`: columns0 salida exacta `[Image pending: asset; alt: ALT; title: b\u{0085}␛&👩‍💻]` con LF. Exacto/-1 incluyen todos los bytes de proyección/decoración. Columns1 conserva cada cluster ZWJ entero, ningún ESC/C1 crudo; quitar sólo LF del output da idénticos bytes semánticos/decorativos al expected, sin pérdidas. Decoded entity aparece como &, sin interpretar origen como Source exacta inventada.
- Título RefID b+9000marcas (18001B) seguidoQ. Columns0 outputcompleto exacto/-1; columns1 con configAlways contiene cluster completo contiguo sin LF ni SGR dentro y conservaQ posterior. RoleSecondary RGBowned antes de glyph verificado. Cancel previo devuelveCancelled. Es emisión inline de TextRef completo: nunca WholeReference TUI ni Symbol16K truncado.
- Enums cerrados: Image0 emite Image pending e incluyeALT; Wiki4 de `![[asset.png|label]]` emite Wiki embed pending y ALTlabel explícito. No excluir ALT permitido para hacer pasar el test; expectativa nueva fijada antes de runtime. Estado pendiente no es gráfica ejecutada.

Nueve tests propios nuevos/retendidos en total, no los16del autor ni rerun factor4+roles. No dc04_32m/fullworkspace/backendRatatui/Unix/ConPTY/CLI.

## Revisión del código real

Leído inline productivo completo y delta frente R2; factor fuente comparado exactamente con AE previa. safe_reference ya no itera scalars manualmente: crea TextRefIndex real con Arc<Document>, id y el MISMO Consumer del walk. Índice/cursor viven dentro de la llamada, cargas retenidas hasta finish y se liberan después; no Run/Domain/TextRef ficticios ni copia plana de leaf. Precontext, controles y GraphemeCursor del motor aceptado proporcionan Units completas. Wrapping/tab8 usan actualcolumn/gutter y antes de emitir glyph; unit oversized conserva todo payload. visit_projected≤4096UTF8 se escribe por Output.push; no segmentador externo ni ANSI dentro de cluster. Caller sigue siendo responsable de count/capacity/completeOutput, no se usa semantic_text crudo como texto seguro.

count→tryreserveexact→capacity≤limit→copy de R2 permanece; controles SGR, etiquetas, ordinals y proyecciones cuentan ambas pasadas. Output.style/has_semantic/column/line/bytes se reinician para segunda pasada. No cambio de límites para hacer green; no justificar RSS/SLA ni carrera allocator por este test. Cancel checks por paso/copia/final y retorno sóloString completo permanecen. Fallback mantiene originalArc y no reabre ni construye índiceSource; helper numerado conservaCRLF/proyección/ordinal.

resource_descriptor y block_role ahora alimentan enum compartido real: Image/Mermaid/Math/WikiPending; input/ALT/title separados, mismos RefID para etiquetas y domains reales paraALT. block_role corresponde Root/Heading/Quote/Callout/ListItemTask/Cell/CodeMermaid/Rule/Footnote semánticos; no cambios a records parser/wire. Match Image0/Wiki4 evita reusar numeric meaning incorrecto. Ningún destino se abre o envía. Roles/style de impresión usan PresentationConfig existente, no caches gráficos ni Syntectfake.

El factor para proceder/préstamos/orígenes ya fue aceptado separadamente; este informe comprueba la integración y misma autoridad de Consumer por fuente/camino y contenido final, no inventa nuevo renderer. count y copy se ejecutan con mismo Document inmutable y config, no snapshots de output usados como proof. Datos bibliotecarios root siguen sin caller CLI ni selección pública; composición con b1R3/shared deberá conservar todas las APIs y verificar build/runtime afectado. No afirmar Sourcecodehighlight activado por esto.
