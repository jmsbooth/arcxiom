# Rust architecture kernel

The Rust 1.89 crate builds the authoritative single-file YAML compiler and ArchIR verifier. It supports `validate`, `export`, `verify`, `query` and `diff`. Python provides a separate compatibility implementation and the generic schematic PDF adapter. The merged a1 kernel and its public distribution have separate gates.

Build and run the Rust implementation from the repository root:

```sh
cargo build --locked --release --manifest-path rust/arcxiom-ir/Cargo.toml
rust/arcxiom-ir/target/release/arcxiom export examples/emergency-response.arch.yaml -o example-archir.json
rust/arcxiom-ir/target/release/arcxiom verify example-archir.json
```

The hash covers the normalized graph under the [0.1-draft compatibility profile](../spec/archyaml-v0.1-compat.md). See the [a2 qualification scope and producer migration](releases/v0.2.0a2.md) for the shared corpus and limits of the evidence.
