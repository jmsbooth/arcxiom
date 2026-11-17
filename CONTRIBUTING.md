# Contributing to Arcxiom

Arcxiom is in experimental v0.1-alpha development. Open an issue or RFC before introducing new domain entities, language grammar, interchange shapes, or standards-derived drawing rules.

1. Fork and branch from `main`, make a scoped change, and submit a pull request.
2. Install the development extras and run `python -m pytest -q` plus the documented CLI smoke tests.
3. Include positive and negative tests, an example and update relevant docs when changing semantics.
4. Avoid private Lattix dependencies, secrets, unlicensed standards text, proprietary drawing symbols and AI-generated standards claims.
5. Document schema migrations, backward compatibility, security consequences, failure modes and limitations.
6. Treat the CI workflow as a baseline, not evidence of external standards conformance or production readiness.

Formal governance, contribution policy, decision authority and Rust MSRV will be established by the M0/M1 workstreams.
