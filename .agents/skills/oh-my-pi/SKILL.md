---
name: oh-my-pi
description: "Agent runtime playbook for Oh My Pi (OMP). Use proactively while doing real work to choose tools, inspect and edit code, reuse eval state, coordinate subagents, supervise commands/services, inspect artifacts, and verify UI. Applies even when the user never mentions OMP; complements domain skills. Not OpenMP or Oh My Zsh."
---

# Oh My Pi: agent runtime playbook

Use OMP to **complete the user's task**, not to teach them the runtime unless they asked. This skill is for the AI agent doing the work. Apply it when choosing or coordinating OMP capabilities during ordinary coding, research, diagnosis, or verification; the user need not mention OMP. Pair it with the relevant domain skill rather than replacing that skill.

Reference baseline: OMP 18.19.0, checked 2026-11-12. The active tool schemas, permissions, user requirements, and higher-priority instructions always win. Do not assume optional features, model identities, or filesystem paths are portable between sessions.

## Start here: act with minimum context

1. Identify the user's actual deliverable, known files, constraints, and smallest useful next action. Reuse context already available; scope one missing fact inline rather than surveying the machine.
2. Choose the tool from the table below. If its live schema is already advertised, use it. Read the device/namespace documentation before first use of an on-demand capability, not the whole OMP manual.
3. Execute the work. Reuse loaded eval state, retained output, and existing workers. Do not repeat successful setup, expensive commands, or side effects to regain visibility.
4. Run the narrowest meaningful check of the changed behavior. Inspect actual output/UI as appropriate. Integrate delegated work before the final check.
5. Deliver the requested result, observed verification, and any real limit. Do not substitute commands, a plan, an OMP explanation, or a claimed child success for completed work.

Load this playbook once when relevant; consult only the needed recipe thereafter. Do not start every task with `omp --version`, `omp --help`, the docs index, settings inspection, or a new model request. Check versions/help only when resolving a real compatibility or syntax question. Ask only for a material choice that tools/context cannot settle.

## Pick the tool, then use it

| Need | Use | Decision that matters |
| --- | --- | --- |
| Known file, directory, static URL, log, artifact | `read` | Put ranges in `path`; follow actual omission/continuation instructions |
| Find unknown paths or text | `glob`, scoped `grep` | Start inline; do not open with a scout or scan all of home |
| Definitions, callers, types, references, refactor actions | `xd://lsp` | Use the live language server when available; references before exported-symbol changes |
| Structural search or codemod | `xd://ast_grep`, `xd://ast_edit` | Read schema, inspect matches, then transform |
| Existing-file change | Active `edit` | Copy real current anchors; patch visible lines only |
| New file / intentional whole-file replacement | `write` | Do not overwrite unrelated user edits |
| Stateful data processing / multi-step computation | `eval` | One cell; retain Python/Bun state instead of rerunning setup |
| Command or focused check | `bash` | Explicit cwd; bounded finite job versus named service |
| Substantial independent slices | `task` | Shared contract, disjoint ownership, one integration owner |
| Actual UI interaction/proof | Browser namespace via `eval` | Read `xd://eval/browser`; choose the correct trust boundary |
| Genuine blocked wait | Top-level `wait` | Only owned running work or an answer actually owed; never poll |

For a device, `read xd://NAME` obtains documentation/schema; a top-level `write` to that URI with JSON-encoded `content` invokes it. Do not invent a device's fields or wrap filesystem helpers around tool dispatch. For eval helpers/namespaces, read the relevant `xd://eval/FEATURE` before first use.

**Copy/adapt execution patterns:** [Agent workflows](references/agent-workflows.md) — `skill://oh-my-pi/references/agent-workflows.md`. Read only the relevant section; the deeper references below are available when that pattern is insufficient.

## Keep the execution state correct

### Reads, edits, and evidence

- A bare code read can omit function bodies. Read only the concrete omitted ranges its footer names; do not infer hidden code or guess ellipses. Repeat the selector on every ranged read.
- In hashline edit mode, copy the actual `[PATH#TAG]` and original line numbers from the latest read. Do not fabricate tags, patch unseen lines, reuse stale anchors, or run `edit` alongside other tools when forbidden. On conflict, read the new state before editing.
- Large outputs arrive with an `artifact://ID`; read the needed range instead of rerunning the producer. `local://NAME.md` shares a concise contract/evidence payload with children. Use returned IDs, not invented ones.
- Prefer independent reads/checks in one parallel batch when the host offers it. Preserve real dependency order and mutation ownership.

### Eval and command lifetime

