# Flux RS Deep Guide

This is the dense reference for agents working with Flux RS. Use it to ground implementation, review, and triage decisions. Verify exact syntax against the local Flux version before treating examples as runnable evidence.

## What Flux Is

Flux is a refinement type checker plugin for the Rust compiler. It lets Rust types carry SMT-decidable logical predicates that are checked at compile time with no runtime cost and no separate proof language.

Core framing:

| Rust type | Refined meaning |
| --- | --- |
| `i32` | Any 32-bit signed integer |
| `i32{v: v >= 0}` | Any nonnegative `i32` |
| `i32[10]` | The singleton `i32` value equal to 10 |
| `Vec<T>[n]` | A vector whose length/index is tracked as `n` |

Flux extends Rust's ownership discipline with logical facts. Rust prevents use-after-free and aliasing bugs; Flux targets lightweight logical bugs such as out-of-bounds indexing, invalid ranges, illegal states, and precondition violations.

## Honest Maturity

Flux is research-grade software as of 2026.

Do state:

- It requires nightly Rust and a pinned toolchain.
- It depends on `liquid-fixpoint` plus an SMT solver such as Z3 4.15 or later, with cvc5 available in some configurations.
- It is usually installed from the upstream repository or release artifacts, not `cargo install` from crates.io.
- Unsupported Rust features can produce internal Flux errors or panics, not polished diagnostics.
- It is best for quantifier-free refinement properties on Rust types, not arbitrary functional correctness.

Do not state:

- That Flux is production-ready for arbitrary crates.
- That the whole standard library is refined by default.
- That a Rust test pass, clippy pass, Kani run, or code review is Flux proof evidence.
- That Flux proves temporal behavior, distributed protocols, async scheduling, or runtime memory behavior.

## Internal Pipeline

Read errors through the three-phase pipeline.

| Phase | What happens | Diagnostic meaning |
| --- | --- | --- |
| Spatial phase | Flux maps program identifiers to heap locations and decides where refinements may be assumed or must be asserted. Unknown intermediate facts are Horn variables. | A location or borrow shape may not match the contract you thought you wrote. |
| Checking phase | Flux performs refinement type checking and emits constrained Horn clauses. Subtyping becomes logical implication, such as proving `n + 1 > 0`. | A postcondition, precondition, or assignment obligation is generated here. |
| Inference phase | `liquid-fixpoint` solves Horn constraints through an SMT solver. | Failure means Flux could not infer/prove the required predicate from available refinements. |

The design is tractable because structure lives in type constructors while logical facts live in quantifier-free refinements. Flux can infer many loop invariants from templates, but it cannot infer facts outside its supported logic or outside visible contracts.

## Core Type Forms

Indexed refinements pin a value to an exact logical index:

```rust
use flux_rs::attrs::*;

#[spec(fn() -> i32[10])]
pub fn mk_ten() -> i32 { 10 }

#[spec(fn(n: i32) -> bool[0 < n])]
pub fn is_pos(n: i32) -> bool {
    if 0 < n { true } else { false }
}
```

Existential refinements constrain a value without naming the exact value:

```rust
#[spec(fn(n: i32) -> i32{v: 0 <= v && n <= v})]
pub fn abs(n: i32) -> i32 {
    if 0 <= n { n } else { 0 - n }
}
```

Constraint types apply a predicate to any type and are useful when an index appears in multiple positions:

```text
fn inc_small({i32[@n] | n < 10}) -> i32[n + 1]
```

Argument syntax names refinement parameters in a Rust-like style:

```text
fn(x: i32, y: i32) -> i32[x + y]
fn(x: i32, y: i32{y > x}) -> i32[x + y]
```

## Refinement Grammar

Keep predicates in the decidable fragment. Common forms include:

```text
r ::= n
    | x
    | x.f
    | r + r
    | r - r
    | n * r
    | if r { r } else { r }
    | f(r...)
    | true | false
    | r == r | r != r
    | r < r | r <= r | r > r | r >= r
    | r || r | r && r | r => r | !r
```

This restriction is intentional. It lets Flux generate and solve Horn clauses automatically. If the desired property needs universal quantifiers, sortedness of an arbitrary collection, deep tree balance, or full relational correctness, consider Prusti, Creusot, Verus, or theorem-prover-backed work instead.

## Ownership And Refinements

Flux's key difference from other refinement systems is that it uses Rust ownership.

