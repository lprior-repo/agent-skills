# Verification Lane Policy

The proof planner must classify every proof seed through an implementation-bound lane profile. No silent omissions.

## Default Rust-Implementation Profile

For Rust behavior, the default required lanes are:

- Verus for Rust-local pure/core invariants, arithmetic, indexing, typestate transitions, and deeper functional proof obligations.
- Kani for bounded state, panic/overflow/index risk, error/rejection claims, and executable implementation checks.
- Flux for illegal states expressible as refinements, length/index relationships, ownership-aware post-states, and API preconditions when practical.
- proptest for behavior/property pressure through executable Rust APIs.

## Conditional Lanes

- Loom is required for implementation concurrency, cancellation, shutdown, task ownership, channel/queue, atomics, locks, or interleaving risk.
- Unsafe, FFI, layout, aliasing, raw-pointer, provenance, invalid-value, or UB-sensitive claims require explicit specialist scoping before they become proof obligations.
- cargo-fuzz is required for parsers, codecs, hostile input, persisted bytes, IPC/storage decoding, and fuzzable canonicalization boundaries.

## Waivers

Waivers are only for non-behavior obligations. Behavior-affecting rows must be proven, blocked, or rejected.

## Non-Applicability

`not_applicable` requires concrete evidence references. "Not needed", "too hard", and "not practical" are invalid without a lane-decision row and reviewer acceptance.

## Proof-Theater Rejections

- Kani `cover!` can show reachability only; it cannot satisfy a property obligation without assertions or verifier-enforced postconditions.
- Verus proofs over standalone model types do not bind to Rust behavior unless the bridge names production source refs and executable evidence.
- Harnesses that copy production logic, hardcode the graph under test, or assert `true` are model/smoke artifacts only.
