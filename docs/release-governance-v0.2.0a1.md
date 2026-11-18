# Arcxiom v0.2.0a1 governance - solo maintainer

Status: **owner-authorized, temporary solo-maintainer governance profile**. GitHub repository settings must be applied and verified by the owner. Committing this file or a JSON policy does not change live GitHub permissions.

Repository: public FOSS repository jmsbooth/arcxiom. Maintainer: authenticated GitHub account jmsbooth. **Public visibility permits anybody to read/fork the code and submit pull requests; it does not grant push, merge, or release authority.** Limit write access to the owner and explicitly approved collaborators/Apps.

## Main branch protection

.github/main-protection.json is the canonical protection payload for this phase.

- A pull request is required before merging to main. The required_pull_request_reviews object remains enabled with **zero required approvals** because GitHub cannot count a PR author's own review as an independent approval.
- Branch must be current and pass all 11 versioned required GitHub Actions checks: Rust Linux/macOS/Windows, RustSec audit, six Python matrix checks, and wheel smoke. The live check entries should be associated with the expected GitHub Actions App.
- Require conversation resolution, linear history, enforce administrator protection, and prohibit force pushes/deletions. No bypass allowances.
- No approval is inferred from passing CI. The sole maintainer assumes review and release accountability, including for untrusted public PRs and privileged workflow changes.
- Re-enable required non-author approval and last-push approval when a second trusted maintainer becomes available; update FOSS-593.

On the owner's Mac after fetching this PR branch:

    export REPO="jmsbooth/arcxiom"
    gh auth status
    git fetch origin
    git switch feat/FOSS-594-rust-native-kernel
    git pull --ff-only
    gh api -X PUT "repos/$REPO/branches/main/protection" --input .github/main-protection.json
    gh api "repos/$REPO/branches/main" --jq '.protected'
    gh api "repos/$REPO/branches/main/protection" --jq '{required_pull_request_reviews,required_status_checks,enforce_admins,required_linear_history,required_conversation_resolution,allow_force_pushes,allow_deletions}'

Expected: protected=true; required_approving_review_count=0, require_last_push_approval=false, strict=true, enforce_admins.enabled=true, and all 11 checks. **Never set required_pull_request_reviews to null simply to unblock a PR**: that removes the PR requirement.

## Releases: separate affirmative owner actions

The release workflow requires exact manual workflow_dispatch by jmsbooth on protected main. It requires the exact commit SHA and typed version acknowledgement, rebuilds/tests with locked dependencies on all three systems, checks archives, and **creates a draft prerelease only**. The owner separately inspects and publishes the draft.

Configure the arcxiom-release environment with jmsbooth as the **sole required reviewer**, allowing self-review because the same account initiates the job. This provides a second explicit approval UI action by the authorized owner, but is **not independent security review**. Restrict deployment to protected branches:

    OWNER_ID=$(gh api user --jq '.id')
    jq -n --argjson id "$OWNER_ID" '{wait_timer:0,prevent_self_review:false,reviewers:[{type:"User",id:$id}],deployment_branch_policy:{protected_branches:true,custom_branch_policies:false}}' |
      gh api -X PUT "repos/$REPO/environments/arcxiom-release" --input -
    gh api "repos/$REPO/environments/arcxiom-release" --jq '{name,protection_rules,deployment_branch_policy}'

Expected: a required_reviewers rule naming jmsbooth with prevent_self_review=false and a protected-branches deployment policy. If another account or prevent_self_review=true is configured, explicitly correct it. Disable admin bypass for the environment in GitHub Settings where supported.

Do not dispatch the release workflow before merge to test it: it only exists on main after merging and a successful run may create a draft release/tag. Test only during actual owner-authorized publication.

## Least privilege for Actions and contributors

Set default GITHUB_TOKEN permission to read, and disable approval of PRs by Actions (the release job separately requests contents:write):

    gh api -X PUT "repos/$REPO/actions/permissions/workflow" -f default_workflow_permissions=read -F can_approve_pull_request_reviews=false
    gh api "repos/$REPO/actions/permissions/workflow"

For public FOSS, **require jmsbooth to approve every outside contributor's fork-PR workflow before CI starts**. GitHub's default first-time-contributor policy is weaker because previously merged external contributors can run CI without new approval. This setting preserves public PR submissions while preventing unapproved external workflow execution:

    gh api -X PUT "repos/$REPO/actions/permissions/fork-pr-contributor-approval" -f approval_policy=all_external_contributors
    gh api "repos/$REPO/actions/permissions/fork-pr-contributor-approval" --jq '.approval_policy'

Expected output: all_external_contributors. An external PR's required checks may wait until the owner approves its workflow run; this is intentional. This policy does not block issues, forks, or PR submissions.

Audit collaborators before revoking anything:

    gh api --paginate "repos/$REPO/collaborators?affiliation=all&per_page=100" --jq '.[] | [.login,.permissions.admin,.permissions.push] | @tsv'

For a specific non-approved collaborator confirmed by the owner, use gh api -X DELETE repos/OWNER/REPO/collaborators/USERNAME, substituting the actual account name. Do not invite jmsbooth as collaborator: the owner already has admin privileges.

Inspect Settings > Integrations > GitHub Apps and Settings > Actions > General for write-enabled automation and tokens. Org-only branch protection writer restrictions are not available on this personal repository, so restrict privileged users through collaborators and installed GitHub Apps.

Public contributors may fork and submit PRs but should not be invited as collaborators by default. Avoid pull_request_target jobs that execute untrusted contributor code or granting write permissions to fork PR workflows.

## FOSS-593 and FOSS-594 acceptance evidence

- Protected main API response showing PR requirement, zero mandatory reviewers, exact CI checks with the expected GitHub App, admin enforcement and no bypass, force pushes, or deletions.
- Read-only default workflow token, no Actions-generated PR approvals, mandatory approval for all external contributor workflow runs, and reviewed collaborator and GitHub App permissions.
- arcxiom-release environment with only jmsbooth as required reviewer, self-review allowed, protected-branch-only deployments.
- Green current-head CI, RustSec, Python, wheel, native release ZIP and SHA256SUMS; verify the exact source SHA.
- Explicit maintainer acceptance in the PR or FOSS-594 as a SOLO-maintainer security review, never presented as independent certification.
- Manual release dispatch after merge, followed by a separate draft publication decision.

**Residual risk:** compromise of the single maintainer account could authorize code and release after the checks pass. Use phishing-resistant MFA/passkeys, limit tokens and apps, and audit repository events. Automated RustSec/CI does not substitute for independent code audit. Reintroduce review when a second maintainer becomes available.

This exception applies to Arcxiom v0.2.0a1 only. It does not waive mandatory CI or grant runtime security assurance.
