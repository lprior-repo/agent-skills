# OpenCode Scheduler Examples

## Enable The Plugin

```json
{
  "$schema": "https://opencode.ai/config.json",
  "plugin": ["opencode-scheduler"]
}
```

## Natural-Language Examples

These are the kinds of requests this skill should map onto scheduler operations:
- "Schedule a daily job at 9am to search Facebook Marketplace for posters under $100 and send the top 5 deals to my Telegram"
- "Schedule a daily job at 9am to search for standing desks under $300"
- "Schedule a job every Monday at 8am to summarize my GitHub notifications"
- "Schedule a job every 6 hours to check if my website is up and alert me on Slack if it's down"

## Inspect Current Scope

Use this for "show my scheduled jobs" in the current project.

```json
{
  "tool": "list_jobs",
  "args": {
    "format": "json"
  }
}
```

## Inspect All Scopes Including Legacy

Use this for "show all scheduled jobs everywhere".

```json
{
  "tool": "list_jobs",
  "args": {
    "allScopes": true,
    "includeLegacy": true,
    "format": "json"
  }
}
```

## Create A Daily Job

Example shape for a recurring prompt-driven job.

```json
{
  "tool": "schedule_job",
  "args": {
    "name": "daily-github-summary",
    "schedule": "0 8 * * 1-5",
    "prompt": "@scheduled-job-best-practices Summarize my GitHub notifications and write a concise report under outputs/github-summary/. If there is nothing new, say so and exit cleanly.",
    "workdir": "/path/to/project",
    "timeoutSeconds": 900,
    "format": "json"
  }
}
```

Verification follow-up:

```json
{
  "tool": "get_job",
  "args": {
    "name": "daily-github-summary",
    "format": "json"
  }
}
```

## Update An Existing Job

Read first, then update.

```json
{
  "tool": "get_job",
  "args": {
    "name": "daily-github-summary",
    "format": "json"
  }
}
```

```json
{
  "tool": "update_job",
  "args": {
    "name": "daily-github-summary",
    "schedule": "0 9 * * 1-5",
    "attachUrl": "http://localhost:4096",
    "timeoutSeconds": 1200,
    "format": "json"
  }
}
```

## Run A Job Immediately And Check Logs

```json
{
  "tool": "run_job",
  "args": {
    "name": "daily-github-summary",
    "format": "json"
  }
}
```

```json
{
  "tool": "job_logs",
  "args": {
    "name": "daily-github-summary",
    "lines": 200,
    "format": "json"
  }
}
```

## Install The Built-In Prompt Hardening Skill

Use this when the user wants a stronger recurring-job prompt template inside a repo.

```json
{
  "tool": "install_skill",
  "args": {
    "name": "scheduled-job-best-practices",
    "directory": "/path/to/repo",
    "overwrite": false,
    "format": "json"
  }
}
```

## Safe Global Cleanup

Always dry-run first.

```json
{
  "tool": "cleanup_global",
  "args": {
    "confirm": false,
    "includeHistory": false,
    "format": "json"
  }
}
```

Only execute deletion when the user clearly approves it.

```json
{
  "tool": "cleanup_global",
  "args": {
    "confirm": true,
    "includeHistory": false,
    "format": "json"
  }
}
```

## Prompt Review Heuristics

A scheduled prompt is not ready if it:
- asks the model to wait for approval
- relies on interactive web login
- assumes magic placeholders like `__TODAY__`
- sends duplicate notifications on rerun
- never writes durable outputs
- never prints a compact summary at the end

When you see those problems, fetch or install `scheduled-job-best-practices` and harden the prompt before scheduling it.
