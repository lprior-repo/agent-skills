# Tools and data: operate the actual OMP surface

Validated against the live tool contract and OMP 18.19.0 documentation on 2026-11-12. Examples below are **agent tool calls**, not terminal commands. Session policy and the currently advertised schema take precedence over these examples.

## Discover before dispatch

`read("xd://")` lists mounted on-demand devices. Read `xd://NAME` to obtain its current JSON schema, then call `write` with that URI and a JSON-encoded argument object. Do not guess field names or invent a top-level tool. Some tools are directly advertised; use those directly. Tool availability depends on settings, platform, installed prerequisites, agent permissions, and plan mode.

```text
read:  {"path":"xd://lsp"}
write: {"path":"xd://lsp","content":"<JSON matching the returned schema>"}
```

The second line is a template, not a runnable request. Read the schema first. An exported-symbol change needs LSP references before edits; use definitions/implementations/hover rather than text-searching for semantic relationships. Use AST search/edit for syntactic structure and codemods. AST editing can be staged; follow its returned resolve/reject instructions instead of assuming an edit landed.

| Need | Prefer | Important boundary |
| --- | --- | --- |
| Known file, URL, directory, document | `read` | Static web first; browser only for interaction/authentication/JavaScript |
| Locate paths or literal/regex occurrences | `glob`, `grep` | Scope the paths; respect ignore/hidden flags; follow continuation instructions |
| Symbol meaning, callers, references | `xd://lsp` | Current device schema and language-server availability govern |
| Structural code search/change | `xd://ast_grep`, `xd://ast_edit` | Inspect matches before transformation |
| Existing-file patch | `edit` | Use the active edit mode, not generic CLI help's description |
| New file / deliberate whole-file replacement | `write` | Not a substitute for precise edits |
| Stateful calculation, data processing, orchestration | `eval` | One cell per call; each language and agent has separate state |
| Command, narrow verification | `bash` | Explicit cwd; distinguish finite job from service |
| GitHub state / issue / PR | `issue://`, `pr://`, `xd://github` | Reading a PR never authorizes a comment, merge, or push |
| Debug a running process | `xd://debug` | Read DAP/device documentation first |
| Material human tradeoff | `ask` | Semantic HTML form; not questions tools can answer |

## Read selectors and snapshots

```json
{"path":"src/parser.ts:40-90"}
{"path":"src/parser.ts:5-16,140-175"}
{"path":"build.log:-40"}
{"path":"src/parser.ts:40-90:raw"}
```

- Put ranges in `path`; do not pass invented `offset`/`limit` fields to the `read` tool. Eval's file helpers have a different contract.
- Line numbers are 1-based; `:50+20` means twenty lines starting at 50. A non-raw `:50` starts at 50; `:50:raw` is exactly line 50. `:50-:raw` runs onward.
- Non-raw reads may add surrounding context or structural boundary lines. Use `:raw` for exact text, but raw output does not supply edit anchors.
- A bare code read can be declarations-only. Read **only the concrete omitted ranges named in its footer** when bodies matter. Never infer code hidden behind ellipses.
- Every ranged continuation needs its selector; rereading a bare path does not advance a cursor.
- Directory reads show the root and a bounded child preview. Read the named child to descend; do not assume the preview is the full subtree.
- `http://localhost:PORT/` needs its trailing slash before a line selector so the port is not confused with a range.
- Read `xd://read/sources` before using archive, SQLite, jq/JSONL, SVG, video, PDF-page, executable, or merge-conflict selectors. These share `path`, not semantics. Do not guess them.

### Hashline edit example

Given a real preceding read returning `[hello.py#A1B2]` and `2:    print("old")`, the shape is:

```text
*** Begin Patch
[hello.py#A1B2]
PUT 2.=2:
+    print("new")
*** End Patch
```

`A1B2` here is illustrative: **copy the actual returned tag**, never this example tag. All hunk numbers refer to the original snapshot. `PUT <N:` / `PUT >N:` insert; `CUT N.=M` removes; block `N*` requires a visible multiline opener. Bodies contain `+`-prefixed verbatim lines, not ordinary diff context. Change only displayed lines. After an edit use its fresh anchors or a fresh read for the next edit. On stale/conflicting state, reread before proceeding. Never run edits concurrently on one file; never run hashline `edit` in parallel with other tools when the host forbids it.

