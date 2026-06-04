# Autofeeder Pipeline — Full Step-by-Step

Execute this pipeline in order. Each phase must complete before the next begins.

## Phase 0: Overlap Guard

Prevent concurrent cycles from corrupting state:

```bash
cd /home/lewis/src/veloxide
if [ -f /tmp/autofeeder.lock ]; then
  PID=$(cat /tmp/autofeeder.lock)
  if kill -0 "$PID" 2>/dev/null; then
    echo "SKIP: autofeeder cycle already running (PID $PID)"
    exit 0
  fi
fi
echo $$ > /tmp/autofeeder.lock
trap 'rm -f /tmp/autofeeder.lock' EXIT
```

## Phase 1: Sync & Status

```bash
cd /home/lewis/src/veloxide
git fetch --all --prune
git checkout main && git pull origin main --rebase
bd dolt pull
bd ready --json
```

Check fleet status:
```bash
gt feed --problems 2>&1
```

Record the number of ready beads and any fleet issues for the report.

## Phase 2: Review Unmerged Polecat Branches

### 2.1 Identify candidates

```bash
cd /home/lewis/src/veloxide
git branch -r --no-merged | grep polecat | grep -v HEAD
```

If no branches found, skip to Phase 4 (Active Scanning).

### 2.2 For each branch, review sequentially

Process one branch at a time. Do NOT merge until ALL gates pass for that branch.

#### Step A: Check out and diff

```bash
BRANCH=<branch-name>
git checkout main
BASE=$(git merge-base main "origin/$BRANCH")
git diff "$BASE" "origin/$BRANCH" --stat
git diff "$BASE" "origin/$BRANCH"
```

Understand what changed. Note files, modules, and scope of change.

#### Step B: Build & Lint Gate

```bash
git checkout origin/$BRANCH
moon run :lint-src 2>&1
```

If clippy fails: record every warning with file:line. Create P0 beads. Do NOT merge.

#### Step C: Test Gate

```bash
moon run :test 2>&1
```

If tests fail: record every failure. Create P0 beads. Do NOT merge.

#### Step D: Architectural Drift Review

Invoke the `architectural-drift` skill on the diff. Check:

1. **File length** — any file now over 300 lines?
   ```bash
   BASE=$(git merge-base main HEAD)
   git diff --name-only "$BASE" HEAD | while read f; do
     [ -f "$f" ] && wc -l "$f"
   done
   ```
2. **DDD violations** — can illegal states be represented? Are domain boundaries crossed?
3. **V2 spec compliance** — FD3/FD4 for IPC, group commits via DbWriterActor, no external DBs, no Wasm execution in engine

Record every finding. Create beads for each violation, including low/minor follow-ups. Any branch-review finding without canonical disposition `fixed_with_evidence`, `owner_approved_debt`, or `owner_approved_no_action` fails the review; `blocker` rejects the branch.

#### Step E: Black Hat Review

Invoke the `black-hat-reviewer` skill on the diff. Check:

1. Contract parity — do types match their usage?
2. Farley constraints — is the code deployable?
3. Holzman Rust — NASA/JPL discipline, panic-free paths, bounded mutation, performance proof?
4. Security — OWASP top 10, injection, `unsafe` usage, credential leaks
5. Error handling — no unwraps in production paths, proper error propagation

Record every finding. P0 for security issues, P1-P3 for quality, Holzman, DDD, UX, docs, style, and follow-up defects. Any branch-review finding without canonical disposition `fixed_with_evidence`, `owner_approved_debt`, or `owner_approved_no_action` fails the review; `blocker` rejects the branch.

#### Step F: Red Queen — Test Design / Assertion Strength

Invoke the `red-queen` skill on the diff. Check:

1. Are new code paths tested?
2. Are tests adversarial (testing failure modes) or just happy-path?
3. Property-based test coverage — are there proptest/invariants for complex logic?
4. Edge cases — boundary values, empty inputs, concurrent access

Record every finding. P1 for missing coverage, P2 for weak test effectiveness or assertion gaps, P3 for lower-severity test hygiene observations. Any branch-review finding without canonical disposition `fixed_with_evidence`, `owner_approved_debt`, or `owner_approved_no_action` fails the review; `blocker` rejects the branch.

#### Step G: Manual QA

Invoke the `hands-on-qa` skill. Actually run:

```bash
# Does the branch build?
moon run :build 2>&1

# Do CLI commands work?
cargo run -p vo-cli -- history --json 2>&1

# Does the frontend build? (if frontend changed)
cd crates/vo-frontend && dx build 2>&1
```

Record every finding. P0 for broken builds, P1 for broken CLI/API, P2-P3 for UX, docs, output formatting, and observation findings. Any branch-review finding without canonical disposition `fixed_with_evidence`, `owner_approved_debt`, or `owner_approved_no_action` fails the review; `blocker` rejects the branch.

#### Step H: Architecture QA

Invoke the `arch-design-qa` skill on the diff. Check:

