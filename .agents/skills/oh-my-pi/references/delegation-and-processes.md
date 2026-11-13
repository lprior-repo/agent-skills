# Delegation and process lifecycle

Use concurrency to reduce independent work, not to multiply owners of the same state. Follow the live task schema: settings can replace batch/flat shape, task/steps, and available optional fields.

## Design the split before spawning

1. Scope inline with `read`/`grep`/`glob`. Do not open with a scout.
2. Split only when at least two independent substantial slices exist, or the read set would flood context. Handle one small/direct job yourself.
3. Name shared interfaces, each slice's file ownership and non-goals, and the integration owner. Settle a true shared prerequisite first. Do not serialize independent tasks or pad the batch.
4. Give each child complete requirements: goal, known files, allowed side effects, API contract, acceptance evidence, risks. Children start without the conversation. Put shared background in `context`; large payloads belong in `local://` files.
5. Choose the most specific available agent. `scout` is read-only exploration for unknown files; `sonic` is mechanical work; `reviewer`/`security-reviewer` review; generic work omits `agent`; `seance` consults history. Project/user agents may override bundled names. Do not assume a name is available.
6. Keep build/lint/test/format work out of implementation workers mid-flight; the integration owner runs the requested narrow verification once after integration. Reviewer findings and child claims are evidence to inspect, not automatically verified success.

### Current batch shape

This is illustrative; supply actual independently owned files and requirements, not these example paths blindly.

```json
{
  "context": "Implement the accepted parser and metrics contracts. Parent integrates and runs the final narrow check. Do not build, lint, test, or format mid-flight.",
  "tasks": [
    {
      "name": "Parser",
      "task": "Own src/parser.ts only. Implement the accepted grammar and documented errors. Preserve the exported interface. Return changed behavior, files, and risks; no verification commands mid-flight.",
      "solutionSpace": "One fixed grammar and error contract; implementation details open"
    },
    {
      "name": "Metrics",
      "task": "Own src/metrics.ts only. Implement the accepted aggregation contract without touching parser code. Return behavior, files, and risks; no verification commands mid-flight.",
      "solutionSpace": "One fixed aggregation contract; implementation details open"
    }
  ]
}
```

`solutionSpace` explains uncertainty, not task size. `agent` and optional `model` belong inside each item, never at the top of a batch. `tools` means **names of functions defined in the parent's eval kernel**, not an arbitrary tool allowlist. An `outputSchema` is only available if the live schema advertises it; strict validation and permissive acceptance have different failure behavior.

For generic work, omit `agent` entirely: do not emit `"agent": "task"` just to name the implicit default. Kernel-hosted functions are registered with Python `@tool` or JavaScript `tool(fn)`; `agent()` spawns a worker and does **not** define a callable function tool.

### Isolation is explicit, not assumed

Absent an explicit isolation contract, children may share the main checkout. `isolated: true` is only accepted when task isolation is enabled and outside plan mode. Backend/merge/apply settings decide what is captured and integrated. A file-system clone or worktree is not a security sandbox and does not remove network or credential access. Inspect returned capture/apply/conflict evidence before treating changes as landed. Never tell a worker to overwrite unrelated user changes.

### Exact model identity

Use the active prompt's approved model list. A per-call `model` is one exact approved `provider/model-id[@upstream][:level]`, not a role, glob, fuzzy name, or list. Availability is not approval. Explicit agent overrides or user-chosen effort can conflict with per-call selection. If unavailable, stop that slice and report it; do not silently substitute, drop the selection, enable prewalk, or configure a fallback. CLI fuzzy matching is a separate surface.

When the user requires an explicitly pinned per-call model, actually put `model` in **every task item** of the JSON example. The generic example above intentionally omits model fields and would use defaults; it does not satisfy a pinning requirement unchanged. If the approved exact identity is unknown, show a clearly non-runnable placeholder template and do not dispatch defaults.

## Spawn → work → receive

