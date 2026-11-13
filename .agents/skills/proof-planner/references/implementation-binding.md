# Implementation Binding (Anti-Laundering)

The hardest failure mode in proof planning is **Verification Laundering**: a proof obligation that looks like it constrains production behavior but actually binds to a model, a stub, or an `assume`/`axiom`. The reviewer (`proof-plan-reviewer`), the writer (`proof-writer`), and the formal verifier (`formal-verifier`) all reject laundering attempts. This reference is the planner's checklist for designing obligations that cannot launder.

The mandate is non-negotiable. Every obligation in `proof-obligations.planned.jsonl` that has `behavior_affecting: true` MUST satisfy these rules. `proof-plan-reviewer` rejects the plan if any rule is violated.

## Rule 1 — `target` Names a Production Symbol

`proof-obligation/v1.target` MUST parse as one of:

- `crate::path::to::symbol`
- `crate::path::to::module::symbol`
- `path/to/file.rs::symbol`
- `path/to/file.rs::module::symbol`

`target` MUST NOT be:

- A bare file path with no symbol (e.g., `src/lib.rs`).
- A model module under `verification/` that does not call production code.
- A path-only string with no symbol after the path.

Validator: `E_SOURCE_REF_SHAPE`.

## Rule 2 — Implementation-Bound Function Bodies

When the verifier is Verus, the obligation MUST require that the `exec fn` contains the actual function body or a direct, verifiable path to production code.

Forbidden patterns:

```rust
// BANNED: external_body makes the exec fn a stub
#[verifier::external_body]
fn exec_counter_tick(c: &mut Counter) { /* nothing */ }
```

```rust
// BANNED: assume short-circuits the obligation
fn exec_counter_tick(c: &mut Counter)
    requires c.value > 0  // assume encodes the desired result
{ ... }
```

```rust
// BANNED: axiom makes the property trivial
pub axiom fn counter_tick_never_underflows() -> bool { true }
```

The obligation MUST specify:

- The `exec fn` body mirrors or directly calls the production `Counter::tick`.
- `requires`/`ensures` describe the caller's contract, not the desired result.
- The proof body uses standard Verus idioms (`assert(...)`, lemmas, `use_type_invariant`, reveal/fuel) to discharge the obligations.

Validator: `E_VERUS_DISCONNECTED_SPEC`.

## Rule 3 — Kani Harnesses Bind to Production

The harness MUST:

- Call the production function directly (`crate::foo::bar::baz(...)`) or call an extracted production helper (a function that exists in `src/` and is `pub(crate)` or `pub`).
- Use `kani::any`, `kani::any_where`, or `kani::bounded_any` for symbolic input, not hardcoded structural inputs.
- Include at least one `kani::cover` reachability point when assumptions constrain inputs (non-vacuity evidence).
- Use `kani::assert` or a function contract postcondition to express the property.

Forbidden patterns:

```rust
// BANNED: cover! only — not a property proof
#[kani::proof]
fn frame_decode_rejects_truncation() {
    let bytes: Vec<u8> = kani::any();
    kani::cover!(bytes.len() < 1024);
    let _ = crate::parser::Frame::decode(&bytes);
    kani::cover!(true);
}
```

```rust
// BANNED: assert(true) is theater
#[kani::proof]
fn frame_decode_rejects_truncation() {
    let bytes: Vec<u8> = kani::any();
    let _ = crate::parser::Frame::decode(&bytes);
    assert!(true);  // vacuous
}
```

```rust
// BANNED: hardcoded structural input
#[kani::proof]
fn frame_decode_rejects_truncation() {
    let bytes = vec![0u8; 1024];
    let _ = crate::parser::Frame::decode(&bytes);
}
```

The obligation MUST specify that the harness:

- Asserts the property over symbolic input.
- Includes `kani::cover` only as reachability evidence, never as the sole satisfaction evidence.
- Uses real assertions for the property.

Validator: `E_KANI_COVER_ONLY` or `E_KANI_ASSUMPTION_VACUITY`.

## Rule 4 — Flux Refinements Are Decidable and Trusted-Shallow

The refinement MUST:

- Be inside Flux's SMT-decidable fragment (quantifier-free arithmetic, booleans, conditionals, field/index access, uninterpreted functions).
- Use `#[refined_by]`, `#[invariant]`, `#[field]` to encode the property at the type boundary.
- Use `&strg T` with `ensures *x: ...` only when ownership-aware post-state is required.

Forbidden patterns:

```rust
// BANNED: #[trusted] hides an unprovable obligation
#[flux_rs::trusted]
#[flux_rs::sig(fn (it: Iterator<i32{v: v >= 0}>) -> i32{v: v >= 0})]
fn sum_positive(it: impl Iterator<Item = i32>) -> i32 { it.sum() }
```

```rust
// BANNED: extern_spec hides a real obligation
#[flux_rs::extern_spec]
fn std_iter_sum<I: Iterator<Item = i32>>(it: I) -> i32 { it.sum() }
```

Without an upstream Flux spec for the standard library, the extern_spec must be paired with a `trusted-base-plan.md` row explaining the gap. A bare extern_spec is `E_FLUX_TRUST_ABUSE`.

The obligation MUST specify:

