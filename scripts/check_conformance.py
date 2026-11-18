"""Exercise a CLI against the shared single-file ArchIR corpus and its frozen digests."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys

from jsonschema import Draft202012Validator, ValidationError

ROOT = Path(__file__).resolve().parents[1]
CORPUS = ROOT / "tests/conformance/0.1-draft/cases.json"


def unique_fields(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON field {key!r}")
        result[key] = value
    return result


def read_output(command, args, expected_exit):
    result = subprocess.run([*command, *args], cwd=ROOT, timeout=15, capture_output=True)
    if result.returncode != expected_exit:
        detail = result.stderr.decode("utf-8", errors="replace").strip()
        raise ValueError(f"{args[0]} exited {result.returncode}; expected {expected_exit}. {detail}")
    payload = json.loads(result.stdout, object_pairs_hook=unique_fields)
    if not isinstance(payload, dict):
        raise ValueError(f"{args[0]} output must be a JSON object")
    return payload


def check_export(command, source, expected, schema):
    validated = read_output(command, ["validate", source, "--json"], 0)
    exported = read_output(command, ["export", source], 0)
    schema.validate(exported)
    claim = exported.pop("sha256", None)
    canonical = json.dumps(exported, sort_keys=True, separators=(",", ":"), ensure_ascii=True,
                           allow_nan=False).encode("ascii")
    computed = hashlib.sha256(canonical).hexdigest()
    diagnostics = validated.get("diagnostics")
    if not isinstance(diagnostics, list) or any(
        not isinstance(item, dict) or item.get("severity") != "warning" for item in diagnostics
    ):
        raise ValueError("accepted model returned malformed diagnostics or errors")
    if validated.get("valid") is not True:
        raise ValueError("validate reported an invalid model")
    if computed != expected or claim != expected or validated.get("digest") != expected:
        raise ValueError(f"expected {expected}, computed {computed}, export {claim}, "
                         f"validate {validated.get('digest')}")
    return {"sha256": computed}


def check_rejection(command, source, expected):
    validated = read_output(command, ["validate", source, "--json"], 2)
    diagnostics = validated.get("diagnostics")
    if validated.get("valid") is not False or not isinstance(diagnostics, list):
        raise ValueError("expected an invalid model with structured diagnostics")
    for diagnostic in diagnostics:
        if (isinstance(diagnostic, dict) and diagnostic.get("severity") == "error"
                and all(diagnostic.get(key) == value for key, value in expected.items())):
            return expected
    raise ValueError(f"missing expected diagnostic {expected!r}; received {diagnostics!r}")


def evaluate(commands):
    corpus = json.loads(CORPUS.read_bytes(), object_pairs_hook=unique_fields)
    if not corpus["accepted"] or not corpus["rejected"]:
        raise ValueError("corpus must contain both accepted and rejected cases")
    if corpus["accepted"].keys() & corpus["rejected"].keys():
        raise ValueError("each source must have a single expected outcome")
    definition = json.loads((ROOT / corpus["schema"]).read_bytes())
    Draft202012Validator.check_schema(definition)
    schema = Draft202012Validator(definition)
    cases = []
    for name, command in commands:
        for group in ("accepted", "rejected"):
            for source, expected in corpus[group].items():
                row = {"compiler": name, "source": source}
                try:
                    if group == "accepted":
                        details = check_export(command, source, expected, schema)
                    else:
                        details = check_rejection(command, source, expected)
                    row.update(status="pass", **details)
                except (OSError, ValueError, ValidationError, subprocess.TimeoutExpired) as exc:
                    row.update(status="fail", error=str(exc))
                cases.append(row)
    return {"edition": corpus["edition"], "passed": all(c["status"] == "pass" for c in cases),
            "cases": cases}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", type=Path, required=True, help="Path to the executable under test")
    parser.add_argument("--python-reference", action="store_true", help="Also evaluate python -m arcxiom")
    parser.add_argument("--report", type=Path, help="Write the JSON results to this path as well")
    args = parser.parse_args()
    commands = [("candidate", [str(args.compiler.resolve())])]
    if args.python_reference:
        commands.append(("python-reference", [sys.executable, "-m", "arcxiom"]))
    report = evaluate(commands)
    payload = json.dumps(report, indent=2, ensure_ascii=True) + "\n"
    sys.stdout.write(payload)
    if args.report:
        args.report.write_text(payload, encoding="utf-8")
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
