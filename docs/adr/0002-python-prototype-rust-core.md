# ADR-0002: Python end-to-end spike; Rust remains the target reference core

**Status:** Accepted for v0.1 prototype only. The Rust kernel subsequently merged in [PR #2](https://github.com/jmsbooth/arcxiom/pull/2). The following records the original prototype decision. See the [current kernel](../rust-archir.md).

## Context

The current bootstrap contains no executable implementation. A short, testable language-to-graph-to-PDF path is useful for validating operator workflows and source semantics before committing to Rust crate boundaries and a formally versioned ArchIR.

## Decision

Use a small standalone Python 3.11+ CLI with PyYAML and ReportLab, accompanied by deterministic golden behavior and negative tests. Keep the machine-readable interchange format language-neutral; make no claim that the Python API constitutes the normative standard. Implement a Rust-first production parser/graph toolchain in M1 after the grammar and type/profile RFCs settle. This spike can remain as an independent cross-language conformance test fixture.

## Consequences

Fast executable validation of the draft format, diagram rendering and CLI, but no Rust packaging, formal CAD conformance, production security guarantees or full YAML 1.2 fidelity. Do not represent a green Python test run as Rust implementation readiness.
