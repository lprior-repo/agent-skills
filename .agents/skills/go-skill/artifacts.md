# Go-Skill Artifacts

## Artifact Root

All canonical artifacts live under:

`.beads/<bead-id>/`

This path is relative to the isolated per-bead workspace only. Never create, update, or verify canonical bead artifacts from the original/source checkout.

## Metadata Header

Canonical narrative artifacts should start with this header when practical:

```text
bead_id: <bead-id>
bead_title: <title from bd show>
phase: <state-id>
updated_at: <ISO-8601 UTC>
attempt: <n-of-7>
```

## Ownership Table

| Artifact | Writer | Primary Reader | Purpose |
|---|---|---|---|
| `STATE.md` | orchestrator | orchestrator | durable resume point, state, attempts, routing |
| `baseline-report.md` | orchestrator | orchestrator, verifier, repair agents | pre-edit repo-wide gate state used to distinguish regressions from old debt |
| `codebase-map.md` | `explore` | all later sub-agents | shared codebase context |
| `delivery-scope.jsonl` | orchestrator | all later sub-agents | compact bead scope: crates/files/APIs/dependencies/contracts/risk tags/required modes |
| `contract.md` | `rust-contract` | proof/test/implementation/review agents | requirements, assumptions, invariants, contract clauses |
| `domain-model-review.md` | `scott-ddd-refactor` or `rust-contract` | proof/test/implementation agents | domain model, illegal-state, type-boundary analysis |
| `tla-spec.md` | `rust-contract` | proof-planner, proof-writer, reviewers | TLA+ temporal model plan or non-applicability rationale |
| `lean-contract.md` | `rust-contract` | proof-planner, reviewers | theorem-kernel plan or explicit statement that another lane owns proof |
| `verification-layers.md` | `rust-contract` | proof-planner, formal-verifier, reviewers | defense-in-depth assignment for each contract clause |
| `proof-obligations.jsonl` | `rust-contract` | proof lifecycle, formal-verifier | initial machine-readable proof obligations |
| `traceability-matrix.jsonl` | `rust-contract` | all reviewers, evidence-packaging | maps requirements/clauses to proofs, tests, commands, and evidence |
| `proof-strategy.md` | `proof-planner` | proof-writer, proof-reviewer | verifier lane strategy, assumptions, budgets, waiver candidates |
| `proof-plan-review-input.md` | `proof-planner` | proof-reviewer, contract-verification-reviewer | compact review input for proof adequacy |
| `proof-obligations.planned.jsonl` | `proof-planner` | proof-writer, formal-verifier | planned/refined obligation rows with commands and expected evidence |
| `proof-writer-report.md` | `proof-writer` | proof-reviewer, orchestrator | changed verification artifacts, command attempts, assumptions, blockers |
| `proof-evidence.md` | `proof-writer` | proof-reviewer, evidence-packaging | raw proof command evidence summary and status by obligation |
| `proof-review.md` | `proof-reviewer` | orchestrator, proof-writer | proof approval/rejection gate |
| `proof-findings.jsonl` | `proof-reviewer` | orchestrator, proof-writer | machine-readable proof review findings |
| `proof-repair-guide.md` | `proof-reviewer` | proof-writer | required fixes when proof review rejects |
| `contract-verification-review.md` | `contract-verification-reviewer` | orchestrator, proof/test/implementation agents | independent approval of contract and proof obligation adequacy |
| `test-plan.md` | `test-planner` | test-writer, test-reviewer, implementer | exhaustive test strategy derived from contract/proofs |
| `test-writer-report.md` | `test-writer` | test-reviewer, orchestrator | failing-first test artifacts and red-phase evidence |
| `test-plan-review.md` | `test-reviewer` | orchestrator | test plan approval gate |
| `test-suite-review.md` | `test-reviewer` | orchestrator, test-writer | implemented test suite approval gate |
| `test-repair-guide.md` | `test-reviewer` | test-planner, test-writer | required fixes when test review rejects |
| `implementation.md` | `holzman-rust` | reviewers and orchestrator | implementation summary and clause/proof/test mapping |
| `machine-gate-report.md` | orchestrator | orchestrator, `formal-verifier`, `holzman-rust` | canonical machine-gate evidence |
| `regression-diff.md` | orchestrator | orchestrator, repair agents, reviewers | compares failures to baseline and classifies blockers vs deferred global debt |
| `compiler-errors.log` | orchestrator | `holzman-rust` | machine-gate failure output |
| `ci-failure-category.txt` | orchestrator | repair agents | targeted repair routing |
| `formal-verification-report.md` | `formal-verifier` | orchestrator, evidence-packaging | rollup decision for every required proof obligation |
| `verification-ledger.jsonl` | `formal-verifier` | orchestrator, evidence-packaging | machine-readable obligation ledger with result, evidence, waiver, rerun target |
| `formal-waivers.jsonl` | `formal-verifier` or orchestrator | orchestrator, evidence-packaging | explicit waived obligations with compensating evidence |
| `tla-report.md` | `formal-verifier` | orchestrator | TLA+ TLC/Apalache evidence |
| `verus-report.md` | `formal-verifier` | orchestrator | Verus proof evidence |
| `kani-report.md` | `formal-verifier` | orchestrator | Kani bounded model checking evidence |
| `flux-report.md` | `formal-verifier` | orchestrator | Flux refinement evidence |
| `loom-report.md` | `formal-verifier` | orchestrator | Loom concurrency evidence |
| `miri-report.md` | `formal-verifier` | orchestrator | Miri UB/provenance evidence |
| `black-hat-review.md` | `black-hat-reviewer` | orchestrator, repair agents | adversarial approval or rejection of spec/proof/test/code adequacy |
| `defects.md` | `black-hat-reviewer` | owning repair agents | repair input when black-hat rejects |
| `assurance-bundle.md` | `evidence-packaging` | truth-serum, landing-skill, user | final requirement-to-evidence map |
| `truth-serum-report.md` | `truth-serum` | orchestrator, landing-skill | audit of hallucinated/missing/laundered evidence |
| `final-evidence-decision.md` | `evidence-packaging` | orchestrator, landing-skill | final evidence approval/rejection decision |
| `landing-report.md` | `landing-skill` | orchestrator, user | main integration, remote push, bead close/sync evidence |
| `cleanup-report.md` | orchestrator | orchestrator, user | workspace cleanup proof or preserved-blocker state |

