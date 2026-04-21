---
name: landing-skill
description: "Session completion enforcer - runs quality gates, syncs code, pushes to remote, and hands off cleanly."
---

# Landing the Plane Skill

## Overview

The "Landing the Plane" workflow ensures that all work is properly committed, tested, synced, and pushed to remote before ending a session. This skill automates the complete session shutdown process with mandatory quality gates.

**The critical principle**: every failure is a signal. A failing test is a design smell. A linter warning is a code smell. A dangling branch is a process smell. Either **fix it now** or **file it as a bead** — nothing gets swept under the rug.

## When to Use This Skill

Invoke when:
- **Ending a work session**: "Land the plane before ending"
- **Before going AFK**: "Complete and push all work"
- **After implementing features**: "Finish up and sync"
- **Before switching contexts**: "Land current work and prepare handoff"

## Core Principles

### 1. Main Must Be Clean
Main is the source of truth. It must build, pass all tests, have zero warnings, and zero lint violations at all times. If main is dirty, nothing else matters until it's clean.

### 2. Every Failure Is a Smell
A failing test is not "something to fix later." It's a design or code smell that demands either:
- **Fix it now** — resolve the issue before landing
- **File it as a bead** — create a tracked issue with full context so it gets fixed properly

There is no third option. "I'll remember to fix it" is not acceptable.

### 3. Nothing Local
All completed work MUST be on remote. Local-only work is lost work.

### 4. No Orphans
No open branches without purpose. No worktrees left dangling. No zjj sessions abandoned. No stale stashes hiding work. Everything is either merged, filed as a bead, or explicitly abandoned.

### 5. Clean Handoff
Next session (or teammate) should know exactly where things stand.

### 6. Exponential Backoff on Merge & Push Failures
Any operation that can fail transiently (merge conflict after rebase, push rejection, network blip) MUST use exponential backoff. Tight retry loops against a merge queue or remote are how sessions end up in a terrible state — hammering a conflict that won't self-resolve, or spinning forever on a network blip.

**Backoff Schedule (shared by merge and push):**

| Attempt | Wait Before Retry | Cumulative |
|---------|-------------------|------------|
| 1 | — (immediate first try) | 0s |
| 2 | 2s | 2s |
| 3 | 4s | 6s |
| 4 | 8s | 14s |
| 5 | 16s | 30s |
| 6 | 32s | 62s |
| 7 | 60s (cap) | 122s |

- **Max retries: 6** (7 total attempts). After exhaustion, STOP. Do not keep trying.
- **Cap: 60s** per wait. Never wait longer than 60s between attempts.
- **Before each retry**: re-sync first (`zjj sync` or `jj git fetch`). The conflict or rejection may have been caused by stale state. Re-syncing is cheap and often resolves the issue.
- **On exhaustion**: run `zjj recover --diagnose` first (surfaces orphans, stale locks, DB issues). Then file a bead, preserve workspace state (do NOT `zjj abort`), flag the user. The workspace is evidence.

**Pseudocode — merge path (uses `zjj retry`):**

```bash
# Safety snapshot BEFORE the loop
zjj checkpoint create

WAIT=2
MAX_RETRIES=6
RETRY=0
EXIT_CODE=3

while [ $RETRY -lt $MAX_RETRIES ] && [ $EXIT_CODE -ne 0 ]; do
  # Re-sync before retry — stale state is the #1 cause of repeated failures
  zjj sync
  [ $? -ne 0 ] && break   # sync conflict needs manual resolution — stop loop

  # zjj retry retries the last failed command (the done)
  zjj retry
  EXIT_CODE=$?
  [ $EXIT_CODE -eq 0 ] && break

  RETRY=$((RETRY + 1))
  echo "Attempt $((RETRY + 1))/$((MAX_RETRIES + 1)) — waiting ${WAIT}s before retry..."
  sleep $WAIT
  WAIT=$((WAIT * 2))
  [ $WAIT -gt 60 ] && WAIT=60
done

if [ $EXIT_CODE -ne 0 ]; then
  echo "EXHAUSTED: merge retries failed."
  zjj recover --diagnose   # see what's broken
  echo "Filing bead. Workspace preserved."
  # Rollback to checkpoint if state is corrupt:
  #   zjj rollback --to <checkpoint-id> <session-name>
fi
```

**Pseudocode — push path (uses `jj git fetch` + re-push):**

```bash
WAIT=2
MAX_RETRIES=6
RETRY=0
EXIT_CODE=1

while [ $RETRY -lt $MAX_RETRIES ] && [ $EXIT_CODE -ne 0 ]; do
  jj git fetch                     # re-fetch — non-fast-forward often self-resolves
  jj git push --bookmark main
  EXIT_CODE=$?
  [ $EXIT_CODE -eq 0 ] && break

  RETRY=$((RETRY + 1))
  echo "Push attempt $((RETRY + 1))/$((MAX_RETRIES + 1)) — waiting ${WAIT}s..."
  sleep $WAIT
  WAIT=$((WAIT * 2))
  [ $WAIT -gt 60 ] && WAIT=60
done

if [ $EXIT_CODE -ne 0 ]; then
  echo "EXHAUSTED: push retries failed. Flag user — permission or branch protection."
fi
```

## Landing Workflow (10 Mandatory Steps)

### Step 1: Audit Open Branches, Worktrees, and Sessions

**Objective:** Identify everything that's open. Nothing hides.

**Actions:**

```bash
# === ZJJ WORKSPACES (CHECK FIRST) ===
# List all zjj workspaces (isolated work sessions)
zjj list 2>/dev/null
zjj status 2>/dev/null  # Current workspace details

# Show workspace age and activity
zjj list --verbose 2>/dev/null

# For detailed status of each workspace:
for ws in $(zjj list 2>/dev/null | awk '{print $1}'); do
  echo "=== Workspace: $ws ==="
  zjj status -n "$ws" 2>/dev/null
done

# === JJ WORKSPACES (if using jj directly) ===
jj workspace list 2>/dev/null

# === GIT BRANCHES ===
# List all local branches
git branch

# List branches NOT merged into main
git branch --no-merged main

# List remote branches
git branch -r

# === GIT WORKTREES ===
git worktree list

# Check for worktrees not tracked by zjj (orphans)
git worktree list | grep -v "$(git rev-parse --show-toplevel)" | grep -v "zjj-"

# === STASHES ===
git stash list
```

**For EACH open item, decide:**
```
Open zjj workspace?
├─ Work is COMPLETE → zjj done -m "..." (Step 4 - merges to main)
├─ Work is INCOMPLETE but VALUABLE → File bead (Step 2), then decide:
│   ├─ Continuing next session → Keep workspace, document in handoff
│   └─ Not continuing → zjj abort -w <name> (proper discard, updates bead)
├─ Work is ABANDONED/STALE → zjj abort -w <name>
└─ Work is IN-PROGRESS → Document state in bead, keep workspace

Open branch/worktree (not zjj-managed)?
├─ Work is COMPLETE → Merge to main (Step 4)
├─ Work is INCOMPLETE but VALUABLE → File bead (Step 2), then decide:
│   ├─ Can merge partial work safely → Merge to main
│   └─ Cannot merge safely → Abandon with bead reference
├─ Work is ABANDONED/STALE → Remove it:
│   ├─ git branch -d <branch>
│   └─ git worktree remove <path>
└─ Work is IN-PROGRESS → Document state in bead, keep open (EXCEPTION)

Open stash?
├─ Recent work → git stash pop, commit properly
├─ Valuable but not applying → File bead with context, git stash drop
└─ Stale/forgotten → Review, file bead if valuable, git stash drop
```

