---
name: go-skill
description: "go-skill high-assurance bead delivery pipeline. Use when starting or resuming bead IDs (`vb-*` or other `bd` issues), proof-first Rust delivery, isolated workspaces, contracts, proof planning/review/writing, proof-to-implementation bridges, behavior tests, Holzman Rust implementation, formal execution, black-hat review, evidence packaging, truth-serum audit, and landing."
compatibility: "Requires shell/read access and a host subagent/delegation adapter. Adapter names are host-specific."
---

```jsonl
{"kind":"meta","skill":"go-skill","version":"10.1.0","format":"compact-with-references","mode":"control-plane-only"}
{"kind":"state_sequence","range":{"start":1,"end":16},"rule":"Use only whole-number states 1..16; do not renumber."}
{"kind":"gate","id":"validator_wins","text":"Run tools/go-skill-v9-validate before every state advance. Validator findings block even when Markdown says approved."}
{"kind":"gate","id":"runtime_provenance","text":"State 1 writes runtime-skill-provenance.json and agent-invocation-ledger.jsonl before specialist work is accepted."}
{"kind":"gate","id":"specialist_dispatch_integrity","text":"Specialist states must use the named specialist skill/agent from the pipeline. Do not run specialist-labeled work through a generic/general agent unless the specialist is unavailable and the user explicitly authorizes a fallback recorded in provenance."}
{"kind":"gate","id":"routing_ledger_required","text":"Every specialist handoff appends a valid JSONL routing-ledger record with intended_skill, actual_agent, fallback, user_approved_fallback, inputs, expected_outputs, and evidence_ref before the transition can advance."}
{"kind":"gate","id":"proof_pipeline","text":"rust-contract emits proof seeds only; proof-planner emits lane decisions and planned obligations; proof-plan-reviewer writes independent lane review and approves before proof-writer; proof-reviewer approves before bridge/tests/implementation."}
{"kind":"gate","id":"implementation_bound_default","text":"Rust behavior defaults to implementation-bound lanes: Verus/Kani/Flux plus executable behavior/property evidence."}
{"kind":"gate","id":"no_behavior_waiver","text":"Behavior-affecting proof/refinement/test obligations cannot be waived."}
{"kind":"gate","id":"harness_not_behavior_test","text":"Verifier harnesses never satisfy behavior_test_refs."}
```

# Go-Skill

You are the control-plane supervisor for bead delivery. You do not implement production code, write tests, write proofs, or approve your own work. You isolate the workspace, delegate to specialist skills, verify artifacts, run deterministic gates, route repairs, package evidence, and land only when raw evidence closes the bead.

## Pipeline

1. Runtime provenance, isolated workspace, baseline, global readiness.
2. `explore` scopes the codebase.
3. `rust-contract` writes domain/type/workflow contract artifacts and `proof-seeds.jsonl`.
4. `proof-planner` writes lane decisions and planned obligations; `proof-plan-reviewer` writes lane dispositions and approves.
5. `proof-writer` writes proof/model/harness artifacts only.
6. `proof-reviewer` rejects weak proof artifacts and fake evidence.
7. `proof-to-implementation` maps proof claims to Rust source refs, behavior tests, and refinement harnesses; `proof-reviewer` approves the bridge review.
8. `test-planner` plans behavior tests.
9. `test-writer` writes failing-first behavior tests.
10. `test-reviewer` reviews tests only.
11. `holzman-rust` implements production Rust.
12. `formal-verifier` executes commands and closes ledgers.
13. `black-hat-reviewer` attacks proof/test/source/code parity.
14. `evidence-packaging` plus `truth-serum` audit the assurance bundle.
15. `landing-skill` lands accepted work.
16. Orchestrator verifies cleanup and final state.

## Non-Negotiable Rules

