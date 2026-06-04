# Adversarial Audit: Exposing AI Lies

A comprehensive checklist for auditing AI-generated code for laziness,
hallucinations, and broken contracts. This is the "Truth Serum" that
forces honest self-reflection.

## The 9 Deadly AI Sins

### 1. Fake Execution (CRITICAL)
**Finding**: Claiming to run tests but generating hallucinated outputs instead of using the `bash` tool.
**Evidence**: No `bash` tool call in the response history for the tests being claimed.
**Action**: FLAG AS HALLUCINATED EXECUTION. Demand actual tool usage.

### 1b. Delegated Proof Laundering (CRITICAL)
**Finding**: A subagent, reviewer, or external summary is treated as Truth Serum execution evidence.
**Evidence**: Final report cites a subagent result without a matching command run in the active execution context, observed stdout/stderr, and exit status.
**Action**: FLAG AS UNVERIFIED DELEGATED PROOF. Rerun the command directly or mark the result `UNVERIFIED` with a blocker.

### 2. Ellipsis Laziness (CRITICAL)
**Finding**: Code contains `...`, `// TODO`, `// rest of code here`, or incomplete implementations.
**Evidence**: Grep for `\.\.\.` or `// .*here` patterns.
**Action**: FLAG AS CRITICAL LAZINESS. Demand complete implementation.

### 2. Hallucinated Paths (CRITICAL)
**Finding**: File paths mentioned in response don't exist.
**Evidence**: Run `ls` on every claimed file path.
**Action**: FLAG AS HALLUCINATION. Demand verification before proceeding.

### 3. Test Deletion (CRITICAL)
**Finding**: Tests deleted or commented out without a bead filing the defect.
**Evidence**: `git diff` showing removed test code.
**Action**: FLAG AS DESTRUCTIVE ACTION. Demand replacement tests or revert.

### 4. Contract Ignorance (CRITICAL)
**Finding**: Spec requires `Must` X, but code has `todo!()`, `None`, or unimplemented.
**Evidence**: Compare `contract-spec.md` invariants with actual code.
**Action**: FLAG AS IGNORED CONTRACT. Demand parity with specification.

### 5. Scope Creep (MAJOR)
**Finding**: Unrelated files modified (e.g., .env, unrelated modules).
**Evidence**: `git status` showing unexpected file changes.
**Action**: FLAG AS COLLATERAL DAMAGE. Demand focused, minimal changes.

