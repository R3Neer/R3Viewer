# Evidencia de ciclo de vida TOML del implementador

Candidato D1 R1; comprobación del autor, no aceptación independiente ni QA visual/matriz.

# D1 core candidate A: functional evidence R1

2026-10-05, Windows, private Rust 1.99 / debug build. Product and private harness compiled offline; no profiles, global installs, process-global console events, desktop/GUI operation, stage or commits. Only own child processes are terminated/reaped. This is functional/process evidence, not visual QA and not a substitute for independent candidate review.

API: immutable Structure retains exactly the original Arc<String>, exposes NodeRef/EntryRef/LeafRef and separate ordered physical declarations. StructureClient is session-owned and asynchronous; document generation + request id + revision guard publication. Ready and Unavailable occur only after child/IO cleanup; local cancel keeps the session flag false and returns a ReapTicket. A failed wait on launch error now propagates Failed instead of assuming Reaped. This last rare OS error branch is supported by code examination, not by an injected invalid OS handle test.

Limits: source128MiB; nodes+edges1M; syntactic/logical/key depth64; persistent arena capacities64MiB; frame1MiB including28-byte header; source/cooked chunks64KiB; batches2048; Windows Job process committed-memory512MiB and kill-on-close. Request total2s TUI/5s inline includes syntax-reap wait, source transfer, parsing, result validation and process completion; progress never renews total. UI/setters do no pipe IO or join. Lease reader precedes initialization; syntax must yield a collected ticket before parser launch. Native Unix memory containment remains unsupported/pending; this evidence is Windows only.

Build fix discovered by the real exclusion scenario: rebuilding the default SyntaxSet plus TOML at runtime consistently crossed the existing500ms Init ACK deadline and yielded LineTimeout. build.rs now generates the COMPLETE compressed SyntaxSet with host Fancy; runtime Onig deserializes the embedded bundle without YAML, catalog rebuild or file access. Deadlines unchanged. Syntect5.3 serializes Regex as its pattern string (local installed source src/parsing/regex.rs92–109); runtime Onig compilation and role result tested below. cargo tree --locked -e normal shows syntect/onig/onig_sys and no yaml-rust/fancy-regex in the normal graph.

Final private runner .tmp/s03-d1-functional/run.py, machine-readable RESULTS-R1.json: all8 cases exit0:

- normal: decoded Unicode cooked string120000 bytes spans64KiB transport chunks;2501 entries crossing2048 batch; same source Arc; final leaf2499;851968 arena bytes;351ms projection/result elapsed.
- blocked: fake child (test-local harness only) stops reading after Init ACK with2MiB source, causing parent IO write/ACK to block; local cancel+kill/wait/join9ms; no Ready; global flag false; next valid generation Ready on same client.
- badexit: syntactically complete empty arena followed by worker exit77 is WorkerExited, no Ready; Unavailable latch prevents retry same snapshot; ticket Reaped.
- extra: valid arena plus extra End is InvalidIpc, no Ready; snapshot latch and Reaped.
- replace:101 rapid document/request generations; newest Ready without request resend;56ms.
- exclusion: actual Onig syntax worker TOML Ready74ms; integer has Process semantic role; syntax.pause_document ticket then actual parser Ready, collected ticket, parser Reaped then same-generation syntax resume Ready; total202ms.
- sibling-lease: directly spawned private worker and unrelated OWN sibling; closing worker pipes terminates worker38ms while sibling lives. Sibling cleanup only its own handle. No leaked lease through std::Command handle inheritance in this scenario.
- parent-loss-before-job: directly spawned private worker; harness parent exits immediately before any Job assignment/Init; opened own child handle signals termination35ms, below2s deadline. No absolute atomic-creation/orphan guarantee is claimed.

Seven normal cases were run again after both final fixes. Full workspace B had lib94/94 + CLI18/18 + all-target check + nu-check before these last bundle/launch-error changes; Root independently had145/145 workspace tests and build before them. Root will repeat final fmt/Clippy/workspace/build on this frozen candidate. Do not attribute those earlier global passes to the final bundle revision.

The physical index/depth viability gate remains15/15 independently approved. Production unit tests added by A cover model provenance/cooked UTF8 chunks/malformed topology/depth/cancel; protocol bounds/truncation; TOML declaration order/AoT/CRLF/raw lexemes/depth. Grammar R4 asset remains the independently reviewed derivative; ownership/licence/pin/transformation hashes are in resources/syntax/toml/MANIFEST.md.

A production signatures are frozen. All source hashes in HASHES-R1.json; temporary source/runner/binary hash below. B's UI/CLI/source files were not edited by A. No visual/manual TUI QA was claimed. Wide-map physical resolution is linear per path component and may reach the strict total deadline on large costly inputs; fallback keeps the complete source, and no deadline was relaxed.
Functional executable SHA256: A8C3BBDFFA9EBF8B6356D14B76FB99AE5FB25FB8BAD37DFE60EE70AD95DE1B23
