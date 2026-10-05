# JSON — revisión independiente de parser rawRange R1

2026-10-05. **SIN_CRITICAS_PENDIENTES** para esta adaptación candidata y superficie raw examinada. No aceptación del adaptador JSON completo, arena, IPC/lifecycle/presupuestos, visual ni hosts Unix. No producción modificada.

## Identidad y alcance del cambio

Leído informe r3viewer-json-parser-spike-r1.md, driver y fuentes exactas originales/derivadas. SHA number_decoder.rs60F789A6F8AF138E06CA4A0802FA5BCA5FDDC198E20A771A30631C80C9BBEA78 y driver1A6E6483430251F4714F8F5A00769C992D349904F868C8EC843AEF05ED72161F coinciden. Comparados hashes de55archivos originales con derivados: solo src/number_decoder.rs cambia. Diff elimina digit_start/check4300 en NumberRange, añade comentario, no cambia el bucle de validación ni conversión NumberInt/NumberAny/NumberFloat. Caps4300 de rutas Integer/BigInt siguen en líneas217–219 y413–414.

Precisión de alcance: NumberRange no lo usa exclusivamente next_number_bytes; también participa en float_error para recuperar diagnóstico de conversión flotante y en rutas de salto léxico. La adaptación puede cambiar diagnóstico/coste de esos consumidores al admitir tramos largos, aunque sus conversiones no hayan sido editadas. No convertir el nombre rawOnly en afirmación de aislamiento absoluto entre APIs. No encontré una regresión de resultado/panic en next_float bajo los casos ejecutados; no certifico igualdad de todos los diagnósticos aritméticos ni runtime de feature num-bigint.

## Pruebas propias

Harness/target privados Cviz/json-parser-adversary, fuente src/main.rs y log tests-derived-r1.log; Cargo.lock fijado, cargo test --offline --locked serial con dependencia derivada exacta, default-features=false. **5 PASS, 0 FAIL**, WindowsRust1.99. No se escribió en spike autor/producción.

1. RawNumber: longitudes4299/4300/4301/4302/65535/65536/65537, signo opcional negativo y entero/decimal/exponente. Bytes exactos y finish limpio. Sufijos truncados e+/e-/punto, segundo valor y cero inicial se rechazan.
2. -0/1e400 y exponente con65536dígitos conservan lexema, no conversión f64.
3. next_float sobre4300/4301/65536dígitos como entero, decimal y exponente: catch_unwind confirma sin panic en todos los casos. Esto no exige conservar números como float ni autoriza usar este camino para presentación JSON.
4. next_object_bytes/next_bytes con string vacío, key CJK, raw escaped y scalar noBMP: pointerrange está dentro de source y las comillas/envueltos UTF8 se verifican, incluidos segmentos vacíos. next_object/next_key conserva tres occurrences a/escaped_a/a en orden, ninguna colapsada.
5. Escapes slash/control y parejaUTF16 válidos; high/low surrogate aislado, pareja rota y escape desconocido rechazados al decodificar. Segundo valor y whitespace noJSON U+00A0/U+000B rechazados, al igual que NaN/Infinity/+1/01/1./1e.

No hubo hallazgos accionables en el cambio exacto ni estos casos. finish sigue siendo indispensable tras consumir exactamente una raíz; el prefijo léxico por sí solo no acredita EOF. No usar next_skip para validar Unicode ni bytesAPI como garantía de UTF8. El contrato mantiene frontera de entradaUTF8 estricta; usar rango crudo de fuente verificado, no pointer de next_str que puede prestar scratch cooked. current_index previo al valor puede incluir whitespace; peek/offset verificable evita atribuirlo como start literal.

## Límites y paso siguiente

Aceptar este gate permite proponer Jiter derivado como candidato del siguiente corte, con pin/crate original/MIT/transformación y patch explícitos, sin alterar paquete compartido global fuera del proyecto. La arena directa debe comprobar tipos/ordinales/spans/depth y contabilizar cooked/metadata/detección exacta de duplicados; null/número propios y surrogate diagnóstico de representación según contrato. Ninguna aceptación de token raw prueba esto todavía.

El bucle retirado es potencialmente largo y no cooperativo; input/proceso/deadline siguen a cargo del consumidor. No se probaron128MiB, memoria pico ni cancelación. La medición autor16MiB94ms queda una observación debug aislada, no SLA total. Reutilizar lifecycle estructural una vez implementado/revisado y comprobar publicación completa, EOF/códigos/límites, contención y cleanup del candidato exacto. S03/RV106 permanecen abiertos; JSON no sustituye aceptación TOML todavía activa.
