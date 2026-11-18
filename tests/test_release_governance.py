"""Contract tests for GitHub release governance; do not claim to enforce settings."""

import json
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / ".github/workflows/publish-v0.2.0a1.yml"
KERNEL = ROOT / ".github/workflows/rust-archir.yml"
PYTHON = ROOT / ".github/workflows/prototype-ci.yml"
WHEEL = ROOT / ".github/workflows/release-candidate.yml"
PROTECTION = ROOT / ".github/main-protection.json"


def _workflow(path):
    return yaml.load(path.read_text(encoding="utf-8"), Loader=yaml.BaseLoader)


def test_manual_release_only_uses_read_only_default_permissions():
    wf = _workflow(WORKFLOW)
    assert set(wf["on"]) == {"workflow_dispatch"}
    assert wf["permissions"] == {"contents": "read"}
    assert {"approved_commit", "acknowledge"} <= set(wf["on"]["workflow_dispatch"]["inputs"])
    approval = wf["jobs"]["verify-approval"]
    assert "github.actor == 'jmsbooth'" in approval["if"]
    assert "github.ref == 'refs/heads/main'" in approval["if"]
    code = "\n".join(step.get("run", "") for step in approval["steps"])
    assert "branches/main" in code and "'.protected'" in code
    assert "test \"${#EXPECTED}\" -eq 40" in code
    assert r"\${#EXPECTED}" not in code
    assert 'test "$EXPECTED" = "$GITHUB_SHA"' in code
    assert 'test "$CONFIRMATION" = \'v0.2.0a1\'' in code


def test_release_publication_cannot_bypass_build_and_environment():
    wf = _workflow(WORKFLOW)
    jobs = wf["jobs"]
    assert jobs["build"]["needs"] == "verify-approval"
    assert jobs["create-draft"]["needs"] == "build"
    assert jobs["create-draft"]["environment"] == "arcxiom-release"
    assert jobs["create-draft"]["permissions"]["contents"] == "write"
    matrix = jobs["build"]["strategy"]["matrix"]
    assert set(matrix["os"]) == {"ubuntu-latest", "macos-latest", "windows-latest"}
    release_commands = "\n".join(step.get("run", "") for step in jobs["create-draft"]["steps"])
    assert "sha256sum -c SHA256SUMS" in release_commands
    assert "gh release create" in release_commands
    assert "--draft" in release_commands and "--prerelease" in release_commands
    assert "--target \"$GITHUB_SHA\"" in release_commands
    assert "Release already exists; refuse mutation" in release_commands
    assert "Tag already exists; refuse to move immutable ref" in release_commands


def test_branch_protection_check_names_match_workflows():
    policy = json.loads(PROTECTION.read_text(encoding="utf-8"))
    required = policy["required_status_checks"]
    assert required["strict"] is True
    assert policy["enforce_admins"] is True
    assert policy["required_pull_request_reviews"]["required_approving_review_count"] >= 1
    assert policy["required_pull_request_reviews"]["require_last_push_approval"] is True
    assert policy["required_pull_request_reviews"]["dismiss_stale_reviews"] is True
    assert policy["required_linear_history"] is True
    assert policy["allow_force_pushes"] is False
    assert policy["allow_deletions"] is False
    assert policy["required_conversation_resolution"] is True
    rust = _workflow(KERNEL)["jobs"]
    expected = {
        "kernel / " + os for os in rust["rust-kernel"]["strategy"]["matrix"]["os"]
    } | {"dependency-audit / RustSec"}
    py = _workflow(PYTHON)["jobs"]["python-prototype"]["strategy"]["matrix"]
    expected |= {
        f"python-prototype ({os}, {version})"
        for os in py["os"] for version in py["python-version"]
    }
    wheel_jobs = _workflow(WHEEL)["jobs"]
    assert "wheel-smoke" in wheel_jobs
    expected.add("wheel-smoke")
    assert set(required["contexts"]) == expected


def test_release_validation_uses_pinned_dependencies_and_no_pr_write_token():
    wf = _workflow(WORKFLOW)
    assert "--locked" in "\n".join(
        s.get("run", "") for s in wf["jobs"]["build"]["steps"]
    )
    assert "--locked" in "\n".join(
        s.get("run", "") for s in _workflow(KERNEL)["jobs"]["rust-kernel"]["steps"]
    )
    rust = _workflow(KERNEL)
    assert rust["permissions"] == {"contents": "read"}
    assert "pull_request" in rust["on"]
    assert "push" in rust["on"]