**FLAG THE USER if:**
- Any zjj workspace has been open > 7 days without commits
- Any branch has been open > 7 days without commits (and not zjj-managed)
- Any worktree exists that isn't tracked by zjj (orphan worktree)
- Any zjj workspace is in `failed` or `stale` state
- Any stash is > 3 days old
- More than 3 branches/workspaces are open simultaneously

**PRIORITY ORDER FOR AUDIT:**
1. **zjj workspaces** - Most important, these are your active isolation units
2. **git branches** - Might be legacy work not yet moved to zjj
3. **git worktrees** - Should be managed by zjj, flag if orphaned
4. **stashes** - Danger zone, often forgotten work

**Required Outputs:**
- [ ] All zjj workspaces inventoried with status (complete/in-progress/abandon)
- [ ] All git branches inventoried with merge decision
- [ ] All worktrees accounted for (zjj-managed or orphaned)
- [ ] All stashes reviewed and decided
- [ ] No orphans remain without a decision
- [ ] User flagged for any anomalies

---

### Step 2: File Beads for Remaining Work

**Objective:** Every incomplete item gets a tracked bead using `br create`. Nothing lives only in memory. Never use GitHub Issues — beads are the single source of truth.

**Actions:**
```bash
# Search for TODOs and FIXMEs in code
grep -r "TODO\|FIXME\|HACK\|XXX\|WARN" src/ --exclude-dir=node_modules --exclude-dir=target 2>/dev/null

# Review current state
git status
git diff
git diff --staged

# Create beads (see Bead Template below for description format)
br create "[smell-type] Brief description" \
  --type task \
  --priority 2 \
  --description "$(cat <<'EOF'
<full bead description — see template below>
EOF
)" \
  --acceptance "$(cat <<'EOF'
<acceptance criteria — see template below>
EOF
)" \
  --labels "smell:code,severity:important"
```

**What gets a bead:**
- Any TODO/FIXME/HACK/XXX in code you touched this session
- Any failing test you couldn't fix in this session
- Any linter warning you couldn't resolve
- Any compiler warning
- Any incomplete branch that's being abandoned
- Any design concern surfaced during work
- Any technical debt deliberately taken on

**Quick capture** (for rapid filing during landing — enrich later):
```bash
BEAD_ID=$(br q "[code] Fix clippy warning in auth module" --type task --priority 2 --labels "smell:code")
br update "$BEAD_ID" --description "..." --acceptance "..."
```

---

### Bead Description Template (EARS + BDD)

Every bead MUST follow this structured format. Use `br create` with `--description` and `--acceptance` flags.

**Description** (`--description`):

```markdown
## Context
What was being done when this surfaced.
Current state: what files, branches, or partial work exist.

## Smell Classification
- **Type**: design | code | test | process | debt
- **Severity**: blocking | important | minor
- **Gate Failed**: test | lint | format | build | type | warning | N/A

## Dependencies
- **Blocks**: [bead IDs this blocks, if any]
- **Blocked By**: [bead IDs that must resolve first, if any]
- **Related**: [bead IDs with relates-to relationship]

## Requirements (EARS)

### Invariants (Ubiquitous — always true, no keyword)
The <system> shall <behavior that must always hold>.
Example: "The CLI shall exit with non-zero status on any error."

### State-Driven (While)
While <precondition>, the <system> shall <expected behavior>.
Example: "While the workspace is active, zjj status shall show the session."

### Event-Driven (When)
When <trigger event>, the <system> shall <expected response>.
Example: "When the user runs `br lint`, all open beads shall be checked for missing sections."

### Optional Feature (Where)
Where <feature is present>, the <system> shall <behavior>.
Example: "Where --json flag is provided, output shall be valid JSON."

### Unwanted Behavior (If/Then)
If <unwanted condition>, then the <system> shall <recovery behavior>.
Example: "If the database is locked, then br shall retry with backoff up to 30s."

## Variants
- **Happy Path**: The expected normal-use scenario
- **Alternate Paths**: Valid alternative flows
- **Error Paths**: Expected failure modes and recovery

## Design Notes
Any architectural context, trade-offs, or constraints.
```

**Acceptance Criteria** (`--acceptance`):

```markdown
## High-Level Acceptance Criteria
1. [Criterion 1 — what must be true for this bead to be closed]
2. [Criterion 2]
3. [Criterion 3]

## Acceptance Tests (BDD — Outer Layer)

### Scenario: <Happy path scenario name>
  Given <precondition>
  When <action>
  Then <expected outcome>
  And <additional verification>

### Scenario: <Error path scenario name>
  Given <precondition>
  When <error-triggering action>
  Then <expected error handling>
  And <state remains clean>

### Scenario: <Edge case scenario name>
  Given <boundary condition>
  When <action at the boundary>
  Then <expected behavior at the edge>

## Verification
- [ ] All acceptance scenarios pass
- [ ] No new warnings introduced
- [ ] `br lint` passes on this bead
```

**After creating, wire dependencies:**
```bash
# If this bead blocks another
br dep $BEAD_ID --blocks $OTHER_ID

# If this bead is blocked by another
br dep add $BEAD_ID $BLOCKER_ID

# If beads are related
br dep relate $BEAD_ID $RELATED_ID
```

**After creating, validate:**
```bash
# Lint the bead to ensure required sections are present
br lint $BEAD_ID
```

---

### Bead Examples

**Example 1: Failing test (code smell)**
```bash
br create "[test] Auth module password validation test fails on empty string" \
  --type bug \
  --priority 1 \
  --labels "smell:test,severity:important,gate:test" \
  --description "$(cat <<'EOF'
## Context
Surfaced during landing quality gates. Test `test_validate_password_empty`
fails with assertion error. Pre-existing failure, not introduced this session.
File: tests/auth_test.gleam:47

## Smell Classification
- **Type**: test
- **Severity**: important
- **Gate Failed**: test

## Dependencies
- **Blocks**: none
- **Blocked By**: none

## Requirements (EARS)

### Event-Driven (When)
When the user provides an empty password, the auth module shall return
Error(EmptyPassword) with a user-facing message.

### Unwanted Behavior (If/Then)
If the password validation receives a null/empty input, then the system
shall reject it without panicking or returning Ok.

## Variants
- **Happy Path**: Non-empty password passes validation
- **Error Path**: Empty string returns Error(EmptyPassword)
- **Edge Case**: Whitespace-only string treated as empty
EOF
)" \
  --acceptance "$(cat <<'EOF'
## High-Level Acceptance Criteria
1. Empty password returns Error(EmptyPassword)
2. Whitespace-only password returns Error(EmptyPassword)
3. Valid password passes validation

## Acceptance Tests (BDD)

### Scenario: Empty password rejected
  Given a user registration form
  When the user submits an empty password
  Then the system returns Error(EmptyPassword)
  And the error message says "Password cannot be empty"

### Scenario: Whitespace-only password rejected
  Given a user registration form
  When the user submits "   " as password
  Then the system returns Error(EmptyPassword)

### Scenario: Valid password accepted
  Given a user registration form
  When the user submits "correcthorsebatterystaple"
  Then the system returns Ok(ValidatedPassword)

## Verification
- [ ] All three scenarios pass in test suite
- [ ] No new warnings introduced
- [ ] `br lint` passes
EOF
)"
```

