# Rust Test Ecosystem Reference

## Cargo.toml — Required Dev Dependencies

```toml
[dev-dependencies]
proptest = "1"
rstest = "0.26.1"        # parametric / table-driven unit tests
insta = { version = "1", features = ["ron", "json"] }
mockall = "0.13"         # use sparingly — prefer fakes
criterion = { version = "0.5", features = ["html_reports"] }

[target.'cfg(fuzzing)'.dependencies]
# cargo-fuzz handles this automatically

# For Kani: install separately
# cargo install --locked kani-verifier && cargo kani setup
```

## rstest — Parametric / Table-Driven Unit Tests

`rstest` eliminates boilerplate when testing many similar cases. Use it aggressively
for unit tests — it makes writing 20 variants as easy as writing 2.

```rust
use rstest::rstest;

// Table-driven happy path
#[rstest]
#[case(1, 1, 2)]
#[case(0, 0, 0)]
#[case(-1, 1, 0)]
#[case(i32::MAX - 1, 1, i32::MAX)]
#[case(i32::MIN, 0, i32::MIN)]
fn add_returns_correct_sum(#[case] a: i32, #[case] b: i32, #[case] expected: i32) {
    assert_eq!(add(a, b), expected);
}

// Table-driven error cases
#[rstest]
#[case("", Error::EmptyInput)]
#[case(" ", Error::BlankInput)]
#[case("a".repeat(256), Error::TooLong { max: 255, got: 256 })]
fn validate_name_rejects_invalid_input(#[case] input: String, #[case] err: ValidationError) {
    assert_eq!(validate_name(&input), Err(err));
}

// Fixture injection
#[fixture]
fn valid_config() -> Config {
    Config { timeout_ms: 1000, max_retries: 3, ..Default::default() }
}

#[rstest]
fn processor_handles_request_with_default_config(valid_config: Config) {
    let result = Processor::new(valid_config).process(&test_request());
    assert_eq!(result, Ok(ProcessResult::Success));
}

// Nested combinations (cartesian product)
#[rstest]
fn matrix_operations_work_for_all_input_sizes(
    #[values(1, 10, 100, 1000)] rows: usize,
    #[values(1, 10, 100)] cols: usize,
) {
    let matrix = Matrix::zeros(rows, cols);
    assert_eq!(matrix.element_count(), rows * cols);
}
```

Key `rstest` features:
- `#[case(...)]` — named test cases, each runs as a separate test
- `#[fixture]` — reusable test setup functions
- `#[values(...)]` — generates cartesian product of all value combinations
- Each case gets its own test name in nextest output for clear failure identification

---

## nextest + tdd-guard-rust

The project's TDD gate. ALWAYS pipe through it:
```bash
cargo nextest run 2>&1 | tdd-guard-rust --project-root . --passthrough
cargo nextest run --test integration_* 2>&1 | tdd-guard-rust --project-root . --passthrough
```

Run specific tests:
```bash
cargo nextest run test_name
cargo nextest run --test-threads 4
```

## proptest — Property-Based Testing

```rust
use proptest::prelude::*;

// Simple strategy
proptest! {
    #[test]
    fn addition_is_commutative(a: i32, b: i32) {
        prop_assert_eq!(add(a, b), add(b, a));
    }
}

// Custom strategy with constraints
proptest! {
    #[test]
    fn bounded_input(x in 1i32..=100) {
        let result = process(x);
        prop_assert!(result > 0);
    }
}

// Composing strategies
fn arb_valid_id() -> impl Strategy<Value = NodeId> {
    "[a-z][a-z0-9_]{0,31}".prop_map(|s| NodeId::new(&s).unwrap())
}

fn arb_document() -> impl Strategy<Value = Document> {
    (arb_valid_id(), prop::collection::vec(arb_valid_id(), 0..10))
        .prop_map(|(id, children)| Document { id, children })
}

proptest! {
    #[test]
    fn document_roundtrip(doc in arb_document()) {
        let serialized = doc.serialize();
        let recovered = Document::deserialize(&serialized).unwrap();
        prop_assert_eq!(doc, recovered);
    }
}
```

