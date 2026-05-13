# Go-Skill State Machine

## Naming Rule

When launching sub-agents, use task titles shaped like:

`[<bead-id>] p<state>-<verb>: <goal>`

That keeps artifacts and logs auditable.

## Core Order

The lifecycle is:

`explore -> contract -> proof plan -> proof write -> proof review loop -> test plan -> test write -> test review loop -> implementation -> formal proof/test execution -> black-hat review -> truth-serum evidence -> landing`

Use whole-number state IDs only: `1` through `15`. Do not use fractional labels, merged labels, renumbered aliases, or skipped numbers.

Every failed gate or review loop has a hard cap of `7` total attempts. Each attempt must record the failing gate, attempt number, repair delta, evidence path/command, and next routing. Attempt 7 failure blocks landing and preserves the workspace as evidence.

When verification fails, repair the nearest invalidated state. Do not rewind to implementation unless behavior or production code is wrong. Strictness is bead-local first and global-ratchet second: `BLOCK_LOCAL`, `BLOCK_REGRESSION`, `BLOCK_RELEASE`, and `REQUIRED_OBLIGATION_FAIL` stop the bead; pre-existing unrelated repo-wide debt becomes `DEFERRED_GLOBAL` follow-up evidence.

## Workspace Isolation Invariant

The checkout where Go-skill was invoked is the source checkout and is control-plane only. Bead artifacts, proof artifacts, tests, implementation changes, repair edits, QA evidence, and specialist work must happen in a per-bead jj workspace or approved worktree outside that source checkout.

State 1 must write `source_checkout`, `isolated_workspace`, and command evidence proving the isolated path is neither equal to nor nested under the source checkout. If that proof is missing, no later state may run.

| State | Name | Primary Action | Required Evidence Before Exit | Retry Budget |
|---|---|---|---|---|
| 1 | Isolation and baseline | `bd show`, `bd update --claim`, create isolated jj workspace or approved external worktree, initialize `.beads/<bead-id>/STATE.md`, capture pre-edit baseline | claimed bead, isolated workspace path proof, `STATE.md`, `baseline-report.md` | 7 |
| 2 | Explore and scope | Launch `explore`; write `codebase-map.md` and `delivery-scope.jsonl` | `codebase-map.md` or best-effort note; valid `delivery-scope.jsonl` with crates/files/APIs/dependencies/contracts/risk tags/required modes | 7 |
| 3 | Contract and type model | Launch `rust-contract`; use `scott-ddd-refactor` when the domain/type model is unclear or illegal states are representable | `contract.md`, `domain-model-review.md` when applicable, `tla-spec.md`, `lean-contract.md`, `verification-layers.md`, `proof-obligations.jsonl`, `traceability-matrix.jsonl` | 7 |
| 4 | Proof planning | Launch `proof-planner` to choose verifier lanes and proof strategy | `proof-strategy.md`, `proof-plan-review-input.md`, `proof-obligations.planned.jsonl` | 7 |
| 5 | Proof/model/harness writing | Launch `proof-writer` to write verification artifacts only | `proof-writer-report.md`, `proof-evidence.md`, and required proof/model/harness artifacts or `BLOCKED_TOOLING` evidence | 7 |
| 6 | Proof and contract review | Launch `proof-reviewer` and `contract-verification-reviewer` | `proof-review.md` and `contract-verification-review.md` say `STATUS: APPROVED`; if rejected, `proof-repair-guide.md` exists and routes to State 5 | 7 |
| 7 | Test planning | Launch `test-planner` using contract, traceability, and approved proof obligations | `test-plan.md` exists and maps requirements to test cases and proof obligations | 7 |
| 8 | Test writing | Launch `test-writer` for failing-first tests; Red Queen is forbidden | failing tests exist, red/failing-first evidence captured, `test-writer-report.md` exists | 7 |
| 9 | Test review | Launch `test-reviewer` for plan and suite review | `test-plan-review.md` and `test-suite-review.md` say `STATUS: APPROVED`; if rejected, `test-repair-guide.md` exists and routes to State 7 or 8 | 7 |
| 10 | Implementation | Launch `holzman-rust` for safe Rust implementation | `implementation.md` exists, code changes map to contract/test/proof obligations, and no production unsafe/panic discipline violations are knowingly introduced | 7 |
| 11 | Formal proof and test execution | Run canonical machine gates and launch `formal-verifier` for approved obligations | `machine-gate-report.md`, `regression-diff.md`, `formal-verification-report.md`, `verification-ledger.jsonl`, layer reports/waivers as applicable | 7 |
| 12 | Black-hat review | Launch `black-hat-reviewer` against requirements, proof evidence, tests, and implementation | `black-hat-review.md` says `STATUS: APPROVED`; if rejected, `defects.md` exists and routes each defect to an owning state | 7 |
| 13 | Truth-serum evidence packaging | Launch `evidence-packaging` and `truth-serum` audit | `assurance-bundle.md`, `truth-serum-report.md`, `final-evidence-decision.md` with `STATUS: APPROVED` | 7 |
| 14 | Landing to main and remote | Launch `landing-skill` to merge accepted work to main, push remote, close/sync bead | `landing-report.md` records main integration, remote reachability, bead close/sync, and exact command evidence | 7 |
| 15 | Cleanup and final resume state | Orchestrator verifies landing evidence, workspace cleanup, and final state | `cleanup-report.md`, final `STATE.md`, workspace gone or explicitly preserved with blocker | 7 |

