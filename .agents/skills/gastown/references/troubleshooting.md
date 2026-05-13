# Gas Town Troubleshooting Reference

## Table of Contents
1. [Dolt Server Problems](#dolt-server-problems)
2. [Stuck Agents](#stuck-agents)
3. [Hook Problems](#hook-problems)
4. [Merge Queue Failures](#merge-queue-failures)
5. [Session Issues](#session-issues)
6. [Permission Errors](#permission-errors)
7. [Convoy Problems](#convoy-problems)
8. [Emergency Protocols](#emergency-protocols)
9. [Wasteland Federation](#wasteland-federation)
10. [Telemetry](#telemetry)
11. [Common Gotchas](#common-gotchas)
12. [Environment Variables](#environment-variables)

---

## Dolt Server Problems

### Symptoms
- `bd` commands hang/timeout
- "connection refused" errors
- "database not found" errors
- Query latency > 5 seconds
- Unexpected empty results from `bd list`

### Protocol (CRITICAL — follow exactly)

**Step 1: Capture goroutine dump** (safe — does NOT kill process)
```bash
kill -QUIT $(cat ~/gt/.dolt-data/dolt.pid)
```

**Step 2: Capture status while still broken**
```bash
gt dolt status 2>&1 | tee /tmp/dolt-hang-$(date +%s).log
```

**Step 3: Escalate with evidence**
```bash
gt escalate -s high "Dolt: <describe symptom>"
# For total outage:
gt escalate -s critical "Dolt: server unreachable"
```

**Step 4: THEN restart**
```bash
gt dolt stop
gt dolt start
gt dolt status   # Verify recovery
```

### NEVER
- `rm -rf ~/.dolt-data/` — use `gt dolt cleanup`
- Restart without diagnostics — destroys evidence
- Ignore orphan databases — they accumulate and degrade performance

### Test Pollution
Orphan databases (testdb_*, beads_t*, beads_pt*, doctest_*) accumulate on the production server.
```bash
gt dolt status    # Check orphan count
gt dolt cleanup   # Remove orphan databases (safe — protects production DBs)
```

---

## Stuck Agents

### Detection
```bash
gt feed --problems              # Problems view in TUI
gt agents                       # Check agent status
gt convoy stranded              # Find stranded convoys
```

### Recovery Levels (Escalating)

**Level 1: Nudge** — check responsiveness
```bash
gt nudge <agent> "Status? Any blockers?"
```

**Level 2: Handoff** — refresh context, continue work
```bash
gt handoff <agent>
# Fresh session picks up from hook
```

**Level 3: Warrant** — kill stuck agent, release work
```bash
gt warrant <agent>
gt release <bead-id>
# Bead returns to pending, can be re-slung
```

**Level 4: Escalate** — notify hierarchy
```bash
gt escalate -s high "<agent> stuck on <bead-id>"
```

**Emergency stop** (freeze ALL work):
```bash
gt estop                        # Freeze everything
gt thaw                         # Resume after estop
```

---

## Hook Problems

### Diagnosis
```bash
gt hooks list                   # Check hooks
gt hooks sync                   # Sync/fix hooks (NOT "gt hooks repair")
```

### Common Issues
- Missing hook: `gt hooks sync`, then re-sling
- Stale hook: `gt release <bead-id>`, then re-sling
- Permission denied: `chmod 700 <rig>/mayor/rig/.beads`
- beads.role warning: `git config beads.role maintainer` or `git config beads.role contributor`

---

## Merge Queue Failures

The Refinery processes in a Bors-style bisecting queue.

### Diagnosis
```bash
gt refinery status <rig>        # Check merge queue
gt convoy stranded              # Find stuck convoys
```

### How Refinery Handles Failures
1. Batches pending MRs
2. Runs verification gates on merged stack
3. If green: all MRs in batch merge to main
4. If red: bisects to isolate failing MR
5. Good MRs merge, failing MR gets isolated
6. Failed MRs: re-dispatched or escalated

### Merge Strategies
```bash
--merge=mr       # Merge queue via Refinery (default)
--merge=direct   # Push directly to main
--merge=local    # Keep on feature branch (no merge)
```

---

## Session Issues

### After compaction/clear/new session
```bash
gt prime                        # ALWAYS run first — reload context
gt hook                         # Check for hooked work
gt mail inbox                   # Check for messages
```

### After crash
```bash
gt prime                        # Reload context
gt hook                         # Check hooked work
gt mail inbox                   # Check messages
gt seance                       # Find predecessor sessions
gt seance --talk <id>           # Get context from previous session
```

### Agent loses connection
```bash
gt hooks sync                   # Fix hooks (NOT "gt hooks repair")
gt mayor detach                  # Stop mayor
gt mayor attach                  # Restart
```

---

## Permission Errors

### .beads directory permissions
```bash
chmod 700 <rig>/mayor/rig/.beads
```

### beads.role not configured
```bash
cd <rig>
git config beads.role maintainer
# Or: git config beads.role contributor
```

### Git authentication
```bash
gh auth status                  # Check GitHub auth
gh auth login                   # Re-authenticate if needed
```

---

## Convoy Problems

### Stranded Convoy
```bash
gt convoy stranded              # Find stuck convoys
gt convoy status <id>           # Check details
gt convoy close <id> --force    # Force close even with open items
```

### Orphaned Work
```bash
gt orphans                      # Find lost polecat work
```

---

## Emergency Protocols

### Full Town Restart
```bash
gt shutdown                     # Graceful shutdown
gt doctor                       # Health checks
gt dolt cleanup                 # Remove orphan databases
```

### Nuclear Recovery
```bash
gt estop                        # Freeze all work first
gt doctor                       # Full health check
gt dolt stop                    # Stop Dolt
gt dolt cleanup                 # Remove orphans
gt dolt start                   # Restart Dolt
gt up                           # Start all services
gt doctor                       # Verify recovery
```

---

## Wasteland Federation

```bash
gt wl join <remote>             # Join a wasteland
gt wl browse                    # View wanted board
gt wl claim <id>                # Claim work
gt wl done <id> --evidence <url>  # Submit completion
gt wl post --title "Need X"    # Post new wanted item
```

Cross-town work coordination via DoltHub. Completions earn portable reputation via multi-dimensional stamps (quality, speed, complexity).

---

## Telemetry

```bash
export GT_OTEL_LOGS_URL="http://localhost:9428/insert/jsonline"
export GT_OTEL_METRICS_URL="http://localhost:8428/api/v1/write"
```

Events: session lifecycle, agent state changes, bd calls, mail operations, sling/nudge/done workflows, polecat spawn/remove, formula instantiation, convoy creation, daemon restarts.

Metrics: `gastown.session.starts.total`, `gastown.bd.calls.total`, `gastown.polecat.spawns.total`, `gastown.done.total`, `gastown.convoy.creates.total`.

---

## Common Gotchas

- **Dolt is fragile** — ALWAYS capture goroutine dump BEFORE restarting
- **NEVER** use `rm -rf` on `.dolt-data/` directories — use `gt dolt cleanup`
- **Dogs are NOT workers** — they're Deacon infrastructure helpers. Use polecats for project work.
- **Default to nudge** for routine communication. Only use mail when persistence is required.
- **Priority is 0-4** — NOT "high/medium/low"
- **Use `gt hooks sync`** — NOT "gt hooks repair" (that command doesn't exist)
- **`--merge=local`** keeps work isolated on a feature branch. Use for work that needs review before merging.
- **Cross-rig work** — use worktrees (`gt worktree <rig>`) for quick fixes. Use dispatch (`gt sling <bead> <rig>`) when the target rig should own the work.
- **Severities are lowercase**: critical, high, medium, low (NOT uppercase)
- **Bead types**: task, bug, feature, epic, chore, decision (not just "task/bug/feature")

---

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `GT_ROLE` | Current agent role (injected by `gt prime`) |
| `GT_RIG` | Current rig name |
| `GT_WORKSPACE` | Town root path |
| `BD_ACTOR` | Identity for beads attribution |
| `RUSTC_WRAPPER` | Must be unset for Dioxus (`env -u RUSTC_WRAPPER`) |
| `GT_OTEL_LOGS_URL` | OTLP logs endpoint |
| `GT_OTEL_METRICS_URL` | OTLP metrics endpoint |
