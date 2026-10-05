# Embedded TOML syntax

Upstream: https://github.com/jasonwilliams/sublime_toml_highlighting
Commit: f5a57e8bff694a4e6c52a491dae579aabc7427cf (bat v0.26.1 pin).
License: MIT, Copyright 2021 Jason Williams; verbatim LICENSE included.
Original SHA256: 581328AEA6C0028AB613FF26B3114C4AE37F9FA29F7BE1042419BA353A6EABC6.
Derived R4 SHA256: 06B80C0D48F6BC9E9B7339E1D2C79ACD1791C61465DAB4A334BDAE1723268F93.

This is a local derivative, not an unmodified upstream grammar. R3 changes add TOML1.1 e/x escapes, optional time seconds, multiline inline tables and trailing commas, and CRLF whitespace; R4 changes multiline continuation with spaces/CRLF and closing 3..5 quotes. Both transformation patches are preserved. Semantic interpretation is handled separately by the TOML parser.

build.rs compiles this fixed resource with Syntect5.3.0 together with the default catalog into a compressed SyntaxSet dump in OUT_DIR. The build-side regex backend is fancy; runtime Windows is Onig. The worker loads only include_bytes! of that dump, never rebuilds the catalog, parses YAML or opens a grammar path. Exact runtime compatibility and corpus are tested independently; this resource does not claim universal TOML1.1 highlighting conformance.