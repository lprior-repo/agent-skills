# Risk Taxonomy

Maps every `proof-seed/v1.risk_tags` value to the mandatory lane profile. The taxonomy is the single source of truth for which verifiers are required, which are forbidden, and which need specialist scoping. Reuse these tags verbatim; do not invent new risk classes.

`rust-contract` emits `risk_tags` as a hint. `proof-planner` resolves them into lane decisions. The taxonomy below is exhaustive for the current Go-skill pipeline; new tags require a Go-skill schema bump.

## Universal Profile Tags

Every Rust behavior proof seed inherits these unless an override applies.

| Tag | Verifier | Required? | Notes |
|---|---|---|---|
| `rust_local` | `verus` | yes | Rust-local pure/core invariants |
| `pure_core` | `verus` | yes | calculator-layer functions |
| `arithmetic` | `verus` | yes | overflow/underflow, conversion, precision |
| `bounded_state` | `kani` | yes | bounded model with finite input space |
| `rejection` | `kani` | yes | negative tests for invalid input rejection |
| `panic_freedom` | `kani` | yes | panic-freedom obligations |
| `refinement` | `flux-rs` | yes (if applicable) | refinement type expressions |
| `index` | `flux-rs` | yes (if applicable) | length/index relationships |
| `ownership` | `flux-rs` | yes (if applicable) | ownership-aware post-states |
| `property` | `proptest` | yes | property-based pressure |

A seed with any of `rust_local`, `pure_core`, `arithmetic`, `bounded_state`, `rejection`, `panic_freedom`, `refinement`, `index`, `ownership`, `property` MUST emit the corresponding lane as `applicability: required` unless the risk is provably absent.

## Conditional Profile Tags

These are added on top of the universal profile when present.

| Tag | Verifier | Required? | Notes |
|---|---|---|---|
| `temporal_workflow` | (TLA+ removed; loom covers) | no | claim/lease/lifecycle/distributed workflows — loom only |
| `protocol` | (TLA+ removed) | no | message-passing protocols — loom only |
| `scheduler` | (TLA+ removed) | no | schedulers, retry/backoff — loom + proptest |
| `lease` | (TLA+ removed) | no | lease/claim/renew — loom only |
| `liveness` | (TLA+ removed) | no | eventual/always/eventually properties — loom + proptest |
| `safety` | (TLA+ removed) | no | invariant-style state predicates — loom + verus |
| `distributed` | (TLA+ removed) | no | multi-node coordination — loom + proptest |
| `concurrency` | `loom` | yes | threads, atomics, channels, locks |
| `interleaving` | `loom` | yes | schedule-dependent behavior |
| `cancellation` | `loom` | yes | cancellation/drop/timeout safety |
| `shutdown` | `loom` | yes | drain/finalize protocols |
| `atomic` | `loom` | yes | atomic ordering, fences |
| `channel` | `loom` | yes | mpsc/broadcast/oneshot |
| `lock` | `loom` | yes | Mutex/RwLock ordering, deadlock freedom |
| `task_ownership` | `loom` | yes | JoinSet/TaskTracker/spawn discipline |
| `ub` | `miri` | yes (specialist scope) | undefined behavior risk |
| `unsafe` | `miri` | yes (specialist scope) | explicit unsafe surface |
| `ffi` | `miri` | yes (specialist scope) | FFI crossing |
| `raw_pointer` | `miri` | yes (specialist scope) | raw pointer arithmetic |
| `aliasing` | `miri` | yes (specialist scope) | aliasing/stacked/tree borrows |
| `provenance` | `miri` | yes (specialist scope) | Strict Provenance violations |
| `layout` | `miri` | yes (specialist scope) | `repr(C)`/packed/layout assumptions |
| `panic_surface` | `miri` | yes | panic-freedom in unsafe region |
| `parser` | `cargo-fuzz` | yes | any byte/text input boundary |
| `codec` | `cargo-fuzz` | yes | serialization/deserialization |
| `hostile_input` | `cargo-fuzz` | yes | adversarial input surface |
| `persisted_bytes` | `cargo-fuzz` | yes | disk/network/IPC payloads |
| `ipc_decode` | `cargo-fuzz` | yes | IPC/storage decoding |
| `fuzzable_canonicalization` | `cargo-fuzz` | yes | canonicalization that affects security |

## Risk Tag → Risk Class

The `proof-obligation/v1.risk` field is the canonical risk class; `risk_tags` are the conditions. The validator checks that at least one tag maps to the named risk class.