**Example 2: Linter warning (code smell)**
```bash
br create "[code] Clippy warns about unnecessary clone in db module" \
  --type task \
  --priority 3 \
  --labels "smell:code,severity:minor,gate:lint" \
  --description "$(cat <<'EOF'
## Context
cargo clippy -- -D warnings fails on src/db/connection.rs:142.
Warning: unnecessary `.clone()` on a value that implements Copy.

## Smell Classification
- **Type**: code
- **Severity**: minor
- **Gate Failed**: lint

## Requirements (EARS)

### Invariant
The codebase shall compile with zero clippy warnings under -D warnings.

### Event-Driven (When)
When `cargo clippy -- -D warnings` is run, the build shall succeed with
exit code 0.

## Variants
- **Happy Path**: Remove .clone(), use Copy semantics
- **Alternate**: If Clone is intentional for future-proofing, add #[allow(clippy::clone_on_copy)] with comment
EOF
)" \
  --acceptance "$(cat <<'EOF'
## High-Level Acceptance Criteria
1. `cargo clippy -- -D warnings` passes with zero warnings
2. No behavior change from removing the clone

## Acceptance Tests (BDD)

### Scenario: Clippy passes clean
  Given the codebase at HEAD
  When `cargo clippy -- -D warnings` is run
  Then exit code is 0
  And stdout contains no "warning:" lines

## Verification
- [ ] clippy clean
- [ ] all tests still pass
EOF
)"
```

**Example 3: Orphan branch (process smell)**
```bash
br create "[process] Stale branch 'feature-cache' open 12 days without commits" \
  --type task \
  --priority 3 \
  --labels "smell:process,severity:minor" \
  --description "$(cat <<'EOF'
## Context
Discovered during landing orphan audit. Branch `feature-cache` was last
committed to 12 days ago. Contains partial caching layer implementation.
3 files changed, 142 insertions.

## Smell Classification
- **Type**: process
- **Severity**: minor
- **Gate Failed**: N/A (orphan audit)

## Requirements (EARS)

### Event-Driven (When)
When the caching feature is prioritized, the work on branch `feature-cache`
shall be resumed, rebased onto main, and completed.

### Unwanted Behavior (If/Then)
If the caching feature is deprioritized, then the branch shall be deleted
and this bead updated with the design notes for future reference.

## Variants
- **Happy Path**: Resume work, complete feature, merge to main
- **Alternate**: Cherry-pick useful parts, abandon the rest
- **Abandon**: Delete branch, preserve design notes in this bead
EOF
)" \
  --acceptance "$(cat <<'EOF'
## High-Level Acceptance Criteria
1. Branch is either merged to main or deleted
2. No orphan branch remains without a tracked bead

## Acceptance Tests (BDD)

### Scenario: Branch merged
  Given branch `feature-cache` exists
  When the caching feature is completed
  Then the branch is merged to main
  And the branch is deleted locally and remotely
  And this bead is closed

### Scenario: Branch abandoned
  Given branch `feature-cache` is deprioritized
  When the decision to abandon is made
  Then the branch is deleted
  And design notes are preserved in this bead
  And this bead is closed with reason "abandoned — design notes preserved"

## Verification
- [ ] `git branch --no-merged main` does not show `feature-cache`
- [ ] Bead closed with clear reason
EOF
)"
```

**Required Outputs:**
- [ ] All TODOs/FIXMEs catalogued
- [ ] Beads created using `br create` with full EARS + BDD template
- [ ] Each bead has smell type, severity, and labels
- [ ] Dependencies wired with `br dep`
- [ ] `br lint` passes on all new beads
- [ ] No untracked work left in memory only

---

### Step 3: Run Quality Gates (ZERO TOLERANCE)

**Objective:** Main must be clean. Every failure is a smell that gets either fixed or filed.

**Gate 1: Tests**
```bash
# Detect project type and run tests
make test 2>/dev/null \
  || cargo test 2>/dev/null \
  || gleam test 2>/dev/null \
  || npm test 2>/dev/null \
  || pytest 2>/dev/null \
  || go test ./... 2>/dev/null
```

**Gate 2: Linting (ZERO warnings)**
```bash
# Rust
cargo clippy -- -D warnings 2>/dev/null

# Gleam
gleam check 2>/dev/null

# JavaScript/TypeScript
npm run lint 2>/dev/null

# Python
ruff check . 2>/dev/null

# Go
golangci-lint run 2>/dev/null
```

**Gate 3: Formatting**
```bash
cargo fmt --check 2>/dev/null \
  || gleam format --check src/ test/ 2>/dev/null \
  || npm run format:check 2>/dev/null \
  || ruff format --check . 2>/dev/null \
  || gofmt -l . 2>/dev/null
```

**Gate 4: Build (ZERO warnings)**
```bash
# Rust — deny warnings at build level
RUSTFLAGS="-D warnings" cargo build 2>/dev/null \
  || cargo build 2>/dev/null

# Other
gleam build 2>/dev/null \
  || npm run build 2>/dev/null \
  || make build 2>/dev/null \
  || go build ./... 2>/dev/null
```

**Gate 5: Type Checking**
```bash
npm run typecheck 2>/dev/null \
  || tsc --noEmit 2>/dev/null \
  || mypy . 2>/dev/null
```

**Gate 6: Compiler/Runtime Warnings**
```bash
# Capture and review any warnings from build output
# Rust: cargo build 2>&1 | grep "warning:"
# Gleam: gleam build 2>&1 | grep "Warning"
# TypeScript: tsc 2>&1 | grep "warning"
```

**FAILURE PROTOCOL — Every failure is a smell:**

```
Quality gate failed?
├─ Can I fix it RIGHT NOW (< 5 minutes, obvious fix)?
│   ├─ YES → Fix it. Re-run ALL gates. Commit the fix.
│   └─ NO → File a bead with full context:
│       ├─ Smell Type: test | lint | format | build | type | warning
│       ├─ Gate: which gate failed
│       ├─ Output: exact error output
│       ├─ Severity: blocking | important | minor
│       └─ Context: what caused this, what was being changed
│
├─ Is this a PRE-EXISTING failure (not caused by this session)?
│   ├─ YES → File bead, note "pre-existing", continue landing
│   └─ NO → This session introduced it. MUST fix before landing.
│
└─ Is this BLOCKING the build/tests entirely?
    ├─ YES → CANNOT land. Fix it or revert the breaking change.
    └─ NO → File bead, continue landing with warning in handoff report.
```

