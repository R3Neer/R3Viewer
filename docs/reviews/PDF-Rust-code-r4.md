# PDF Rust R4 — independent code review

SIN_CRITICAS_PENDIENTES for the private gate delta; independent acceptance still required.

Frozen R4 MANIFEST283D501428075005723D6476D646D18571260B1BBFC78986CF27E941D72B6381 with949entries. Root is distinct from author. Own repro verifies all frozen entries before and after execution. Rust lib/main/Cargo/lock/executable/DLL compared byte-exact against R3; no native ABI changes to previous reviewed code. Executable C8C6717A2161E23ED1A7B0D548C6008CB07DEC80F36D28FFC1A46B0E4C7D7B73.

PRUST-ROOT02 was reproduced independently in R3: a reader blocked at EOF until cleanup join and then raised; R3 returned validated_response with reader.error set, resources closed, errors empty. Source-derived root script/result preserved in pdf-root-late-capture-probe-r3.py and -results.json, no rescue.

R4 samples capture error/overflow after reader/writer joins. Writer catches BaseException in write/flush and separately in finally stdin.close, latching both before final validation. Successful cleanup cannot erase execution errors. Failed wait/resource proof retains authority Pending and blocks subsequent launches. Publication occurs only after cleanup error check; a live reader cannot provide complete receipt. Native process and Job closures still have separate proof requirements.

Own R4 replay uses the same LateEOF barrier, with explicit expectation that correction rejects publication. Result: RuntimeError includes late OSError; validated_response absent; state Reaped, successful wait receipt, all three streams/process handle closed, both readers joined; no external rescue. Root script/result pdf-root-late-capture-probe-r4.py and -results.json outside Git. 949 records verified before/after. Independent AE is responsible for broader writer/lateoverflow/control replay, not inferred from this single repro.

This resolves ROOT02 within the supervisor gate. Native indexed extraction still has semantic_state Unverified; astral glyphs and single-TJ truncation original failures remain. No claims of semantic PDF completeness, full resource limits/Unix/visual/product PDF functionality or RV-204 closure. R1/R2/R3 rejected snapshots and traces remain preserved.