### 6. Runtime Panic Surface (CRITICAL)
**Finding**: Production Rust contains `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `unreachable!`, production `assert!` macros, unchecked indexing/slicing, unsafe code, ignored fallible results, or arithmetic side-effect surprises.
**Evidence**: Strict clippy denials plus static scan for production panic macros.
**Action**: FLAG AS RUNTIME PANIC SURFACE. Demand typed errors, checked access, explicit bounds, or proof the match is non-production.

### 7. No Validation (MAJOR)
**Finding**: Claimed "it works" without running any tests or commands.
**Evidence**: No bash output showing test execution.
**Action**: FLAG AS UNVERIFIED CLAIM. Demand actual execution with evidence.

## Audit Workflow

### Step 1: Git Archaeology
```bash
git status
git diff --staged
git diff HEAD
```
**Goal**: See exactly what changed. Any surprises?

### Step 2: Path Verification
```bash
# For every file mentioned in the response
ls -la src/mentioned_file.rs
```
**Goal**: Confirm files actually exist.

### Test Preservation Check
```bash
# Find deleted tests
git diff --name-only | grep -E "test|spec"
git diff --stat | grep -E "deletion"
```
**Goal**: Ensure no tests were silently removed.

### Step 3: Runtime Panic Surface Scan
```bash
# Find production panic and unsafe surfaces
cargo clippy --all-features -- -D warnings -D unsafe_code -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::panic_in_result_fn -D clippy::todo -D clippy::unimplemented -D clippy::dbg_macro -D clippy::indexing_slicing -D clippy::string_slice -D clippy::get_unwrap -D clippy::arithmetic_side_effects -D clippy::as_conversions -D clippy::let_underscore_must_use
cargo test --all-features --no-run
rg -n '(^|[^A-Za-z0-9_])(assert!|assert_eq!|assert_ne!|unreachable!)' --glob '*.rs' --glob '!**/tests/**' --glob '!**/benches/**' --glob '!**/examples/**' --glob '!build.rs'
```
**Goal**: Find every shortcut that can panic or bypass typed error handling in production.

### Step 4: Contract Parity
```bash
# If contract-spec.md exists
grep "Must" contract-spec.md
# Compare each Must with actual implementation
```
**Goal**: Verify every requirement is implemented, not just mentioned.

### Step 5: Execution Proof
```bash
# Run the actual code/tests
cargo test 2>&1
cargo build 2>&1
cargo clippy 2>&1
```
**Goal**: Prove it actually works. No "should" or "probably".

### Step 6: Evidence Ownership Check
```text
For every claimed PASS, verify:
- command was run in the active execution context
- stdout/stderr were observed directly
- exit status is known or the blocker is explicit
- subagent-only findings are labeled UNVERIFIED
```
**Goal**: Prevent delegated review from being laundered into proof.

## The Truth Report Template

After auditing, output this exact format:

| Check | Result | Evidence |
|-------|--------|----------|
| Fake Execution | ❌ FAIL / ✅ PASS | Bash tool was actually invoked |
| Ellipsis Laziness | ❌ FAIL / ✅ PASS | Found `...` at line X |
| Path Integrity | ❌ FAIL / ✅ PASS | `ls` confirmed file exists |
| Test Preservation | ❌ FAIL / ✅ PASS | No tests deleted |
| Contract Parity | ❌ FAIL / ✅ PASS | All `Must` requirements met |
| Scope Integrity | ❌ FAIL / ✅ PASS | Only intended files changed |
| Runtime Panic Surface | ❌ FAIL / ✅ PASS | No production unwrap/expect/panic/assert/unreachable/unchecked indexing/unsafe |
| Execution Proof | ❌ FAIL / ✅ PASS | Tests pass with exit 0 |
| Delegated Proof | ❌ FAIL / ✅ PASS | Subagent claims rerun directly or labeled UNVERIFIED |

## Automated Self-Audit Trigger

After any large code change (>50 lines), automatically run:

```bash
# Self-audit script
echo "=== Self-Audit: Truth Serum ==="
echo "Checking for runtime panic surface..."
cargo clippy --all-features -- -D warnings -D unsafe_code -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::panic_in_result_fn -D clippy::todo -D clippy::unimplemented -D clippy::dbg_macro -D clippy::indexing_slicing -D clippy::string_slice -D clippy::get_unwrap -D clippy::arithmetic_side_effects -D clippy::as_conversions -D clippy::let_underscore_must_use
cargo test --all-features --no-run
if rg -n '(^|[^A-Za-z0-9_])(assert!|assert_eq!|assert_ne!|unreachable!)' --glob '*.rs' --glob '!**/tests/**' --glob '!**/benches/**' --glob '!**/examples/**' --glob '!build.rs'; then exit 1; else true; fi
echo "Checking test integrity..."
git diff --name-only | grep test || echo "No test changes"
echo "Verifying scope..."
git diff --name-only
echo "=== Audit Complete ==="
```

## Coverage Thresholds (For Rust Projects)

- **Domain code**: 90%+ line coverage, zero surviving mutants
- **Application code**: 80%+ line coverage
- **Infrastructure code**: 60%+ line coverage

## Examples of AI Lies vs Truth

### Lie: "I fixed the bug"
**Truth**: Only changed 2 lines, didn't run tests, didn't verify fix.

### Lie: "This is idiomatic Rust"
**Truth**: Uses `unwrap()` in domain code, ignores error handling conventions.

### Lie: "There are no panics"
**Truth**: Clippy only checked `panic!`; production `assert!`, `unreachable!`, indexing, and `.expect()` were never scanned.

### Lie: "The tests pass"
**Truth**: Never ran tests. Assumes they pass based on code review.

### Lie: "The reviewer subagent verified this"
**Truth**: A subagent finding is review input, not execution proof. Rerun the command directly or mark it UNVERIFIED.

### Lie: "I added comprehensive error handling"
**Truth**: Added `unwrap()` everywhere instead of proper `Result` types.

### Lie: "I refactored the module"
**Truth**: Left old symbol names, broken imports, didn't verify compilation.

---

**Remember**: If you didn't run it, it doesn't work. If you didn't verify it, it's a lie.
