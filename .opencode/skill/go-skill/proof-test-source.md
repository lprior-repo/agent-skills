# Proof-Test-Source Alignment

Behavior-affecting proof claims need a closed chain:

`proof obligation -> Rust source refs -> independent behavior tests -> refinement harness/command evidence -> verification ledger`

## Non-Negotiable Rules

- `rust-refinement-obligations.jsonl` uses `rust-refinement-obligation/v1` from `references/proof-schemas.md`.
- State 7 may use `mapping_status: planned`; State 12 rejects `planned`.
- `source_refs` must be concrete `path::symbol` refs.
- `behavior_test_refs` must be independent executable behavior checks.
- `refinement_harness_refs` must be separate from behavior tests.
- Verifier harnesses are not behavior tests.
- Behavior-affecting Rust evidence cannot be waived.
- `evidence_command`, `evidence_workdir`, `evidence_artifact`, and `expected_evidence` are required.

## State 7 Bridge Artifacts

- `proof-to-rust-map.md`
- `rust-refinement-obligations.jsonl`
- `proof-to-rust-review.md STATUS: APPROVED` from `proof-reviewer`
- `proof-to-rust-repair-guide.md` when rejected

## Required Matrices

`proof-to-rust-map.md` must include:

| Proof ID | Claim | Behavior Affecting | Rust Source Refs | Behavior Test Refs | Refinement Harness Refs | Verifier | Evidence Command | Rerun From |
|---|---|---|---|---|---|---|---|---|

`test-plan.md` and `test-writer-report.md` must include a Proof/Refinement Coverage Matrix.

`implementation.md` must include a Source Coverage Matrix.

`proof-test-source-alignment.md` must include final parity:

| Requirement | Proof ID | Refinement ID | Source Refs | Behavior Test Refs | Refinement Harness Refs | Commands Run | Ledger Result | Status |
|---|---|---|---|---|---|---|---|---|

## Closure Failures

- `E_BRIDGE_REFS_NOT_DISJOINT`
- `E_SOURCE_REF_SHAPE`
- `E_MAPPING_PLANNED_AT_CLOSURE`
- `E_COMMAND_EVIDENCE_MISSING`
- `E_BEHAVIOR_WAIVER`
