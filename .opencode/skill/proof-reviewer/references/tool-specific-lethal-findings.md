# Tool Specific Lethal Findings

- TLA+: toy model, weak invariant, missing TypeOK, hidden bounds, no deadlock/fairness stance.
- Verus: spec detached from exec Rust, `requires` encodes result, trusted expansion, tautology.
- Kani: assumptions encode result, no cover, arbitrary unwind, hidden stubs.
- Flux: broad trusted/ignore, tautological refinement, no invalid-state rejection.
- Loom: toy sync, missing cancellation/drop, no meaningful interleavings.
- Miri: risky path skipped, weakening flags unreported, seed coverage inadequate.
