# v0.1.0a1 initial prototype release contract

This is an **alpha source release**, not a finalized industry standard, certified architectural tool or production security boundary.

## Demonstrated usage

```bash
python -m pip install -e '.[dev]'
python -m pytest -q
arcxiom validate examples/emergency-response.arch.yaml --json
arcxiom export examples/emergency-response.arch.yaml --output archir.json
arcxiom query examples/emergency-response.arch.yaml --id service.dispatch-api --depth 4 --json
arcxiom render examples/emergency-response.arch.yaml --format pdf --output schematic.pdf
arcxiom diff examples/emergency-response.arch.yaml examples/emergency-response.arch.yaml --json
arcxiom diff baseline.arch.yaml proposed.arch.yaml --check-guardrails --json
```

`diff` returns 0 for semantic equality, 3 for architectural changes, and 2 for invalid documents, incompatible namespaces/versions or rejected guardrails. The guarded mode does not authenticate the baseline or grant approval.

## Verification matrix

- Validation diagnostics and YAML source controls: automated positive and adversarial tests.
- Canonical ArchIR digest: stable across input list reorder; frozen reference digest fixture.
- JSON Schema: structural validation of the draft interchange (not full semantic proof).
- Graph traversal: bounded and typed by named relationships.
- Diff/guardrails: changed/added/removed elements; immutability check for pre-existing hard constraints; review requirement for classification changes.
- Generic PDF: visually inspect rendered pages and extract selectable text, labels and direction markers. ISO/BS/NCS claims explicitly prohibited.
- Linux/macOS/Windows with Python 3.11 and 3.13: GitHub Actions matrix for the current release commit.

## Publication prerequisites

1. Exact commit SHA and six successful current-head CI jobs.
2. Local test log and generated PDF verified visually.
3. Independent review or documented release exception explicitly approved by repository owner.
4. License/dependency review and release metadata; protect `main` against unchecked pushes where GitHub plan permits.
5. Merge the reviewed PR, then tag `v0.1.0a1` from the immutable merged commit. Create a clearly marked prerelease; do not publish crates.io or PyPI without separate authorization.
