# Review Provenance

Independent review requires evidence outside the reviewed artifact.

## Required Artifacts

- `agent-invocation-ledger.jsonl`
- Review Markdown with `reviewer_skill`, `reviewer_invocation_id`, `review_state`, `reviewed_artifacts`, reviewed hashes, and `STATUS`.
- Finding JSONL with `schema_version: finding/v1`.

## Rejection Rules

Reject when the review has no matching invocation-ledger row, hash chain fails, `entry_hash` does not match the canonical row hash excluding `entry_hash`, transcript is missing or hash-mismatched, artifact hashes do not match, reviewed artifacts did not exist before review start, writer and reviewer invocation IDs match, or the same skill self-approves where independent review is required.

When the host exposes a control-plane ledger, the workspace copy is only a mirror and must match the authoritative source.
