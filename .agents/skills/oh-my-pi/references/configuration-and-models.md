# Configuration, profiles, models, and context

This is a usage guide, not permission to change settings. Only inspect `cfg://` or live settings when the user asks about configuration. Do not open auth/config/model files wholesale for a generic how-to request; they can contain credentials or private endpoints.

## Choose the smallest scope

| Intent | Surface | Persistence |
| --- | --- | --- |
| One launch | CLI flags such as `--model`, `--thinking`, `--hide-thinking`, `--approval-mode`, `--config` | Runtime override; flags/overlays are not written back |
| One currently running session | Approved `write` to `cfg://NAMESPACE/KEY` | Session-only; does not update config.yml |
| Persist an agent-made setting | Approved `write` to `cfg://NAMESPACE/KEY/save` | Persistent; requires explicit approval |
| Human settings UI | `/settings` | Persistent changes generally target active global native config |
| Terminal configuration management | `omp config get/set/reset` | `set`/`reset` target main global native config, not the project |
| Committed project convention | Project `.omp/config.yml` | Project layer; follow repo ownership and do not commit secrets |
| Isolated native state | `omp --profile NAME` | Separate auth/sessions/settings/caches under its profile |

Agent setting workflow:

1. Read `cfg://` only for a configuration request. Read the relevant namespace/key for its exact type, default, source, and effective value. Do not guess a key or dispatch a made-up settings tool.
2. Explain only a material tradeoff; choose the requested scope. If no setting was specified, keep the key/value generic until the intended setting and its schema are known; do not invent a setting/value or relax approvals.
3. Make the requested approved session-only write. If long-term persistence is desired, ask whether to save. Unanswered approval is not approval; move on rather than changing YAML to evade the prompt.
4. For persistence, append `/save` to the setting URI: `cfg://NAMESPACE/KEY/save`, with approval. This is a URI suffix, **not a TUI `/save` command**. Do not assume a session-only change survives restart.
5. Verify the affected key and behavior, not the entire settings database. Session-specific flags/env overrides may still win.

Dispatch with the advertised top-level `write` tool's `{path, content}` object, using the value format required by that setting. Eval's filesystem `write()` helper accepts local/file-backed targets, not `cfg://` device writes; it is not a substitute. Do not invent positional helper signatures or a generic save slash command.

For an explanation without a specified setting, use this **non-runnable template**, not an arbitrary permission setting. Replace placeholders only after inspecting the intended setting; do not send the template literally:

```json
{"path":"cfg://NAMESPACE/KEY","content":"VALUE_IN_THE_SETTING_SCHEMA_FORMAT"}
```

The persistent template changes only the URI to `cfg://NAMESPACE/KEY/save` and additionally requires persistence approval.

```bash
# Read-only terminal examples; choose a real known setting.
omp config get theme.dark
omp config list --scope project --json
omp config --help
```

`list --scope global|project` reports keys **as written in that layer**, not the same thing as an effective merged view. List only when needed; never paste whole output containing sensitive values. `omp config set` is persistent; it is not the terminal equivalent of a session-only cfg write.

### Layers and merge behavior

Low to high: built-in defaults → global native config → project config → repeatable CLI `--config` overlays → setting environment overrides → runtime overrides. Use `omp://config-usage.md` and the setting's live source for exceptions. Objects generally deep-merge; arrays/scalars replace rather than append. `null` commonly means unset at read time, allowing a default/fallback. Do not edit one array entry as though it merges with a lower-priority list.

Interactive/RPC/ACP sessions watch global/project/overlay files. A parse failure can retain last-good values; it does not mean the bad edit was accepted. Reload is not a full restart: newly discovered modules and startup-only behavior may require `/restart`. Changed model roles affect later resolution/spawns, not necessarily a healthy active model or existing workers. Never claim every setting is hot-reloaded.

`modelRoleStorage: project` is a specific exception for model-role assignments; it does not redirect arbitrary settings writes to the project. Profiles default to `~/.omp/profiles/NAME/agent`; `PI_CODING_AGENT_DIR` can relocate the native directory. Foreign user directories such as `~/.claude` are not automatically profile-isolated. Named profiles can inherit base-profile keybindings. A profile is not an OS sandbox.

## Model selection versus model registration

```bash
omp models --help
omp models find qwen
omp models --json
# When a catalog is actually stale; this may use the network:
omp models refresh

# Replace the quoted placeholder with an available, approved exact identity.
omp --model 'PROVIDER/MODEL_ID' --thinking high
```