Set case count via env:
```bash
PROPTEST_CASES=100000 cargo nextest run
```

Regression file: `proptest-regressions/` — commit these, they replay known failures.

## cargo-fuzz — Fuzzing

Setup (one-time):
```bash
# Install cargo-fuzz (requires nightly toolchain)
cargo install cargo-fuzz
rustup toolchain install nightly  # fuzz requires nightly
cargo fuzz init
cargo fuzz add parse_input  # creates fuzz/fuzz_targets/parse_input.rs
```

Fuzz target template:
```rust
// fuzz/fuzz_targets/parse_input.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use my_crate::Parser;

fuzz_target!(|data: &[u8]| {
    // Rule: must not panic, must not UB, must not OOM on any input
    let _ = Parser::parse(data);
});
```

Structured fuzzing with `arbitrary`:
```rust
use arbitrary::{Arbitrary, Unstructured};

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    if let Ok(input) = MyStruct::arbitrary(&mut u) {
        let _ = process_structured_input(input);
    }
});
```

Run the fuzzer:
```bash
cargo fuzz run parse_input          # run until crash
cargo fuzz run parse_input -- -max_total_time=60  # 60 second run
cargo fuzz corpus parse_input       # show corpus stats
```

Add seeds to `fuzz/corpus/parse_input/` directory.

## Kani — Formal Model Checking

Install:
```bash
cargo install --locked kani-verifier
cargo kani setup
```

Write harnesses in `#[cfg(kani)]` modules:
```rust
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    #[kani::proof]
    fn verify_no_overflow_in_increment() {
        let x: u32 = kani::any();
        kani::assume(x < u32::MAX);
        let result = safe_increment(x);
        assert!(result == x + 1);
    }

    #[kani::proof]
    fn verify_state_machine_transitions_are_exhaustive() {
        let state: State = kani::any();
        let event: Event = kani::any();
        // This should compile — all match arms must be handled
        let _ = state.transition(event);
    }

    #[kani::proof]
    fn verify_index_always_in_bounds() {
        let len: usize = kani::any();
        kani::assume(len > 0 && len <= 100);
        let idx: usize = kani::any();
        kani::assume(idx < len);
        // The function must not panic for any valid (len, idx) pair
        let vec: Vec<i32> = vec![0; len];
        let _ = safe_get(&vec, idx);
    }
}
```

Run:
```bash
cargo kani                         # prove all harnesses
cargo kani --harness verify_no_overflow_in_increment  # single harness
```

## cargo-mutants — Mutation Testing

Install:
```bash
cargo install cargo-mutants
```

Run:
```bash
cargo mutants --timeout 30         # 30s per mutant
cargo mutants --jobs 4             # parallel
cargo mutants -- --test-threads 2  # limit nextest threads
```

Output: `mutants.out/outcomes.json` — lists survived mutants.

Interpret results:
- **caught**: test suite killed this mutant ✓
- **missed**: test suite FAILED to kill this → gap in coverage ✗
- **timeout**: test took too long (likely infinite loop mutant)
- **unviable**: mutant didn't compile (often expected)

For each **missed** mutant, identify the behavior and add a test that kills it.

Typical target: ≥90% catch rate. Below 80% is a serious coverage gap.

## insta — Snapshot Testing

```rust
use insta::assert_snapshot;

#[test]
fn error_message_is_human_readable() {
    let err = ParseError::UnexpectedToken { found: "}", expected: "expression" };
    assert_snapshot!(err.to_string());
}

#[test]
fn complex_ast_serializes_correctly() {
    let ast = build_test_ast();
    insta::assert_json_snapshot!(ast);
    insta::assert_ron_snapshot!(ast);
}
```

Review and accept new snapshots:
```bash
cargo insta review      # interactive review
cargo insta accept      # accept all pending
```

Snapshots live in `src/snapshots/` or `tests/snapshots/`. Commit them.

## criterion — Benchmarking

