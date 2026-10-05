# R3Viewer vendored Saphyr parser R2

Saphyr-parser0.1.0 MIT OR Apache-2.0. LICENSE and all four verbatim .licenses files retain the notices of Chen Yuheng and Ethiraric; both authors notices required. UPSTREAM.json pins published crate/VCS/all original files and both modified source hashes. Original parser/scanner and manifests retained under upstream/. Local .patch files show source derivation.

Reviewed gate R2 adapts quoted/block lexical ranges, anchor scope in pull API, R3Viewer explicit-version1.2 profile, TAG directive accumulation/duplicate rejection and exact percent-escape identity. It does not certify full YAML conformance. Marker.index counts Unicode characters; caller must map validated UTF8 positions. Synthetic empty scalars do not own raw delimiter spans. Tag handle/suffix metadata is inert; no constructors or alias expansion.

Packaging-only Cargo.toml adds empty workspace to avoid application-workspace membership/dev dependencies. R3Viewer uses default-features=false, excludes this crate from its workspace, and pins it through patch.crates-io. The reviewed R2 candidate is integrated into the YAML worker; product acceptance, adversarial review, visual QA and platform certification remain separate gates. UPSTREAM.json retains the private packaging snapshot scope as historical provenance.
