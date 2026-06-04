# Proof Obligation Schema

Use one JSON object per line.

Required fields:

```json
{"id":"PO-001","requirement_id":"REQ-001","contract_clause":"C-001","risk":"bounded_transition","verifier":"kani","artifact":"harnesses/kani/transition.rs","command":"cargo kani --harness transition_obligation --output-format=regular","expected_evidence":"Kani reports successful verification for the named harness","assumptions":["input length <= 8"],"required":true,"mode":"verify-proof","owner_state":6,"rerun_from":6,"status":"planned","waiver":null}
```

Status values:
- `planned`: proof-writer must create or repair the artifact.
- `blocked_tooling`: required tool is unavailable; include install or environment need.
- `waived`: accepted only with owner, reason, expiry, compensating evidence, and follow-up.
- `not_applicable`: risk is absent and rationale is explicit.

Planner rows must not use `PASS`. Only execution tools can produce pass evidence.
