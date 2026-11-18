# Arcxiom

**Architecture, defined as code.**

Arcxiom is an experimental, vendor-neutral Architecture-as-Code project for modeling, querying, validating and rendering architectures across software, hardware, systems-of-systems, services, organizations, actors, missions and business capabilities.

**Published baseline: v0.1.0a1. Next release candidate: v0.2.0a1 (Rust-Native Architecture Kernel, in draft PR).** The .arch.yaml language and ArchIR remain **experimental 0.1-draft contracts**, not a completed standard or certified security control. Rust is becoming the authoritative source compiler; the shipped Python package remains the reference/compatibility implementation and the generic PDF adapter. The release candidate is not yet approved or published.

## Run the published Python v0.1.0a1 reference

Python 3.11+ is required. Install in a virtual environment:

```bash
python3 -m venv .venv
source .venv/bin/activate
python -m pip install -e '.[dev]'
arcxiom validate examples/emergency-response.arch.yaml
arcxiom query examples/emergency-response.arch.yaml --id service.dispatch-api --depth 3
arcxiom export examples/emergency-response.arch.yaml --format json --output archir.json
arcxiom render examples/emergency-response.arch.yaml --format pdf --output architecture.pdf
arcxiom diff examples/emergency-response.arch.yaml examples/emergency-response.arch.yaml --json
pytest -q
```

`python -m arcxiom` and `arch` are CLI aliases of `arcxiom`. The repository includes a complete cross-domain example. The PDF is vector-based, searchable, and explicitly labeled **SCHEMATIC / NOT TO SCALE**. It is not a CAD deliverable or an ISO, NCS, or BS compliant drawing set.

## Minimal YAML draft

```yaml
version: "0.1-draft"
namespace: sample
entities:
  - id: service.api
    type: service
    name: API service
  - id: capability.response
    type: capability
relationships:
  - from: service.api
    relation: supports
    to: capability.response
```

The prototype checks type membership, duplicate IDs, references, a small rule grammar, and hard constraints. Security properties in YAML are **assertions**, not verified telemetry. Unsupported syntax, drawing profiles, and imports fail explicitly rather than being silently ignored.

## Repository organization

- `rust/arcxiom-ir/`: Rust-native alpha architecture compiler, typed graph, canonicalization and portable CLI.
- `src/arcxiom/`: Python 0.1 reference parser, graph utilities, CLI, and standards-neutral PDF schematic adapter.
- `tests/`: runnable positive, negative, security and deterministic-output tests.
- `examples/`: executable `.arch.yaml` files.
- `spec/`: evolving draft contract, separate from any implementation.
- `docs/adr/`: architectural decisions, including the temporary Python-prototype choice.
- `docs/standards-registry.md`: planned standards profiles and rights gates.

## Architecture direction

- **Authoring:** human-readable `.arch.yaml` using a strict YAML subset; a future `.arch` shorthand must be compatible.
- **Interchange:** versioned ArchIR graph with deterministic JSON and SHA-256 content digest.
- **Model:** typed entities, directed relationships, requirements/constraints and future evidence links.
- **Tooling:** `validate`, `query`, `export`, generic `render --format pdf`, and conservative `diff --check-guardrails` are implemented in limited form.
- **Deferred:** module imports, LSP, formal type profiles and semantics, verified evidence, baseline approvals, agent MCP, physical geometry, multi-sheet technical drafting, ISO/BS/NCS conformance and independent implementation testing.

## Rust-native kernel (v0.2.0a1 candidate)

The Rust 1.89 crate at [rust/arcxiom-ir/](rust/arcxiom-ir/) builds a portable `arcxiom` executable with native YAML compilation, typed graph validation, deterministic canonical ArchIR, digest verification, query traversal and semantic diff. This is the target implementation for the next alpha, **not yet a published release**.

```bash
cargo build --locked --release --manifest-path rust/arcxiom-ir/Cargo.toml
rust/arcxiom-ir/target/release/arcxiom validate examples/emergency-response.arch.yaml --json
rust/arcxiom-ir/target/release/arcxiom export examples/emergency-response.arch.yaml -o archir.json
rust/arcxiom-ir/target/release/arcxiom query examples/emergency-response.arch.yaml --id role.dispatcher --depth 8 --json
rust/arcxiom-ir/target/release/arcxiom diff examples/emergency-response.arch.yaml examples/emergency-response.arch.yaml --check-guardrails --json
cargo test --locked --manifest-path rust/arcxiom-ir/Cargo.toml --all-targets
```

Windows builds produce `arcxiom.exe`. Existing `python -m arcxiom` and the Python wheel still expose the original v0.1.0a1 reference behavior and PDF rendering; they are **not aliases for the Rust executable**. For the draft grammar and canonicalization rules, see [the versioned compatibility contract](spec/archyaml-v0.1-compat.md). For explicit release blockers and exclusions, see [v0.2.0a1 release notes](docs/releases/v0.2.0a1.md). The Rust compiler does not run Python.

## Initial alpha distribution

The `v0.1.0a1` release candidate is built as a Python wheel and installed into a fresh virtual environment during CI. A passing test suite alone is insufficient for release. See [alpha publication gates](docs/release-candidate.md); merge, tag and GitHub prerelease publication require explicit maintainer approval. No PyPI or crates.io distribution is planned for this first alpha.

## License and standards rights

The source repository contains an Apache-2.0 `LICENSE` inherited from the initial GitHub repository. The open specification's long-term publication and contribution terms are not yet finalized.

Third-party standards such as ISO 128, ISO 5457, ISO 7200, BS EN ISO 19650 and NCS/UDS have distinct rights and applicability constraints. Do not copy their normative text, symbol sets, licensed tables or graphical assets into this repository without documented permission. No conformance or certification claim is made. See `docs/standards-registry.md`.

## Development and planning

- [Contribution guide](CONTRIBUTING.md)
- [Prototype limits and security posture](docs/prototype.md)
- [Threat model and release gates](docs/security.md)
- [Alpha release checklist](docs/release.md)
- [Linear roadmap](https://linear.app/lattix/project/architecture-as-code-arch-open-standard-and-toolchain-91615bff7f0d)

Arcxiom is an independently governed open-source effort. Commercial or proprietary modeling workbenches can consume its interchange format but do not define its authoritative semantics.
