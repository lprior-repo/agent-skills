---
name: gastown
description: "Gas Town multi-agent orchestration expert for workspace setup, rig management, agent runtime configuration (claude, opencode, codex, cursor, amp, gemini, copilot, auggie, pi, omp), bead/issue tracking with bd, convoy work tracking, polecat lifecycle, witness/refinery/deacon monitoring, formula workflows, molecules, mail/nudge communication, scheduling, escalation, dashboard/feed monitoring, and cross-rig coordination. Use this skill whenever the user mentions Gas Town, gt commands, rigs, polecats, convoys, beads issues, slinging work, multi-agent coordination, agent orchestration, or any gt CLI operation — even if they don't explicitly say 'Gas Town'."
argument-hint: "[command or topic] — e.g. 'setup new rig', 'sling work to opencode', 'configure Seshat rig', 'convoy create', 'troubleshoot stuck agent'"
allowed-tools:
  - Read
  - Bash
  - Glob
  - Grep
  - Write
  - Edit
---

# Gas Town — Multi-Agent Orchestration Skill

Gas Town coordinates multiple AI coding agents with persistent work tracking via git-backed hooks and Beads issue tracking. It solves three problems: agents lose context on restart, manual coordination is chaotic at scale, and work state vanishes when sessions die.

The Propulsion Principle: if work is on your hook, YOU RUN IT. No confirmation. No waiting. The hook IS your assignment.

## Progressive Disclosure

Read these reference files when context signals the need:
- `references/setup.md` — Full setup workflows (new rig, OpenCode, per-rig runtime)
- `references/command-reference.md` — Complete command catalog with all flags
- `references/workflows.md` — Workflow patterns (Mountain-Eater, polecat lifecycle, cross-rig, formulas)
- `references/troubleshooting.md` — Recovery procedures (Dolt, stuck agents, emergency)

**Related skills (auto-invoked by context):**
- **dolt** — Invoke when `bd` commands fail ("no database selected", "unknown url scheme", "embeddeddolt"), when setting up new rig beads, or fixing dolt remotes. The dolt skill handles metadata.json, dolt remote config, and the shared Dolt server on port 3307.
- `references/troubleshooting.md` — Recovery procedures (Dolt, stuck agents, emergency)

## Context Recovery

```bash
gt prime                    # ALWAYS run after compaction/clear/new session
gt hook                     # Check for hooked work
gt mail inbox               # Check for messages
```

## Architecture Quick Map

```
Town (~/gt/)
├── mayor/          — Primary AI coordinator (singleton)
├── <rig>/          — Project containers
│   ├── mayor/rig/  — Mayor's working clone
│   ├── crew/<you>/ — Personal workspace
│   ├── witness/    — Per-rig health monitor
│   ├── refinery/   — Merge queue processor
│   ├── .beads/     — Issue tracking (Dolt-backed)
│   └── polecats/   — Ephemeral workers
├── daemon.json     — Daemon configuration
└── town.json       — Town-level config
```

## Agent Runtimes (10 Built-In Presets)

| Preset | Command |
|--------|---------|
| claude (default) | `claude --dangerously-skip-permissions` |
| opencode | `opencode` |
| codex | `codex --dangerously-bypass-approvals-and-sandbox` |
| copilot | `copilot --yolo` |
| cursor | `cursor-agent -f` |
| auggie | `auggie --allow-indexing` |
| amp | `amp --dangerously-allow-all --no-ide` |
| gemini | `gemini --approval-mode yolo` |
| omp | `omp --hook .omp/hooks/gastown-hook.ts` |
| pi | `pi -e .pi/extensions/gastown-hooks.js` |

## OpenCode Model Registry (CRITICAL)

**NEVER use short model names** — they fall back to Gemini and show "quota exhausted" errors.
Always use the FULL model identifier from the table below:

