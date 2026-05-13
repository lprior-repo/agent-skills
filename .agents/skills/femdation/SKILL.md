---
name: femdation
description: "Concurrent multi-bead dispatcher for the current TLA+/Verus-first go-skill lifecycle. Always delegates bead work to sub-agents, keeps a visible state table, and preserves main-thread context."
argument-hint: "[bead ids or selection query]"
disable-model-invocation: true
allowed-tools:
  - Bash
  - Read
  - Task
---

# Femdation Multi-Bead Orchestrator

Use this skill when dispatching multiple beads through the current `go-skill` pipeline concurrently.

Canonical sources:
- `/home/lewis/.agents/skills/go-skill/SKILL.md`
- `/home/lewis/.agents/skills/go-skill/state-machine.md`
- `/home/lewis/.agents/skills/go-skill/checklist.md`
- `/home/lewis/.agents/skills/go-skill/artifacts.md`
- `/home/lewis/.opencode/skill/femdation/SKILL.md`
- `/home/lewis/.opencode/agent/femdation.md`

If files conflict, canonical `go-skill` wins for the bead lifecycle. This skill adds only multi-bead scheduling, context hygiene, and delegation rules.

Do not proceed from memory. The current `go-skill` state machine, checklist, and artifacts are the only authoritative per-bead pipeline.

## Runtime Table

At the start of every response, print a compact table for every active bead before any prose:

| Bead ID | State | Assigned Agent | Retry Class | Status |
|---|---|---|---|---|

The table is mandatory even when all beads are blocked, waiting, or complete. Keep it compact, but never omit active beads.

## Rules

- `femdation` is a parallel control plane, not a replacement for specialists.
- `femdation` must run as exactly one top-level controller per invocation. Do not invoke a `master` agent, load a `master` skill, spawn per-bead `femdation` controllers, or start recursive orchestrators.
- `femdation` is not a fork of `go-skill`: the only authoritative per-bead pipeline is the current `go-skill` state machine, checklist, and artifacts. If this file conflicts with or omits detail from `go-skill`, follow `go-skill`.
- `femdation` is sub-agent-only for bead work: delegate every state transition, implementation, review, repair, QA pass, and formal step to the relevant specialist agent instead of doing the work inline.
- Sub-agents spawned by `femdation` must be direct children of this one controller. Each specialist prompt must forbid nested delegation, nested `Task` calls, nested `opencode run` calls, invoking a `master` agent/skill, and spawning another orchestrator; specialists return artifacts/evidence to `femdation` instead.
- In Claude-compatible environments, delegation means `Task` with the exact specialist `subagent_type` whenever that state needs specialist work.
- Every bead must have its own isolated jj workspace or approved worktree outside the current/main checkout before any specialist handoff. A sibling under `~/src` is fine only when it is not the source checkout and not nested inside it. Pass that isolated path as the required `workdir`, and treat any attempt to write `.beads/<bead-id>/` or code/test changes in the source checkout as a failed State 1.
- Before advancing any bead, verify specialist coverage for the next `go-skill` state: `explore` when available or the documented State 2 best-effort note, `rust-contract`, `contract-verification-reviewer`, `test-planner`, `test-reviewer`, `test-writer`, `holzman-rust`, `hands-on-qa`, `qa-enforcer`, `red-queen`, `black-hat-reviewer`, `formal-verifier`, `architectural-drift`, and `scott-ddd-refactor`.
- Keep main-thread context clean: use the orchestrator context only for bead routing, state-table updates, artifact checks, gate evidence, retry classification, and cross-bead scheduling decisions.
- Each bead follows canonical current Go-skill states.
- Parallelize independent beads only.
- Never skip states.
- Every state transition must name the delegated agent and be backed by an artifact or command result. A sub-agent claim without the expected file is a failed transition.
- Block on local defects, new regressions, required proof failures, and release/critical gates.
- Record old unrelated global debt as `DEFERRED_GLOBAL` follow-up evidence.
- Exception: if a quick global infrastructure issue blocks multiple agents, spawn exactly one small focused repair agent for that global blocker, verify it, update the table, and continue. Do not fix bead-local code inline.
- Never invent sub-agent output, command output, bead status, artifact contents, or state transitions.

## Canonical Pipeline Rule

Do not maintain or execute a local copy of the Go pipeline from this file. For each bead, load the current `go-skill` files and execute the exact whole-number states from `/home/lewis/.agents/skills/go-skill/state-machine.md`, using `/home/lewis/.agents/skills/go-skill/checklist.md` and `/home/lewis/.agents/skills/go-skill/artifacts.md` for gates.

## Specialist Coverage

Before advancing a bead, verify the delegated specialist for the next `go-skill` state is available. Do not replace missing specialist work with main-context work.

| Go State | Required Delegate |
|---|---|
| 2 | `explore` when available; otherwise the documented State 2 best-effort note, while still producing `delivery-scope.jsonl` |
| 3 | `rust-contract` |
| 4 | `contract-verification-reviewer`, `test-planner`, `test-reviewer` |
| 5 | `test-writer` |
| 6 | `holzman-rust` |
| 7 | `hands-on-qa` |
| 9 | `qa-enforcer` |
| 10 | `test-reviewer` |
| 11 | `red-queen`, `black-hat-reviewer` |
| 12 | `formal-verifier` |
| 13 | `architectural-drift`, `scott-ddd-refactor` |
| 14 | `hands-on-qa` |

States 1, 8, and 15 remain orchestrator states, but the main context still only performs routing, artifact checks, command/file evidence capture, failure classification, and cleanup verification.

## Controller Topology

There is one top-level `femdation` controller per run. It may spawn specialist children for Go-skill states, but those children must not spawn grandchildren, start another `femdation`, invoke a `master` agent/skill, or delegate work further. `femdation` owns all scheduling, artifact verification, retries, and state-table updates.

Every child prompt must include: `You are a direct child of the femdation controller. Do not spawn sub-agents, do not invoke a master agent or master skill, do not invoke nested orchestrators, and do not run nested opencode/Task delegation. Write the required artifacts in the isolated workspace and return evidence to femdation.`

## Workspace Parent Rule

`~/src/...` is an acceptable parent for isolated workspaces only when the chosen real path is outside the source checkout. Reject the workspace if `realpath(isolated_workspace)` equals `realpath(source_checkout)` or is under `realpath(source_checkout)/`. A sibling such as `/home/lewis/src/<repo>-<bead>` is allowed; `/home/lewis/src/<repo>/<bead>` is forbidden when `/home/lewis/src/<repo>` is the source checkout.
