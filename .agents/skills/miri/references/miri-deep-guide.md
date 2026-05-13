# Miri Deep Guide

This is the dense reference for agents working with Miri. Verify exact flags and diagnostics against the local nightly before treating examples as runnable evidence.

## What Miri Is

Miri is Rust's nightly-only Undefined Behavior detector. It executes Rust MIR with an interpreter instead of running compiled machine code. This lets it diagnose Rust-specific failures that native execution often hides:

- Out-of-bounds accesses.
- Use-after-free and dangling pointer accesses.
- Uninitialized reads and invalid values such as a non-0/1 `bool`.
- Alignment violations.
- Intrinsic precondition failures such as overlapping `copy_nonoverlapping`.
- Memory leaks.
- Some data races and weak-memory behaviors.
- Experimental aliasing and provenance violations through Stacked Borrows or Tree Borrows.

The mental model is evidence, not proof. A passing Miri run means Miri observed no issue for this crate/test/input/features/target/MIRIFLAGS/seed set. A failure is usually high-value and often decisive, but must still be classified as definite UB, unsupported operation, platform gap, interpreter bug, or model-sensitive aliasing/provenance issue.

## Architecture

`cargo miri` builds a custom sysroot, asks `rustc` to lower code to MIR, then interprets MIR with a machine state that tracks more than raw bytes.

Core flow:

```text
Rust crate
  -> cargo miri
  -> custom sysroot
  -> rustc lowers crate to MIR
  -> shared MIR interpreter family
  -> Miri machine state
  -> diagnostics or unsupported-operation reports
```

The interpreter family is closely related to rustc constant evaluation and CTFE. Miri is therefore Rust-shaped: it sees typed values, initialization state, allocation lifetimes, pointer provenance, borrow tags, and some weak-memory behavior. It is not a binary instrumentation layer and does not see arbitrary native-library behavior as a native tool would.

## Relation To Rust Semantics

The Rust Reference says unsafe code is still subject to UB rules, the UB list is not exhaustive, and the memory model is incomplete. Miri is an executable approximation of the evolving Rust abstract machine, not a finalized normative semantics.

Rust memory is not just bytes. The Unsafe Code Guidelines model an abstract byte as either uninitialized or initialized with optional provenance. This explains why Miri can preserve uninitialized state through copies and why pointer-to-integer-to-pointer round trips are semantically fragile.

Use precise language:

```text
Miri found no UB in `cargo +nightly miri test -p core --test ptr_model` on nightly-2026-05-01, x86_64-unknown-linux-gnu, with MIRIFLAGS="-Zmiri-many-seeds=0..16".
```

Avoid broad language:

```text
Miri proved the unsafe abstraction is sound.
```

## Aliasing Models

Miri defaults to experimental Stacked Borrows checking. Stacked Borrows acts like a dynamic counterpart to Rust's borrow checker for references, with special handling for raw pointers. It can reject code that native execution appears to tolerate because the reference or raw-pointer access violates the aliasing story Rust optimizations rely on.

Tree Borrows is an experimental alternative available through `-Zmiri-tree-borrows`. It organizes derived references as a tree rather than a stack and is usually more permissive on some real-world unsafe code. Treat Stacked-vs-Tree differences as evidence for current experimental models, not a final language ruling.

For an aliasing diagnostic, record:

- Whether the run used default Stacked Borrows or `-Zmiri-tree-borrows`.
- Pointer tags or allocation IDs named in the diagnostic.
- The reference creation point and the later raw-pointer/reference access that invalidated it.
- Whether `UnsafeCell` legitimately permits mutation behind a shared reference.

## Provenance And Strict Provenance

Modern unsafe Rust should preserve provenance explicitly. Pointers semantically contain more than addresses; integers do not preserve that information. Prefer the Strict Provenance APIs when manipulating pointer addresses:

- `addr_of!` and `addr_of_mut!` create raw pointers without an intermediate reference.
- `with_addr` and `map_addr` change addresses while preserving provenance.
- `AtomicPtr<T>` is usually better than storing pointers in `AtomicUsize`.
- Exposed Provenance APIs are explicit bypass APIs for MMIO, legacy APIs, and unavoidable integer-address boundaries.

Treat `-Zmiri-permissive-provenance` as temporary triage. It can suppress warnings while shrinking old pointer code, but it also misses the class of bugs you are supposed to find.

## UB Categories Miri Handles Well

Memory and validity:

