# Trust Marker Scan Patterns

Use this reference for proof review, formal verification, and validator scans. Any hit needs `trusted-base-ledger/v1` unless the reviewer documents a false positive.

## Universal

- `assume`, `ASSUME`, `axiom`, `admit`, `sorry`
- `trusted`, `external_body`, `ignore`, `skip`, `stub`
- disabled checks, suppression attributes, model reductions, bounded constants, symmetry reductions

## Rust / Verus

- `assume(`, `assert_by`, `admit`, `external_body`, `#[verifier::external_body]`
- `trusted`, `recommends`, broad `requires` that encode the desired result
- unproved specs disconnected from executable Rust targets

## Kani

- `kani::assume`, `#[kani::stub]`, `#[kani::unwind]`, `#[kani::proof]`
- missing `kani::cover` for non-vacuity where assumptions constrain inputs
- stubs or contracts that remove failing behavior

## Flux

- `#[trusted]`, `#[ignore]`, opaque specs used to hide invalid states
- refinements that repeat the constructor precondition without rejecting invalid cases

## Loom

- toy models that do not use production synchronization indirection
- missing cancellation, drop, timeout, and shutdown paths

## Validator Rule

A trust marker without one matching `trusted-base-ledger.jsonl` row is `E_TRUST_UNLEDGERED_MARKER`. Pending trusted-base disposition at State 12 is `E_TRUST_PENDING_AT_CLOSURE`.
