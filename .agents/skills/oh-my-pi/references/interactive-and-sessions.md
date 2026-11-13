# Interactive use, sessions, and context lifecycle

Baseline: installed OMP 18.19.0, checked 2026-11-12. Commands are grouped by surface. Use the current CLI help and `/hotkeys` rather than assuming another Pi fork or an old tutorial has the same controls.

## Start in the intended workspace

**Terminal:**

```bash
omp --cwd /path/to/repo
omp --cwd /path/to/repo @docs/requirements.md 'Explain the affected behavior before editing.'
omp --cwd /path/to/repo --add-dir /path/to/shared
omp --profile work --cwd /path/to/repo
omp --cwd /path/to/repo --no-session
```

Replace paths; attachments can send private source/doc/image content to the selected model. `--add-dir` broadens workspace discovery; it is not an OS sandbox or an instruction to scan everything. `--profile` separates native settings/auth/sessions/caches; it does not isolate filesystem permissions or all foreign configuration sources. Starting in the home directory normally has special safety handling; `--allow-home` intentionally bypasses that behavior, not a recommended default.

`omp` can follow configured auto-resume behavior; do not promise a bare launch is always a fresh conversation. `/new` explicitly starts a new identity. In terminal automation, non-TTY stdin is treated as input; do not expect a TUI merely because the command was spelled `omp`.

## Give a contract, not an unbounded wish

A useful initial request names:

```text
Goal: implement the documented importer behavior.
Scope: src/importer and its existing consumer tests; preserve unrelated edits.
Contract: accepted inputs, outputs/errors, and public API compatibility.
Evidence: reuse the reported failure; run the narrow consumer scenario after the fix.
Constraints: no new service/dependency; ask only for a material tradeoff tools cannot answer.
Delivery: changed behavior, files, actual verification, remaining limits.
```

For a question, request explanation only. For a coding task, do not confuse explaining a plan with delivering a working change. Name the surface to verify (CLI output, browser interaction, API response, or targeted test).

## Composer controls (defaults, remappable)

Run **`/hotkeys`** for actual active application mappings; fixed editor controls, terminal forwarding, extensions, and Vim mode can affect what a chord does.

| Control | Purpose |
| --- | --- |
| Enter during active work | Submit steering: change the current direction at the next supported boundary |
| Ctrl+Enter / Ctrl+Q | Queue follow-up; when approval UI owns focus, these approve instead |
| Alt+Up / Shift+Up | Move a queued message back into the editor |
| Ctrl+R | Search prompt history |
| Ctrl+G | Edit draft in `$VISUAL` / `$EDITOR` |
| Ctrl+P / Shift+Ctrl+P | Cycle role/scoped model selections |
| Alt+P | Temporary model choice for the session |
| Alt+M | Full model/role selector |
| Shift+Tab | Cycle thinking level |
| Ctrl+T | Show/hide thinking blocks; not a reasoning-effort switch |
| Ctrl+O | Expand/collapse tool output |
| Ctrl+Shift+O | Show/hide tool activity |
| Alt+Shift+P | Toggle plan mode |
| F5 / Alt+R | Retry the last failed assistant turn deliberately |

Steering is not rollback and does not promise an already running command/model request was undone. A retry can duplicate side effects: inspect the observed failure/current state first. Escape can dismiss a popup, deny approval, interrupt work, or change Vim mode depending on focus; do not treat it as one universal action. Double Ctrl+C can exit. Do not send interruption/exit keystrokes to the user's active session merely to demonstrate them.

In the composer, slash commands are TUI commands; they are not shell commands. File attachments use `@path`. Local shell execution sigils are a separate user-execution surface; executing a command is still a real side effect. Terminal `omp bash`/`omp sh`/`omp zsh` expose OMP's shell engines for explicit shell use; agent `bash` has its own structured job/service contract.

## Common inspection commands (TUI)

- `/model`: choose active model; configured role selectors are supported where documented.
- `/settings`: human settings interface; changes can persist.
- `/context`: read-only snapshot of system prompt, loaded instructions/skill listings, tool definitions, and messages. Reopen to refresh. Do not publish it unredacted.
- `/stats`, `/trace`: usage and execution evidence.
- `/extensions`: inspect enabled/discovered executable modules and their status.
- `/local`: inspect/edit session-local artifacts via the configured editor.
- `/skill:oh-my-pi QUESTION`: inject this skill if skill commands are enabled.

For less common commands use the TUI command completion and `omp://slash-command-internals.md`; do not invent a plausible alias.

## Choose the correct continuation operation

| Operation | What it does | What it does not promise |
| --- | --- | --- |
| `omp --continue` | Opens terminal breadcrumb / most-recent applicable session; can create one if none exists | Selection of a specific ID or continuation of all interrupted work |
| `omp --resume` | Opens session picker | Recovery of unfinished child runs |
| `omp --resume ID_OR_PATH` | Opens the selected saved conversation | Filesystem/process rollback |
| `omp --restore ID_OR_PATH` | Resumes and continues recorded unfinished work/interrupted turn and subagents | Resurrection of arbitrary OS processes or eval variables |
| `/resume ...` | Switches the active conversation to another saved session | A new fork or reset of on-disk code |
| `/tree` | Moves current session's active leaf to an earlier entry; optional branch summary | New session file or restoring files |
| `/branch` (alias `/rewind`) | Transcript rewind selector; moves leaf **inside the same session file** | The programmatic `AgentSession.branch()` API, which has different semantics |
| `/fork` | Duplicates current session/artifacts into a separate session identity | Separate worktree/isolated code checkout |
| `/seance ID_OR_PATH` | Read-only consultant over an earlier session; main session stays active | Resume/modify that source session |

