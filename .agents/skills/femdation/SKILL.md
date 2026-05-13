---
name: femdation
description: "Aggressive work-conserving multi-bead dispatcher for the proof-first go-skill lifecycle. Keeps every unblocked bead active with direct specialist subagents while preserving gates, artifacts, retries, and landing order."
argument-hint: "[bead ids, bd query, or fleet goal]"
disable-model-invocation: true
allowed-tools:
  - Bash
  - Read
  - Task
---

```jsonl
{"kind":"meta","skill":"femdation","version":"2.0.0","format":"jsonl-progressive","mode":"aggressive-multi-bead-control-plane"}
{"kind":"mission","goal":"Maximize bead throughput by keeping every unblocked bead continuously assigned to a direct specialist child while preserving the canonical go-skill state order, artifact gates, retry limits, and landing semantics."}
{"kind":"principle","id":"work_conserving_fleet","text":"A ready bead without an active child, queued dispatch, verification step, serial landing wait, or explicit blocker is scheduler failure. Refill worker slots immediately after every child result."}
{"kind":"principle","id":"phase_integrity","text":"Parallelism is global across beads, not permission to skip per-bead phases. Each bead advances only through the current go-skill whole-number states and only after required artifacts and evidence pass."}
{"kind":"rule","id":"single_controller","text":"Run exactly one top-level femdation controller. Do not spawn master, go-skill, another femdation, per-bead controllers, or full-pipeline children."}
{"kind":"rule","id":"direct_children_only","text":"Every child is a direct specialist child for exactly one bead and one state or safe same-state sublane. Child prompts must forbid nested Task/opencode delegation and nested orchestrators."}
{"kind":"rule","id":"one_mutating_state_per_bead","text":"At most one mutating child may work on a bead at a time. Same-state fanout is allowed only for independent read-only reviews or independent verifier/gate lanes over frozen inputs."}
{"kind":"rule","id":"aggressive_fill","text":"Launch all currently ready independent dispatches in one batch when the environment supports parallel tool calls. Minimum target occupancy is one active dispatch per unblocked bead, plus safe same-state fanout."}
{"kind":"rule","id":"serialize_global_critical_sections","text":"Serialize landing to main/remote, shared global blocker repair, and any operation that mutates shared non-isolated state. Keep other beads moving while one bead waits for a serial section."}
{"kind":"rule","id":"retry_budget","text":"Inherit go-skill's 7-attempt cap per failed gate or review loop. Retries must route to the nearest invalidated state and record attempt, failure class, repair delta, evidence, and next dispatch."}
{"kind":"rule","id":"no_gate_weakening","text":"Throughput pressure never justifies weakening proof, test, review, truth-serum, machine, or landing gates. Block the bead and dispatch other ready beads instead."}
```

# Femdation Aggressive Multi-Bead Orchestrator

Use this skill when multiple beads should move through the current proof-first `go-skill` lifecycle concurrently.

Canonical lifecycle sources:
- `/home/lewis/.agents/skills/go-skill/SKILL.md`
- `/home/lewis/.agents/skills/go-skill/state-machine.md`
- `/home/lewis/.agents/skills/go-skill/checklist.md`
- `/home/lewis/.agents/skills/go-skill/artifacts.md`
- `/home/lewis/.opencode/agent/femdation.md`

If this skill conflicts with `go-skill`, `go-skill` wins for per-bead states, artifacts, gates, retry semantics, and landing requirements. `femdation` adds only fleet scheduling, child-dispatch discipline, and context hygiene.

## Runtime Table

Start every response with this table before prose:

| Bead ID | State | Active Child | Gate | Attempts | Queue | Status |
|---|---|---|---|---|---|---|

Every active bead must be in exactly one status: `READY`, `RUNNING`, `VERIFYING`, `SERIAL_WAIT`, `BLOCKED`, or `DONE`. If a bead is `READY`, dispatch it before doing explanatory work.

Also maintain a compact pool line: `Fleet: active=<n> ready=<n> blocked=<n> serial=<n> done=<n>`.

## Work-Conserving Scheduler

Use a fill-drain-refill loop:

