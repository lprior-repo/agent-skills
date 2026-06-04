# Reviewer Integration Details

## Reviewer Skills Reference

Each reviewer is invoked as a skill during the pipeline. Here's how to integrate each one.

### 1. Architectural Drift (`architectural-drift`)

**When to invoke:** Phase 2 (branch review) and Phase 4 (active scan)

**What it checks:**
- Files exceeding 300 lines (hard limit)
- Scott Wlaschin DDD patterns — illegal states representable? Domain types leaked across boundaries?
- V2 architecture spec compliance:
  - FD3/FD4 IPC (no stdout for state)
  - Group commits via DbWriterActor (actors never write to fjall directly)
  - No external DBs (no Redis, no Postgres)
  - No Wasm execution in engine (only in vo-frontend)
  - At-most-one actor per workflow ID

**How to invoke:** Pass the diff to the skill. For branch review, scope to the diff. For active scan, run against the full `crates/` directory.

**Bead creation:**
Create or link one bead for every finding from this reviewer, regardless of severity.
```bash
bd create --title="arch-drift: <file> exceeds 300 lines (<count>)" \
  --description="File: <file>
Lines: <count>
Reviewer: architectural-drift
Fix: Split into focused submodules under <file_without_ext>/ directory.
CRITICAL: After creating the module directory and moving code, you MUST delete the original <file> file. Leaving both file.rs and file/mod.rs causes E0761 ambiguous module error and breaks the build.
Steps: 1) mkdir <dir> 2) split code into focused modules 3) create mod.rs 4) DELETE the original file 5) run 'moon run :build' to verify" \
  --type=task -p 1 --json
```

### 2. Black Hat Reviewer (`black-hat-reviewer`)

**When to invoke:** Phase 2 (branch review) and Phase 4 (active scan)

**What it checks:**
- Contract parity — types match their usage, no phantom fields
- Farley constraints — code is continuously deployable
- Holzman Rust — NASA/JPL discipline, zero-panic paths, bounded mutation, performance proof
- Security — OWASP top 10, command injection, XSS, SQL injection
- Unsafe usage — any `unsafe` blocks need justification
- Credential leaks — secrets in code, .env committed, hardcoded keys
- Error handling — no `unwrap()` in production paths, proper `?` propagation

**How to invoke:** Pass the diff to the skill with context about what changed.

**Bead creation:**
```bash
bd create --title="black-hat: <severity> — <concise issue>" \
  --description="File: <file:line>\nReviewer: black-hat-reviewer\nSeverity: <P0|P1|P2|P3>\nIssue: <what's wrong>\nFix: <what to do>\nCategory: <security|quality|holzman-rust|ux|docs|style|observation>" \
  --type=bug -p <0-3> --json
```

### 3. Red Queen (`red-queen`)

**When to invoke:** Phase 2 (branch review) and Phase 4 (active scan)

**What it checks:**
- Test coverage for new code paths
- Test quality — adversarial vs happy-path
- Property-based testing gaps (proptest, Kani)
- Boundary value testing
- Error path testing
- Concurrency testing where applicable

**How to invoke:** Pass the diff + list of test files to the skill.

**Bead creation:**
Create or link one bead for every finding from this reviewer, including weak assertions, minor hygiene, and observations.
```bash
bd create --title="red-queen: missing test coverage for <module>" \
  --description="File: <file:line>\nReviewer: red-queen\nModule: <module>\nIssue: <what's untested or weak>\nFix: Write tests or record the deterministic check/disposition covering <specific cases>\nPriority: P1 if new code has zero tests, P2 if tests are weak, P3 for lower-severity hygiene or observation" \
  --type=task -p <1-3> --json
```

### 4. Hands-on QA (`hands-on-qa`)

**When to invoke:** Phase 2 (branch review) and Phase 4 (active scan)

**What it actually runs:**
```bash
# Build check
moon run :build 2>&1

# CLI commands
cargo run -p vo-cli -- history --json 2>&1
cargo run -p vo-cli -- check 2>&1

# Frontend build (if frontend files changed)
cd crates/vo-frontend && dx build 2>&1

# Type check
moon run :check 2>&1
```

**Bead creation:**
Create or link one bead for every finding from this reviewer, including minor UX/output/help observations.
```bash
bd create --title="qa: <tool> <failure-type>" \
  --description="Tool: <tool>\nCommand: <exact command>\nOutput: <relevant output>\nReviewer: hands-on-qa\nFix: <what needs to happen>" \
  --type=bug -p <0-3> --json
```

### 5. Architecture Design QA (`arch-design-qa`)

**When to invoke:** Phase 2 (branch review) and Phase 4 (active scan)

**What it checks:**
- V2 spec alignment (see `docs/adr/v2/`)
- Domain boundary enforcement — code in correct crate?
- AI-native compliance — JSON CLI output, strict schemas
- Type-driven design — illegal states unrepresentable
- Double Diamond — discover, define, develop, deliver

**Bead creation:**
```bash
bd create --title="arch-qa: <violation type>" \
  --description="File: <file:line>\nReviewer: arch-design-qa\nIssue: <what violates spec>\nSpec ref: <docs/adr/v2/xxx>\nFix: <how to comply>" \
  --type=bug -p <1-2> --json
```

## Batch Bead Creation

When creating multiple beads, group related findings:

```bash
# Create all findings from a single reviewer in one batch. Do not filter by severity.
for finding in "${FINDINGS[@]}"; do
  bd create --title="$finding" \
    --description="Branch: $BRANCH\nReviewer: $reviewer\nFile: $file_line\nIssue: $issue\nFix: $fix\nDisposition: $disposition" \
    --type=bug -p "$priority" --json
done
```

Always link back to the branch being reviewed:
```bash
bd create --title="<reviewer>: <concise finding>" \
  --description="Branch: $BRANCH\nReviewer: <reviewer>\nFile: <file:line>\nIssue: <issue>\nFix: <fix>\nDisposition: <fixed_with_evidence|owner_approved_debt|owner_approved_no_action|blocker>" \
  --deps discovered-from:<parent-bead> --json
```
