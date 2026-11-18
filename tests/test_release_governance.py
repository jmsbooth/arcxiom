"""Contract tests for GitHub release governance; do not claim to enforce settings."""

import copy
import importlib.util
import json
import os
import re
import subprocess
import sys
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
    build_commands = "\n".join(step.get("run", "") for step in jobs["build"]["steps"])
    assert "python scripts/package_release.py --output dist" in build_commands
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
    # Solo-maintainer FOSS posture: PRs/CI remain mandatory; self-review is
    # impossible for GitHub PRs, so no independent approval is required yet.
    assert policy["required_pull_request_reviews"] is not None
    assert policy["restrictions"] is None  # personal GitHub repository
    assert policy["required_pull_request_reviews"]["required_approving_review_count"] == 0
    assert policy["required_pull_request_reviews"]["require_last_push_approval"] is False
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
    kernel_steps = _workflow(KERNEL)["jobs"]["rust-kernel"]["steps"]
    assert any("python scripts/package_release.py --output dist" in step.get("run", "")
               for step in kernel_steps)
    rust = _workflow(KERNEL)
    assert rust["permissions"] == {"contents": "read"}
    assert "pull_request" in rust["on"]
    assert "push" in rust["on"]

def test_actual_approval_shell_rejects_wrong_commit_acknowledgement_and_length():
    # Windows runner validates the exact workflow content above; Linux/macOS
    # additionally execute the real Bash assertions in the release job.
    if sys.platform == "win32":
        return
    workflow = _workflow(WORKFLOW)
    approval = workflow["jobs"]["verify-approval"]
    script = next(step["run"] for step in approval["steps"]
                  if step.get("name") == "Reject incorrect commit or typed approval")
    sha = "a" * 40
    cases = [
        (sha, sha, "v0.2.0a1", 0),
        (sha, "b" * 40, "v0.2.0a1", 1),
        (sha, sha, "not-approved", 1),
        (sha[:39], sha[:39], "v0.2.0a1", 1),
    ]
    for actual, expected, confirmation, expected_success in cases:
        env = dict(os.environ, GITHUB_SHA=actual, EXPECTED=expected,
                   CONFIRMATION=confirmation)
        completed = subprocess.run(["bash", "-e", "-c", script],
                                   env=env, check=False, capture_output=True)
        assert (completed.returncode == 0) == (expected_success == 0), (
            actual, expected, confirmation, completed.stderr
        )


def test_external_github_actions_are_full_sha_pinned():
    for path in (WORKFLOW, KERNEL, PYTHON, WHEEL):
        workflow = _workflow(path)
        for job in workflow["jobs"].values():
            for step in job.get("steps", []):
                action = step.get("uses")
                if not action or action.startswith("./"):
                    continue
                assert re.fullmatch(r"[^@\s]+@[0-9a-f]{40}", action), (
                    f"Unpinned action {action} in {path}"
                )


def test_public_pull_request_workflows_are_read_only_and_never_target_privileged_context():
    for path in (KERNEL, PYTHON, WHEEL):
        workflow = _workflow(path)
        assert "pull_request" in workflow["on"]
        assert "pull_request_target" not in workflow["on"]
        assert workflow["permissions"] == {"contents": "read"}
        for job in workflow["jobs"].values():
            perms = job.get("permissions", {})
            assert all(value != "write" for value in perms.values()), (
                f"Untrusted PR job has write privileges: {path}"
            )


def test_release_checks_exact_sha_pr_provenance_and_trusted_check_run_results():
    source = ROOT / "scripts/verify_release_qualification.py"
    spec = importlib.util.spec_from_file_location("verify_release_qualification", source)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)

    sha = "a" * 40
    policy = json.loads(PROTECTION.read_text(encoding="utf-8"))
    branch = {"name": "main", "protected": True, "commit": {"sha": sha}}
    pulls = [{"merged_at": "2026-11-18T12:00:00Z", "merge_commit_sha": sha,
              "base": {"ref": "main"}}]
    checks = [
        {"id": n, "name": name, "app": {"id": module.ACTIONS_APP_ID},
         "head_sha": sha, "status": "completed", "conclusion": "success"}
        for n, name in enumerate(sorted(module.EXPECTED_CHECKS), 1)
    ]
    assert len(checks) == 11
    module.qualify(sha, branch, policy, checks, pulls)

    def rejects(b=branch, p=policy, c=checks, prs=pulls, commit=sha):
        try:
            module.qualify(commit, b, p, c, prs)
        except ValueError as exc:
            assert str(exc).startswith("AXREL001:")
        else:
            raise AssertionError("unsafe release source was accepted")

    rejects(b={**branch, "protected": False})
    rejects(b={**branch, "commit": {"sha": "b" * 40}})
    rejects(p={**policy, "enforce_admins": False})
    rejects(p={**policy, "required_pull_request_reviews": None})
    rejects(p={**policy, "required_status_checks": {"strict": True, "contexts": ["wheel-smoke"]}})
    rejects(prs=[])
    rejects(prs=[{**pulls[0], "merged_at": None}])
    rejects(prs=[{**pulls[0], "merge_commit_sha": "b" * 40}])
    rejects(c=checks[:-1])
    wrong_app = copy.deepcopy(checks)
    wrong_app[0]["app"]["id"] = 12345
    rejects(c=wrong_app)
    wrong_sha = copy.deepcopy(checks)
    wrong_sha[0]["head_sha"] = "b" * 40
    rejects(c=wrong_sha)
    failed_latest = copy.deepcopy(checks)
    failed_latest.append({**checks[0], "id": 1000, "conclusion": "failure"})
    rejects(c=failed_latest)
    running_latest = copy.deepcopy(checks)
    running_latest.append({**checks[0], "id": 1001, "conclusion": None, "status": "in_progress"})
    rejects(c=running_latest)
    rejects(commit="not-a-full-sha")


def test_manual_release_requires_read_only_checks_and_pr_provenance():
    workflow = _workflow(WORKFLOW)
    gate = workflow["jobs"]["verify-approval"]
    assert gate["permissions"] == {
        "contents": "read", "checks": "read", "pull-requests": "read"
    }
    code = "\n".join(step.get("run", "") for step in gate["steps"])
    assert "scripts/verify_release_qualification.py" in code
    assert "commits/$GITHUB_SHA/check-runs" in code
    assert "commits/$GITHUB_SHA/pulls" in code
    assert "--policy .github/main-protection.json" in code
