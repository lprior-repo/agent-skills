---
name: proof-to-implementation
description: "Bridge approved proof claims to Rust implementation obligations. Use after proof-reviewer and before bridge review, test planning, or implementation. Maps implementation-bound Verus/Kani/Flux/Loom/proptest/fuzz claims to Rust source refs, independent behavior tests, refinement harness refs, and exact evidence commands. Does not approve its own bridge output."
---

# Proof To Implementation

Proof artifacts do not implement behavior. This skill forces every approved proof claim to name the Rust behavior it constrains.

## Owns

- `proof-to-rust-map.md`
- `rust-refinement-obligations.jsonl`

## Inputs

- Approved `proof-review.md` and `proof-findings.jsonl`.
- Proof obligations, proof evidence, trusted-base ledger, and proof artifacts.
- Contract, domain model, and traceability artifacts.

## Workflow

1. For every behavior-affecting proof claim, identify concrete Rust target symbols, events, state transitions, or types.
2. Write `rust-refinement-obligation/v1` rows linking proof IDs to `source_refs`, `behavior_test_refs`, `refinement_harness_refs`, and exact evidence commands.
3. Require independent behavior tests. Verifier harnesses do not count as behavior tests.
4. Allow `mapping_status: planned` during State 7, but make closure obligations explicit for State 12.
5. Reject implementation claims whose only proof target is a copied model, local harness builder, comment, `cover!`, or `assert(true)`.
7. Reject file-only refs, prose refs, missing harness refs, missing evidence paths, and behavior-affecting waivers.
8. Return bridge mapping evidence for `proof-reviewer`; do not write `proof-to-rust-review.md` or approve your own output.

## Bridge Standard

- `source_refs` must name production code symbols or extracted production helpers, not just files.
- `behavior_test_refs` must be executable unit/integration/BDD/proptest tests that would fail if the production behavior were deleted.
- `refinement_harness_refs` must be separate from behavior tests and must compile/run by State 12.
- If a proof artifact models behavior outside production code, the bridge must mark the copy/reality gap and add an implementation-bound obligation.

## References

- `../go-skill/references/proof-schemas.md`
- `../go-skill/references/proof-pipeline-contract.md`
- `../go-skill/references/evidence-standards.md`
- `../go-skill/references/finding-codes.md`
- `references/bridge-mapping-guide.md`
- `references/bridge-review-rubric.md`

## Output Rules

`proof-to-rust-review.md` is written by `proof-reviewer`, not this skill. Final response must list mapping artifacts, unresolved mapping gaps, and exact reviewer handoff inputs.
