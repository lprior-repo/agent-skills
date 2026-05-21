---
name: femdation
description: "OpenCode bridge for aggressive work-conserving multi-bead dispatch through the proof-first go-skill lifecycle. Use when multiple beads should move concurrently while preserving per-bead gates, artifacts, retries, and landing order."
---

```jsonl
{"kind":"meta","skill":"femdation","version":"2.0.0","format":"jsonl-progressive","mode":"opencode-aggressive-multi-bead-control-plane"}
{"kind":"input","arguments":"$ARGUMENTS","rule":"Treat explicit bead IDs as authoritative. Do not replace them with unrelated `bd ready` work unless the user asked for a fleet-fill goal."}
{"kind":"mission","goal":"Maximize bead throughput by keeping every unblocked bead continuously assigned to a direct specialist child while preserving canonical go-skill states, artifacts, retry limits, and landing semantics."}
{"kind":"principle","id":"work_conserving_fleet","text":"A ready bead without an active child, queued dispatch, verification step, serial landing wait, or explicit blocker is scheduler failure."}
{"kind":"rule","id":"one_controller","text":"Run one top-level femdation controller only. Never spawn go-skill, master, another femdation, per-bead controllers, or full-pipeline children."}
{"kind":"rule","id":"direct_children_only","text":"Every child is exactly one specialist, one bead, and one go-skill state or safe same-state sublane. Children must not delegate further."}
{"kind":"rule","id":"one_mutating_state_per_bead","text":"At most one mutating child may work on a bead at a time. Same-state fanout is allowed only for independent read-only reviews or independent verifier/gate lanes over frozen inputs."}
{"kind":"rule","id":"aggressive_fill","text":"Minimum target occupancy is one active child for every unblocked bead, plus safe same-state fanout over frozen inputs."}
{"kind":"rule","id":"occupancy_target","text":"For 10 selected unblocked beads, target 10 active direct specialist children across their current lifecycle states. Anything less requires concrete evidence: blocked gate, serial landing wait, queued dispatch, active verification, resource cap, or terminal done."}
{"kind":"rule","id":"heterogeneous_wavefront","text":"Beads do not need to be in the same go-skill state. If one bead is in State 4 and another is in State 11, dispatch both ready state workers. Refill across mixed states until every selected bead has landed and completed cleanup."}
{"kind":"rule","id":"serialize_global_critical_sections","text":"Serialize landing to main/remote, shared global blocker repair, and any operation that mutates shared non-isolated state. Keep other beads moving while one bead waits for a serial section."}
{"kind":"rule","id":"retry_budget","text":"Inherit go-skill's 7-attempt cap per failed gate or review loop. Retries route to the nearest invalidated state and record attempt, failure class, repair delta, evidence, and next dispatch."}
{"kind":"rule","id":"no_gate_weakening","text":"Throughput pressure never justifies weakening proof, test, review, truth-serum, machine, or landing gates. Block the bead and dispatch other ready beads instead."}
{"kind":"rule","id":"phase_integrity","text":"Parallelism is global across beads. Per-bead state order, artifacts, approvals, and retry gates are never skipped or merged."}
```

# Femdation OpenCode Bridge

Use this skill to run many beads through the current proof-first `go-skill` lifecycle at maximum safe throughput. The scheduler is work-conserving: one active child for every unblocked bead is the minimum acceptable occupancy. Beads advance independently through mixed lifecycle states; the fleet keeps refilling workers until every selected bead lands and completes cleanup.

Invoke through OpenCode's native skill/agent routing. Do not shell out to launch agents; child delegation is Task-only.

Canonical lifecycle sources:
- `/home/lewis/.opencode/skill/femdation/SKILL.md`
- `/home/lewis/.agents/skills/femdation/SKILL.md`
- `/home/lewis/.agents/skills/go-skill/SKILL.md`
- `/home/lewis/.agents/skills/go-skill/state-machine.md`
- `/home/lewis/.agents/skills/go-skill/checklist.md`
- `/home/lewis/.agents/skills/go-skill/artifacts.md`
- `/home/lewis/.opencode/skill/go-skill/SKILL.md`
- `/home/lewis/.opencode/agent/femdation.md`

If this bridge conflicts with `/home/lewis/.agents/skills/femdation/SKILL.md`, this bridge wins only for OpenCode routing and tool adapter behavior. If either femdation source conflicts with `go-skill`, `go-skill` wins for per-bead lifecycle requirements. Femdation only adds fleet scheduling, child-dispatch discipline, and context hygiene.

