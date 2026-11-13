# State Machine Integration

Proof-planner sits at Go-skill State 4. Its inputs come from State 3 (rust-contract); its outputs flow to State 4b (proof-plan-reviewer), then downstream to State 5 (proof-writer), State 7 (proof-to-implementation), and State 12 (formal-verifier).

This reference defines the contract at each boundary.

## Position in the Pipeline

```
State 1  runtime-skill-provenance.json, agent-invocation-ledger.jsonl
State 2  explore
State 3  rust-contract     →  domain-model.md, type-contracts.md, workflow-model.md,
                              error-taxonomy.md, boundary-map.md, hazard-analysis.md,
                              contract.md, proof-seeds.jsonl, traceability-matrix.jsonl
State 4  proof-planner     →  proof-strategy.md, verifier-lane-matrix.md,
                              verifier-lane-decisions.jsonl, proof-coverage-matrix.md,
                              proof-obligations.planned.jsonl, trusted-base-plan.md,
                              waiver-candidates.md, waiver-candidates.jsonl,
                              proof-to-implementation-input.md
State 4b proof-plan-reviewer (independent)
State 5  proof-writer      →  proof artifacts (Verus, Kani, Flux, Loom, Miri, fuzz, proptest)
State 6  proof-reviewer
State 7  proof-to-implementation  →  proof-to-rust-map.md, rust-refinement-obligations.jsonl
State 8  test-planner
State 9  test-writer
State 10 test-reviewer
State 11 holzman-rust
State 12 formal-verifier   →  verification-ledger.jsonl, formal-verification-report.md,
                              refinement-verification-report.md, layer reports
State 13 black-hat-reviewer
State 14 truth-serum + evidence packaging
```

The planner's `owner_state` is `4`. Its `rerun_from` is `4` (the planner is the source of these artifacts).

## Inputs (from State 3)

The planner reads these artifacts:

| Input | Source | Purpose |
|---|---|---|
| `contract.md` | State 3 | Domain/type/workflow/error/boundary/hazard artifacts |
| `proof-seeds.jsonl` | State 3 | `(requirement_id, contract_clause, domain_claim, risk_tags, suggested_layers, behavior_affecting, model_boundary, notes)` tuples |
| `traceability-matrix.jsonl` | State 3 | `(requirement_id, contract_clause)` → implementation binding |
| `delivery-scope.jsonl` | State 3 | Bead scope, in/out, dependencies |
| `baseline-report.md` | State 1/2 | Existing global failures (so the plan doesn't reintroduce them) |
| `agent-invocation-ledger.jsonl` | State 1 | The planner's own `invocation_id` (required for review provenance) |

## Outputs (to State 4b and beyond)

| Output | Schema | Consumed by |
|---|---|---|
| `proof-strategy.md` | narrative | reviewer, downstream |
| `verifier-lane-matrix.md` | narrative | reviewer |
| `verifier-lane-decisions.jsonl` | `verifier-lane-decision/v1` | reviewer, formal-verifier |
| `proof-coverage-matrix.md` | narrative | reviewer |
| `proof-obligations.planned.jsonl` | `proof-obligation/v1` | proof-writer, formal-verifier |
| `trusted-base-plan.md` | narrative | proof-writer (ledger), formal-verifier |
| `waiver-candidates.md` | narrative | reviewer |
| `waiver-candidates.jsonl` | `waiver-candidate/v1` | reviewer, formal-verifier |
| `proof-to-implementation-input.md` | narrative | proof-to-implementation |

## Reviewer Handoff (State 4 → 4b)

The reviewer (`proof-plan-reviewer`) writes:

- `verifier-lane-review.jsonl` — one `verifier-lane-review/v1` row per planner lane decision with `reviewer_disposition: accepted` or `rejected`.
- `proof-plan-review.md` — narrative with `reviewer_skill`, `reviewer_invocation_id`, `review_state`, `STATUS: APPROVED` or `STATUS: REJECTED`.
- `proof-plan-findings.jsonl` — `finding/v1` rows for any rejection.

Provenance requirements:

- The reviewer's `invocation_id` MUST differ from the planner's `invocation_id` in `agent-invocation-ledger.jsonl`.
- The reviewer MUST record the SHA-256 hashes of the planner's artifacts at the time of review.
- The reviewer MUST NOT modify planner artifacts; if a repair is needed, the planner re-emits.

## Writer Handoff (State 4b → 5)

The writer (`proof-writer`) reads:

- Approved `proof-plan-review.md` with `STATUS: APPROVED`.
- Approved `verifier-lane-review.jsonl` rows with `reviewer_disposition: accepted` for every required lane.
- `proof-obligations.planned.jsonl` rows with `status: planned`.
- `trusted-base-plan.md` (planner's contribution).
- `waiver-candidates.jsonl` approved rows (the writer records matching `trusted-base-ledger/v1` rows).

The writer produces proof/model/harness artifacts only. It writes `proof-writer-report.md`, `proof-evidence.md`, and `trusted-base-ledger.jsonl`.

## Bridge Handoff (State 6 → 7)

After the proof reviewer approves, `proof-to-implementation` reads the planner's `proof-to-implementation-input.md` and materializes it into `rust-refinement-obligation/v1` rows.

Every behavior-affecting obligation needs a paired `rust-refinement-obligation/v1` row with `source_refs`, `behavior_test_refs`, `refinement_harness_refs`, and an `evidence_command`. The bridge is the bridge from proof claim to Rust source symbol.

## Closure (State 11 → 12)

`formal-verifier` runs the exact `command` from each `proof-obligation/v1` row and writes `verification-ledger/v1` rows with `PASS`, `FAIL_*`, or `WAIVED`. A behavior-affecting obligation cannot be `WAIVED`; only non-behavior obligations (covered by `waiver-candidates.jsonl` with `behavior_affecting: false`) can.

## Closure Discipline

At State 12 closure:

- `mapping_status: planned` is `E_MAPPING_PLANNED_AT_CLOSURE`. Every behavior-affecting obligation MUST have `mapping_status: verified`.
- `status: pending` (in `trusted-base-ledger.jsonl`) is `E_TRUST_PENDING_AT_CLOSURE`. Every trust disposition MUST be closed.
- `PENDING_FORMAL_EXECUTION` is `E_FORMAL_PENDING_AT_CLOSURE`. Every obligation MUST have a final PASS/FAIL/WAIVED row.

## Rerun Discipline

The planner's `rerun_from` is `4`. Any change to a planned obligation after State 4b invalidates the reviewer disposition and forces a rerun from State 4. Changes that require re-planning:

- Adding a new verifier or obligation.
- Changing a `target`, `command`, `workdir`, or `model_bounds`.
- Changing `behavior_affecting` or `required`.
- Adding or changing a trust marker.
- Adding or removing a waiver row.

Changes that do NOT require re-planning (these are accepted as cosmetic and do not invalidate the reviewer):

- Renaming an artifact path that the writer will produce (the writer reports the actual path).
- Adding additional `expected_evidence` text without changing the marker.
- Adding additional `tool_metadata` keys.

The reviewer and writer decide what is cosmetic; the planner documents the discipline in `proof-strategy.md`.

## Cross-Reference

- `references/handoff-contract.md` — full handoff contract.
- `../go-skill/references/proof-pipeline-contract.md` — canonical state ownership.
- `../go-skill/references/review-provenance.md` — provenance requirements.
- `references/plan-quality-gates.md` — gates the planner runs before handoff.