Owned locations support strong updates. If `x` has type `i32[n]`, a verified `x += 1` can update the logical type to `i32[n + 1]` because Rust guarantees no alias can observe the old value.

Ordinary mutable references preserve invariants. A `&mut nat` borrow can mutate through the reference only if the pointee remains a `nat` afterward. This is a weak update relative to the caller-visible logical type.

Strong references model caller-visible refinement changes. Use the local Flux version's `&strg` plus `ensures` pattern when a borrow must change the caller's known refinement:

```rust
#[flux_rs::sig(
    fn(x: &strg i32[@n])
    ensures *x: i32[n + 1]
)]
fn incr(x: &mut i32) {
    *x += 1;
}
```

Polymorphism can carry refinements for free. Generic functions such as `swap<T>` can preserve a refined `T` without writing a custom spec for every refined type, assuming the boundary itself is specified or visible.

## Refined Data Types

Refined enums use `#[refined_by(...)]` to declare logical state and `#[variant(...)]` to assign constructor states:

```rust
use flux_rs::*;

#[refined_by(len: int)]
enum List<T> {
    #[flux_rs::variant(List<T>[0])]
    Nil,
    #[flux_rs::variant((T, Box<List<T>[@n]>) -> List<T>[n + 1])]
    Cons(T, Box<List<T>>),
}
```

Refined structs can tie runtime fields to logical indices. Opaque structs hide representation and expose only refined APIs. Opaque internals usually require audited `#[trusted]` methods or external specs because Flux cannot inspect the hidden representation from clients.

## Central Collection Pattern

The canonical vector pattern tracks length in the type and bounds in method preconditions:

```text
impl RVec<T> {
    fn new() -> RVec<T>[0];
    fn len(self: &RVec<T>[@n]) -> usize[n];
    fn get(self: &RVec<T>[@n], idx: usize{v: v < n}) -> &T;
    fn get_mut(self: &mut RVec<T>[@n], idx: usize{v: v < n}) -> &mut T;
    fn push(self: &strg RVec<T>[@n], value: T) ensures *self: RVec<T>[n + 1];
}
```

Use this shape for length-indexed collections, bounded queues, and wrappers around standard collections. Do not assume the local Flux install already has every standard-library method refined; check the local specs or write a narrow `extern_spec`.

## Typestate Pattern

Typestate is a strong Flux use case. Encode state machine facts as logical indices and put state-dependent method preconditions on receivers:

```rust
use flux_rs::attrs::*;

#[opaque]
#[refined_by(enabled: bool, direction: bool, mode: int)]
struct GpioConfig { /* hardware register */ }

impl GpioConfig {
    #[flux_rs::sig(fn(me: &strg GpioConfig[@old], is_enabled: bool)
           ensures *me: GpioConfig{v: v.enabled == is_enabled})]
    pub fn set_enable(&mut self, _is_enabled: bool) { /* adapter */ }

    #[flux_rs::sig(fn(me: &strg {GpioConfig[@old] | old.enabled}, is_output: bool)
           ensures *me: GpioConfig[old.enabled, is_output, old.mode])]
    pub fn set_direction(&mut self, _is_output: bool) { /* adapter */ }

    #[spec(fn(me: &GpioConfig{v: v.enabled && !v.direction}) -> bool)]
    pub fn get_input_status(&self) -> bool { true }
}
```

For real hardware, I/O bodies are usually trusted or external boundaries. The verified value is in the compile-time protocol restriction and strong-reference post-state, not in pretending hardware effects were proved. If the local Flux version uses different receiver syntax, require command evidence before accepting the example.

## Extern Specs

Use `#[extern_spec]` to refine code you do not own:

```rust
use flux_rs::extern_spec;

#[extern_spec(std::mem)]
#[flux_rs::sig(fn(&mut i32[@a], &mut i32{v: a < v}) -> ())]
fn swap(a: &mut i32, b: &mut i32);
```

Extern specs are assumptions. They belong in the trusted-boundary report and should be as narrow as possible.

## Incremental Adoption

Use `#[flux_rs::ignore]` or crate-level ignore to scope migration, then remove ignores module by module. Ignored code is not verified and should not be counted as proof. Use `#[flux_rs::trusted]` for small wrappers whose signatures are checked but bodies are assumed.

Good migration order:

