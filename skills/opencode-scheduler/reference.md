# OpenCode Scheduler Reference

Validated in this environment:
- Plugin: `opencode-scheduler@1.3.0`
- OpenCode: `1.4.7`
- Platform: Linux

## Install

To enable the plugin in a repo or global config, add it to `opencode.json`:

```json
{
  "plugin": ["opencode-scheduler"]
}
```

Use this skill for the plugin after that installation step exists or when the user wants help setting it up.

## Natural-Language Requests To Recognize

These should map cleanly to scheduler operations:
- "Schedule a daily job at 9am to search for standing desks under $300"
- "Schedule a job every Monday at 8am to summarize my GitHub notifications"
- "Schedule a job every 6 hours to check if my website is up and alert me on Slack if it's down"
- "Show my scheduled jobs"
- "Show scheduler version"
- "Install the scheduled job best practices skill"
- "Show details for standing-desk"
- "Update standing-desk to run at 10am"
- "Run the standing-desk job now"
- "Show logs for standing-desk"
- "Delete the standing-desk job"
- "Run scheduler global cleanup"

## Primary Tool Map

Use scheduler tools first.

| Tool | Use it for | Notes |
| --- | --- | --- |
| `get_version` | Confirm plugin and opencode versions | Good first step when debugging setup or backend behavior |
| `schedule_job` | Create a new recurring job | Verify afterward with `list_jobs` or `get_job` |
| `list_jobs` | List jobs in the current scope | Use `allScopes: true` for cross-project inventory; `includeLegacy: true` for old storage |
| `get_job` | Show one job's stored metadata | Use before and after updating a specific job |
| `update_job` | Change an existing job | Verify with `get_job` or `list_jobs` |
| `run_job` | Trigger a job immediately | Fire-and-forget; follow with `job_logs` |
| `job_logs` | Read the latest logs for one job | Primary source of truth for run output |
| `delete_job` | Remove one job | Verify with `list_jobs` |
| `cleanup_global` | Preview or remove scheduler artifacts globally | Dry-run first unless the user explicitly wants deletion |
| `get_skill` | Fetch the built-in scheduler skill template | Built-in name: `scheduled-job-best-practices` |
| `install_skill` | Install the built-in scheduler skill into a repo | Writes `.opencode/skill/scheduled-job-best-practices/SKILL.md` by default |

When the plugin tools are not enough, use shell commands only to confirm the underlying backend state.

## Common Parameters

The scheduler tools expose many fields because jobs can wrap `opencode run` deeply.

Most common fields when creating or updating jobs:
- `name`: stable human-readable job name; becomes the slug basis
- `schedule`: 5-field cron expression like `0 9 * * *`
- `prompt`: natural-language task for `opencode run`
- `workdir`: job scope and working directory; defaults to the current directory if omitted
- `timeoutSeconds`: hard stop for long-running jobs; `0` or omitted disables timeout
- `attachUrl`: attach the run to an existing `opencode serve` or `opencode web` backend when needed
- `agent`, `model`, `variant`, `title`, `share`, `continue`, `session`, `runFormat`: forwarded to `opencode run` behavior
- `files`, `command`, `arguments`: extra run inputs when the job needs them
- `format: "json"`: structured tool response for verification-heavy flows

Use the exact field names above. Do not invent wrapper names.

## Scope Model

Jobs are scoped by `workdir`.

Important consequences:
- `list_jobs` defaults to the current workspace scope
- jobs from different projects do not collide
- logs, run history, lock files, and scheduler unit names are isolated per scope
- if the user says "show all scheduled jobs", use `allScopes: true`
- if they ask about old pre-scope jobs, also set `includeLegacy: true`

Working-directory rules:
- jobs run from the `workdir` where they were created
- that `workdir` determines which `opencode.json` and MCP configuration are picked up
- if the user says "from /path/to/project", use that as `workdir`
- if they omit it, current directory scope is the default

## Built-In Skill

The plugin ships one built-in skill:
- `scheduled-job-best-practices`

Use `get_skill` to inspect it or `install_skill` to write it into a repo.

What it is for:
- hardening recurring job prompts
- enforcing non-interactive execution
- pushing idempotent outputs and compact end-of-run summaries
- computing runtime values like dates instead of assuming magic placeholders

Use it when the user is asking for the job prompt itself to be robust, not just when they want the scheduler plumbing configured.

The install path written by the plugin is:
- `.opencode/skill/scheduled-job-best-practices/SKILL.md`