- Submit independent slices in one batch. Keep doing useful parent work.
- Results auto-deliver. Do **not** poll `proc://`, `agent://`, logs, `.status()`, or `.done()` to watch progress.
- Completely blocked on your own running work or an owed direct reply? Call the top-level `wait` tool. Act on a returned result before waiting again.
- `read agent://ID` retrieves published output or a status/progress snapshot; `read history://ID` supplies the agent transcript when needed. A missing process-local job does not prove a parked agent or its artifacts are gone.
- `write agent://ID` messages/revives the existing child. Prefer it for a genuine follow-up over spawning a replacement that loses context. Do not broadcast a question and wait for a reply nobody owes you.
- `write proc://ID/kill` without content cancels/stops owned work. Inspect the ownership/target first. Do not use cancellation as a cleanup shortcut for somebody else's job.

### Historical consultation

If the user points to an earlier session, use the available `seance` agent with `sourceSession` and a focused question. This is a read-only consult: do not reread/grep/run that session's `.jsonl` yourself. Source artifacts stay in place. A missing saved model requires an approved explicit selection, never a guessed fallback. Check live state only for facts that could have changed since the session, such as a PR's current status.

## Finite Bash job versus supervised service

| Work | Tool call shape | Lifetime |
| --- | --- | --- |
| Short command / narrow verification | `command`, `cwd`, positive `timeout` | Runs to exit or deadline |
| Finite independent long job | Same plus `async: true` when available | Returns a job ID; original deadline still applies |
| Interactive one-off program | `pty: true`, explicit cwd/deadline | Needs supported interactive terminal |
| Server / watcher | Unique `name`, `command`, optional `ready` | Supervised service; no `async` or command `timeout` |

```json
{"command":"cargo test -p selected-crate selected_test","cwd":"/path/to/repo","timeout":300,"async":true}
```

This is a command template, not a claim that any particular project has that crate/test.

```json
{"command":"python3 -m http.server 8765","cwd":"/path/to/static-site","name":"docs-preview","ready":{"port":8765,"host":"127.0.0.1","timeout":30}}
```

- Bash state persists, but cwd resets between calls. Always set `cwd` instead of relying on a previous `cd`.
- Async does not extend a deadline. For finite calls `timeout: 0` explicitly disables it; do not use unbounded jobs by accident. Auto-backgrounding may happen after the configured foreground threshold; the result tells you the actual deadline.
- A named service must omit `timeout` and `async`; `ready.timeout` is readiness, not a lifetime limit. A log regex and a port are **both** required if both are supplied. `ready` without a name is not supervision.
- Reusing a live service name restarts it with the new spec. Use unique names; this can terminate real work.
- `read proc://NAME` inspects service logs; `write proc://NAME` sends stdin (including empty input as Enter). `write proc://NAME/mode` changes persistence according to its docs. `/kill` stops it. Do not change persistence without a reason.
- Large output spills to an artifact. Read the relevant range; do not rerun the command to see output, or hide status by piping everything through `head`/`tail`/`tee` without preserving exit status.
- Embedded Bash has in-process utilities before PATH. A literal `/usr/bin/PROGRAM` selects the system binary when a diagnosed compatibility difference requires it. Tool routing failures are instructions to use a dedicated tool, not invitations to obfuscate a shell command.

## Eval agents and workpools

Read `xd://eval/agents` before first use. In Python, `h = agent(...)` returns a handle immediately; `await h` yields its final result. In JS, `h = await agent(prompt, {...})` returns the handle. Handles expose `agent://` IDs; passing a result/handle to a downstream task creates a dependency. Keep the graph acyclic.

Kernel `wait(handles)` is an ordered barrier; `raise_errors=False` / JS `raiseErrors: false` preserves each failure in its slot. It is not the top-level wait tool. A workpool is for a batch of independent items using bounded reusable workers; submit the whole wave, then leave eval and use top-level `wait` when blocked. The first full drain closes the pool. There is no `pool.wait()`. Kernel-defined tools execute in the parent's kernel; subagents do not inherit that kernel's variables.

Do not promise unlimited concurrency: task concurrency, async-job caps, recursion depth, provider rate limits, memory/VRAM, shared locks, and budget all constrain actual throughput. Increase fan-out only when work is independent and permitted; never mutate global settings merely to make a batch larger.

Sources: `omp://tools/task.md`, `omp://task-agent-discovery.md`, `omp://tools/wait.md`, `omp://tools/bash.md`, `omp://bash-tool-runtime.md`, `omp://tools/eval.md`, `xd://eval/agents`. The current session's schemas/policies override optional fields shown here.
