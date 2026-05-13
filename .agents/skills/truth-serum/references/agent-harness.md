# Agent Harness Setup

Templates for CLAUDE.md, AGENTS.md, stop hooks, and the verification
script. These tie the 10 verification layers into a deterministic cage.

## Table of contents

1. [CLAUDE.md template](#claudemd-template)
2. [AGENTS.md symlink](#agentsmd-symlink)
3. [Stop hook config](#stop-hook-config)
4. [verify.sh script](#verifysh-script)
5. [Anti-pattern prevention](#anti-pattern-prevention)

---

## CLAUDE.md template

Keep this under 150 lines. Document what agents get wrong, not what they
get right. The codebase teaches patterns; this file teaches deviations.

```markdown
# [Project Name] — CLAUDE.md

## Architecture

DDD layers: domain/ → application/ → infrastructure/ → presentation/
Dependency rule: inner layers NEVER depend on outer layers.
See doc/adr/0002-layer-boundaries.md for rationale.

## Commands

- Build: `cargo build --workspace`
- Test: `cargo nextest run`
- Lint: `cargo clippy --all-features -- -D warnings`
- Full verify: `./scripts/verify.sh`

## Conventions

- Error handling: `thiserror` in domain, `anyhow` in application/infra.
- All domain types use validated constructors (parse, don't validate).
- State machines use the typestate pattern with `PhantomData`.
- See docs/conventions.md for full patterns.
- See docs/testing.md for test patterns and snapshot workflow.

## Gotchas

- NEVER use runtime panic surfaces in production code: `.unwrap()`, `.expect()`, `panic!`, `todo!`, `unimplemented!`, `unreachable!`, production `assert!` macros, unchecked indexing/slicing, or unsafe code. Use typed errors and checked access.
- NEVER modify test files to make tests pass. Fix the code instead.
- NEVER run `cargo insta accept` without reviewing diffs.
- Domain crate must have zero I/O dependencies.
- PropTest regressions in `proptest-regressions/` are committed to git.

## Verification

All changes must pass `./scripts/verify.sh` before committing.
The stop hook enforces this automatically.
```

### Hierarchical CLAUDE.md files

Place domain-specific instructions in subdirectories:

```
crates/domain/CLAUDE.md    → "All types here use typestate. See Order<S>."
crates/infra/CLAUDE.md     → "DB migrations go in migrations/. Never raw SQL in code."
```

---

## AGENTS.md symlink

AGENTS.md is the cross-tool open standard (OpenAI Codex, Cursor, Copilot).
CLAUDE.md is Claude Code specific. Symlink both:

```bash
ln -s CLAUDE.md AGENTS.md
```

---

## Stop hook config

The stop hook runs `verify.sh` every time the agent tries to finish.
Exit code 2 means "you're not done — here's what's wrong" and feeds
stderr back into the agent's context.

### .claude/settings.json

```json
{
  "hooks": {
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "./scripts/verify.sh"
          }
        ]
      }
    ],
    "PreToolUse": [
      {
        "matcher": "Bash(git commit)",
        "hooks": [
          {
            "type": "command",
            "command": "test -f /tmp/verify-pass"
          }
        ]
      }
    ]
  }
}
```

---

## verify.sh script

Ordered from fast/cheap to slow/expensive. Fails fast on the first error.

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "=== Step 1/9: Format check ==="
cargo fmt --check

echo "=== Step 2/10: Source clippy + test compile + zero runtime panic surface ==="
cargo clippy --all-features -- -D warnings -D unsafe_code -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::panic_in_result_fn -D clippy::todo -D clippy::unimplemented -D clippy::dbg_macro -D clippy::indexing_slicing -D clippy::string_slice -D clippy::get_unwrap -D clippy::arithmetic_side_effects -D clippy::as_conversions -D clippy::let_underscore_must_use
cargo test --all-features --no-run

echo "=== Step 3/10: Production panic macro scan ==="
if rg -n '(^|[^A-Za-z0-9_])(assert!|assert_eq!|assert_ne!|unreachable!)' --glob '*.rs' --glob '!**/tests/**' --glob '!**/benches/**' --glob '!**/examples/**' --glob '!build.rs'; then
  echo "FAIL: production panic macro found. Return typed errors or prove the match is non-production."
  exit 1
fi

echo "=== Step 4/10: Custom lints (dylint) ==="
cargo dylint --all --workspace

echo "=== Step 5/10: Dependency bans (cargo-deny) ==="
cargo deny check bans

echo "=== Step 6/10: Crate boundary check ==="
./scripts/check-boundaries.sh

echo "=== Step 7/10: Tests (nextest + PropTest + insta snapshots) ==="
INSTA_UPDATE=no cargo nextest run

echo "=== Step 8/10: Semver check ==="
cargo semver-checks --package domain --baseline-rev origin/main

echo "=== Step 9/10: Coverage ==="
cargo llvm-cov --package domain --fail-under-lines 90
cargo llvm-cov --package application --fail-under-lines 80
cargo llvm-cov --package infra --fail-under-lines 60

echo "=== Step 10/10: Mutation testing (changed files only) ==="
if [ -f pr.diff ]; then
  cargo mutants --in-diff pr.diff -vV --in-place
else
  echo "No pr.diff found, skipping incremental mutation testing."
  echo "Run full mutation testing with: cargo mutants"
fi

# Signal success for the PreToolUse gate
touch /tmp/verify-pass
echo "All checks passed."
```

For Kani proofs, add a targeted step between 7 and 8 if the project
uses Kani:

```bash
echo "=== Step 7b/10: Kani proofs ==="
cargo kani --harness "verify_*" --output-format terse
```

Run the full Kani suite on a nightly schedule, not on every PR.

---

## Anti-pattern prevention

### Block test rewriting

In the stop hook or CI, restrict which files the agent can modify:

```bash
# In verify.sh or as a separate hook
CHANGED_TEST_FILES=$(git diff --name-only HEAD | grep -E "tests/.*\.rs$" || true)
if [ -n "$CHANGED_TEST_FILES" ]; then
  echo "WARNING: Test files were modified:"
  echo "$CHANGED_TEST_FILES"
  echo "Review carefully — agents sometimes rewrite tests to pass."
fi
```

For strict enforcement, fail if test files are in the diff:

```bash
if git diff --name-only HEAD | grep -qE "^crates/domain/tests/"; then
  echo "FAIL: Domain test files must not be modified by agents."
  exit 1
fi
```

### Context window management

Aggressively truncate error logs before feeding back to the agent.
Large compiler error output causes context exhaustion:

```bash
# Truncate stderr to last 50 lines
cargo nextest run 2>&1 | tail -50
```

### Circuit breaker

Add `--max-iterations` to the retry loop. Consider an LLM-as-judge
to detect lack of progress after 3+ retries on the same error.
