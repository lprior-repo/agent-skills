# Verification Lane Policy

The proof planner must classify every proof seed across the core verifier set. No silent omissions.

## Required Defaults

- Temporal workflow, protocol, queue, retry, claim, lease, lifecycle, distributed state: TLA+ required.
- Rust-local pure/core invariant, arithmetic, indexing, typestate transition: Verus required.
- Natural bounded state or panic/overflow/index risk: Kani required unless lane decision proves non-applicability.
- Illegal state representable as refinement type: Flux required when practical evidence exists; otherwise explain with lane decision.
- Implementation concurrency, cancellation, shutdown, interleaving risk: Loom required.
- Unsafe, FFI, layout, aliasing, provenance: Miri required.
- Parser, codec, hostile input: proptest and cargo-fuzz required unless non-applicable by source evidence.

## Waivers

Waivers are only for non-behavior obligations. Behavior-affecting rows must be proven, blocked, or rejected.

## Non-Applicability

`not_applicable` requires concrete evidence references. "Not needed", "too hard", and "not practical" are invalid without a lane-decision row and reviewer acceptance.
