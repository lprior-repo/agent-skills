# Flux RS Patterns

Use these patterns as starting points. Verify exact syntax against the local Flux version before treating any snippet as evidence.

## Four Core Type Forms

Indexed types track exact logical values:

```rust
use flux_rs::attrs::*;

#[spec(fn() -> i32[10])]
fn ten() -> i32 { 10 }

#[spec(fn(x: i32{x < 2147483647}) -> i32[x + 1])]
fn inc_indexed(x: i32) -> i32 { x + 1 }
```

Existential types track a property without requiring the exact value:

```rust
#[spec(fn(x: i32{x != -2147483648}) -> i32{v: 0 <= v})]
fn abs_nonnegative(x: i32) -> i32 {
    if x < 0 { -x } else { x }
}
```

Constraint types attach a predicate to an already named refinement parameter:

```text
fn inc_small({i32[@n] | n < 10}) -> i32[n + 1]
```

Argument syntax is the readable default when supported by the local version:

```text
fn(x: i32, y: i32) -> i32[x + y]
fn(x: i32, y: i32{y > x}) -> i32[x + y]
```

## Function Contracts

```rust
use flux_rs::attrs::*;

#[spec(fn(x: i32{x < 2147483647}) -> i32[x + 1])]
fn inc(x: i32) -> i32 {
    x + 1
}

#[spec(fn(x: i32{x != -2147483648}) -> i32{v: 0 <= v})]
fn abs_nonnegative(x: i32) -> i32 {
    if x < 0 { -x } else { x }
}
```

The input refinement is the precondition. The output refinement is the postcondition. Arithmetic examples must carry overflow preconditions or a proven `no_panic`/overflow obligation; do not teach unbounded `i32 + 1` as safe.

## Domain Type Invariants

```rust
use flux_rs::attrs::*;

#[refined_by(n: int)]
#[invariant(0 < n)]
struct PositiveI32 {
    #[field(i32[n])]
    val: i32,
}

#[spec(fn() -> PositiveI32)]
fn one() -> PositiveI32 {
    PositiveI32 { val: 1 }
}
```

Use this pattern when a value should never exist in an invalid state after construction.

## Validated Input Constructors

```rust
use flux_rs::attrs::*;

#[refined_by(n: int)]
#[invariant(0 <= n && n <= 100)]
struct Percent {
    #[field(i32[n])]
    raw: i32,
}

#[spec(fn(v: i32{0 <= v && v <= 100}) -> Percent)]
fn percent(v: i32) -> Percent {
    Percent { raw: v }
}
```

This is the Flux analogue of validated forms or parsed domain inputs. It is compile-time contract checking, not runtime UI validation.

## Mutable Borrow Post-States

```rust
use flux_rs::attrs::*;

#[flux_rs::sig(
    fn(x: &strg i32[@n])
    ensures *x: i32[n + 1]
)]
fn incr(x: &mut i32) {
    *x += 1;
}
```

Use the local Flux version's strong-reference syntax, commonly `&strg`, whenever callers need the precise post-state of a mutable reference. Ordinary `&mut` is for invariant-preserving weak updates.

```rust
#[flux_rs::sig(fn(x: &mut i32{v: v >= 0}))]
fn decr_nat(x: &mut i32) {
    let y = *x;
    if y > 0 { *x = y - 1; }
}
```

## Refined Enums

```rust
use flux_rs::attrs::*;

#[refined_by(valid: bool)]
enum MyOption<T> {
    #[variant((T) -> MyOption[true])]
    Some(T),
    #[variant(MyOption[false])]
    None,
}
```

Refined enums are good for parser outcomes, validated/unvalidated modes, and small protocol-like state machines. Use a dedicated temporal or interleaving model instead when the property is temporal, distributed, or scheduling-dependent.

## Vector Length Facts

```rust
use flux_rs::attrs::*;

#[spec(fn(xs: &Vec<i32>[@n]) -> usize[n])]
fn len_i32(xs: &Vec<i32>) -> usize {
    xs.len()
}

#[flux_rs::sig(
    fn(xs: &strg Vec<i32>[@n], x: i32)
    ensures *xs: Vec<i32>[n + 1]
)]
fn push_i32(xs: &mut Vec<i32>, x: i32) {
    xs.push(x)
}
```

