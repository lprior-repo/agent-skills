# Flux RS Practical Verification

## Source Priority

Use these sources in order when deciding whether syntax or behavior is real:

1. The local crate, pinned toolchain, committed Flux annotations, and Flux config.
2. The official Flux Book: `https://flux-rs.github.io/flux/index.html`.
3. The official repository: `https://github.com/flux-rs/flux`.
4. Local Flux source tree specs and regression tests, when installed from source.
5. The PLDI 2023 paper and artifact for design rationale and evaluation context.
6. The POPL 2025 generic refinement material for Horn, Hindley, and associated refinements.
7. Community discussions and issues for prototype caveats.

Do not rely on memory for newly evolving syntax. Flux is research-grade, nightly-only, and version-sensitive. Unsupported Rust constructs may produce Flux internal errors rather than clean user diagnostics.

## Correct Scope

Flux RS is a Rust refinement type checker and compiler plugin. It is not the UI or state-management pattern named Flux.

| Requested concept | Flux RS status | Correct translation |
| --- | --- | --- |
| Reactive primitives | Not applicable | Refinement types, qualifiers, spec functions |
| Signals and effects | Not applicable | Preconditions, postconditions, `ensures`, panic-freedom |
| UI scopes | Not applicable | Rust ownership and borrow scopes |
| State management | Applicable | Refined structs, enums, and verified mutation |
| Component composition | Loose analogy only | Composition of refined APIs, traits, and opaque wrappers |
| Forms | No built-in abstraction | Validated constructors and type invariants |
| Async data fetching | No native tutorial | Trusted or extern boundary around async/I/O code |
| Memory leaks | Not primary purpose | Rust ownership plus complementary runtime tools |

Flux is strongest for lightweight correctness properties that fit types: bounds safety, legal-state invariants, refined constructors, panic-freedom preconditions, size-tracked collections, ownership-sensitive mutation, typestate, and API contracts for external or standard-library items.

Do not assume the standard library is refined by default. Use committed local specs when present; otherwise model exactly what you need with `#[extern_spec]` or a narrow opaque wrapper.

## Mental Model

Think of Flux as proof-carrying Rust interfaces:

- A function signature is a precondition and postcondition.
- A struct or enum definition is a legal-state space.
- A mutable borrow preserves the pointee invariant unless the local Flux version's strong-reference pattern, such as `&strg` plus `ensures`, is used for a caller-visible post-state.
- An opaque wrapper hides representation and exposes only refined API effects.
- An extern spec or trusted wrapper defines what the verifier assumes about code it does not check.

Common type ideas:

| Form | Meaning |
| --- | --- |
| `B[r]` | Base type with refinement index, such as `i32[n]` or `Vec[n]` |
| `B{v: p(v)}` | Existential refined value satisfying predicate `p` |
| `{T | p}` | Constraint over an already named refinement parameter |
| `@n` | Bind an input refinement value so outputs can mention it |
| `&T[@n]` | Shared reference whose pointee refinement is named |
| `&mut T` | Mutable borrow that must preserve the underlying refined type/invariant |
| `&strg T[@n] ... ensures *x: ...` | Strong reference with a caller-visible post-state |

Rust lifetimes remain Rust lifetimes. Flux layers refinements on top of Rust ownership and aliasing; it does not introduce a separate lifetime language for day-to-day use.

## Architecture

Flux runs as a rustc driver with macros on the front and solver-backed checking behind the scenes. The useful mental model is spatial analysis, refinement checking, then Horn-clause inference.

Pipeline:

```text
Rust code with Flux attributes
Spatial phase: map identifiers to locations and introduce Horn variables
Checking phase: generate refinement subtyping and postcondition obligations
Inference phase: solve constrained Horn clauses with Liquid Fixpoint
SMT solver such as Z3 or cvc5
Flux diagnostics, metadata, and pass/fail results
```

Relevant official crates include `flux-driver`, `flux-desugar`, `flux-fhir-analysis`, `flux-refineck`, `flux-fixpoint`, config, metadata, middle-layer type crates, macros, and tests.

Practical implication: Flux feels type-directed at the source level, but proof failures often require tightening source types, adding a stronger receiver/post-state, adding a small invariant/qualifier, or inspecting generated constraints and checker traces.

## Surface API Map

| Surface form | Purpose | Typical use |
| --- | --- | --- |
| `#[spec(fn(...) -> ...)]` | Refined function signature | Most user-facing contracts |
| `#[sig(fn(...) -> ...)]` | Compact signature form | Extern specs and helper specs |
| `#[refined_by(...)]` | Add refinement indices | Length, validity, dimensions, ranges |
| `#[invariant(...)]` | State legal relation over indices | Domain invariants |
| `#[field(T[n])]` | Tie a runtime field to an index | Mirror data into refinements |
| `#[variant(...)]` | Give enum constructors states | Option-like modes, protocol states |
| `&strg T ... ensures *x: T[...]` | State mutable-reference post-state | Caller-visible strong updates |
| `#[opaque]` | Hide representation | Verified collection/map wrappers |
| `#[trusted]` | Assume signature for unchecked body | Thin trusted wrappers only |
| `#[trusted_impl]` | Trust impl surface | Audited impl boundaries only |
| `#[ignore]` | Skip analysis | Incremental adoption |
| `#[extern_spec]` | Specify external/std items | Library modeling |
| `#[alias(...)]` | Name refined aliases | Domain-specific type names |
| `#[assoc(...)]` | Associated refinements | Traits and iterators |
| `detached_spec!` | Specs outside inline attrs | Large module organization |
| `invariant!` | Local invariant or qualifier help | Loops and solver guidance |
| `#[no_panic]` | Panic-freedom claim | Safe unwrap/index/precondition work |
| `#[no_panic_if(...)]` | Conditional panic-freedom | Evolving feature; verify support |
| `#[should_fail]` | Expected rejection | Negative tests and regressions |

