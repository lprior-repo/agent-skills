# Evidence Audit Checklist

Approve only if:
- Every required artifact exists and is non-empty.
- JSONL artifacts parse one object per line.
- Each requirement maps to at least one proof or test evidence row.
- Every proof obligation has PASS or WAIVED, with no unresolved FAIL_GLOBAL/BLOCK_GLOBAL evidence.
- Every waiver has owner, reason, expiry/follow-up, and compensating evidence.
- Black-hat review has `STATUS: APPROVED` after any repairs. Repair evidence without black-hat re-review is `STATUS: UNVERIFIED` or `STATUS: REJECTED`.
- Every reviewer finding at every severity uses a canonical `finding/v1.disposition`: `fixed_with_evidence`, `owner_approved_debt`, `owner_approved_no_action`, or `blocker`.
- Truth-serum ran in the active context or the bundle is marked UNVERIFIED.
- Landing has not happened before evidence approval.

Reject if:
- A subagent summary is used as command evidence.
- Paths referenced by the bundle do not exist.
- A required command is missing output or exit status.
- Tests/proofs were modified after their reviews without rerunning affected gates.
- Any status line is missing, contradictory, or unsupported by raw evidence.
- Any low, minor, observation, or informational finding is omitted or lacks disposition.
- Any blocker finding is packaged as approval instead of `STATUS: REJECTED` or `STATUS: UNVERIFIED`.
- Any finding uses a noncanonical disposition such as `waiver`, `deferred`, `later`, or free-form prose.
