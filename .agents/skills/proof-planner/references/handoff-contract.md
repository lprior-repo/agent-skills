# Handoff Contract

The planner hands off to three downstream skills at different states. Each handoff has a defined contract; missing artifacts block advancement. This reference is the canonical handoff shape.

## Handoff 1 — To `proof-plan-reviewer` (State 4 → State 4)

The planner submits the plan for independent review.

### Inputs the planner provides

- `proof-strategy.md` — narrative strategy, risk profile, lane selection rationale.
- `verifier-lane-matrix.md` — narrative table of every `(requirement_id, contract_clause, proof_seed_id)` and its required lane profile.
- `verifier-lane-decisions.jsonl` — every lane decision, schema-valid.
- `proof-coverage-matrix.md` — narrative mapping of each obligation to a contract clause and a behavior-affecting flag.
- `proof-obligations.planned.jsonl` — every obligation, schema-valid, target-bound, evidence-specific.
- `trusted-base-plan.md` — every trust marker, schema-keyed note.
- `waiver-candidates.md` and `waiver-candidates.jsonl` — non-behavior waiver rows.
- `proof-to-implementation-input.md` — bridge input stub.
- `agent-invocation-ledger.jsonl` row for the planner's own `invocation_id`.

### Inputs the reviewer requires from the planner

- Every artifact's SHA-256 hash recorded in the reviewer's review header.
- The planner's `invocation_id` from `agent-invocation-ledger.jsonl`.
- The bead ID, the current Go-skill state (4), and the prior state (3) with `rust-contract`'s `invocation_id`.

### Outputs the reviewer produces

- `proof-plan-review.md` with `STATUS: APPROVED` or `STATUS: REJECTED`.
- `verifier-lane-review/v1` rows with `reviewer_disposition` for every lane decision.
- `proof-plan-findings.jsonl` rows with `finding/v1` schema.

### Failure modes

- Plan submitted without SHA-256 hashes ⇒ `E_REVIEW_PROVENANCE_MISSING`.
- Plan submitted with planner's `invocation_id` matching the reviewer's ⇒ `E_REVIEW_SELF_APPROVAL` (fatal).
- Plan missing required artifacts ⇒ `E_PLAN_INCOMPLETE` and the reviewer cannot advance.

## Handoff 2 — To `proof-writer` (State 4 → State 5)

After the reviewer approves, the planner's obligations are the writer's contract.

### Inputs the writer requires

