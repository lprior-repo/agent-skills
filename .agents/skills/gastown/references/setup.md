# Gas Town Setup & Configuration Guide

## Table of Contents
1. [Initial Workspace Setup](#initial-workspace-setup)
2. [Adding a Rig with OpenCode](#adding-a-rig-with-opencode)
3. [Adding a Rig with Claude Code](#adding-a-rig-with-claude-code)
4. [Agent Runtime Configuration](#agent-runtime-configuration)
5. [Per-Rig Settings](#per-rig-settings)
6. [Custom Agent Presets](#custom-agent-presets)
7. [Scheduler Configuration](#scheduler-configuration)
8. [Dolt Data Plane](#dolt-data-plane)
9. [Hooks Configuration](#hooks-configuration)

---

## Initial Workspace Setup

```bash
# 1. Ensure mise is installed (https://mise.jdx.dev)
mise --version

# 2. Build and install gt + bd via mise (from gastown source)
cd ~/src/gastown
mise run install          # Builds gt via Makefile + syncs bd to go.mod version

# 3. Verify alignment
mise run check-versions   # Confirms bd matches go.mod beads version

# 4. Create workspace with git initialization
gt install ~/gt --git
cd ~/gt

# 5. Verify installation
gt doctor

# 6. Boot all services
mise run up               # mkdir -p ~/.beads-wisp/config && gt up --restore

# 7. Start the Mayor (your primary interface)
gt mayor attach
```

### Mise Tasks (~/src/gastown/mise.toml)

| Task | Description |
|------|-------------|
| `mise run sync` | Reinstall bd to match go.mod beads version |
| `mise run install` | Build gt via Makefile (proper ldflags) + sync bd |
| `mise run safe-install` | Replace gt binary without daemon restart |
| `mise run rebuild` | Full rebuild + gt doctor --fix |
| `mise run doctor` | Run gt doctor from workspace root |
| `mise run check-versions` | Verify bd matches go.mod beads version |
| `mise run up` | Boot all services with restore |
| `mise run start` | Start Gas Town from workspace root |

**Important:** Always use `mise run install` instead of `go build` or `go install` to build gt.
The Makefile embeds version/commit/build-time ldflags that prevent version drift between gt and bd.

The workspace creates this structure:
```
~/gt/
├── .beads/         Town-level beads (hq-* prefix, mail)
├── mayor/          Mayor config (town.json, rigs.json, daemon.json)
├── deacon/         Deacon daemon + dogs
└── <rig>/          One directory per project
```

---

## Adding a Rig with OpenCode

OpenCode is a built-in agent preset. This is the recommended setup for rigs where you want OpenCode as the primary runtime.

```bash
# 1. Add the rig
gt rig add Seshat https://github.com/lprior-repo/Seshat.git

# 2. Set OpenCode as the runtime for this rig
gt rig settings set Seshat runtime.provider opencode
gt rig settings set Seshat runtime.command opencode

# 3. Boot the rig (starts witness + refinery)
gt rig boot Seshat

# 4. Verify
gt rig status Seshat
# Should show: Status: OPERATIONAL, Witness: running, Refinery: running

# 5. Create crew workspace for yourself
gt crew add lewis --rig Seshat
```

### What happens on boot:
- Witness starts monitoring polecats for this rig
- Refinery starts the merge queue processor
- `.beads/` directory is initialized for issue tracking
- Git worktrees are set up for the rig

---

## Adding a Rig with Claude Code

```bash
gt rig add hardline https://github.com/lprior-repo/hardline.git
# Claude is the default runtime, no settings change needed
gt rig boot hardline
gt rig status hardline
```

---

## Agent Runtime Configuration

### Listing Available Presets
```bash
gt config agent list
```

Returns 10 built-in presets: claude, opencode, codex, copilot, cursor, auggie, amp, gemini, omp, pi

### Per-Rig Runtime Override
```bash
# Set OpenCode for a specific rig
gt rig settings set <rig> runtime.provider opencode
gt rig settings set <rig> runtime.command opencode

# View current settings
gt rig settings show <rig>
```

### Sling Override (One-Time)
```bash
# Use a different agent for one sling without changing rig settings
gt sling <bead-id> <rig> --agent opencode
gt sling <bead-id> <rig> --agent codex
```

### Codex-Specific Setup
For Codex agents, set the project doc fallback so role instructions are picked up:
```toml
# ~/.codex/config.toml
project_doc_fallback_filenames = ["CLAUDE.md"]
```

### Copilot-Specific Setup
- Requires a Copilot seat
- Requires org-level CLI policy
- Uses executable lifecycle hooks in `.github/hooks/gastown.json`
- Uses a 5-second ready delay instead of prompt detection

---

## Per-Rig Settings

Settings are stored in `<rig>/settings/config.json`.

```bash
# View all settings
gt rig settings show <rig>

# Set a value (dot notation for nested keys)
gt rig settings set <rig> namepool.style mad-max
gt rig settings set <rig> runtime.provider opencode
gt rig settings set <rig> merge.strategy bisect

# Remove a setting
gt rig settings unset <rig> runtime.provider
```

Common settings:
```json
{
  "runtime": {
    "provider": "opencode",
    "command": "opencode"
  },
  "namepool": {
    "style": "mad-max"
  },
  "merge": {
    "strategy": "bisect"
  },
  "scheduler": {
    "max_polecats": 5
  }
}
```

---

## Custom Agent Presets

```bash
# Register a custom agent command
gt config agent set claude-glm "claude-glm --model glm-4"
gt config agent set codex-low "codex --thinking low"

# Set as default
gt config default-agent claude-glm

# Use in sling
gt sling <bead-id> <rig> --agent claude-glm
```

---

## Scheduler Configuration

The scheduler controls how many polecats can run concurrently to prevent API rate limit exhaustion.

```bash
# Default: no limit (immediate dispatch)
gt config set scheduler.max_polecats -1

# Enable capacity governor (max 5 concurrent)
gt config set scheduler.max_polecats 5

# Check status
gt scheduler status

# Pause/resume dispatch
gt scheduler pause
gt scheduler resume
```

When a limit is set, the daemon dispatches incrementally, respecting capacity. Without a limit, `gt sling` dispatches immediately.

---

## Dolt Data Plane

Dolt is the data plane for beads (issues, mail, identity, work history). It runs on port 3307. **It is fragile.**

### Health Check
```bash
gt dolt status    # Server health, latency, orphan count
```

### Lifecycle
```bash
gt dolt start     # Start server
gt dolt stop      # Stop server
gt dolt cleanup   # Remove orphan test databases (safe)
```

### If Dolt Is Having Trouble

Symptoms: `bd` commands hang/timeout, "connection refused", latency > 5s.

**BEFORE restarting, capture diagnostics:**
```bash
# 1. Goroutine dump (safe — doesn't kill process)
kill -QUIT $(cat ~/gt/.dolt-data/dolt.pid)

# 2. Capture status while still broken
gt dolt status 2>&1 | tee /tmp/dolt-hang-$(date +%s).log

# 3. Escalate with evidence
gt escalate -s HIGH "Dolt: <describe symptom>"
```

**NEVER:**
- `rm -rf` on `~/.dolt-data/` directories — use `gt dolt cleanup`
- Restart Dolt without diagnostics — you destroy evidence of the hang
- Ignore orphan databases — they accumulate and degrade performance

### Test Pollution
Orphan databases (testdb_*, beads_t*, beads_pt*, doctest_*) accumulate on the production server:
```bash
gt dolt status    # Check orphan count
gt dolt cleanup   # Safe removal
```

---

## Hooks Configuration

Hooks are git worktree-based persistent storage. Claude Code uses hooks in `.claude/settings.json`. Other runtimes use their own hook mechanisms.

### Communication Hygiene
- `gt mail send` creates a permanent bead + Dolt commit — use for handoffs, escalations, structured protocol messages
- `gt nudge` creates nothing — use for routine agent-to-agent communication
- **Default to nudge** for most communication. Only use mail when the message must survive session death.

### Hook Lifecycle
1. `gt sling <bead> <rig>` — work placed on hook (git worktree)
2. Agent reads hook on startup → executes work
3. `gt done` — signals completion, pushes branch
4. Refinery processes merge

The hook IS the assignment. When an agent finds work on their hook, they execute immediately without waiting for confirmation. This is the Propulsion Principle.