## Backend Behavior

Backends by platform:
- macOS: `launchd`
- Linux with `systemd --user`: `systemd`
- Linux or POSIX without usable systemd: `cron` fallback
- Windows: Task Scheduler via `schtasks`

Reliability behavior from the plugin:
- scheduled runs are supervised on macOS and Linux
- no overlap for supervised scheduled runs
- optional timeout support
- scheduled runs force non-interactive permissions so they do not hang on approval prompts

How it works at runtime:
1. The user describes the schedule and the work in natural language.
2. The plugin writes a scoped job definition.
3. The OS scheduler invokes the plugin's supervisor at the scheduled time.
4. The supervisor runs the job, appends logs, and updates metadata.
5. `run_job` can also trigger the same job immediately and append to the same log stream.

Windows caveat:
- not all cron expressions map cleanly
- complex cron schedules may expand into multiple task entries
- no-overlap and timeout guarantees are weaker than on macOS and Linux

## Cron Syntax

The plugin uses standard 5-field cron expressions:

```text
┌───────────── minute (0-59)
│ ┌───────────── hour (0-23)
│ │ ┌───────────── day of month (1-31)
│ │ │ ┌───────────── month (1-12)
│ │ │ │ ┌───────────── day of week (0-6, Sunday=0)
│ │ │ │ │
* * * * *
```

Examples:
- `0 9 * * *` -> daily at 9:00 AM
- `0 */6 * * *` -> every 6 hours
- `30 8 * * 1` -> Mondays at 8:30 AM
- `0 9,17 * * *` -> 9 AM and 5 PM daily

Use the exact cron text in tool payloads unless the tool itself is doing natural-language conversion for the user request.

## Attach URL

If the user already has `opencode serve` or `opencode web` running, a job can target that backend with `attachUrl`.

Example:
- "Update the standing-desk job to use attachUrl http://localhost:4096"

Use `attachUrl` only when the user explicitly wants the job attached to an already-running backend.

## Storage Paths

Useful when debugging or explaining where scheduler state lives:
- Job configs: `~/.config/opencode/scheduler/scopes/<scopeId>/jobs/*.json`
- Run history: `~/.config/opencode/scheduler/scopes/<scopeId>/runs/*.jsonl`
- Locks: `~/.config/opencode/scheduler/scopes/<scopeId>/locks/*.json`
- Logs: `~/.config/opencode/logs/scheduler/<scopeId>/*.log`
- Supervisor script: `~/.config/opencode/scheduler/supervisor.pl`
- macOS launchd units: `~/Library/LaunchAgents/com.opencode.job.<scopeId>.*.plist`
- Linux systemd user units: `~/.config/systemd/user/opencode-job-<scopeId>-*.{service,timer}`
- Windows Task Scheduler entries: `\\OpenCode\\opencode-job-<scopeId>-*`

These are diagnostic paths, not the first interface. Prefer tools first.

Legacy note:
- older versions stored jobs in `~/.config/opencode/jobs/*.json`
- older versions used unscoped unit names
- `delete_job` removes both scoped and legacy artifacts

## Troubleshooting Commands

Linux:

```bash
systemctl --user list-timers | rg opencode-job
systemctl --user status opencode-job-<scope>-<slug>.timer
```

Cron fallback:

```bash
crontab -l | rg opencode-scheduler
```

macOS:

```bash
launchctl list | rg opencode
```

Windows:

```bash
schtasks /Query /TN "\\OpenCode\\opencode-job-*"
```

Logs:
- use `job_logs` first
- use shell only if the user explicitly wants filesystem inspection

If jobs are not running:
1. Verify the scheduler backend is installed with the platform command above.
2. Check `job_logs` for the target job.
3. Verify the job's `workdir` has the right `opencode.json` and MCP configuration.

If MCP tools are missing inside the run:
- make sure the job's `workdir` contains the intended `opencode.json`
- do not assume MCP config from some other directory will be visible

## Project Philosophy

Treat the plugin as a thin wrapper around scheduled `opencode run` invocations.

Implications:
- prefer tool calls over direct file edits
- logs are the source of truth for scheduled execution
- explain backend details only after verification
- do not promise resilience features beyond what the plugin actually implements

## Response Rules

When reporting results:
- always give the exact job name
- state the effective scope or `workdir`
- state the cron schedule after create or update
- mention timeout if set
- mention backend only if you actually verified it
- if something is missing, say what is missing instead of inferring success
