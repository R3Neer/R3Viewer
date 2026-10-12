# Nushell SDK local patch

Upstream crates.io nu-plugin-core and nu-plugin 0.116.1. Their MIT licenses, sources and original manifests are preserved. UPSTREAM.json records original archive and file SHA256 values. No nu-protocol ABI or message protocol change.

Changes against upstream: core src/interface/stream/mod.rs introduces per-reader Signals and polling cancellation; its tests add seven regressions. core src/interface/mod.rs passes input signals before type erasure. plugin src/plugin/interface/mod.rs creates latched invocation flags, prunes weak entries, propagates input cancellation and leaves global Reset for new calls only. Its tests add three regressions and adapt eleven private-manager bindings to mutable access.

Both crates use exactly 0.116.1, matching the selected host version and nu-protocol dependency. This is a maintained local fork, not an upstream Nushell release or installed shell modification. An SDK upgrade requires reviewing/removing this patch and rerunning blocked-input, local/global signal, flow-control and actual Nu reuse tests. EngineCallResponse streams remain unchanged; this encoder does not request them.

Historical 0.115.1 patch review: SIN_CRITICAS_PENDIENTES. Historical isolated Windows validation:43 plugin unit tests,78 core unit tests,17 doc examples passed. Synthetic lifecycle spike using actual Nu0.115.1 passed idle list/byte producers, repeated abandonment and independent concurrent calls. Product wire/encoder integration must be validated separately. Arbitrary upstream I/O and a blocked protocol writer are outside the polling receiver guarantee.

Git normalizes text line endings to LF; upstream raw and normalized hashes are both recorded. nu-plugin README has one redundant final blank line removed for diff checks, without changing its prose.

## Upgrade to 0.116.1 (2026-10-12)

Official crates were downloaded from static.crates.io; UPSTREAM.json records their archive and original raw/LF file hashes. Every upstream file affected by our polling/signals patch is byte-identical after LF normalization between 0.115.1 and 0.116.1. The five Rust code/test file deltas were therefore reapplied without API edits. The redundant README final blank remains removed. No local nu-protocol fork or protocol/ABI change is introduced.

Official 0.116.1 changes are retained: Nushell dependencies and minimum Rust version (1.96.1), plugin local-socket feature forwarding, and Unix local sockets in a fresh owner-only directory with cleanup. The Windows named-pipe implementation remains upstream. Unix socket regressions are cfg(unix) and cannot be certified by Windows tests. Current upgrade validation is recorded separately; historical tests above do not establish 0.116.1 host compatibility.

Archive packaging metadata (.cargo_vcs_info.json and each published crate Cargo.lock) is not copied into the maintained source tree, following the previous vendor layout; hashes remain in UPSTREAM.json and the downloaded archives retain originals. The isolated vendor workspace generates its own private/ignored Cargo.lock for tests; the root Cargo.lock is the production dependency authority.

Upgrade author validation on Windows: companion `cargo check -p r3viewer-nu-plugin` passed; isolated vendor `cargo test --manifest-path vendor/nushell/Cargo.toml --workspace -- --test-threads=1` passed 43 plugin unit tests, 78 core unit tests and 17 documentation examples, with zero failures or ignored tests. These include retained polling/ACK/drop, per-call cancellation, global Reset latching and flow-control regressions. Actual Nu 0.116.1 integration/reuse and independent review are recorded in the project validation report; this author check alone does not establish host acceptance or Unix socket execution.
