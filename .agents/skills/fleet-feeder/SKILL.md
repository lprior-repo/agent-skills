---
name: fleet-feeder
description: >
  Fleet feeding automation for Gas Town's 28-polecat Veloxide fleet. Wraps the compiled
  Rust binary that checks polecat status, recovers stale beads, assigns ready work,
  prepares worktrees, and launches tmux sessions with correct runtimes. Use this skill
  when: running or debugging fleet-feed, checking fleet status, restarting idle polecats,
  rebuilding the fleet-feed binary, managing the cron loop, or anytime someone says
  "fleet feed", "feed the fleet", "fleet-feed", "fleet status", "check polecats",
  "restart polecats", or "cron loop". Also trigger when discussing polecat lifecycle,
  work dispatch at scale, or fleet automation.
allowed-tools:
  - Read
  - Bash
  - Glob
  - Grep
---

# Fleet Feeder — Automated Polecat Fleet Management

The fleet-feeder is a compiled Rust binary that automates the full feed cycle for
Veloxide's 28-polecat fleet. One run = check all polecats, recover stale work, assign
new beads, launch sessions.

## Quick Start

```bash
# Run one feed cycle
/home/lewis/gt/fleet-feed/target/release/fleet-feed

# Start persistent cron loop (runs every 3 minutes)
tmux new-session -d -s "fleet-feed-cron" \
  "while true; do /home/lewis/gt/fleet-feed/target/release/fleet-feed 2>&1 | tee -a /home/lewis/gt/.fleet-feed.log; sleep 180; done"

# Check cron status
tmux has-session -t fleet-feed-cron 2>/dev/null && echo "RUNNING" || echo "STOPPED"

# Stop cron
tmux kill-session -t fleet-feed-cron 2>/dev/null
```

## Paths

| What | Path |
|------|------|
| **Source code** | `/home/lewis/src/gastown-feeder/` |
| **Binary (symlink)** | `/home/lewis/gt/fleet-feed/target/release/fleet-feed` |
| **Binary (real)** | `/home/lewis/src/gastown-feeder/target/release/fleet-feed` |
| **Cron log** | `/home/lewis/gt/.fleet-feed.log` |
| **Cron tmux session** | `fleet-feed-cron` |

## Rebuilding

After any source change, rebuild and test:

```bash
cd /home/lewis/src/gastown-feeder
cargo build --release && cargo test
```

## The Fleet (28 Polecats)

| Runtime | Count | Names | Model Flag | Agent Flag |
|---------|-------|-------|------------|------------|
| MiniMax | 10 | brahmin, chrome, dust, fury, ghoul, guzzle, mirelurk, mutant, raider, nitro | `minimax-coding-plan/MiniMax-M2.7-highspeed` | `opencode-minimax` |
| Qwen-5090 | 5 | vault, thunder, nuka, pipboy, radrat | `qwen36-5090/Qwen3.6-35B-A3B-UD-Q5_K_XL.gguf` | `opencode-qwen5090` |
| Qwen-3090 | 4 | gecko, lancer, scavenger, bandit | `qwen36-3090/Qwen3.6-35B-A3B-UD-Q4_K_XL.gguf` | `opencode-qwen3090` |
| Claude Opus | 3 | rust, deathclaw, behemoth | `opus` | `claude` |
| Claude Sonnet | 3 | shiny, synth, sentinel | `sonnet` | `claude-sonnet` |
| Claude Haiku | 3 | moira, stahl, braun | `haiku` | `claude-haiku` |

- 19 OpenCode-based (MiniMax + Qwen) + 9 Claude CLI-based = 28 total
- Tmux sessions named `ve-{name}` (e.g. `ve-brahmin`, `ve-rust`)
- Worktrees at `/home/lewis/gt/veloxide/polecats/{name}/veloxide`

## What One Cycle Does

The `run_fleet_feed()` function executes this sequence:

1. **Ensure Dolt alive** — checks `gt dolt status`, auto-restarts if dead, waits 3s
2. **Check all 28 polecats** — tmux session exists? child process running? Classifies as Working/Idle/Dead
3. **Recover stale beads** — for each idle/dead polecat, releases their `in_progress` beads back to `open`
4. **Fetch ready beads** — runs `bd ready -n 50 --json` from `/home/lewis/src/veloxide`
5. **Feed idle/dead polecats** — for each: find unassigned bead, assign it, prepare worktree, launch tmux session
6. **Stagger launches** — 2-second sleep between each feed to let Dolt recover

## Manual Fleet Status Check

```bash
for p in brahmin chrome dust fury ghoul guzzle mirelurk mutant raider nitro vault thunder nuka pipboy radrat gecko lancer scavenger bandit rust deathclaw behemoth shiny synth sentinel moira stahl braun; do
  PID=$(tmux list-panes -t ve-$p -F '#{pane_pid}' 2>/dev/null)
  CHILDREN=$(pgrep -P $PID 2>/dev/null | head -1)
  if [ -n "$CHILDREN" ]; then echo "$p: WORKING"; else echo "$p: IDLE"; fi
done
```

