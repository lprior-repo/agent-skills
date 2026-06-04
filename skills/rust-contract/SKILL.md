---
name: rust-contract
description: "Turn a Rust bead or feature request into a Fowler/Wlaschin domain model and type-level contract: ubiquitous language, value objects, typestates, workflows, railway error taxonomy, functional-core/imperative-shell boundary, hazards, and proof seeds. Use before proof planning, tests, or implementation. Does not write verifier artifacts, tests, implementation, or final proof plans."
---

# Rust Contract

Model the domain before anyone writes proofs, tests, or production Rust. Make illegal states unrepresentable and emit proof seeds, not proof obligations.

## Owns

- Ubiquitous language, entities, value objects, aggregates, commands, events, policies.
- Type contracts: newtypes, smart constructors, typestates, parsers at boundaries, railway errors.
- Workflow model: legal states, transitions, guards, outcomes, terminal states.
- Boundary map: pure core, imperative shell, async shell, storage/network/time/FFI/unsafe/parser boundaries.
- Hazard analysis and proof seeds.

## Does Not Own

- Verus, Kani, Flux, Loom, proptest, fuzz, or proof artifacts.
- Final proof obligations or verifier commands.
- Test plans, test code, production code, or review approval.

## Inputs

- Bead ID or feature request.
- Existing docs, source context, API boundaries, and `delivery-scope.jsonl` when present.
- Domain language and constraints from the user or repository.

## Outputs

- `domain-model.md`
- `type-contracts.md`
- `workflow-model.md`
- `error-taxonomy.md`
- `boundary-map.md`
- `hazard-analysis.md`
- `contract.md`
- `proof-seeds.jsonl`
- `traceability-matrix.jsonl`

## Workflow

1. Extract domain language and reject primitive obsession, boolean behavior flags, stringly IDs, and `Option` lifecycle state.
2. Define value objects and parsers so invalid external input cannot enter the core unchecked.
3. Define workflows as typed state transitions with explicit outcomes and semantic errors.
4. Split pure core from imperative shell, async shell, storage, network, time, FFI, unsafe, and parser boundaries.
5. Write hazards: temporal, Rust-core invariant, bounded state, refinement, concurrency, unsafe/provenance, hostile input, performance, release/API.
6. Classify each proof seed's intended lane profile: Rust-local implementation, temporal workflow/protocol, concurrency, unsafe/provenance, hostile input, or performance/release. This is a hint only; proof-planner owns final lane decisions.
7. Emit `proof-seeds.jsonl` using `proof-seed/v1`; do not choose final commands or claim proof coverage.
8. Stop if the model cannot make illegal states unrepresentable; report the missing domain decision instead of papering over it.

## Proof Seed Intent

- Rust-local implementation seeds should point toward Verus/Kani/Flux/proptest by default.
- Temporal workflow/protocol/replay/recovery/lifecycle seeds should name the Rust events, states, or APIs that will need bridge evidence later.
- Hostile input seeds should mention fuzz/proptest surfaces.
- Concurrency seeds should mention Loom or equivalent schedule exploration.
- Unsafe/provenance seeds should mark the risk explicitly and require specialist review only when that risk is present.

## References

- `references/domain-model-template.md`
- `references/type-contract-checklist.md`
- `references/workflow-hazard-template.md`
- `references/proof-seed-guide.md`
- `../go-skill/references/proof-schemas.md`

## Final Response

List artifacts written, open domain questions, illegal-state risks that remain representable, and proof seeds emitted. Never say proof is complete.
