# Layer Configurations

Detailed configuration snippets and code templates for each verification
layer. Read the relevant section before generating config for a project.

## Table of contents

1. [Crate boundaries](#1-crate-boundaries)
2. [cargo-deny](#2-cargo-deny)
3. [Typestates and phantom types](#3-typestates-and-phantom-types)
4. [Sealed traits and newtypes](#4-sealed-traits-and-newtypes)
5. [trybuild compile-fail tests](#5-trybuild-compile-fail-tests)
6. [Custom lints with dylint](#6-custom-lints-with-dylint)
7. [Snapshot tests with insta](#7-snapshot-tests-with-insta)
8. [Kani bounded model checking](#8-kani-bounded-model-checking)
9. [Coverage gating](#9-coverage-gating)
10. [Mutation testing](#10-mutation-testing)

---

## 1. Crate boundaries

The compiler is the first guardian. Each DDD layer is its own workspace
crate. If `domain/Cargo.toml` does not list `infra` as a dependency, any
code referencing infrastructure types from domain code fails to compile.

### Workspace layout

```
workspace/
├── Cargo.toml              # [workspace] members
├── crates/
│   ├── domain/             # Pure business logic, no I/O
│   ├── application/        # Use cases, orchestration
│   ├── infrastructure/     # DB, HTTP, external services
│   └── presentation/       # API handlers, UI
├── deny.toml
└── .cargo/mutants.toml
```

### Dependency direction enforcement script

Run this in CI to verify domain never depends on infra:

```bash
#!/bin/bash
set -euo pipefail
DOMAIN_CRATE="my-domain"  # adjust to your crate name
BANNED="my-infra my-persistence"

DEPS=$(cargo metadata --format-version 1 --no-deps | \
  jq -r ".packages[] | select(.name == \"$DOMAIN_CRATE\") | .dependencies[].name")

for dep in $DEPS; do
  for banned in $BANNED; do
    if [ "$dep" = "$banned" ]; then
      echo "FAIL: $DOMAIN_CRATE depends on $banned"
      exit 1
    fi
  done
done
echo "OK: crate boundaries intact"
```

Also run `cargo modules dependencies --lib --acyclic` (from the
cargo-modules crate) to detect cyclic module dependencies.

---

## 2. cargo-deny

Prevents banned external crates from leaking into the wrong layer.
Install: `cargo install cargo-deny`

### deny.toml

```toml
[bans]
multiple-versions = "deny"
deny = [
  { crate = "sqlx",    wrappers = ["my-infra"], reason = "DB access only in infra" },
  { crate = "reqwest", wrappers = ["my-infra"], reason = "HTTP only in infra" },
  { crate = "diesel",  wrappers = ["my-infra"], reason = "ORM only in infra" },
  { crate = "tokio-postgres", wrappers = ["my-infra"], reason = "DB only in infra" },
]
```

Run: `cargo deny check bans`

Also use `cargo-semver-checks` to catch public API shape changes:
`cargo semver-checks --package domain --baseline-rev origin/main`

---

## 3. Typestates and phantom types

Encode state machines into the type system so invalid transitions fail at
compile time. Zero runtime cost via `PhantomData`.

### Template

```rust
use std::marker::PhantomData;

// Define states as zero-sized types
struct Draft;
struct Submitted;
struct Fulfilled;

struct Order<S> {
    id: OrderId,
    items: Vec<LineItem>,
    _state: PhantomData<S>,
}

impl Order<Draft> {
    fn new(id: OrderId, items: Vec<LineItem>) -> Self {
        Order { id, items, _state: PhantomData }
    }

    fn submit(self) -> Order<Submitted> {
        Order { id: self.id, items: self.items, _state: PhantomData }
    }
}

impl Order<Submitted> {
    fn fulfill(self) -> Order<Fulfilled> {
        Order { id: self.id, items: self.items, _state: PhantomData }
    }
}

// Order<Draft>.fulfill() -> compile error
// Order<Fulfilled>.submit() -> compile error
```

Rust's move semantics consume the old state on transition, preventing
use-after-transition bugs.

---

## 4. Sealed traits and newtypes

### Sealed traits

Prevent unauthorized implementations of domain contracts:

```rust
mod private {
    pub trait Sealed {}
}

pub trait DomainEvent: private::Sealed {
    fn aggregate_id(&self) -> &AggregateId;
}

// Only types that impl Sealed in this module can impl DomainEvent
```

### Newtypes with validated constructors

"Parse, don't validate" — invalid values cannot exist:

```rust
pub struct EmailAddress(String);

impl EmailAddress {
    pub fn new(raw: &str) -> Result<Self, ValidationError> {
        if raw.contains('@') && raw.len() > 3 {
            Ok(Self(raw.into()))
        } else {
            Err(ValidationError::InvalidEmail)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

### Railway-oriented error handling

Compose fallible steps with `Result<T, E>` and `?`:

```rust
async fn handle(&self, input: CreateUserInput) -> Result<User, UseCaseError> {
    let validated = validate_input(input)?;
    let checked = check_email_not_exists(validated, &self.repo).await?;
    let user = build_user(checked, &self.factory)?;
    save_user(user, &self.repo).await
}
```

Use `thiserror` in the domain layer, `anyhow` in application/infra.

---

## 5. trybuild compile-fail tests

Prove that forbidden imports produce compiler errors. If someone removes
a crate boundary, these tests fail.

### Setup

Add to domain crate's `Cargo.toml`:

```toml
[dev-dependencies]
trybuild = "1"
```

### Test file

```rust
// tests/architecture.rs
#[test]
fn architecture_boundaries() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/*.rs");
}
```

### Example compile-fail case

```rust
// tests/compile-fail/domain_cannot_use_sqlx.rs
use my_infra::SomeDbType;  // This import must fail

fn main() {}
```

Place a `.stderr` file next to it with the expected compiler error.
trybuild will fail if the code compiles successfully.

---

## 6. Custom lints with dylint

dylint (v5+) loads custom lint rules from dynamic libraries.
Install: `cargo install cargo-dylint dylint-link`

### Scaffold a lint

```bash
cargo dylint new no_unwrap_in_domain
```

### Implementation

```rust
dylint_linting::declare_late_lint! {
    pub NO_UNWRAP_IN_DOMAIN, Deny,
    "`.unwrap()` is forbidden in domain layer code"
}

impl<'tcx> LateLintPass<'tcx> for NoUnwrapInDomain {
    fn check_expr(&mut self, cx: &'tcx LateContext<'tcx>, expr: &'tcx Expr<'_>) {
        if let ExprKind::MethodCall(path, _receiver, _, span) = &expr.kind {
            if path.ident.name.as_str() == "unwrap" {
                span_lint_and_help(
                    cx,
                    NO_UNWRAP_IN_DOMAIN,
                    *span,
                    "`.unwrap()` not allowed in domain code",
                    None,
                    "use `?`, `.map_err()`, or return a typed error instead",
                );
            }
        }
    }
}
```

### Workspace config

```toml
# Cargo.toml (workspace root)
[workspace.metadata.dylint]
libraries = [
    { path = "lints/no_unwrap_in_domain" },
    { path = "lints/domain_no_infra_import" },
]
```

### Built-in clippy lints to enable

```toml
# Cargo.toml (workspace root)
[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
panic_in_result_fn = "deny"
todo = "deny"
unimplemented = "deny"
dbg_macro = "deny"
indexing_slicing = "deny"
string_slice = "deny"
get_unwrap = "deny"
arithmetic_side_effects = "deny"
as_conversions = "deny"
let_underscore_must_use = "deny"
wildcard_imports = "deny"

[workspace.lints.rust]
unsafe_code = "forbid"
```

---

## 7. Snapshot tests with insta

insta captures output as golden reference files. Any change to struct
fields, serialization format, or API surface breaks the snapshot.

### Setup

```toml
[dev-dependencies]
insta = { version = "1", features = ["json", "redactions"] }
```

### Usage

```rust
#[test]
fn test_order_response_shape() {
    let order = OrderResponse {
        id: 42,
        status: "fulfilled".into(),
    };
    insta::assert_json_snapshot!(order);
}
```

### Redactions for non-deterministic fields

```rust
insta::assert_json_snapshot!(response, {
    ".created_at" => "[timestamp]",
    ".id" => "[uuid]",
});
```

### Public API surface locking

Combine insta with cargo-public-api:

```rust
#[test]
fn public_api() {
    let rustdoc_json = rustdoc_json::Builder::default()
        .toolchain(public_api::MINIMUM_NIGHTLY_RUST_VERSION)
        .build()
        .unwrap();
    let api = public_api::Builder::from_rustdoc_json(rustdoc_json)
        .build()
        .unwrap();
    insta::assert_snapshot!(api);
}
```

In CI, set `INSTA_UPDATE=no` so snapshots cannot be silently accepted.

---

## 8. Kani bounded model checking

Kani (v0.66+) provides exhaustive verification within bounded input
domains. Choose proofs with high ROI and keep verification times short.

### Three proof categories

**Panic freedom** — call your function with `kani::any()` inputs:

```rust
#[cfg(kani)]
#[kani::proof]
fn verify_no_panics() {
    let x: i64 = kani::any();
    kani::assume(x >= 0 && x < 1_000_000);
    let m = Money::new(x, Currency::USD);
    // Kani auto-checks for panics
}
```

**Invariant preservation** — valid in, valid out:

```rust
#[cfg(kani)]
#[kani::proof]
fn verify_money_add_preserves_invariant() {
    let a: i64 = kani::any();
    let b: i64 = kani::any();
    kani::assume(a >= 0 && a < 1_000_000);
    kani::assume(b >= 0 && b < 1_000_000);
    if let (Ok(m1), Ok(m2)) = (Money::new(a, Currency::USD), Money::new(b, Currency::USD)) {
        if let Ok(result) = m1.add(&m2) {
            kani::assert(result.amount() >= 0, "money amount remains non-negative");
        }
    }
}
```

**Constructor rejection** — invalid inputs are always rejected:

```rust
#[cfg(kani)]
#[kani::proof]
fn verify_negative_money_rejected() {
    let x: i64 = kani::any();
    kani::assume(x < 0);
    kani::assert(Money::new(x, Currency::USD).is_err(), "negative money is rejected");
}
```

### Avoiding verification blowup

- Never use unbounded `Vec` — cap with `kani::assume(len <= 5)`.
- Bound numeric inputs tightly.
- Use `#[kani::requires()]` / `#[kani::ensures()]` for modular verification.
- Use `#[kani::stub_verified]` in callers of already-verified functions.

### CI integration

```yaml
- uses: model-checking/kani-github-action@v1.1
  with:
    args: --harness verify_money_invariant --output-format terse
```

Run targeted harnesses on PRs, full suite on nightly.

---

## 9. Coverage gating

Use cargo-llvm-cov (not tarpaulin) for coverage measurement.

```yaml
- run: cargo llvm-cov --package domain --fail-under-lines 90
- run: cargo llvm-cov --package application --fail-under-lines 80
- run: cargo llvm-cov --package infra --fail-under-lines 60
```

Coverage tells you what code is reached. Mutation testing (next layer)
tells you what code is actually verified.

---

## 10. Mutation testing

cargo-mutants (v27+) systematically breaks your code and checks whether
tests catch it.

### Config

```toml
# .cargo/mutants.toml
examine_globs = ["src/domain/**/*.rs", "src/application/**/*.rs"]
exclude_re = ["impl Debug", "impl Display", "impl Serialize"]
```

### PR-level gating with --in-diff

```yaml
- run: git diff origin/${{ github.base_ref }}..HEAD > pr.diff
- run: cargo mutants --in-diff pr.diff -vV --in-place
```

Exits non-zero if any mutant survives. For large codebases, shard:
`cargo mutants --shard 0/4`.

### Mutation-property testing feedback loop

1. Write PropTest properties for domain invariants.
2. Run `cargo-mutants`.
3. If a mutant survives despite property tests, the property is too weak.
4. Strengthen the property. Repeat.

Zero tolerance for surviving mutants in domain code.
