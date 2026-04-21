# agent-skills

Reusable AI agent skills for OpenCode, Claude Code, and compatible toolchains.

This repo mirrors the contents of `~/.agents/skills`.

## Structure

```text
agent-skills/
├── skills/
│   ├── arch-design-qa/
│   ├── async-rust-reviewer/
│   ├── gastown/
│   ├── opencode-scheduler/
│   ├── truth-serum/
│   └── ...
├── LICENSE
└── README.md
```

## Included Skills

The repo mirrors your full `~/.agents/skills` collection.

### Architecture And Planning

| Skill | What it is trying to do |
| --- | --- |
| `arch-design-qa` | Acts like a ruthless architectural product owner that pressure-tests domain models, invariants, edge cases, and failure modes before implementation. |
| `arch-spec-to-beads` | Takes an `architecture-spec.md`, decomposes it, and persists validated beads through the bead pipeline. |
| `architectural-drift` | Checks for codebase drift, oversized files, and violations of Scott Wlaschin-style DDD structure. |
| `decomposer` | Shreds architecture specs into smaller, molecular tasks using a BEAM-style supervisor approach. |
| `plan-shredder` | Stress-tests decomposition plans and attacks weak planning with constraint-driven review. |
| `planner` | Produces deterministic, atomic bead plans using the enhanced planning template. |
| `master` | Coordinates sub-agent-driven development by acting as the top-level GoMasterOrchestrator control plane. |
| `go-skill` | Runs the BEAM-style state machine that moves work from top-priority bead through execution to landing. |
| `doc-to-beads` | Reads an existing document, derives the architecture/spec, and immediately turns it into persisted beads. |
| `beads` | Provides execution doctrine and workflow guidance for the `beads` issue tracker. |

### Rust And Engineering Discipline

| Skill | What it is trying to do |
| --- | --- |
| `functional-rust` | Enforces functional-first Rust with zero-panic discipline, strong layering, and reliability-focused implementation. |
| `async-rust-reviewer` | Reviews async Rust for spawn discipline, stream usage, cancellation safety, Send/Sync hygiene, and observability. |
| `rust-contract` | Produces Rust contracts, invariants, and Given/When/Then behavior plans before implementation. |
| `scott-ddd-refactor` | Refactors code toward Scott Wlaschin-style type-driven design so illegal states become unrepresentable. |
| `moon-v2` | Guides Moon v2 build setup for Rust, including CI/CD, caching, and lint gate design. |
| `velocity` | Pushes for fast but disciplined delivery through TDD, functional core practices, and outcome-driven execution. |

### Testing And QA

| Skill | What it is trying to do |
| --- | --- |
| `bdd-enforcer` | Ensures implemented behavior is backed by executable Given/When/Then scenarios and fixes gaps when proof is missing. |
| `hands-on-qa` | Manually tests CLIs, APIs, and interfaces by actually invoking them and reporting only evidence-backed results. |
| `qa-enforcer` | Performs ruthless execution-first QA like a product owner, deeply validating commands, APIs, and real outputs. |
| `red-queen` | Evolves tests adversarially through the Digital Red Queen workflow to ratchet regressions and force validation. |
| `truth-serum` | Audits AI-produced code for hallucinations, weak verification, deleted tests, and missing execution evidence. |
| `black-hat-reviewer` | Serves as a hardline engineering gatekeeper for contracts, constraints, DDD, and reliability standards. |
| `test-planner` | Writes exhaustive Rust test plans spanning unit, BDD, property, and mutation testing. |
| `test-reviewer` | Reviews test plans and suites adversarially, looking for weak assertions, tautologies, and mutation gaps. |
| `test-writer` | Implements exhaustive Rust tests across unit, integration, proptest, and Kani layers. |

### Tooling, Platforms, And Ops

| Skill | What it is trying to do |
| --- | --- |
| `opencode` | Acts as the OpenCode CLI expert for sessions, agents, providers, MCP, config, server mode, and GitHub integration. |
| `opencode-scheduler` | Operates the `opencode-scheduler` plugin for creating, inspecting, updating, running, and troubleshooting recurring jobs. |
| `rtk` | Installs, verifies, initializes, and diagnoses Rust Token Killer integrations across supported AI tools. |
| `jj` | Guides Jujutsu workflows including revsets, rebasing, isolation, and GitHub or Gerrit usage. |
| `landing-skill` | Handles end-of-session quality gates, sync, push, and clean handoff steps. |
| `dolt` | Troubleshoots and operates Dolt-backed bead data when `bd` commands or remotes go wrong. |
| `gastown` | Covers Gas Town rig orchestration, convoy workflows, agent runtime configuration, and cross-rig operations. |

### UI, Desktop, And Framework Skills

| Skill | What it is trying to do |
| --- | --- |
| `dioxus` | Provides framework guidance for Dioxus 0.7 development, debugging, and browser automation workflows. |
| `dioxus-qa` | Performs ruthless QA for Dioxus apps through headless browser automation and DOM validation. |
| `omarchy` | Handles Linux desktop and WM customization for Hyprland, waybar, walker, terminals, themes, and UI configuration. |

### Meta And Authoring Skills

| Skill | What it is trying to do |
| --- | --- |
| `skill-writer` | Authors Claude/OpenCode-style skills using contract-first design and progressive disclosure. |

### Support Directories Also Mirrored

Not everything in `skills/` is a directly invocable skill.

| Directory | Purpose |
| --- | --- |
| `async-rust-reviewer-workspace/` | Evaluation and workspace artifacts mirrored from `~/.agents/skills` support data. |

### Example: `opencode-scheduler`

### Example: `opencode-scheduler`

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
cp -R skills/* ~/.agents/skills/
```

### Into `.claude`

```bash
mkdir -p ~/.claude/skills
cp -R skills/* ~/.claude/skills/
```

### Into a project-local `.claude`

```bash
mkdir -p .claude/skills
cp -R skills/* .claude/skills/
```

## Notes

- `SKILL.md` uses the OpenCode-style YAML + JSONL format.
- Supporting docs live beside each skill to keep the main skill compact.
- Some skills include large supporting references and evaluation workspace artifacts because this repo mirrors `~/.agents/skills` as-is.
- The scheduler skill assumes the `opencode-scheduler` plugin is already installed in `opencode.json`.

## Mirror Guarantee

This repo's `skills/` tree was mirrored recursively from `~/.agents/skills` and verified with a recursive diff to avoid losing nested files or subfolders.

## License

MIT