```rust
// benches/my_bench.rs
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_parse(c: &mut Criterion) {
    let small_input = b"small";
    let large_input = vec![b'x'; 10_000];

    let mut group = c.benchmark_group("parse");
    group.bench_with_input(BenchmarkId::new("small", ""), small_input, |b, i| {
        b.iter(|| parse(black_box(i)))
    });
    group.bench_with_input(BenchmarkId::new("large", ""), &large_input, |b, i| {
        b.iter(|| parse(black_box(i)))
    });
    group.finish();
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);
```

```toml
[[bench]]
name = "my_bench"
harness = false
```

Run: `cargo bench`

## cargo-llvm-cov — Coverage

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --all-features --workspace
cargo llvm-cov --html --output-dir coverage/  # HTML report
cargo llvm-cov --lcov --output-path lcov.info  # for CI
```

Note: High coverage ≠ good tests. 70% with quality assertions > 100% with `is_ok()`.
The mutation kill rate is a better proxy for test quality than line coverage.

## mockall — Mocking (Use Sparingly)

Only mock external I/O side effects. Never mock your own domain logic.

```rust
use mockall::automock;

#[automock]
trait EmailSender {
    fn send(&self, to: &str, body: &str) -> Result<(), SendError>;
}

#[test]
fn notification_sends_email_on_completion() {
    let mut mock = MockEmailSender::new();
    mock.expect_send()
        .with(mockall::predicate::eq("user@example.com"), mockall::predicate::any())
        .times(1)
        .returning(|_, _| Ok(()));

    let notifier = Notifier::new(mock);
    notifier.on_completion("user@example.com", "Job done").unwrap();
    // mock verifies on drop
}
```

**Prefer fakes over mocks:**
```rust
// Better: a fake that actually works
struct FakeEmailSender {
    sent: RefCell<Vec<(String, String)>>,
}

impl EmailSender for FakeEmailSender {
    fn send(&self, to: &str, body: &str) -> Result<(), SendError> {
        self.sent.borrow_mut().push((to.to_string(), body.to_string()));
        Ok(())
    }
}

#[test]
fn notification_sends_email_on_completion() {
    let sender = FakeEmailSender { sent: RefCell::new(vec![]) };
    let notifier = Notifier::new(&sender);
    notifier.on_completion("user@example.com", "Job done").unwrap();
    assert_eq!(sender.sent.borrow().len(), 1);
    assert_eq!(sender.sent.borrow()[0].0, "user@example.com");
}
```

## Test Helpers and Builders

Use the Builder pattern for complex test data. DAMP principle: each test should be
self-contained, but helper functions reduce boilerplate without hiding intent.

```rust
// Good: Builder that makes test intent clear
fn a_document() -> DocumentBuilder {
    DocumentBuilder::new()
}

struct DocumentBuilder { /* ... */ }
impl DocumentBuilder {
    fn with_title(mut self, title: &str) -> Self { /* ... */ self }
    fn with_n_children(mut self, n: usize) -> Self { /* ... */ self }
    fn build(self) -> Document { /* ... */ }
}

#[test]
fn document_rejects_empty_title() {
    let doc = a_document().with_title("").build();
    // title validation happens at creation, not assertion
    assert!(doc.is_err());
}
```

## Running the Full Test Stack

```bash
# 1. Compile checks
cargo clippy --all-targets --all-features -- -D warnings

# 2. Unit + integration tests
cargo nextest run 2>&1 | tdd-guard-rust --project-root . --passthrough

# 3. Proptest (extended)
PROPTEST_CASES=10000 cargo nextest run 2>&1 | grep proptest

# 4. Snapshot review (if any new snapshots)
cargo insta review

# 5. Mutation testing
cargo mutants --timeout 30 --jobs 4 2>&1 | tail -20

# 6. Coverage report
cargo llvm-cov --all-features --workspace 2>&1 | grep -E "TOTAL|^src"

# 7. Kani verification (for critical harnesses)
cargo kani 2>&1

# 8. Moon CI gate
moon run :ci-source
```