- Python and Bun each retain their own kernel state; children and new OMP processes do not inherit it. Reuse existing variables/setup after normal compaction. Reload necessary state only after actual kernel loss/reset or deliberate refresh.
- Python file helpers are synchronous; JS equivalents are awaited. `await tool.NAME({...})` invokes a session tool. Register parent-kernel functions with `@tool` / `tool(fn)`; `agent()` spawns workers, not function tools.
- A failed cell may already have produced side effects. Retry the failed step, not the successful prefix. Do not use `asyncio.run()`; top-level `await` is supported.
- Bash cwd resets between calls: specify it. A finite async job retains its deadline. A supervised service uses a **unique name plus readiness**, with **no `async` or command `timeout`**; `ready.timeout` only bounds readiness. Reusing a live name restarts it. Stop only owned, unneeded work.

### Delegation without wasted work

- Scope inline first. Keep one small/direct job inline. Delegate only substantial independent slices or a read set that would flood context, under current policy; do not manufacture parallel work.
- Submit independent slices in one `tasks[]` batch with shared `context`. Each item needs a self-contained `task`, meaningful `solutionSpace`, owned files, non-goals, and acceptance. Children do not inherit the conversation; supply actual requirements and artifact links.
- Use an available specialist when appropriate. Omit `agent` for the generic default; never spell it `"agent":"task"`. Do not assume isolated checkouts or inherited kernel variables.
- Follow the live model allowlist and agent/user overrides. If explicitly pinning per call, put the exact approved `model` inside **every item**, never at batch level. A generic example omitting `model` does not satisfy a pin. If unknown/unavailable, do not dispatch a placeholder, silently choose defaults, or substitute a model.
- `tools` names parent-kernel-defined functions, not a built-in-tool allowlist. Workers skip mid-flight build/lint/test/format; the integration owner runs the final narrow check once.
- Keep useful parent work going. Results auto-deliver. Do not poll `proc://`, `agent://`, logs, `.status()`, or `.done()`. Use top-level `wait` only when fully blocked. Act on the result; message the existing `agent://ID` for a genuine follow-up rather than replacing it.

## Safety boundaries that affect tool choice

- Read known static URLs with `read`; use a browser only when interaction, JavaScript, authentication, or UI proof requires it. With relay on, default `browser.open` may drive signed-in Chrome. `app:"auto"` only skips relay; `app:"own"`/`"chromium"` selects OMP-owned runtime, not an OS sandbox or guaranteed fresh identity.
- Do not inspect/change settings, permissions, accounts, models, or profiles merely to make a normal task easier. For a requested setting, inspect its exact `cfg://NAMESPACE/KEY` schema/source. Approved top-level `write` with `{path, content}` is session-only; appending `/save` requests persistence and approval, not a TUI command. Keep an unspecified key/value generic. Do not relax approvals as an example or edit YAML to bypass them.
- Available credentials, extensions, MCP commands, and browser relay have real authority. Installs, publishing, login, sharing, destructive cleanup, and consequential browser actions need their required authorization. Exported conversations can contain unredacted private data; `--max-cost` is not a strict cap on in-flight spend.
- Conversation moves, compaction, checkpoint/rewind, forks, and restoration do not undo files or processes. Reconcile current state before repeating side effects. A referenced prior session uses the available read-only `seance` contract, not your own JSONL dump.

## Deep references: load only for the current need

| Need | Reference |
| --- | --- |
| Ready-to-adapt agent execution patterns | [Agent workflows](references/agent-workflows.md) |
| Selectors, edit details, eval, devices, internal URIs | [Tools and data](references/tools-and-data.md) |
| Full task JSON, exact model identity, isolation, workpools, process lifecycle | [Delegation and processes](references/delegation-and-processes.md) |
| Requested configuration, profiles, model roles, context/rules discovery | [Configuration and models](references/configuration-and-models.md) |
| MCP/LSP, extensions/plugins, print/JSONL/RPC/ACP/SDK automation | [Extensibility and automation](references/extensibility-and-automation.md) |
| Browser/privacy/cost/memory, failure triage, trust | [Troubleshooting and safety](references/troubleshooting-and-safety.md) |
| User asks about TUI controls, sessions, restore/resume/tree/fork, or teaching | [Interactive and sessions](references/interactive-and-sessions.md) |

Use direct `omp://TOPIC.md` for an identified missing detail, or `omp://` only when the topic is unknown. Live tool schemas outrank CLI summaries. A how-to request is guidance, not authorization to mutate settings or launch a billable integration.

## Completion contract

Return the user's artifact/answer, relevant changed paths, and checks actually run. State unverified limits at the claim. Do not report a plan as execution, source inspection as UI proof, or a dispatched worker as a completed change. Stop when the requested behavior is verified; no extra broad audits.

[Agent workflow evals](evals/agent-workflows.json) exercise real data/state reuse, economical tool use, and service lifecycle. [Deep-contract evals](evals/evals.json) retain recovery/model/privacy checks. [Trigger cases](evals/trigger-cases.json) include ordinary agent work without an explicit OMP question. Expectations are not measured trigger rates or guarantees of model compliance.
