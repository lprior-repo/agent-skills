# Holzmann's 12 Rules — Applied to Tests

Gerard Holzmann's original 10 rules for mission-critical C code, extended to 12 for
AI-assisted development, mapped concretely to test code. Each rule has a specific
failure condition and a specific grep/audit command.

Rules 11 (read every line) and 12 (tests first) are enforced by the go-skill pipeline
order, not by the test-reviewer. They are omitted here.

---

## Rule 1 — Keep it Linear

**General**: No deep nesting. Code reads top-to-bottom.

**Applied to tests**:
- Test body has one clear Given → When → Then flow. No nested conditionals.
- Test helper chains ≤ 1 level deep. `setup()` calling `create_fixture()` calling
  `init_state()` = three levels = unreadable on failure.
- Early-exit chains (`if x { return; }` patterns) inside tests = hidden branching = MINOR.

**Audit**:
```bash
# Nesting depth heuristic — flag tests with >2 indent levels
grep -n "        if \|        match " tests/ src/
```

**Failure**: Nested conditional inside test body = **MINOR**.
More than 1 layer of test helper abstraction = **MINOR**.

---

## Rule 2 — Bound Every Loop

**General**: Every iteration needs an explicit maximum.

**Applied to tests**:
- No loops in test bodies. Period. A test with a loop is a test with hidden logic.
  It is no longer a straight-line proof. It is a program. Programs have bugs.
- If you need to test N items, use `rstest` cartesian product or a proptest invariant.
  Not a loop.

**Audit**:
```bash
grep -rn "for .* in \|while \|loop {" tests/ src/
```
Filter to `#[test]` function bodies only.

**Failure**: Any loop in a test body = **LETHAL**.

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

## Rule 4 — One Function, One Job

**General**: Each function does exactly one thing. ≤ 60 lines.

**Applied to tests**:
- One test = one logical assertion. One behavior proven. One failure tells you
  exactly one thing about what broke.
- Test body ≤ 20 lines. If longer: the Given setup is too complex (extract a
  builder), or the test is testing multiple behaviors (split it).
- Test name must describe the one thing being proven.

**Audit**:
```bash
# Find long test functions (approximate — counts lines between fn and closing brace)
awk '/fn .*\(\)/{count=0; name=$0} /#\[test\]/{in_test=1} in_test{count++} /^    \}$/{if(in_test && count>20) print NR": "count" lines: "name; in_test=0}' tests/*.rs
```

**Failure**: Test body > 20 lines = **MINOR**.
Test asserting multiple independent behaviors = **MINOR**.

---

## Rule 5 — State Your Assumptions

**General**: Every function's preconditions must be explicit and checkable.

**Applied to tests**:
- Every test must have an explicit `// Given` block that states preconditions.
  Not implied. Written out. A reader should know the system state without
  tracing setup helpers.
- DAMP: Descriptive And Meaningful Phrases. Each test is self-contained.
  Copy the relevant setup inline rather than hiding it in a shared fixture
  that requires cross-referencing.
- Fixtures built with the builder pattern are acceptable IF the builder call
  makes the intent clear at the test site.

**Audit**: Manual review — scan for tests with `setup()` calls where the
preconditions are not obvious from the test body itself.

**Failure**: Test whose Given state requires reading another file to understand = **MINOR**.

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
- Each test creates its own state from scratch. No shared mutable state between tests.
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

**Failure**: Shared mutable state in test code = **LETHAL**.

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

**Failure**: Side-effectful test helper with an innocent name = **MINOR**.

---

## Rule 9 — One Layer of Magic

**General**: Every layer of indirection makes tracing harder.

**Applied to tests**:
- Max 1 layer of test helper abstraction. Test calls helper. Helper does thing.
  Helper does not call another helper.
- Deep fixture chains (`test → setup → create → init → configure`) = when the
  test fails you spend 10 minutes tracing setup. That is not a test — it is a mystery.
- `rstest` fixtures: 1 level deep. Fixtures that depend on other fixtures = 2 levels =
  flag.

**Audit**: Manual — trace the call depth from test body to terminal side effect.

**Failure**: Helper abstraction depth > 2 = **MINOR**.

---

## Rule 10 — Warnings Are Errors

**General**: Zero compiler warnings from day one.

**Applied to tests**:
- `cargo clippy --tests --all-features -- -D warnings` must produce zero output.
- Unused test variables, dead test code, deprecated test APIs = warnings = failures.
- Clippy catches real test bugs: `assert_eq!(a, b)` vs `assert_eq!(b, a)` argument
  order produces a clippy hint. Treat it as a defect.

**Audit**:
```bash
cargo clippy --tests --all-features -- -D warnings 2>&1
```

**Failure**: Any clippy warning in test code = **LETHAL**.