**CRITICAL RULES:**
- **ZERO test failures on main** — if tests fail, fix or revert
- **ZERO linter warnings** — `-D warnings` is the standard
- **ZERO compiler warnings** — warnings are future bugs
- **ZERO format violations** — run the formatter, commit the result
- **New failures introduced this session MUST be fixed, not filed**
- **Pre-existing failures get filed as beads but don't block landing**

**Required Outputs:**
- [ ] All tests passing (exit code 0)
- [ ] Zero linting violations
- [ ] Zero compiler/runtime warnings
- [ ] Code properly formatted
- [ ] Build succeeds
- [ ] No type errors
- [ ] Any failures that couldn't be fixed have beads filed

---

### Step 4: Merge to Main (Merge Queue)

**Objective:** All completed work lands on main. Branches close. zjj acts as merge queue.

**ZJJ MERGE QUEUE WORKFLOW (PREFERRED):**

zjj `done` command acts as a merge queue - it syncs workspace changes to main, runs quality gates, and cleans up the workspace automatically.

```bash
# === ZJJ MERGE QUEUE (ONE COMMAND) ===
# From within the zjj workspace
zjj done -m "Brief description of completed work"
# This does:
#   1. Syncs workspace commits to main branch
#   2. Merges into main (rebase or merge based on config)
#   3. Pushes to remote
#   4. Removes the workspace
#   5. Cleans up the Zellij session

# If you want to review before merge:
zjj sync              # Sync changes without merging
zjj diff main         # Review what will be merged
git log main..HEAD    # See commits to be merged
zjj done -m "..."     # Proceed with merge
```

**If `zjj done` exits with code 3 (merge conflict) — use exponential backoff:**

See the **Exponential Backoff Protocol** in Core Principles above for the full schedule (max 6 retries, 2→4→8→16→32→60s waits, capped at 60s). Use `zjj retry` — it retries the exact last failed command. Preview first with `zjj whatif done`.

```bash
# 0. Preview before attempting (more detail than --dry-run)
zjj whatif done

# 1. Safety checkpoint BEFORE the retry loop
zjj checkpoint create

# 2. Backoff retry loop
WAIT=2
MAX_RETRIES=6
RETRY=0
EXIT_CODE=3

while [ $RETRY -lt $MAX_RETRIES ] && [ $EXIT_CODE -ne 0 ]; do
  # Re-sync FIRST — main may have changed, rebase may clear the conflict
  zjj sync
  if [ $? -ne 0 ]; then
    echo "Conflict during sync. Resolve manually in workspace, then retry."
    break   # hand off to user for manual resolution
  fi

  # zjj retry retries the last failed command (the done)
  zjj retry
  EXIT_CODE=$?
  [ $EXIT_CODE -eq 0 ] && break

  RETRY=$((RETRY + 1))
  echo "Merge attempt $((RETRY + 1))/$((MAX_RETRIES + 1)) — waiting ${WAIT}s..."
  sleep $WAIT
  WAIT=$((WAIT * 2))
  [ $WAIT -gt 60 ] && WAIT=60
done

# 3. On exhaustion: diagnose, preserve, flag
if [ $EXIT_CODE -ne 0 ]; then
  zjj recover --diagnose   # surfaces orphans, stale locks, DB issues
  echo "EXHAUSTED: merge retries failed. Workspace preserved. Filing bead."
  # DO NOT zjj abort. The workspace is evidence.
  # File a bead (see Step 2), document exact conflict output.
  # If state is corrupt, rollback: zjj rollback --to <cp-id> <session>
  # Flag the user — this needs manual intervention.
fi
```

**TRADITIONAL GIT/JJ WORKFLOW (if not using zjj):**

```bash
# === GIT WORKFLOW ===
# On the branch to merge:
git checkout main
git pull --rebase
git merge --no-ff <branch-name>
# Or if clean history preferred:
git rebase main <branch-name> && git checkout main && git merge --ff-only <branch-name>

# Delete the merged branch
git branch -d <branch-name>

# === JJ WORKFLOW ===
jj rebase -d main@origin
jj git push --bookmark <name>
```

**Post-merge verification:**
```bash
# Re-run quality gates on main after merge
# This catches integration issues
git checkout main  # Ensure you're on main
make test || cargo test || gleam test || npm test

# If using zjj done, it already switched you to main
# Just verify tests pass:
cargo test || gleam test || npm test
```

**If merge introduces failures:**
```
Post-merge tests fail?
├─ Merge conflict resolution error → Fix, recommit
├─ Integration issue → Fix now if quick, or:
│   ├─ zjj users: No undo needed, workspace is gone. Fix in main.
│   └─ git users: Revert merge, file bead, fix in new branch
└─ Pre-existing → File bead (should have been caught in Step 3)
```

**ZJJ MERGE QUEUE ADVANTAGES:**
- **Single command** - `zjj done` handles sync, merge, push, cleanup
- **Atomic** - Either completes fully or fails with workspace intact
- **Clean** - Removes workspace and Zellij session automatically
- **Safe** - Can `zjj sync` to preview before `zjj done`
- **Tracked** - All workspaces visible in `zjj list`

**Required Outputs:**
- [ ] All completed branches/workspaces merged to main
- [ ] Merged branches/workspaces deleted/removed
- [ ] Quality gates pass on main after merge
- [ ] No orphan branches/workspaces remain (except documented in-progress work)
- [ ] All zjj sessions cleaned up for merged work

---

### Step 5: Update Issue/Bead Status

**Objective:** Every issue/bead reflects reality.

**Actions:**
```bash
# List all open beads assigned to you
br list --status open --assignee @me

# List beads in progress
br list --status in_progress

# Close completed beads with reason
br close <bead-id> --reason "Completed: <summary of what was done>"
br close <bead-id> --suggest-next    # close and show what's unblocked

# Update in-progress beads with current state
br update <bead-id> --notes "Session status: <current state>"
br update <bead-id> --status in_progress

# Verify new beads from Step 2 are well-formed
br lint

# Check dependency graph for cycles
br dep cycles

# Show what's ready for next session
br ready
```

**Required Outputs:**
- [ ] Completed beads closed with reason
- [ ] In-progress beads updated with current state
- [ ] New beads from Step 2 are filed, labeled, and pass `br lint`
- [ ] Dependencies wired (`br dep`) and no cycles
- [ ] `br ready` shows accurate next-up work

---

### Step 6: Push to Remote (MANDATORY)

**Objective:** Everything on remote. Nothing local-only.

```bash
# === JJ (preferred) ===
jj git fetch                        # fetch first — rebase happens automatically
jj git push --bookmark main

# === GIT (fallback) ===
git pull --rebase
git push

# Verify
git log --branches --not --remotes
# Expected: empty (no unpushed commits)
git status
# MUST show: "Your branch is up to date with 'origin/main'"
```

