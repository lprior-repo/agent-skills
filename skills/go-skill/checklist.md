# Go-Skill Checklist

## Preflight

- Resolve bead ID and claim it.
- Load `go-skill` and write `runtime-skill-provenance.json`.
- Create isolated workspace outside the source checkout.
- Initialize `STATE.md` and `agent-invocation-ledger.jsonl`.
- Capture `baseline-report.md` and `global-readiness-report.md`.
- Run `tools/go-skill-v9-validate` before every state advance.

## Proof Planning Gates

- `rust-contract` emits domain/type/workflow/error/boundary/hazard artifacts and `proof-seeds.jsonl`; it does not emit proof obligations.
- `proof-planner` emits `verifier-lane-decisions.jsonl` for every `(requirement_id, contract_clause, proof_seed_id, verifier)` tuple.
- `proof-obligations.planned.jsonl` uses `proof-obligation/v1` and no legacy aliases.
- `proof-plan-reviewer` approves before proof writing and writes accepted `verifier-lane-review.jsonl` rows for every lane.
- Behavior-affecting waivers are invalid.

## Proof Writing And Review Gates

- `proof-writer` edits proof/model/harness artifacts only.
- Every trust marker has a `trusted-base-ledger.jsonl` row.
- Every touched proof artifact has smoke/typecheck evidence; `BLOCKED_TOOLING` blocks advancement.
- `proof-reviewer` approves before bridge/test/implementation work.
- `PENDING_FORMAL_EXECUTION` without smoke evidence rejects.

## Bridge Gates

- `proof-to-implementation` owns `proof-to-rust-map.md` and `rust-refinement-obligations.jsonl`; `proof-reviewer` owns `proof-to-rust-review.md`.
- Behavior-affecting rows need concrete source refs, independent behavior tests, and separate refinement harness refs.
- Verifier harnesses are not behavior tests.
- `mapping_status: planned` is allowed at State 7 and rejected at State 12.

## Test Gates

- `test-reviewer` reviews behavior tests only.
- Tests require sharp assertions, exact error variants, deterministic execution, and public API integration coverage.
- Proof harnesses do not satisfy behavior coverage.

## Formal Closure Gates

- `formal-verifier` writes `verification-ledger/v1` rows joined by `obligation_id`.
- Required proof/refinement/test obligations close as `PASS`, `FAIL_LOCAL`, `FAIL_REGRESSION`, `FAIL_GLOBAL`, or valid non-behavior `WAIVED`.
- `formal-waivers.jsonl` is required for every final `WAIVED` row.
- Pending formal execution, planned mappings, and pending trusted-base dispositions fail closure.

## Landing Gates

- Black-hat review includes proof/test/source parity matrix.
- Truth-serum distinguishes raw evidence from agent claims.
- Assurance bundle maps every requirement to contract, proof/refinement/test/source evidence.
- Landing report proves main integration, remote reachability, bead close/sync, and cleanup.
