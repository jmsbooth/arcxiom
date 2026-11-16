# ADR-0001: YAML-first authoring with canonical ArchIR

**Status:** Proposed

## Context

Arcxiom requires human-readable, agent-usable architectural models spanning multiple disciplines without tying interchange to a single source syntax or proprietary workbench.

## Proposal

Start with strict `.arch.yaml` files for authoring. Compile to a typed, versioned canonical ArchIR graph with deterministic JSON interchange. A future `.arch` shorthand must preserve semantic equivalence and migration guarantees.

## Not decided

- Exact YAML feature subset and duplicate-key handling
- Stable identity and canonical hash requirements
- Type and extension registry governance
- Unit/dimensional semantics and physical-coordinate systems
- Graph query and constraint language
- Versioning and signature format

## Constraints

Do not infer measured geometry from schematic topologies. Do not let agents alter approved guardrail baselines without separate authorization.