**Crash recovery terminal example:**

```bash
omp --cwd /path/to/repo --restore /path/to/session.jsonl
```

Use the known session path/ID. Do not inspect a user's private transcript or launch recovery just to explain the command. Recovery can resume side-effecting work; reconcile the working tree, failed command state, and reported child results before repeating writes. Resume/import/fork/continue require persistence and cannot be combined with `--no-session`.

### Reset has several meanings

| TUI command | Identity/history behavior |
| --- | --- |
| `/fresh` | Resets provider-facing stream/session state; keeps visible conversation and session file |
| `/clear` | Clears live/model context in place; keeps identity and persisted history via a reset boundary |
| `/new` | Starts a new conversation identity/session |
| `/delete` | Attempts deletion of current saved session/artifacts, then starts a successor: destructive |
| `/restart` | Relaunches OMP preserving launch configuration; restores a materialized session |

None of these is a Git reset or a code undo. Task `checkpoint`/`rewind`, when enabled, also preserve/prune **conversation context only**. They do not snapshot files or processes. Do not confuse the tool `rewind` with the TUI transcript command alias.

## Context maintenance without losing the contract

Use `/compact [focus instructions]` when a long thread needs a concise retained state. Preserve goal, exact decisions/invariants, owned files, current failing evidence, completed checks, active jobs/agent IDs, artifacts, and next action. Compaction rewrites model-visible context and persists summary entries; it does not mean the source transcript/working tree was erased.

Do not hard-code an automatic-compaction percentage, strategy order, or context-promotion model. Those depend on model capabilities and settings. A vision-based method can fail against a text-only endpoint; a nominal context window does not prove the endpoint accepts a byte/media-heavy payload. Inspect the actual error and `omp://compaction.md` rather than repeating an unchanged request.

Optional experimental notes-backed windows use `context_notes`, `new_context`, and a session-bound `history://current/full`. These APIs are absent unless enabled. Save the notebook before rollover; do not request nonexistent tools or enable an experimental mode just for a routine task. Read `omp://tools/context-notes.md` and `omp://tools/new-context.md` first. Fresh processes do not recreate historical eval kernels.

## Plan, goal, advisor, and prewalk are different

| Mechanism | Purpose | Key consequence |
| --- | --- | --- |
| `/plan` / plan toggle | Read-only planning with plan review | Finish review/resolve through the supported UI/tool; do not edit production files during planning |
| `--goal 'OBJECTIVE'` | Fresh interactive goal-directed startup | Requires enabled goal mode; bypasses auto-resume; incompatible with positional input/resume/import/startup plan combinations |
| `--advisor` | Passive model review/notes | Additional model work/cost; not independent acceptance proof |
| `--prewalk` | One-shot handoff from active planner to another execution model | Intentional model/effort switch near the first eligible edit/write boundary |
| `--plan-yolo` | Plan, automatically approve the proposal, then implement on target | Auto-approval is a material safety choice, not ordinary read-only planning |

`--plan MODEL` assigns the planning model; it is not the same as entering plan mode. `--plan-yolo-into MODEL` requires `--plan-yolo`. `--prewalk-into MODEL` arms prewalk directly; `--no-prewalk` prevents it for startup and conflicts with explicit prewalk flags.

Prewalk's precise trigger: with todo active, a successful todo call opens the gate (even `view`); without todo the gate starts open. The handoff happens at the completed assistant-turn boundary containing the first eligible edit/write result, even a failed eligible result. Read-only device calls routed through write do not count. It is one-shot and recorded on resume/restore. `/prewalk off` cancels a pending handoff without automatically switching back; `/new` follows configured settings afresh. See `omp://prewalk.md` before relying on its boundary.

Never enable prewalk, fallback, context promotion, or plan-yolo to get around a user's exact-model/no-substitution requirement. Choose ordinary planning plus explicit implementation on the permitted model instead.

## Advanced entrypoints: load on demand

- Headless bounded print/JSONL, RPC/ACP, SDK: [automation](extensibility-and-automation.md).
- Shared control versus view-only links: `omp://collab.md`; disclose who can control the session before joining/sharing.
- Session transfer/export/share: `omp://session-operations-export-share-fork-resume.md`, `omp session --help`. Do not copy live session leases blindly across machines.
- Public recording/broadcasting: `omp://stream.md`, `omp clip --help`. Publication is an authorized side effect, never part of a local help probe.
- Speech/dictation and optional local models: `omp://local-models.md`; microphone/model-download behavior needs an intentional request.
- Handoff generation internals: `omp://handoff-generation-pipeline.md`. A configured handoff compaction method is not proof an imagined `/handoff` command exists.

Sources: `omp://cli-reference.md`, `omp://keybindings.md`, `omp://tree.md`, `omp://session-operations-export-share-fork-resume.md`, `omp://session-switching-and-recent-listing.md`, `omp://compaction.md`, `omp://prewalk.md`, `omp://slash-command-internals.md`, installed CLI help.
