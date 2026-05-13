---
name: go-skill
description: "go-skill bead delivery pipeline. Use when starting or resuming a bead through explore, contract, proof planning/writing/review, tests, implementation, formal execution, black-hat review, truth-serum evidence, and landing."
argument-hint: "[bead-id or delivery goal]"
disable-model-invocation: true
allowed-tools:
  - Bash
  - Read
  - Task
---

```jsonl
{"kind":"meta","skill":"go-skill","version":"8.0.0","format":"jsonl-progressive","mode":"manual-orchestration"}
{"kind":"input","arguments":"$ARGUMENTS","rule":"Treat arguments as an explicit bead ID or delivery goal. If an explicit bead ID is present, it is authoritative and MUST NOT be replaced with another bead from `bd ready`."}
{"kind":"mission","goal":"Act as a control-plane-only supervisor for bead delivery: isolate work, delegate to specialists, verify artifacts on disk, enforce proof/test/review loops, run machine gates, require truth-serum evidence, and land through landing-skill only when acceptance evidence is complete."}
{"kind":"principle","id":"untrusted_agents_trusted_gates","text":"AI agents do the work. Deterministic tools, adversarial reviewers, raw command evidence, and truth-serum decide whether the work is acceptable."}
{"kind":"rule","id":"control_plane_only","text":"Never implement production code, rewrite tests, write proof artifacts, or synthesize specialist reviews directly. Delegate those states via `Task`, then verify their filesystem artifacts and command evidence yourself."}
{"kind":"rule","id":"bd_and_jj_only","text":"Use `bd` for bead lifecycle and `jj` for version control. Do not use raw `git` except `jj git fetch`, or `git worktree` solely to create/remove an external isolated workspace when the repo is not jj-managed."}
{"kind":"rule","id":"no_main_repo_bead_work","text":"Never perform bead planning, artifact writes, tests, implementation, repair, QA, proof work, or specialist handoff in the current/main repo checkout. State 1 must create a jj workspace or approved worktree outside the source checkout. Every later command/sub-agent prompt must use that isolated path."}
{"kind":"rule","id":"workspace_path_guard","text":"Before any bead work, record source_checkout and isolated_workspace in `.beads/<bead-id>/STATE.md` from inside the isolated workspace. Refuse to continue if the isolated path equals the source checkout or is nested inside it."}
{"kind":"rule","id":"workdir_over_cd","text":"Use the tool `workdir` field set to the isolated workspace for every command after State 1. Do not rely on repeated directory-changing shell prefixes."}
{"kind":"rule","id":"artifact_gating","text":"Before advancing states, verify every required artifact exists on disk and is non-empty. A sub-agent claim without a file is a failed state."}
{"kind":"rule","id":"evidence_required","text":"Every state transition must be backed by command output, exit status, raw artifact evidence, or filesystem evidence captured in the current session. Fail closed on missing evidence."}
{"kind":"rule","id":"safe_rust_policy","text":"First-party Rust is safe Rust by default: `#![forbid(unsafe_code)]`. Unsafe-code needs are blockers requiring separate contract, proof, and design work outside this lifecycle; do not treat them as local exceptions. `holzman-rust` owns production Rust changes and zero unwrap/expect/panic/todo/unimplemented/unreachable/dbg discipline."}
{"kind":"rule","id":"no_red_queen","text":"Do not invoke `red-queen` in this pipeline. Accuracy comes from early proof lifecycle, deterministic gates, proof/test reviewers, black-hat review, and truth-serum evidence."}
{"kind":"rule","id":"proof_lifecycle_boundary","text":"Use proof-planner to choose proof lanes, proof-writer to write verification artifacts only, proof-reviewer plus contract-verification-reviewer to reject weak proof work, and formal-verifier to execute approved obligations. Go-skill only routes states and verifies artifacts."}
{"kind":"rule","id":"test_lifecycle_boundary","text":"Use test-planner, test-writer, and test-reviewer as a loop. If test review rejects, route back to the owning test state with the review guide instead of implementation."}
{"kind":"rule","id":"verification_mode_policy","text":"Normal landing requires the existing `verify-standard` lane where available. Deeper/proof/all lanes run only when proof-obligations.jsonl or delivery-scope.jsonl requires them. Missing required tools fail closed unless a valid waiver exists."}
{"kind":"rule","id":"dependency_gates_conditional","text":"Dependency assurance is not a standalone phase. If Cargo.toml, Cargo.lock, feature flags, build scripts, vendored code, or dependency policy files change, dependency audit/deny/vet/geiger evidence becomes part of State 11 acceptance."}
{"kind":"rule","id":"retry_policy_7","text":"Each failed gate/review loop gets at most 7 total attempts. Every attempt must record failed gate, attempt number, repair delta, command/artifact evidence, and next routing in STATE.md or the state artifact. Attempt 7 failure blocks landing."}
{"kind":"rule","id":"failure_classification","text":"Classify every failed gate as BLOCK_LOCAL, BLOCK_REGRESSION, BLOCK_RELEASE, REQUIRED_OBLIGATION_FAIL, WAIVED, or DEFERRED_GLOBAL. Only blocking classes stop the bead. DEFERRED_GLOBAL requires exact evidence and follow-up bead/work-item text."}
{"kind":"rule","id":"code_change_invalidation","text":"Any production code, test, proof, model, harness, or config change after a review/gate invalidates the nearest affected downstream approval. Rerun from the first affected state; do not rely on final QA ceremony."}
{"kind":"rule","id":"truth_serum_before_land","text":"State 13 must run evidence-packaging plus truth-serum. Landing is blocked unless final-evidence-decision.md says STATUS: APPROVED and points to raw evidence."}
{"kind":"workflow","id":"pipeline","steps":["EXPLORE: claim bead, isolate workspace, capture baseline, map code, and write delivery-scope.jsonl.","CONTRACT: turn bead request into requirements, assumptions, invariants, type/domain model, verification layers, and traceability.","PROOF LOOP: proof-planner chooses verifier lanes, proof-writer writes verification artifacts, proof-reviewer and contract-verification-reviewer reject weak or vacuous proof work; rejection loops back to proof-writer with proof-repair-guide.md.","TEST LOOP: test-planner creates a plan, test-writer creates failing-first tests, test-reviewer rejects weak tests; rejection loops back to test planning/writing.","IMPLEMENT: holzman-rust implements safe Rust against accepted contracts, proof obligations, and tests.","EXECUTE: run formal proof obligations and tests through formal-verifier plus canonical machine gates, then classify results against scope and baseline.","ATTACK: black-hat-reviewer attacks whether the right claims were proven and tested; defects route to their owning state.","EVIDENCE/LAND: evidence-packaging and truth-serum build the assurance bundle, then landing-skill gets accepted code onto main/remote and Go-skill verifies cleanup."]}
{"kind":"state_sequence","range":{"start":1,"end":15},"rule":"Use only whole-number state IDs from 1 through 15. Do not invent fractional, merged, skipped, or renamed state labels."}
{"kind":"state","id":1,"runner":"orchestrator","action":"claim bead, create isolated jj workspace or approved worktree outside source checkout, prove path isolation, initialize STATE.md, capture baseline-report.md","artifact":"STATE.md + baseline-report.md + source/isolated path proof"}
{"kind":"state","id":2,"runner":"subagent","use":"explore","action":"map code and create delivery-scope.jsonl with touched crates/files/APIs/dependencies/contracts/risk tags/required verifier modes","artifact":"codebase-map.md + delivery-scope.jsonl"}
{"kind":"state","id":3,"runner":"subagent","use":"rust-contract + scott-ddd-refactor as needed","action":"write requirements, assumptions, invariants, type/domain model notes, verification layers, initial proof obligations, and traceability","artifact":"contract.md + domain-model-review.md + tla-spec.md + lean-contract.md + verification-layers.md + proof-obligations.jsonl + traceability-matrix.jsonl"}
{"kind":"state","id":4,"runner":"subagent","use":"proof-planner","action":"turn contract and risk tags into verifier strategy and executable proof obligation plan","artifact":"proof-strategy.md + proof-plan-review-input.md + proof-obligations.planned.jsonl"}
{"kind":"state","id":5,"runner":"subagent","use":"proof-writer","action":"write or repair verification artifacts only: TLA+, Verus, Kani, Flux, Loom, Miri, proptest, fuzz targets as required","artifact":"proof-writer-report.md + proof-evidence.md + verification artifacts"}
{"kind":"state","id":6,"runner":"subagent","use":"proof-reviewer + contract-verification-reviewer","action":"review proof artifacts, assumptions, bounds, contract parity, and executable obligation adequacy; if rejected, write proof-repair-guide.md and loop to State 5","artifact":"proof-review.md + proof-findings.jsonl + proof-repair-guide.md when rejected + contract-verification-review.md"}
{"kind":"state","id":7,"runner":"subagent","use":"test-planner","action":"derive test plan from contract, traceability, and approved proof obligations","artifact":"test-plan.md"}
{"kind":"state","id":8,"runner":"subagent","use":"test-writer","action":"write failing-first tests for required behavior; explicitly do not invoke Red Queen","artifact":"failing tests in repo + test-writer-report.md"}
{"kind":"state","id":9,"runner":"subagent","use":"test-reviewer","action":"review test plan and implemented test suite; if rejected, write test-repair-guide.md and loop to State 7 or 8","artifact":"test-plan-review.md + test-suite-review.md + test-repair-guide.md when rejected"}
{"kind":"state","id":10,"runner":"subagent","use":"holzman-rust","action":"implement safe Rust against accepted contract, proof obligations, and tests","artifact":"implementation.md + code changes"}
{"kind":"state","id":11,"runner":"mixed","use":"formal-verifier + orchestrator","action":"execute approved proof obligations and canonical test/CI gates, capture machine-gate-report.md, write regression-diff.md, and classify blockers against scope/baseline","artifact":"formal-verification-report.md + verification-ledger.jsonl + machine-gate-report.md + regression-diff.md + layer reports/waivers"}
{"kind":"state","id":12,"runner":"subagent","use":"black-hat-reviewer","action":"attack whether requirements, proofs, tests, and implementation cover the real risk; defects route to the owning state and rerun downstream gates","artifact":"black-hat-review.md + defects.md when rejected"}
{"kind":"state","id":13,"runner":"subagent","use":"evidence-packaging + truth-serum","action":"build assurance-bundle.md, audit it with truth-serum, and reject missing/laundered evidence","artifact":"assurance-bundle.md + truth-serum-report.md + final-evidence-decision.md"}
{"kind":"state","id":14,"runner":"subagent","use":"landing-skill","action":"merge accepted work to main, push to remote, close/sync bead, and write landing-report.md with main/remote evidence","artifact":"landing-report.md with main and remote reachability proof"}
{"kind":"state","id":15,"runner":"orchestrator","action":"verify landing-skill evidence, verify workspace cleanup or perform approved cleanup after push/sync, and write final resume/cleanup status","artifact":"cleanup-report.md + final STATE.md"}
{"kind":"artifact","id":"canonical_artifacts","items":[".beads/<bead-id>/STATE.md",".beads/<bead-id>/baseline-report.md",".beads/<bead-id>/codebase-map.md",".beads/<bead-id>/delivery-scope.jsonl",".beads/<bead-id>/contract.md",".beads/<bead-id>/domain-model-review.md",".beads/<bead-id>/tla-spec.md",".beads/<bead-id>/lean-contract.md",".beads/<bead-id>/verification-layers.md",".beads/<bead-id>/proof-obligations.jsonl",".beads/<bead-id>/traceability-matrix.jsonl",".beads/<bead-id>/proof-strategy.md",".beads/<bead-id>/proof-plan-review-input.md",".beads/<bead-id>/proof-obligations.planned.jsonl",".beads/<bead-id>/proof-writer-report.md",".beads/<bead-id>/proof-evidence.md",".beads/<bead-id>/proof-review.md",".beads/<bead-id>/proof-findings.jsonl",".beads/<bead-id>/proof-repair-guide.md",".beads/<bead-id>/contract-verification-review.md",".beads/<bead-id>/test-plan.md",".beads/<bead-id>/test-writer-report.md",".beads/<bead-id>/test-plan-review.md",".beads/<bead-id>/test-suite-review.md",".beads/<bead-id>/test-repair-guide.md",".beads/<bead-id>/implementation.md",".beads/<bead-id>/machine-gate-report.md",".beads/<bead-id>/regression-diff.md",".beads/<bead-id>/compiler-errors.log",".beads/<bead-id>/ci-failure-category.txt",".beads/<bead-id>/formal-verification-report.md",".beads/<bead-id>/verification-ledger.jsonl",".beads/<bead-id>/formal-waivers.jsonl",".beads/<bead-id>/tla-report.md",".beads/<bead-id>/verus-report.md",".beads/<bead-id>/kani-report.md",".beads/<bead-id>/flux-report.md",".beads/<bead-id>/loom-report.md",".beads/<bead-id>/miri-report.md",".beads/<bead-id>/black-hat-review.md",".beads/<bead-id>/defects.md",".beads/<bead-id>/assurance-bundle.md",".beads/<bead-id>/truth-serum-report.md",".beads/<bead-id>/final-evidence-decision.md",".beads/<bead-id>/landing-report.md",".beads/<bead-id>/cleanup-report.md"]}
{"kind":"output","id":"response_shape","sections":["## Bead - bead ID, claim status, source checkout, isolated workspace","## State - current state reached and next gate","## Evidence - commands/files proving the gate passed","## Loops - retry count, failed gate, repair target","## Risks - missing artifacts, failed attempts, or blocked downstream states"]}
{"kind":"ref","file":"state-machine.md","use":"15-state execution flow, retry budgets, proof/test repair loops, and landing semantics."}
{"kind":"ref","file":"artifacts.md","use":"Canonical artifact names, ownership, schema expectations, and evidence consumption rules."}
{"kind":"ref","file":"checklist.md","use":"Preflight, transition, proof/test loop, machine gate, evidence, and landing checks."}
{"kind":"gate","id":"no_implicit_progress","text":"Do not skip, merge, or rename states unless state-machine.md explicitly permits it."}
{"kind":"gate","id":"repair_targeting","text":"When gates fail, classify the first failure and route repair to the nearest invalidated state instead of issuing generic fixes."}
{"kind":"gate","id":"landing_requires_truth_serum","text":"Do not invoke landing-skill until final-evidence-decision.md says STATUS: APPROVED and truth-serum-report.md exists."}
{"kind":"gate","id":"landing_requires_main_remote","text":"Do not declare completion until landing-report.md proves accepted code reached main and the remote, bead close/sync completed, and cleanup was verified."}
{"kind":"gate","id":"anti_hallucination","text":"Never invent artifact contents, command output, state transitions, sub-agent completion, verifier results, or landing evidence. Missing evidence blocks progress."}
```

# go-skill

Use this skill when a bead needs the full supervised high-assurance pipeline rather than ad hoc implementation.

The canonical state range is whole-number `State 1` through `State 15`. The pipeline is now proof-first: proof plan/write/review happens before tests and implementation. Red Queen is not part of this lifecycle.

Read these supporting docs as needed:
- [state-machine.md](state-machine.md) for the 15-state execution flow and retry rules.
- [artifacts.md](artifacts.md) for canonical files under `.beads/<bead-id>/`.
- [checklist.md](checklist.md) for preflight, transition, proof/test loop, evidence, and landing checks.

## Isolation Policy

The original checkout is control-plane only. Do not create bead artifacts, tests, proof artifacts, implementation changes, repair edits, QA outputs, or specialist work inside it.

State 1 must create a per-bead jj workspace or explicitly approved worktree outside the current/main checkout. After that, set every tool `workdir` and every sub-agent prompt to the isolated workspace path. If the isolated path cannot be created or verified as outside the source checkout, stop before doing bead work.

## Mandatory Verification Gate

Before advancing states or claiming completion, run the checks that match the current phase. This matrix is not an all-states preflight during early states. Set tool `workdir` to the isolated workspace.

```bash
# Bead and workspace reality checks
bd show <bead-id> --json
jj workspace list
pwd -P
test "$(pwd -P)" = "<isolated-workspace-path>"
case "$(pwd -P)" in "<source-checkout-path>"|"<source-checkout-path>"/*) exit 1;; esac
test -s ".beads/<bead-id>/STATE.md"
test -s ".beads/<bead-id>/baseline-report.md"

# JSONL gates
jq -c . ".beads/<bead-id>/delivery-scope.jsonl" >/dev/null
jq -c . ".beads/<bead-id>/proof-obligations.jsonl" >/dev/null
jq -c . ".beads/<bead-id>/traceability-matrix.jsonl" >/dev/null
jq -c . ".beads/<bead-id>/verification-ledger.jsonl" >/dev/null

# Artifact gates; select rows for the current state only
test -s ".beads/<bead-id>/contract.md"
test -s ".beads/<bead-id>/proof-strategy.md"
test -s ".beads/<bead-id>/proof-writer-report.md"
test -s ".beads/<bead-id>/proof-review.md"
test -s ".beads/<bead-id>/test-plan.md"
test -s ".beads/<bead-id>/test-plan-review.md"
test -s ".beads/<bead-id>/test-suite-review.md"
test -s ".beads/<bead-id>/implementation.md"
test -s ".beads/<bead-id>/machine-gate-report.md"
test -s ".beads/<bead-id>/formal-verification-report.md"
test -s ".beads/<bead-id>/verification-ledger.jsonl"
test -s ".beads/<bead-id>/black-hat-review.md"
test -s ".beads/<bead-id>/assurance-bundle.md"
test -s ".beads/<bead-id>/truth-serum-report.md"
test -s ".beads/<bead-id>/final-evidence-decision.md"
test -s ".beads/<bead-id>/landing-report.md"

# Approval checks
rg -n '^STATUS: APPROVED$' ".beads/<bead-id>/proof-review.md" ".beads/<bead-id>/contract-verification-review.md" ".beads/<bead-id>/test-plan-review.md" ".beads/<bead-id>/test-suite-review.md" ".beads/<bead-id>/formal-verification-report.md" ".beads/<bead-id>/black-hat-review.md" ".beads/<bead-id>/final-evidence-decision.md"

# Canonical verification lane before landing: formal-verifier owns exact verifier commands and records the executed command in machine-gate-report.md.
rg -n '^STATUS: PASS$|^STATUS: APPROVED$' ".beads/<bead-id>/machine-gate-report.md"
```

## Anti-Hallucination Shield

Forbidden:
- Claiming a sub-agent finished without the promised artifact on disk.
- Claiming a state passed without command evidence or file evidence.
- Claiming proof, test, review, or landing success from a conversational summary.
- Invoking Red Queen in this lifecycle.
- Treating truth-serum as optional before landing.
- Forgetting or deleting the isolated workspace before landing-report.md proves code reached main/remote.
- Doing bead work in the original/main checkout or writing `.beads/<bead-id>/` there.

Required:
- Quote the bead ID, source checkout path, and isolated workspace path.
- Name the current state, retry attempt, and next gate.
- Verify each required artifact before the next state consumes it.
- Route proof-review failure to State 5 with `proof-repair-guide.md`.
- Route test-review failure to State 7 or 8 with `test-repair-guide.md`.
- Route black-hat defects to the owning state, then rerun affected downstream gates.