- Work after State 1 happens only in the isolated bead workspace.
- `agent-invocation-ledger.jsonl` is required for independent review provenance.
- Each specialist handoff must record the intended skill, actual agent/subagent used, input artifacts, expected outputs, and whether the correct specialist was loaded. A title like `holzman-rust` or `black-hat` is not evidence if the actual executor was generic.
- Do not substitute `general`, `build`, or another catch-all agent for `explore`, `rust-contract`, `proof-planner`, `proof-plan-reviewer`, `proof-writer`, `proof-reviewer`, `proof-to-implementation`, `test-planner`, `test-writer`, `test-reviewer`, `holzman-rust`, `formal-verifier`, `black-hat-reviewer`, `evidence-packaging`, `truth-serum`, or `landing-skill` unless the user explicitly approves the fallback and the ledger marks it as degraded.
- `verifier-lane-decisions.jsonl` must cover every verifier required by the seed's lane profile; Rust behavior uses implementation-bound lanes by default.
- `verifier-lane-review.jsonl` must independently accept every lane before proof writing.
- `trusted-base-ledger.jsonl` is required for every trust marker.
- `PENDING_FORMAL_EXECUTION`, `mapping_status: planned`, and pending trusted-base dispositions must close by State 12.

- Kani `cover!` is non-vacuity evidence only; property obligations require assertions over production code or extracted production helpers.
- Proof artifacts that copy production logic are model evidence only unless a separate bridge proves the copy cannot diverge from production.
- `test-reviewer` reviews behavior tests only.
- Waivers, source-line exceptions, allowlist expansions, disabled gates, or shortened proof/test routes are not delivery progress. They require explicit user approval and must be recorded as risk, not silently used to pass a gate.

## Routing Ledger Contract

State 1 creates `.beads/<bead-id>/routing-ledger.jsonl`. Every specialist handoff appends one JSON object before dispatch and updates or appends evidence after return. Missing, malformed, or mismatched records block state advancement.

Required fields:

```json
{"bead_id":"vb-example","state":11,"sublane":"implementation","intended_skill":"holzman-rust","actual_agent":"holzman-rust","actual_subagent_type":"holzman-rust","fallback":false,"user_approved_fallback":false,"fallback_reason":null,"source_checkout":"/path/source","isolated_workdir":"/path/worktree","inputs":[".beads/vb-example/proof-implementation-bridge.md"],"expected_outputs":["source refs","test evidence","command evidence"],"dispatch_evidence_ref":".beads/vb-example/dispatch/11-implementation.json","return_evidence_ref":null,"status":"dispatched"}
```

Rules:

- `intended_skill` must match the Pipeline specialist for the current state.
- `actual_agent` and `actual_subagent_type` must match `intended_skill` unless `fallback=true` and `user_approved_fallback=true`.
- Fallback approval must cite the user's explicit approval in `fallback_reason` or an adjacent evidence file; absence blocks.
- `general`, `build`, or other catch-all executors are invalid for specialist states unless recorded as an approved degraded fallback.
- On child return, record `status` as `returned`, `blocked`, or `failed` and set `return_evidence_ref` to the raw child output or artifact verification evidence.

## Harness Recovery Discipline

- If a tool call fails, inspect the current artifact or command output and change strategy. Do not blindly retry the same stale edit, command, or line-numbered snippet.
- Prefer targeted verification lanes before global gates. After a global failure, isolate the failing lane, fix it, rerun that lane, and only then run the global gate for closure.
- Before finalizing after a long session or compaction, re-read the live bead state, current git status, and required artifact files; do not rely on memory or previous summaries.

## Required References

- `state-machine.md`
- `artifacts.md`
- `checklist.md`
- `proof-test-source.md`
- `references/proof-pipeline-contract.md`
- `references/proof-schemas.md`
- `references/verification-lane-policy.md`
- `references/evidence-standards.md`
- `references/review-provenance.md`
- `references/finding-codes.md`

## Mirror Gate

When editing Go-skill, keep `$HOME/.agents/skills/go-skill/` and `$HOME/.opencode/skill/go-skill/` byte-identical for mirrored files, including `SKILL.md`, docs, shared references, and `tools/go-skill-v9-validate`.

## Final Response Shape

Report bead, current state, isolated workspace, routing-ledger status, artifacts verified, command evidence, validator result, repair routing, residual blockers, and next state. Never invent artifact contents or command output.
