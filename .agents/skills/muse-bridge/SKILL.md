---
name: muse-bridge
description: "Run the installed Muse Code CLI from Codex for an explicitly authorized, bounded review request. Use when the user specifically asks Codex to invoke Muse; do not launch it for ordinary coding tasks."
---

# Muse bridge

Use the repository's canonical helper at `scripts/muse-bridge` (relative to this skill directory). Do not call Muse directly or turn a prompt-only “read-only” request into a claimed security boundary.

## Before invoking

- Require current-task authorization to send the prompt to Muse, and an explicit workspace path. Never default to the current repository or share the whole repository implicitly; prefer a narrow directory containing only the files the user wants Muse to inspect.
- Use review-only behavior. The helper pins the locally verified `muse-spark-1.3` model with Contributor mode, uses low reasoning effort, disables shell, workspace writes, web tools, and unrelated personal context, keeps Muse sandboxing on, disables session-log persistence, and leaves approval mode at `on-request`.
- The helper checks that the installed CLI still resolves `muse-spark-1.3` before sending a prompt. It removes `META_API_KEY` from the child environment so Muse uses its existing account login; it never copies credential contents. If account login is missing or rejected, stop and report that. Never initiate login or fall back to an API key without separate user authorization.
- Use a workspace outside the system temp directory; the installed Muse runtime reserves TMPDIR for its process-lifetime output. The prompt file is capped at 16 KiB. Each Muse output stream is capped at 256 KiB, tool output fed back to the model at 16 KiB, model steps at 8, and wall time at 180 seconds by default. The user may choose smaller bounds. The helper saves stdout JSONL, stderr, model-profile diagnostics, and a status record in a private output directory outside the workspace.
- The requested workspace is available to Muse's read tools. The CLI safety flags are the actual read-only controls; do not rely on wording in the prompt alone.

## Invocation

Prepare a short prompt file containing only the requested task and approved context, then run:

```sh
/path/to/this/skill/scripts/muse-bridge \
  --workspace /explicit/narrow/path \
  --prompt-file /path/to/prompt.txt
```

The helper prints the output directory and a stable status. Inspect `status.json`, `stdout.jsonl`, and `stderr.log`; report the helper and Muse exit statuses without treating silence as success. Never include credentials or unrelated private data in the prompt.
