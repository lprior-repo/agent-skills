# Tool Specific Lethal Findings

- Verus: spec detached from exec Rust, `requires` encodes result, trusted expansion, tautology.
- Kani: assumptions encode result, no cover, arbitrary unwind, hidden stubs.
- Flux: broad trusted/ignore, tautological refinement, no invalid-state rejection.
- Loom: toy sync, missing cancellation/drop, no meaningful interleavings.
