# agent-skills

Reusable AI agent skills for OpenCode, Claude Code, and compatible toolchains.

## Structure

```text
agent-skills/
├── skills/
│   └── opencode-scheduler/
│       ├── SKILL.md
│       ├── reference.md
│       └── examples.md
├── LICENSE
└── README.md
```

## Included Skills

### `opencode-scheduler`

Operate the `opencode-scheduler` plugin to:
- create recurring jobs
- inspect existing jobs
- update schedules and runtime settings
- trigger jobs immediately
- inspect logs
- install the built-in `scheduled-job-best-practices` skill
- troubleshoot backend issues across `launchd`, `systemd`, `cron`, and `schtasks`

## Installation

### Into `.agents`

```bash
mkdir -p ~/.agents/skills
cp -R skills/opencode-scheduler ~/.agents/skills/
```

### Into `.claude`

```bash
mkdir -p ~/.claude/skills
cp -R skills/opencode-scheduler ~/.claude/skills/
```

### Into a project-local `.claude`

```bash
mkdir -p .claude/skills
cp -R skills/opencode-scheduler .claude/skills/
```

## Notes

- `SKILL.md` uses the OpenCode-style YAML + JSONL format.
- Supporting docs live beside each skill to keep the main skill compact.
- The scheduler skill assumes the `opencode-scheduler` plugin is already installed in `opencode.json`.

## License

MIT
