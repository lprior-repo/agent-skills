# Go-Skill Checklist

## Preflight

- Resolve the target bead ID from user input before touching the workspace.
- If the user supplied a bead ID, do not swap it for another ready bead.
- Claim the bead before planning, proof work, tests, or implementation.
- Record the source checkout path before creating the bead workspace.
- Create an isolated jj workspace or approved worktree outside the source checkout; a sibling under `~/src` is allowed only when it is not the source checkout and not below it.
- Verify the isolated path is not equal to the source checkout and does not match `<source-checkout>/*` before writing artifacts or launching sub-agents.
- Initialize `.beads/<bead-id>/STATE.md` immediately inside the isolated workspace with `source_checkout`, `isolated_workspace`, current state, and retry counters.
- Capture `.beads/<bead-id>/baseline-report.md` in the isolated workspace before implementation, proof, or test edits.
- Use the tool `workdir` field set to the isolated workspace over shell `cd` chains for every post-State-1 command.

## Transition Rules

- Update `STATE.md` at the start of each state.
- Verify the promised artifact exists and is non-empty before advancing.
- If a sub-agent says it is done but the artifact is missing, treat the state as failed.
- Carry `codebase-map.md` forward once it exists.
- If `codebase-map.md` cannot be produced, record that best-effort failure in `STATE.md` before continuing.
- Never invoke `red-queen` in this pipeline.
- Contract artifacts must exist before proof planning.
- Proof planning/writing/review must complete before test planning/writing/review.
- Proof review failure loops to State 5 with `proof-repair-guide.md`, or State 4/3 if the review says the plan/contract is wrong.
- Test review failure loops to State 7 or 8 with `test-repair-guide.md`.
- Implementation starts only after proof and test reviews are approved.
- Formal proof/test execution starts only after implementation and approved proof/test artifacts exist.
- Black-hat rejection routes every defect to its owning state; rerun affected downstream states after repair.
- Truth-serum evidence approval is required before landing-skill runs.
- Landing-skill must get accepted code onto main and remote; Go-skill verifies, it does not merely mark ready.

## Retry Policy

- Each failed gate/review loop has at most 7 attempts.
- Attempts are per failing gate class, not blind whole-pipeline restarts.
- Each attempt must record attempt number, failed gate, primary failure class, repair delta, command/artifact evidence, and next state.
- Re-running without a repair delta is not a valid repair attempt; it is evidence collection.
- Attempt 7 failure blocks landing, preserves the isolated workspace, and requires a blocker report.

## Proof Loop Checks

- `proof-strategy.md` exists and maps risk tags to verifier lanes.
- `proof-obligations.planned.jsonl` is valid JSONL and includes commands, expected evidence, required flags, assumptions, owner_state, and rerun_from.
- `proof-writer-report.md` names changed verification artifacts and proof obligation IDs.
- `proof-evidence.md` records commands, exit statuses, blocked tooling, assumptions, bounds, and artifact paths.
- `proof-review.md` says `STATUS: APPROVED` before tests begin.
- `contract-verification-review.md` says `STATUS: APPROVED` before tests or implementation consume proof artifacts.
- Any proof rejection includes `proof-repair-guide.md`.

## Test Loop Checks

- `test-plan.md` maps requirements to tests and proof obligations.
- `test-writer-report.md` records failing-first evidence.
- `test-plan-review.md` says `STATUS: APPROVED`.
- `test-suite-review.md` says `STATUS: APPROVED`.
- Any test rejection includes `test-repair-guide.md` and routes to State 7 or 8.
- Red/failing-first tests that are green before implementation are a stop condition, not success.

## Scope-Aware Blocking Policy

- Bead-local defects block.
- New failures relative to `baseline-report.md` block.
- Required proof-obligation failures block.
- Touched unsafe, panic, API, dependency, and contract-policy failures block.
- Dependency audit/deny/vet/geiger evidence is required only when dependency files, feature flags, build scripts, vendored code, or dependency policy files changed.
- Release/critical beads block on global failures selected by release policy.
- Pre-existing unrelated repo-wide failures are `DEFERRED_GLOBAL`: record exact evidence and follow-up text, but do not restart unrelated bead work.
- Unknown risk escalates to stricter local verification; it never downgrades to a cheaper lane.

## CI Failure Classification

When State 11 fails, record the first primary category in `.beads/<bead-id>/ci-failure-category.txt`:

