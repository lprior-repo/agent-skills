# Evidence Audit Checklist

Approve only if:
- Every required artifact exists and is non-empty.
- JSONL artifacts parse one object per line.
- Each requirement maps to at least one proof or test evidence row.
- Every proof obligation has PASS, WAIVED, or non-blocking DEFERRED_GLOBAL with reason.
- Every waiver has owner, reason, expiry/follow-up, and compensating evidence.
- Black-hat review is approved or all defects have repair evidence.
- Truth-serum ran in the active context or the bundle is marked UNVERIFIED.
- Landing has not happened before evidence approval.

Reject if:
- A subagent summary is used as command evidence.
- Paths referenced by the bundle do not exist.
- A required command is missing output or exit status.
- Tests/proofs were modified after their reviews without rerunning affected gates.
- Any status line is missing, contradictory, or unsupported by raw evidence.
