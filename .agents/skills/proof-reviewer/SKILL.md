---
name: proof-reviewer
description: "Ruthless proof review gate for TLA+, Verus, Kani, Flux, Loom, Miri, proptest, fuzz, and proof evidence. Use after proof-writer and before tests/implementation, or after formal execution when checking proof adequacy."
argument-hint: "[proof artifacts, proof-evidence.md, obligation ID, or verification report]"
allowed-tools:
  - Bash
  - Read
  - Glob
  - Grep
  - Write
disable-model-invocation: true
---

```jsonl
{"kind":"meta","skill":"proof-reviewer","version":"1.0.1","format":"jsonl-progressive","mode":"artifact-writing-adversarial-proof-review"}
{"kind":"mission","goal":"Reject weak, vacuous, fake, unmapped, or under-executed proof artifacts before they contaminate tests, implementation, or landing evidence."}
{"kind":"rule","id":"findings_first","text":"Report findings first, ordered by severity with artifact paths, obligation IDs, and required fixes. Summaries come last."}
{"kind":"rule","id":"no_self_approval","text":"Do not write or repair proofs. Write only review artifacts under `.beads/<bead-id>/`. If proof review fails, route back to proof-writer with a concrete correction guide."}
{"kind":"rule","id":"raw_evidence_or_reject","text":"Subagent summaries are not proof. A PASS claim requires raw command evidence, artifact paths, and obligation mapping."}
{"kind":"rule","id":"vacuity_hunt","text":"Actively search for assume-heavy models, tautological invariants, shallow bounds, no-op harnesses, trusted-boundary expansion, weak oracles, and unmapped obligations."}
{"kind":"workflow","id":"proof_review","steps":["Read proof obligations, proof strategy, proof-writer-report.md, proof-evidence.md, contract.md, traceability matrix, and changed proof artifacts.","Check each obligation has artifact evidence or an explicit waiver/blocker.","Run relevant verifier commands when feasible, or mark UNVERIFIED_TOOLING and reject approval.","Write proof-review.md with STATUS: APPROVED only when all required obligations are non-vacuous and evidenced.","When rejected, write proof-repair-guide.md with exact fixes for proof-writer and rerun targets."]}
{"kind":"artifact","id":"outputs","items":[".beads/<bead-id>/proof-review.md",".beads/<bead-id>/proof-findings.jsonl",".beads/<bead-id>/proof-repair-guide.md"]}
{"kind":"gate","id":"mandatory_verification_gate","text":"Run discovery and applicable verifier commands for reviewed artifacts when feasible. If not feasible, mark claims UNVERIFIED and do not approve."}
{"kind":"gate","id":"anti_hallucination","text":"Never invent line numbers, command output, pass status, solver behavior, seeds, unwind bounds, schedules, or coverage. Missing evidence is a finding."}
```

# proof-reviewer

Use this skill as the proof equivalent of `test-reviewer`: it reviews proof quality, not production behavior. It must be colder than the proof writer.

This skill writes review artifacts only. It must not edit production code, proof code, tests, harnesses, models, specs, dependencies, or CI config.

Read these supporting docs as needed:
- [references/adversarial-proof-checklist.md](references/adversarial-proof-checklist.md) for verifier-specific rejection rules.
- [references/evidence-standards.md](references/evidence-standards.md) for what counts as proof evidence.

## Mandatory Verification Gate

Run relevant discovery and verifier checks from the isolated workspace. Use exact commands from `proof-obligations.jsonl` or `proof-writer-report.md` when present.

```bash
pwd -P
test -s ".beads/<bead-id>/proof-obligations.jsonl" || test -s ".beads/<bead-id>/proof-obligations.planned.jsonl"
test -s ".beads/<bead-id>/proof-writer-report.md"
rg -n "ASSUME|assume|axiom|admit|sorry|trusted|unimplemented|todo|unwind|invariant|PROPERTY|THEOREM|proof fn|requires|ensures|loom::model|fuzz_target|proptest!|kani::" <proof-artifact-paths>
rg -n "PASS|passed|verified|discharged|counterexample|unwind|bound|coverage|seed|runs|exit" ".beads/<bead-id>/proof-evidence.md" ".beads/<bead-id>/proof-writer-report.md"
```

Run lane-specific commands when the required tools and artifacts exist. Tool absence is not approval; it is `UNVERIFIED_TOOLING` unless a valid waiver exists.

## Rejection Rules

Reject if any required obligation is unmapped, unexecuted, or based on hidden assumptions.

Reject TLA+ when temporal behavior exists but the model lacks meaningful safety invariants, deadlock checks, liveness/fairness where needed, or has unconstrained constants that make the result trivial.

Reject Verus when proofs only restate assumptions, trusted boundaries expand without justification, specs are detached from executable functions, or failures are hidden behind recommends/admits/stubs.

Reject Kani when unwind bounds are absent, arbitrary, too shallow, or assumptions encode the expected result.

Reject Flux when refinements are tautological, disconnected from constructors, or fail to exclude illegal states.

Reject Loom when the model does not represent production synchronization, cancellation/drop paths, or meaningful interleavings.

Reject Miri when the exercised path does not touch the unsafe/aliasing/provenance risk or evidence is only a prose claim.

Reject proptest/fuzz when generators are narrow, oracles are shallow, corpus/budget is absent, or the target only checks no panic without a panic-freedom requirement.

## Anti-Hallucination Shield

Forbidden:
- Approving because proof-writer says it passed.
- Treating screenshots or summaries as command evidence.
- Ignoring failed commands or missing logs.
- Letting waivers replace safety-critical proof without owner, expiry, and compensating evidence.

Required:
- `proof-review.md` must include `STATUS: APPROVED` or `STATUS: REJECTED`.
- Rejection must include `proof-repair-guide.md`.
- Findings must include severity, location, obligation ID, problem, required fix, and evidence.