1. V2 architecture spec alignment (see `docs/adr/v2/`)
2. Domain boundaries — is code in the right crate?
3. AI-native — do CLI interfaces output strict JSON? Are schemas well-defined?
4. Type-driven design — are illegal states unrepresentable?

Record every finding. P1 for spec violations, P2 for boundary leaks, P3 for lower-severity design observations. Any branch-review finding without canonical disposition `fixed_with_evidence`, `owner_approved_debt`, or `owner_approved_no_action` fails the review; `blocker` rejects the branch.

## Phase 3: Gate Decision

### Branch PASSED all reviews:

This state means zero open reviewer findings, or every finding at every severity has canonical disposition `fixed_with_evidence`, `owner_approved_debt`, or `owner_approved_no_action`. A newly created bead is not sufficient by itself.

```bash
git checkout main
git merge --no-edit origin/$BRANCH

# Final CI gate on merged result
moon run :ci 2>&1

if CI passes:
  git push origin main
  echo "APPROVED: $BRANCH merged and pushed"
else:
  git merge --abort  # safe revert — never use git reset --hard
  # Create P0 bead for CI failure on merged result
  bd create --title="CI failure on merge of $BRANCH" \
    --description="Branch: $BRANCH
Phase: Post-merge CI
CI output: <paste>
Fix: Resolve conflicts between main and $BRANCH" \
    --type=bug -p 0 --json
fi
```

### Branch FAILED any review:

Do NOT merge. Before creating beads, check for duplicates:

```bash
# Dedup: search existing beads for this branch + finding
bd search "$BRANCH <finding-keyword>" 2>&1
```

For every finding, if no existing bead matches, create. Newly created beads mean the branch remains rejected until disposition changes to `fixed_with_evidence`, `owner_approved_debt`, or `owner_approved_no_action`:

```bash
bd create \
  --title="<finding>" \
  --description="Branch: $BRANCH
Reviewer: <which gate failed>
File: <file:line>
Issue: <what's wrong>
Fix: <what the polecat needs to do>
Severity: <P0/P1/P2/P3>" \
  --type=bug \
  -p <0-3> \
  --deps discovered-from:<branch-bead-if-exists> \
  --json
```

Then proceed to the next branch.

## Phase 4: Active Scanning

This phase ALWAYS runs, regardless of whether there are branches to review.

**Deduplication**: Before creating any bead from active scanning, search existing beads:
```bash
bd search "<finding-keyword>" 2>&1 | head -5
```
If an open bead already exists for the same file + issue, skip it.

### 4.1 Architectural Drift Scan

Invoke `architectural-drift` skill on the full codebase:
```bash
find crates -name "*.rs" -exec wc -l {} \; | sort -rn | head -20
```
Any file over 300 lines gets a P2 bead. DDD, V2, low/minor, and observation findings get P2-P3 beads.

### 4.2 Architecture QA Scan

Invoke `arch-design-qa` skill. Look for:
- Spec compliance gaps
- Domain boundary violations
- Missing JSON schemas
- Type safety gaps

P2-P3 beads for each finding.

### 4.3 Red Queen Scan

Invoke `red-queen` skill. Look for:
- Untested modules
- Weak test suites
- Missing proptest coverage
- Happy-path-only tests

P2-P3 beads for each finding.

### 4.4 Hands-on QA Scan

Invoke `hands-on-qa` skill. Smoke test:
- `vo-cli history --json`
- `vo-cli check`
- API health endpoint (if running)
- Frontend build

P2-P3 beads for every failure or lower-severity finding.

### 4.5 Black Hat Scan

Invoke `black-hat-reviewer` skill on recent commits:
```bash
git log --oneline -20
```
Scan for security patterns, unsafe code, credential leaks.

P2-P3 beads for every finding.

## Phase 5: Sync Beads to Fleet

```bash
cd /home/lewis/src/veloxide
bd dolt push

# If git push failed during Phase 3, retry with rebase
git pull origin main --rebase && git push origin main
```

If dolt push fails, invoke the `dolt` skill for recovery. If git push fails after
3 retries, create a P0 bead and report in Phase 6. Do NOT skip dolt push — the
fleet cannot see beads until they're synced to the remote.

## Phase 6: Report

Print this exact format:

```
=== AUTOFEEDER REPORT ===
Timestamp: <ISO 8601>

Branches reviewed: <count>
  APPROVED + merged: <count>
    <branch-name>: merged, CI passed, pushed to main
  REJECTED: <count>
    <branch-name>: failed <gate-name> (bead: <id>)

Active scan findings: <count>
  arch-drift: <count>
  arch-qa: <count>
  red-queen: <count>
  qa: <count>
  black-hat: <count>

Beads created this cycle: <count>
  P0: <count>
  P1: <count>
  P2: <count>
  P3: <count>

Fleet-ready beads: <count>
Dolt sync: <pushed/failed>
Main: <short sha> (<clean/dirty>)
Issues needing human: <list any, or "none">
```
