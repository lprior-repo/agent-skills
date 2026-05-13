# Miri Curriculum And Review Checklist

Use this when teaching Miri, reviewing Miri evidence, or deciding whether another tool is more appropriate.

## Learning Roadmap

Days 1-2: Tool basics.

- Install nightly Miri or inspect an existing pinned toolchain.
- Run `cargo +nightly miri test` on a small crate.
- Add one use-after-free test and one invalid-`bool` test.
- Milestone: explain why a clean run is evidence, not proof.

Days 3-5: Validity and abstract bytes.

- Write `MaybeUninit` experiments for `u8`, `f32`, a struct, a slice, and an enum.
- Observe which values are invalid immediately and which byte copies preserve uninit state.
- Milestone: explain uninitialized state without equating it to an arbitrary byte value.

Week 2: Provenance and raw pointers.

- Refactor one tagged-pointer or offset-heavy example from integer casts to `addr_of!`, `addr_of_mut!`, `with_addr`, or `map_addr`.
- Compare default Miri behavior with temporary `-Zmiri-permissive-provenance` only for triage.
- Milestone: distinguish Strict Provenance from Exposed Provenance.

Week 3: Aliasing models.

- Build tiny examples involving shared references, raw writes, reborrows, and `UnsafeCell`.
- Compare default Stacked Borrows with `-Zmiri-tree-borrows`.
- Milestone: explain why a raw pointer can syntactically outlive a reference but still be semantically invalid.

Week 4: Concurrency and seeds.

- Write one racy `static mut` example and one broken relaxed-atomic example.
- Run `-Zmiri-many-seeds=0..16` and `-Zmiri-track-weak-memory-loads`.
- Try Loom or Shuttle for a small state-space example.
- Milestone: distinguish one observed bad execution from exhaustive schedule exploration.

Week 5: Production workflow.

- Add Miri to CI.
- Add a big-endian target job if relevant.
- Use `#[cfg_attr(miri, ignore)]` for unsupported integration tests.
- Compare one bug against ASan, TSan, MSan, UBSan, or Valgrind where possible.
- Milestone: explain what Miri covers and what the native tools cover.

Ongoing: Contributor-grade internals.

- Read the rustc interpreter chapter and Miri contribution guide.
- Run or build Miri itself.
- Use `MIRI_LOG`, `MIRI_BACKTRACE=1`, `MIRI_TRACING=1`, and `MIRI_LIB_SRC` only for interpreter or standard-library debugging.
- Milestone: distinguish language-model questions from Miri implementation bugs.

## Source Priority

Prefer sources in this order:

1. Local command output from the same machine and toolchain.
2. Miri README and help output.
3. Rust Reference UB and memory model pages.
4. Unsafe Code Guidelines glossary.
5. Standard-library pointer provenance docs and RFC 3559.
6. Rustc dev guide interpreter docs.
7. Stacked Borrows and Tree Borrows papers.
8. Miri contribution guide.
9. Sanitizer, Valgrind, nextest, rust-analyzer, and rustup docs for integration details.

If sources conflict, record the conflict and scope the claim to observed local behavior.

## Tool Selection

Use Miri when:

- The question involves unsafe Rust UB in executed tests.
- The failure may involve validity, provenance, aliasing, initialization, alignment, data races, leaks, or intrinsic preconditions.
- You need cheap cross-target interpretation such as big-endian checks.
- You need runtime evidence for a minimized unsafe-code path.

Use sanitizers or Valgrind when:

- The interesting behavior is in native code, linked libraries, FFI, OS APIs, networking, or optimized machine code.
- You need to observe production-like runtime behavior.

Use Loom, Shuttle, or Stateright when:

- The main risk is schedule explosion, interleaving coverage, or lock-free algorithm behavior.
- One or a few Miri seeds are not enough.

Use Kani when:

- A bounded symbolic harness can cover all inputs in the modeled domain.
- The claim is about assertions, panics, arithmetic, indexing, or state transitions under explicit bounds.

Use Flux when:

- The property is a lightweight refinement type fact such as range, length, legal state, or panic precondition.

Use Verus when:

- The property needs deductive proof, loop invariants, algebraic reasoning, or unbounded Rust-local correctness.

Use TLA+ when:

- The property is temporal, distributed, concurrent protocol-level, or about liveness/fairness independent of Rust implementation details.

## Review Checklist

Reject or downgrade Miri evidence if any are true:

- The command, exit status, nightly, target, package, features, test filter, or MIRIFLAGS are missing.
- The report says "Miri proves soundness" or otherwise overclaims.
- Tests were skipped under Miri but not reported.
- `cfg(miri)` changes core logic and the claim does not mention the alternate path.
- The failure is an unsupported operation but is reported as definite UB without evidence.
- The code uses FFI, networking, OS APIs, or native libraries without complementary native-tool coverage.
- Raw pointer code uses integer round trips and ignores Strict Provenance guidance.
- `-Zmiri-permissive-provenance`, native-library bypasses, disabled checks, or other weakening flags are used as final evidence without waiver.
- Concurrency claims rely on one seed or deterministic concurrency only.
- A Stacked-vs-Tree Borrows difference is presented as final Rust semantics rather than model evidence.

## Evidence Wording

Strong wording:

```text
Miri found no UB for `cargo +nightly miri test -p arena --test raw_slots slot_reuse` on nightly-2026-05-01, target x86_64-unknown-linux-gnu, MIRIFLAGS="-Zmiri-many-seeds=0..16 -Zmiri-backtrace=full". The Miri path skipped `network_roundtrip`, so host I/O remains covered only by native integration tests.
```

Weak wording:

```text
Miri proves the arena allocator is sound.
```

Failure wording:

```text
Miri reports a Stacked Borrows violation in `shared_reference_then_raw_write`. The diagnostic names pointer tag `<tag>` invalidated after a shared borrow. I re-ran with `-Zmiri-track-pointer-tag=<tag>` and the fix should remove the shared reference before the raw write or use `UnsafeCell` if interior mutability is intended.
```

Unsupported wording:

```text
Miri cannot model this foreign call. This is not accepted as UB evidence. The test is skipped under `cfg_attr(miri, ignore)`, and the FFI boundary requires ASan/Valgrind/native integration coverage.
```
