# RTK Checklist

## Preflight

- Confirm the user's target tool: OpenCode, Claude Code, Codex, Cursor, or another supported agent.
- Check whether `mise` is available if the user did not request a different installer.
- Check whether `rtk` already exists.
- Verify identity with `rtk gain` before deciding to reinstall.

## Install

- Use the user-requested installer, otherwise prefer `mise`.
- Do not assume PATH activation; verify the actual binary path.
- If `mise` installed RTK but `rtk` is still unresolved, run `mise reshim rtk`.
- If the installed `rtk` fails `gain`, treat it as the wrong package and replace it with the correct one.

## Init

- Run the exact `rtk init ...` command for the chosen target.
- Capture the created hook or plugin path.
- Tell the user if a restart is required.

## Ship Gate

- `rtk --version` succeeded.
- `rtk gain` succeeded.
- `rtk init --show` reflects the intended integration state.
- Final report includes exact commands, exact paths, and any remaining restart step.
