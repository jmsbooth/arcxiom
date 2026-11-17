# Security boundary and threat model: v0.1.0a1

**Release classification:** experimental, single-user local reference tool. Not an authenticated, multi-tenant policy decision point, secure provenance engine, or verified compliance service.

## Assets and trust boundaries

1. **Architecture source** is untrusted YAML. `StrictLoader` rejects custom tags, duplicate mapping keys, aliases and anchors. Input size, parser nesting, entity, edge and constraint counts are bounded. Source text may still contain hostile metadata; do not execute it.
2. **Graph output** is a normalized representation of assertions in source. An attribute such as `transport_encrypted: true` does not demonstrate a live cryptographic control. The first prototype does **not** ingest independent runtime evidence.
3. **Baseline files** for `diff --check-guardrails` are *caller-supplied, unauthenticated input*. The check detects removal or alteration of hard rules and changes to classified data assets, but it does not authenticate approval, attest provenance, prevent a malicious actor replacing both files, or authorize promotion.
4. **PDF rendering** is an offline, bounded and standards-neutral schematic. It refuses unsupported fonts and standard profiles, renders no arbitrary user media, is not to scale and is not ISO/BS/NCS-compliant. Do not use it as an engineering construction drawing.
5. **Filesystem** is a local-user trust domain. Symlink input/output paths are rejected as a prototype boundary, not a complete mitigation against TOCTOU or directory symlink races. Run on a trusted workstation against trusted workspace permissions; do not expose directly as a public service.
6. **Third-party dependencies and standards** have distinct licensing requirements. The Apache-2.0 repository license does not grant rights to reproduce ISO, BSI, NIBS or other proprietary rules, symbols or figures.

## Non-goals and unsolved threats

- Trusted identity, approvals, immutable baselines, signed assertions and independent attestation.
- End-to-end validation of physical geometry, drawing scale and profile-specific CAD/technical specifications.
- Untrusted network import resolution, remote package registries and federation.
- Production-grade parser sandboxing, denial-of-service resilience under hostile multi-tenant workloads and supply-chain attestation.
- Independent security review; a green CI matrix alone is not that review.

## Release gate

For the public **initial prototype**, require current-head CI across OS/Python matrix, local regression and attack-case evidence, independent manual source review or an explicitly approved documented exception, a reproducible clean-install recipe, truthful version/feature matrix and a maintained known-limitations statement. This gate is not a production security certification.

To report a vulnerability, open a private GitHub security advisory for the repository when available. Do not include exploitation details in public issues until the maintainer can coordinate handling.