| Agent Alias | OpenCode Model Flag | Provider |
|-------------|-------------------|----------|
| opencode-minimax | `-m minimax-coding-plan/MiniMax-M2.7-highspeed` | MiniMax Cloud |
| opencode-glm51 | `-m zai-coding-plan/glm-5.1` | Z.AI Cloud |
| opencode-glm5t | `-m zai-coding-plan/glm-5-turbo` | Z.AI Cloud |
| opencode-qwen5090 | `-m qwen36-5090/Qwen3.6-35B-A3B-UD-Q5_K_XL.gguf` | Local GPU (127.0.0.1:11000) |
| opencode-qwen3090 | `-m qwen36-3090/Qwen3.6-35B-A3B-UD-Q4_K_XL.gguf` | Local GPU (127.0.0.1:11001) |

### Veloxide Fleet (20 Polecats)

| Polecat | Runtime | Model Flag |
|---------|---------|-----------|
| brahmin, chrome, dust, fury, ghoul, guzzle, mirelurk, mutant, nitro, raider | opencode-minimax | `minimax-coding-plan/MiniMax-M2.7-highspeed` |
| nuka, pipboy | opencode-glm51 | `zai-coding-plan/glm-5.1` |
| radrat, scavenger | opencode-glm5t | `zai-coding-plan/glm-5-turbo` |
| rust, deathclaw | claude opus | `claude --model opus --dangerously-skip-permissions` |
| shiny, synth, thunder | claude sonnet | `claude --model sonnet --dangerously-skip-permissions` |
| vault | opencode-qwen5090 | `qwen36-5090/Qwen3.6-35B-A3B-UD-Q5_K_XL.gguf` |
| gecko | opencode-qwen3090 | `qwen36-3090/Qwen3.6-35B-A3B-UD-Q4_K_XL.gguf` |

**Note**: Claude CLI uses positional prompt arg (NO `--prompt` flag). OpenCode uses `--prompt` flag.

## Essential Commands

### Workspace & Rigs
```bash
gt install ~/gt --git              # Initialize workspace
gt rig add <name> <repo-url>       # Add project
gt rig boot <rig>                  # Start witness + refinery
gt rig list                        # List rigs
gt rig status <rig>                # Detailed status
gt crew add <name> --rig <rig>     # Create crew workspace
```

### Agent Configuration
```bash
gt config agent list               # List presets
gt config agent set <alias> "<cmd>" # Custom agent
gt config default-agent <alias>    # Set default
gt rig settings set <rig> runtime.provider opencode  # Per-rig runtime
gt rig settings show <rig>         # View settings
```

### Beads (Issue Tracking)
```bash
bd create --title="..." --description="..." --type=task --priority 0
bd ready                           # Unblocked work
bd list --status=open              # All open
bd show <id>                       # Details
bd update <id> --claim             # Claim work
bd close <id1> <id2> ...           # Close completed
bd search <query>                  # Search
bd dep add <blocked-id> <blocker-id>  # blocked-id depends on blocker-id
```

Priority: P0–P4 (0=critical, 4=backlog). NOT "high/medium/low".
Types: task, bug, feature, epic, chore, decision.

### Slinging Work
```bash
gt sling <bead-id> <rig>           # Auto-spawn polecat
gt sling <bead-id> <rig>/<name>    # Specific polecat
gt sling <bead-id> <rig> --agent opencode  # Override runtime
gt sling <bead-id> <rig> --merge=direct    # Push to main
gt sling <bead-id> <rig> --merge=mr        # Merge queue (default)
gt sling <bead-id> <rig> --merge=local     # Keep on branch
gt sling <id1> <id2> <id3> <rig> --max-concurrent 3  # Batch
```

### Convoys
```bash
gt convoy create "Name" <beads...>  # Create convoy
gt convoy add <id> <beads...>       # Add work
gt convoy stage <id>                # Compute dependency waves
gt convoy launch <id>               # Dispatch Wave 1
gt convoy land <id>                 # Cleanup + close
gt convoy stranded                  # Find stuck convoys
gt convoy list                      # Dashboard view
```

