---
name: proof-writer
description: "Write and repair verification artifacts only: TLA+ specs, Verus specs/proofs, Kani harnesses, Flux refinements, Loom models, Miri checks, proptest properties, and fuzz targets. Use after approved proof plans and before proof-reviewer. Never edit production Rust."
---

# Proof Writer

Discharge approved proof obligations by writing the smallest verification artifacts needed. Production behavior belongs to `holzman-rust`, not this skill.

## Owns

- TLA+ `.tla` / `.cfg` files.
- Verus proof/spec artifacts.
- Kani harnesses, Flux annotations, Loom models, Miri checks, proptest properties, fuzz targets.
- `proof-writer-report.md`
- `proof-evidence.md`
- `trusted-base-ledger.jsonl`

## Does Not Own

- Production Rust behavior edits.
- Test suite implementation.
- Review approval or final verification closure.

## Workflow

1. Read approved `proof-plan-review.md`, `proof-obligations.planned.jsonl`, lane decisions, contract artifacts, and traceability.
2. Work only on obligations with IDs. If a proof gap lacks an obligation, stop and route back to `proof-planner`.
3. Write or repair only proof/model/harness artifacts.
4. Record every assumption, trusted boundary, stub, bound, model reduction, disabled check, and verifier limitation in `trusted-base-ledger.jsonl`.
5. Run the cheapest syntax/typecheck/smoke command for every touched artifact when tooling exists.
6. Use `PENDING_FORMAL_EXECUTION` only for expensive deep runs after smoke evidence exists.
7. If tooling is unavailable, record `BLOCKED_TOOLING` as a blocker; it cannot satisfy State 5 exit.
8. If production design blocks proof, write a blocker and route to implementation; do not silently edit production code.

## References

- `../go-skill/references/proof-schemas.md`
- `../go-skill/references/evidence-standards.md`
- `../go-skill/references/trust-marker-scan-patterns.md`
- `references/artifact-boundaries.md`
- `references/lane-command-templates.md`
- `references/trusted-base-writing-guide.md`

## Final Response

List obligations touched, artifacts changed, commands run or blocked, trust ledger entries, pending deep executions, and blockers. Never claim final proof success.
