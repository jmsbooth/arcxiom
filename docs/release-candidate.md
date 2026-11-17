# Arcxiom v0.1.0a1 initial alpha publication gate

The release is an **experimental open-source reference prototype**, not a finalized Architecture-as-Code standard, an authoritative policy enforcement service, or standards-certified CAD software.

## Verified capabilities for the release candidate
- YAML-first Python 3.11+ command-line prototype: `validate`, `export`, `query`, conservative `diff`, and schematic vector `render --format pdf`.
- Draft ArchIR interchange with frozen deterministic golden digest and draft JSON schema.
- Separate Rust 1.85 command-line structural checker for **exported ArchIR JSON**, with cross-language digest parity tests on Ubuntu, macOS and Windows. Rust does **not** yet parse YAML or evaluate policies.
- The Python prototype documents that YAML security fields are assertions, not independently verified runtime evidence.
- ISO 128, ISO 5457, ISO 7200, BS EN ISO 19650, NCS and UDS remain unimplemented and cannot be claimed as supported.

## Publishing criteria
1. Current PR head has all six Python CI jobs, all three Rust CI jobs and wheel-distribution smoke test green.
2. Python wheel installs from distribution into a fresh environment and CLI `validate`, `export`, `render` run successfully.
3. Release source tree, Rust cross-language numeric limitations, security boundary, licensing and standards-rights register are reviewed.
4. An independent reviewer approves, **or the repo owner deliberately documents an exception**. CI cannot substitute for that decision.
5. GitHub repository branch protections and any applicable signing, review or artifact attestation requirements are checked by the owner. No tag or package upload before approved merge.
6. After accepted merge, create immutable `v0.1.0a1` tag and a GitHub **prerelease** from that commit only. There is no authorization to publish to PyPI/crates.io in this release task.

## Known unresolved gaps
- Cross-language JSON floating-point canonicalization is **not** specified, and Rust rejects floats.
- Strict untrusted JSON duplicate-key rejection and full domain-type semantics remain future qualification; Rust is a structural verifier for the normalized exporter path.
- Trustworthy evidence signatures, promotion authorization, resource-constrained multi-tenant parsing, architecture drift adapters, physical measurements and CAD-standard profiles are not implemented.
- Formal ISO/BS/NCS standards-rights review and independent implementation qualification remain outstanding.

## Release evidence record
Record final commit SHA, successful job URLs, wheel artifact hash, reviewer decision and any manual PDF inspection evidence here before a production release announcement.