## Manual Polecat Restart

If fleet-feed can't handle it, restart a polecat manually:

```bash
p="brahmin"  # replace with target
CLONE="/home/lewis/gt/veloxide/polecats/$p/veloxide"

# Clean up
rm -f "$CLONE/.runtime/agent.lock"
cp -n /home/lewis/src/veloxide/.beads/metadata.json "$CLONE/.beads/metadata.json" 2>/dev/null
BRANCH=$(cd "$CLONE" && git branch --show-current)

# Launch (MiniMax example — adjust model for other runtimes)
tmux new-session -d -s "ve-$p" -c "$CLONE" \
  "export GT_BRANCH=$BRANCH GT_POLECAT=$p GT_POLECAT_PATH=$CLONE GT_RIG=veloxide GT_ROLE=veloxide/polecats/$p GT_TOWN_ROOT=/home/lewis/gt BD_ACTOR=veloxide/polecats/$p BD_DOLT_AUTO_COMMIT=off BEADS_AGENT_NAME=veloxide/$p BEADS_DOLT_PORT=3307 GT_DOLT_PORT=3307 GT_AGENT=opencode-minimax GT_PROCESS_NAMES=opencode,node,bun OPENCODE_PERMISSION='{\"*\":\"allow\"}' && cd $CLONE && git checkout main && git pull origin main && gt agents fix -a 2>/dev/null; rm -f .runtime/agent.lock && opencode -m minimax-coding-plan/MiniMax-M2.7-highspeed --prompt '[GAS TOWN] polecat $p (rig: veloxide). Claim bead. Run bd update BEAD --claim. Then gt prime --hook and begin work. AFTER completing: git add -A && git commit -m \"polecat/$p: completed\" && git push origin HEAD:main --force-with-lease.'"
```

### Runtime-Specific Launch Commands

**MiniMax** (brahmin, chrome, dust, fury, ghoul, guzzle, mirelurk, mutant, raider, nitro):
- `GT_AGENT=opencode-minimax`
- `opencode -m minimax-coding-plan/MiniMax-M2.7-highspeed --prompt "..."`

**Qwen-5090** (vault, thunder, nuka, pipboy, radrat):
- `GT_AGENT=opencode-qwen5090`
- `opencode -m qwen36-5090/Qwen3.6-35B-A3B-UD-Q5_K_XL.gguf --prompt "..."`

**Qwen-3090** (gecko, lancer, scavenger, bandit):
- `GT_AGENT=opencode-qwen3090`
- `opencode -m qwen36-3090/Qwen3.6-35B-A3B-UD-Q4_K_XL.gguf --prompt "..."`

**Claude Opus** (rust, deathclaw, behemoth):
- `GT_AGENT=claude GT_PROCESS_NAMES=claude`
- `claude --model opus --dangerously-skip-permissions "..."` (positional arg, NO `--prompt` flag)

**Claude Sonnet** (shiny, synth, sentinel):
- `GT_AGENT=claude-sonnet GT_PROCESS_NAMES=claude`
- `claude --model sonnet --dangerously-skip-permissions "..."` (positional arg, NO `--prompt` flag)

**Claude Haiku** (moira, stahl, braun):
- `GT_AGENT=claude-haiku GT_PROCESS_NAMES=claude`
- `claude --model haiku --dangerously-skip-permissions "..."` (positional arg, NO `--prompt` flag)

## Source Architecture

The codebase follows strict Data/Calculations/Actions separation:

| Module | Role | I/O? |
|--------|------|------|
| `data.rs` | Domain types (PolecatName, BeadId, RuntimeSpec, FleetEntry), fleet catalog (`Fleet::all()`), constants, error types | Pure |
| `calculations.rs` | Pure functions: prompt building, env var construction, launch command generation, status classification | Pure |
| `actions.rs` | Async I/O: dolt health checks, tmux commands, bd operations, git operations. `run_fleet_feed()` is the orchestrator | Side effects |
| `branch_landing.rs` | Cross-rig branch merging (6 repos: veloxide, hardline, twerk, seshat, cdocs, clarity) | Side effects |
| `scheduling.rs` | Multi-rig scheduling with proportional quotas and cross-rig work stealing (planned, not yet wired) | Pure |

## Critical Rules

1. **NEVER use short model names** — falls back to Gemini with quota exhaustion. Always use the full identifier.
2. **Claude CLI uses positional arg for prompt** — NO `--prompt` flag. OpenCode uses `--prompt`.
3. **Stale locks kill polecats** — always clean `.runtime/agent.lock` before launch.
4. **Source DB is authoritative** — `bd` commands must run from `/home/lewis/src/veloxide` (port 3307).
5. **Two-second stagger** — launches are staggered to prevent Dolt lock contention.
6. **Rebuild after source changes** — `cd /home/lewis/src/gastown-feeder && cargo build --release && cargo test`.