**Push failures use exponential backoff** — see the **Exponential Backoff Protocol** in Core Principles (push pseudocode is there). Same schedule: 2→4→8→16→32→60s, max 6 retries. `jj git fetch` before each retry — a non-fast-forward rejection often self-resolves after a fresh fetch + rebase. Permission denied or branch protection: STOP immediately, backoff won't help.

**CRITICAL RULES:**
- **Work is NOT complete until push succeeds**
- **NEVER stop before pushing**
- **NEVER say "ready to push when you are" — YOU must push**
- **If push fails, use backoff. If backoff exhausts, flag the user — do not spin forever**

**Required Outputs:**
- [ ] `git push` succeeded
- [ ] `git status` shows up to date
- [ ] No unpushed commits remain

---

### Step 7: Clean Up Orphans

**Objective:** No dangling state. Main is the only thing left.

```bash
# === ZJJ WORKSPACES (CHECK FIRST) ===
# List all zjj workspaces
zjj list 2>/dev/null

# For each workspace, decide:
#   - Work complete → zjj done -w <name> (merges and removes)
#   - Work incomplete → File bead, then zjj abort -w <name>
#   - Work abandoned → zjj abort -w <name>

# Clean up completed/abandoned workspaces
zjj clean --dry-run 2>/dev/null  # Preview what will be cleaned
zjj clean 2>/dev/null            # Remove stale workspaces

# Verify no sessions remain
SESSIONS=$(zjj list 2>/dev/null | wc -l)
if [ "$SESSIONS" -gt 0 ]; then
  echo "WARNING: $SESSIONS zjj workspaces still active"
  zjj list 2>/dev/null
  echo "Each must be completed (zjj done -w <name>), aborted (zjj abort -w <name>), or documented as in-progress"
fi

# === VERIFY WORKSPACE DIRECTORIES ARE ACTUALLY GONE ===
# zjj list may show clean, but the directory could still be on disk
# (zjj done cleans up, but failures or interruptions can leave orphans)
WORKSPACE_BASE=$(zjj config workspace_dir 2>/dev/null || echo "../lewis__workspaces")
if [ -d "$WORKSPACE_BASE" ]; then
  LEFTOVER_DIRS=$(ls -d "$WORKSPACE_BASE"/*/ 2>/dev/null | wc -l)
  if [ "$LEFTOVER_DIRS" -gt 0 ]; then
    echo "WARNING: $LEFTOVER_DIRS workspace directories still on disk in $WORKSPACE_BASE:"
    ls -la "$WORKSPACE_BASE"/
    echo "These are orphans. Each must be traced to a session (zjj list --all) or removed manually."
  fi
fi

# === GIT BRANCHES ===
# Delete all branches merged into main
git branch --merged main | grep -v "\*\|main\|master" | xargs -n 1 git branch -d 2>/dev/null

# Flag unmerged branches (should have been handled in Step 1)
UNMERGED=$(git branch --no-merged main | grep -v "\*" | tr -d ' ')
if [ -n "$UNMERGED" ]; then
  echo "WARNING: Unmerged branches remain: $UNMERGED"
  echo "Each must have a bead filed or be explicitly abandoned."
fi

# === GIT WORKTREES ===
# Worktrees should be managed by zjj, but check for orphans
git worktree prune
WORKTREES=$(git worktree list | grep -v "$(git rev-parse --show-toplevel)" | wc -l)
if [ "$WORKTREES" -gt 0 ]; then
  echo "WARNING: $WORKTREES worktrees still exist (not tracked by zjj)"
  git worktree list
  echo "These should be removed: git worktree remove <path>"
fi

# === STASHES ===
STASH_COUNT=$(git stash list | wc -l)
if [ "$STASH_COUNT" -gt 0 ]; then
  echo "WARNING: $STASH_COUNT stashes exist"
  git stash list
  echo "Stashes should be applied, filed as bead, or dropped"
fi

# === REMOTE BRANCHES ===
git remote prune origin

# === TEMP FILES ===
git clean -n  # Dry run
# If safe: git clean -fd
```

**FLAG THE USER for any remaining:**
- Unmerged branches (must have beads or be abandoned)
- Open zjj workspaces (must be completed with `zjj done`, in-progress with bead, or removed)
- Orphan worktrees not tracked by zjj (must be removed with `git worktree remove`)
- Stashes older than today (must be applied, filed as bead, or dropped)

**ZJJ CLEANUP DECISION TREE:**
```
zjj workspace found?
├─ Work is COMPLETE → zjj done -w <name> (merges, pushes, cleans up)
├─ Work is INCOMPLETE but VALUABLE → File bead, then:
│   ├─ Want to resume soon → Keep workspace, document in handoff
│   └─ Not resuming → zjj abort -w <name> (proper discard, updates bead)
├─ Work is ABANDONED → zjj abort -w <name>
└─ Workspace is STALE (>7 days) → Flag user, likely abandoned
```

**Note:** `zjj done -w <name>` and `zjj abort -w <name>` both work from main — no need to `cd` into each workspace during the audit.

**Required Outputs:**
- [ ] All zjj workspaces either completed (zjj done) or removed
- [ ] Merged git branches deleted
- [ ] Worktrees pruned
- [ ] Stashes reviewed and cleared
- [ ] Remote branches pruned
- [ ] Any remaining orphans flagged to user with explanation

---

### Step 8: Final Verification (Main Is Clean)

**Objective:** Prove main is in a pristine state.

```bash
# 1. On main
git checkout main 2>/dev/null || jj edit main 2>/dev/null

# 2. Working tree is clean
git status
# Expected: "nothing to commit, working tree clean"

# 3. All commits pushed
git log --branches --not --remotes
# Expected: empty

# 4. Quality gates pass on main
make test || cargo test || gleam test || npm test
cargo clippy -- -D warnings 2>/dev/null
cargo fmt --check 2>/dev/null || gleam format --check src/ test/ 2>/dev/null

# 5. No uncommitted changes
git diff --quiet && git diff --cached --quiet

# 6. No orphan branches
ORPHANS=$(git branch --no-merged main | grep -v "\*" | wc -l)
echo "Unmerged branches: $ORPHANS"

# 7. No dangling worktrees (zjj-managed or orphaned)
WORKTREES=$(git worktree list | grep -v "$(git rev-parse --show-toplevel)" | wc -l)
echo "Extra worktrees: $WORKTREES"

# 8. No zjj workspaces remaining (all should be merged or removed)
WORKSPACES=$(zjj list 2>/dev/null | wc -l)
echo "Active zjj workspaces: $WORKSPACES"
if [ "$WORKSPACES" -gt 0 ]; then
  echo "WARNING: zjj workspaces still active:"
  zjj list 2>/dev/null
fi

# 9. No stashes
STASHES=$(git stash list | wc -l)
echo "Stashes: $STASHES"

# 10. Verify you're not in a zjj session
CURRENT_SESSION=$(echo $ZELLIJ_SESSION_NAME 2>/dev/null)
if [ -n "$CURRENT_SESSION" ] && [[ "$CURRENT_SESSION" == zjj-* ]]; then
  echo "WARNING: Still in zjj Zellij session: $CURRENT_SESSION"
  echo "Exit the session or use zjj attach to switch to main session"
fi
```

