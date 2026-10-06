# D2b0 R2 — aceptación independiente focal

Dictamen: **SIN_CRITICAS_PENDIENTES en la biblioteca b0 HASH/SOURCE y kernel de ownership examinados**. Candidato A1D17B65686630EA352A737914A79212CC1E6E7DA17B65B201B8C9BFBC1C66CD (602 registros), contexto 558 archivos. No aprobación de b1, sesión, UI o producto completo.

Revisión estática completa de preparation_owner.rs y source_preparation.rs; delta de Document::source_handle, módulos privados lib.rs y receipt en presentation.rs contrastados. El getter clona exactamente inner.source inmutable; no permite fabricar Document/proof, rehash ni reopen. Binding preserva input/generation/request/revision/digest. El contexto se construye sólo por HASH completo o Document validado y comprueba Arc original/origin/generation/request. SourcePrepared conserva el contexto original.

OwnedPreparation conserva input, stop, receiver, candidate, JoinHandle, receipt y permit. La Box de AuthorityState se asigna antes de spawn y traslada completa al retainer global si Drop ocurre con el hilo vivo. PendingTask sólo despacha finished/reap y no extrae ni publica resultado. La autorización global se libera al destruir Parts después de join y de soltar output retenido/input; los outputs validados que devuelve la API pueden sobrevivir y compartir el original legítimamente. TakeValidated compara identidad e intención de salida después del receipt y join. SendFull/Disconnected/panic y stop son errores terminales, sin promover candidato.

Se verificaron 602 identidades antes de copiar; copia de repo byteexacta. La revisión final vuelve a comprobar esos registros y el contexto 558. Sólo dos sufijos cfg(test) se añadieron a la copia propia: módulo de pruebas hijo en preparation_owner y observador de capacities privadas en presentation. Los prefijos productivos son byteexactos al freeze. No se cambiaron funciones, constructores ni visibilidad productiva. El observador contrasta capacities reales con receipt; no acredita RSS. Build propio offline escalado, ejecución de binarios propios bajo sandbox Windows/PowerShell. No procesos auxiliares/Jobs/perfiles ni rescates externos.

Expectativas previas: EXPECTED-before-output.json 067E00B30DE7FF982CB351D197AC25557DE9FEEDCE5C08BE86E0828F8B81ACC6; root SP01–09 3A12A96126321E1AFEF4576C6C9C185CEF068E0BB1E963DF46C2344A550867E9. Fixture propia 15 bytes SHA94A344C1AA1B333CC9F457548E351B01F1664BF79B3AD9CF3E4F7D9BF0B1044F y proyección/rangos fijados en ADDENDUM-before-output.json 2122F4DC89A0ABC72444D953F831D1A80C83D7ABFD566F80133B4D2A8732E1F0. Addendums2/3 previos al runtime nuevo completan cobertura sin cambiar expectativas anteriores. La revisión del diseño HASH→SOURCE F7489E8169FFFAAC44DE990B9165DE21939675D4E8E32ECB6A9BCB823A3AD966 es antecedente, no sustituto del runtime.

## Evidencia funcional propia

12 grupos únicos pasan: ocho originales en TEST-attempt1 (0.03 s), once incluyendo tres añadidos en TEST-attempt3 (0.13 s), sólo el control nuevo Pending-local-Source en TEST-attempt4 (0.01 s). Los ocho repetidos en attempt3 no se cuentan como nuevos. Ninguna suite completa se ejecutó.

- HASH→SOURCE real: SHA independiente, original Arc idéntico, origen y gen11/request19, digest None→Some sin cambiar inputIdentity, revisión1→2; contexto idéntico en modelo. Texto seguro completo, ESC/C1/CRLF y rangos de procedencia. Capacities original/visible/checkpoints/lines/index comprobadas contra campos privados observados.
- Ready real recibido mientras wrapper mantiene pendiente su FinalReceipt: poll no finalizado y TakeValidated=None; sólo join permite devolver modelo/receipt joined.
- Error final explícito y panic después de enviar Ready: candidato rechazado. El panic registrado en logs es inyección deliberada, no fallo espontáneo.
- Canal real lleno mediante seed previo y receptor real desconectado antes de callback Source: SendFull/Disconnected, salida retenida hasta final/join, sin éxito ni autorización global residual.
- Pending local con resultado Source retenido: mismo owner rechaza Hash y Source con Pending; otra Authority rechaza con Busy mientras OCCUPIED conserva el owner local. Es distinción nominal correcta: no admite thread ni resultado. Drop traslada el owner a PENDING global y las factories Hash/Document() rechazan con Pending; callback Document() no ejecutado. Liberar barrera permite reap original joined/causeCancelled/binding y soltar input; después Hash admitido.
- Segundo caso local: Source también denegado en Authority ajena; retry_cleanup del propietario exacto devuelve joined/causeCancelled, libera slot y Source posterior real pasa. No se atribuye al retainer global una limpieza local.
- Identidad revision obsoleta e intención de salida rechazan modelo terminado; cancel inicial de prepare_source real devuelve Cancelled sin prefijo.
- Spawn falla explícitamente antes de crear hilo: input se libera y permit global queda vacío. Inyección en callback de Builder, no afirmación de fallo natural del SO.
- Document real project/validate/quarantine→from_document: source Arc y original exactos; leases Document sobreviven hasta último modelo. Dos modelos simultáneos poseen visible distinto y original compartido; liberar uno no invalida otro. Capacities y headers reportados no se suman como RSS ni como Consumer.
- Entrada sintética 1.000.000 LF: falla la cota de 1.000.000 líneas (produce una línea adicional) sin modelo parcial; contexto externo sigue válido. No se repiten matrices gigantes/128MiB.

## Intentos e identidades

BUILD-attempt1 terminal0, 47.81 s; OWN-UNIT-attempt1.exe AD74C5BFCB82BBC5B2D09F7FF96CF6FBADEE549D5D33A53B18DB6BFA146ED380.
BUILD-attempt2 terminal101 por import privado de ActorArena en el harness (E0603), preservado junto HARNESS-attempt2-before-import-fix.rs. Se corrigió únicamente el import a document_budget::ActorArena; ni candidato ni expectativas cambiaron. BUILD-attempt3 terminal0; OWN-UNIT-attempt3.exe 3CCE5C0AA27C1EFE3FD3D4B043CD6182790D4B8462331E89AFA87E612BFBD0D2. BUILD-attempt4 terminal0; OWN-UNIT-attempt4.exe 2B1F3C14103CF7060E109964E2E74E588F29130F21B9B3C9B55CAEC1D6656ECC. Sesión6970 terminó0 antes de copiar/ejecutar bin; no handles activos al cierre.

La advertencia de resolución de C:\Python314\python.exe del setup se conserva en SETUP-output.txt; script terminó0 y validó/copió archivos. No se atribuye ese diagnóstico a Rust/runtime candidato.

## Límites

SP01–09 se acreditan en la parte de ownership, stale, salida/cancel, panic, canal y reap de biblioteca que corresponde al corte b0. No se afirma q0/CtrlC130 físico, fatal1 de terminal, Loading/pump, ConPTY/restauración, cancel durante el recorrido del texto, ni un plazo nuevo de preparación; estos requieren integración posterior. DI()/DO() se usa sólo para comprobar la autoridad del kernel, no representa DocumentPrepared/b1. Source conserva sus límites existentes, separados de Document arena y Consumer; se midieron capacities de fixture pequeña, no todas las fronteras 128/256MiB, memoria agregada/RSS o internals de canal/hilo/allocator. No runtimes nativos Unix, TUI, b1/layout/search/render ni cierre RV.
