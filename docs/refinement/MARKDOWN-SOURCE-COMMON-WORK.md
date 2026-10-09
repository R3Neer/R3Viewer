# SC-W01 — calibración prospectiva SOLO Source Common

Fecha2026-10-10. Diseño mínimo, pendiente revisión independiente ANTES cambiar producción. BaseSource R1 preservada; R2 derivado separadamente de3db. Root autorizó API aditiva metered en text_engine.rs; métodos Legacy siguen intactos. Producción Common todavíaWork256KiB; TEST-source-common-r2-attempt2 conserva dense120x40 FAIL. Diagnóstico cfg(test) previo512KiB ejecuta MISMO prepare/factory/authority/model/Scope real: TEST-SC-W01-diagnostic512K-attempt2 informa Work300398, Frame123712. No Source clone/hash rebuild/allowance renovado.

## Coste medido por admisiones efectivas, sin costes nuevos para pasar

Dense120x40,4800glyphs. Las primeras cinco filas suman el coste histórico241111; resto añade59287 de aperturas/metadata realmente cobrada. No se elimina ningún cargo.

|Componente|Work|
|---|---:|
|Input descriptor físico|200|
|Glyph packed24 + symbol1,4800 escrituras|120000|
|Copias efectivas oldSymbols al crecer16→8192|8176|
|Mecánica Unicode/proyección histórica restante|112735|
|Row range header3usize +hasta2bytes guard26×40|1040|
|Index headers efectivos:validate16×40,2vec.len8×40,exceptions.len8×4801|39688|
|Index Point probes:608bytes de campos line/column +111comparaciones|719|
|Point clone144×40|5760|
|Point fallback39:constructor144 +range metadata16|6240|
|Provenance headers3usize×40|960|
|Checkpoint tuple2usize×80probes|1280|
|Checkpoint comparaciones80|80|
|Checkpoint recordfinal2usize×40|640|
|ProvenanceCursor constructed72×40|2880|
|TOTAL|300398|

Counters cfg(test) del worker real corroboran Index headers39688/fields+cmp719/clones5760/constructions6240/exceptionclones0. Frame no cambia. Headers/copies se miden por valores/tipos/condiciones conocidas admitidos; guards admiten el peor caso local declarado. No CPU exacta, TLS-test RAM, RSS ni internos de canal/thread/allocator acreditados. Point probe NO cobra Point completo: campos prestados8B por lectura y comparación1; sólo clone efectivo cobra sizeofPoint144. Source checkpoint son tuplas16B leídas/copias locales y comparación1. Tests extra permiten remaining−1/cancel tipados sin resultado parcial ni reset.

## Alternativas

A. Mantener256K y entregar Limit para dense120. Rechazada: contradice cobertura funcional previa y no arregla el producto solicitado.
B. Reducir importes, omitir metadata/lookup o volver a Legacy lookup. Rechazada: incumple SC-W01 y SAMEWork.
C. Nuevo cursor Common monotónico de excepciones/cache de row entries. Puede evitar búsquedas/lecturas repetidas, pero requiere otra representación retained y pruebas de identidad/transiciones/exception/no-op. No debe simplemente borrar cargos. No existe aún y no se necesita para cerrar la biblioteca; no empezar esa arquitectura ahora.
D. Política explícita SourceCommonWork512KiB, manteniendo todo algoritmo/cargo. Propuesta: suficiente para la fixture medida con223890 de margen; Workhardlimit sigue finito, SourceFrame256KiB/Consumer1MiB/Actor64MiB y límites símbolos/modelo no cambian. No recalibrar Legacy ni DocumentWork8MiB.

ElegimosD para revisión: un constructor/política privada SOLO Common define `SOURCE_COMMON_WORK=512*1024`; `source_view_preparation::prepare` crea FrameBudget con ese Work mediante Work::permit y conserva Frame256K. `FrameBudget::new`, Work::frame y TODOS métodos Legacy existentes quedan byteexactos; no booleano/cuota recibida del caller, ningún doble allowance. Input/Output y metadata layout no cambian. Common Prepared workused sigue cerrado y readonly.

Cancel se comprueba al abrir y antes de cada admisión/probe/clone, además mecanismos Unicode existentes. Cleanup máximo2s y Pending global unchanged; incremento de Work no renueva ni alarga deadline, no rescans síncronos UI. Mediciones de tests no acreditan latencia ConPTY real. Un límite larger admite más unidades, no promete tiempo total; mantener cuotas y fixture/barreras cancel existentes. Revisión AE deberá verificar agotamiento con perfil exact−1/exact, typedLimit/current preserved y cancel.

Esta calibración se refiere a la fase de preparación SourceViewport que implementa esta biblioteca. No promete que512KiB cubra renderer NativeSource +publication/pump futuros; D2c deberá componer receipt/retained spend bajo operación única y admitir coste completo, sin permit nuevo/refund. DocumentWork8MiB es otra política real de layout+drawMarkdown y no se usa para financiar Source por el lado.

## Oráculos suplementarios PREVIOS

`EXPECTED-SC-W01-common512K-before-output-r2.json` se fija antes cambiar el constructor y antes output bajo nueva política. Sustituye explícitamente sólo el límite Common de expectativas históricas: dense120 produce4800glyphs40rows con coste300398, Frame123712 y SAMEModel; dense80 sigue1920glyphs24rows coste129078, Frame48448. SourceLegacy256K/Frame256K no cambia. Old256KFAIL y diagnóstico permanecen evidencia histórica, no se reetiquetan PASS. Original SourceCommon pruebas mantienen texto/rangos/keys/rollback/lastDrop; se cambia únicamente su lectura del límite activo si revisión aprueba. Fuente de límite será constante Common cerrada, no valores mutables del caller. Tests nuevos de exact−1/exact del Work completo preparador usan permiso acotado con factura real, nunca objetos fake ni permit renovado. Publicación/current sigue fuera de UI y sin terminal output acreditado.