**PASS criteria (ALL must be true):**
```
Main Is Clean Checklist:
  [PASS/FAIL] Working tree clean
  [PASS/FAIL] All commits pushed
  [PASS/FAIL] Tests passing
  [PASS/FAIL] Zero lint violations
  [PASS/FAIL] Zero warnings
  [PASS/FAIL] Code formatted
  [PASS/FAIL] No orphan branches (or all have beads)
  [PASS/FAIL] No dangling worktrees
  [PASS/FAIL] No active zjj workspaces (all merged via zjj done or removed)
  [PASS/FAIL] No stale stashes
  [PASS/FAIL] Not in a zjj session (should be in main session or no session)
```

**If ANY check fails at this point:**
```
Final verification failure?
├─ Quality gate failure → MUST fix or revert. Cannot land dirty.
├─ Orphan branch → File bead if not already done. Flag user.
├─ Dangling worktree → Remove with git worktree remove
├─ Active zjj workspace → Either:
│   ├─ Work complete → zjj done -w <name>
│   ├─ Work in-progress → Document in handoff, keep (exception)
│   └─ Work abandoned → zjj abort -w <name>
├─ Stale stash → Apply, file as bead, or drop. Flag user.
└─ In zjj session → Exit session or zjj attach to main
```

**Required Outputs:**
- [ ] Every check PASS
- [ ] Main is provably clean
- [ ] No zjj workspaces remain (or documented as in-progress)
- [ ] Not in a zjj Zellij session
- [ ] Any exceptions documented and flagged

---

### Step 9: Bead Reconciliation

**Objective:** Ensure every smell surfaced during landing has been properly tracked.

```bash
# List all open beads — these should include everything filed this session
br list --status open

# Lint all open beads for missing required sections
# br lint enforces: Acceptance Criteria (task/feature), Steps to Reproduce (bug)
br lint

# For each bead, verify EARS + BDD sections are present:
# - Description has: Context, Smell Classification, Dependencies, Requirements (EARS), Variants
# - Acceptance has: High-Level Criteria, BDD Scenarios (Given/When/Then), Verification checklist
# If sparse, enrich now:
br update <bead-id> --description "..." --acceptance "..."

# Check dependency graph is clean
br dep cycles

# Verify dependencies are wired
br dep list <bead-id>            # for each bead with known relationships

# Show what's ready for next session
br ready --pretty
```

**Reconciliation check:**
```
For each quality gate failure that was filed (not fixed):
├─ Bead exists via br show <id>? → Good
├─ Bead exists but missing EARS/BDD sections? → Enrich NOW:
│   br update <id> --description "..." --acceptance "..."
├─ No bead filed? → File it now with full template. This is BLOCKING.
│   br create "[smell-type] ..." --description "..." --acceptance "..."
└─ br lint passes for this bead? → Good. If not, fix sections.

For each orphan branch/worktree/session that remains:
├─ Bead exists explaining why it's open? → Good
├─ No bead? → File one or remove the orphan. No middle ground.
│   br create "[process] Orphan: ..." --labels "smell:process"
└─ Dependency wired to parent work? → br dep add <orphan-bead> <parent-bead>
```

**Required Outputs:**
- [ ] Every unfixed failure has a bead with full EARS + BDD template
- [ ] `br lint` passes on all open beads
- [ ] Dependencies wired with `br dep` — no cycles (`br dep cycles`)
- [ ] Every remaining orphan has a bead or has been removed
- [ ] `br ready --pretty` shows accurate next-up work

---

### Step 10: Hand Off

**Objective:** Next session knows exactly where things stand.

**Handoff Report:**
```markdown
## Session Complete — Landing Report

### Work Completed
- [List of features/fixes implemented]
- [Commits pushed: X commits]
- [Issues/beads closed: #123, #456]

### Main Status
- Branch: main
- Quality Gates: ALL PASSING | EXCEPTIONS NOTED
- Tests: [count] passing, [count] failing (beads filed)
- Lint: clean | [count] warnings (beads filed)
- Warnings: zero | [count] (beads filed)
- Format: clean
- Remote Sync: up to date

### Smells Surfaced (Beads Filed)
- [bead-id]: [smell type] — [brief description] — [severity]
- [bead-id]: [smell type] — [brief description] — [severity]

### Orphans Remaining (with justification)
- Branch `feature-x`: In-progress, bead [id] tracks it, ETA next session
- zjj session `experiment`: Paused, bead [id], will resume or abandon

### Cleanup Performed
- Branches deleted: [list]
- Worktrees removed: [list]
- Sessions closed: [list]
- Stashes cleared: [count]

### Next Steps
- [What should be done next]
- [Any blockers or dependencies]
- [Which beads to tackle first]

### Notes
- [Technical decisions made this session]
- [Known issues or workarounds]
- [Context that won't be obvious next time]
```

**Required Outputs:**
- [ ] Summary of work completed
- [ ] Main status with quality gate results
- [ ] All beads listed with smell types
- [ ] All orphans justified or removed
- [ ] Next steps clear
- [ ] Important context captured

---

## Landing Checklist (Full)

Run through this before ending ANY session:

```
Session Landing Checklist:

Step 1: Audit Orphans
  [ ] All branches inventoried
  [ ] All worktrees accounted for
  [ ] All zjj sessions accounted for
  [ ] All stashes reviewed
  [ ] User flagged for any anomalies

Step 2: File Beads for Remaining Work
  [ ] TODOs/FIXMEs catalogued
  [ ] Beads created for every incomplete item
  [ ] Each bead has smell type and severity

Step 3: Quality Gates (ZERO TOLERANCE)
  [ ] Tests passing (zero failures)
  [ ] Linting clean (zero warnings, -D warnings)
  [ ] Zero compiler/runtime warnings
  [ ] Formatting clean
  [ ] Build succeeds
  [ ] Type checking passes
  [ ] Failures either fixed or filed as beads

Step 4: Merge to Main
  [ ] All completed branches merged
  [ ] Merged branches deleted
  [ ] Quality gates pass post-merge

Step 5: Update Issue/Bead Status
  [ ] Completed beads closed
  [ ] In-progress beads updated
  [ ] New beads filed and tagged

Step 6: Push to Remote (MANDATORY)
  [ ] git push succeeded
  [ ] Branch up to date with remote
  [ ] No unpushed commits

Step 7: Clean Up Orphans
  [ ] Merged branches deleted
  [ ] Worktrees pruned
  [ ] zjj sessions cleaned
  [ ] Stashes cleared
  [ ] Remote branches pruned

Step 8: Final Verification
  [ ] Main is clean (all checks PASS)
  [ ] Working tree clean
  [ ] Tests pass on main
  [ ] Zero lint/warnings on main

Step 9: Bead Reconciliation
  [ ] Every unfixed failure has a bead
  [ ] Every orphan has a bead or is removed
  [ ] All beads have sufficient context

Step 10: Hand Off
  [ ] Landing report written
  [ ] Main status documented
  [ ] Smells listed with beads
  [ ] Next steps clear
```

