# Private Jiter derivative

Jiter0.17.0, MIT; verbatim upstream LICENSE retained. UPSTREAM.json fixes original crate/VCS and per-file hashes. Only NumberRange::decode removes the artificial4300digit count; raw-number.patch and original upstream/number_decoder.rs preserve the derivation. NumberInt/Float implementations remain unchanged; NumberRange is also used by float diagnostic helpers, so no blanket equivalence claim is made. Product uses only raw lexeme APIs, never float/int conversion or map DOM. Source/arena/Job/deadline limits belong to R3Viewer. Reviewed parser gate: docs/reviews/S03-D2-parser.md; product projection and acceptance remain separate. No Python feature enabled.

Packaging-only change: normalized Cargo.toml adds an empty workspace to isolate the private dependency and its benches/dev features. Original manifest is preserved at upstream/Cargo.toml. Application selects default-features=false.
