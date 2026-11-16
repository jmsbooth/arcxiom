"""Strict YAML draft parsing, typed graph validation, and deterministic ArchIR."""

from __future__ import annotations

from collections import deque
from dataclasses import asdict, dataclass
from hashlib import sha256
from pathlib import Path
import json
import math
import re

import yaml

DRAFT_VERSION = "0.1-draft"
MAX_BYTES = 1_048_576
MAX_ENTITIES = 1_000
MAX_DEPTH = 20
IDENT = re.compile(r"^[A-Za-z][A-Za-z0-9._-]{0,127}$")
NAMESPACE = re.compile(r"^[a-z][a-z0-9_-]{0,63}$")
TYPES = frozenset({
    "mission", "objective", "capability", "organization", "actor", "role",
    "business_process", "business_service", "system", "system_of_systems",
    "subsystem", "software", "application", "service", "component", "api",
    "interface", "data_asset", "data_flow", "event", "network", "hardware",
    "device", "facility", "location", "requirement", "control", "risk", "decision",
})
RELATIONS = frozenset({
    "contains", "realizes", "supports", "implements", "depends_on", "operates",
    "invokes", "connects_to", "deployed_on", "governs", "carries", "flows_to",
    "verified_by", "assigned_to", "exchanges_with",
})


class SourceError(Exception):
    def __init__(self, code: str, message: str):
        super().__init__(message)
        self.code = code


class StrictLoader(yaml.SafeLoader):
    """No aliases, merge keys, custom tags, duplicate keys or YAML 1.1 booleans."""

    yaml_implicit_resolvers = {
        key: [(tag, pattern) for tag, pattern in entries
              if tag not in {"tag:yaml.org,2002:bool", "tag:yaml.org,2002:timestamp"}]
        for key, entries in yaml.SafeLoader.yaml_implicit_resolvers.items()
    }

    def compose_node(self, parent, index):
        if self.check_event(yaml.AliasEvent):
            raise SourceError("AXY003", "YAML aliases are not supported")
        return super().compose_node(parent, index)

    def construct_mapping(self, node, deep=False):
        if not isinstance(node, yaml.MappingNode):
            raise SourceError("AXY001", "expected a mapping")
        mapping = {}
        for key_node, value_node in node.value:
            if key_node.tag == "tag:yaml.org,2002:merge":
                raise SourceError("AXY004", "YAML merge keys are not supported")
            key = self.construct_object(key_node, deep=deep)
            if not isinstance(key, str):
                raise SourceError("AXY005", "mapping keys must be strings")
            if key in mapping:
                raise SourceError("AXY002", f"duplicate YAML key: {key}")
            mapping[key] = self.construct_object(value_node, deep=deep)
        return mapping


StrictLoader.add_implicit_resolver(
    "tag:yaml.org,2002:bool", re.compile(r"^(?:true|True|TRUE|false|False|FALSE)$"),
    list("tTfF"),
)


@dataclass(frozen=True)
class Diagnostic:
    code: str
    severity: str
    message: str
    path: str

    def as_json(self) -> dict:
        return asdict(self)


@dataclass
class Compiled:
    ir: dict
    diagnostics: list[Diagnostic]

    @property
    def valid(self) -> bool:
        return not any(d.severity == "error" for d in self.diagnostics)

    @property
    def digest(self) -> str:
        serialized = json.dumps(self.ir, sort_keys=True, ensure_ascii=True, separators=(",", ":"))
        return sha256(serialized.encode("utf-8")).hexdigest()


def _value_is_json(value, depth=0) -> bool:
    if depth > MAX_DEPTH:
        return False
    if value is None or isinstance(value, (str, bool, int)):
        return True
    if isinstance(value, float):
        return math.isfinite(value)
    if isinstance(value, list):
        return all(_value_is_json(item, depth + 1) for item in value)
    if isinstance(value, dict):
        return all(isinstance(k, str) and _value_is_json(v, depth + 1)
                   for k, v in value.items())
    return False


def read_source(path: str | Path) -> dict:
    path = Path(path)
    if path.is_symlink():
        raise SourceError("AXY010", "symlink input is not supported")
    try:
        if path.stat().st_size > MAX_BYTES:
            raise SourceError("AXY011", "input exceeds 1 MiB prototype limit")
        content = path.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as exc:
        raise SourceError("AXY012", f"cannot read UTF-8 source: {exc}") from exc
    try:
        loaded = yaml.load(content, Loader=StrictLoader)
    except SourceError:
        raise
    except RecursionError as exc:
        raise SourceError("AXY008", "YAML nesting exceeds parser limits") from exc
    except yaml.YAMLError as exc:
        raise SourceError("AXY001", f"invalid YAML: {exc}") from exc
    if not isinstance(loaded, dict):
        raise SourceError("AXY006", "document root must be a mapping")
    if not _value_is_json(loaded):
        raise SourceError("AXY007", "invalid scalar, depth or non-finite numeric value")
    return loaded