---

## Failure Scenarios and Recovery

### Scenario 1: Tests Fail
```
Problem: Tests fail during quality gates
Smell Type: test | code | design

Recovery:
1. Read test output — understand the failure
2. Is this a new failure (introduced this session)?
   YES → Fix it now. This is YOUR responsibility.
   NO  → File bead as pre-existing, continue landing.
3. If fixing: fix, re-run ALL gates, commit the fix
4. If filing: bead must include exact test output, file path, and context

NEVER push with failing tests you introduced
Pre-existing failures get beads but don't block landing
```

### Scenario 2: Linter Warnings
```
Problem: Clippy/ESLint/etc. reports warnings
Smell Type: code

Recovery:
1. Read each warning
2. Quick fix (< 2 min each)? → Fix, commit, re-run
3. Suppression justified? → Add allow with comment explaining why, commit
4. Complex fix needed? → File bead with smell type "code", severity "important"

STANDARD: -D warnings (warnings are errors)
```

### Scenario 3: Compiler Warnings
```
Problem: Build produces warnings (not errors)
Smell Type: code | design

Recovery:
1. Warnings about unused imports/variables → Fix now (trivial)
2. Warnings about deprecated APIs → File bead, severity "important"
3. Warnings about unsafe patterns → Fix now if possible, else file bead severity "blocking"

STANDARD: Zero warnings. Warnings are future bugs.
```

### Scenario 4: Open Branches Discovered
```
Problem: Branches exist that aren't merged
Smell Type: process

Recovery:
1. For each branch, determine status:
   - Last commit date (stale if > 7 days)
   - Relationship to any bead/issue
   - Whether work is complete or partial
2. Complete work → Merge to main
3. Partial but valuable → File bead, decide merge-or-abandon
4. Stale/abandoned → Delete branch, file bead if work had value
5. Active in-progress → Document in handoff, keep (exception)

FLAG USER: "Branch 'feature-x' has been open N days without commits.
           Merge, file as bead, or abandon?"
```

### Scenario 5: Dangling Worktrees/Sessions
```
Problem: git worktrees or zjj sessions exist
Smell Type: process

Recovery:
1. Run zjj recover --diagnose first — may auto-surface the issue
2. For each worktree/session:
   - Is it tracked by zjj? → Use zjj done -w <name> or zjj abort -w <name>
   - Is it a raw git worktree? → git worktree remove <path>
   - Does it have uncommitted work? → Commit first, then handle
3. If work is complete → zjj done -w <name> (merges to main)
4. If work is incomplete → File bead, zjj abort -w <name>

FLAG USER: "Found N worktrees/sessions not tracked by zjj.
           These need manual review."
```

### Scenario 6: Push Rejected
```
Problem: git push / jj git push fails
Smell Type: process

Recovery — USE EXPONENTIAL BACKOFF (see Core Principles):
1. Rejected (non-fast-forward) → jj git fetch (auto-rebases), then retry with backoff
2. Network error → Retry with backoff. If persistent after exhaustion, flag user.
3. Permission denied → STOP immediately. Flag user. Backoff won't help.
4. Branch protection → STOP immediately. Flag user, may need PR.

MUST use backoff loop from Step 6. Do NOT retry in a tight loop.
If backoff exhausts (6 retries), STOP and flag user — do not spin forever.
```

---

## Smell Classification Reference

Every failure surfaced during landing gets classified:

| Smell Type | Examples | Default Severity |
|------------|----------|-----------------|
| **design** | Test reveals architectural issue, conflicting requirements, API inconsistency | important |
| **code** | Linter warning, compiler warning, dead code, type error, unsafe pattern | important |
| **test** | Failing test, missing test coverage, flaky test | important |
| **process** | Stale branch, orphan worktree, uncommitted stash, unfiled work | minor |
| **debt** | TODO/FIXME, deprecated API usage, known workaround, deferred refactor | minor |

**Severity levels:**
- **blocking** — Cannot land until fixed. New failure introduced this session.
- **important** — Must be fixed soon. Filed as bead. Next session priority.
- **minor** — Should be fixed eventually. Filed as bead. Backlog.

---

## Integration with Development Workflow

### End of Feature Development (zjj workflow)
```
Feature complete → Landing workflow → zjj done -m "..." → Close bead → Push (automatic) → Clean
```

**With zjj:**
- Work in isolated workspace: `zjj spawn feature-name`
- Commit regularly in workspace
- When complete: `zjj done -m "Implement feature X"` (merges, pushes, cleans up)
- Close bead: `br close <bead-id>`

### End of Feature Development (traditional git)
```
Feature complete → Landing workflow → Merge to main → Close bead → Push → Clean branches
```

### End of Bug Fix (zjj workflow)
```
Bug fixed → Tests pass → zjj done -m "..." → Close bead → Automatic cleanup
```

**With zjj:**
- Spawn workspace: `zjj spawn fix-bug-123`
- Fix + commit + test
- Merge with: `zjj done -m "Fix bug #123"`
- No manual branch cleanup needed

### End of Bug Fix (traditional git)
```
Bug fixed → Tests pass → Landing workflow → Merge → Close bead → Push → Clean
```

### Partial Work (Interrupted Session with zjj)
```
Work incomplete → File bead with full context → Commit in workspace →
zjj sync (optional) → Document in handoff → Workspace remains for next session
```

**Advantage:** zjj workspace preserves context. Next session: `zjj attach <workspace-name>` or `zjj list` to see what was in progress.

### Partial Work (Interrupted Session - traditional)
```
Work incomplete → File bead with full context → Commit WIP → Push branch →
Document in handoff → Flag as orphan for next session
```

### End of Refactoring (zjj workflow)
```
Refactor complete → ALL tests pass → Zero warnings → zjj done -m "..." → Clean
```

### ZJJ Merge Queue Workflow (The Recommended Way)

**Single-Command Merge:**
```bash
# From within your zjj workspace
zjj done -m "Brief description of work"
# This automatically:
#   1. Syncs commits to base branch
#   2. Rebases or merges to main
#   3. Pushes to remote
#   4. Removes workspace
#   5. Cleans up Zellij session
#   6. Returns you to main branch
```

**Review-Before-Merge:**
```bash
# See what will be merged
zjj sync                    # Sync changes but don't merge
zjj diff main               # Review diff
git log main..HEAD          # See commit history

# Decide:
zjj done -m "..."           # Proceed with merge
# OR
zjj abort -w <workspace>    # Discard work (file bead first! Updates bead to abandoned)
```

**Conflict Resolution (use backoff — see Core Principles §6):**
```bash
# If zjj done fails with exit code 3:
zjj checkpoint create       # safety snapshot first
zjj sync                    # re-sync onto main (may auto-resolve)
# If sync itself has conflicts, resolve in workspace:
#   jj add . && jj commit -m "resolve conflicts"
zjj retry                   # retries the last failed done
# If still failing after backoff exhaustion:
zjj recover --diagnose      # see what's broken
# DO NOT abort — workspace is evidence
```

