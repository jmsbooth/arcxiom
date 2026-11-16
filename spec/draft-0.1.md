# Arcxiom 0.1-draft experimental language contract

This document describes an **experimental executable subset**. It is not a published normative standard.

## Source

File suffix `.arch.yaml`, root mapping with `version`, `namespace`, `entities`, optional `relationships`, optional `constraints`. `version` must be exactly `0.1-draft`. Supported node IDs are 1-128 characters beginning with an ASCII letter and then ASCII letters, numbers, underscore, period or hyphen. Namespace is lowercase ASCII with hyphen or underscore.

## Entities and edges

Each entity has a unique `id`, a registered `type` and an optional display `name`. Additional JSON-compatible source fields become ArchIR `attributes`. A relationship has `from`, `relation`, `to` and must reference existing IDs. Prototype relation type checking is vocabulary-only: strict domain/range semantics are deferred.

The current type vocabulary is defined in `src/arcxiom/model.py`. This is a versioned experimental registry, not an extensible external ontology yet.

## Rules

Prototype constraints require unique `id`, `select` with `type` and optional `carries_classification`, `must` with exact attribute-value equalities, `enforcement` hard/advisory, and `missing_evidence` fail/warn. The `carries_classification` selector resolves the referenced data assets of `data_flow.carries`. Rules do not process arbitrary expressions, cryptographic attestations or outside observations.

## ArchIR

Output is sorted JSON with `archir_version`, `namespace`, `entities`, `relationships`, `constraints`. The `sha256` field from the `export` command is SHA-256 of the compact JSON map without the digest field. Canonical JSON field ordering and Unicode escaping are currently implementation-defined prototype contracts; a cross-language canonicalization specification and test corpus are still required.

## Restrictions

Aliases, merge keys, duplicate map keys, custom tags, implicit YAML 1.1 booleans, deep/unbounded structures, imports, arbitrary plugins and new standards-derived profiles are not supported. Inputs exceeding 1 MiB or 1000 entities are rejected. The `render` command emits a generic not-to-scale PDF for at most 20 ASCII-labeled nodes.