1. Enable Flux metadata in the target crate.
2. Add public preconditions and postconditions for one small module.
3. Encode stable legal states in refined structs or enums.
4. Add negative tests or expected-failure cases for illegal states.
5. Reduce trusted and ignored surface as specs mature.

## Generic Refinements

Generic refinements are advanced and version-sensitive.

Horn generic refinements abstract predicates that appear positively, such as available indices in a multi-borrow API:

```text
struct MultiIdx<'a, T>[hrn available: int -> bool];

fn get(self: &strg Self[@av], idx: usize{ av(idx) }) -> &'a mut T
    ensures self: Self[|i| av(i) && i != idx];
```

Hindley generic refinements can appear negatively and are useful for modular query-like predicates:

```text
struct Query<R>[hdl inv: R -> bool];
fn and(self: Query<R>[@q1], rhs: Query<R>[@q2]) -> Query<R>[|r| q1(r) && q2(r)];
fn or(self: Query<R>[@q1], rhs: Query<R>[@q2]) -> Query<R>[|r| q1(r) || q2(r)];
fn not(self: Query<R>[@q]) -> Query<R>[|r| !q(r)];
```

Associated generic refinements let traits expose abstract in-bounds predicates. Use the POPL 2025 material and local tests before relying on this syntax in production work.

## Tool Comparison

| Tool | Approach | Best for | Limits |
| --- | --- | --- | --- |
| Flux | Liquid/refinement types over Rust | Bounds, value invariants, typestate, size-indexed APIs | Research-grade, decidable fragment, partial Rust support |
| Prusti | Program logic/separation logic | More expressive functional contracts | More manual invariants and annotation overhead |
| Creusot | Pearlite logic with proof tooling | Deep functional correctness | More proof work and toolchain complexity |
| Verus | SMT-backed verification language for Rust-like code | Ghost state, unsafe-code proofs, rich invariants | New constructs and larger learning curve |
| Kani | Bounded model checking | Concrete bug finding on existing code | Bounded exploration, not unbounded proofs |

Flux's unique advantage is low annotation cost for properties expressible as type refinements. The PLDI 2023 evaluation reported much lower annotation overhead and substantially faster verification than Prusti for the evaluated vector-heavy benchmarks, especially because Flux avoids many explicit loop invariant annotations.

## Debugging Cues

When Flux says a postcondition cannot be proved, ask:

- Does the implementation actually satisfy the spec?
- Is the necessary fact visible in the input type, refined struct, enum variant, or `ensures` post-state?
- Is a std/external API being assumed without an `extern_spec`?
- Did an ordinary `&mut` preserve an invariant where a strong-reference post-state was needed?
- Is arithmetic overflow or underflow blocking the proof?
- Is the desired property outside Flux's decidable fragment?

## Minimal Example

Single-file mode should be checked with the local Flux binary and exact flags accepted by the installed version, for example `flux --crate-type=lib demo.rs`.

```rust
#![allow(unused)]
extern crate flux_rs;
use flux_rs::attrs::*;

#[spec(fn() -> i32[10])]
pub fn mk_ten() -> i32 { 5 + 5 }

// Expected failure when uncommented: returns 9, not 10.
// #[spec(fn() -> i32[10])]
// pub fn mk_ten_broken() -> i32 { 5 + 4 }

#[spec(fn(b: bool[true]))]
pub fn assert_true(b: bool) {
    if !b { panic!("assertion failed") }
}

// Expected failure when uncommented: precondition is false.
// pub fn bad_assert() { assert_true(2 + 2 == 5); }

#[spec(fn(n: i32) -> i32{v: 0 <= v && n <= v})]
pub fn abs(n: i32) -> i32 {
    if 0 <= n { n } else { 0 - n }
}
```

## Primary Resources

- Flux Book: `https://flux-rs.github.io/flux/index.html`
- Online playground: `https://flux-rs.github.io/`
- Main repository: `https://github.com/flux-rs/flux`
- Flux demo repository: `https://github.com/flux-rs/flux-demo`
- Additional examples: `https://github.com/flux-rs/examples`
- PLDI 2023 paper: `https://dl.acm.org/doi/abs/10.1145/3591283`
- PLDI 2023 artifact: `https://github.com/flux-rs/pldi23-artifact`
- Generic Refinement Types, POPL 2025: check upstream paper links from the Flux repository and discussion channels.
- GitHub Discussions: `https://github.com/flux-rs/flux/discussions`
- Zulip/community links: use official upstream links when needed.
