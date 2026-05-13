# RTK Reference

## Install Matrix

Use the lightest correct installer the user asked for.

| Method | When to use | Commands |
|---|---|---|
| `mise` | Preferred when available | `mise use -g rtk@latest` then `mise exec -- rtk gain` |
| Cargo (git) | User explicitly wants Cargo or source-based install | `cargo install --git https://github.com/rtk-ai/rtk` then `rtk gain` |
| Script | User explicitly wants the official install script | `curl -fsSL https://raw.githubusercontent.com/rtk-ai/rtk/master/install.sh | sh` then `rtk gain` |

## Identity Checks

Use all of these when diagnosing an existing install:

```bash
command -v rtk
which rtk
rtk --version
rtk gain
mise which rtk
mise current rtk
mise reshim rtk
```

## Init Matrix

| Target | Command | Notes |
|---|---|---|
| OpenCode | `rtk init -g --opencode` | Installs `~/.config/opencode/plugins/rtk.ts` |
| Claude Code | `rtk init -g` | Installs hook and may patch `~/.claude/settings.json` |
| Claude Code, minimal | `rtk init -g --hook-only` | Hook only, no `RTK.md` |
| Codex | `rtk init -g --codex` | Configures Codex global instructions |
| Cursor | `rtk init -g --agent cursor` | Installs Cursor agent hooks |

## Fast Checks

```bash
rtk init --show
ls ~/.config/opencode/plugins
ls ~/.claude/hooks
```

## Notes

- `rtk gain` is the high-signal verification command because it proves the installed binary is Rust Token Killer.
- If `mise` has installed RTK but `command -v rtk` still fails, run `mise reshim rtk` and re-check.
- OpenCode plugin installation still requires an OpenCode restart before live rewrites occur.
- Built-in `Read`, `Grep`, and `Glob` tools are not rewritten by RTK. Only bash/shell command execution is affected.