Other edit modes exist. `omp --help` currently summarizes edit as find/replace; the active tool schema and `omp://tools/edit.md` are authoritative for how this session edits.

## Persistent eval, without shell one-liners

Python call 1:

```json
{"language":"py","code":"import json\nfrom pathlib import Path\ndata = json.loads(read('package.json'))","title":"Load package metadata"}
```

Python call 2:

```json
{"language":"py","code":"print(sorted(data.get('scripts', {})))","title":"List available scripts"}
```

- Use `py` or `js`, not `python`/`javascript`, and one cell per call. Python is a subprocess, not IPython; no `get_ipython()`, `%matplotlib`, or `asyncio.run()`.
- Top-level `await` works. JS runs Bun; `Bun.file`, `Bun.write`, and `Bun.$` are available. Python state does not cross into JS, other agents, or a fresh OMP process.
- `tool.NAME` calls are awaited. Python `read`/`write` helpers are synchronous; JS equivalents are awaited. Do not confuse Python helper `read(...)` with `await tool.read({...})`.
- Reusable setup: inspect a local script, then `%load "path with spaces.py"` as its own cell. Load once; edits to the file require an intentional reload. A kernel-loss notice requires reloading needed state; ordinary compaction is not itself a reason to rerun setup.
- `%pip install DISTRIBUTION` and `%bun add PACKAGE` are standalone cells. JS installs default to an OMP-managed per-project environment; `%environment project` deliberately changes repository dependencies. Installing packages is a mutation, not a harmless documentation probe.
- Retry only the failed step: a failing cell may already have written a file or sent a request. Do not reset or rerun successful setup by habit.
- Read `xd://eval/helpers`, `xd://eval/agents`, `xd://eval/judge`, `xd://eval/browser`, or `xd://eval/archive` **before first use of that feature**. Session history questions use the available archive/seance contract, not indiscriminate transcript dumps.
- Printing an image/figure attaches it. Print only needed fields, not giant data structures, credentials, cookies, or private histories. MCP structured results are in `details.structuredContent` when supplied; check `hasError` first and do not trust a JSON-looking text rendering as a validated payload.

## Internal URI map

| URI | Purpose |
| --- | --- |
| `omp://` | Bundled documentation index; best starting point for version-matched help |
| `skill://NAME` / `skill://NAME/references/FILE.md` | Load skill and its bundled references |
| `xd://NAME` | Tool documentation/schema; JSON writes invoke the device |
| `local://NAME.md` | Shared task artifact; parent/subagents share its root |
| `artifact://ID` | Full spilled tool output; use selectors for relevant evidence |
| `agent://ID` | Latest published child result; writes message/revive the child |
| `history://ID` | Concise registered-agent transcript; not a universal history database |
| `proc://ID` | Owned job/service state; `/kill` controls it, service writes send stdin |
| `cfg://...` | Effective settings with source/type; writes require the settings workflow |
| `out://NAME.md` | Write-only nonterminal report when the user asks for progress |
| `issue://N`, `pr://N` | GitHub issue/PR; short forms use the current checkout |
| `ssh://HOST/...` | Configured remote UTF-8 file/directory, not an unrestricted shell |
| `memory://ID` | Full Mnemopi row; backend/scoping dependent |

Only use schemes advertised by the current host. Immutable output/skill/history sources do not provide mutable-file anchors. Internal URIs work with many embedded Bash filesystem operations, but not with every external program. For SSH, `/~/` means the connected account's home; percent-encode literal `:`, `?`, and `#`; read its URI documentation before remote writes.

Observed integration trap: invoking terminal `omp read skill://NAME:1-5:raw` through the agent's embedded Bash can fail before OMP starts because the shell tries to resolve that URI as an external-program file operand. Use the dedicated `read` tool for agent reads. To test the actual CLI's URI handling, invoke it with direct subprocess argv (without embedded-shell translation); that route successfully resolved this installed skill. Ordinary terminal CLI syntax and embedded-tool operand handling are different surfaces.

Sources: `omp://tools/read.md`, `omp://tools/edit.md`, `omp://tools/eval.md`, `omp://tools/bash.md`, `omp://tools/ast-edit.md`, `xd://read/urls`, `xd://eval/helpers`, `xd://eval/agents`; live tools are the final wire contract.