Flux also exposes advanced mechanisms such as qualifiers, constants, reflected/spec functions, hide/reveal, `reft`, detached specification forms, Horn generic refinements, Hindley generic refinements, and associated refinements. Treat exact syntax as source-version-sensitive.

## Working Habits

### Public Contracts First

Start with functions humans already care about. Put preconditions in argument refinements and postconditions in return refinements. This makes failures meaningful and avoids annotation noise.

### Validated State By Construction

For parser, config, and form-like inputs, encode the valid value as a refined type with a constructor whose input refinement proves legality. Do not scatter repeated runtime assertions across consumers.

### Ownership-Aware Mutation

Flux can reason strongly about local mutation and mutable borrows, but ordinary `&mut` borrows maintain the underlying refined type. If a caller must know that a counter increased or a vector length changed, use the local Flux version's strong-reference/`ensures` pattern and prove it.

### Collection Sizes As Types

Use Flux's committed stdlib specs for `Option` and `Vec` only when the local version actually provides and checks the needed methods. If the method is not modeled, add a narrow `extern_spec` or wrap it behind an opaque API.

Use an `#[opaque]` wrapper when you need a domain abstraction such as a refined queue, bounded map, or collection with representation hidden behind a trusted API.

### Trusted Shell, Verified Core

Flux is not an async/network/database verifier. Put external effects at thin boundaries with `extern_spec` or carefully reviewed `trusted` wrappers. Verify the pure parser, state transition, index calculation, or invariant-preserving core around that boundary.

### Incremental Adoption

Use crate-level ignore, item-level re-enable, include patterns, query caching, and package-local config. Flux adoption should be scoped and ratcheted, not all-or-nothing.

### Debugging Loop

Isolate the failing item, run with timings and constraint/checker dumps, inspect whether inference lacks a needed fact, then strengthen a type, add an invariant, add a qualifier, or localize a trusted boundary.

### Verification Performance

Verification performance is not runtime performance. Main knobs are cache, solver choice, include patterns, `FLUXFLAGS`, timing output, and constraint/checker dumps.

## Project Shape

A practical Flux layout:

```text
src/
  lib.rs            # public API and crate-level Flux attrs
  types.rs          # refined structs, enums, aliases
  specs.rs          # detached specs, qualifiers, assoc refinements
  verified/         # algorithmic code expected to verify
  adapters/         # extern specs, trusted wrappers, I/O boundaries
tests/
  flux/
    pos/            # examples expected to verify
    neg/            # should_fail or expected rejections
flux.toml           # optional crate-local Flux execution config
```

Editor integration may exist in upstream docs, but editor output is not verification evidence. CLI output from `cargo flux`, `flux`, or the repo's exact verification script is evidence.

## Pitfalls

- Treating Flux RS like a reactive UI framework.
- Using `#[trusted]` to silence hard proof work instead of narrowing the trusted shell.
- Forgetting the strong-reference `ensures` pattern and expecting ordinary `&mut` APIs to expose exact post-state facts.
- Expecting inference to solve every loop or arithmetic fact without invariants or qualifiers.
- Ignoring prototype/WIP status around newer features and lifetime corner cases.
- Writing spec soup that repeats the same stable domain fact locally instead of encoding it in a type.
- Claiming runtime memory or leak guarantees from Flux alone.

## Practical Checklist

- Install and run Flux in crate mode before relying on single-file experiments.
- Pin toolchain and expect prototype breakage.
- Start with public contracts and domain type invariants.
- Prefer type-level invariants over repeated local assertions.
- Use the local strong-reference `ensures` pattern for exact caller-visible post-states.
- Use existing `Option` and `Vec` specs only after confirming the local Flux version provides them.
- Keep `opaque` and `trusted` APIs small and auditable.
- Use extern specs or trusted wrappers only at external boundaries.
- Adopt incrementally with ignore, include patterns, and crate-local configuration.
- Use cache, timings, solver selection, and dumps before broad rewrites.
- Treat source, std specs, and regression tests as part of the reference set.

## Resources

- Flux Book: `https://flux-rs.github.io/flux/index.html`
- Install and run guide: `https://flux-rs.github.io/flux/guide/install.html`
- Specifications guide: `https://raw.githubusercontent.com/flux-rs/flux/main/book/src/guide/specifications.md`
- Architecture guide: `https://raw.githubusercontent.com/flux-rs/flux/main/book/src/guide/architecture.md`
- Online playground and tutorial entry: `https://flux-rs.github.io/`
- Main repository: `https://github.com/flux-rs/flux`
- Flux demo repository: `https://github.com/flux-rs/flux-demo`
- Additional examples: `https://github.com/flux-rs/examples`
- PLDI artifact repository: `https://github.com/flux-rs/pldi23-artifact`
- PLDI 2023 paper: `https://dl.acm.org/doi/abs/10.1145/3591283`
- GitHub Discussions: `https://github.com/flux-rs/flux/discussions`
- Related repos: `https://github.com/flux-rs/flux-to-lean-demo`, `https://github.com/flux-rs/verify-rust-std`
