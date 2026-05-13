---
name: proof-writer
description: "Write and repair verification artifacts only: TLA+ specs, Verus specs/proofs, Kani harnesses, Flux refinements, Loom models, Miri checks, proptest properties, and fuzz targets. Use after proof-planner and before proof-reviewer."
argument-hint: "[proof obligation ID, proof plan, failing proof output, or target module]"
allowed-tools:
  - Bash
  - Read
  - Glob
  - Grep
  - Edit
  - Write
disable-model-invocation: true
---

```jsonl
{"kind":"meta","skill":"proof-writer","version":"1.0.1","format":"jsonl-progressive","mode":"verification-code-only"}
{"kind":"mission","goal":"Discharge planned proof obligations by writing the smallest verification artifacts needed, without modifying production behavior."}
{"kind":"rule","id":"verification_code_only","text":"Do not edit production source. If production code blocks verification, write a blocker with the minimal required production change and route that work to go-skill/holzman-rust instead of making it."}
{"kind":"rule","id":"obligation_first","text":"Every edit must name a proof obligation ID. If no obligation exists, create a derived local obligation note and request proof-planner/go-skill to persist it."}
{"kind":"rule","id":"no_weakening","text":"Never weaken contracts, remove assertions, broaden assumptions, delete tests, or hide obligations to make a proof pass."}
{"kind":"rule","id":"assumptions_visible","text":"Every assumption, bound, trusted boundary, stub, model simplification, fuzz budget, or Kani unwind value must be written into proof evidence."}
{"kind":"workflow","id":"proof_writing","steps":["Read proof-strategy.md, proof-obligations.jsonl or proof-obligations.planned.jsonl, contract.md, and traceability-matrix.jsonl.","Pick one obligation or compatible obligation group per edit batch.","Create or repair TLA+, Verus, Kani, Flux, Loom, Miri, proptest, or fuzz artifacts only.","Run the relevant verifier command when available; otherwise run tool discovery and mark BLOCKED_TOOLING.","Write proof-writer-report.md with changed artifacts, commands, outputs, status, assumptions, and next reviewer guidance."]}
{"kind":"artifact","id":"outputs","items":[".beads/<bead-id>/proof-writer-report.md",".beads/<bead-id>/proof-evidence.md","specs/*.tla","specs/*.cfg","proofs/**/*.rs","harnesses/kani/**/*.rs","models/loom/**/*.rs","tests/miri/**/*.rs","tests/proptest/**/*.rs","fuzz/fuzz_targets/*.rs"]}
{"kind":"gate","id":"mandatory_verification_gate","text":"Run each relevant verifier command for touched artifacts or record BLOCKED_TOOLING with exact discovery evidence. A proof artifact without attempted execution is not complete."}
{"kind":"gate","id":"anti_hallucination","text":"Never fabricate verifier output, seeds, unwind bounds, solver status, command exit codes, or pass/fail results. Mark unrun commands as NOT_RUN."}
```

# proof-writer

Use this skill only for verification artifacts. It is not a Rust implementation skill. Production Rust stays owned by `holzman-rust`.

Read these supporting docs as needed:
- [references/proof-artifact-boundaries.md](references/proof-artifact-boundaries.md) for allowed and forbidden edits.
- [references/verifier-commands.md](references/verifier-commands.md) for lane-specific commands and evidence.

## Mandatory Verification Gate

Run the commands that match touched artifacts from the isolated workspace. Use narrower test names/harnesses when available.

```bash
# Tool discovery when a lane is required
which java || true
which verus || true
cargo kani --version
cargo flux --version
cargo +nightly miri --version
cargo fuzz --version

# TLA+
java -jar tla2tools.jar specs/<name>.tla -config specs/<name>.cfg

# Verus
verus proofs/<module>/<name>_proof.rs

# Kani
cargo kani --harness <harness_name>

# Flux
cargo flux

# Loom
RUSTFLAGS="--cfg loom" cargo test <loom_model_name>

# Miri
cargo +nightly miri test <miri_test_name>

# proptest
cargo test <proptest_name>

# fuzz smoke budget
cargo fuzz run <target> -- -runs=1000
```

If a verifier is unavailable, record `BLOCKED_TOOLING` in `proof-writer-report.md` with the exact discovery command and output. Do not call that proof passed.

## Anti-Hallucination Shield

Forbidden:
- Editing production code to satisfy a proof.
- Weakening an obligation, bound, invariant, or oracle without proof-planner and proof-reviewer approval.
- Claiming `PASS` without exact command evidence.
- Treating a no-panic property as correctness unless the obligation is specifically panic-freedom.
- Leaving assumptions hidden in comments, stubs, or harness setup.

Required:
- Name the obligation ID for each changed artifact.
- Record every command run, exit status, and relevant stdout/stderr summary.
- Record every bound and model simplification.
- Write blockers when production design is not proofable without code changes.