- Approved `proof-plan-review.md` with `STATUS: APPROVED`.
- Approved `verifier-lane-review/v1` rows with `reviewer_disposition: accepted` for every lane.
- `proof-obligations.planned.jsonl` rows with `status: planned`.
- `trusted-base-plan.md` (the planner's contribution; the writer maintains the ledger).
- `waiver-candidates.jsonl` approved rows (the writer records the matching `trusted-base-ledger/v1` rows).

### Inputs the writer may produce

- `trusted-base-ledger.jsonl` rows for trust markers discovered during writing.
- `proof-writer-report.md`.
- `proof-evidence.md`.

### Failure modes

- Writer runs a `proof-obligation/v1` whose lane decision is not accepted by the reviewer ⇒ `E_LANE_REVIEW_MISSING`.
- Writer introduces a new trust marker without a ledger row ⇒ `E_TRUST_UNLEDGERED_MARKER`.
- Writer relaxes a bound without authorization ⇒ `E_BOUND_RELAXED_UNAUTHORIZED`.

## Handoff 3 — To `proof-to-implementation` (State 6 → State 7)

After the proof reviewer approves the proof artifacts, the bridge skill materializes the planner's `proof-to-implementation-input.md` into `rust-refinement-obligation/v1` rows.

### Inputs the bridge requires

- Approved `proof-review.md` with `STATUS: APPROVED`.
- Approved `proof-findings.jsonl` rows.
- `proof-to-implementation-input.md` from the planner (the bridge stub).
- Behavior-affecting `proof-obligation/v1` rows from the plan.
- `agent-invocation-ledger.jsonl` for the bridge's own `invocation_id`.

### Outputs the bridge produces

- `proof-to-rust-map.md` — concrete source_refs, behavior_test_refs, refinement_harness_refs.
- `rust-refinement-obligations.jsonl` — `rust-refinement-obligation/v1` rows, one per behavior-affecting obligation.

### Failure modes

- Bridge row has file-only `source_refs` ⇒ `E_SOURCE_REF_SHAPE`.
- Bridge row has `behavior_test_refs == refinement_harness_refs` ⇒ `E_BEHAVIOR_TEST_NOT_INDEPENDENT`.
- Bridge row missing `refinement_harness_refs` ⇒ `E_REFINEMENT_HARNESS_MISSING`.
- Bridge row missing `evidence_command` ⇒ `E_COMMAND_EVIDENCE_MISSING`.
- Behavior-affecting obligation without a bridge row at State 12 ⇒ `E_PROOF_TO_RUST_MISSING` and `E_MAPPING_PLANNED_AT_CLOSURE`.

## Handoff 4 — To `formal-verifier` (State 11 → State 12)

After implementation is complete and behavior tests pass, the formal verifier executes the obligations.

### Inputs the verifier requires

- Approved `proof-review.md`, `proof-to-rust-review.md`, `test-plan-review.md`.
- `proof-obligations.planned.jsonl` (status: planned at planning time, now status: ready).
- `rust-refinement-obligations.jsonl` (mapping_status: materialized at bridge time).
- `trusted-base-ledger.jsonl`.
- `formal-waivers.jsonl` for any waiver rows.
- `agent-invocation-ledger.jsonl` for the verifier's own `invocation_id`.

### Outputs the verifier produces

- `verification-ledger.jsonl` — one row per obligation with `PASS`, `FAIL_*`, or `WAIVED`.
- `formal-verification-report.md`.
- `refinement-verification-report.md` (if refinement obligations exist).
- Layer reports: `verus-report.md`, `kani-report.md`, `flux-report.md`, `loom-report.md`, `miri-report.md`.

### Failure modes

- Obligation with `behavior_affecting: true` has `WAIVED` without `formal-waiver/v1` ⇒ `E_BEHAVIOR_WAIVER`.
- Obligation with `verification-ledger/v1.status: pending` at closure ⇒ `E_FORMAL_PENDING_AT_CLOSURE`.
- Bridge row with `mapping_status: planned` at closure ⇒ `E_MAPPING_PLANNED_AT_CLOSURE`.
- Trusted-base row with `status: pending` at closure ⇒ `E_TRUST_PENDING_AT_CLOSURE`.

## Cross-Handoff Discipline

The four handoffs form a chain. The planner MUST design obligations that survive the chain:

- Every obligation has a `target` that the writer can produce, the bridge can map, and the verifier can execute.
- Every obligation has an `expected_evidence` that the verifier can match in raw output.
- Every obligation has `model_bounds` that the verifier can honor.
- Every trust marker has a `trusted-base-plan.md` row that the writer can ledger, the bridge can cite, and the verifier can close.

A plan that breaks any link in the chain is `E_PLAN_CHAIN_BROKEN` and the reviewer rejects.

## Mirror Parity

Before every handoff, the planner MUST verify byte-identity between `~/.agents/skills/proof-planner/` and `~/.opencode/skill/proof-planner/`. Mirror drift is `E_MIRROR_DRIFT`. The validator runs the mirror check; the planner MUST also run it manually before submission.

## Cross-Reference

- `../go-skill/references/proof-pipeline-contract.md` — state ownership.
- `../go-skill/references/proof-schemas.md` — schemas.
- `../go-skill/references/review-provenance.md` — provenance requirements.
- `../go-skill/references/finding-codes.md` — full finding-code vocabulary.
- `references/plan-quality-gates.md` — gates the planner runs before handoff.
- `references/implementation-binding.md`, `references/evidence-requirements.md`, `references/waiver-planning-guide.md` — gate source documents.
