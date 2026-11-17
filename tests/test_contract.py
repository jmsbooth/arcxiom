"""Interchange fixtures: structural checks are separate from semantic validation."""

from pathlib import Path
import copy
import json

from jsonschema import Draft202012Validator
import pytest

from arcxiom.cli import main
from arcxiom.model import compile_file

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = json.loads((ROOT / "spec" / "archir-v0.1.schema.json").read_text())
EXAMPLE = ROOT / "examples" / "emergency-response.arch.yaml"


def test_schema_is_well_formed():
    Draft202012Validator.check_schema(SCHEMA)


def test_compiled_example_validates_against_archir_schema():
    doc = compile_file(EXAMPLE)
    assert doc.valid
    assert not list(Draft202012Validator(SCHEMA).iter_errors(doc.ir))


def test_schema_rejects_unknown_interchange_field():
    ir = copy.deepcopy(compile_file(EXAMPLE).ir)
    ir["unrecognized"] = {"value": "silent downgrade"}
    with pytest.raises(Exception):
        Draft202012Validator(SCHEMA).validate(ir)


def test_stable_golden_digest():
    expected = (ROOT / "tests" / "fixtures" / "emergency-response.digest").read_text().strip()
    assert compile_file(EXAMPLE).digest == expected


def test_cli_export_with_digest_conforms_to_published_schema(tmp_path):
    path = tmp_path / "exported-archir.json"
    assert main(["export", str(EXAMPLE), "--output", str(path)]) == 0
    exported = json.loads(path.read_text(encoding="utf-8"))
    assert len(exported["sha256"]) == 64
    Draft202012Validator(SCHEMA).validate(exported)
    assert exported["sha256"] == compile_file(EXAMPLE).digest


@pytest.mark.parametrize("invalid", ["bad", "G" * 64, 123])
def test_export_schema_rejects_invalid_digest(invalid):
    ir = copy.deepcopy(compile_file(EXAMPLE).ir)
    ir["sha256"] = invalid
    with pytest.raises(Exception):
        Draft202012Validator(SCHEMA).validate(ir)
