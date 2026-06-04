# Flux RS Curriculum

## Recommended Learning Path

1. Start in the online playground and interactive tutorial before installing anything.
2. Work through tutorial chapter 1 on refinements: indexed types, existentials, preconditions, and postconditions.
3. Read the specification guide for refinement grammar, extern specs, opaque types, trusted, and ignore.
4. Read `flux-demo` examples such as typestate, vector examples, and k-means.
5. Install Flux locally and add refinements to one small real module.
6. Read PLDI 2023 sections 2 and 3 to understand the formal model behind diagnostics.
7. Read the POPL 2025 generic refinement material only when specifying traits or higher-order predicate abstractions.

## Stage 0: Tool Reality

Goal: prove Flux is installed and the target crate actually uses it.

Tasks:

- Run `cargo flux --help` or the repo's exact Flux setup check.
- Find `[package.metadata.flux] enabled = true` or the accepted repo equivalent.
- Identify Rust nightly, Flux binary, Liquid Fixpoint, solver, and config assumptions.
- Identify all crate-level ignore/trust settings.

Failure mode: writing annotations for a crate Flux is not checking, or reporting success from editor integration rather than CLI evidence.

## Stage 1: Simple Function Contracts

Goal: use refinements as preconditions and postconditions.

Practice:

- `inc`: return input plus one.
- `abs`: return nonnegative value.
- Bounded arithmetic helper with a caller-visible postcondition.
- Boolean precondition helper such as `assert_true(bool[true])`.

Acceptance: `cargo flux` or `flux` proves the exact target.

## Stage 2: Legal State Types

Goal: encode stable domain facts once.

Practice:

- Positive integer wrapper.
- Percent/range-bounded config value.
- Non-empty or bounded collection abstraction.

Acceptance: legal constructors verify, consumers rely on the refined type instead of repeated assertions, and an exact negative or `#[should_fail]` Flux target proves illegal constructors are rejected.

## Stage 3: Ownership And Mutation

Goal: prove post-states through mutable borrows.

Practice:

- Counter increment with the local strong-reference `ensures` pattern, commonly `&strg`.
- `Vec` push length increases by one.
- `take`-style transition that changes enum/option state.

Acceptance: callers can use the new post-state without extra assumptions, and the report distinguishes ordinary invariant-preserving `&mut` from strong-reference updates.

## Stage 4: Collections And Opaque APIs

Goal: rely on committed local/upstream specs when present, then introduce `extern_spec` or `#[opaque]` only for abstraction or missing external behavior.

Practice:

- Length-safe indexing using local std `Vec` specs when available, or a narrow `extern_spec`/opaque wrapper otherwise.
- Option unwrap only under a proven condition when the local specs support it.
- Opaque refined wrapper with tiny trusted constructor and verified public effects.

Acceptance: trusted surface is documented and narrower than the verified API.

## Stage 5: Extern Boundaries

Goal: verify the core while acknowledging unverified shells.

Practice:

- Extern spec for a small library function.
- Trusted adapter around parsing, FFI, async fetch, or database read.
- Pure verified function consuming the trusted result.
- Audit that std/external assumptions are not counted as verified bodies.

Acceptance: the report separates verified refinement facts from assumed boundary facts.

## Stage 6: Debugging And Scale

Goal: keep verification usable in a larger crate.

Practice:

- Use include patterns to narrow scope.
- Use `FLUXFLAGS="-Ftimings"` to find slow items.
- Use `FLUX_DUMP_TIMINGS=true` and `FLUX_CATCH_BUGS=1` when appropriate for broad triage.
- Use constraint/checker dumps for one failing item.
- Replace repeated local specs with a domain type or alias.

Acceptance: changes reduce proof noise without widening trust or ignoring the property being claimed.

## Stage 7: Advanced Generic Refinements

Goal: use Horn, Hindley, or associated generic refinements only when the simpler API shape cannot express the contract.

Practice:

- Trait in-bounds predicate via associated refinement.
- Query-like predicate composition with Hindley generics.
- Multi-borrow availability tracking with Horn generics.

Acceptance: exact local syntax is proven by `cargo flux` or a dedicated Flux regression, and unsupported syntax is reported as a blocker.

## Evaluation Tasks

An agent using this skill should be able to:

- Explain why Flux RS is not a reactive framework.
- Explain that Flux is research-grade, nightly-only, and solver-backed.
- Add a refined public contract to a Rust function.
- Use indexed, existential, constraint, and argument syntax appropriately.
- Encode a domain invariant with `refined_by`, `invariant`, and `field`.
- Add strong-reference `ensures` to a mutable-borrow API when callers need a post-state.
- Check local std `Option`/`Vec` support before wrappers or extern specs.
- Identify and report `trusted`, `extern_spec`, and `ignore` boundaries.
- Run the exact Flux command or report a blocker.
- Avoid claiming memory-leak, temporal, or async-scheduling proof from Flux alone.
- Compare Flux honestly with Prusti, Creusot, Verus, and Kani.

## Tool Selection

Choose Flux for lightweight Rust refinement properties: bounds, legal states, length/index facts, panic preconditions, and ownership-aware mutation.

Choose Verus for richer Rust-local proof engineering, ghost/proof code, loop proofs, and functional correctness beyond Flux's lightweight refinement style.

Choose Kani, fuzzing, Loom, Shuttle, Stateright, or dedicated UB tooling for concrete execution exploration, malformed input, thread interleavings, and runtime-sensitive defects.
