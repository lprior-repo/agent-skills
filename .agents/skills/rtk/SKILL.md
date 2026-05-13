---
name: rtk
description: >
  Install, verify, initialize, and diagnose Rust Token Killer (rtk-ai/rtk)
  for OpenCode, Claude Code, Codex, Cursor, and other supported agents. Use
  when the user wants RTK installed, wired into an AI tool, or checked for the
  correct package and working rewrite hooks.
disable-model-invocation: true
argument-hint: '[install|verify|init|diagnose] [--opencode|--claude|--codex|--agent <name>]'
allowed-tools:
  - Bash
  - Read
  - Grep
  - Glob
---

```jsonl
{"kind":"meta","skill":"rtk","version":"1.0.0","format":"jsonl-progressive","mode":"manual-install-and-setup"}
{"kind":"mission","goal":"Install the correct RTK, verify it is Rust Token Killer rather than the unrelated rtk package, initialize the right agent integration, and prove the final state with real command output."}
{"kind":"rule","id":"mise_first","text":"Prefer `mise` when it is available and the user has not requested another install method. Use the registry-backed `rtk` tool entry rather than crates.io guesswork."}
{"kind":"rule","id":"reshim_if_missing","text":"If `mise` reports RTK installed but `command -v rtk` still fails, run `mise reshim rtk` before diagnosing PATH or plugin issues."}
{"kind":"rule","id":"name_collision","text":"Always guard against the package-name collision. `rtk --version` alone is insufficient. You MUST verify with `rtk gain` after installation."}
{"kind":"rule","id":"agent_specific_init","text":"Choose init mode based on the actual target tool: `rtk init -g --opencode` for OpenCode, `rtk init -g` for Claude Code, `rtk init -g --codex` for Codex, or the documented `--agent` variant for supported editors."}
{"kind":"rule","id":"anti_hallucination","text":"FORBIDDEN: inventing install status, hook status, plugin paths, or command output. Every claim about RTK state MUST come from executed commands."}
{"kind":"workflow","id":"verify_install_init","steps":["Discover current state: check whether `rtk` exists, how it was installed, and whether `mise` is available.","Verify the package identity with `rtk gain` or `mise exec -- rtk gain`.","Install RTK if missing or wrong, preferring `mise` unless the user requested a different method.","Initialize the correct agent integration for the current tool or the user's requested target.","Run post-setup verification commands and report exact paths, versions, and any remaining restart steps."]}
{"kind":"decision","id":"install_method","options":[{"mode":"mise","when":"`mise` exists and the user did not request another installer","commands":["mise registry | rg '^rtk\\s'","mise use -g rtk@latest","mise exec -- rtk --version","mise exec -- rtk gain"]},{"mode":"cargo_git","when":"user explicitly requests Cargo or mise is unavailable","commands":["cargo install --git https://github.com/rtk-ai/rtk","rtk --version","rtk gain"]},{"mode":"script","when":"user explicitly requests installer script or binary-style setup","commands":["curl -fsSL https://raw.githubusercontent.com/rtk-ai/rtk/master/install.sh | sh","rtk --version","rtk gain"]}]}
{"kind":"decision","id":"init_targets","options":[{"target":"OpenCode","command":"rtk init -g --opencode","verify":"rtk init --show"},{"target":"Claude Code","command":"rtk init -g","verify":"rtk init --show"},{"target":"Codex","command":"rtk init -g --codex","verify":"rtk init --show"},{"target":"Cursor","command":"rtk init -g --agent cursor","verify":"rtk init --show"}]}
{"kind":"output","id":"report_format","sections":["## State Before — whether rtk existed, install manager, current path/version if any","## Actions Taken — exact install/init commands executed","## Verification — exact version, `gain` result, and integration status","## Remaining User Step — restart or manual follow-up only if still required"]}
{"kind":"ref","file":"reference.md","use":"Install and init command matrix, common verification commands, and target-specific notes."}
{"kind":"ref","file":"checklist.md","use":"Preflight and ship checklist for RTK installation and integration work."}
```

# RTK Install And Setup

## Mandatory Verification Gate

Before you finish, run the commands that match the chosen install path and target agent.

```bash
# Identity check: prove this is Rust Token Killer
rtk --version
rtk gain

# If using mise, verify the managed binary too
mise which rtk
mise exec -- rtk --version
mise exec -- rtk gain

# Verify integration status after init
rtk init --show
```

For OpenCode, also confirm the plugin file exists:

```bash
ls ~/.config/opencode/plugins
readlink -f ~/.config/opencode/plugins/rtk.ts 2>/dev/null || true
```

## Anti-Hallucination Shield

Required:
- Report the exact installer used.
- Report the exact binary path used for verification.
- Quote real command output for version and `gain` checks.
- State when a restart is still needed instead of implying activation already happened.

Forbidden:
- Claiming RTK is correctly installed without a successful `gain` check.
- Claiming hooks or plugins are active without `rtk init --show` or a verified file path.
- Confusing `rtk-ai/rtk` with the unrelated crates.io package of the same name.