1. Refresh the bead table from disk and bead metadata.
2. Verify or create isolated workspaces for all selected beads before specialist handoff.
3. Build a ready queue for every bead whose current state has satisfied inputs and no active child.
4. Dispatch every ready independent unit immediately, using one child per bead/state and safe same-state fanout where allowed.
5. When any child returns, verify artifacts on disk before updating the table.
6. If the gate passes, enqueue the bead's next state immediately; do not wait for a cohort.
7. If the gate fails, classify it, route repair to the nearest owning state, and keep other beads moving.

Do not walk one bead end-to-end while other unblocked beads wait. The fleet advances as a wavefront: each bead moves as soon as its own gate clears, without waiting for slower beads in earlier states.

## Parallelism Policy

- Default posture is aggressive: launch all independent ready children the runtime can support.
- Minimum posture is one active child for every unblocked bead.
- Add same-state fanout on top of the minimum when the sublanes consume frozen inputs and write separate artifacts.
- Do not voluntarily reduce parallelism for readability or comfort. Reduce only for real resource contention, tool locks, rate limits, shared-state mutation, or repeated infrastructure failures.
- When capacity is constrained, schedule round-robin by bead and prefer the oldest `READY` bead to prevent starvation.
- A noisy bead may consume at most one retry worker at a time; its failures must not stop unrelated beads.

Safe same-state fanout:

| State | Fanout |
|---|---|
| 6 | Run `proof-reviewer` and `contract-verification-reviewer` in parallel after proof artifacts are frozen. |
| 11 | Run machine-gate capture and `formal-verifier` in parallel when both consume the same frozen implementation/proof/test artifacts. |
| 12 | Run one `black-hat-reviewer` per bead in parallel across beads. |
| 13 | Run evidence packaging across beads in parallel; each bead still needs its own active-context truth/evidence approval before landing. |
| 15 | Cleanup verification may run in parallel after each bead has its own accepted landing evidence. |

Unsafe fanout:
- Do not run implementation, test-writing, proof-writing, or repair children concurrently for the same bead.
- Do not launch State 8 before State 7 approval, State 10 before State 9 approval, State 14 before State 13 approval, or cleanup before landing evidence.
- Do not parallelize main/remote landing for the same repository unless the landing mechanism provides a proven serialized merge queue.

## Specialist Matrix

Before dispatch, verify the specialist exists and the dispatch is for exactly one bead and one state.

| Go State | Delegate |
|---|---|
| 2 | `explore` |
| 3 | `rust-contract`; use `scott-ddd-refactor` only when the state requires domain/type-model repair |
| 4 | `proof-planner` |
| 5 | `proof-writer` |
| 6 | `proof-reviewer`, `contract-verification-reviewer` |
| 7 | `test-planner` |
| 8 | `test-writer` |
| 9 | `test-reviewer` |
| 10 | `holzman-rust` |
| 11 | `formal-verifier` plus controller machine-gate evidence capture |
| 12 | `black-hat-reviewer` |
| 13 | `evidence-packaging`, `truth-serum` |
| 14 | `landing-skill` |

States 1 and 15 are controller verification states. Keep them small, evidence-backed, and batched across beads where safe.

## Dispatch Manifest

Every child launch must have a manifest checked before dispatch:

```json
{"bead_id":"<id>","state":4,"sublane":"proof-planning","delegate":"proof-planner","source_checkout":"<forbidden-path>","isolated_workdir":"<required-path>","inputs":[".beads/<id>/contract.md"],"outputs":[".beads/<id>/proof-strategy.md"],"attempt":1}
```

The task title, prompt, workdir, artifact paths, bead claim/status, and child output must name the same single bead ID. Any mismatch invalidates the dispatch and routes the bead to repair from the earliest contaminated state.

Every child prompt must include this instruction:

`You are a direct child of the femdation controller. Work on exactly one bead and one go-skill state or approved sublane. Do not spawn sub-agents, invoke go-skill, invoke a master agent or skill, run nested opencode/Task delegation, or start another orchestrator. Use the isolated workspace only, write the required artifacts, and return raw evidence to femdation.`

## Workspace Rule

Each bead must have its own isolated jj workspace or approved worktree before any specialist handoff. The real isolated path must not equal the source checkout and must not be under the source checkout. A sibling under `~/src` is allowed only when it is outside the source checkout.

Overlap in touched files does not stop upstream exploration, contracts, proof planning, or test planning. It does require collision notes and may serialize implementation repair or landing if merge conflicts appear.

