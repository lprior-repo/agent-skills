# Gas Town Command Reference

Complete catalog of every `gt` and `bd` command with usage patterns.

## Table of Contents
1. [Workspace Management](#workspace-management)
2. [Rig Operations](#rig-operations)
3. [Agent Management](#agent-management)
4. [Beads (bd) Issue Tracking](#beads-issue-tracking)
5. [Sling — Work Dispatch](#sling--work-dispatch)
6. [Convoy — Work Tracking](#convoy--work-tracking)
7. [Molecules & Formulas](#molecules--formulas)
8. [Communication](#communication)
9. [Monitoring & Health](#monitoring--health)
10. [Scheduler](#scheduler)
11. [Escalation](#escalation)
12. [Dolt Data Plane](#dolt-data-plane)
13. [Session Management](#session-management)
14. [Identity & Attribution](#identity--attribution)
15. [Diagnostics](#diagnostics)

---

## Workspace Management

| Command | Description |
|---------|-------------|
| `gt install <path> --git` | Initialize workspace with git |
| `gt status` | Overall town status |
| `gt doctor` | Run health checks |
| `gt upgrade` | Post-install migration + sync |
| `gt uninstall` | Remove Gas Town |

---

## Rig Operations

| Command | Description |
|---------|-------------|
| `gt rig add <name> <repo-url>` | Add project |
| `gt rig list` | List all rigs |
| `gt rig status <rig>` | Detailed status |
| `gt rig boot <rig>` | Start witness + refinery |
| `gt rig stop <rig>` | Stop agents |
| `gt rig park <rig>` | Stop agents, prevent auto-restart |
| `gt rig unpark <rig>` | Allow auto-restart |
| `gt rig dock <rig>` | Global persistent shutdown |
| `gt rig undock <rig>` | Remove docked status |
| `gt rig remove <rig>` | Remove from registry (keeps files) |
| `gt rig reset <rig>` | Reset state (handoff, mail, stale issues) |
| `gt rig reboot <rig>` | Restart witness + refinery |
| `gt rig config <rig>` | View/manage rig configuration |
| `gt rig settings show <rig>` | View all settings |
| `gt rig settings set <rig> <key> <value>` | Set setting |
| `gt rig settings unset <rig> <key>` | Remove setting |
| `gt crew add <name> --rig <rig>` | Create crew workspace |

---

## Agent Management

| Command | Description |
|---------|-------------|
| `gt agents` | List active agent sessions |
| `gt config agent list` | List built-in presets |
| `gt config agent set <alias> "<cmd>"` | Register custom agent |
| `gt config default-agent <alias>` | Set default |
| `gt polecat list` | List polecats |
| `gt polecat spawn <name> --rig <rig>` | Spawn manually |
| `gt polecat remove <name>` | Remove polecat |
| `gt polecat status <name>` | Polecat details |
| `gt mayor attach` | Start/attach to Mayor |
| `gt mayor start --agent <alias>` | Mayor with specific agent |
| `gt mayor detach` | Stop Mayor |
| `gt witness start/stop <rig>` | Manage witness |
| `gt refinery start/stop <rig>` | Manage refinery |
| `gt deacon start/stop` | Manage Deacon |
| `gt role` | Show current role |

---

## Beads (bd) Issue Tracking

### CRUD
```bash
bd create --title="Summary" --description="Details" --type=task --priority 1
bd show <id>
bd update <id> --claim                    # Claim work
bd update <id> --title="New title"        # Update title
bd update <id> --description="New desc"   # Update description
bd update <id> --assignee=<name>          # Assign to someone
bd update <id> --notes="progress notes"   # Add notes
bd close <id1> <id2> <id3>               # Close multiple
bd close <id> --reason="explanation"      # Close with reason
```

### Querying
```bash
bd ready                    # Unblocked work (no active blockers)
bd list --status=open       # All open
bd list --status=in_progress # Active work
bd show <id>                # Full details
bd search <query>           # Search by keyword
bd stats                    # Open/closed/blocked counts
```

### Dependencies
```bash
bd dep add <issue> <depends-on>   # issue depends on depends-on
bd blocked                       # All blocked issues
bd show <id>                     # See what's blocking/blocked by
```

### Types and Priorities
- **Types**: task, bug, feature, epic, chore, decision
- **Priorities**: P0 (critical) → P4 (backlog). Use integers 0-4 or P0-P4. NOT "high/medium/low".
- **Bead IDs**: prefix + 5-char alphanumeric (e.g., se-gjd, gt-abc12, hq-x7k2m)

---

## Sling — Work Dispatch

`gt sling` is THE unified work dispatch command. It handles agent spawning, work assignment, and auto-convoy creation.

```bash
# Basic: auto-spawn polecat in rig
gt sling <bead-id> <rig>

# Specific polecat
gt sling <bead-id> <rig>/<polecat-name>

# Override runtime for this dispatch
gt sling <bead-id> <rig> --agent opencode

# Self (assign to current agent)
gt sling <bead-id>

# Crew worker in current rig
gt sling <bead-id> crew

# Merge strategies
gt sling <bead-id> <rig> --merge=direct   # Push to main
gt sling <bead-id> <rig> --merge=mr       # Merge queue (default)
gt sling <bead-id> <rig> --merge=local    # Keep on branch

# Skip auto-convoy creation
gt sling <bead-id> <rig> --no-convoy

# Remove work from hook
gt unsling <bead-id>
```

### What sling does:
1. Places work on agent's hook (git worktree)
2. Auto-creates convoy for tracking (unless --no-convoy)
3. Spawns polecat if target is a rig (auto-spawn)
4. Dispatches via scheduler if max_polecats is set

---

## Convoy — Work Tracking

Convoys track batches of related work across rigs and agents.

```bash
gt convoy create "Name" <bead-ids...>     # Create with issues
gt convoy create "Name" --notify --human  # With notifications
gt convoy add <convoy-id> <bead-ids...>   # Add more work
gt convoy list                            # Dashboard view
gt convoy status [id]                     # Progress details
gt convoy stage <id>                      # Compute dependency waves
gt convoy launch <id>                     # Dispatch Wave 1
gt convoy land <id>                       # Cleanup + close
gt convoy close <id>                      # Close convoy
gt convoy close <id> --force              # Force close
gt convoy check                           # Auto-close completed
gt convoy stranded                        # Find stuck convoys
gt convoy watch <id>                      # Subscribe to completion
gt convoy unwatch <id>                    # Unsubscribe
```

### Convoy semantics:
- Tracks bead IDs (cross-prefix capable)
- Swarm = ephemeral set of workers currently assigned
- Landed = all tracked issues closed → notification sent
- Can be reopened by adding more issues

### Staged convoys (Mountain-Eater pattern):
1. `gt convoy stage <id>` — analyzes dependencies, computes dispatch waves
2. `gt convoy launch <id>` — dispatches Wave 1
3. As Wave 1 lands, subsequent waves auto-dispatch

---

## Molecules & Formulas

### Molecules (running workflow instances)
```bash
gt mol current              # What to work on now
gt mol progress             # Execution progress
gt mol status               # Same as gt hook
gt mol step done            # Complete current step
gt mol attach               # Attach to hook
gt mol detach               # Detach from hook
gt mol burn                 # Discard (no record)
gt mol squash               # Compress to digest (permanent)
gt mol dag                  # Visualize dependency DAG
gt mol await-signal         # Wait for activity feed signal
gt mol attach-from-mail     # Attach from mail message
gt mol attachment           # Show attachment status
```

### Formulas (reusable templates — 47+ built-in)
```bash
gt formula list             # List all formulas
gt formula show <name>      # View steps and variables
gt formula run <name>       # Execute (pour + dispatch)
gt formula create           # Create new formula
```

Key formula categories:
- **Workflow**: mol-polecat-work, mol-idea-to-plan, shiny, tdd-cycle
- **Patrol**: mol-witness-patrol, mol-refinery-patrol, mol-deacon-patrol
- **Infrastructure**: mol-dog-doctor, mol-dog-backup, mol-dog-compactor
- **Convoy**: mol-convoy-feed, mol-convoy-cleanup, code-review, design
- **Expansion**: rule-of-five, tdd-cycle, security-audit
- **Testing**: towers-of-hanoi (3/7/9/10 disk variants)

### Creating custom formulas
Formulas are TOML files in `internal/formula/formulas/`:
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

## Communication

### Mail (persistent — survives session death)
```bash
gt mail inbox               # Check messages
gt mail send <role> "msg"   # Send (creates bead + Dolt commit)
```

**Use mail for**: handoffs, structured protocol messages, escalations, anything that MUST survive the recipient's session.

### Nudge (ephemeral — no record)
```bash
gt nudge <agent> "msg"      # Transient message
```

**Use nudge for**: routine agent-to-agent communication, status checks, quick coordination.

**Rule: default to nudge.** Only mail when persistence is required.

### Other
```bash
gt broadcast "msg"          # Nudge all workers
gt dnd                      # Toggle Do Not Disturb
```

---

## Monitoring & Health

### Real-Time Feed
```bash
gt feed                     # Interactive TUI dashboard
gt feed --problems          # Start in problems view
gt feed --plain             # Text output (no TUI)
gt feed --window            # Dedicated tmux window
gt feed --since 1h          # Events from last hour
```

Navigation: j/k scroll, Tab switch panels, 1/2/3 jump panels, ? help, q quit.

### Problems View (stuck agent detection)
| State | Condition |
|-------|-----------|
| GUPP Violation | Hooked work, no progress for extended period |
| Stalled | Hooked work, reduced progress |
| Zombie | Dead tmux session |
| Working | Active, progressing normally |
| Idle | No hooked work |

Intervention keys: n=nudge, h=handoff.

### Dashboard (Web)
```bash
gt dashboard                # Port 8080
gt dashboard --port 3000    # Custom port
gt dashboard --open         # Auto-open browser
```

### Agent Tree
```bash
gt agents                   # List all active sessions
gt trail                    # Recent activity
gt vitals                   # Unified health dashboard
gt health                   # Comprehensive system health
```

---

## Scheduler

```bash
gt config set scheduler.max_polecats 5   # Enable governor
gt scheduler status                      # Current state
gt scheduler pause                       # Pause dispatch
gt scheduler resume                      # Resume dispatch
```

Default (-1): immediate dispatch. With a limit: daemon dispatches incrementally respecting capacity.

---

## Escalation

```bash
gt escalate -s CRITICAL "total outage"   # P0: Mayor + Overseer
gt escalate -s HIGH "blocker desc"       # P1: Mayor
gt escalate -s MEDIUM "issue desc"       # P2: Queued
gt escalate list                         # Open escalations
gt escalate ack <bead-id>                # Acknowledge
```

Routing: Deacon → Mayor → Overseer based on severity.

---

## Dolt Data Plane

```bash
gt dolt status              # Health + latency + orphan count
gt dolt start               # Start server
gt dolt stop                # Stop server
gt dolt cleanup             # Remove orphan databases
```

### Emergency Protocol
```bash
# 1. Capture goroutine dump (safe, doesn't kill process)
kill -QUIT $(cat ~/gt/.dolt-data/dolt.pid)

# 2. Capture status while still broken
gt dolt status 2>&1 | tee /tmp/dolt-hang-$(date +%s).log

# 3. Escalate with evidence
gt escalate -s HIGH "Dolt: <describe symptom>"
```

**NEVER** restart Dolt without diagnostics. **NEVER** `rm -rf` on `.dolt-data/` — use `gt dolt cleanup`.

---

## Session Management

```bash
gt hook                     # Show hooked work
gt handoff                  # Hand off to fresh session
gt resume                   # Check for handoff messages
gt seance                   # List predecessor sessions
gt seance --talk <id>       # Full conversation
gt seance --talk <id> -p "?" # One-shot question
gt checkpoint               # Crash recovery
gt session list/manage      # Polecat session management
```

### Recovery after compaction/clear/new session
```bash
gt prime                    # Reload role context
gt hook                     # Check for work
gt mail inbox               # Check messages
```

---

## Identity & Attribution

```bash
gt whoami                   # Current identity
gt role                     # Show/manage role
gt commit                   # Git commit with agent identity
```

All work is attributed: git commits, beads issues, events. Identity persists even cross-rig.

---

## Diagnostics

```bash
gt doctor                   # Health checks
gt stale                    # Check if binary is stale
gt audit                    # Query work history by actor
gt changelog                # Completed work across rigs
gt costs                    # Claude session costs
gt metrics                  # Command usage statistics
gt log                      # Town activity log
gt info                     # Version + what's new
gt activity                 # Emit/view activity events
```

### Cleanup
```bash
gt cleanup                  # Orphan Claude processes
gt release <bead-id>        # Release bead back to pending
gt warrant <agent>          # Kill stuck agent
gt prune-branches           # Remove stale tracking branches
gt compact                  # Compact expired wisps
```

### Repair
```bash
gt hooks list               # Check hooks
gt hooks sync               # Sync/fix hooks (NOT "gt hooks repair")
gt repair                   # Fix database/config issues
```