- `BANNED_ASSERTION`
- `CLIPPY`
- `COMPILE_ERROR`
- `TEST_FAILURE`
- `CONTRACT_PARITY`
- `FORMAT`
- `PROOF_FAILURE`
- `MISSING_TOOLING`
- `BLACK_HAT_DEFECT`
- `EVIDENCE_GAP`

Use that category in the repair prompt instead of a generic "fix CI" instruction.

Then classify blocking scope in `.beads/<bead-id>/regression-diff.md`:

- `BLOCK_LOCAL`: failure is in delivery scope.
- `BLOCK_REGRESSION`: failure is new compared with baseline.
- `BLOCK_RELEASE`: failure is global and release/critical policy requires it clean.
- `REQUIRED_OBLIGATION_FAIL`: proof/test obligation has no passing evidence or valid waiver.
- `DEFERRED_GLOBAL`: unrelated pre-existing global debt; create follow-up text.

## Specialist Ownership Matrix

- Scope and orchestration: `go-skill` owns state order, artifact existence, status-line checks, retry counters, failure classification, and repair routing.
- Codebase mapping: `explore` owns `codebase-map.md`.
- Requirements/contracts/types: `rust-contract` owns requirements, assumptions, invariants, verification layers, traceability, and initial proof obligations.
- DDD/type model: `scott-ddd-refactor` owns illegal-state and domain model critique/repair guidance when needed.
- Proof planning: `proof-planner` owns verifier choice, planned commands, required flags, assumptions, and proof strategy.
- Proof writing: `proof-writer` owns verification artifacts only; it never edits production behavior.
- Proof review: `proof-reviewer` owns vacuity, assumption, bound, harness, model, and evidence adequacy rejection.
- Contract/proof adequacy review: `contract-verification-reviewer` owns binary approval of contract and obligation adequacy.
- Test strategy/writing/review: `test-planner`, `test-writer`, and `test-reviewer` own test design and assertion doctrine.
- Rust source safety/performance: `holzman-rust` owns production Rust implementation and CI repair.
- Formal execution: `formal-verifier` owns command execution, tool availability, waiver validation, ledger results, and compact failure packets.
- Black-hat adversarial review: `black-hat-reviewer` owns the late attack on whether the right claims were proven/tested/implemented.
- Evidence audit: `evidence-packaging` and `truth-serum` own assurance bundle and hallucination checks.
- Landing: `landing-skill` owns merge/main/remote/bead close/sync behavior.

## Failure Packets

Use the compact `failure_packet` emitted by the owning specialist. Go-skill only routes it to the state named by `rerun_from`; it does not rewrite specialist rubrics.

A failure packet should include obligation/test/finding ID, owning state, rerun_from, exact command, last relevant output lines, changed artifacts, and next repair instruction.

## Landing Checklist

- isolated workspace exists and source checkout was not used for bead work
- `delivery-scope.jsonl`, `proof-obligations.jsonl`, `proof-obligations.planned.jsonl`, `traceability-matrix.jsonl`, and `verification-ledger.jsonl` parse as JSONL
- contract artifacts exist and are approved
- proof strategy exists
- proof-writer report and proof evidence exist
- proof review is approved
- contract-verification review is approved
- test plan exists
- test-plan review is approved
- test-suite review is approved
- implementation report exists
- `machine-gate-report.md` exists and records the canonical `verify-standard` or project-equivalent gate evidence
- deeper/proof/all modes ran only when risk or obligations required them
- formal verification report is approved and every required obligation has a ledger result
- every required ledger result is `PASS`, `WAIVED`, or non-blocking `DEFERRED_GLOBAL`
- black-hat review is approved
- assurance bundle exists and maps requirements to evidence
- truth-serum report exists
- final evidence decision says `STATUS: APPROVED`
- landing report proves main integration and remote reachability
- bead is closed and synced
- isolated workspace cleanup is verified or preserved with explicit blocker

## Anti-Footgun Rules

- Do not edit source code directly from the orchestrator role.
- Do not write proof artifacts directly from the orchestrator role.
- Do not do bead work, artifact writes, implementation, tests, proof work, QA, or repair inside the original/source checkout.
- Do not rewrite tests to manufacture green builds.
- Do not weaken proofs or assumptions to manufacture verification pass.
- Do not report success from conversational summaries alone.
- Do not treat a missing approval file as implied approval.
- Do not invoke Red Queen.
- Do not skip truth-serum evidence approval.
- Do not skip remote/main verification before workspace cleanup.
- Do not forget or delete an isolated workspace until `landing-report.md` proves accepted code reached main and remote.