### Mountain-Eater (Epic Grinding)
```bash
gt mountain <epic-id>               # Activate autonomous epic grinding
gt mountain status                  # Check progress
gt mountain pause / resume          # Control
```

### Molecules & Formulas
```bash
gt formula list                     # 47+ built-in formulas
gt formula show <name>              # View steps
gt mol current                      # What to work on
gt mol step done                    # Complete current step
gt mol progress                     # Execution status
```

Key formulas: `mol-polecat-work`, `mol-idea-to-plan`, `mol-witness-patrol`, `mol-refinery-patrol`, `shiny`, `tdd-cycle`

### Communication & Monitoring
```bash
gt mail inbox                       # Messages (persistent beads)
gt mail send <role> "msg"           # Send message
gt nudge <agent> "msg"              # Ephemeral nudge (no record — DEFAULT)
gt feed                             # Real-time activity TUI
gt feed --problems                  # Stuck agent detection
gt dashboard                        # Web dashboard (port 8080)
gt agents                           # List active agents
gt doctor                           # Health checks
```

Default to nudge for routine communication. Only use mail when the message MUST survive session death.

### Escalation & Recovery
```bash
gt escalate -s high "description"   # Escalate blocker
gt escalate list                    # Open escalations
gt seance                           # Previous sessions
gt seance --talk <id> -p "?"        # Query predecessor
gt warrant <agent>                  # Kill stuck agent
gt release <bead-id>                # Release bead to pending
gt estop                            # Emergency stop — freeze all work
gt thaw                             # Resume after estop
```

Severities: critical, high, medium, low.

### Service Lifecycle
```bash
gt up                               # Start all services
gt down                             # Stop all services
gt shutdown                         # Graceful shutdown
gt daemon start / stop              # Manage daemon
```

### Scheduler
```bash
gt config set scheduler.max_polecats 5  # Capacity governor
gt scheduler status
gt scheduler pause
gt scheduler resume
```

### Dolt (Data Plane)
```bash
gt dolt status / start / stop / cleanup
```

**Before restarting Dolt** — see `references/troubleshooting.md` for the full diagnostic protocol.

## Role Taxonomy

| Role | Lifecycle | Purpose |
|------|-----------|---------|
| Mayor | Singleton, persistent | Cross-rig coordinator |
| Witness | Per-rig, persistent | Polecat health monitor |
| Refinery | Per-rig, persistent | Merge queue processor |
| Deacon | Singleton, persistent | Cross-rig watchdog, dispatches Dogs |
| Polecat | Transient | Ephemeral worker (Witness-managed) |
| Crew | Persistent | Human workspace (user-managed) |
| Dog | Ephemeral | Deacon infrastructure helper (NOT project work) |

## Polecat vs Crew

| Aspect | Crew | Polecat |
|--------|------|---------|
| Lifecycle | Persistent | Transient |
| Monitoring | None | Witness watches/nudges |
| Assignment | Self-directed | `gt sling` |
| Merge | Pushes directly | Branch → Refinery or `--merge=direct` |
| Cleanup | Manual | Automatic |

### Direct Polecat Session Spawning

`gt sling` creates scheduled work (wisps) but does NOT start tmux sessions.
The witness processes wisps one at a time (~3 min each), which is too slow for
20+ polecats. To bypass the witness bottleneck, start sessions directly:

```bash
for p in <idle-polecats>; do
  SESSION="ve-$p"
  CLONE="/home/lewis/gt/veloxide/polecats/$p/veloxide"
  tmux has-session -t "$SESSION" 2>/dev/null && continue
  cp -n /home/lewis/gt/veloxide/mayor/rig/.beads/metadata.json "$CLONE/.beads/metadata.json" 2>/dev/null
  tmux new-session -d -s "$SESSION" -c "$CLONE" \
    "export GT_BRANCH=$(cd $CLONE && git branch --show-current) GT_POLECAT=$p GT_POLECAT_PATH=$CLONE GT_RIG=veloxide GT_ROLE=veloxide/polecats/$p GT_TOWN_ROOT=/home/lewis/gt BD_ACTOR=veloxide/polecats/$p BD_DOLT_AUTO_COMMIT=off BEADS_AGENT_NAME=veloxide/$p BEADS_DOLT_PORT=3307 GT_DOLT_PORT=3307 GT_AGENT=<runtime> GT_PROCESS_NAMES=opencode,node,bun OPENCODE_PERMISSION='{\"*\":\"allow\"}' && cd $CLONE && opencode -m <model> --prompt '[GAS TOWN] polecat $p (rig: veloxide). Run gt prime --hook and begin work.'"
done
```

