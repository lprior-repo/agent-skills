---
name: opencode-scheduler
description: "Operate the opencode-scheduler plugin to create, inspect, update, run, troubleshoot, and clean up recurring OpenCode jobs. Use when scheduling recurring agent work, checking job state or logs, installing the scheduler best-practices skill, or debugging scheduler backend issues."
argument-hint: "[schedule request, job name, or scheduler operation]"
disable-model-invocation: true
---

```jsonl
{"kind":"meta","skill":"opencode-scheduler","version":"1.0.0","format":"jsonl-progressive","mode":"manual-workflow"}
{"kind":"input","arguments":"$ARGUMENTS","rule":"Treat arguments as the user's scheduler goal, job name, or operation request. If empty, infer the scheduler task from the conversation and current workspace."}
{"kind":"mission","goal":"Use opencode-scheduler safely and accurately: create resilient recurring jobs, inspect existing jobs, install the built-in best-practices skill when useful, and debug scheduler failures with real evidence."}
{"kind":"rule","id":"tool_first","text":"Prefer the opencode-scheduler tools over hand-editing scheduler files. Use schedule_job, list_jobs, get_job, update_job, delete_job, cleanup_global, run_job, job_logs, get_version, get_skill, and install_skill before low-level shell commands."}
{"kind":"rule","id":"scope_awareness","text":"Treat workdir scope as the primary isolation boundary. Default to the current workspace scope unless the user explicitly wants a different directory, all scopes, or legacy jobs."}
{"kind":"rule","id":"manual_side_effects_only","text":"Only create, update, delete, install, run-now, or clean up jobs when the user explicitly asks. Read-only inspection can proceed directly."}
{"kind":"rule","id":"non_interactive_jobs","text":"Scheduled jobs must be non-interactive. Reject plans that depend on prompts, QR scans, browser logins, approvals, or any question flow during scheduled execution."}
{"kind":"rule","id":"idempotent_jobs","text":"When shaping recurring job prompts, push for idempotent behavior, durable outputs, and a compact end-of-run summary. Recommend the built-in scheduled-job-best-practices skill if the prompt is brittle or duplicate-prone."}
{"kind":"rule","id":"anti_hallucination","text":"Never invent scheduler state, installed backends, job metadata, or logs. Every status claim must come from a scheduler tool call or a real shell command executed in this session."}
{"kind":"workflow","id":"scheduler_flow","steps":["DISCOVER: Run get_version first when plugin/backend details matter. Use list_jobs for the current scope unless the user wants all scopes or legacy jobs. Use get_job before changing an existing job.","PLAN: Identify whether the request is create, inspect, update, run-now, logs, install-skill, cleanup, or backend troubleshooting. For create/update, pin down schedule, workdir, timeout, and whether attachUrl or specific opencode run flags are needed.","EXECUTE: Prefer scheduler tools for the operation. For new recurring prompts that do real work, use get_skill or install_skill for scheduled-job-best-practices and apply its constraints when useful.","VERIFY: After side effects, re-read state with list_jobs or get_job, and inspect logs or immediate run results when relevant. For backend/debug cases, use shell commands only to verify launchd/systemd/cron/schtasks state described in reference.md.","REPORT: Return the exact job name, scope/workdir, schedule, backend if known, verification evidence, and any follow-up risks or missing prerequisites."]}
{"kind":"decision","id":"tool_selection","options":[{"when":"create recurring job","use":["schedule_job"],"then":"verify with list_jobs or get_job"},{"when":"inspect jobs in this workspace","use":["list_jobs"]},{"when":"inspect all scopes or old jobs","use":["list_jobs"],"with":{"allScopes":true,"includeLegacy":true}},{"when":"change an existing job","use":["get_job","update_job"],"then":"verify with get_job or list_jobs"},{"when":"run a job now","use":["run_job"],"then":"follow with job_logs or get_job if needed"},{"when":"view logs","use":["job_logs"]},{"when":"install or inspect the built-in scheduler skill","use":["get_skill","install_skill"]},{"when":"remove scheduler artifacts globally","use":["cleanup_global"],"guard":"Always start with confirm=false unless the user explicitly approves executing cleanup."}]}
{"kind":"output","id":"response_shape","sections":["## Operation - requested action and target job or scope","## Evidence - exact tool or shell results supporting the outcome","## Result - created, updated, or current state including schedule, workdir, timeout, and backend if known","## Risks or Follow-ups - missing env vars, non-interactive concerns, platform caveats, or prompt hardening suggestions"]}
{"kind":"ref","file":"reference.md","use":"Tool catalog, parameter guidance, scope model, backend behavior, storage paths, and troubleshooting commands."}
{"kind":"ref","file":"examples.md","use":"Concrete create, update, run-now, logs, install-skill, and cleanup examples."}
{"kind":"gate","id":"verification_gate","text":"After create, update, delete, install, or run-now operations, you MUST verify resulting state with scheduler tools or real shell evidence before concluding."}
{"kind":"gate","id":"cleanup_guard","text":"Global cleanup must begin with confirm=false unless the user explicitly asks to execute the cleanup."}
{"kind":"gate","id":"prompt_hardening_gate","text":"If the scheduled job prompt is interactive, brittle, or duplicate-prone, recommend or install scheduled-job-best-practices before finalizing."}
```

# OpenCode Scheduler

Use this skill when the user wants to operate the `opencode-scheduler` plugin itself, not just write a prompt that happens to be scheduled.

Read these supporting files as needed:
- [reference.md](reference.md) for tool arguments, scope rules, backend behavior, and troubleshooting.
- [examples.md](examples.md) for concrete tool payloads and prompt patterns.

## Mandatory Verification Gate

Before concluding any side-effecting scheduler task, run the checks that match the operation:

```bash
# Show installed plugin + binary versions when backend details matter
opencode --version

# Linux backend check
systemctl --user list-timers | rg opencode-job

# Cron fallback check
crontab -l | rg opencode-scheduler

# Mac backend check
launchctl list | rg opencode

# Windows backend check
schtasks /Query /TN "\\OpenCode\\opencode-job-*"
```

Also verify state through scheduler tools:
- `list_jobs` after create, delete, or broad changes
- `get_job` after updating one job
- `job_logs` after `run_job` or when diagnosing failures
- `cleanup_global` with `confirm: false` before any destructive cleanup

## Anti-Hallucination Shield

Forbidden:
- Claiming a job exists without `list_jobs` or `get_job`
- Claiming a backend is active without tool output or shell evidence
- Claiming a scheduled run succeeded without `job_logs`, `get_job`, or equivalent evidence
- Making up log lines, scheduler unit names, or stored paths

Required:
- Quote the exact job name and scope or workdir involved
- Prefer `format: "json"` when structured verification helps
- Surface missing prerequisites plainly: env vars, MCP config, network access, backend availability, or non-interactive constraints
