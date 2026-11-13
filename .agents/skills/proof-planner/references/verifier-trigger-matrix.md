# Verifier Trigger Matrix

Keyed on `risk_tags` from `references/risk-taxonomy.md`. Each row names the primary lane, the rationale, the counterindication (when NOT to use the lane), and the companion lanes that should also appear.

This matrix is the canonical bridge between `rust-contract` proof seeds and the verifier tools. If a seed's `risk_tags` map to a primary lane that is missing from the plan, that is `E_LANE_DECISION_MISSING`. If a lane appears without a corresponding seed tag, that is `E_LANE_DECISION_WEAK`.

## Verus

| Trigger | Required tag(s) | Counterindication |
|---|---|---|
| Rust-local pure/core invariant | `rust_local`, `pure_core` | The invariant crosses an `async`/`.await` boundary; prefer Loom + Flux |
| Arithmetic/overflow/precision | `arithmetic` | The arithmetic is bounded and the verifier is already a primary; Kani may be a companion |
| Typestate transition | `ownership`, `refinement` | A refinement type expresses it more cleanly; Flux becomes primary, Verus companion |
| Deep functional correctness | `pure_core`, `arithmetic` | Pure functions that mutate thread-local state; prefer Loom |
| Field sensitivity | `property`, `field_sensitivity` | Bounded inputs with finite field combinations; Kani may be primary |

**Rationale.** Verus is the deductive proof engine for Rust. Use it for properties that hold for all inputs (no bound), where the invariant is naturally expressed as `requires`/`ensures` on `exec fn`, and where the function body is the unit of proof.

**Companions.** Kani for bounded cross-check; Flux when refinement overlaps; proptest for property pressure; Miri for unsafe bodies.

**Anti-pattern.** Using Verus to model behavior in standalone spec types disconnected from production. The bridge (`proof-to-implementation`) MUST cite `source_refs`; otherwise the obligation is `E_VERUS_DISCONNECTED_SPEC`.

## Kani

| Trigger | Required tag(s) | Counterindication |
|---|---|---|
| Bounded state machine | `bounded_state` | The state space is unbounded; Verus or Flux is the right tool |
| Panic/overflow/index risk | `panic_freedom`, `index_safety` | The bound is large enough that CBMC cannot complete within resource budget |
| Rejection claim | `rejection` | The rejection is structural and provable at construction; Flux is sufficient |
| Parser/codec rejection | `parser`, `codec`, `rejection` | Pure happy path only; cargo-fuzz or proptest is the right tool |
| Counterexample generation | `bounded_state`, `rejection` | When symbolic execution is too slow; proptest can substitute for counterexample pressure |
| Unsafe code harnesses | `unsafe`, `ub` | When the unsafe path is exotic and outside Kani's supported features |

**Rationale.** Kani is bounded model checking over Rust MIR. Use it when the input space is honestly finite, the property is checkable via assertion/contract, and the harness inventory is enumerable.

**Companions.** Verus for the unbounded invariant; proptest for randomized pressure; cargo-fuzz for hostile byte coverage.

**Anti-pattern.** Using `kani::cover!` as the sole evidence for a property. `cover!` is reachability evidence only; safety/equality/ordering/panic-freedom/field-sensitivity obligations need `kani::assert` or a contract postcondition. See `references/implementation-binding.md`.

**Resource.** CBMC memory grows superlinearly. Default broad Kani runs to `-j 1` with a cgroup cap; see `references/resource-governance.md`.

## Flux

| Trigger | Required tag(s) | Counterindication |
|---|---|---|
| Refinement type | `refinement` | The property is temporal or involves allocation lifetimes; Verus or ownership-deep reasoning is the right tool |
| Length/index relationship | `index` | The relationship is dynamic and beyond Flux's decidable fragment; Kani may substitute |
| Ownership-aware post-state | `ownership`, `refinement` | The mutation crosses an `async` boundary; Loom is required |
| API precondition refinement | `refinement`, `panic_freedom` | The precondition is broader than Flux's decidable fragment; Verus may substitute |
| Typestate | `ownership`, `refinement` | The state is dynamic at runtime; Loom + proptest substitute |

**Rationale.** Flux is a nightly refinement-type checker. Use it when the property is a quantifier-free arithmetic predicate that can be expressed as a refinement, an invariant, or a strong-reference post-state.

**Companions.** Verus for deeper functional proof; Kani for executable cross-check; proptest for property pressure.

**Anti-pattern.** Broadening `#[trusted]`, `#[ignore]`, `extern_spec`, or `opaque` to make a failing refinement pass. Each broadening needs a `trusted-base-plan.md` row, a `limitation_kind`, and a paired `Flux negative target` (see `references/evidence-requirements.md`).

## Loom

| Trigger | Required tag(s) | Counterindication |
|---|---|---|
| Threads/atomics/locks | `concurrency`, `atomic`, `lock` | Single-threaded calculator; no concurrency surface |
| Channel/mpsc/broadcast | `concurrency`, `channel` | The channel is purely sequencing (no parallel senders); proptest suffices |
| Cancellation/drop/timeout | `cancellation`, `shutdown` | Cancellation is purely synchronous; Kani or proptest suffices |
| Async runtime internals | `task_ownership`, `shutdown` | A custom runtime with no Tokio equivalent; `proof-to-implementation` bridge is mandatory |
| Schedule-dependent | `interleaving` | The interleaving is provably impossible by construction; proptest + Verus suffice |
| Drain/finalize protocol | `shutdown`, `task_ownership` | Loom is the right tool for interleavings; the temporal aspect is proved by loom + proptest (TLA+ removed) |