### Direct Merge (--merge=direct)

Use `--merge=direct` to have polecats push straight to main, bypassing the
refinery merge queue entirely. This eliminates the refinery bottleneck:

```bash
gt sling <bead-id> veloxide --agent <runtime> --create --merge=direct
```

## Autonomous Fleet Feed Protocol

The Mayor runs a cron every 3 minutes to keep the full fleet fed and pushing to main.
When the cron triggers, execute the full protocol below with NO confirmation.

### Fleet Status Check

```bash
for p in brahmin chrome dust fury ghoul guzzle mirelurk nitro nuka pipboy radrat scavenger vault mutant raider synth thunder deathclaw rust shiny gecko; do
  PID=$(tmux list-panes -t ve-$p -F '#{pane_pid}' 2>/dev/null)
  CHILDREN=$(pgrep -P $PID 2>/dev/null | head -1)
  if [ -n "$CHILDREN" ]; then echo "$p: WORKING"; else echo "$p: IDLE"; fi
done
```

### Ready Beads

```bash
cd /home/lewis/src/veloxide && bd ready -n 20 --plain 2>&1
```

### Polecat Restart Protocol (for each idle/dead polecat)

**CRITICAL**: Always do these three steps before restarting:

```bash
# 1. Remove stale lock (ROOT CAUSE of gt prime exit code 1)
rm -f "/home/lewis/gt/veloxide/polecats/$p/veloxide/.runtime/agent.lock"

# 2. Copy correct metadata from source DB (NOT GT rig DB which uses stale port 3308)
cp -n /home/lewis/src/veloxide/.beads/metadata.json "/home/lewis/gt/veloxide/polecats/$p/veloxide/.beads/metadata.json" 2>/dev/null

# 3. Get current branch
BRANCH=$(cd "/home/lewis/gt/veloxide/polecats/$p/veloxide" && git branch --show-current)
```

Then launch with the correct model for the polecat's runtime:

**MiniMax** (brahmin, chrome, dust, fury, ghoul, guzzle, mirelurk, mutant, nitro, raider):
```bash
tmux new-session -d -s "ve-$p" -c "/home/lewis/gt/veloxide/polecats/$p/veloxide" \
  "export GT_BRANCH=$BRANCH GT_POLECAT=$p GT_POLECAT_PATH=/home/lewis/gt/veloxide/polecats/$p/veloxide GT_RIG=veloxide GT_ROLE=veloxide/polecats/$p GT_TOWN_ROOT=/home/lewis/gt BD_ACTOR=veloxide/polecats/$p BD_DOLT_AUTO_COMMIT=off BEADS_AGENT_NAME=veloxide/$p BEADS_DOLT_PORT=3307 GT_DOLT_PORT=3307 GT_AGENT=opencode-minimax GT_PROCESS_NAMES=opencode,node,bun OPENCODE_PERMISSION='{\"*\":\"allow\"}' && cd /home/lewis/gt/veloxide/polecats/$p/veloxide && git checkout main && git pull origin main && gt agents fix -a 2>/dev/null; rm -f .runtime/agent.lock && opencode -m minimax-coding-plan/MiniMax-M2.7-highspeed --prompt \"[GAS TOWN] polecat $p (rig: veloxide). Claim bead \$BEAD. Run bd update \$BEAD --claim. Then gt prime --hook and begin work. AFTER completing your bead: git add -A && git commit -m 'polecat/$p: completed \$BEAD' && git push origin HEAD:main --force-with-lease. This is MANDATORY — always push to main.\""
```

