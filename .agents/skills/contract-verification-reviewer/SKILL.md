---
name: contract-verification-reviewer
description: "Retired compatibility shim. Do not use for live Go-skill delivery. Route pre-proof plan review to proof-plan-reviewer and post-proof artifact review to proof-reviewer."
---

# Contract Verification Reviewer

This skill is historical. It must not issue live Go-skill approvals.

## Replacement Routing

- Before proof writing: use `proof-plan-reviewer`, which owns `proof-plan-review.md` and `verifier-lane-review.jsonl`.
- After proof writing: use `proof-reviewer`, which owns `proof-review.md` and `proof-findings.jsonl`.
- For proof/source/test bridging: use `proof-to-implementation`.

## Failure Behavior

- If an old artifact asks for `contract-verification-review.md`, stop and route the bead to the Go-skill migration path.
- Do not write `STATUS: APPROVED`.
- Do not write or update `contract-verification-review.md` for live delivery.

## Output

Return a concise routing note naming the replacement skill and required artifact. Do not claim review completion.
