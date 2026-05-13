# Test Evidence Rules — Applied to Rust Suites

These rules gate test evidence, not test implementation style. Loops, conditionals,
table-driven tests, helpers, and local mutability are acceptable when they increase
coverage and keep assertions exact. Reject only patterns that hide behavior, weaken
assertions, or create nondeterminism.

Rules 11 (read every line) and 12 (tests first) are enforced by the go-skill pipeline
order, not by the test-reviewer. They are omitted here.

---

## Rule 1 — Keep Evidence Traceable

**General**: A failing test should point to a specific behavior and expected value.

**Applied to tests**:
- Test body has clear Given → When → Then evidence.
- Conditionals and helper chains are allowed if every path asserts an exact expected value.
- Early-exit chains (`if x { return; }` patterns) that skip assertions are evidence holes.

**Audit**:
```bash
# Branching heuristic — inspect only for skipped assertions or hidden cases
grep -n "        if \|        match " tests/ src/
```

**Failure**: Conditional/helper path that skips assertions or hides which case failed = **MAJOR**.

---

## Rule 2 — Bound Generated Coverage

**General**: Every iteration needs an explicit maximum.

**Applied to tests**:
- Loops, table-driven tests, and generated cases are allowed.
- Every generated space must be bounded or reproducible: fixed vectors, named cases,
  proptest strategies with committed regressions, or fuzz corpora with seeds.
- Each case must assert exact values or exact error variants.

**Audit**:
```bash
grep -rn "for .* in \|while \|loop {" tests/ src/
```
Inspect only for unbounded iteration, nondeterministic generation, or cases without assertions.

**Failure**: Unbounded/random generation without reproducibility, or generated cases with weak assertions = **LETHAL**.

---

## Rule 3 — Know What You Own

**General**: Every resource opened must be closed, including on the error path.

**Applied to tests**:
- `tempfile::tempdir()` without cleanup at test end = resource leak.
- Database connections, file handles, sockets opened in test setup must be
  explicitly dropped or cleaned up.
- `NamedTempFile` is self-cleaning — preferred over manual cleanup.
- Any test that creates side effects (files, DB rows, network state) without
  cleanup pollutes other tests and causes ordering failures.

**Audit**:
```bash
# Look for manual file creation without guaranteed cleanup
grep -rn "File::create\|fs::write\|OpenOptions" tests/
grep -rn "tempdir\|tempfile\|TempDir" tests/
```

**Failure**: Resource opened without cleanup in test code = **MAJOR**.

---

## Rule 4 — One Behavior, Exact Evidence

**General**: Each function does exactly one thing. ≤ 60 lines.

**Applied to tests**:
- One test should prove one behavior or invariant. Multiple assertions are fine when
  they prove the same observable behavior.
- Long tests are acceptable if the setup and assertions remain readable and precise.
- Test name must describe the one thing being proven.

**Audit**:
```bash
# Manual review: does a failure identify the exact behavior/value that broke?
```

**Failure**: Test asserting unrelated behaviors with ambiguous failure evidence = **MINOR**.

---

## Rule 5 — State Your Assumptions

**General**: Every function's preconditions must be explicit and checkable.

**Applied to tests**:
- Every test must have an explicit `// Given` block that states preconditions.
  Not implied. Written out. A reader should know the system state without
  tracing setup helpers.
- DAMP: Descriptive And Meaningful Phrases. Shared helpers are fine if the call site
  still makes the preconditions obvious.
- Fixtures built with the builder pattern are acceptable IF the builder call
  makes the intent clear at the test site.

**Audit**: Manual review — scan for tests with `setup()` calls where the
preconditions are not obvious from the test body itself.

**Failure**: Test whose Given state is not inferable at the assertion site = **MINOR**.

---

## Rule 6 — Never Swallow Errors

**General**: Every failure path must be handled, logged, or propagated.

**Applied to tests**:
- `let _ = result;` in a test = silent discard of the result = the test proves nothing.
- `.ok()` called on a `Result` in test code without an assertion on the value = **LETHAL**.
- `unwrap()` in test setup (not the assertion itself) is acceptable for known-good
  setup data. `unwrap()` where the unwrap IS the assertion = **LETHAL** (use `assert_eq!`).
- Any test that calls a fallible function and never checks the return = hollow test.

**Audit**:
```bash
grep -rn "let _ = \|\.ok();" tests/ src/
grep -rn "\.unwrap()" tests/ src/ | grep -v "// setup\|// Given"
```

**Failure**: `let _ = result` or `.ok()` discard in test assertion = **LETHAL**.
`unwrap()` as the assertion = **LETHAL** (replace with `assert_eq!(result.unwrap(), expected)`
— which is also banned: use `assert_eq!(result, Ok(expected))`).

---

## Rule 7 — Narrow Your State

**General**: Data should live as close to its use as possible. No global state.

**Applied to tests**:
- Local mutable state inside one test is fine. Shared mutable state between tests is not.
- `static mut` in test code = **LETHAL**. Non-deterministic test ordering.
- `lazy_static!` or `once_cell::sync::Lazy` with mutable interior (`Mutex`, `RwLock`)
  in test code = **LETHAL** unless explicitly designed as a one-time init with no
  subsequent mutation.
- Test databases must be per-test, not shared across the test suite.

**Audit**:
```bash
grep -rn "static mut\|lazy_static!\|Lazy::new" tests/ src/
grep -rn "Mutex\|RwLock" tests/ src/ | grep "static\|Lazy"
```

**Failure**: Shared mutable state that can affect another test = **LETHAL**.

---

## Rule 8 — Surface Your Side Effects

**General**: I/O, mutations, and network calls must be obvious at the call site.

**Applied to tests**:
- A test helper named `setup()` that secretly creates files, network connections,
  or DB rows is the most dangerous kind of test helper. Name it `create_test_database()`,
  `write_fixture_files()` — make the side effect visible in the name.
- Test helpers that return values (pure builders) are fine. Test helpers that
  have side effects must be named to advertise them.
- If a test touches the filesystem, it should be obvious from reading the test body.

**Audit**: Manual — read every helper function called from test bodies. Does the
name advertise the side effect?

**Failure**: Side-effectful test helper that hides setup or cleanup obligations = **MINOR**.

---

## Rule 9 — Preserve Failure Locality

**General**: Every failure must identify the exact case and expected value.

**Applied to tests**:
- Helper abstraction depth is not a failure by itself.
- Table/fixture/helper code must preserve case names, input values, and expected values
  in failure output.
- If a helper hides the expected value or turns multiple cases into one vague failure,
  the test evidence is weak.

**Audit**: Manual — trace a hypothetical failure. Does the output tell you the broken behavior?

**Failure**: Helper/fixture indirection that obscures the failing behavior or expected value = **MAJOR**.

---

## Rule 10 — Tests Compile and Execute

**General**: Tests must compile and run; style warnings in test code are not a rejection gate.

**Applied to tests**:
- `cargo test --all-features --no-run` must pass.
- `cargo nextest run` must pass deterministically.
- Strict clippy applies to production/source targets. Do not fail a suite for test
  implementation style warnings unless they also weaken evidence or determinism.

**Audit**:
```bash
cargo test --all-features --no-run 2>&1
```

**Failure**: Test compile failure or execution failure = **LETHAL**.