Prefer committed standard-library specs when the local Flux version covers the needed `Option` or `Vec` property. Otherwise use a narrow `extern_spec` or create an opaque wrapper only for domain abstraction or representation hiding.

The canonical refined-vector API shape is:

```text
impl RVec<T> {
    fn new() -> RVec<T>[0];
    fn len(self: &RVec<T>[@n]) -> usize[n];
    fn get(self: &RVec<T>[@n], idx: usize{v: v < n}) -> &T;
    fn get_mut(self: &mut RVec<T>[@n], idx: usize{v: v < n}) -> &mut T;
    fn push(self: &strg RVec<T>[@n], value: T) ensures *self: RVec<T>[n + 1];
}
```

## Opaque Wrapper Boundary

```rust
use flux_rs::attrs::*;

#[opaque]
#[refined_by(n: int)]
struct RVec<T> {
    inner: Vec<T>,
}

impl<T> RVec<T> {
    #[trusted]
    #[spec(fn() -> RVec<T>[0])]
    fn new() -> Self {
        Self { inner: Vec::new() }
    }
}
```

The trusted constructor is part of the trusted base. Keep wrappers tiny, inspectable, and covered by runtime tests when possible.

## Extern Specs And Trusted APIs

Use `#[extern_spec]` when specifying behavior for external or std items. Use `#[trusted]` only when the verifier cannot see or cannot prove a body but the contract is independently justified.

```rust
use flux_rs::extern_spec;

#[extern_spec(std::mem)]
#[flux_rs::sig(fn(&mut i32[@a], &mut i32{v: a < v}) -> ())]
fn swap(a: &mut i32, b: &mut i32);
```

Trust audit rule: every trusted item needs a reason, scope, evidence, and owner. Do not call broad trusted regions verified.

## Typestate Contracts

Encode protocol state in logical indices and require the correct state on method receivers:

```rust
use flux_rs::attrs::*;

#[opaque]
#[refined_by(enabled: bool, direction: bool, mode: int)]
struct GpioConfig { /* hardware register */ }

impl GpioConfig {
    #[spec(fn(me: &GpioConfig{v: v.enabled && !v.direction}) -> bool)]
    pub fn get_input_status(&self) -> bool { true }
}
```

Do not claim hardware, async, or temporal behavior is verified. The Flux property is the compile-time receiver-state precondition.

## Generic Refinement Sketches

Treat Horn, Hindley, and associated generic refinements as advanced and version-sensitive. Verify against the POPL 2025 material and local tests before using them in a claimed proof.

```text
struct Query<R>[hdl inv: R -> bool];
fn and(self: Query<R>[@q1], rhs: Query<R>[@q2]) -> Query<R>[|r| q1(r) && q2(r)];
fn not(self: Query<R>[@q]) -> Query<R>[|r| !q(r)];
```

## Detached Specs And Local Invariants

```rust
flux_rs::macros::detached_spec! {
    fn inc(n: i32{n < 2147483647}) -> i32[n + 1];
}

// Use local invariant helpers only where the solver lacks a loop fact.
// flux_rs::macros::invariant!(i: int, n: int; i <= n);
```

Detached specs are useful at module boundaries. Local invariants and qualifiers are targeted solver help, not a substitute for domain type design.

## Panic Freedom

Use `#[no_panic]` or conditional panic specifications only when the local Flux version supports the exact syntax and the command proves it.

Good targets include safe unwrap preconditions, checked indexing, and impossible branches. Do not claim panic freedom from ordinary Rust tests.

## Async And I/O

Flux does not provide native async data-fetching or lifecycle models. Use a trusted shell and verified core:

```text
async/network/database adapter -> thin trusted or extern boundary -> verified parser/state transition/core function
```

If the correctness property is about ordering, retries, cancellation, fairness, leases, or eventuality, use a dedicated temporal or interleaving model rather than Flux alone.

## Anti-Patterns

- Describing signals, effects, or components as Flux RS concepts.
- Adding `#[trusted]` until the crate passes.
- Repeating the same stable fact in many function specs instead of creating a refined type.
- Omitting `ensures` from mutation APIs that need exact post-state facts.
- Creating a wrapper for `Vec` or `Option` before checking whether the local Flux version has usable specs.
- Treating `#[ignore]` as harmless because the command exits zero.
- Claiming verification from snippets that were not run with the exact local Flux toolchain.
- Claiming full functional correctness, temporal behavior, or async scheduling from lightweight refinements.
