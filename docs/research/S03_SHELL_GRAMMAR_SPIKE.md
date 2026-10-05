# Entrega del prototipo privado de gramáticas shell R2

Candidato congelado para revisión adversaria y aceptación independientes; todavía no adoptado en R3Viewer. Se corrigen SHP01 y SHP02 en N05/N06 sin modificar producción, Cargo raíz, PowerShell, TOML, runner, lock ni binarios.

## Identidad

Snapshot final: `shell-grammar-prototype-r2-final/` junto a este informe. MANIFEST.json SHA256 **C6BFD9C471E4635720147EF0A25F927D238F3E0CE9A5D5EC2BF5FFBE48CB1418**, 230 archivos verificados sin divergencias. R1 conserva sus 153 archivos y MANIFEST SHA **5A2EA552FEA81E9C59271F16F968719251AC4EF3BDF6B1F8254495556E368FC9**, también verificados íntegros.

Nu recurso SHA **1A0972876A8EE4BF1C3E97E12B9B71FDF82F42F87685062620484818B3D577AF**. Derivador Nu SHA **58DC1DBE8984CE2D0F751DF4297C61167286525763B5BB1380EE0D57415A8DD5**. Ledger SHA **D6756AF3F3C8647866B0F71B7EE4292F53F1438CAD58C7BEA0DCA310987132A3**. DELTA.json acredita identidad de PS/TOML/Rust/lock/bins respecto a R1.

Dump nuevo utilizado realmente en las cuatro matrices SHA **301A65C46F0B86764BBF557674616EA899CBE67C402E48751EB3AEA6B7B62C51**; fancy lo genera y ambos motores lo leen. Son bytes iguales en estas ejecuciones, sin promesa de reproducibilidad entre plataformas. Bin fancy **33DEB3817DAFE2BC3D1684A2CA519EA291CFF9FB1C647F67A61F578C9FCC0376**, Onig **6E1A8EAF6CB425A29A066540522B9A68C601A7F2CE4C8AF0B1D3418C8F3941A6**.

## Corrección

Antes de crear `meta.record-entry.nushell`, el gate exige el owner completo original, incluido cierre válido de comillas y dos puntos. Así, una interpolación sin ':' sigue la regla original de string y no crea entrada persistente.

Dentro del owner, un lookahead captura el interior original y el sufijo desde la comilla terminal hasta el final de línea. La apertura consume `$"` o `$'` y aplica una variable; un embed propio retokeniza exclusivamente capture2 mediante `#paren-expression`, con las dos variables correspondientes a los captures anidados originales. El escape nativo identifica el extremo por el sufijo capturado y anclado; Syntect sustituye su backreference por literal escapado y lo propaga a contextos anidados. Por ello un paréntesis incompleto dentro de capture2 no puede consumir el cierre, el valor ni la próxima línea. La comilla terminal recibe una variable y ':' recibe keyword.control. El valor mantiene la regla record-value y el cierre original de entry.

Ledger contiene owners, gates, remapeos, scopes y mecanismo de recorte. No se añaden scopes de escape que la retokenización original no asigna. Se conservan AST/remapeos N02/N03, N04 y SH02 de R1.

## Resultados ejecutados

| Corpus independiente | Fixtures | Intervalos exactos fancy / Onig | End stacks exactos fancy / Onig |
|---|---:|---:|---:|
| Oracle base R2 SHA8918CBCF… | 57 | 161/161 ambos | sin oracle general de fin |
| Addendum R3 SHA54BB7528… | 19 | 48/48 ambos | sin oracle general de fin |
| Nuevo addendum revisor SHA65B12B89… | 8 | 60/60 ambos | 16/16 ambos |

Los corpus y expectativas permanecen inmutables, escritos por otros agentes antes de resultados R2. Total: 84 documentos reconstruidos, 269 intervalos seleccionados y 16 estados prescritos por motor; no full golden de cada documento. El nuevo corpus comprende los seis contraejemplos previos y dos variantes UTF8. Se verifican explícitamente expectedEndStacks mediante `verify-shell-r2-endstacks.py`, porque el verificador anterior sólo usa assertions.

Además, selfcheck separado de seis criterios previos del adversario confirma no-entry sin ':', doble variable en `b` después de quote escapado, recorte del paren en capture2 y ausencia de contaminación en echo; se etiqueta como selfcheck, no dictamen AE.

Load de los tres recursos y enlaces pasa en ambos motores. La carga Syntect compila expresiones con placeholders; runtime y backrefs dinámicos quedan acreditados por estos casos concretos, no por todas las posibles entradas. Inventario R2 corregido registra 292 campos match/escape explícitos y 29 definiciones de variables; no es el número de objetos Regex internos del loader. El inventario R1 se conserva con otro nombre, sin reutilizar su cifra como prueba R2.

Cada corrida usa Popen propio, stdin cerrado, comunicación limitada a 10 s, kill/wait únicamente del proceso retenido en timeout/finally, salida y hashes en RUNS. Todas terminan 0 sin timeout. Clonado y replay de ParseState comprueba operaciones idénticas por línea en el runner. No demuestra Job, cuotas o lifecycle del worker de producto.

## Preservación y límites

Intentos R1 y errores documentados siguen en el snapshot original y en R2. Una copia preparatoria `shell-grammar-prototype-r2/` se creó antes del nuevo addendum y del inventario documental actualizado; se conserva, pero **no es el candidato final entregado**. El candidato exacto es `shell-grammar-prototype-r2-final/` y su MANIFEST anterior.

Los scripts auxiliares tuvieron fallos propios de lectura cp1252/alias `utf8-sig` y formato de manifiesto; se corrigieron a UTF-8 y ambos esquemas explícitos. No se transformó ningún fallo de gramática en PASS mediante cambios de oracle. La fuente y los criterios congelados no cambian.

Pendiente: informe independiente del adversario R2 y aceptación independiente sobre este snapshot; después, integración de catálogo/aliases y worker real con límites/timeout/cancel/reap/Fuente/viewport, pruebas de entradas hostiles y visuales, y ejecución nativa Unix. No equivalencia universal TextMate, color semántico, cobertura de todas las reglas ni cierre global S03. No leer módulos ni ejecutar Nushell/PowerShell para analizar sintaxis.

Estado final: revisión y AE independientes sin críticas dentro del corte; ambos motores269intervalos+16estados finales. Informes canónicos en ../reviews/S03-shell-prototipo-r2.md y ../reviews/S03-shell-prototipo-aceptacion-r2.md. Integración de producto aún pendiente.
