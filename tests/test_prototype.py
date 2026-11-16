from __future__ import annotations

from pathlib import Path
import json

import fitz
import pytest

from arcxiom.cli import main
from arcxiom.model import compile_document, compile_file, query_reachable
from arcxiom.drawing import render_pdf

EXAMPLE = Path(__file__).resolve().parents[1] / "examples" / "emergency-response.arch.yaml"


def test_example_valid():
    model = compile_file(EXAMPLE)
    assert model.valid, model.diagnostics
    assert len(model.ir["entities"]) == 8
    assert len(model.ir["relationships"]) == 7


def test_archir_deterministic_on_reorder():
    import copy
    from arcxiom.model import read_source
    original = read_source(EXAMPLE)
    reordered = copy.deepcopy(original)
    for field in ("entities", "relationships", "constraints"):
        reordered[field].reverse()
    assert compile_document(original).digest == compile_document(reordered).digest


def test_restricted_unencrypted_fails():
    from arcxiom.model import read_source
    raw = read_source(EXAMPLE)
    raw["entities"][-1]["transport_encrypted"] = False
    compiled = compile_document(raw)
    assert not compiled.valid
    assert any(d.code == "AXC005" for d in compiled.diagnostics)


def test_missing_security_attribute_fails():
    from arcxiom.model import read_source
    raw = read_source(EXAMPLE)
    del raw["entities"][-1]["transport_encrypted"]
    assert any(d.code == "AXC004" for d in compile_document(raw).diagnostics)


def test_invalid_reference_fails():
    from arcxiom.model import read_source
    raw = read_source(EXAMPLE)
    raw["relationships"][0]["to"] = "unknown"
    assert any(d.code == "AXR004" for d in compile_document(raw).diagnostics)


def test_duplicate_id_fails():
    from arcxiom.model import read_source
    raw = read_source(EXAMPLE)
    raw["entities"].append(dict(raw["entities"][0]))
    assert any(d.code == "AXE003" for d in compile_document(raw).diagnostics)


@pytest.mark.parametrize("source,code", [
    ("version: 0.1-draft\nversion: 0.1-draft\nnamespace: app\n", "AXY002"),
    ("version: 0.1-draft\nnamespace: app\nanchor: &x {a: b}\ncopy: *x\n", "AXY003"),
    ("version: 0.1-draft\nnamespace: app\nentities: !!python/object/apply:os.system ['echo x']", "AXY001"),
    ("version: 0.1-draft\nnamespace: app\nentities: [\n", "AXY001"),
])
def test_strict_yaml(tmp_path, source, code):
    path = tmp_path / "bad.arch.yaml"
    path.write_text(source)
    result = compile_file(path)
    assert not result.valid
    assert result.diagnostics[0].code == code


def test_yaml_12_bool_values(tmp_path):
    path = tmp_path / "bool.arch.yaml"
    path.write_text('version: "0.1-draft"\nnamespace: sample\nentities:\n  - id: role.a\n    type: role\n    on: yes\n    enabled: true\n')
    model = compile_file(path)
    assert model.valid
    attrs = model.ir["entities"][0]["attributes"]
    assert attrs["on"] == "yes"
    assert attrs["enabled"] is True


def test_query_trace_to_mission():
    model = compile_file(EXAMPLE)
    result = query_reachable(model.ir, "service.dispatch-api", depth=4)
    assert "mission.coordinate-response" in {n["id"] for n in result["nodes"]}


def test_cli_validate_json(capsys):
    assert main(["validate", str(EXAMPLE), "--json"]) == 0
    parsed = json.loads(capsys.readouterr().out)
    assert parsed["valid"] is True and len(parsed["digest"]) == 64


def test_cli_export_json(tmp_path):
    target = tmp_path / "archir.json"
    assert main(["export", str(EXAMPLE), "--format", "json", "-o", str(target)]) == 0
    assert json.loads(target.read_text())["namespace"] == "emergency-response"


def test_generic_pdf_is_vector_readable_and_deterministic(tmp_path):
    model = compile_file(EXAMPLE)
    first, second = tmp_path / "a.pdf", tmp_path / "b.pdf"
    render_pdf(model.ir, first)
    render_pdf(model.ir, second)
    assert first.read_bytes() == second.read_bytes()
    assert first.read_bytes().startswith(b"%PDF")
    with fitz.open(first) as doc:
        assert len(doc) == 1
        text = doc[0].get_text()
        assert "NOT TO SCALE" in text
        assert "dispatch-api" in text
        assert len(doc[0].get_drawings()) > 5


def test_reject_unlicensed_profiles(capsys, tmp_path):
    assert main(["render", str(EXAMPLE), "--format", "pdf", "--drawing-profile", "iso-tpd",
                 "-o", str(tmp_path / "out.pdf")]) == 2
    assert "AXP006" in capsys.readouterr().err


def test_refuse_symlink_inputs(tmp_path):
    link = tmp_path / "linked.arch.yaml"
    link.symlink_to(EXAMPLE)
    result = compile_file(link)
    assert not result.valid and result.diagnostics[0].code == "AXY010"


def test_outgoing_query_has_flow():
    model = compile_file(EXAMPLE)
    result = query_reachable(model.ir, "hardware.mobile-terminal", depth=3)
    assert "flow.incident-submission" in {n["id"] for n in result["nodes"]}
    assert "service.dispatch-api" in {n["id"] for n in result["nodes"]}


def test_invalid_type_rejected():
    from arcxiom.model import read_source
    raw = read_source(EXAMPLE)
    raw["entities"][0]["type"] = "nonexistent_type"
    assert any(d.code == "AXE004" for d in compile_document(raw).diagnostics)


def test_pdf_output_rejects_non_ascii_label(tmp_path):
    from arcxiom.model import read_source
    from arcxiom.model import SourceError
    raw = read_source(EXAMPLE)
    raw["entities"][0]["name"] = "Caf\u00e9"
    with pytest.raises(SourceError, match="ASCII"):
        render_pdf(compile_document(raw).ir, tmp_path / "drawing.pdf")


def test_nonexistent_source_is_typed_failure(tmp_path):
    model = compile_file(tmp_path / "absent.arch.yaml")
    assert not model.valid and model.diagnostics[0].code == "AXY012"


def test_non_standard_view_fails(tmp_path, capsys):
    rc = main(["render", str(EXAMPLE), "--format", "pdf", "--view", "physical",
               "-o", str(tmp_path / "bad.pdf")])
    assert rc == 2 and "AXP001" in capsys.readouterr().err


def test_query_unknown_node_fails(capsys):
    assert main(["query", str(EXAMPLE), "--id", "nope"]) == 2
    assert "AXQ001" in capsys.readouterr().err