## State Notes

### State 2

Run State 2 and every later state with the tool `workdir` set to the isolated workspace path recorded by State 1. Do not pass the source checkout path to sub-agents except as a forbidden path guard.

`delivery-scope.jsonl` is mandatory and compact. It must include touched crates, touched files or expected file globs, public APIs, changed dependencies, contract clauses, risk tags, required verifier modes, and whether the bead is release/critical. Unknown risk escalates to the next stricter local verifier lane; it does not downgrade to convenience.

### State 3

`rust-contract` owns requirements, assumptions, invariants, verification layers, traceability, and initial proof obligations. Use `scott-ddd-refactor` before or inside this state when the code model makes illegal states representable, hides workflows in booleans/options, or lacks parse-don't-validate boundaries.

If DDD/type-model repair changes production code before implementation, rerun State 2 scope and State 3 contract artifacts before proof planning.

### States 4-6: Proof Loop

Proof planning, writing, and review happen before test writing and implementation.

If `proof-review.md` says `STATUS: REJECTED`, do not continue to tests. Route to State 5 with `proof-repair-guide.md`. If the rejection says the plan is wrong rather than the proof artifact, route to State 4. If the contract is wrong, route to State 3.

Proof-review attempts are capped at 7. Every rejected attempt must record:

- attempt number
- rejected obligation IDs
- exact finding summary
- changed proof artifacts or explicit no-change blocker
- next state to run

Do not call proof complete from proof-writer claims alone. Proof work is accepted only after proof-review and contract-verification-reviewer approval.

### States 7-9: Test Loop

Tests derive from requirements, traceability, and approved proof obligations. State 8 is failing-first TDD, not Red Queen. Red Queen is explicitly forbidden in this lifecycle.

If `test-plan-review.md` or `test-suite-review.md` says `STATUS: REJECTED`, route to State 7 for plan defects or State 8 for test implementation defects. Do not route to implementation until tests are approved.

### State 10

`holzman-rust` owns production Rust. It must not alter proof/test artifacts to manufacture green gates. If implementation exposes a bad contract, proof obligation, or test oracle, route back to the owning earlier state and rerun downstream states.

### State 11

State 11 executes proof and test acceptance. Use the repository's canonical gates where available. `verify-standard` is the normal landing lane; `verify-deep`, `verify-proof`, or `verify-all` run only when `proof-obligations.jsonl`, `proof-obligations.planned.jsonl`, or `delivery-scope.jsonl` requires them.

State 11 must compare gate output against `baseline-report.md` and `delivery-scope.jsonl`:

- `BLOCK_LOCAL`: failure touches scoped crates/files/APIs/contracts/dependencies.
- `BLOCK_REGRESSION`: failure is new relative to baseline.
- `BLOCK_RELEASE`: global failure blocks because the bead is release/critical or explicitly workspace-scoped.
- `REQUIRED_OBLIGATION_FAIL`: required proof/test obligation has no passing evidence or valid waiver.
- `DEFERRED_GLOBAL`: pre-existing unrelated repo-wide failure; record exact evidence and create follow-up text.

Dependency audit/deny/vet/geiger evidence is required only when dependency files, feature flags, build scripts, vendored code, or dependency policy files changed.

### State 12

Black-hat review attacks whether the right thing was specified, proven, tested, and implemented. It does not replace deterministic proof/test gates.

If `black-hat-review.md` says `STATUS: REJECTED`, `defects.md` is mandatory. Route each defect to its owning state:

- wrong/missing requirement -> State 3
- weak proof plan -> State 4
- weak proof artifact -> State 5
- proof review miss -> State 6
- weak/missing test -> State 7 or 8
- implementation defect -> State 10
- execution/evidence gap -> State 11 or 13

After any code, test, proof, model, harness, or config change, rerun from the first affected downstream state.

### State 13

Evidence packaging is not a summary phase. It is an audit. `assurance-bundle.md` must map every requirement to contract clause, proof/test evidence, review evidence, command evidence, and final status. `truth-serum-report.md` must distinguish raw active-context evidence from subagent claims. Missing evidence yields `STATUS: REJECTED` or `STATUS: UNVERIFIED`, never approval.

### State 14

Landing is delegated to `landing-skill`. It must get accepted work onto main and remote, close/sync bead metadata, and write `.beads/<bead-id>/landing-report.md`. Go-skill must verify landing evidence; it must not treat "ready to land" as landed.

### State 15

Cleanup happens only after landing evidence exists. Verify:

- `landing-report.md` exists and records main/remote reachability
- bead close/sync evidence exists
- `jj workspace list` no longer contains the bead workspace or the workspace is intentionally preserved due to a blocker
- the isolated workspace directory is gone when cleanup was expected
- the original/source checkout was not used for `.beads/<bead-id>/` artifacts or bead code/test/proof edits

Write `cleanup-report.md` and final `STATE.md` with the terminal state, landing status, cleanup status, and any preserved blocker.
