# PDF Rust: revisión independiente de diseño R1

2026-10-05. Revisor distinto del autor unix_r3_review. Sólo fuentes/propuesta; no Cargo, implementación, carga PDFium, PDF ni ejecución nativa.

## Identidad y dictamen

Propuesta `PDF-Rust-gate-proposal-r1.md` SHA256 `BC7298CF6FFD28515D9C9CFDAAEFDBA08BD07CC3AB50A7132910661835374883`; evidencia `PDF-Rust-gate-source-evidence-r1.json` SHA256 `D03288858DED1D89A4E1C60F594CA0C4449AB147AEDC903179F954698A003110`. Ambas leídas/comprobadas;19 archivos declarados contrastados con bytes/hashes actuales, cero divergencias. Contraste con PDF-G0/G1, fuentes7881 y wrapper0.9.4 exactos, review propia del tiny ctypesR2.

**SIN_CRITICAS_PENDIENTES exclusivamente para el diseño del siguiente gate privado Rust.** FFI mínima recomendada como experimento; no arquitectura definitiva, ABI/unsafe aceptado, cuotas finales ni producto PDF aprobado.

## Alternativas y ABI

La distinción entre loader y wrapper completo está fundada: bind_to_library usa Library::new; los objetos altos introducen bindings globales OnceCell y form-fill, y rutas de texto que reservan/copian regiones enteras o colapsan resultados. Cambiar flags no elimina estas propiedades. La alternativa de usar bindings bajos con RAII propio evita las rutas altas, pero no equivale a aprobar su catálogo de símbolos o global lifetime. No hace falta implementar ambas alternativas para este gate.

Flags0x100|0x800=0x900 y ruta absoluta mantienen la corrección G1. libloading0.8.9 local no se confunde con latest ni con el rango abierto del wrapper; lock y whitelist de símbolos deben congelarse antes de compilar. [LoadLibraryExW, fuente primaria actual](https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw). La consulta docs.rs0.8.9 no devolvió contenido en esta revisión; las semánticas se contrastan con la fuente local fijada, no con memoria o una versión distinta.

extern system y repr(C) completos son adecuados al corte Windowsx64, sin extrapolar Unix/x86. Préstamos a API/padre/source y buffer mutable, NonNull sin Clone, marcadores !Send/!Sync, scopes anidados y cierre antes de unload cubren las obligaciones de diseño. No prueban todavía que una implementación mantenga cada préstamo ni que no permita dos InitGuard simultáneos: el candidato debe conservar una inicialización exclusiva por proceso y orden documento→DestroyLibrary→Library. Drop no puede hacer logging/alloc/panic que impida otros cierres. Compile-fail del préstamo y fallos parciales son gates reales pendientes, no evidencia aportada por ctypes.

## Texto, mapping e identidad

Header7881 CountChars da -1 en error; GetText documenta UCS2, capacidad count+1 y omisión de caracteres no representables. GetUnicode devuelve unsigned int por índice PDF y cero cuando no puede convertir; no se sustituye automáticamente por una extracción completa verificada. La propuesta reconoce esa condición y fija identidad de capa por método, impidiendo concatenar dos rutas que dupliquen caracteres.

fpdf_searchex.h fija conversiones de índice con -1 en error. Mapping no se deriva de longitudes UTF8/UTF16 ni de una suma de count devuelto. El header también ofrece HasUnicodeMapError con1/0/-1 e IsGenerated: deberán considerarse al concretar diagnóstico del candidato, porque un scalar válido por sí solo no acredita ausencia de problemas de mapping. Esta es la concreción de «verificar unmapped/generated» ya exigida, no permiso para declarar Complete desde GetUnicode sin evidencia.

UTF16 incremental independiente con carry, reserva checked antes de append, estados separados y ningún reemplazo inventado buscable preservan G0. El texto fuera CropBox sigue perteneciendo a la capa; falta de BBox no borra match ni permite highlight ficticio. n/N físico+intervaloUTF8+hash/generación y huecos que impiden cruzar segmentos conservan identidad. Los oracles de ligadura/astral/generated/controles/huecos deben fijarse antes de resultados y distinguir resultado de API, capa prometida y cobertura incompleta.

## Memoria, ventana y supervisor

Los techos iniciales1MiB/256páginas/4MiBbitmap/65536índices/256KiBtexto/chunk1024/controlJSON64KiB están etiquetados como experimento. Preadmisión checked y try_reserve, ledger conjunto y límite+1 quedan exigidos; no basta comprobar longitudes después. El candidato deberá fijar tamaños/capacidades del mapping y coexistencias, incluyendo transporte/old+new, antes de reservar; retener múltiples páginas no puede escapar del ledger. LoadPage/Text_LoadPage internos se acreditan como coste nativo contenido/medido, no prealloc gratuito.

Transformación/clipping/rotación/matte y bbox deben contrastarse con fixtures MediaBox/CropBox/offset/rotación conocidas. Dimensiones finitas positivas, conversión c_int, stride y bytes preceden CreateEx. Guardas comprueban el buffer observado, no equivalencia visual ni memoria interna. La propuesta no usa resolución natural/copia de bitmap como atajo.

El supervisor hereda ownership desde Popen, setup/readback Job antes de bootstrap, captures bounded, deadline único y cleanup que intenta todo antes de evidencia. Regresión de fallos ctypesR2 no aprueba automáticamente un nuevo helper Rust/Python. Holds y stub FFI bloqueado son controles distintos: el stub sólo demuestra la ruta de terminación frente a llamada sin retorno. No afirma PDFium real bloqueado ni ejecuta Drop al matar el proceso. EOF/parcial/stale/request/hash invalidan publicación.

## Alcance pendiente

Congelación del paquete/lock/whitelist/ABI/RAII/corpus/oracles y revisión antes de ejecución satisfacen el carácter preparatorio. Quedan pruebas propias nativas del candidato Rust, adversario y AE distintos del autor. Cifrado/permisos no se excluyen del producto por el corte empty-password; búsqueda completa/lazy/historial/links/protocolo/cache/terminal/URLs y cuotas finales siguen G0 y RV204. Unix requiere evidencia nativa, no crosscheck. No se pidió ni generó aprobación adicional para el trabajo ya autorizado; este dictamen tampoco amplía por sí solo el scope privado a integración de producto.
