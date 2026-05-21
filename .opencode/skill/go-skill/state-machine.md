# Go-Skill State Machine

Do not renumber states. Validator output beats Markdown. Work happens in the isolated bead workspace, never the source checkout.

## Core Order

`explore -> rust-contract -> proof-planner + proof-plan-reviewer -> proof-writer -> proof-reviewer -> proof-to-implementation -> test-planner -> test-writer -> test-reviewer -> holzman-rust -> formal-verifier -> black-hat-reviewer -> truth-serum/evidence -> landing -> cleanup`

## State Table

| State | Owner | Required Exit Evidence |
|---|---|---|
| 1 | orchestrator | bead claimed, isolated workspace, `STATE.md`, `runtime-skill-provenance.json`, `agent-invocation-ledger.jsonl`, baseline and global readiness reports, validator run |
| 2 | `explore` | `codebase-map.md`, `delivery-scope.jsonl` |
| 3 | `rust-contract` | `domain-model.md`, `type-contracts.md`, `workflow-model.md`, `error-taxonomy.md`, `boundary-map.md`, `hazard-analysis.md`, `contract.md`, `proof-seeds.jsonl`, `traceability-matrix.jsonl` |
| 4 | `proof-planner` then `proof-plan-reviewer` | `proof-strategy.md`, `verifier-lane-matrix.md`, `verifier-lane-decisions.jsonl`, `verifier-lane-review.jsonl`, `proof-coverage-matrix.md`, `proof-obligations.planned.jsonl`, `trusted-base-plan.md`, waiver candidates, `proof-plan-review.md STATUS: APPROVED` |
| 5 | `proof-writer` | proof/model/harness artifacts, `proof-writer-report.md`, `proof-evidence.md`, `trusted-base-ledger.jsonl`, smoke/typecheck evidence; `BLOCKED_TOOLING` is a blocker, not exit evidence |
| 6 | `proof-reviewer` | `proof-review.md STATUS: APPROVED`, `proof-findings.jsonl`, repair guide when rejected |
| 7 | `proof-to-implementation` then `proof-reviewer` | `proof-to-rust-map.md`, `rust-refinement-obligations.jsonl`, `proof-to-rust-review.md STATUS: APPROVED` |
| 8 | `test-planner` | `test-plan.md` with proof/refinement behavior coverage matrix |
| 9 | `test-writer` | failing-first behavior tests and `test-writer-report.md` |
| 10 | `test-reviewer` | `test-plan-review.md STATUS: APPROVED`, `test-suite-review.md STATUS: APPROVED`; tests only, no proof review |
| 11 | `holzman-rust` | implementation, `implementation.md`, source coverage matrix, Rust safety evidence |
| 12 | `formal-verifier` | machine gate reports, layer reports, `formal-verification-report.md`, `refinement-verification-report.md`, `proof-test-source-alignment.*`, `verification-ledger.jsonl`, valid `formal-waivers.jsonl` only for non-behavior exceptions |
| 13 | `black-hat-reviewer` | `black-hat-review.md STATUS: APPROVED` with proof/test/source parity matrix |
| 14 | `evidence-packaging` + `truth-serum` | `assurance-bundle.md`, `truth-serum-report.md`, `final-evidence-decision.md STATUS: APPROVED` |
| 15 | `landing-skill` | `landing-report.md` proving main integration, remote reachability, bead close/sync |
| 16 | orchestrator | `cleanup-report.md`, final `STATE.md`, workspace removed or preserved with blocker |

## Hard Routing Rules

- Every failed gate has at most 7 attempts and must route to the nearest invalidated state.
- `proof-plan-reviewer` replaces pre-proof `contract-verification-reviewer` duties.
- `proof-reviewer` replaces post-proof contract/proof adequacy review.
- `proof-to-implementation` owns State 7 bridge mapping artifacts; `proof-reviewer` owns the bridge review approval; TLA+ is not Rust evidence.
- `test-reviewer` reviews behavior tests only.
- Behavior-affecting waivers are forbidden.
- Verifier harnesses are never behavior tests.
- `PENDING_FORMAL_EXECUTION`, `mapping_status: planned`, and pending trusted-base dispositions must close by State 12.

## References

- `references/proof-pipeline-contract.md`
- `references/proof-schemas.md`
- `references/verification-lane-policy.md`
- `references/evidence-standards.md`
- `proof-test-source.md`