**GLM-5.1** (nuka, pipboy): Same template but `GT_AGENT=opencode-glm51` and `-m zai-coding-plan/glm-5.1`

**GLM-5T** (radrat, scavenger): Same template but `GT_AGENT=opencode-glm5t` and `-m zai-coding-plan/glm-5-turbo`

**Claude Opus** (rust, deathclaw): `GT_AGENT=claude GT_PROCESS_NAMES=claude`, prompt is **positional arg** (NO `--prompt` flag):
```bash
tmux new-session -d -s "ve-$p" -c "/home/lewis/gt/veloxide/polecats/$p/veloxide" \
  "export GT_BRANCH=$BRANCH GT_POLECAT=$p ... GT_AGENT=claude GT_PROCESS_NAMES=claude && cd /home/lewis/gt/veloxide/polecats/$p/veloxide && claude --model opus --dangerously-skip-permissions \"[GAS TOWN] polecat $p ...\""
```

**Claude Sonnet** (shiny, synth, thunder): Same but `--model sonnet` and `GT_AGENT=claude-sonnet`

**Qwen-5090** (vault): `GT_AGENT=opencode-qwen5090` and `-m qwen36-5090/Qwen3.6-35B-A3B-UD-Q5_K_XL.gguf`

### Cron Cleanup

```bash
cd /home/lewis/src/veloxide && for b in $(bd search "sling-context" 2>&1 | grep -oP 've-wisp-\w+'); do bd close "$b" --reason="Cron recycle" 2>/dev/null; done
```

### Deacon Escalation Handling

Recurring false positive from stale-agent-dog. Ack and close:
```bash
gt mail mark-read <mail-id> && gt escalate close <escalation-id> --reason "Deacon alive on gt-d27447 socket. Stale heartbeat alert. Recurring false positive."
```

### Critical Rules

1. **Stale locks kill polecats** — The launch command now auto-runs `gt agents fix -a && rm -f .runtime/agent.lock` before starting OpenCode. This prevents `gt prime` exit code 1. If a polecat still fails, manually clean the lock.
2. **Two beads databases** — Source DB at `~/src/veloxide` (port 3307 default) is authoritative. GT rig DB at port 3308 is stale. Always `cd /home/lewis/src/veloxide` for `bd` commands.
3. **OpenCode uses `--prompt` flag. Claude CLI uses positional arg** (NO `--prompt`).
4. **NEVER use short model names** like `qwen-5090` or `glm-5.1` — they fall back to Gemini with quota exhaustion.
5. **Qwen GPU endpoints**: 5090 at `127.0.0.1:11000`, 3090 at `127.0.0.1:11001`.
6. **Always push to main** (merge=direct).

## Common Workflows

Read `references/workflows.md` for detailed step-by-step guides for:
- Mountain-Eater staged convoy pattern
- Polecat lifecycle (birth → work → merge → death)
- Cross-rig work (worktrees vs dispatch)
- Formula execution and custom formula creation
- Stuck agent recovery (nudge → handoff → warrant → escalate)

Read `references/setup.md` for:
- New rig setup with OpenCode or other agent runtimes
- Per-rig runtime configuration
- Crew workspace creation

## Mandatory Verification Gate

After any Gas Town configuration or setup change:
```bash
gt doctor                          # Health checks must pass
gt rig status <rig>                # Must show OPERATIONAL
gt agents                          # Expected agents running
```

## Anti-Hallucination

- NEVER fabricate bead IDs — only use IDs from `bd` command output
- NEVER skip `gt doctor` after configuration changes
- NEVER restart Dolt without capturing diagnostics first
- NEVER use `rm -rf` on `.dolt-data/` directories — use `gt dolt cleanup`
- NEVER use `gt hooks repair` — use `gt hooks sync` instead
- NEVER confuse Dogs (infrastructure) with Polecats (project work)
