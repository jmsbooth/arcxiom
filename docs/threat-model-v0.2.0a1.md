# Rust Architecture Kernel threat model and security review checklist

Status: security review draft for FOSS-587/FOSS-588/FOSS-590; not an independent approval.

## Trust boundary

The .arch.yaml file, its file path, ArchIR JSON supplied to verify, command-line flags, and output destinations are untrusted. The Rust process runs with the privileges of the invoking user. The parsed graph, its constraints, and any security classifications are **asserted descriptions**, not authenticated runtime evidence. Data leaves the trust boundary only via terminal diagnostics/JSON, optional output files, or OS-level process exit.

**Assets:** integrity of canonical IR and its digest; deterministic graph interpretation; filesystem integrity; process availability; release artifact provenance. SHA-256 proves bitwise equality to a given JSON document; it does not prove authorship, approval, or correctness.

## Threats and controls

| Threat | Planned mitigation and tests | Residual risk |
| --- | --- | --- |
| YAML aliases, anchors, merge keys, tags and expansion | Lexical rejection before deserialization; duplicate-key rejecting deserializer; adversarial fixtures | Lexical screening is deliberately conservative and may reject some legitimate text; parser differential fuzzing and additional syntax tests needed |
| Unbounded inputs, recursive composites, memory exhaustion | 1 MiB source limit, 20 nesting limit, 1000 entities, 4000 relationships, 500 rules, bounded maps/sequences | YAML parse cost can still vary; measure resource use under fuzzing |
| Ambiguous scalar interpretation | Reject floating-point, out-of-profile integers, non-string map keys; reject silent coercion | Future YAML 1.2 grammar work needed for edge-case implicit scalars |
| Graph hijack / dangling references | Global unique IDs, bounded vocabulary, typed selected edges, data_flow reference validation | Remaining relation domain/range rules are intentionally permissive |
| Rule downgrade | Hard constraints with missing evidence fail; offline diff detects hard rule removal/mutation and data classification changes | Supplied baseline is **not authenticated**, runtime controls are **not observed**, and advisory rules are non-blocking |
| Symlink and path races | Reject symlink input components and symlink output files; no imports/remote reads | Filesystem check-then-open race remains on adversarial multi-user systems; OS handle-based no-follow opening is required before high-trust deployment |
| Hash confusion / downgrade | Frozen ArchIR version, deterministic ordering, digest excludes only sha256, mismatched digest fails | SHA-256 is not a signature; callers must authenticate provenance out of band |
| Dependency compromise / malicious CI | Rust lockfile, pinned core CI actions, least-privilege CI, no release on PR pushes, maintainer-only explicit release draft | Full dependency vulnerability, license and transitive audit must be signed off before publication |
| Artifact substitution | Candidate archive and per-file checksums; independent checksum verification in release packaging | Checksum file and artifact must be distributed/authenticated through trusted channel; attested provenance is not yet delivered |

## Required independent checks

1. Review entire Rust parser, canonicalization and graph semantics with the published Python reference and negative fixtures.
2. Run compiler fuzz/property inputs and cross-platform memory/CPU limits; inspect failures, not just elapsed time.
3. Inspect dependency inventory and vulnerabilities (RustSec advisory database) and validate Apache-2.0 compatibility.
4. Verify published emergency-response digest, additional multi-domain fixture, and no-output-on-error invariants.
5. Confirm that the default branch requires review and successful checks; restrict bypass; configure an arcxiom-release environment with required reviewers.
6. Sign off the final SHA and archive SHA256SUMS prior to manually publishing the draft prerelease.

No security assurance or release approval should be inferred from this document alone.
