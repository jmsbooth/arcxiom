"""Arcxiom command-line reference prototype."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import sys

from .diff import compare_ir, check_guardrail_delta
from .drawing import render_pdf
from .model import SourceError, compile_file, query_reachable


def _print_diagnostics(model, as_json: bool) -> None:
    if as_json:
        print(json.dumps({"valid": model.valid, "digest": model.digest if model.ir else None,
                          "diagnostics": [d.as_json() for d in model.diagnostics]},
                         indent=2, sort_keys=True))
    else:
        for diagnostic in model.diagnostics:
            print(f"{diagnostic.severity.upper()} {diagnostic.code} "
                  f"{diagnostic.path}: {diagnostic.message}", file=sys.stderr)
        if model.valid:
            print(f"valid  |  {len(model.ir['entities'])} entities  |  "
                  f"{len(model.ir['relationships'])} relationships  |  sha256:{model.digest}")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="arcxiom", description="Architecture-as-Code prototype")
    sub = parser.add_subparsers(dest="command", required=True)
    validate = sub.add_parser("validate", help="Validate the YAML draft model")
    validate.add_argument("input")
    validate.add_argument("--json", action="store_true")
    query = sub.add_parser("query", help="Traverse the architecture graph")
    query.add_argument("input")
    query.add_argument("--id", required=True)
    query.add_argument("--direction", choices=["outgoing", "incoming", "both"], default="outgoing")
    query.add_argument("--depth", type=int, default=3)
    query.add_argument("--json", action="store_true")
    export = sub.add_parser("export", help="Export deterministic normalized ArchIR")
    export.add_argument("input")
    export.add_argument("--format", choices=["json"], default="json")
    export.add_argument("--output", "-o")
    render = sub.add_parser("render", help="Render schematic architecture drawing")
    render.add_argument("input")
    render.add_argument("--format", choices=["pdf"], required=True)
    render.add_argument("--view", default="overview")
    render.add_argument("--drawing-profile", default="generic")
    render.add_argument("--info-profile", default="none")
    render.add_argument("--output", "-o")
    diff = sub.add_parser("diff", help="Compare two architecture documents")
    diff.add_argument("baseline", help="Original .arch.yaml")
    diff.add_argument("candidate", help="Proposed .arch.yaml")
    diff.add_argument("--json", action="store_true")
    diff.add_argument("--check-guardrails", action="store_true", help="Reject changes to approved hard constraints and data classifications (baseline authenticity NOT verified)")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    if args.command == "diff":
        baseline, candidate = compile_file(args.baseline), compile_file(args.candidate)
        if not baseline.valid or not candidate.valid:
            _print_diagnostics(baseline, False)
            _print_diagnostics(candidate, False)
            return 2
        try:
            delta = compare_ir(baseline.ir, candidate.ir)
            guardrails = check_guardrail_delta(baseline.ir, candidate.ir) if args.check_guardrails else []
        except ValueError as exc:
            print(f"ERROR AXD001: {exc}", file=sys.stderr)
            return 2
        delta["guardrail_violations"] = [v.as_json() for v in guardrails]
        if args.json:
            print(json.dumps(delta, sort_keys=True, indent=2))
        else:
            for change in delta["changes"]:
                print(f"{change['section']}  {change['action']:<9} {change['id']}")
            for violation in guardrails:
                print(f"ERROR {violation.code}: {violation.object_id}: {violation.message}", file=sys.stderr)
            if not delta["changed"]:
                print("no semantic changes")
        return 2 if guardrails else (3 if delta["changed"] else 0)
    model = compile_file(args.input)
    if args.command == "validate":
        _print_diagnostics(model, args.json)
        return 0 if model.valid else 2
    if not model.valid:
        _print_diagnostics(model, False)
        return 2
    try:
        if args.command == "query":
            result = query_reachable(model.ir, args.id, args.direction, args.depth)
            if args.json:
                print(json.dumps(result, indent=2, sort_keys=True))
            else:
                for node in result["nodes"]:
                    print(f"{node['distance']}  {node['type']:<20} {node['id']}")
            return 0
        if args.command == "export":
            payload = json.dumps({**model.ir, "sha256": model.digest}, indent=2, sort_keys=True) + "\n"
            if args.output:
                output = Path(args.output)
                if output.is_symlink() or not output.parent.is_dir():
                    raise SourceError("AXO001", "unsafe or missing output directory")
                output.write_text(payload, encoding="utf-8")
                print(str(output))
            else:
                print(payload, end="")
            return 0
        if args.command == "render":
            if args.drawing_profile != "generic" or args.info_profile != "none":
                raise SourceError("AXP006", "only generic drawing and none information profiles are supported")
            output = args.output or str(Path(args.input).with_suffix(".pdf"))
            print(render_pdf(model.ir, output, args.view))
            return 0
    except (SourceError, OSError) as exc:
        code = exc.code if isinstance(exc, SourceError) else "AXO002"
        print(f"ERROR {code}: {exc}", file=sys.stderr)
        return 2
    return 2