## Runtime Table

Every response begins with:

| Bead ID | State | Active Child | Gate | Attempts | Queue | Status |
|---|---|---|---|---|---|---|

Statuses are `READY`, `RUNNING`, `VERIFYING`, `SERIAL_WAIT`, `BLOCKED`, and `DONE`. A `READY` bead must be dispatched before prose. Also print: `Fleet: active=<n> ready=<n> blocked=<n> serial=<n> done=<n>`.

## Scheduler Doctrine

- Use a fill-drain-refill loop: refresh state, dispatch every ready independent unit, verify returned artifacts, immediately enqueue next state or repair route.
- Target occupancy equals the count of selected unblocked beads: 10 selected unblocked beads means 10 active direct specialist children, each at its own current `go-skill` state.
- Do not walk one bead end-to-end while other ready beads wait.
- Do not wait for a cohort or same-phase wave. Each bead advances as soon as its own gate clears, even when other beads are in earlier or later states.
- Keep heterogeneous waves saturated: a State 4 bead, State 8 bead, and State 11 bead can all have active children at the same time if their own inputs and gates allow it.
- Continue refill/verify/refill until every selected bead reaches accepted landing evidence and cleanup completion.
- Launch all independent children the runtime can support. Reduce only for real tool contention, rate limits, locks, shared-state mutation, or repeated infrastructure failure.
- Use round-robin when capacity is constrained so one noisy bead cannot starve the fleet.
- Serialize landing to main/remote, shared global blocker repair, and other non-isolated global mutations. Keep unrelated beads moving while one bead waits.

## Safe Fanout

Same-bead fanout is allowed only over frozen inputs and separate outputs:

| State | Safe fanout |
|---|---|
| 4 | `proof-planner`, then `proof-plan-reviewer` after planner artifacts are frozen. |
| 6 | `proof-reviewer`; `contract-verification-reviewer` is historical and not a live gate. |
| 12 | Machine-gate capture and `formal-verifier` in parallel when both consume frozen artifacts. |
| 13 | `black-hat-reviewer` per bead, parallel across beads. |
| 14 | Evidence packaging across beads; each bead still needs truth/evidence approval before landing. |
| 16 | Cleanup verification in parallel after each bead has landing evidence. |

Never run implementation, proof-writing, test-writing, or repair concurrently for the same bead.
Never launch State 8 before State 7 approval, State 10 before State 9 approval, State 14 before State 13 approval, or cleanup before landing evidence.
Never parallelize main/remote landing for the same repository unless the landing mechanism provides a proven serialized merge queue.

## Specialist Matrix

Before dispatch, verify the specialist exists and the handoff is for exactly one bead and one state.

| Go State | Delegate |
|---|---|
| 2 | `explore` |
| 3 | `rust-contract`; `scott-ddd-refactor` only when domain/type-model repair is required |
| 4 | `proof-planner`, then `proof-plan-reviewer` |
| 5 | `proof-writer` |
| 6 | `proof-reviewer` |
| 7 | `proof-to-implementation`, then `proof-reviewer` for bridge review |
| 8 | `test-planner` |
| 9 | `test-writer` |
| 10 | `test-reviewer` |
| 11 | `holzman-rust` |
| 12 | `formal-verifier` plus controller machine-gate evidence capture |
| 13 | `black-hat-reviewer` |
| 14 | `evidence-packaging`, `truth-serum` |
| 15 | `landing-skill` |

States 1 and 16 are controller verification states. Batch them across beads where safe.

## Dispatch Integrity

Before every child handoff, create and check a manifest with `bead_id`, `state`, `sublane`, `delegate`, `source_checkout`, `isolated_workdir`, required inputs, expected outputs, and attempt number.

Use only native OpenCode `Task` agent handoff. Set `subagent_type` to the delegate and pass a compact prompt shaped as `[<bead-id>] p<state>-<verb>: <one-state goal with required inputs and outputs>`. If Task handoff is unavailable, block the bead with evidence instead of using a shell fallback.

The task title, prompt, workdir, artifact paths, bead status, and child output must name the same single bead. Any mismatch invalidates the transition.

Every child prompt must include:

`You are a direct child of the femdation controller. Work on exactly one bead and one go-skill state or approved sublane. Do not spawn sub-agents, invoke go-skill, invoke a master agent or skill, run nested opencode/Task delegation, or start another orchestrator. Use the isolated workspace only, write required artifacts, and return raw evidence to femdation.`

