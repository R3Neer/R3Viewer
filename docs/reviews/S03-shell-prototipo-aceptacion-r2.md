# AE independiente del prototipo shell R2 final

RESULTADO: SIN_CRITICAS_PENDIENTES en el corpus y viabilidad acotados ejecutados. No aprobación visual, soporte universal de shell, integración de worker ni cierre S03. Autor yaml_remaining_lifecycle; AE graphics_gate_experiment.

## Corte exacto

Único candidato ejecutado: Cviz/shell-grammar-prototype-r2-final. Manifest SHA C6BFD9C471E4635720147EF0A25F927D238F3E0CE9A5D5EC2BF5FFBE48CB1418:230 entradas/tamaños/SHA verificadas antes/después. Entrega SHA C78D30211DB768AE35C7D894B912B851A5357B6326A94F3AE08F80023FDFDF4D verificada. Bin/resource fijados por manifest; copia preparatoria no ejecutada. R1 original153 entradas/SHA5A2EA552FEA81E9C59271F16F968719251AC4EF3BDF6B1F8254495556E368FC9 nuevamente comprobadas íntegras; evidencia/oracles anteriores preservados. Ninguna fuente/producción/candidato/oracle editados.

## Ejecución AE propia

BaseR2 161/161 intervalos por motor57docs531spans; addendumR3 48/48 por motor19docs221spans; addendum fuente-only del revisor SHA65B12B89A6BEB863EA518BEB877FDC7BC8DB6A5B215A63ED4FD08924E90E442A:60/60 intervalos y16/16 expectedEndStacks por motor8docs113spans. Total269 intervalos exactos y16 estados prescritos por motor/84docs865spans. Todos los outputs fixtures/spans/stacks y endStacks completos idénticos entre fancy/Onig, incluso fuera de intervalos seleccionados; esa igualdad no prueba corrección de los scopes fuera del oracle.

Verifier propio preparado antes de observar R2 SHA00C13CB5C67334F7EF89592852F3B253D71406B893F666373670FDCC61B26F07 usado sin cambios. Comprueba expectedEndStacks ordenados realmente:16PASS; control sintético previo comprobó que un estado final contaminado falla aunque todos los intervalos pasen. Además: IDs exactos sin duplicados, bytes/text_sha/token/UTF8boundary, continuidad y root source único, número y orden líneas, checkpointReplay real del binario con assert operaciones idénticas en cloned ParseState. Oracles fijos161+48+60 desde fuentes originales, sin expectativas nuevas generadas desde candidato.

Tres matrices AE con eager load Nu/PS/TOML+defaults ambos motores, cero refs pendientes, dump fancy propio consumido por ambos. Quince procesos matrix exit0 sin timeout, logs de PID propio/command/stdout/stderr/hashes/tiempo; watchdog10s sólo Popen retenido/kill/wait, no prueba deadline del producto. Los3dumps SHA301A65C46F0B86764BBF557674616EA899CBE67C402E48751EB3AEA6B7B62C51 idénticos y coherentes con entrega. Catálogo propio ejecutado ambos motores:78 nombres únicos/78scopes únicos y outputs idénticos. No se equipara número de expresiones explícitas del inventario con regex internos.

## Evidencia y reproducción

Cviz/shell-grammar-ae-r2 contiene verify.py, run_matrix.py adaptado sólo para fijar candidate-root r2-final/prototype-r2 y salida AE, base/addendum/review con stdout/stderr/RUNS/dump/verify, catalog y AE-RESULTS. AE-MANIFEST SHA 5A67C96F7710572F8442074072DCC2CAEFDF6F5349F57D833E3516D21E3C58A0. Repetir C:\Python314\python.exe <AE/run_matrix.py> <oracle> <tag nuevo>; después <AE/verify.py> <oracle> <AE/tag/run-backend.stdout> <resultado nuevo>. Verifier exit1 si intervalo/endStack falla; otros invariantes fallan por assert. rootR2/resources/bin siempre snapshot final explícito.

## Fronteras

Esta aceptación ejecutada cierra dentro del corpus regresiones N05/N06 no-colon/escapedquote/paren-capture/UTF8 y ausencia de contaminación en echo/endStack, además de209intervalos anteriores. El adversario independiente debe evaluar todavía otros predicados/diseño que su revisión considere; AE no reemplaza esa revisión. Sin Nu/PS ejecución ni módulos fuente, sin constructor de UI/renderer/terminal visual.

No worker real ni Job/memoria/cancel/reap/hostiles/performance/deadline/ownedI/O de producto, viewport/SourceSafeText/índice, selector/aliases/wrapperNu actual, paleta/QA visual o ejecución/buildsUnix/distribución. Clone de parser por línea no equivale a checkpoints generacionales/paginados del producto. Los16expectedEndStacks están prescritos sólo en8docs, no oracle general de todos los finales. Corpus parcial frente a toda TextMate; backrefs runtime acreditadas por casos concretos, no universalidad. Ningún requisito rebajado para dar verde.