- The refinement is contained in the planner's crate or a verified upstream crate.
- Each `#[trusted]`, `#[ignore]`, `extern_spec`, or `opaque` boundary is paired with a `trusted_base_refs` entry.
- A paired `negative` or `#[should_fail]` target exists for the invalid-state claim.

## Rule 5 — Loom Models Use Production Sync Indirection

The Loom harness MUST:

- Route through the crate's local sync indirection (`crate::sync::sync::*` or equivalent) so `cfg(loom)` swaps in `loom::sync::*`.
- Use the production concurrency primitives, not a toy model.
- Bound threads (`max_threads ≤ MAX_THREADS = 5`), preemptions, branches, and permutations per `references/resource-governance.md`.
- Include cancellation, drop, timeout, and shutdown paths when the seed's risk includes any of those tags.

Forbidden patterns:

```rust
// BANNED: toy model that does not match production sync
#[test]
fn mpsc_drain_loom() {
    loom::model(|| {
        let (tx, rx) = std::sync::mpsc::channel();  // NOT loom-aware
        // ... toy logic ...
    });
}
```

```rust
// BANNED: bare std::sync::Mutex across an await
let guard = std::sync::Mutex::new(0).lock().unwrap();
some_async_op().await;  // guard crosses await
```

The obligation MUST specify that the harness uses the crate's `cfg(loom)`-aware sync module and that cancellation/drop paths are exercised.

Validator: `E_LOOM_TOY_MODEL`.

## Rule 6 — (TLA+ removed)

Temporal workflows are proved by loom (interleavings) + proptest (randomized pressure) plus a paired `rust-refinement-obligation/v1` row. The temporal model is loom's schedule exploration; there is no separate temporal verifier lane.

## Rule 7 — Miri Specialist Scoping

When the seed's `risk_tags` include `ub`, `unsafe`, `ffi`, `raw_pointer`, `aliasing`, `provenance`, `layout`, or `panic_surface`, the obligation MUST include a Miri Specialist Scoping Note in `trusted-base-plan.md` with:

- The exact `unsafe` API surface under test (`target::function`).
- The provenance/aliasing model (`-Zmiri-strict-provenance`, `-Zmiri-tree-borrows`, default Stacked Borrows).
- The `#[cfg_attr(miri, ignore)]` tests and the reason each cannot be modeled.
- The planned `MIRIFLAGS` and seed range.
- The complementary native sanitizer lane (ASan, MSan, TSan, Valgrind) for surfaces Miri cannot model.

Marking the obligation `not_applicable` without this note is `E_MIRI_PATH_SKIPPED`.

## Rule 8 — Fuzz Harnesses Have Oracles

`cargo-fuzz` harnesses MUST:

- Have an explicit oracle (round-trip, differential, metamorphic, optimized-vs-unoptimized, verifier-vs-executor). Crash-only fuzzing is insufficient.
- Bound fuel, recursion, AST nodes, bytecode length, heap growth, output size, spawned threads, and JIT warm-up loops inside the harness.
- Reset persistent state deterministically (no leaked RNG, file descriptors, threads, allocator growth).
- Use a smoke command with explicit `-max_total_time` and `-rss_limit_mb` per `references/resource-governance.md`.

Validator: missing oracle is `E_FUZZ_NO_ORACLE` (fatal for parser/codec/hostile-input claims).

## Rule 9 — proptest Properties Have Strategies and Anti-Invariants

`proptest` properties MUST:

- Use a strategy that exercises non-trivial input space (`prop::collection::vec(any::<T>(), 0..N)`, not just `Just(value)`).
- Include an explicit anti-invariant or property that must NOT hold for invalid inputs (rejection evidence).
- Assert exact values, never `is_ok()` / `is_err()`.
- Run with `PROPTEST_CASES=10000` minimum per claim.

Validator: missing anti-invariant is `E_PROPTEST_NO_ANTI_INVARIANT` (fatal).

## Rule 10 — Cross-Cutting Anti-Patterns

These anti-patterns fail regardless of verifier:

- **Hardcoded inputs in symbolic harnesses** (Kani, Loom, fuzz, proptest) — the harness is a smoke check, not a property proof. Pair with a `verify-smoke` row and require a paired `verify-proof` row.
- **Comments-only proofs** — `// proof: this is safe because ...` is not proof. The obligation is `E_PROOF_COMMENT_ONLY`.
- **Local model builders** that copy production logic — the obligation is `E_MODEL_COPIES_PRODUCTION`. The bridge MUST name the copy/reality gap.
- **Generic commands** — `cargo test`, `cargo build`, `cargo check` without a harness or test name are `E_COMMAND_GENERIC`. The obligation MUST name the harness, the test, the target, and the feature flags.

## Reviewer Triggers

`proof-plan-reviewer` writes a `verifier-lane-review/v1` row with `reviewer_disposition: rejected` whenever the obligation violates any rule above. The disposition includes a `finding_refs` pointing to a `finding/v1` row with `severity: blocker` and a `required_fix` citing the rule violated.

The planner MUST self-audit `proof-obligations.planned.jsonl` against this reference before handoff. Any violation is a self-rejection. See `references/plan-quality-gates.md`.