## Mandatory Verification Gate

Run the relevant checks before claiming the fleet is healthy or a bead advanced:

```bash
# Canonical lifecycle exists
test -s /home/lewis/.agents/skills/go-skill/state-machine.md
test -s /home/lewis/.agents/skills/go-skill/checklist.md
test -s /home/lewis/.agents/skills/go-skill/artifacts.md

# Current proof-first delegates exist
test -s /home/lewis/.agents/skills/explore/SKILL.md
test -s /home/lewis/.agents/skills/proof-planner/SKILL.md
test -s /home/lewis/.agents/skills/proof-writer/SKILL.md
test -s /home/lewis/.agents/skills/proof-reviewer/SKILL.md
test -s /home/lewis/.agents/skills/evidence-packaging/SKILL.md
test -s /home/lewis/.agents/skills/truth-serum/SKILL.md
test -s /home/lewis/.agents/skills/landing-skill/SKILL.md

# Per-bead workspace and artifact checks; run with workdir set to the isolated workspace
bd show <bead-id> --json
pwd -P
test "$(pwd -P)" = "<isolated-workspace-path-from-STATE>"
case "$(pwd -P)" in "<source-checkout-path-from-STATE>"|"<source-checkout-path-from-STATE>"/*) exit 1;; esac
test ! -e "<source-checkout-path-from-STATE>/.beads/<bead-id>"
test -s .beads/<bead-id>/STATE.md
test -s .beads/<bead-id>/baseline-report.md
jq -c . .beads/<bead-id>/delivery-scope.jsonl >/dev/null

# Current-state gates: run every matching row from go-skill state-machine.md, checklist.md, and artifacts.md.
# Paired approvals are all mandatory; one approved file never substitutes for another.
test -s .beads/<bead-id>/proof-review.md && test -s .beads/<bead-id>/contract-verification-review.md
rg -n '^STATUS: APPROVED$' .beads/<bead-id>/proof-review.md
rg -n '^STATUS: APPROVED$' .beads/<bead-id>/contract-verification-review.md
test -s .beads/<bead-id>/test-plan-review.md && test -s .beads/<bead-id>/test-suite-review.md
rg -n '^STATUS: APPROVED$' .beads/<bead-id>/test-plan-review.md
rg -n '^STATUS: APPROVED$' .beads/<bead-id>/test-suite-review.md
test -s .beads/<bead-id>/machine-gate-report.md && test -s .beads/<bead-id>/formal-verification-report.md && test -s .beads/<bead-id>/verification-ledger.jsonl
rg -n '^STATUS: PASS$|^STATUS: APPROVED$' .beads/<bead-id>/machine-gate-report.md
rg -n '^STATUS: PASS$|^STATUS: APPROVED$' .beads/<bead-id>/formal-verification-report.md
jq -c . .beads/<bead-id>/verification-ledger.jsonl >/dev/null
test -s .beads/<bead-id>/assurance-bundle.md && test -s .beads/<bead-id>/truth-serum-report.md && test -s .beads/<bead-id>/final-evidence-decision.md
rg -n '^STATUS: PASS$|^STATUS: APPROVED$' .beads/<bead-id>/truth-serum-report.md
rg -n '^STATUS: APPROVED$' .beads/<bead-id>/final-evidence-decision.md
```

For scheduler health, every active bead must have exactly one of: active child PID/session/task ID, queued dispatch manifest, verifier/artifact check in progress, serial landing wait, blocker with evidence, or terminal done status.

## Anti-Hallucination Shield

Forbidden:
- Claiming a child ran without a dispatch manifest and returned artifact evidence.
- Claiming a bead is busy when no child, queue entry, verification action, serial wait, or blocker exists.
- Advancing a bead from a specialist summary without checking required files on disk.
- Reusing a prompt, workdir, artifact path, or bead ID across parallel children.
- Letting throughput pressure skip proofs, tests, reviews, machine gates, truth/evidence approval, landing, or cleanup evidence.
- Mutating source checkout bead artifacts or letting specialists work outside their isolated workspace.

Required:
- Keep the runtime table current before prose.
- Dispatch all ready independent beads before long explanation.
- Verify artifacts after every child result.
- Record retry attempts and failure classes.
- Serialize only real global critical sections and keep unrelated beads moving.