| `risk` | Required tag(s) |
|---|---|
| `bounded_transition` | `bounded_state` |
| `arithmetic_overflow` | `arithmetic` |
| `index_safety` | `index` |
| `panic_freedom` | `panic_freedom` |
| `illegal_state` | `refinement` or `ownership` |
| `refinement` | `refinement` |
| `concurrency_interleaving` | `concurrency` or `interleaving` |
| `cancellation_safety` | `cancellation` |
| `shutdown_drain` | `shutdown` |
| `temporal_liveness` | `liveness` (loom + proptest; no TLA+) |
| `temporal_safety` | `safety` (loom + verus; no TLA+) |
| `ub_safety` | `ub` or `unsafe` |
| `hostile_input` | `hostile_input` |
| `rejection` | `rejection` |
| `equality` | `property` |
| `ordering` | `property` |
| `field_sensitivity` | `property` |
| `parse_canonicalization` | `parser` or `codec` |

## Specialist-Scoping Tags (Miri Family)

Tags in the Miri family (`ub`, `unsafe`, `ffi`, `raw_pointer`, `aliasing`, `provenance`, `layout`, `panic_surface`) require the planner to write a `Miri Specialist Scoping Note` in `trusted-base-plan.md`. The note MUST include:

- The exact `unsafe` API surface (path::symbol) under test.
- The provenance/aliasing model in scope (`-Zmiri-strict-provenance`, `-Zmiri-tree-borrows`, or default Stacked Borrows).
- The set of `#[cfg_attr(miri, ignore)]` tests and why they cannot be modeled.
- The planned Miri command with `MIRIFLAGS` and seed range.
- The complementary native sanitizer lane (ASan, MSan, TSan, Valgrind) when Miri cannot model the surface.

The note ID appears in `trusted_base_refs` for the Miri obligation. The note is mandatory; absence is `E_MIRI_PATH_SKIPPED`.

## Forbidden Tag Substitutions

These are common mistakes. The validator rejects them with `E_RISK_TAG_UNRECOGNIZED`.

- `unsafe_code` → use `unsafe`.
- `concurrency_safe` → use `concurrency` plus `cancellation_safety`.
- `tla` or `tla+` → **removed.** TLA+ is not a supported verifier in this skill; use `loom` for the temporal surface instead.
- `fuzzing` → use `hostile_input` and the appropriate surface tag (`parser`, `codec`, etc.).
- `unit_test` or `integration_test` → test plans belong to `test-planner`, not proof planning.
- `property_test` → use `proptest` verifier with `property` tag.

## Risk Class → Mandatory Lane Set (Cheat Sheet)

```
RUST BEHAVIOR (default)
  +-- pure_core            ⇒ verus + kani + flux-rs (if refinement) + proptest
  +-- bounded_state        ⇒ verus + kani + proptest
  +-- arithmetic           ⇒ verus + kani + proptest
  +-- illegal_state        ⇒ verus + flux-rs + kani
  +-- property             ⇒ proptest (+ companion verus/kani for invariants)

TEMPORAL WORKFLOW (TLA+ removed; loom covers)
  +-- lease/claim          ⇒ loom (channel) + proptest (race)
  +-- retry/backoff        ⇒ loom + proptest
  +-- distributed          ⇒ loom + flux-rs (region types) + proptest

CONCURRENCY
  +-- channel/queue        ⇒ loom (temporal) + kani (bounded drain) + proptest
  +-- atomics              ⇒ loom + verus (ordering spec)
  +-- cancellation         ⇒ loom + flux-rs (typestate) + proptest
  +-- shutdown             ⇒ loom (drain protocol) + proptest

UB / UNSAFE
  +-- raw_pointer          ⇒ miri + verus (ownership spec) + proptest
  +-- ffi                  ⇒ miri + native sanitizer (out-of-Miri) + proptest
  +-- layout               ⇒ miri + verus + proptest
  +-- panic_freedom        ⇒ miri + verus + kani + proptest

HOSTILE INPUT
  +-- parser               ⇒ cargo-fuzz + kani (rejection) + proptest + verus (parse spec)
  +-- codec                ⇒ cargo-fuzz + kani + proptest + verus (round-trip spec)
  +-- persisted_bytes      ⇒ cargo-fuzz + kani + proptest

REFINEMENT
  +-- refinement only      ⇒ flux-rs (+ companion verus/kani for runtime check)
```

A lane may be added but never silently dropped from this set unless the seed's `risk_tags` provably do not include the relevant tag and the decision cites concrete evidence (`references/lane-decision-guide.md`).

## Cross-Reference With Verifier Triggers

The trigger matrix in `references/verifier-trigger-matrix.md` is keyed on the same `risk_tags`. If a tag appears here but its lane is missing from the trigger matrix, that is `E_TAG_LANE_GAP` and is a planner self-rejection.
