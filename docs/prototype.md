# Functional prototype scope and safety boundaries

This prototype demonstrates a complete user path: `.arch.yaml` -> strict parse -> typed node/edge validation -> deterministic ArchIR JSON -> reachability query -> standards-neutral searchable/vector PDF schematic.

## Implemented

- YAML 1.2-like boolean handling; rejects duplicate map keys, anchors/aliases, merge keys, custom tags, nonfinite numbers and non-string keys. Max source size is 1 MiB and max 1000 nodes; nested data is bounded after parse.
- Known cross-domain entity types and relationship vocabulary; unique identities and resolution of explicit edges, plus `data_flow` source/destination/carries references.
- A deliberately small constraint shape: `select.type`, optional `select.carries_classification`, and exact `must` attribute equalities. Hard violations are errors; advisory violations are warnings. A constraint selecting no entities warns rather than silently claiming coverage.
- Stable machine-readable diagnostics, deterministic sort/serialization and SHA-256 over canonical JSON (a prototype digest, not yet a cryptographic signing contract).
- Incoming/outgoing/bidirectional bounded graph traversal.
- A3 landscape generic single-sheet PDF schematic with original layout, vector paths, selectable labels, draft title block, explicit scale and standards limitations. Deterministic bytes under the same renderer environment.

## Not yet implemented

- Rust production parser, all YAML 1.2 features, standard-specific drawing rules, font embedding/PDF-A/PDF-UA, CAD dimensioning, multi-sheet sets, physical coordinate systems or construction validity.
- Module/package imports, extension registry, formal profile typing and qualified names, stable cross-file IDs, internationalized fonts, query language, graph database, incremental compilation, drift observation and evidence attestations.
- Trusted approval state, exception cryptography or MCP agent write/promotion APIs. A declaration such as `transport_encrypted: true` is only an assertion by the YAML author, **not independently verified transport security**.
- Comprehensive resource exhaustion defense against all parser-level recursion cases; no untrusted multi-tenant service deployment should consume this prototype.

The generic renderer rejects requests for ISO, UK and NCS drawing/information profiles instead of pretending to implement them. For non-ASCII labels or more than 20 nodes it also fails explicitly. The graph layout does not represent distances, geometry or operational timings.

## Test and demo commands

```
python -m pytest -q
arcxiom validate examples/emergency-response.arch.yaml --json
arcxiom query examples/emergency-response.arch.yaml --id service.dispatch-api --depth 3 --json
arcxiom export examples/emergency-response.arch.yaml --format json -o /tmp/archir.json
arcxiom render examples/emergency-response.arch.yaml --format pdf -o /tmp/architecture.pdf
```