## Workspace Rule

Each bead must have its own isolated `jj` workspace or approved worktree before any specialist handoff. The real isolated path must not equal the source checkout and must not be under the source checkout. A sibling under `~/src` is allowed only when it is outside the source checkout.

Overlap in touched files does not stop upstream exploration, contracts, proof planning, or test planning. It does require collision notes and may serialize implementation repair or landing if merge conflicts appear.

## Mandatory Verification Gate

```bash
test -s /home/lewis/.opencode/skill/femdation/SKILL.md
test -s /home/lewis/.agents/skills/femdation/SKILL.md
test -s /home/lewis/.opencode/agent/femdation.md
test -s /home/lewis/.opencode/agent/explore.md
test -s /home/lewis/.opencode/agent/proof-plan-reviewer.md
test -s /home/lewis/.opencode/agent/proof-to-implementation.md
test -s /home/lewis/.opencode/agent/evidence-packaging.md
test -s /home/lewis/.opencode/agent/landing-skill.md
test -s /home/lewis/.opencode/skill/go-skill/SKILL.md
test -s /home/lewis/.opencode/skill/go-skill/state-machine.md
test -s /home/lewis/.opencode/skill/go-skill/checklist.md
test -s /home/lewis/.opencode/skill/go-skill/artifacts.md
test -s /home/lewis/.opencode/skill/explore/SKILL.md
test -s /home/lewis/.opencode/skill/rust-contract/SKILL.md
test -s /home/lewis/.agents/skills/scott-ddd-refactor/SKILL.md
test -s /home/lewis/.opencode/skill/proof-planner/SKILL.md
test -s /home/lewis/.opencode/skill/proof-plan-reviewer/SKILL.md
test -s /home/lewis/.opencode/skill/proof-writer/SKILL.md
test -s /home/lewis/.opencode/skill/proof-reviewer/SKILL.md
test -s /home/lewis/.opencode/skill/proof-to-implementation/SKILL.md
test -s /home/lewis/.agents/skills/test-planner/SKILL.md
test -s /home/lewis/.agents/skills/test-writer/SKILL.md
test -s /home/lewis/.opencode/skill/test-reviewer/SKILL.md
test -s /home/lewis/.opencode/skill/holzman-rust/SKILL.md
test -s /home/lewis/.opencode/skill/formal-verifier/SKILL.md
test -s /home/lewis/.agents/skills/black-hat-reviewer/SKILL.md
test -s /home/lewis/.opencode/skill/evidence-packaging/SKILL.md
test -s /home/lewis/.agents/skills/truth-serum/SKILL.md
test -s /home/lewis/.agents/skills/landing-skill/SKILL.md
bd show <bead-id> --json
pwd -P
test "$(pwd -P)" = "<isolated-workspace-path-from-STATE>"
case "$(pwd -P)" in "<source-checkout-path-from-STATE>"|"<source-checkout-path-from-STATE>"/*) exit 1;; esac
test ! -e "<source-checkout-path-from-STATE>/.beads/<bead-id>"
test -s .beads/<bead-id>/STATE.md
test -s .beads/<bead-id>/baseline-report.md
jq -c . .beads/<bead-id>/delivery-scope.jsonl >/dev/null

# Current-state gate: validate only the bead's current state/transition.
"/home/lewis/.agents/skills/go-skill/tools/go-skill-v9-validate" --workspace "$(pwd -P)" --bead "<bead-id>" --state "<current-state>" --source-checkout "<source-checkout-path-from-STATE>" --skill-root /home/lewis/.agents/skills/go-skill --mirror-root /home/lewis/.opencode/skill/go-skill
```

For scheduler health, every active bead must have exactly one active child, queued dispatch, verification action, serial landing wait, blocker with evidence, or terminal done status.

## Anti-Hallucination Shield

Forbidden:
- Claiming a child ran without a dispatch manifest and returned artifact evidence.
- Claiming a bead is busy without an active child, queue entry, verification action, serial wait, or blocker.
- Advancing from child prose without checking required artifacts on disk.
- Reusing prompt buffers, workdirs, bead IDs, or artifact paths across parallel children.
- Skipping proof, test, review, machine, truth/evidence, landing, or cleanup gates for throughput.
- Letting specialists work in the source checkout.

Required:
- Keep the fleet table current before prose.
- Dispatch all ready independent beads before explanation.
- Verify artifacts after every child result.
- Route failures to the nearest owning state under the 7-attempt cap.
- Serialize only real global critical sections.
