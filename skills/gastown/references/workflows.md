# Gas Town Workflows

## Table of Contents
1. [Mayor Workflow (Recommended)](#mayor-workflow)
2. [Polecat Lifecycle](#polecat-lifecycle)
3. [Convoy Mountain-Eater Pattern](#convoy-mountain-eater-pattern)
4. [Formula Execution](#formula-execution)
5. [Stuck Agent Recovery](#stuck-agent-recovery)
6. [Cross-Rig Work](#cross-rig-work)
7. [Session Continuity](#session-continuity)
8. [Escalation Protocol](#escalation-protocol)
9. [Model A/B Testing](#model-ab-testing)

---

## Mayor Workflow

The Mayor is the entry point for all work coordination.

```
User → Mayor → bd create → gt sling → polecat executes → gt done → refinery merges
```

### Step by Step

1. **Tell the Mayor** what you want to accomplish
2. **Mayor analyzes** — breaks into tasks, creates beads
3. **Convoy creation** — `gt convoy create "Feature X" <bead-ids...>`
4. **Agent spawning** — `gt sling <bead-id> <rig>` auto-spawns polecat
5. **Work distribution** — beads slung to agents via hooks
6. **Progress monitoring** — `gt feed`, `gt convoy list`, `gt dashboard`
7. **Completion** — polecat runs `gt done`, refinery processes merge

### Example: Adding a Feature to Seshat

```bash
# 1. Create bead for the work
bd create --title="Implement snap-to-grid" \
  --description="Add grid snapping for node positions. See SNP-001 to SNP-010." \
  --type=feature --priority=2

# Output: ✓ Created issue: se-x4k2

# 2. Create convoy (optional but recommended)
gt convoy create "Snap-to-Grid" se-x4k2 --notify

# 3. Sling to Seshat rig (auto-spawns polecat with opencode)
gt sling se-x4k2 Seshat

# 4. Monitor
gt convoy list
gt feed

# 5. When polecat finishes, refinery processes merge
# Check merge status
gt convoy status <convoy-id>
```

---

## Polecat Lifecycle

Polecats are ephemeral workers with persistent identity. The Witness manages their lifecycle.

### Birth
```bash
gt sling <bead-id> <rig>
# → Creates polecat worktree
# → Assigns bead to hook
# → Spawns agent session (tmux)
# → Witness begins monitoring
```

### Life
```
1. Agent reads hook → finds work assignment
2. Executes work in isolated worktree
3. Makes commits with attributed identity
4. Runs verification (tests, CI)
```

### Death (Completion)
```bash
gt done
# → Pushes branch
# → Creates MR bead
# → Refinery adds to merge queue
# → Polecat session ends
# → Worktree cleaned up (after merge)
```

### Witness Monitoring
The Witness continuously patrols its polecats:
- Checks for GUPP violations (hooked work, no progress)
- Detects stalled agents (reduced progress)
- Identifies zombies (dead tmux sessions)
- Triggers recovery: nudge → handoff → warrant

---

## Convoy Mountain-Eater Pattern

For epic-scale work (many beads), use the staged convoy pattern.

```bash
# 1. Create convoy with ALL beads
gt convoy create "Seshat MVP" \
  se-gjd se-3nm se-nso se-zxv se-cdi se-5od se-dcz \
  se-xvv se-7l5 se-60z se-e1d se-cmu se-iad se-1ol se-7za

# 2. Stage: analyze dependencies, compute dispatch waves
gt convoy stage <id>
# Output: Wave 1 (no blockers): se-gjd, se-3nm, se-7l5
#         Wave 2 (needs Wave 1): se-nso, se-60z, se-e1d
#         Wave 3 (needs Wave 2): ...

# 3. Launch Wave 1
gt convoy launch <id>
# → Dispatches all Wave 1 beads in parallel

# 4. Subsequent waves auto-dispatch as predecessors complete
# Witness + Deacon monitor progress

# 5. Land when all complete
gt convoy land <id>
```

### Smart Skip Logic
If a bead in Wave 2 fails, Wave 3+ is NOT blocked. Work that doesn't depend on the failed bead gets dispatched immediately.

### Capacity Governor
```bash
# Limit concurrent polecats
gt config set scheduler.max_polecats 5
gt scheduler status
```

---

## Formula Execution

Formulas are reusable TOML-defined workflows. 47+ built-in.

### List and Inspect
```bash
gt formula list              # All 47+ formulas
gt formula show mol-polecat-work  # View steps
```

### Execute
```bash
# Method 1: Direct run
gt formula run mol-polecat-work --var bead=se-abc12

# Method 2: Via beads
bd cook mol-polecat-work --var bead=se-abc12

# Method 3: Create trackable molecule instance
bd mol pour mol-polecat-work --var bead=se-abc12
```

### Key Formulas

| Formula | Purpose |
|---------|---------|
| `mol-polecat-work` | Full polecat lifecycle |
| `mol-idea-to-plan` | Vague idea → approved plan |
| `mol-witness-patrol` | Per-rig polecat monitor loop |
| `mol-refinery-patrol` | Merge queue processor loop |
| `mol-deacon-patrol` | Mayor's daemon patrol |
| `shiny` | Engineer in a Box: design → implement → review |
| `tdd-cycle` | Replaces step with red/green/refactor |
| `security-audit` | Cross-cutting security scanning |
| `mol-dog-doctor` | Probe Dolt health |
| `mol-dog-backup` | Sync Dolt backups |
| `mol-dog-compactor` | Compact Dolt history |
| `mol-dog-reaper` | Reap stale wisps + issues |
| `code-review` | Parallel specialized reviewers |
| `design` | Structured design exploration |

### Custom Formulas
Create in `internal/formula/formulas/` or `~/.gt/formulas/`:

```toml
description = "My custom workflow"
formula = "my-workflow"
version = 1

[vars.input]
description = "What to process"
required = true

[[steps]]
id = "step-1"
title = "First step"
description = "Run ./scripts/step1.sh {{input}}"

[[steps]]
id = "step-2"
title = "Second step"
description = "Run ./scripts/step2.sh"
needs = ["step-1"]
```

---

## Stuck Agent Recovery

### Detection
```bash
gt feed --problems              # Problems view in TUI
gt convoy stranded             # Find stuck convoys
gt agents                      # Check agent status
```

### Recovery Levels (Escalating)

**Level 1: Nudge** — Check if agent responds
```bash
gt nudge <agent> "Status? Any blockers?"
```

**Level 2: Handoff** — Refresh context, continue work
```bash
gt handoff <agent>
# Fresh session picks up from hook
```

**Level 3: Warrant** — Kill stuck agent, release work
```bash
gt warrant <agent>
gt release <bead-id>
# Bead goes back to pending, can be re-slung
```

**Level 4: Escalate** — Notify Mayor/Overseer
```bash
gt escalate -s HIGH "Agent <name> stuck on <bead-id>"
```

---

## Cross-Rig Work

### Using Worktrees (Preferred)
```bash
# You're in gastown, need to fix something in Seshat
gt worktree Seshat
# Creates ~/gt/Seshat/crew/gastown-<your-name>/
# Identity preserved: BD_ACTOR = gastown/crew/<name>

# Work normally in the worktree
cd ~/gt/Seshat/crew/gastown-<name>/

# When done, work appears on YOUR CV, not Seshat's
```

### Using Dispatch
```bash
# Create issue in target rig
bd create --prefix se "Fix auth bug in Seshat"
# Output: se-xyz12

# Dispatch to Seshat's workers
gt sling se-xyz12 Seshat
# Seshat's witness manages the polecat
```

---

## Session Continuity

### Seance: Query Previous Sessions
```bash
gt seance                           # List discoverable sessions
gt seance --talk <id>               # Full conversation
gt seance --talk <id> -p "Why?"     # One-shot question
```

Sessions discovered via `.events.jsonl` logs. Enables context recovery without re-reading entire codebases.

### Handoff: Clean Session Transfer
```bash
gt handoff              # Create handoff message, end session
                         # Fresh session reads handoff on startup
```

### Checkpoint: Crash Recovery
```bash
gt checkpoint           # Save current state
gt checkpoint list      # View checkpoints
gt checkpoint restore   # Recover from crash
```

---

## Escalation Protocol

### Severity Levels
| Level | Code | Route | Response Time |
|-------|------|-------|---------------|
| CRITICAL | P0 | Deacon → Mayor → Overseer | Immediate |
| HIGH | P1 | Deacon → Mayor | Within session |
| MEDIUM | P2 | Deacon queues | Next patrol cycle |

### When to Escalate
- Agent hits blocker it can't resolve
- Dolt server hangs (ALWAYS capture diagnostics first)
- Convoy stuck with no progress
- Infrastructure failure (tmux, git, network)

### Example
```bash
# Agent hits Dolt hang
kill -QUIT $(cat ~/gt/.dolt-data/dolt.pid)
gt dolt status 2>&1 | tee /tmp/dolt-hang-$(date +%s).log
gt escalate -s HIGH "Dolt: queries timing out, goroutine dump captured"
```

---

## Model A/B Testing

Gas Town's attribution system enables objective model comparison.

```bash
# Dispatch same task type to different models
gt sling se-abc Seshat --agent opencode
gt sling se-def Seshat --agent claude

# Compare outcomes after completion
gt audit --actor=Seshat/polecats/* --group-by=model
gt changelog --rig Seshat
```

### Metrics Available
- Completion time per bead
- Quality signals (pass/fail CI, review grade)
- Revision count (commits per bead)
- Token usage per session

### Common Comparisons
- Claude vs OpenCode for Rust code
- GPT vs Claude for test writing
- Small model vs large model for routine tasks