- `omp models` shows the model catalog, not successful authentication/inference and not permission to use every listed model. Do not infer endpoint health from a catalog entry.
- CLI/TUI selection supports fuzzy names and configured role selectors; a provider-qualified ID must be exact. Per-call subagent `model` selection is stricter: use one exact operator-approved identity, including any route/effort suffix. See [delegation](delegation-and-processes.md).
- Roles such as default, smol, slow, plan, advisor, tiny, memory, judge, and commit serve different consumers. Setting smol does not switch the current chat model. Do not overwrite every role to fix one request.
- `--models A,B` scopes cycling; `--model` selects the active model. `--thinking` controls reasoning effort only within model support. `--hide-thinking` hides TUI blocks without disabling reasoning. Print mode has separate `--print-thoughts` output control.
- Prewalk and plan-yolo intentionally switch models at a workflow boundary; advisor adds model work. Do not enable them when exact-model/no-fallback policy applies. No silent provider, account, or model substitution.
- Custom providers are registered in the active agent directory's `models.yml`. Read `omp://providers.md`, `omp://models.md`, and relevant `omp://provider-compat-reference.md` / `omp://provider-endpoint-constraints.md` **before** choosing API type, URL shape, context size, or capability flags. A server exposing `/v1/models` does not prove it supports Responses, tools, images, or the desired reasoning format.
- Use secret references/environment-variable names, not literals. A configured provider key can override stored OAuth; do not diagnose credential precedence by dumping values. Use `omp login --help` or `/login` for the intended provider, never `omp token`/auth export as a help probe.
- `omp://local-models.md` covers tiny/memory/speech/dictation engines. Built-in Ollama, llama.cpp, and LM Studio discovery is a different surface from custom local HTTP providers and the local tiny-model role. Do not assume the user's GPU workers have those provider IDs or ports.
- Downloads, model refresh, login, and `omp setup` have network/install/account effects. Inspect their help first; do not run them merely to make a document example true.

## Instructions: context, rules, skills, templates, agents

| Kind | Purpose | Loading |
| --- | --- | --- |
| Context file (`AGENTS.md`, `CLAUDE.md`, etc.) | Repository/user background and conventions | Discovered opening context with scope/precedence rules |
| Sticky native `RULES.md` | Short hard requirements that stay on requests | Always-apply rule from native user/selected project root |
| Regular rule | Conditional/on-demand policy | Matching/discovery rules; `rule://NAME` when advertised |
| Skill `SKILL.md` | Reusable procedure + progressive references | Description advertised; load via `skill://NAME` or `/skill:NAME` |
| Prompt/slash template | Reusable prompt expansion | Host-specific command/template discovery |
| Agent definition | Child role, model/tool contract | Discovered by exact name for task/eval spawning |
| Extension/plugin | Executable customization | Trusted host code, not merely Markdown |

### Context discovery traps

- Native project discovery chooses the **nearest non-empty `.omp/`** walking cwd toward repo root. Missing `AGENTS.md`/`RULES.md` there does not make it keep walking to a farther native `.omp/`.
- One user-scope context file wins; generally one project file per directory depth wins by provider priority. Native wins at an equal scope. Across depths, surviving ancestor and closer files can coexist. `contextFiles.extra` is additive and has different slot behavior.
- Foreign user providers can require explicit opt-in; their project support is not the same as user discovery. `.claude/CLAUDE.md` and similar foreign config-directory files do not all have ancestor walking.
- `disabledProviders` is shared by model-provider and config-discovery IDs. `google` and `gemini` are not synonyms. Disabling a discovery source can also remove its settings, commands, skills, MCP, and hooks.
- `disabledExtensions` can disable one context filename while keeping the provider. It is not identical to provider disablement: disabling a winner can expose a lower-priority file.
- Context `@path` imports resolve relative to the importing file, recurse to a bounded depth, and leave missing paths literal. Do not assume an import succeeded.
- Task subagents drop files named `AGENTS.md`, including newly discovered ones, but retain native sticky `RULES.md`. Put mandatory helper requirements in the explicit task contract and appropriate short sticky rules, not an assumed inheritance of the parent's entire context. Do not migrate a user's policy files without a requested change.
- Already injected context does not need rereading just to prove it exists. Read deeper-directory instruction pointers when required before editing there. Inspect discovery for an actual missing-policy diagnosis.

### Skill usage and installation

This skill is invoked as `/skill:oh-my-pi [request]` when skill commands are enabled, or the agent loads `skill://oh-my-pi`. References are reachable as `skill://oh-my-pi/references/FILE.md`. A natural request such as “use the oh-my-pi skill to recover this session” also works when the host selects it.

Native authored skills use `<agentDir>/skills/NAME/SKILL.md` or project `.omp/skills/NAME/SKILL.md`; managed skills use `<agentDir>/managed-skills/NAME/SKILL.md`. Other enabled discovery providers/custom directories can contribute skills. Descriptions are trigger surfaces, not the whole skill body. Avoid nested folder layouts the provider does not scan and collisions with an authored skill; authored skills outrank managed ones. `--skills` filters and `--no-skills` can hide a correctly installed skill.

`omp skill` is the **Skillshare registry manager**, not a generic command to execute a local skill. `omp skill list` is not proof every local managed/authored skill is discoverable. Installing third-party skills can introduce scripts; publishing uploads content. Read `omp skill --help` and `omp://skills.md` first. Never use `--allow-secrets` to bypass a release finding or change user-authored skills through auto-learning.

Sources: `omp://config-usage.md`, `omp://settings.md`, `omp://models.md`, `omp://providers.md`, `omp://local-models.md`, `omp://context-files.md`, `omp://skills.md`, `omp://prewalk.md`; live `cfg://` policy and installed CLI help define runtime behavior.
