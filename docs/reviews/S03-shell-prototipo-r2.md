# S03 shell prototype R2 — revisión independiente

2026-10-05. **SIN_CRITICAS_PENDIENTES** en el prototipo privado y perfil léxico revisado; SHP-01/SHP-02 cerrados. No adopción en producción, equivalencia universal TextMate, aceptación de worker real, QA visual ni runtime Unix.

## Candidato exacto

Cviz/shell-grammar-prototype-r2-final, MANIFEST SHA256 `C6BFD9C471E4635720147EF0A25F927D238F3E0CE9A5D5EC2BF5FFBE48CB1418`,230entradas. Entrega `C78D30211DB768AE35C7D894B912B851A5357B6326A94F3AE08F80023FDFDF4D`. Todas las entradas/bytes/hash verificadas antes/después de ejecución propia; `shell-review-r2-owned/hashes-before.json` y hashes-after.json conservan identidad. Snapshot y R1 no editados. No run_matrix sobre snapshot porque escribe evidencia adentro; todos los outputs nuevos están fuera.

Nu recurso `1A0972876A8EE4BF1C3E97E12B9B71FDF82F42F87685062620484818B3D577AF`, derivador `58DC1DBE8984CE2D0F751DF4297C61167286525763B5BB1380EE0D57415A8DD5`; dump ejecutado `301A65C46F0B86764BBF557674616EA899CBE67C402E48751EB3AEA6B7B62C51`. Bins originales fancy `33DEB3817DAFE2BC3D1684A2CA519EA291CFF9FB1C647F67A61F578C9FCC0376` y Onig `6E1A8EAF6CB425A29A066540522B9A68C601A7F2CE4C8AF0B1D3418C8F3941A6`. PS/TOML/AST/generador general/Rust/lock/bins conservan identidad R1; no ejecución Nu/PS ni carga usuario.

## SHP-01/SHP-02 cerrados

N05/N06 ahora gatean owner original completo antes de crear entry. String sin colon sigue su propia regla: el brace cierra y siguiente echo tiene scopes nuevos sin record-entry. Dentro del owner, apertura conserva capture1, embed fija dos capas variables para capture2, escape literal por sufijo original corta estados anidados en b y asigna una capa variable a quote terminal; colon/valor y entry original continúan. No se añade string-escape scope que capture2 original no incluía, ni retokenizer/overlay extra de runtime.

La sustitución de suffix backref es escape propio de Syntect, no backref como permiso de consumir cualquier región posterior. Se revisaron gate/remaps/variables/priority/native escape propagation y convivencia con N04/command/declaration. El prefijo validado impide activar entry en falsos owners; la región retokenizada termina en el sufijo capturado incluso con paren incompleto. Los casos concretos demuestran el cierre, no rendimiento/cobertura de todas las entradas.

Oracle adversario fuente-only congelado antes de ver R2, SHA `65B12B89A6BEB863EA518BEB877FDC7BC8DB6A5B215A63ED4FD08924E90E442A`: ocho fixtures (seis reproducciones exactas+dos UTF8),60intervalos completos y16endStacks. Reejecución propia de **ambos motores:60/60 y16/16 PASS**. Se comprobó expectedEndStacks explícitamente; no se confió en el verifier anterior que lo ignora. Base independiente161/161 y addendum anterior48/48 también PASS por motor. Total84documentos reconstruidos byte-exact y269assertions seleccionadas por motor.

Además, ambos binarios procesaron directamente los recursos nuevos para esos ocho fixtures, y el JSON completo es idéntico a run-dump sobre el pack congelado. Esto acredita relación resource→pack en ese corpus, sin exigir reproducibilidad de bytes de pack entre procesos/plataformas. Popen propios, stdin cerrado,10s y kill/wait5s sólo si sigue vivo; todos completaron0, waited/readers cerrados. Evidencia y comandos en `shell-review-r2-owned/RUNS.json`, `RESULTS.json` y outputs por backend/corpus. **SHP-01/SHP-02 cerrados** sobre exacto candidato.

## Exploración adicional y límites

Diez documentos propios más sobre empty double/single keys, single paren incompleto, suffix con metacaracteres/backslash, colon interior en ambas comillas, CRLF, EOF, dos keys en misma línea y string incompleta sin colon. Expectativas nuevas se fijaron desde owners/captureScopes antes de sus outputs.28assertions exactas adicionales PASS en ambos motores; string incompleta verifica retorno acotado, sin exigir recuperación que la fuente original no promete. Outputs `extra-*.stdout`, oracle `extra-source-only-input.json`, resultados `EXTRA-RESULTS.json`.

Primer intento de una fixture de sufijo tenía string externa sin cierre (quote terminal escapado), por lo que su expectativa de echo siguiente fuera de string era incorrecta. Se conservan script/input/output attempt1. Se corrigió únicamente esa fixture para poner cierre real después de backslash+x, a partir de regla string-double-quote original; no se modificaron gramáticas, oracle adversario congelado, corpus AE ni expectativas para ocultar defecto de candidato.

Inventario R2 distingue292campos explícitos match/escape y29variables del conteo de objetos Regex compilados; no repite cifra299R1 como afirmación falsa R2. Ledger/origen/remaps, MIT y separación de procesos/features preservados. No nuevas críticas accionables tras este delta y exploración acotada. Los oracles siguen seleccionados; revisión no certifica toda gramática, toda regex/backref posible ni fullgolden de cada documento.

Antes de adopción quedan AE independiente y diseño de integración revisado, worker real/límites/line64KiB+1/deadlines/cancel/reap/fallback Fuente/SourceSafeText/viewport, I/O owned, catálogo/aliases efectivos, corpus hostil, paleta/QA visual y runtime Unix. Checkpoint clone inmediato de cada línea no sustituye checkpoint/viewport real del producto. Ningún RV/S03 se cierra globalmente. **SIN_CRITICAS_PENDIENTES**, sólo prototipo privado R2 y gates léxicos aquí enumerados.