def compile_document(root: dict) -> Compiled:
    diagnostics: list[Diagnostic] = []

    def emit(code: str, message: str, path: str, severity: str = "error"):
        diagnostics.append(Diagnostic(code, severity, message, path))

    for key in sorted(set(root) - {"version", "namespace", "entities", "relationships", "constraints"}):
        emit("AXS001", f"unsupported top-level field: {key}", key)
    version = root.get("version")
    namespace = root.get("namespace")
    if version != DRAFT_VERSION:
        emit("AXS002", f"expected version {DRAFT_VERSION!r}", "version")
    if not isinstance(namespace, str) or not NAMESPACE.fullmatch(namespace):
        emit("AXS003", "namespace must be lowercase alphanumeric with - or _", "namespace")
    entities = root.get("entities", [])
    edges = root.get("relationships", [])
    constraints = root.get("constraints", [])
    for field, items in (("entities", entities), ("relationships", edges), ("constraints", constraints)):
        if not isinstance(items, list):
            emit("AXS004", f"{field} must be an array", field)
    if not isinstance(entities, list):
        entities = []
    if not isinstance(edges, list):
        edges = []
    if not isinstance(constraints, list):
        constraints = []
    if len(entities) > MAX_ENTITIES:
        emit("AXS005", f"too many entities (max {MAX_ENTITIES})", "entities")
    nodes_by_id = {}
    for idx, item in enumerate(entities[:MAX_ENTITIES]):
        loc = f"entities[{idx}]"
        if not isinstance(item, dict):
            emit("AXE001", "entity must be an object", loc)
            continue
        eid, kind = item.get("id"), item.get("type")
        if not isinstance(eid, str) or not IDENT.fullmatch(eid):
            emit("AXE002", "invalid entity id", loc + ".id")
            continue
        if eid in nodes_by_id:
            emit("AXE003", f"duplicate entity id: {eid}", loc + ".id")
            continue
        if kind not in TYPES:
            emit("AXE004", f"unknown entity type: {kind}", loc + ".type")
            continue
        name = item.get("name", eid)
        if not isinstance(name, str) or not name.strip() or len(name) > 256:
            emit("AXE005", "name must be a nonempty string <=256 characters", loc + ".name")
            continue
        nodes_by_id[eid] = {"id": eid, "type": kind, "name": name,
                            "attributes": {k: v for k, v in item.items() if k not in {"id", "type", "name"}}}
    normalized_edges = []
    dedup = set()
    for idx, item in enumerate(edges):
        loc = f"relationships[{idx}]"
        if not isinstance(item, dict):
            emit("AXR001", "relationship must be an object", loc)
            continue
        source, target, relation = item.get("from"), item.get("to"), item.get("relation")
        if not all(isinstance(v, str) for v in (source, target, relation)):
            emit("AXR002", "from, to and relation must be strings", loc)
            continue
        if relation not in RELATIONS:
            emit("AXR003", f"unknown relation: {relation}", loc + ".relation")
            continue
        if source not in nodes_by_id or target not in nodes_by_id:
            emit("AXR004", f"unknown relationship endpoint: {source} -> {target}", loc)
            continue
        identity = (source, relation, target)
        if identity in dedup:
            emit("AXR005", f"duplicate relationship: {identity}", loc)
            continue
        dedup.add(identity)
        normalized_edges.append({"from": source, "relation": relation, "to": target})
    normalized_constraints = []
    seen_constraints = set()
    for idx, item in enumerate(constraints):
        loc = f"constraints[{idx}]"
        if not isinstance(item, dict):
            emit("AXC001", "constraint must be an object", loc)
            continue
        cid, selector, must = item.get("id"), item.get("select"), item.get("must")
        if not isinstance(cid, str) or not IDENT.fullmatch(cid) or cid in seen_constraints:
            emit("AXC002", "constraint id must be unique and well-formed", loc + ".id")
            continue
        seen_constraints.add(cid)
        if (not isinstance(selector, dict) or not selector or not isinstance(must, dict) or not must):
            emit("AXC003", "constraint needs nonempty select and must maps", loc)
            continue
        if set(selector) - {"type", "carries_classification"}:
            emit("AXC006", "unsupported selector keys", loc + ".select")
            continue
        if not isinstance(selector.get("type"), str) or selector["type"] not in TYPES:
            emit("AXC006", "select.type must be a known entity type", loc + ".select.type")
            continue
        if "carries_classification" in selector and not isinstance(selector["carries_classification"], str):
            emit("AXC006", "carries_classification must be text", loc + ".select")
            continue
        missing = item.get("missing_evidence", "fail")
        enforcement = item.get("enforcement", "hard")
        if missing not in {"fail", "warn"} or enforcement not in {"hard", "advisory"}:
            emit("AXC007", "invalid missing_evidence or enforcement", loc)
            continue
        normalized_constraints.append({"id": cid, "select": selector, "must": must,
                                       "missing_evidence": missing, "enforcement": enforcement})
    for eid, entity in nodes_by_id.items():
        if entity["type"] != "data_flow":
            continue
        attrs = entity["attributes"]
        for key in ("source", "destination"):
            if key in attrs and attrs[key] not in nodes_by_id:
                emit("AXE006", f"unknown {key} reference: {attrs[key]}", eid + "." + key)
        if "carries" in attrs:
            if not isinstance(attrs["carries"], list) or any(
                not isinstance(ref, str) or ref not in nodes_by_id or nodes_by_id[ref]["type"] != "data_asset"
                for ref in attrs["carries"]
            ):
                emit("AXE007", "carries must reference existing data_asset entities", eid + ".carries")
    for rule in normalized_constraints:
        selected = []
        for eid, entity in nodes_by_id.items():
            selector = rule["select"]
            if entity["type"] != selector["type"]:
                continue
            expected_classification = selector.get("carries_classification")
            if expected_classification is not None:
                if entity["type"] != "data_flow":
                    continue
                carries = entity["attributes"].get("carries", [])
                if not isinstance(carries, list):
                    continue
                if not any(ref in nodes_by_id and
                           nodes_by_id[ref]["attributes"].get("classification") == expected_classification
                           for ref in carries):
                    continue
            selected.append(eid)
        if not selected:
            emit("AXC008", f"constraint {rule['id']} selected no entities", rule["id"], "warning")
        for eid in selected:
            attrs = nodes_by_id[eid]["attributes"]
            for key, expected in rule["must"].items():
                if key not in attrs:
                    severity = "error" if rule["missing_evidence"] == "fail" and rule["enforcement"] == "hard" else "warning"
                    emit("AXC004", f"{eid} missing required attribute {key} for {rule['id']}", eid, severity)
                elif attrs[key] != expected:
                    severity = "error" if rule["enforcement"] == "hard" else "warning"
                    emit("AXC005", f"{eid}.{key}: expected {expected!r}; got {attrs[key]!r}", eid, severity)
    ir = {"archir_version": DRAFT_VERSION, "namespace": namespace,
          "entities": sorted(nodes_by_id.values(), key=lambda n: n["id"]),
          "relationships": sorted(normalized_edges, key=lambda r: (r["from"], r["relation"], r["to"])),
          "constraints": sorted(normalized_constraints, key=lambda c: c["id"])}
    diagnostics.sort(key=lambda d: (d.severity != "error", d.path, d.code, d.message))
    return Compiled(ir, diagnostics)


