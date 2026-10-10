# S03: fronteras materiales de YAML y XML

2026-10-10. Aceptación focal M01–07 con [revisión y ejecución independientes](../reviews/S03-material-final.md), sin críticas pendientes. S03 y RV-106 siguen abiertos. Este cierre incorpora evidencia; los observadores privados no se integran en el producto.

## Resultado

| Control | Evidencia aceptada |
|---|---|
| M01: financiación e inventario | Capacidades reales y tamaños compilados de modelo, scratch, receptor, serialización y salida; reservas conservadoras separadas. No equivale a medir toda la RAM o RSS. |
| M02: arena de 64 MiB | Buffers reales tocados en la frontera exacta y sus vecinos; el preflight cobra capacidad antigua y nueva antes de crecer y conserva el contenido ante rechazo. |
| M03: estructura y fallback | YAML/XML pequeños y medianos Ready; grandes ArenaBudget. Cuatro salidas CLI conservan la fuente segura completa, comparada byte a byte. |
| M04: millón de items YAML | 999.999 y 1.000.000 Ready con aliases reales; 1.000.001 rechaza antes de añadir el item. Sin estructura parcial. |
| M05: XML | Los vecinos materiales 999.999/1.000.001 encuentran arena antes del límite de items. Exactamente un millón es imposible en un árbol XML válido de este modelo; el guard de Begin se verifica por separado. |
| M06: Job Windows | El worker original toca 128 MiB; el intento de comprometer 600 MiB recibe error 1455 bajo PROCESS_MEMORY de 512 MiB. El mismo control fuera del Job lo consigue. La denegación no demuestra autokill. |
| M07: fallo, cancelación y recogida | Fuente/Arc original, ausencia de Ready parcial, reap antes de retry y recuperación. Cuatro ConPTY q/Ctrl-C restauran modos y terminan físicamente con 0/130; 18 recibos comprueban wait, join y Job sin procesos activos. |

## Adjudicación del límite XML

El árbol válido incluye Document: con N nodos tiene N−1 relaciones, por lo que nodes+entries=2N−1 siempre es impar. No existe un árbol válido con exactamente 1.000.000 items. Los vecinos necesitan al menos 500.000 nodos; Node48+XmlMetadata132+Parent12 consumen como mínimo 96.000.000 bytes antes de relaciones, source, sesión y scratch, superando la arena de 67.108.864 bytes.

Se acepta esa precedencia y se registra el caso material exacto como no aplicable por invariante. El control de Begin comprueba los conteos con una lectura, sin crear un Ready ficticio. No se elevan cuotas ni se sustituye una prueba material por el tamaño de un DTO.

## Identidad y validación

Código productivo de base más el arreglo XML de 29993a0; feature privada de observación y artefactos separados. Arena64MiB, items1M, fuente128MiB, Job512MiB y plazos originales 5s inline/2s TUI permanecen intactos.

El autor registra 40 regresiones pertinentes existentes, fmt y Clippy all-targets con -D warnings. El revisor ejecuta 22 casos propios: 14 controles materiales, cuatro CLI byteexact y cuatro ConPTY; todos pasan. No recompila la biblioteca ni atribuye esos 40 tests a una ejecución propia. La raíz coteja los 2.389 archivos del handoff y los 697 de la entrega independiente.

Artefactos privados: `.tmp/s03-material-boundaries-2026-10-10` y `.tmp/s03-closure-adversary-2026-10-10/material-own-r1`. App SHA256 1BFCCC3A81DB22C25CA80C31904D2C8E1213E47E460DC8B9114B9F244B38271C; manifest autor 3B0A583CC57454B76B9B3641B06FB676C233308692679E306968F7487DCA67A4; manifest independiente 03BE2152C839D045D7C4FA148BD08014093D23FB354F92C1756739D45FC2D7C1; dictamen 9C3919C20C6CBC430D4A8139B5D020B81825E2182E7FDC24AD9E54E561186B5B.

## Historia y límites

Se conservan el primer oráculo incorrecto WorkerExited ante EOF antes de Begin (el contrato exige InvalidIpc), el Timeout debug XML y la configuración privada inicial inválida. Las correcciones de expectativas/configuración fueron prospectivas; el perfil release conserva los plazos originales y no explica retrospectivamente los fallos debug.

El observador duplica el mismo Job para comprobar Active0 tras el wait y join originales. Ese duplicado retrasa el efecto de KILL_ON_LAST_CLOSE: la evidencia acredita recogida directa y cierre comprobado, no autokill aislado ni cualquier descendiente imaginable. El ledger del pequeño roundtrip se sobrescribe entre formatos; no se atribuye su fichero residual a ambos.

Esta aceptación no acredita apariencia de terminal, runtime Unix, todos los faults de IO o los restantes criterios de S03.
