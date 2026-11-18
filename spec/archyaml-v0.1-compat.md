# Arcxiom Rust kernel: draft 0.1 compatibility profile

Status: **experimental; pre-standard; implemented by the merged Rust kernel**. This profile freezes the existing published `0.1-draft` source and interchange contract for the Rust-first release. The release version is **not** the ArchIR version. Unsupported versions are rejected; no implicit migration occurs.

## Grammar and accepted subset

A single UTF-8 `.arch.yaml` document (at most 1 MiB) contains the required string fields `version: "0.1-draft"` and `namespace`, and arrays `entities`, `relationships`, and `constraints` (last two optional). Root fields not listed here are errors. Namespaces match `^[a-z][a-z0-9_-]{0,63}$`; IDs match `^[A-Za-z][A-Za-z0-9._-]{0,127}$`. Maximum counts: 1,000 nodes; 4,000 edges; 500 constraints. Depth: 20.

The source subset supports YAML block and flow mappings/sequences, plain and quoted UTF-8 strings, null, true/false, and decimal signed 64-bit / unsigned 64-bit integers. Explicit tags, anchors, aliases, merge keys, duplicate map keys, non-string keys, multiple YAML documents, binary values, and floats are errors. YAML 1.1 booleans (`on`, `yes`, `off`) remain strings. To avoid ambiguous YAML dialect conversion, non-decimal integer forms, leading-zero integers, non-finite numbers, and implicit timestamps are not coerced into numeric or date types. Authors should quote strings that resemble alternate numeric or special values.

Inputs with symlinks in their path components are rejected; imports and external references are outside this profile. The compiler does not follow user-supplied paths contained in attributes.

## ArchIR 0.1-draft and canonical digest

A successful export contains exactly `archir_version`, `namespace`, `entities`, `relationships`, `constraints`, and optional `sha256`. Entity shape: `{id,type,name,attributes}` where omitted `name` defaults to ID and additional source fields are preserved verbatim in `attributes`. Relationship shape: `{from,relation,to}`. Constraint shape: `{id,select,must,missing_evidence,enforcement}` with defaults `fail` and `hard`. Unmapped constraint `description` is intentionally rejected by the new compiler to avoid silent metadata loss; existing published example does not use it.

Canonical JSON is recursive, compact (no insignificant whitespace), with object keys sorted by Unicode scalar value, arrays in normalized order, `ensure_ascii` escaping (U+007F and non-ASCII code points as lowercase `\\u` escapes, supplementary code points as surrogate pairs), JSON boolean/null literals and base-10 integer tokens, and no floating-point values. Nodes sorted by ID, relationships by `(from,relation,to)`, rules by ID. SHA-256 is calculated from the UTF-8 encoding of this JSON **without** the `sha256` key. The in-memory IR may omit the digest until export; the `verify` command requires an explicit 64-hex-character `sha256` claim and rejects missing, malformed, or mismatched claims. A digest alone does not authenticate authorship. This matches the Python `json.dumps(sort_keys=True, ensure_ascii=True, separators=(",", ":"))` profile used by v0.1.0a1.

## Semantic constraints and relation typing

Entity `type` and relation `relation` use the frozen registries of `src/arcxiom/model.py`. Entity IDs are unique within the namespace and all edge endpoints must exist. The following relation domains/ranges are additionally enforced in Rust: `carries` requires data_flow -> data_asset; `flows_to` must involve a data_flow at one endpoint; `operates` originates at an actor/role/organization; `implements` originates at a service/API/application/software/component; `realizes` targets a capability; and `supports` may target a mission/objective/capability. Remaining registered relations currently have endpoint-existence and vocabulary checks only, not unrestricted extensibility. Cross-domain relation schemas beyond this subset remain future work.

Data-flow `source` and `destination`, if present, must reference existing entities; `carries` must contain only existing data_asset IDs. A rule has a unique ID, `select.type` and optionally `select.carries_classification`, nonempty `must` and valid policy modes. Hard + missing_evidence:fail rejects both empty selections and unmet required attributes; advisories produce warnings, not observed security evidence.

## Diagnostics and CLI compatibility

Diagnostics are machine-readable `{code,severity,message,path}` records, sorted by severity and path; `path` is an architectural property path, not a file-system authorization claim. The CLI exposes `validate`, `export`, `query`, `diff`, and a backward-compatible `verify` for v0.1 ArchIR. Exit 0 is success / no diff; 2 is invalid source or guardrail violation; 3 means a valid semantic diff; 64 is command-line usage error. `--json` is supported where documented. PDF generation remains in Python as an adapter.

## Conformance boundary

The frozen emergency-response fixture digest in `tests/fixtures/emergency-response.digest` is the authoritative v0.1 compatibility golden. Cross-platform Rust/Python parity, malicious input tests, and supply-chain review are **release gates**, not assumptions. The compiler interprets declared architecture facts; no runtime policy enforcement, standards compliance, baseline authenticity, or evidence provenance is implied.

The [a1-to-a2 producer migration](../docs/releases/v0.2.0a2.md#migration-from-the-a1-rust-producer) covers a Rust discrepancy for U+007F. The published canonicalization profile and ArchIR edition remain stable.