def compile_file(path: str | Path) -> Compiled:
    try:
        return compile_document(read_source(path))
    except SourceError as exc:
        return Compiled({}, [Diagnostic(exc.code, "error", str(exc), str(path))])


def query_reachable(ir: dict, start: str, direction: str = "outgoing", depth: int = 3) -> dict:
    nodes = {node["id"]: node for node in ir["entities"]}
    if start not in nodes:
        raise SourceError("AXQ001", f"unknown entity id: {start}")
    if direction not in {"incoming", "outgoing", "both"} or depth not in range(0, 11):
        raise SourceError("AXQ002", "direction or depth outside supported range (0..10)")
    seen = {start: 0}
    reached_edges = set()    queue = deque([start])
    while queue:
        current = queue.popleft()
        if seen[current] >= depth:
            continue
        for edge in ir["relationships"]:
            if direction in {"outgoing", "both"} and edge["from"] == current:
                next_node = edge["to"]
            elif direction in {"incoming", "both"} and edge["to"] == current:
                next_node = edge["from"]
            else:
                continue
            reached_edges.add((edge["from"], edge["relation"], edge["to"]))
            if next_node not in seen:
                seen[next_node] = seen[current] + 1
                queue.append(next_node)
    return {"start": start, "direction": direction, "depth": depth,
            "nodes": [{"id": eid, "type": nodes[eid]["type"], "distance": dist}
                      for eid, dist in sorted(seen.items())],
            "relationships": [{"from": a, "relation": rel, "to": b}
                              for a, rel, b in sorted(reached_edges)]}