**Rationale.** Loom is a deterministic concurrency model checker. Use it when correctness depends on thread interleavings, channel ordering, lock acquisition order, or atomic ordering.

**Companions.** Flux for ownership typestate; proptest for randomized interleaving pressure.

**Anti-pattern.** Toy models that do not use production synchronization indirection. The harness MUST go through `crate::sync::sync::*` (or equivalent) so that `cfg(loom)` swaps in the Loom types; otherwise the model is a lie. See `E_LOOM_TOY_MODEL`.

## Miri

| Trigger | Required tag(s) | Counterindication |
|---|---|---|
| Unsafe block/function | `unsafe` | Strict-mode `forbid(unsafe_code)` and a non-Miri surface; planner MAY mark `not_applicable` only with evidence |
| Raw pointer arithmetic | `raw_pointer` | The pointer is fully encapsulated by safe abstractions and proven by Verus; specialist scoping note required |
| FFI crossing | `ffi` | Native integration tests + sanitizers; Miri cannot model the native side |
| Provenance/aliasing | `provenance`, `aliasing` | Native sanitizer lane is complementary; Miri is the primary |
| Layout assumptions | `layout` | The layout is enforced by `repr(C)` and the consumer is the same compiler; Verus + layout test |
| Strict Provenance refactor | `provenance`, `raw_pointer` | The refactor already eliminates provenance risk; specialist scoping note required |

**Rationale.** Miri is the dynamic MIR interpreter for UB detection. Use it when the unsafe surface is in scope and the test can run under `cargo +nightly miri`.

**Companions.** Verus for the ownership spec; native sanitizer for the integration boundary; proptest for randomized behavior pressure.

**Anti-pattern.** Marking Miri `not_applicable` for a seed tagged `unsafe`, `ub`, `raw_pointer`, `provenance`, or `aliasing` without a specialist scoping note. See `references/tooling-availability-gate.md` and `references/implementation-binding.md`.

## cargo-fuzz

| Trigger | Required tag(s) | Counterindication |
|---|---|---|
| Parser | `parser` | The parser is purely structural and exercised by Kani; fuzz may be redundant |
| Codec | `codec` | Same |
| Hostile/untrusted input | `hostile_input` | Pure-happy-path claim; fuzz is overkill |
| Persisted bytes | `persisted_bytes` | Trusted internal serialization; proptest suffices |
| IPC decode | `ipc_decode` | Native integration test; fuzz may not reach the IPC layer |
| Fuzzable canonicalization | `fuzzable_canonicalization` | Same |

**Rationale.** cargo-fuzz (libFuzzer) is coverage-guided dynamic fuzzing. Use it when the input space is too large for honest Kani bounds and the parser/codec/hostile-input surface accepts arbitrary bytes.

**Companions.** Kani for bounded rejection checks; Verus for parse-spec round-trip; proptest for typed-input property pressure.

**Anti-pattern.** Crash-only fuzzing without an oracle. Parsers and codecs need round-trip, differential, or verifier-vs-executor oracles. See `references/rust-fuzzer` (skill).

**Resource.** Default smoke budgets of 120s with `-rss_limit_mb=2048` and explicit sanitizer flags. See `references/resource-governance.md`.

## proptest

| Trigger | Required tag(s) | Counterindication |
|---|---|---|
| Property pressure | `property` | The property is bounded and Verus/Kani is sufficient; proptest may be redundant |
| Equality (round-trip) | `equality`, `codec` | Bounded round-trip covered by Kani |
| Ordering | `ordering` | A property over a total order; Kani may suffice |
| Field sensitivity | `field_sensitivity` | Same |
| Random typed input | `hostile_input`, `property` | Random typed input where fuzz harnesses are heavier than needed |

**Rationale.** proptest is property-based testing with shrink. Use it when the property is naturally expressed over a strategy-generated input space, when shrinkage is valuable for debugging, and when proptest can be executed by `cargo test`.

**Companions.** Kani for bounded cross-check; Verus for the unbounded invariant; fuzz for hostile bytes.

**Anti-pattern.** Asserting `is_ok()` instead of an exact value; using `proptest!` over only one shape; missing a non-vacuity strategy; missing an anti-invariant.

## Lane Selection Cheat Sheet

```
seed.risk_tags → primary lane + companions

rust_local/pure_core          → verus (primary) + kani (companion)
arithmetic                    → verus (primary) + kani (companion)
bounded_state                 → kani (primary) + verus (companion)
refinement/index              → flux-rs (primary) + verus/kani (companion)
property/equality/ordering    → proptest (primary) + kani/verus (companion)
concurrency/channel/atomic    → loom (primary) + flux-rs (typestate)
cancellation/shutdown         → loom (primary) + proptest (pressure)
temporal_workflow/protocol    → loom (primary, interleavings + temporal surface) + proptest (pressure)
ub/unsafe/ffi/raw_pointer     → miri (primary) + verus (spec) + native sanitizer (companion)
parser/codec/hostile_input    → cargo-fuzz (primary) + kani (rejection) + proptest (pressure)
```

The planner MUST justify any deviation from this table by way of `decision_reason` and `non_applicability_evidence_refs`.
