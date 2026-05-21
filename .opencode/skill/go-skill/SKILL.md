---
name: go-skill
description: "go-skill high-assurance bead delivery pipeline. Use when starting or resuming a bead through isolated workspace setup, domain contract modeling, proof planning/review/writing, proof-to-implementation bridging, behavior tests, Rust implementation, formal execution, black-hat review, truth-serum evidence, and landing."
compatibility: "Requires shell/read access and a host subagent/delegation adapter. Adapter names are host-specific."
---

```jsonl
{"kind":"meta","skill":"go-skill","version":"10.0.0","format":"compact-with-references","mode":"control-plane-only"}
{"kind":"state_sequence","range":{"start":1,"end":16},"rule":"Use only whole-number states 1..16; do not renumber."}
{"kind":"gate","id":"validator_wins","text":"Run tools/go-skill-v9-validate before every state advance. Validator findings block even when Markdown says approved."}
{"kind":"gate","id":"runtime_provenance","text":"State 1 writes runtime-skill-provenance.json and agent-invocation-ledger.jsonl before specialist work is accepted."}
{"kind":"gate","id":"proof_pipeline","text":"rust-contract emits proof seeds only; proof-planner emits lane decisions and planned obligations; proof-plan-reviewer writes independent lane review and approves before proof-writer; proof-reviewer approves before bridge/tests/implementation."}
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
- `verifier-lane-decisions.jsonl` must cover every `(requirement_id, contract_clause, proof_seed_id, verifier)` tuple in the core verifier set.
- `verifier-lane-review.jsonl` must independently accept every lane before proof writing.
- `trusted-base-ledger.jsonl` is required for every trust marker.
- `PENDING_FORMAL_EXECUTION`, `mapping_status: planned`, and pending trusted-base dispositions must close by State 12.
- TLA+ is temporal evidence, not Rust implementation evidence.
- `contract-verification-reviewer` is historical; do not use it as a live gate.
- `test-reviewer` reviews behavior tests only.

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

Report bead, current state, isolated workspace, artifacts verified, command evidence, validator result, repair routing, residual blockers, and next state. Never invent artifact contents or command output.
