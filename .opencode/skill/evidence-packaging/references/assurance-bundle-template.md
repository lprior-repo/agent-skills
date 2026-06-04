# Assurance Bundle Template

```markdown
# Assurance Bundle

bead_id: <bead-id>
source_checkout: <path>
isolated_workspace: <path>
commit_or_change: <id>

## Requirement Coverage

| Requirement | Contract Clause | Proof/Test Evidence | Review Evidence | Status |
|---|---|---|---|---|

## Proof Evidence

| Obligation | Tool | Command | Artifact | Result | Waiver |
|---|---|---|---|---|---|

## Test Evidence

| Test/Gate | Command | Artifact | Result |
|---|---|---|---|

## Review Evidence

| Review | Artifact | Status | Findings |
|---|---|---|---|

## Findings Disposition

| Finding | Severity | Source Review | Disposition | Evidence Or Owner Approval |
|---|---|---|---|---|

## Waivers And Deferred Work

Waivers and deferred work are not finding dispositions. Findings must use only canonical `finding/v1.disposition` values: `fixed_with_evidence`, `owner_approved_debt`, `owner_approved_no_action`, or `blocker`.

| Item | Reason | Owner | Expiry/Follow-up | Compensating Evidence |
|---|---|---|---|---|

## Truth Serum Audit

- report: `.beads/<bead-id>/truth-serum-report.md`
- status: <APPROVED|REJECTED|UNVERIFIED>
```
