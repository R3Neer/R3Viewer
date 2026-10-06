# Avisos de dependencias incorporadas

R3Viewer usa licencia MIT. Este directorio conserva avisos específicos de la incorporación de sintaxis; no es todavía el inventario transitivo completo ni certifica el paquete final.

- Syntect5.3.0: Cargo declara MIT; LICENSE original conservado. Se usan default-syntaxes y regex-onig, sin ThemeSet ni gramáticas de usuario. Código y assets integrados requieren revisar sus avisos al distribuir.
- Onig6.5.3 y onig_sys69.9.3: wrappers MIT, LICENSE originales conservados.
- Oniguruma incluido por onig_sys: COPYING original con copyright y dos condiciones de redistribución conservado; no sustituirlo por la licencia del wrapper.

Procedencia: paquetes exactos de Cargo.lock, registry descargado por toolchain privado. Archivos de licencia copiados sin modificación. Upstreams: https://github.com/trishume/syntect y https://github.com/rust-onig/rust-onig. El backend Crossterm, SDK Nu y roles R3CLI tienen su procedencia/avisos en vendor. E10 completará todas las dependencias de los bundles antes de distribuir.

## Saphyr-parser 0.1.0

YAML serialization parser, MIT OR Apache-2.0. Vendored exact reviewed R2 candidate under vendor/saphyr-parser; UPSTREAM.json records 39 original files, published crate/VCS, modified parser/scanner hashes and packaging manifest. LICENSE plus .licenses/ChenYuheng-MIT, ChenYuheng-Apache, Ethiraric-MIT, Ethiraric-Apache are retained verbatim. Local patches and upstream source copies preserve derivation. No constructors or alias expansion.

ArrayDeque0.5.1, dependencia del parser: Cargo declara MIT/Apache-2.0; LICENSE completo original conservado en licenses/arraydeque-0.5.1-LICENSE.txt (Andy Lok). Fuente registry exacta fijada en Cargo.lock, sin modificación.

## quick-xml 0.42.0

XML slice event reader, MIT (Johann Tuffe, 2016), registry package pinned in Cargo.lock. Default features disabled; memchr dependency shared. Published crate SHA256 41B1177FDF999D2321D3FB46FF47159D9C1FB9AD66A4879F8C50A0B504615E9B, VCS 36a2c52a4f6c90878c3f60c3c3b5c62efaef28a9 as recorded in XML research. Original license retained verbatim under licenses. No upstream patches, serde DOM, external resource resolver or NsReader semantic authority. Upstream: https://github.com/tafia/quick-xml. This entry is not a complete distribution inventory.

## Unicode-width adapter 0.2.2

Adaptación local mínima de unicode-width0.2.2 (MIT OR Apache-2.0), copiada del gate privado Unicode R2 aceptado. vendor/unicode-width-adapter conserva LICENSE-MIT/LICENSE-APACHE, fuentes upstream originales como prefijos byte-exactos y R3VIEWER-DERIVATION.json. Añade checks de cancel/preadmisión y estado inverso WidthInfo entre bloques, sin cambiar tablas/reglas/wrapping. Cargo usa nombre r3viewer-unicode-width-adapter para no sustituir unicode-width utilizado por terceros. Manifiesto privado normalizado elimina sólo dev/bench/rustc dependencias ajenas al perfil; originales conservados. La integración de UI sigue en desarrollo, no certifica distribución ni equivalencia Unicode universal.
