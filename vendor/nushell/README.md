# Nushell SDK local patch

Upstream crates.io nu-plugin-core and nu-plugin 0.115.1. Their MIT licenses, sources and original manifests are preserved. UPSTREAM.json records original archive and file SHA256 values. No nu-protocol ABI or message protocol change.

Changes against upstream: core src/interface/stream/mod.rs introduces per-reader Signals and polling cancellation; its tests add seven regressions. core src/interface/mod.rs passes input signals before type erasure. plugin src/plugin/interface/mod.rs creates latched invocation flags, prunes weak entries, propagates input cancellation and leaves global Reset for new calls only. Its tests add three regressions and adapt eleven private-manager bindings to mutable access.

Both crates remain version 0.115.1 for protocol compatibility. This is a maintained local fork, not an upstream Nushell release or installed shell modification. An SDK upgrade requires reviewing/removing this patch and rerunning blocked-input, local/global signal, flow-control and actual Nu reuse tests. EngineCallResponse streams remain unchanged; this encoder does not request them.

Patch review: SIN_CRITICAS_PENDIENTES. Isolated Windows validation:43 plugin unit tests,78 core unit tests,17 doc examples passed. Synthetic lifecycle spike using actual Nu0.115.1 passed idle list/byte producers, repeated abandonment and independent concurrent calls. Product wire/encoder integration must be validated separately. Arbitrary upstream I/O and a blocked protocol writer are outside the polling receiver guarantee.

Git normalizes text line endings to LF; upstream raw and normalized hashes are both recorded. nu-plugin README has one redundant final blank line removed for diff checks, without changing its prose.
