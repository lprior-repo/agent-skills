# Agent workflows: use the runtime, finish the user's task

These are instructions for the AI agent operating inside OMP, not a tutorial to repeat to the user. Load one relevant recipe, act, and return the task result. Use the current tool schema and higher-priority policy over any example. Paths/commands below are templates unless identified as an in-memory example; resolve real paths and requirements before dispatch. Include required host fields, such as `i`.

## 1. Inspect → change → prove

1. Start with the known path or one scoped `glob`/`grep`. Read the implementation and relevant callers, not the entire repository. If a code read omits bodies, read only the concrete ranges its footer names.
2. For symbol definitions, callers, references, types, and code actions, read `xd://lsp` and use its live schema when the language server is available. Check references before changing exported symbols. For syntactic structure/codemods, use `xd://ast_grep` / `xd://ast_edit` after reading the device schema. Text search locates text; it does not establish symbol identity.
3. Patch existing files with the advertised `edit`. For hashline mode, copy the actual `[PATH#TAG]` and visible line numbers from the latest read. Never reuse an illustrative tag or edit omitted lines. Use `write` for a new file or intentional whole-file replacement. Do not run hashline edits alongside other tools if the host forbids it.
4. Carry the change through every affected caller. Preserve unrelated user changes. If the file changes underneath you, read the new state before retrying.
5. Run the smallest actual command/scenario proving the requested behavior. A known failing test or the user's report is already a repro; do not rerun it just to reconfirm. Avoid redundant repo-wide checks.
6. Report what changed, the real check result, and any unresolved limit. Do not deliver an OMP tutorial instead of the patch.

Tools and selector details: [tools and data](tools-and-data.md).

## 2. Reuse eval state instead of rescanning or rerunning setup

For JSON rows with `id`, `status`, and `elapsed_ms`, load once in Python:

```json
{"language":"py","title":"Load run evidence","code":"import json\nrun_rows = json.loads(read('/resolved/path/runs.json'))\nprint([row['id'] for row in run_rows if row['status'] == 'failed'])"}
```

Next cell, reuse `run_rows`; do not repeat the file load or successful setup:

```json
{"language":"py","title":"Aggregate failed-run duration","code":"print(sum(row['elapsed_ms'] for row in run_rows if row['status'] == 'failed'))"}
```

- One call is one cell. Python and Bun kernels are separate; children and new OMP processes do not inherit their variables. Reload necessary setup only after real kernel loss/reset or an intentional data refresh.
- Python file helpers are synchronous; JS equivalents are awaited. For internal tool/device dispatch use `await tool.read({...})` or `await tool.write({...})`, not a filesystem write helper. Top-level `await` works; do not call `asyncio.run()`.
- A failed cell can already have changed files or sent a request. Retry the failed step, not the whole workflow.
- Read `xd://eval/helpers`, `xd://eval/agents`, `xd://eval/judge`, or `xd://eval/browser` before first use of that feature. Do not import imagined OMP client libraries to access built-in tools.
- Print the answer/evidence fields, not complete logs, private records, or credentials. Register optional parent-kernel functions with `@tool` / `tool(fn)`; `agent()` spawns a worker instead.

## 3. Delegate real independent work; keep the parent useful

1. First map the files and shared interface inline. Keep a single small job inline. Delegate only substantial independent slices or a read set that would flood context, under current policy.
2. Submit independent slices together via `task` with shared `context` and per-item `task`, `solutionSpace`, file ownership, non-goals, and observable acceptance. Give children the necessary requirements: they do not inherit your conversation.
3. Omit `agent` for generic work; do not write `"agent":"task"`. Use a specialist only when available and appropriate. `scout` is for genuinely unmapped read-only exploration, not the default opener.
4. Model selection is governed by the active allowlist and agent/user overrides. For a requested per-call pin, put the exact approved `model` inside **every item**; do not put it at batch level or silently use defaults. If the identity is unknown, do not dispatch a placeholder; if unavailable, report it rather than substitute.
5. `tools` names parent-kernel-defined functions, not a built-in-tool allowlist. Do not assume checkout isolation; use it only when supported/enabled. Partition shared edits under one integration owner. Workers skip mid-flight build/lint/test/format; parent checks once after integration.
6. Continue the parent's independent work. Results auto-deliver; do not poll job/agent state. Use top-level `wait` only when completely blocked on your own work or a reply actually owed. Inspect the returned result, integrate, then prove the requested behavior.
7. For a real follow-up, `write agent://ID` messages the existing worker. Read published `agent://ID` output or `history://ID` only when needed for evidence, not as a progress loop.

