# Gate privado de IPC gráfico: R2

2026-10-05. Estado: revisión de código y aceptación independiente **sin críticas pendientes en el alcance privado Windows**. [Código y regresión de raíz](../reviews/GRAFICOS-ipc-code-r2.md); [aceptación independiente](../reviews/GRAFICOS-ipc-aceptacion-r2.md). GIPC-ROOT01 queda cerrado para el experimento. G2, integración y tareas RV siguen pendientes.

## Corrección y evidencia

R1 falló: cleanup=true con tres pipes no transferidos aún abiertos, comprobado mediante GetHandleInformation. R2 conserva los propietarios hasta cierre explícito, incluso tras error/panic, y separa join, receipt limpio y cierre. El duplicado rechazado se retiene para cierre sin asumir autoridad para esperar/matar un D no validado. Pending bloquea el siguiente actor; cualquier error sigue irreversible aunque después se logre cerrar.

Freeze R2: MANIFEST F93A15AF6D9AFCD73459BAA8E74081EFB46F7536A02A5B531B4DFF246F44DAB8,383 registros. H C6C72D810B48F29E84D50E279B5B9CD4EF2161B31D7422F3C79A37EDB52EAAC3; C BA50BF423EE29BF53AB04AFFA255305DA2EF13BBB883EC1D458B6E7D2CD41088; D D8E533002EE5A7759C933799E6691C0FA4AA32930329DD9BC4522935E3ADB066. Snapshot temporal graphics-ipc-candidate-r2 y espejo .tmp/graphics-ipc-gate-r2; no se versionan binarios/cachés.

Raíz verificó383 registros original/espejo/copia AE y63 registros de evidencia independiente. Delta fuente propio: sólo windows/managed/hooks/H/C cambian; wire/codec/ledger/Sixel/D y lock permanecen byteidénticos. Prueba propia append-only de ROOT01 pasa: tres valores HANDLE originales inválidos antes del teardown. Los informes fijan hashes del oracle previo, log y fuentes.

AE:149 casos de binarios exactos,18 pixels+119 controles+12 cierres; cuatro grupos de tests propios nativos adicionales. Seis fallos transitorios terminan FailedReaped; seis persistentes mantienen FailedPending sin publicación ni nuevo actor. Su terminación por watchdog externo es teardown, no éxito de cierre interno. Se conservan rechazo R1, fallos de harness/oracle anteriores y fallo inicial de wait50ms del driver R2. Controles de close fallan antes de la llamada kernel; no se presentan como errores naturales Windows.

## Lo que sigue pendiente

Integrar IPC en servicio gráfico real después de la frontera Unicode y validar combinación con documento/recursos/sesión. Mantener cuotas completas del producto, terminal/framing, inline, cache/coalescing, SLA, corpus/stress, QA visual, Unix y distribución abiertos. Requested256MiB H40+C80+D136, Job512MiB por proceso y RGBA24MiB privado no son aceptación del producto RGBA64MiB ni medición RSS/margen nativo. La aprobación de este experimento no sustituye la revisión posterior del conjunto.
