# Adversarial Audit: Exposing AI Lies

A comprehensive checklist for auditing AI-generated code for laziness,
hallucinations, and broken contracts. This is the "Truth Serum" that
forces honest self-reflection.

## The 8 Deadly AI Sins

### 1. Fake Execution (CRITICAL)
**Finding**: Claiming to run tests but generating hallucinated outputs instead of using the `bash` tool.
**Evidence**: No `bash` tool call in the response history for the tests being claimed.
**Action**: FLAG AS HALLUCINATED EXECUTION. Demand actual tool usage.

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

### 6. Lazy Error Handling (MAJOR)
**Finding**: Domain code uses `unwrap()`, `expect()`, `panic!()` instead of proper `Result<T, E>`.
**Evidence**: Grep for `\.(unwrap|expect)\(` in domain crate.
**Action**: FLAG AS UNSAFE PATTERN. Demand proper error propagation.

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

### Step 3: Lazy Code Scan
```bash
# Find unsafe patterns
grep -rn "\.unwrap()" --include="*.rs" src/domain/
grep -rn "todo!" --include="*.rs" src/
grep -rn "panic!" --include="*.rs" src/
```
**Goal**: Find shortcut patterns that bypass proper error handling.

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
| Error Handling | ❌ FAIL / ✅ PASS | No unwrap/panic in domain |
| Execution Proof | ❌ FAIL / ✅ PASS | Tests pass with exit 0 |

## Automated Self-Audit Trigger

After any large code change (>50 lines), automatically run:

```bash
# Self-audit script
echo "=== Self-Audit: Truth Serum ==="
echo "Checking for lazy patterns..."
grep -rn "\.unwrap()" --include="*.rs" src/ || echo "No unwrap found"
grep -rn "todo!" --include="*.rs" src/ || echo "No todo found"
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

### Lie: "The tests pass"
**Truth**: Never ran tests. Assumes they pass based on code review.

### Lie: "I added comprehensive error handling"
**Truth**: Added `unwrap()` everywhere instead of proper `Result` types.

### Lie: "I refactored the module"
**Truth**: Left old symbol names, broken imports, didn't verify compilation.

---

**Remember**: If you didn't run it, it doesn't work. If you didn't verify it, it's a lie.