- Dangling or freed allocations.
- Reading uninitialized memory.
- Producing invalid typed values such as bad `bool`, invalid enum discriminants, invalid references, or invalid function pointers.
- Misaligned pointer dereference.
- Violating intrinsic contracts.

Aliasing and provenance:

- Mutating through a raw pointer while a live shared reference promises immutability.
- Reusing references or raw pointers after a conflicting borrow invalidated their tag.
- Pointer-integer-pointer patterns that lose provenance under the model.

Concurrency and weak memory:

- Data races on non-atomic memory.
- Some ordering-sensitive bugs through weak-memory emulation.
- Seed-sensitive schedules and outdated atomic loads when explored.

Portability:

- Big-endian assumptions via `--target s390x-unknown-linux-gnu`.
- Target-layout and alignment assumptions that are invisible on the host.

## Unsupported Or Model-Limited Areas

Unsupported operations are not automatically code bugs. Miri has limited support for networking, many platform APIs, most arbitrary FFI, inline assembly, and native libraries. The experimental Unix-only native-library bypass can be useful for triage but is unsound for several Miri checks because native code can mutate state Miri cannot observe.

Miri also explores only the executions you run. Vary seeds for concurrency, address reuse, and weak-memory behavior. Use fuzzing or property tests for input breadth. Use Loom, Shuttle, or Stateright for richer schedule exploration. Use native sanitizers and Valgrind for machine-code, linked-library, and platform reality.

## Tool Comparison

Miri:

- Interprets Rust MIR.
- Strong on Rust-specific UB, validity, provenance, aliasing, initialization, alignment, intrinsic contracts, leaks, and some races.
- Weak on native FFI, networking, OS APIs, and exhaustive execution coverage.

AddressSanitizer:

- Instruments native binaries.
- Strong on out-of-bounds, use-after-free, use-after-return/scope, double free, and some leaks.
- Does not model Rust reference aliasing or provenance.

MemorySanitizer:

- Instruments native binaries for uninitialized reads.
- Requires all relevant code to be instrumented, which is hard with external libraries.

ThreadSanitizer:

- Instruments native binaries for data races.
- Useful for native execution but does not reason about Rust provenance or all atomic-design obligations.

UBSan:

- Instruments native binaries for many low-level undefined or erroneous native conditions.
- Not a Rust abstract-machine checker.

Valgrind Memcheck:

- Runs dynamic binary instrumentation on a synthetic CPU.
- Strong on illegal reads/writes, undefined values, bad frees, overlaps, and leaks across linked code it can see.
- Very slow and not Rust-provenance-aware.

Kani, Flux, Verus, and TLA+:

- Kani gives bounded symbolic evidence for named Rust harnesses.
- Flux checks refinement types at compile time.
- Verus proves Rust-local deductive obligations.
- TLA+ models temporal workflows, protocols, and distributed systems.

## Future Direction

Expect change. Rust's memory model is incomplete; provenance APIs continue to mature; Stacked Borrows and Tree Borrows are research-backed but experimental; host/FFI support is intentionally limited; future rustc versions may change what Miri reports. Always anchor claims to the local nightly and exact command output.

## Source Priority

Prefer current official sources and local evidence in this order:

1. Local `cargo +nightly miri --version`, `cargo +nightly miri --help`, `rustc --version --verbose`, `rust-toolchain.toml`, and actual command output.
2. Miri README: `https://github.com/rust-lang/miri`.
3. Rust Reference UB page: `https://doc.rust-lang.org/reference/behavior-considered-undefined.html`.
4. Rust Reference memory model notes: `https://doc.rust-lang.org/reference/memory-model.html`.
5. Unsafe Code Guidelines glossary: `https://rust-lang.github.io/unsafe-code-guidelines/glossary.html`.
6. Standard-library pointer provenance docs: `https://doc.rust-lang.org/std/ptr/index.html`.
7. RFC 3559, Rust has provenance: `https://rust-lang.github.io/rfcs/3559-rust-has-provenance.html`.
8. Rustc dev guide interpreter chapter: `https://rustc-dev-guide.rust-lang.org/const-eval/interpret.html`.
9. Miri contribution guide: `https://github.com/rust-lang/miri/blob/master/CONTRIBUTING.md`.
10. Stacked Borrows and Tree Borrows papers or project pages linked from the Miri repository.

If local command output and docs disagree, report the conflict and fail closed for required claims.
