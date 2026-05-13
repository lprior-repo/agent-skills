---
name: proof-planner
description: "Plan high-assurance proof obligations for Rust delivery work. Use after requirements/contracts and before proof writing when deciding which TLA+, Verus, Kani, Flux, Loom, Miri, proptest, fuzz, or CI gates matter."
argument-hint: "[bead-id, contract artifact, module, or proof goal]"
allowed-tools:
  - Bash
  - Read
  - Glob
  - Grep
  - Write
disable-model-invocation: true
---

```jsonl
{"kind":"meta","skill":"proof-planner","version":"1.0.1","format":"jsonl-progressive","mode":"artifact-writing-proof-planning"}
{"kind":"mission","goal":"Convert accepted requirements, domain invariants, and risk tags into a concrete proof strategy and machine-readable obligation matrix without writing proof code or production code."}
{"kind":"rule","id":"planner_not_writer","text":"Do not edit production code, tests, proof files, harnesses, models, or specifications. Write only planning artifacts under `.beads/<bead-id>/`."}
{"kind":"rule","id":"risk_triggered_depth","text":"Use the strongest relevant verifier for the actual risk, but do not require expensive proof lanes for docs, formatting, copy, or behavior-free refactors."}
{"kind":"rule","id":"agents_untrusted_tools_decide","text":"Agents propose obligations; executable tools and later reviewers decide acceptance. Never call an unexecuted obligation proven."}
{"kind":"rule","id":"traceability_required","text":"Every proof obligation must map to a requirement, contract clause, invariant, or explicit risk. Unmapped obligations are invalid."}
{"kind":"rule","id":"waivers_are_obligations","text":"Every skipped applicable verifier needs a waiver row with reason, owner, compensating evidence, and follow-up trigger."}
{"kind":"workflow","id":"proof_planning","steps":["Read bead request, delivery-scope.jsonl, contract.md, traceability-matrix.jsonl, and codebase-map.md when present.","Classify risks: temporal, Rust-local invariant, bounded state, refinement/type-state, concurrency, unsafe/UB, untrusted input, dependency/supply-chain when dependency files changed.","Choose verifier lanes: TLA+, Verus, Kani, Flux, Loom, Miri, proptest, fuzz, CI, or explicit waiver.","Write obligation rows with exact artifact targets, commands, expected evidence, owner_state, rerun_from, and retry guidance.","Emit proof-strategy.md, proof-plan-review-input.md, and proof-obligations.jsonl-ready content for proof-writer."]}
{"kind":"artifact","id":"outputs","items":[".beads/<bead-id>/proof-strategy.md",".beads/<bead-id>/proof-plan-review-input.md",".beads/<bead-id>/proof-obligations.planned.jsonl"]}
{"kind":"schema","id":"obligation_row","fields":["id","requirement_id","contract_clause","risk","verifier","artifact","command","expected_evidence","assumptions","required","mode","owner_state","rerun_from","status","waiver"]}
{"kind":"gate","id":"mandatory_verification_gate","text":"Before finalizing, run scoped discovery commands or record why they cannot run. Plans without discovery evidence are rejected."}
{"kind":"gate","id":"anti_hallucination","text":"Never invent files, commands, verifier availability, pass results, or proof coverage. Mark unknowns as UNKNOWN and missing artifacts as MISSING."}
```

# proof-planner

Use this skill after `rust-contract` has created requirements, assumptions, invariants, and traceability, and before `proof-writer` writes any verification code.

This skill answers: what must be proven, by which tool, with which artifact, using which command, and what evidence will count.

This skill writes planning artifacts only. It must not write proof code, tests, production code, harnesses, models, specs, dependencies, or CI config.

Read these supporting docs as needed:
- [references/verifier-trigger-matrix.md](references/verifier-trigger-matrix.md) for risk-triggered verifier selection.
- [references/obligation-schema.md](references/obligation-schema.md) for JSONL fields and status values.

## Mandatory Verification Gate

Run scoped discovery from the isolated workspace before finalizing proof plans. Narrow `<scope-path>` from `delivery-scope.jsonl`; do not scan unrelated monorepo trees unless the bead is workspace-scoped.

```bash
pwd -P
test -s ".beads/<bead-id>/contract.md"
test -s ".beads/<bead-id>/traceability-matrix.jsonl"
test -s ".beads/<bead-id>/delivery-scope.jsonl"
rg -n "unsafe|unwrap\\(|expect\\(|panic!|todo!|unimplemented!|assert!|spawn|tokio|Mutex|RwLock|Atomic|serialize|deserialize|state|transition|lease|queue|retry|cancel" <scope-path>
rg -n "requires|ensures|proof fn|invariant|kani::|loom::|proptest!|fuzz_target|Flux|TLA|Miri|unsafe" <scope-path>
```

If a command cannot run, record `DISCOVERY_BLOCKED` with the exact command, reason, and reduced fallback scope.

## Anti-Hallucination Shield

Forbidden:
- Claiming a verifier is required without a risk trigger.
- Claiming a verifier is not applicable without a reason tied to scope.
- Inventing tool availability, command output, or future pass status.
- Optionalizing safety-critical obligations to preserve throughput.
- Writing proof code, production code, tests, harnesses, models, specs, dependencies, or CI config from this skill.

Required:
- Use stable obligation IDs like `PO-001`.
- Map every obligation to a requirement or contract clause.
- Include exact artifact paths and commands.
- Include assumptions and model bounds explicitly.
- Mark skipped lanes as `not_applicable`, `waived`, or `blocked_tooling`; never omit them silently when risk tags demand them.
