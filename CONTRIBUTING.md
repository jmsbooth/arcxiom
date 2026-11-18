# Contributing to Arcxiom

Arcxiom remains experimental: v0.1.0a1 is published and the v0.2.0a1 Rust-native kernel is a draft release candidate. Open an issue or RFC before introducing new domain entities, language grammar, interchange shapes, or standards-derived drawing rules.

1. Fork and branch from `main`, make a scoped change, and submit a pull request.
2. Run `python -m pytest -q`, `cargo test --locked --manifest-path rust/arcxiom-ir/Cargo.toml --all-targets`, and the Rust/Python CLI smoke and parity checks. CI is authoritative for Windows/macOS/Linux qualification.
3. Include positive and negative tests, an example and update relevant docs when changing semantics.
4. Avoid private Lattix dependencies, secrets, unlicensed standards text, proprietary drawing symbols and AI-generated standards claims.
5. Document schema migrations, backward compatibility, security consequences, failure modes and limitations.
6. Treat the CI workflow as a baseline, not evidence of external standards conformance or production readiness.

**Write/merge authority is restricted.** Public contributors may fork, open issues, and submit pull requests but are not automatically granted collaborator permissions, push access, Actions secrets, branch bypass, or release authority. The sole maintainer must explicitly approve workflow execution for PRs originating from external forks before GitHub runs their required checks. The repository currently uses a temporary solo-maintainer profile: `jmsbooth` remains the sole authorized maintainer, all 11 required CI checks and PRs remain mandatory, and no external approval is required until an actual independent trusted maintainer is added. All releases require separate owner-initiated preparation, an environment approval action, and manual publication. See `docs/release-governance-v0.2.0a1.md` for the controls and reduced-assurance caveat.