**Advantages of zjj as Merge Queue:**
- **Atomic:** Either completes fully or fails cleanly
- **No orphans:** Workspace removal is automatic
- **No manual cleanup:** Branches, worktrees, sessions all handled
- **Safe preview:** `zjj sync` + `zjj diff` before `zjj done`
- **Tracked state:** `zjj list` shows all in-flight work
- **Zellij integration:** Terminal session management included

**When NOT to use zjj done:**
- Work is incomplete and you're coming back → Keep workspace, document in handoff
- Work is experimental and should be discarded → `zjj abort -w <name>` (updates bead to abandoned)
- Work has merge conflicts you can't resolve → File bead, get help, resume later. Do NOT abort — workspace is evidence

---

## Anti-Patterns to Avoid

| Anti-Pattern | Problem | Landing Skill Way |
|--------------|---------|-------------------|
| "I'll fix that test later" | Tests rot, failures accumulate | Fix now or file bead. No third option. |
| Ignoring warnings | Warnings become bugs | Zero tolerance. `-D warnings` standard. |
| Leaving branches open | Process debt, confusion | Merge, file bead, or abandon. Every branch decided. |
| Stashing instead of committing | Stashes are invisible work | Commit or file bead. Stashes die at landing. |
| "Main is mostly clean" | Mostly clean is dirty | ALL checks PASS or fix/file. |
| Pushing without re-testing | Merge can break things | Quality gates run AFTER merge to main. |
| Filing beads without context | Useless beads nobody can act on | Smell type + severity + reproduction + exact output. |
| Landing without auditing orphans | Worktrees/sessions accumulate | Step 1 is mandatory. Audit everything. |
| `zjj remove -f` to discard | Silent loss, bead not updated | `zjj abort -w <name>` — proper discard. |
| Retrying done in a tight loop | Hammers merge queue | `zjj sync` + `zjj retry` with backoff. |
| No checkpoint before done | Can't recover if merge corrupts | `zjj checkpoint create` before every done. |
| Ignoring exit code 3 | Silently failed merge | Check exit code, enter backoff, run `recover --diagnose`. |

## Best Practices

### Smell-Driven Quality
- **Treat every failure as information** — it's telling you something about design, code, or process
- **Classify the smell** — knowing it's a "design smell" vs "code smell" guides the fix
- **File with reproduction context** — a bead without reproduction steps is useless
- **Severity drives priority** — blocking > important > minor

### Landing Discipline
- **Land frequently** — smaller landings are cleaner landings
- **Quality gates are non-negotiable** — no exceptions, no "just this once"
- **Orphan audit is mandatory** — the first step, not an afterthought
- **Main is sacred** — if main is dirty, fix it before doing anything else
- **Flag the user, don't hide problems** — transparency prevents debt accumulation

---

## Commands Reference

### Audit Orphans
```bash
# ZJJ workspaces (check first)
zjj list 2>/dev/null
zjj status 2>/dev/null

# Git branches
git branch --no-merged main

# Worktrees
git worktree list

# Stashes
git stash list
```

### ZJJ Merge Queue (Recommended Workflow)
```bash
# Pre-flight checks
zjj can-i done              # verify preconditions
zjj whatif done             # detailed preview (steps, reversibility)
zjj checkpoint create       # safety snapshot

# Complete and merge work
zjj done -m "Brief description"
# zjj done is reversible: zjj undo (24hr, not pushed)

# From main (audit mode — no need to cd into workspace)
zjj done -w <name> -m "..."

# If done fails with exit code 3: backoff loop
zjj sync && zjj retry       # sync then retry (see backoff protocol)

# Preview before merging
zjj sync                    # Sync without merging
zjj diff main               # Review changes
zjj done -m "..."           # Proceed

# Discard workspace (opposite of done)
zjj abort -w <name>         # proper discard, updates bead to abandoned

# Recovery
zjj recover --diagnose      # see what's broken
zjj recover                 # auto-fix orphans, stale locks, DB issues
zjj rollback --to <cp> <name>  # restore from checkpoint

# List all workspaces
zjj list
zjj list --verbose

# Clean stale workspaces
zjj clean --dry-run
zjj clean
```

### Quality Gates (One Command)
```bash
# Rust
cargo test && cargo clippy -- -D warnings && cargo fmt --check && RUSTFLAGS="-D warnings" cargo build

# Gleam
gleam test && gleam format --check src/ test/ && gleam build

# Node
npm test && npm run lint && npm run format:check && npm run build && npm run typecheck
```

### Clean Orphans
```bash
# Diagnose first — surfaces orphans before you touch anything
zjj recover --diagnose

# ZJJ workspaces — abort discards properly, done merges
zjj abort -w <name>         # discard (updates bead)
zjj done -w <name>          # merge (from main, no cd needed)
zjj clean 2>/dev/null       # remove sessions with missing workspaces

# Git branches
git branch --merged main | grep -v "\*\|main\|master" | xargs -n 1 git branch -d

# Worktrees
git worktree prune

# Stashes (after review!)
git stash clear

# Remote branches
git remote prune origin
```

### Push and Verify (Traditional Git)
```bash
git pull --rebase && git push && git log --branches --not --remotes && git status
```

**Note:** If using zjj, `zjj done` handles push automatically.

### File Bead (Full Template)
```bash
br create "[smell-type] Brief description" \
  --type task \
  --priority 2 \
  --labels "smell:<type>,severity:<level>" \
  --description "## Context
<what was being done>

## Smell Classification
- **Type**: <design|code|test|process|debt>
- **Severity**: <blocking|important|minor>
- **Gate Failed**: <test|lint|format|build|type|warning|N/A>

## Requirements (EARS)
<When/While/Where/If requirements>

## Variants
- **Happy Path**: <expected flow>
- **Error Path**: <failure recovery>" \
  --acceptance "## Acceptance Criteria
1. <criterion>

## Acceptance Tests (BDD)
### Scenario: <name>
  Given <precondition>
  When <action>
  Then <expected result>"
```

### Quick Capture (Enrich Later)
```bash
BEAD_ID=$(br q "[smell-type] Brief description" --labels "smell:<type>")
br update "$BEAD_ID" --description "..." --acceptance "..."
```

### Wire Dependencies
```bash
br dep $BEAD_ID --blocks $OTHER_ID
br dep add $BEAD_ID $BLOCKER_ID
br dep relate $BEAD_ID $RELATED_ID
```

### Validate Beads
```bash
br lint                           # lint all open beads
br lint $BEAD_ID                  # lint specific bead
```

---

**Landing Skill Version**: 2.1.0
**Last Updated**: February 2026
**Status**: Production Ready
**Priority**: MANDATORY for session completion
**Standard**: Zero warnings, zero failures, zero orphans — or beads filed for every exception
**Backoff**: Exponential backoff on all merge and push retries. Max 6 retries, 60s cap. Never spin.
