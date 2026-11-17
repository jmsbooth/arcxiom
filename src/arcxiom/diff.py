"""Deterministic architecture diff and conservative draft baseline checks.

The 'approved' file is supplied by the caller and is NOT authenticated by this
module. Do not treat this as a promotion/authorization service.
"""

from __future__ import annotations

from dataclasses import dataclass


def _indexed(sequence: list[dict], key: str) -> dict[str, dict]:
    return {item[key]: item for item in sequence}


def _edge_id(edge: dict) -> str:
    return f"{edge['from']}:{edge['relation']}:{edge['to']}"


def compare_ir(before: dict, after: dict) -> dict:
    """Return an ordered JSON-compatible delta between two valid ArchIR values."""
    if before.get("archir_version") != after.get("archir_version"):
        raise ValueError("incompatible ArchIR versions")
    if before.get("namespace") != after.get("namespace"):
        raise ValueError("incompatible namespaces; migration is not a simple diff")
    sections = (
        ("entities", lambda e: e["id"]),
        ("relationships", _edge_id),
        ("constraints", lambda c: c["id"]),
    )
    changes = []
    summary = {}
    for section, key in sections:
        older = {key(item): item for item in before[section]}
        newer = {key(item): item for item in after[section]}
        counts = {"added": 0, "removed": 0, "modified": 0}
        for objid in sorted(set(older) | set(newer)):
            if objid not in older:
                action = "added"
            elif objid not in newer:
                action = "removed"
            elif older[objid] != newer[objid]:
                action = "modified"
            else:
                continue
            counts[action] += 1
            record = {"section": section, "id": objid, "action": action}
            if action != "added":
                record["before"] = older[objid]
            if action != "removed":
                record["after"] = newer[objid]
            changes.append(record)
        summary[section] = counts
    return {"archir_version": before["archir_version"], "namespace": before["namespace"],
            "changed": bool(changes), "summary": summary, "changes": changes}


@dataclass(frozen=True)
class Violation:
    code: str
    message: str
    object_id: str

    def as_json(self) -> dict:
        return {"code": self.code, "message": self.message, "object_id": self.object_id}


def check_guardrail_delta(approved: dict, proposed: dict) -> list[Violation]:
    """Fail closed on weakening an externally supplied, *unverified* baseline.

    The caller must obtain and authenticate the baseline separately. Rules only
    prove equality of asserted model fields, never truth of external evidence.
    """
    diff = compare_ir(approved, proposed)
    problems = []
    old_rules = _indexed(approved["constraints"], "id")
    new_rules = _indexed(proposed["constraints"], "id")
    for rid, rule in sorted(old_rules.items()):
        if rule["enforcement"] != "hard":
            continue
        if rid not in new_rules:
            problems.append(Violation("AXG001", "approved hard constraint removed", rid))
        elif new_rules[rid] != rule:
            problems.append(Violation("AXG002", "approved hard constraint modified", rid))
    old_nodes = _indexed(approved["entities"], "id")
    new_nodes = _indexed(proposed["entities"], "id")
    for nid, node in sorted(old_nodes.items()):
        if node["type"] != "data_asset":
            continue
        old_class = node["attributes"].get("classification")
        if old_class is None:
            continue
        new = new_nodes.get(nid)
        if new is None:
            problems.append(Violation("AXG003", "classified asset removed; review required", nid))
        elif new["type"] != "data_asset" or new["attributes"].get("classification") != old_class:
            problems.append(Violation("AXG004", "classification changed; review required", nid))
    return problems
