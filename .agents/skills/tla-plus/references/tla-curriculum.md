# TLA+ Learning And Evaluation Curriculum

## Source Priority

Use sources in this order when uncertain:

1. Official TLA+ tools and TLC documentation.
2. Leslie Lamport TLA+ materials.
3. Official `tlaplus/tlaplus` repository and `tla2tools.jar` CLI behavior.
4. Official `tlaplus/Examples` corpus.
5. Learn TLA+ core chapters for practical modeling patterns.
6. Apalache documentation only when bounded/symbolic checking is selected by obligation.
7. Project-local specs, scripts, and prior model-checking reports.

Local workflow note: use these sources through a CLI-first path. Do not make VS Code part of the required learning or execution workflow for this environment.

## Stages

### Stage 0: CLI Setup

- Install or locate Java 11+.
- Locate `tla2tools.jar` or a `tlc` wrapper.
- Prove the path by running SANY and TLC on a tiny module from the command line.
- Record exact commands; do not rely on editor-integrated output as evidence here.

### Stage 1: State Model Basics

- Write constants, variables, `Init`, actions, `Next`, and `vars`.
- Check type invariant and deadlock.
- Run TLC on tiny finite constants.

### Stage 2: Safety Properties

- Add mutual exclusion, no-loss, ownership, monotonicity, and conservation invariants.
- Force one invariant failure and learn trace reading.
- Record state counts and exact bounds.

### Stage 3: Nondeterminism And Failure

- Add environment actions: timeout, crash, retry, duplicate message, cancellation, partial commit.
- Prove the workflow still preserves safety.
- Map each action to Rust/runtime events.

### Stage 4: Liveness And Fairness

- Convert safety spec to temporal behavior spec.
- Add weak or strong fairness only where justified.
- Check eventual completion, no starvation, or lease recovery on small models.
- Record liveness limitations separately from safety evidence.

### Stage 5: State-Space Control

- Use model values, finite constants, safe state constraints, and safety-only symmetry.
- Add `VIEW` when relevant to reduce state identity noise.
- Use profiling to diagnose state explosion or disabled actions.

### Stage 6: Implementation Refinement

- Connect model variables to Rust state.
- Connect model actions to Rust functions, commands, events, or database transitions.
- Identify what TLA+ does not prove and route those gaps to Verus, Kani, Miri, fuzzing, Loom, Stateright, QA, or tests.

## Evaluation Tasks

A competent TLA+ agent must be able to:

- explain why TLA+ applies or does not apply to a bead;
- identify variables, actions, invariants, temporal properties, and bounds;
- detect missing failure actions;
- reject liveness without fairness;
- reject symmetry with liveness unless explicitly waived;
- run exact TLC command or report blocker; run Apalache only when selected by obligation;
- read a TLC counterexample trace and name the failing action;
- distinguish simulation from proof;
- write evidence that includes command, config, state counts, deadlock status, and temporal status;
- map model actions to runtime events and route implementation proof to other tools.
- reject editor-dependent workflows when command-line TLC evidence is required.

## Current Stack Fit

Use TLA+ first for temporal design. Use Verus first for Rust-local pure/core invariants. Use Lean/Aeneas/Hax only for tiny theorem kernels beyond Verus. Use Kani/Crux for bounded counterexamples and panic/unsafe contracts. Use Miri/cargo-careful/sanitizers for UB-sensitive Rust. Use fuzz/Bolero/proptest for hostile input spaces. Use Loom/Shuttle/Stateright/Lockbud for implementation interleavings and protocol realization. Use mutation/coverage/static/supply-chain gates when scoped by risk.
