# Avisos de dependencias incorporadas

R3Viewer usa licencia MIT. Este directorio conserva avisos específicos de la incorporación de sintaxis; no es todavía el inventario transitivo completo ni certifica el paquete final.

- Syntect5.3.0: Cargo declara MIT; LICENSE original conservado. Se usan default-syntaxes y regex-onig, sin ThemeSet ni gramáticas de usuario. Código y assets integrados requieren revisar sus avisos al distribuir.
- Onig6.5.3 y onig_sys69.9.3: wrappers MIT, LICENSE originales conservados.
- Oniguruma incluido por onig_sys: COPYING original con copyright y dos condiciones de redistribución conservado; no sustituirlo por la licencia del wrapper.

Procedencia: paquetes exactos de Cargo.lock, registry descargado por toolchain privado. Archivos de licencia copiados sin modificación. Upstreams: https://github.com/trishume/syntect y https://github.com/rust-onig/rust-onig. El backend Crossterm, SDK Nu y roles R3CLI tienen su procedencia/avisos en vendor. E10 completará todas las dependencias de los bundles antes de distribuir.

## Saphyr-parser 0.1.0

YAML serialization parser, MIT OR Apache-2.0. Vendored exact reviewed R2 candidate under vendor/saphyr-parser; UPSTREAM.json records 39 original files, published crate/VCS, modified parser/scanner hashes and packaging manifest. LICENSE plus .licenses/ChenYuheng-MIT, ChenYuheng-Apache, Ethiraric-MIT, Ethiraric-Apache are retained verbatim. Local patches and upstream source copies preserve derivation. No constructors or alias expansion.

ArrayDeque0.5.1, dependencia del parser: Cargo declara MIT/Apache-2.0; LICENSE completo original conservado en licenses/arraydeque-0.5.1-LICENSE.txt (Andy Lok). Fuente registry exacta fijada en Cargo.lock, sin modificación.
