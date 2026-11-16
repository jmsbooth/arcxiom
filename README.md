# Arcxiom

**Architecture, defined as code.**

Arcxiom is a proposed, vendor-neutral Architecture-as-Code specification and reference toolchain for modeling, exchanging, validating, rendering, and governing architectures across software, hardware, systems-of-systems, services, organizations, actors, missions, and business capabilities.

> Project status: **pre-specification / bootstrap**. No implemented compiler, renderer, or conformance claims exist yet.

## Design direction

- **Human authoring:** `.arch.yaml` using a deliberately constrained YAML 1.2 subset.
- **Future shorthand:** `.arch` may introduce YAML-compatible or YAML-like syntax, but must compile to the same semantics.
- **Canonical interchange:** versioned, deterministic JSON-based typed graph (`ArchIR`).
- **Tools:** CLI and APIs for `validate`, `query`, `diff`, `plan`, and `render`.
- **Rendering:** first a standards-neutral vector PDF drawing-set engine, then edition-locked optional profiles for ISO 128, ISO 5457, ISO 7200, BS EN ISO 19650, US NCS/UDS, and related standards where authorized.
- **Agents:** read-only architecture queries, scoped proposals, approval-separated policy enforcement, evidence-backed observed-state comparison.

Drawings generated from graph topology alone must be explicitly labeled *schematic / not to scale*. Physical geometry and construction documentation require authenticated measurements, units, design authority, and validation.

## Repository organization

- [`spec/`](spec/) - proposed normative semantics and conformance model.
- [`examples/`](examples/) - illustrative, not yet machine-validated models.
- [`docs/adr/`](docs/adr/) - architecture decisions.
- [`CONTRIBUTING.md`](CONTRIBUTING.md) - initial contribution expectations.

## First milestone

1. Approve the semantic kernel, representation boundaries, grammar, and version negotiation.
2. Establish rights/editions register for external standards before implementing standards-specific rules or assets.
3. Define a minimal v0.1 YAML sample and ArchIR JSON normalization contract.
4. Establish positive and negative conformance fixtures for a standalone validator.

## License and external standards

The project is intended to be open source. **A source license has not yet been selected. Do not treat this bootstrap as granting reuse rights until a LICENSE file is adopted.** Apache-2.0 is an option for the reference implementation; specification/IP governance must be decided explicitly.

Referenced ISO, BSI, NIBS and other standards are third-party works. Do not copy or encode restricted normative material, drawing symbols, tables, or assets without legal rights. This repository does not claim standards conformance or certification.

## Planning

Linear project: [Architecture-as-Code (.arch) - Open Standard & Toolchain](https://linear.app/lattix/project/architecture-as-code-arch-open-standard-and-toolchain-91615bff7f0d)

The specification and reference implementation should remain independent of proprietary Lattix products. Existing workbenches may adopt Arcxiom as consumers.
