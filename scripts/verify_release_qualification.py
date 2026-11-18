#!/usr/bin/env python3
"""Fail-closed offline validation of release-source PR provenance and exact CI.

The workflow fetches only read-accessible GitHub API data. It does NOT claim to
read the live full branch-protection policy (which requires Administration
permission). Instead it independently verifies the merged main SHA passed the
complete, frozen test matrix and is associated with a merged PR.
"""
import argparse
import json
from pathlib import Path

ACTIONS_APP_ID = 15368
EXPECTED_CHECKS = frozenset({
    "kernel / ubuntu-latest",
    "kernel / macos-latest",
    "kernel / windows-latest",
    "dependency-audit / RustSec",
    "python-prototype (ubuntu-latest, 3.11)",
    "python-prototype (ubuntu-latest, 3.13)",
    "python-prototype (macos-latest, 3.11)",
    "python-prototype (macos-latest, 3.13)",
    "python-prototype (windows-latest, 3.11)",
    "python-prototype (windows-latest, 3.13)",
    "wheel-smoke",
})


def reject(message: str) -> None:
    raise ValueError("AXREL001: " + message)


def qualify(sha: str, branch: dict, policy: dict,
            checks: list[dict], pulls: list[dict]) -> None:
    if len(sha) != 40 or any(c not in "0123456789abcdef" for c in sha):
        reject("approved SHA is not full lowercase git SHA-1")
    if branch.get("name") != "main" or branch.get("protected") is not True:
        reject("main is not protected")
    if branch.get("commit", {}).get("sha") != sha:
        reject("main moved after the approval was submitted")
    required = policy.get("required_status_checks", {})
    if required.get("strict") is not True or set(required.get("contexts", [])) != EXPECTED_CHECKS:
        reject("frozen 11-check qualification profile changed")
    if (policy.get("enforce_admins") is not True
        or policy.get("required_conversation_resolution") is not True
        or policy.get("allow_force_pushes") is not False
        or policy.get("allow_deletions") is not False
        or policy.get("required_linear_history") is not True
        or not isinstance(policy.get("required_pull_request_reviews"), dict)
        or policy["required_pull_request_reviews"].get("required_approving_review_count") != 0):
        reject("committed solo-maintainer protection profile changed")
    associated = any(
        item.get("merged_at") and item.get("merge_commit_sha") == sha
        and item.get("base", {}).get("ref") == "main"
        for item in pulls
    )
    if not associated:
        reject("approved main commit lacks a merged PR association")
    for name in sorted(EXPECTED_CHECKS):
        matched = [
            c for c in checks
            if c.get("name") == name
            and c.get("app", {}).get("id") == ACTIONS_APP_ID
            and c.get("head_sha") == sha
        ]
        if not matched:
            reject(f"missing trusted GitHub Actions check for {name}")
        newest = max(matched, key=lambda c: c.get("id", 0))
        if newest.get("status") != "completed" or newest.get("conclusion") != "success":
            reject(f"latest check is not successful: {name}")
    print(f"Release source verified: merged PR on protected main, {len(EXPECTED_CHECKS)}"
          f" successful trusted checks, sha={sha}")


def ndjson(path: Path) -> list[dict]:
    return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines()
            if line.strip()]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--sha", required=True)
    parser.add_argument("--branch", required=True, type=Path)
    parser.add_argument("--policy", required=True, type=Path)
    parser.add_argument("--checks", required=True, type=Path)
    parser.add_argument("--pulls", required=True, type=Path)
    args = parser.parse_args()
    qualify(
        args.sha,
        json.loads(args.branch.read_text(encoding="utf-8")),
        json.loads(args.policy.read_text(encoding="utf-8")),
        ndjson(args.checks),
        ndjson(args.pulls),
    )


if __name__ == "__main__":
    main()
