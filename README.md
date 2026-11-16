# Arcxiom

**Architecture, defined as code.**

Arcxiom is an experimental, vendor-neutral Architecture-as-Code project for modeling, querying, validating and rendering architectures across software, hardware, systems-of-systems, services, organizations, actors, missions and business capabilities.

**Current status: v0.1 alpha reference prototype.** The source format and ArchIR are **draft contracts**, not a published or independently conformant standard. This prototype is intentionally narrow and implemented in Python to validate semantics rapidly. A Rust reference core remains the target for production.

## Run the prototype

Python 3.11+ is required. Install in a virtual environment:

```bash
python3 -m venv .venv
source .venv/bin/activate
python -m pip install -e '.[dev]'
arcxiom validate examples/emergency-response.arch.yaml
arcxiom query examples/emergency-response.arch.yaml --id service.dispatch-api --depth 3
arcxiom export examples/emergency-response.arch.yaml --format json --output archir.json
arcxiom render examples/emergency-response.arch.yaml --format pdf --output architecture.pdf
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

- `src/arcxiom/`: Python prototype parser, ArchIR builder, graph query engine, CLI, and standards-neutral PDF schematic renderer.
- `tests/`: runnable positive, negative, security and deterministic-output tests.
- `examples/`: executable `.arch.yaml` files.
- `spec/`: evolving draft contract, separate from any implementation.
- `docs/adr/`: architectural decisions, including the temporary Python-prototype choice.
- `docs/standards-registry.md`: planned standards profiles and rights gates.

## Architecture direction

- **Authoring:** human-readable `.arch.yaml` using a strict YAML subset; a future `.arch` shorthand must be compatible.
- **Interchange:** versioned ArchIR graph with deterministic JSON and SHA-256 content digest.
- **Model:** typed entities, directed relationships, requirements/constraints and future evidence links.
- **Tooling:** `validate`, `query`, `export`, and generic `render --format pdf` are implemented in limited form.
- **Deferred:** module imports, LSP, formal type profiles and semantics, verified evidence, baseline approvals, agent MCP, Rust core, physical geometry, multi-sheet technical drafting, ISO/BS/NCS conformance and independent implementation testing.

## License and standards rights

The source repository contains an Apache-2.0 `LICENSE` inherited from the initial GitHub repository. The open specification's long-term publication and contribution terms are not yet finalized.

Third-party standards such as ISO 128, ISO 5457, ISO 7200, BS EN ISO 19650 and NCS/UDS have distinct rights and applicability constraints. Do not copy their normative text, symbol sets, licensed tables or graphical assets into this repository without documented permission. No conformance or certification claim is made. See `docs/standards-registry.md`.

## Development and planning

- [Contribution guide](CONTRIBUTING.md)
- [Prototype limits and security posture](docs/prototype.md)
- [Linear roadmap](https://linear.app/lattix/project/architecture-as-code-arch-open-standard-and-toolchain-91615bff7f0d)

Arcxiom is an independently governed open-source effort. Commercial or proprietary modeling workbenches can consume its interchange format but do not define its authoritative semantics.