The complete task example, optional isolation, and workpool rules live in [delegation and processes](delegation-and-processes.md). Do not read that whole guide for a one-file edit.

## 4. Match command lifetime to the work

A finite narrow check uses `bash`, explicit cwd, and a bounded timeout. Example template:

```json
{"i":"Running focused behavior check","command":"THE_PROJECTS_FOCUSED_CHECK","cwd":"/resolved/repo","timeout":300}
```

Add `async:true` only for finite independent long work if supported. Its original deadline still applies. Do not rerun a background command to fetch its output: use its delivered result/artifact.

A server uses a unique service name and readiness, not a finite-job spec. Example template:

```json
{"i":"Starting isolated preview service","command":"python3 -m http.server 8765 --bind 127.0.0.1","cwd":"/resolved/static-site","name":"preview-REPLACE_WITH_UNIQUE_NAME","ready":{"port":8765,"host":"127.0.0.1","timeout":30}}
```

Omit command `timeout` and `async` for a named service. `ready.timeout` is the readiness deadline, not service lifetime. Choose an available port and unique name; reusing a live name restarts it. Retain the returned ID and stop only your own unneeded service through `write proc://ID/kill` without content.

## 5. Inspect output instead of repeating expensive work

- If a result spills, use its returned `artifact://ID` and a relevant selector such as `:120-180` or `:-40`. Do not guess an artifact ID or rerun the command for visibility.
- `local://NAME.md` shares a concise contract/evidence payload with children. Include its URI in their task. Do not assume arbitrary parent variables or attachments are inherited.
- Use `read` for known static URLs/documents. For JSONL, SQLite, archives, and other special selectors, first read `xd://read/sources`; do not invent syntax or dump the entire data source.
- For requested mid-task progress, use `out://NAME.md` when the host supports it, then keep working. Do not end the turn with an intermediate report while actionable work remains.

## 6. Verify the actual UI without borrowing the user's session accidentally

Read `xd://eval/browser` before browser namespace use. A live relay means `browser.open` without `app` may control signed-in Chrome. `app:"auto"` skips relay but can still select configured external backends; choose `app:"own"` or `app:"chromium"` for the OMP-owned runtime. These are not OS sandboxes or guarantees of a fresh identity. Avoid consequential actions without authorization.

Observe the actual changed page/component, not just a successful HTTP status. Close only tabs/services you created when finished. For CLI/TUI changes, launch the program and inspect output/state. If the needed runtime is unavailable, state the limit rather than claiming visual proof.

## 7. Recover the right state; do not hide a tool failure

- Stale edit: read the changed file, then patch the real current snapshot.
- Missing optional device: inspect its documentation/availability. Do not invent a tool or obfuscate a shell command to bypass routing.
- Kernel loss: reload the necessary setup, not previously completed side effects.
- Interrupted conversation: `omp --restore ID_OR_PATH` is terminal recovery, not an agent tool call and not filesystem/process rollback. If the user references a prior session, follow the available read-only `seance` contract rather than dumping its JSONL yourself.
- Configuration request: inspect the exact `cfg://NAMESPACE/KEY` schema/source, then use the top-level write object. Session changes and `/save` persistence are distinct; never weaken approvals just to make a tool work. Read [configuration and models](configuration-and-models.md) only when needed.

Success is the user's completed task plus observed evidence, not the number of OMP features used.
