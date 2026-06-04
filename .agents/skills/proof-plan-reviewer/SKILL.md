---
name: proof-plan-reviewer
description: "Brutal pre-proof reviewer for implementation-bound proof plans. Use after proof-planner and before proof-writer. Rejects missing Verus/Kani/Flux/proptest default Rust lanes, unjustified Loom/fuzz decisions, weak commands, shallow bounds, invalid waivers, absent bridge planning, and self-approved artifacts. Writes review artifacts only."
---

# Proof Plan Reviewer

Stop bad proof plans before proof-writer wastes time. Assume the planner skipped hard lanes or hid uncertainty in prose.

## Owns

- `proof-plan-review.md`
- `verifier-lane-review.jsonl`
- `proof-plan-findings.jsonl`
- `proof-plan-repair-guide.md` when rejected

## Inputs

- `contract.md`, domain/type/workflow artifacts, `proof-seeds.jsonl`, `traceability-matrix.jsonl`.
- `proof-strategy.md`, `verifier-lane-decisions.jsonl`, `proof-obligations.planned.jsonl`, `trusted-base-plan.md`, waiver candidates.
- `agent-invocation-ledger.jsonl` or host control-plane provenance when available.

## Workflow

1. Verify review provenance: reviewer invocation must differ from planner invocation.
2. Validate every proof seed has lane decisions for the required lane profile from `verification-lane-policy.md`; Rust-local behavior defaults to Verus, Kani, Flux, and proptest.
3. Reject weak `not_applicable` decisions, missing evidence refs, self-stamped reviewer fields, or any planner-owned lane that cannot receive an accepted review row.
4. Check required lanes against the shared verification-lane policy.
5. Check every planned obligation has schema version, exact command, workdir, bounds, assumptions, expected evidence, and no legacy alias fields.
6. Reject behavior-affecting waivers and waivers that exist because proof is hard.
7. Check non-vacuity and trusted-base planning.
8. Write one `verifier-lane-review/v1` row for every planner lane decision, with independent planner/reviewer invocation IDs and `reviewer_disposition: accepted` only for valid lanes.
9. Approve only when the plan is precise enough for proof-writer and proof-to-implementation.

## References

- `../go-skill/references/proof-schemas.md`
- `../go-skill/references/verification-lane-policy.md`
- `../go-skill/references/review-provenance.md`
- `../go-skill/references/finding-codes.md`
- `references/plan-review-rubric.md`
- `references/plan-rejection-catalog.md`

## Output Rules

`proof-plan-review.md` must include `reviewer_skill`, `reviewer_invocation_id`, `review_state`, reviewed artifacts/hashes, and `STATUS: APPROVED` or `STATUS: REJECTED`. `verifier-lane-review.jsonl` uses `verifier-lane-review/v1`.

Findings use `finding/v1`. If rejected, write exact repair instructions and the smallest state to rerun.
