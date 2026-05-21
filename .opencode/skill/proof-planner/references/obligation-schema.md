# Proof Obligation Schema

Use one JSON object per line.

Required fields:

```json
{"id":"PO-001","requirement_id":"REQ-001","contract_clause":"C-001","risk":"temporal_retry","verifier":"tla-plus","artifact":"specs/retry_model.tla","command":"java -jar tla2tools.jar specs/retry_model.tla -config specs/retry_model.cfg","expected_evidence":"TLC reports invariant and deadlock checks pass","assumptions":["bounded retries <= 3"],"required":true,"mode":"verify-proof","owner_state":6,"rerun_from":6,"status":"planned","waiver":null}
```

Status values:
- `planned`: proof-writer must create or repair the artifact.
- `blocked_tooling`: required tool is unavailable; include install or environment need.
- `waived`: accepted only with owner, reason, expiry, compensating evidence, and follow-up.
- `not_applicable`: risk is absent and rationale is explicit.

Planner rows must not use `PASS`. Only execution tools can produce pass evidence.
