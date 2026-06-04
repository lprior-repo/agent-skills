# Proof Artifact Boundaries

Allowed edits:
- `proofs/**/*.rs`
- `harnesses/kani/**/*.rs`
- `models/loom/**/*.rs`
- `tests/proptest/**/*.rs`
- `fuzz/fuzz_targets/**/*.rs`
- proof evidence files under `.beads/<bead-id>/`

Forbidden by default:
- production source behavior
- public API changes
- test rewrites meant to hide a defect
- dependency changes
- CI gate weakening
- deleting obligations or waivers

If production code is not proofable, write a blocker that names the smallest required production change and route it to `holzman-rust` through `go-skill`.
