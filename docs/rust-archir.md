Rust ArchIR checker (experimental)
====================================

The Rust 1.85 crate is an independent, offline structural verifier of the JSON produced by the Python Arcxiom prototype. It is **not** a YAML parser, policy authority, baseline approval mechanism, verified evidence engine, CAD exporter or ISO/BS/NCS conformance implementation.

Run `arcxiom export examples/emergency-response.arch.yaml --output example-archir.json`, then `cargo run --manifest-path rust/arcxiom-ir/Cargo.toml -- verify example-archir.json`.

The cross-language digest contract currently covers integer-valued JSON numbers. Rust refuses floats pending an independently specified numeric canonicalization. The GitHub Rust test matrix cross-checks against the frozen Python digest fixture. This is an experimental pre-standard alpha artifact.