## Consumption Rules

- Downstream states read only approved upstream artifacts.
- If an artifact is missing, stale, empty, or contradicted by raw evidence, stop and repair the pipeline.
- `baseline-report.md` is captured before edits and must not be rewritten to hide regressions.
- `delivery-scope.jsonl`, `proof-obligations.jsonl`, `proof-obligations.planned.jsonl`, `traceability-matrix.jsonl`, `proof-findings.jsonl`, `verification-ledger.jsonl`, and `formal-waivers.jsonl` must be valid JSONL: exactly one JSON object per non-empty line.
- Proof obligation entries should include `id`, `requirement_id`, `contract_clause`, `risk`, `verifier`, `artifact`, `command`, `expected_evidence`, `assumptions`, `required`, `mode`, `owner_state`, `rerun_from`, `status`, and waiver data when applicable.
- `contract.md` and `traceability-matrix.jsonl` must exist before proof planning.
- `proof-review.md` and `contract-verification-review.md` must approve before test planning or implementation consumes proof artifacts.
- `test-plan-review.md` and `test-suite-review.md` must approve before implementation.
- `regression-diff.md` must classify failures as `BLOCK_LOCAL`, `BLOCK_REGRESSION`, `BLOCK_RELEASE`, `REQUIRED_OBLIGATION_FAIL`, or `DEFERRED_GLOBAL`.
- `formal-verification-report.md` must account for every required proof obligation by ID with pass, fail, waiver, or deferred-global evidence.
- `verification-ledger.jsonl` result values are `PASS`, `FAIL_LOCAL`, `FAIL_REGRESSION`, `WAIVED`, or `DEFERRED_GLOBAL`.
- `proof-review.md`, `contract-verification-review.md`, `test-plan-review.md`, `test-suite-review.md`, `formal-verification-report.md`, `black-hat-review.md`, and `final-evidence-decision.md` must include explicit status lines.
- `proof-repair-guide.md` is mandatory whenever `proof-review.md` says `STATUS: REJECTED`.
- `test-repair-guide.md` is mandatory whenever test review rejects.
- `defects.md` is mandatory whenever `black-hat-review.md` says `STATUS: REJECTED`.
- `assurance-bundle.md` must map every requirement to contract, proof/test evidence, review evidence, and final status.
- `truth-serum-report.md` must distinguish raw command evidence from subagent claims.
- `landing-report.md` must prove accepted code reached main and remote before cleanup.

## Non-Canonical Outputs

Sub-agent conversational summaries are not substitutes for artifact files. The filesystem artifact is the gate.
