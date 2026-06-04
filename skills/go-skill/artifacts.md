# Go-Skill Artifacts

All canonical artifacts live under `.beads/<bead-id>/` in the isolated workspace.

## Ownership

| Artifact | Owner | Purpose |
|---|---|---|
| `STATE.md` | orchestrator | durable state, attempts, routing |
| `runtime-skill-provenance.json` | orchestrator | proves loaded Go-skill version/state range |
| `agent-invocation-ledger.jsonl` | orchestrator/control plane | non-forgeable specialist/reviewer provenance |
| `baseline-report.md`, `global-readiness-report.md` | orchestrator | baseline and global blockers |
| `codebase-map.md`, `delivery-scope.jsonl` | `explore` | scoped context and risk tags |
| `domain-model.md`, `type-contracts.md`, `workflow-model.md`, `error-taxonomy.md`, `boundary-map.md`, `hazard-analysis.md`, `contract.md`, `proof-seeds.jsonl`, `traceability-matrix.jsonl` | `rust-contract` | domain/type contract and proof seeds |
| `proof-strategy.md`, `verifier-lane-matrix.md`, `verifier-lane-decisions.jsonl`, `proof-coverage-matrix.md`, `proof-obligations.planned.jsonl`, `trusted-base-plan.md`, `waiver-candidates.*`, `proof-to-implementation-input.md` | `proof-planner` | defense-in-depth proof plan |
| `proof-plan-review.md`, `verifier-lane-review.jsonl`, `proof-plan-findings.jsonl`, `proof-plan-repair-guide.md` | `proof-plan-reviewer` | pre-proof review gate and lane dispositions |
| `proof-writer-report.md`, `proof-evidence.md`, `trusted-base-ledger.jsonl`, proof/model/harness files | `proof-writer` | verification artifacts and debt |
| `proof-review.md`, `proof-findings.jsonl`, `proof-repair-guide.md`, `proof-to-rust-review.md`, `proof-to-rust-repair-guide.md` | `proof-reviewer` | post-proof and bridge adversarial review |
| `proof-to-rust-map.md`, `rust-refinement-obligations.jsonl` | `proof-to-implementation` | proof/source/test/harness bridge mapping |
| `test-plan.md` | `test-planner` | behavior-test plan |
| `test-writer-report.md` | `test-writer` | failing-first tests and evidence |
| `test-plan-review.md`, `test-suite-review.md`, `test-repair-guide.md` | `test-reviewer` | behavior-test review only |
| `implementation.md` | `holzman-rust` | production Rust and source coverage matrix |
| `machine-gate-report.md`, `regression-diff.md`, `formal-verification-report.md`, `refinement-verification-report.md`, `verification-ledger.jsonl`, `formal-waivers.jsonl`, layer reports, `proof-test-source-alignment.*` | `formal-verifier` | final command evidence and ledger closure |
| `black-hat-review.md`, `defects.md` | `black-hat-reviewer` | final parity attack |
| `assurance-bundle.md`, `truth-serum-report.md`, `final-evidence-decision.md` | `evidence-packaging`, `truth-serum` | assurance audit |
| `landing-report.md`, `cleanup-report.md` | `landing-skill`, orchestrator | landing and cleanup |

## Machine Ledger Rules

- JSONL artifacts use canonical schemas in `references/proof-schemas.md`.
- Legacy proof aliases `layer`, `checker`, and alias-only `claim` are invalid in v1 proof obligations.
- `proof-plan-review.md`, `proof-review.md`, `proof-to-rust-review.md`, test reviews, black-hat, truth-serum, and final evidence decisions need explicit status lines and reviewer provenance where independent review is required.
- Validator findings block state advancement.
