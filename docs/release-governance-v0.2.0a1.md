# v0.2.0a1 required GitHub governance (FOSS-593)

Status: **configuration proposal, not proof of enforcement.** GitHub repository settings are not enforced by files committed to a repository. The repository owner must apply the protection and release-environment settings, then capture API screenshots or JSON responses as acceptance evidence before merging this PR.

## Protect the main branch

After the PR checks have executed so GitHub recognizes each context, run with an authenticated repository administrator's GitHub CLI:

    gh api --method PUT repos/jmsbooth/arcxiom/branches/main/protection --input .github/main-protection.json
    gh api repos/jmsbooth/arcxiom/branches/main --jq '.protected'
    gh api repos/jmsbooth/arcxiom/branches/main/protection

The proposed file .github/main-protection.json requires up-to-date PR checks for all three Rust kernel OS jobs, the RustSec dependency audit, six Python compatibility matrix jobs and the wheel smoke job. It requires one non-author review (including review of the last push), stale-review dismissal, resolved conversations, linear history, no administrator bypass, no force pushes, and no branch deletions. Check the actual reported status names; change the JSON contexts if GitHub names differ. This policy will intentionally prevent a sole author from self-approving. Add an independent collaborator reviewer or reviewer team before activation. Do not weaken it simply to merge an unreviewed PR.

**Do not apply the protection file until all required check names exist on the PR.** With a required check that has never run on the branch, GitHub will block all merges. Review and update for any third-party status collisions; provider-specific app IDs may be configured if required.

## Require maintainer approval for release preparation

Repository Settings > Environments > arcxiom-release:

- Create this environment.
- Configure at least one trusted **independent** required reviewer; disable administrator bypass and self-review where supported.
- Restrict deployment branches/tags to protected main and reviewed release workflow only.
- Verify an unapproved run of Prepare v0.2.0a1 release draft (manual only) actually waits for reviewer approval. Do not treat a file named environment as enforcement without a protection rule.

The release workflow is manual only. It checks that the invoking account is the authorized maintainer, the input is exactly v0.2.0a1, and the supplied 40-character commit SHA is the checked-out main SHA. It re-runs locked builds/tests on Linux, macOS, and Windows, generates and verifies archive SHA-256 files, and **creates an unpublished draft prerelease**, never a public release. The maintainer must separately inspect and publish the draft after reviewing all evidence.

The branch should be merged into main **only after** an independent reviewer has approved the code and all required checks pass, and should be tagged/published **only after** separate maintainer release approval. Do not run the manual release workflow before these conditions.

## Evidence to attach to FOSS-594

- Protected main branch status / exact branch protection API response.
- Required check contexts with successful results on the merge SHA.
- Independent PR reviewer identity and time; code and security review resolution.
- Release environment reviewer rule and a blocked/unblocked test.
- Exact SHA of approved main build, RustSec audit report, dependency inventory, candidate archive SHA256SUMS (each platform), and independently recomputed checksums.
- Confirmation the release remains **draft** until explicit publication approval.

No approval, publication or runtime security evidence is implied by the current commits.
