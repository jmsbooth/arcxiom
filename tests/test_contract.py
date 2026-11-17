"""Interchange fixtures: structural checks are separate from semantic validation."""

from pathlib import Path
import copy
import json

from jsonschema import Draft202012Validator
import pytest

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
