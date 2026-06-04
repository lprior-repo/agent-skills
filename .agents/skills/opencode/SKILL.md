---
name: opencode
description: "OpenCode CLI expert. Covers sessions, agents, providers, MCP servers, config, server mode, and GitHub integration."
---

You are an expert in **OpenCode** (v1.1.31+), the AI coding CLI that provides a TUI, headless server, web interface, and desktop app for AI-assisted development. OpenCode is feature-comparable to Claude Code but with multi-provider support, custom agents, and a server architecture.

## Core Mental Model

- **OpenCode = AI coding agent + multi-provider + server architecture** -- one tool gives you TUI, headless server, web UI, and desktop app
- **Sessions** are the unit of work -- each session has messages, tool calls, file diffs, and can be exported/imported/shared
- **Agents** define behavior -- built-in (`build`, `plan`, `explore`, `general`) or custom (`.opencode/agent/*.md`)
- **Providers** are pluggable -- Anthropic, OpenAI, Google, AWS Bedrock, Azure, xAI, Mistral, Groq, OpenRouter, etc.
- **MCP servers** extend capabilities -- local (stdio) or remote (HTTP/SSE) with OAuth support
- **Skills** provide domain expertise -- `.opencode/skill/**/SKILL.md` or `.claude/skills/**/SKILL.md` (compat mode)
- **Commands** are reusable prompts -- `.opencode/command/*.md` with dynamic shell interpolation
- **Plugins** extend hooks and tools -- TypeScript/JavaScript modules via `.opencode/plugin/`

## Architecture

```
~/.config/opencode/           # Global config
  opencode.json[c]            # Global settings
~/.local/share/opencode/      # Data (sessions, storage)
  session/{projectID}/        # Session files
  message/{sessionID}/        # Message files
  part/{messageID}/           # Message parts (text, tool calls)
  session_diff/{sessionID}/   # File diffs per session
~/.cache/opencode/            # Cache
~/.local/state/opencode/      # State

project/                      # Project root
  opencode.json[c]            # Project config (searched up tree)
  .opencode/
    agent/*.md                # Custom agents (YAML frontmatter + prompt)
    command/*.md              # Custom commands (with shell interpolation)
    skill/**/SKILL.md         # Custom skills
    tool/*.{ts,js}            # Custom tools (TypeScript/JS)
    plugin/                   # Local plugins
    themes/*.json             # Custom TUI themes
    plans/*.md                # Plan mode files (VCS-backed)
```

## CLI Reference

### Primary Commands

| Command | Purpose |
|---------|---------|
| `opencode` | Start TUI (default) |
| `opencode run [message..]` | Non-interactive single run |
| `opencode serve` | Start headless HTTP server |
| `opencode web` | Start server + open web interface |
| `opencode attach <url>` | Attach to running server |

### `opencode run` -- Non-Interactive Mode

```bash
opencode run "fix the login bug"                    # single message
opencode run -c                                      # continue last session
opencode run -s <session-id>                         # continue specific session
opencode run -m anthropic/claude-sonnet-4-20250514      # specify model
opencode run --agent plan                            # use plan agent
opencode run --format json                           # JSON output
opencode run -f screenshot.png "what's wrong here?"  # attach file
opencode run --title "Auth Fix"                      # set session title
opencode run --variant high                          # reasoning effort variant
opencode run --prompt-file instructions.md           # prompt from file
opencode run --attach --port 3000                    # attach to running server
```

### `opencode serve` -- Headless Server

```bash
opencode serve                                       # default port
opencode serve --port 3000                           # custom port
opencode serve --hostname 0.0.0.0                    # bind all interfaces
opencode serve --mdns                                # enable mDNS discovery
opencode serve --cors "https://example.com"          # CORS whitelist
```

Requires `OPENCODE_SERVER_PASSWORD` for basic auth.

### Authentication

| Command | Purpose |
|---------|---------|
| `opencode auth login [url]` | Log in to provider |
| `opencode auth logout` | Log out |
| `opencode auth list` / `auth ls` | List providers |

### Agent Management

| Command | Purpose |
|---------|---------|
| `opencode agent list` | List all agents |
| `opencode agent create` | Create new agent |
| `opencode agent create --path ./my-agent.md` | From file |
| `opencode agent create --mode primary` | Primary agent |
| `opencode agent create --mode subagent` | Subagent |
| `opencode agent create -m provider/model` | With specific model |
| `opencode agent create --tools bash,read,edit` | Tool restrictions |

### MCP Server Management

| Command | Purpose |
|---------|---------|
| `opencode mcp add` | Add MCP server |
| `opencode mcp list` / `mcp ls` | List servers + status |
| `opencode mcp auth <name>` | OAuth authenticate |
| `opencode mcp auth list` / `auth ls` | List OAuth-capable servers |
| `opencode mcp logout <name>` | Remove OAuth credentials |
| `opencode mcp debug <name>` | Debug OAuth connection |

### Session Management

| Command | Purpose |
|---------|---------|
| `opencode session list` | List sessions |
| `opencode session list -n 20` | Limit count |
| `opencode session list --format json` | JSON output |
| `opencode export [sessionID]` | Export session as JSON |
| `opencode import <file>` | Import session from JSON/URL |

### Model Management

| Command | Purpose |
|---------|---------|
| `opencode models` | List all available models |
| `opencode models anthropic` | Models for specific provider |
| `opencode models --verbose` | Detailed model info |
| `opencode models --refresh` | Refresh model list |

### GitHub Integration

| Command | Purpose |
|---------|---------|
| `opencode pr <number>` | Fetch PR, checkout branch, start opencode |
| `opencode github install` | Install GitHub Actions agent |
| `opencode github run` | Run GitHub agent locally |
| `opencode github run --event <json>` | With event payload |

### Statistics

| Command | Purpose |
|---------|---------|
| `opencode stats` | Token usage and cost stats |
| `opencode stats --days 30` | Specific time range |
| `opencode stats --tools` | Tool usage breakdown |
| `opencode stats --models` | Model usage breakdown |
| `opencode stats --project` | Current project only |

### Debugging

| Command | Purpose |
|---------|---------|
| `opencode debug config` | Show resolved config |
| `opencode debug paths` | Show global paths |
| `opencode debug skill` | List all skills |
| `opencode debug agent <name>` | Agent config details |
| `opencode debug lsp` | LSP debugging |
| `opencode debug rg` | Ripgrep debugging |
| `opencode debug file` | Filesystem debugging |
| `opencode debug scrap` | List known projects |
| `opencode debug snapshot` | Snapshot debugging |

### Maintenance

| Command | Purpose |
|---------|---------|
| `opencode upgrade` | Upgrade to latest |
| `opencode upgrade 1.2.0` | Specific version |
| `opencode upgrade -m curl` | Upgrade method (curl/npm/pnpm/bun/brew) |
| `opencode uninstall` | Uninstall opencode |
| `opencode uninstall --keep-config` | Keep config files |
| `opencode uninstall --dry-run` | Preview removal |
| `opencode completion` | Generate shell completions |

### ACP (Agent Client Protocol)

| Command | Purpose |
|---------|---------|
| `opencode acp` | Start ACP server |
| `opencode acp --port 3001` | Custom port |

### Global Flags

| Flag | Purpose |
|------|---------|
| `-h/--help` | Show help |
| `-v/--version` | Show version |
| `--print-logs` | Print logs to stderr |
| `--log-level DEBUG` | Set log level (DEBUG/INFO/WARN/ERROR) |

## Deep CLI Execution Model

OpenCode is best understood as a server-backed coding agent with several frontends:

- `opencode` launches the TUI frontend.
- `opencode run` sends one prompt non-interactively.
- `opencode serve` exposes the same engine over HTTP.
- `opencode attach` connects a UI/CLI client to an existing server.
- Sessions are the durable unit of work: messages, tool calls, diffs, permissions, and metadata hang off a session.

### Core Flow

OpenCode's normal execution path is roughly:

1. Resolve config from global, project, and environment sources.
2. Resolve working directory and project root.
3. Load agents, skills, tools, plugins, MCP servers, provider credentials, model defaults, and permissions.
4. Create or resume a session.
5. Send a user message into that session.
6. The selected agent calls tools, asks for permissions/questions if needed, and streams events.
7. Session data, messages, parts, and diffs are persisted.
8. The client exits, stays interactive, or keeps serving depending on the subcommand.

For bead/go-skill automation, `opencode run` is the main entry point:

```bash
opencode run --agent build --title "go-skill VB-123" \
  "Use the go-skill skill. Run the full lifecycle for bead VB-123."
```

To feed follow-up commands into the same session:

```bash
opencode run -c "Continue from the current go-skill state."
```

Or target a known session:

```bash
opencode run -s <session-id> "Repair the State 6 proof-reviewer blocker."
```

### `opencode` -- TUI Frontend

Default command. Starts the terminal UI.

```bash
opencode
opencode /some/project
opencode -m provider/model
opencode --agent build
opencode -c
opencode -s <session-id>
```

What it does:

- Starts an interactive client in the terminal.
- Creates or resumes a session.
- Lets you type prompts, approve tools, answer questions, inspect diffs, and continue work.
- Uses the same backend/session machinery as `run` and `serve`.

Use the TUI when active human supervision is wanted. It is not required for headless bead execution.

### `opencode run [message..]` -- Headless Prompt Execution

Single-shot, non-TUI execution.

```bash
opencode run "fix the parser bug"
opencode run --format json "summarize this repo"
opencode run -c "continue"
opencode run -s <session-id> "next step"
opencode run --agent build "do the work"
opencode run --command my-command "arg text"
```

Important flags:

- `--agent <name>` selects the primary/custom agent.
- `--model provider/model` overrides model.
- `--format json` emits raw JSON events, best for logs and automation.
- `-c`, `--continue` resumes the latest session.
- `-s`, `--session` resumes a specific session.
- `--fork` copies a prior session before continuing.
- `--title` names the session.
- `-f`, `--file` attaches files.
- `--attach <url>` sends the run to an existing server.
- `--dir` sets the project directory, especially useful with `--attach`.
- `--dangerously-skip-permissions` auto-approves permissions not explicitly denied.

For feeding commands, this is the key pattern:

```bash
opencode run --title "go-skill bead-123" \
  "Use go-skill. Start bead bead-123."

opencode run -c \
  "Proceed to the next state after validating the current gate."

opencode run -c \
  "The verifier failed. Repair according to go-skill routing and rerun the gate."
```

Each `run` invocation is just another message into a session unless it starts fresh. Feeding OpenCode commands headlessly means appending messages to the same persisted session with `-c` or `-s <session-id>`.

### `opencode serve` -- Headless HTTP Server

Starts a headless HTTP server.

```bash
OPENCODE_SERVER_PASSWORD=secret opencode serve --port 4096
```

What it does:

- Runs OpenCode's backend without opening a TUI.
- Exposes sessions, messages, events, permissions, questions, config, files, PTY, and other APIs.
- Enables remote clients, automation, browser UI, or `opencode run --attach`.

Use this for a long-lived agent service:

```bash
OPENCODE_SERVER_PASSWORD=secret opencode serve --port 4096

opencode run --attach http://localhost:4096 \
  "Use go-skill. Run bead VB-123."
```

### `opencode attach <url>` -- Attach To Existing Server

Attaches a local client to an existing server.

```bash
opencode attach http://localhost:4096
opencode attach http://localhost:4096 -c
opencode attach http://localhost:4096 -s <session-id>
```

What it does:

- Connects to a running `opencode serve`.
- Lets you interact with sessions hosted by that server.
- Supports auth via `--username`, `--password`, or environment variables.

Use this when the server is already running somewhere else.

### `opencode web` -- Browser Frontend

Starts the server and opens the web interface.

```bash
opencode web --port 4096
```

What it does:

- Combines `serve` plus browser UI.
- Uses the same backend with a browser frontend instead of a terminal frontend.
- Helps with visual session management, but is not needed for headless automation.

### `opencode acp` -- Agent Client Protocol

Starts an Agent Client Protocol server.

```bash
opencode acp --port 3001
```

What it does:

- Exposes OpenCode through ACP.
- Targets editor/agent-client integrations.
- Provides a protocol-specific agent backend surface similar in purpose to server mode.

Use this when another tool wants to drive OpenCode as an agent backend.

### `opencode mcp` -- Model Context Protocol Servers

Manages MCP servers.

```bash
opencode mcp list
opencode mcp add
opencode mcp auth <name>
opencode mcp logout <name>
opencode mcp debug <name>
```

What it does:

- MCP servers add external tools and capabilities.
- Local MCP servers usually run as subprocesses.
- Remote MCP servers connect over network transports.
- OAuth-capable MCPs need `mcp auth`.

MCP tools become callable by agents, subject to OpenCode permissions and configuration. For go-skill work, MCP is only relevant if the lifecycle depends on external systems exposed through MCP.

### `opencode providers` / `opencode auth` -- Provider Credentials

Manages AI provider credentials.

```bash
opencode providers list
opencode providers login
opencode providers logout

opencode auth list
opencode auth login
opencode auth logout
```

What it does:

- Configures credentials for Anthropic, OpenAI, Google, and other providers.
- Provider credentials feed model availability.
- Config can also use environment references such as `{env:ANTHROPIC_API_KEY}`.

Model names use `provider/model`:

```bash
opencode run -m openai/gpt-5.5 "..."
```

### `opencode models [provider]` -- Model Catalog

Lists available models.

```bash
opencode models
opencode models openai
opencode models --verbose
opencode models --refresh
```

What it does:

- Shows models OpenCode knows how to route to.
- `--verbose` includes metadata like pricing and capabilities.
- `--refresh` updates the model cache.

Use this when a run fails with model lookup or provider issues.

### `opencode agent` -- Agent Profiles

Manages agents.

```bash
opencode agent list
opencode agent create
```

What agents are:

- An agent is a named behavior profile.
- It defines prompt/persona, model, mode, tool permissions, and step limits.
- Built-ins include `build`, `plan`, `general`, and `explore`.
- Custom agents live in config or `.opencode/agent/*.md`.

For go-skill:

```bash
opencode run --agent build \
  "Use the go-skill skill. Run bead VB-123."
```

If a dedicated go-skill agent exists:

```bash
opencode run --agent go-skill \
  "Run bead VB-123 through the full lifecycle."
```

Skills and agents are different concepts:

- Skill: domain instructions loaded into context.
- Agent: execution profile that determines behavior, permissions, model, and mode.

### `opencode debug` -- Diagnostics

Diagnostic commands.

```bash
opencode debug config
opencode debug paths
opencode debug skill
opencode debug agent <name>
opencode debug lsp
opencode debug rg
opencode debug file
opencode debug startup
opencode debug info
```

What it does:

- Shows resolved config.
- Shows loaded skills and agents.
- Helps debug file search, LSP, ripgrep, project detection, and startup.
- Proves OpenCode sees the right skill, agent, and config before automation.

For go-skill readiness:

```bash
opencode debug skill
opencode debug agent build
opencode debug config
```

### `opencode session` -- Session Inventory

Manages sessions.

```bash
opencode session list
opencode session delete <session-id>
```

What sessions are:

- Durable conversations with state.
- They include messages, tool calls, file diffs, and metadata.
- `run -c` resumes the latest session.
- `run -s <id>` resumes an exact session.

For feeding commands:

```bash
opencode session list
opencode run -s <id> "Continue go-skill State 8."
```

### `opencode export [sessionID]` -- Export Sessions

Exports session data.

```bash
opencode export <session-id> > session.json
opencode export <session-id> --sanitize > session-redacted.json
```

What it does:

- Serializes a session for backup, sharing, review, or migration.
- `--sanitize` redacts sensitive transcript and file data.

Use this when an audit trail of a go-skill run is needed.

### `opencode import <file>` -- Import Sessions

Imports a session.

```bash
opencode import session.json
opencode import https://share-url
```

What it does:

- Restores an exported/shared session into local OpenCode storage.
- Helps reproduce or continue someone else's run.

### `opencode stats` -- Usage And Cost

Shows token, cost, and tool usage.

```bash
opencode stats
opencode stats --days 7
opencode stats --models
opencode stats --tools 20
opencode stats --project ""
```

What it does:

- Summarizes usage across projects and sessions.
- Helps identify costly models, long runs, and heavy tool usage.

For go-skill, this matters because full lifecycle runs can be expensive.

### `opencode github` -- GitHub Agent

Manages the GitHub agent integration.

```bash
opencode github install
opencode github run
```

What it does:

- Installs/runs OpenCode's GitHub automation.
- Targets issue and PR event handling.
- Stays separate from local bead/go-skill work unless beads are wired to GitHub workflows.

### `opencode pr <number>` -- PR Checkout Workflow

Fetches and checks out a GitHub PR, then starts OpenCode.

```bash
opencode pr 123
```

What it does:

- Uses GitHub repository context.
- Checks out the PR branch.
- Starts OpenCode for review or changes.

This is interactive-oriented and is not the main path for bead automation.

### `opencode plugin <module>` -- Plugin Installer

Installs a plugin and updates config.

```bash
opencode plugin some-npm-plugin
opencode plugin some-npm-plugin --global
opencode plugin some-npm-plugin --force
```

What plugins do:

- Extend OpenCode with hooks, tools, providers, and config behavior.
- Can intercept events, tool execution, chat params, permissions, and more.
- Are installed from npm modules or local project plugin config.

Plugin installation changes config, so treat it as setup, not routine bead execution.

### `opencode db` -- Local Database Tooling

Database tooling.

```bash
opencode db path
opencode db "select * from session limit 5" --format json
opencode db migrate
```

What it does:

- Opens or queries OpenCode's SQLite database.
- Prints the DB path.
- Migrates older JSON data into SQLite.

Use this for low-level debugging, not normal operation.

### `opencode completion` -- Shell Completion

Generates shell completions.

```bash
opencode completion
```

What it does:

- Emits shell completion script.
- Supports shell integration setup.
- Does not participate in agent execution.

### `opencode upgrade [target]` -- Upgrade CLI

Upgrades OpenCode.

```bash
opencode upgrade
opencode upgrade 1.2.0
opencode upgrade -m npm
```

What it does:

- Updates the installed CLI.
- Supports methods like `curl`, `npm`, `pnpm`, `bun`, `brew`, `choco`, and `scoop`.

This changes the installed tool, so only use it intentionally.

### `opencode uninstall` -- Remove CLI

Removes OpenCode.

```bash
opencode uninstall --dry-run
opencode uninstall --keep-config
opencode uninstall --keep-data
opencode uninstall --force
```

What it does:

- Removes installed OpenCode files.
- Can preserve config or session data.
- `--dry-run` previews removal.

### Deep Go-Skill Headless Pattern

For a full go-skill run on a bead without opening the TUI, use:

```bash
opencode run --format json \
  --title "go-skill VB-123" \
  "Use the go-skill skill. Run the full go-skill lifecycle for bead VB-123. Follow all state gates, use the isolated workspace, record evidence, and do not skip verifier/reviewer gates." \
  > go-skill-VB-123.jsonl
```

Then feed it more commands in the same session:

```bash
opencode run -c --format json \
  "Continue from the current go-skill state. If blocked, route repair according to the go-skill state machine." \
  >> go-skill-VB-123.jsonl
```

For a long-lived server-backed run:

```bash
OPENCODE_SERVER_PASSWORD=secret opencode serve --port 4096

opencode run --attach http://localhost:4096 \
  --password secret \
  --format json \
  --title "go-skill VB-123" \
  "Use the go-skill skill. Run the full lifecycle for bead VB-123."
```

Core distinction:

- `run`: best for scripted, one-shot, CI-style prompting.
- `run -c` / `run -s`: best for feeding more commands to the same work.
- `serve`: best for a persistent backend.
- `attach`: best for controlling an already-running backend.
- TUI/web: best for human supervision.

## Configuration System

### Config File Hierarchy (Low to High Precedence)

1. **Remote well-known**: `${AUTH_URL}/.well-known/opencode`
2. **Global**: `~/.config/opencode/opencode.json[c]`
3. **Custom path**: `OPENCODE_CONFIG` env var
4. **Project**: `opencode.json[c]` (searched up directory tree)
5. **Inline**: `OPENCODE_CONFIG_CONTENT` env var (highest)

### Config Schema

```jsonc
{
  "$schema": "https://opencode.ai/config.json",

  // Models
  "model": "provider/model-id",         // Default model
  "small_model": "provider/model-id",   // For titles/summaries
  "default_agent": "build",             // Default agent

  // User
  "username": "lewis",
  "theme": "default",

  // Sharing
  "share": "manual",                    // "manual" | "auto" | "disabled"

  // Updates
  "autoupdate": true,                   // true | false | "notify"

  // Providers
  "provider": {
    "anthropic": {
      "options": { "apiKey": "{env:ANTHROPIC_API_KEY}" }
    }
  },
  "disabled_providers": [],
  "enabled_providers": [],              // Whitelist mode

  // Agents
  "agent": {
    "my-agent": {
      "model": "anthropic/claude-sonnet-4-20250514",
      "prompt": "You are a specialist...",
      "mode": "subagent",
      "steps": 20
    }
  },

  // MCP Servers
  "mcp": {
    "local-server": {
      "type": "local",
      "command": ["node", "server.js"],
      "environment": { "KEY": "value" },
      "timeout": 5000
    },
    "remote-server": {
      "type": "remote",
      "url": "https://mcp.example.com/mcp",
      "headers": { "Authorization": "Bearer ..." }
    }
  },

  // Permissions
  "permission": {
    "*": "ask",
    "read": "allow",
    "edit": { "src/**": "allow", "*.env": "deny" },
    "bash": { "git*": "allow", "rm*": "deny" }
  },

  // Commands
  "command": {},

  // Plugins
  "plugin": ["@opencode/my-plugin"],

  // LSP
  "lsp": {},                            // false to disable

  // File Watching
  "watcher": { "ignore": ["node_modules"] },

  // Instructions
  "instructions": ["./AGENTS.md"],      // Additional prompt files

  // Compaction
  "compaction": {
    "auto": true,                       // Auto-compact when context full
    "prune": true                       // Prune old tool outputs
  },

  // TUI
  "tui": {
    "scroll_speed": 3,
    "diff_style": "auto"               // "auto" | "stacked"
  },

  // Server
  "server": {
    "port": 3000,
    "hostname": "127.0.0.1",
    "mdns": false,
    "cors": []
  },

  // Experimental
  "experimental": {
    "hook": {},
    "batch_tool": false,
    "primary_tools": [],
    "mcp_timeout": 5000
  }
}
```

### Variable Substitution

- `{env:VAR_NAME}` -- Replace with environment variable
- `{file:path/to/file}` -- Inline file contents

## Agent Configuration

### Built-in Agents

| Agent | Mode | Purpose |
|-------|------|---------|
| `build` | Primary | Full-access coding agent (default) |
| `plan` | Primary | Read-only planning, can write to `.opencode/plans/` |
| `general` | Subagent | General-purpose research and multi-step tasks |
| `explore` | Subagent | Fast read-only codebase exploration |

### Custom Agent Format (`.opencode/agent/*.md`)

```markdown
---
description: When to invoke this agent
model: anthropic/claude-sonnet-4-20250514
temperature: 0.7
mode: "subagent"
steps: 20
color: "#FF5733"
hidden: false
disable: false
permission:
  read: allow
  edit: deny
  bash:
    "*": ask
    "git*": allow
---

Your system prompt here. This agent specializes in...
```

## Built-in Tools

### Core
| Tool | Purpose |
|------|---------|
| `bash` | Execute shell commands |
| `read` | Read files (supports images, PDFs) |
| `edit` | Edit files (search/replace) |
| `write` | Write new files |
| `glob` | Find files by pattern |
| `grep` | Search file contents (ripgrep) |
| `ls` | List directories |

### Advanced
| Tool | Purpose |
|------|---------|
| `apply_patch` | Apply unified diffs |
| `multiedit` | Edit multiple files at once |
| `task` | Task list management |
| `lsp` | LSP integration (experimental) |
| `batch` | Batch tool execution (experimental) |

### Web & Search
| Tool | Purpose |
|------|---------|
| `webfetch` | Fetch and analyze web pages |
| `websearch` | Web search |
| `codesearch` | Code search |

### Interaction
| Tool | Purpose |
|------|---------|
| `question` | Ask user questions |
| `todowrite` / `todoread` | TODO management |
| `skill` | Invoke skills |
| `plan_enter` / `plan_exit` | Plan mode |

## Custom Tools (`.opencode/tool/*.{ts,js}`)

```typescript
import { tool } from "@opencode-ai/plugin"

export default tool({
  description: "What this tool does",
  args: {
    query: tool.schema.string().describe("Search query"),
  },
  async execute(args, context) {
    // context.sessionID, context.messageID, context.agent
    // context.metadata({ title: "...", metadata: {...} })
    // await context.ask({ permission, patterns, always, metadata })
    return "Tool output"
  }
})
```

## Custom Commands (`.opencode/command/*.md`)

```markdown
---
description: Brief description
agent: build
model: provider/model
subtask: true
---

Do something with the current code.

## Context
<!-- Dynamic shell commands go here using the ! prefix with backticks -->
```

Dynamic sections use the exclamation mark prefix with backticks to inline shell command output.

## MCP Server Configuration

### Local Server (stdio)
```jsonc
{
  "mcp": {
    "my-server": {
      "type": "local",
      "command": ["node", "/path/to/server.js"],
      "environment": { "API_KEY": "{env:MY_KEY}" },
      "enabled": true,
      "timeout": 5000
    }
  }
}
```

### Remote Server (HTTP)
```jsonc
{
  "mcp": {
    "remote": {
      "type": "remote",
      "url": "https://mcp.example.com/mcp",
      "headers": { "Authorization": "Bearer ..." },
      "oauth": {
        "clientId": "...",
        "clientSecret": "...",
        "scope": "read write"
      }
    }
  }
}
```

## Permission System

Permission rules use glob/prefix matching:

```jsonc
{
  "permission": {
    "*": "ask",                    // Default: ask for everything
    "read": "allow",              // Allow all reads
    "edit": {
      "src/**": "allow",          // Allow editing src/
      "*.env": "deny"             // Deny editing .env files
    },
    "bash": {
      "git*": "allow",            // Allow git commands
      "rm*": "deny",              // Deny rm commands
      "*": "ask"                  // Ask for everything else
    },
    "external_directory": {
      "/tmp/*": "allow"           // Allow operations outside project
    }
  }
}
```

Values: `"allow"` | `"ask"` | `"deny"`

## Plugin System

### Plugin Hooks

| Hook | When |
|------|------|
| `event` | Any event fires |
| `config` | Config loaded |
| `chat.message` | Message sent/received |
| `chat.params` | Before API call (modify temp, topP) |
| `chat.headers` | Before API call (add headers) |
| `tool.execute.before` | Before tool runs (modify args) |
| `tool.execute.after` | After tool runs (modify output) |
| `command.execute.before` | Before command runs |
| `permission.ask` | Permission requested |
| `auth` | Authentication needed |

### Plugin Format

Plugins export hooks and tools from TypeScript/JavaScript modules. Install via `plugin` array in config or place in `.opencode/plugin/`.

## Environment Variables

### Core
| Variable | Purpose |
|----------|---------|
| `OPENCODE_CONFIG` | Custom config file path |
| `OPENCODE_CONFIG_DIR` | Additional config directory |
| `OPENCODE_CONFIG_CONTENT` | Inline JSON config |
| `OPENCODE_CLIENT` | Client type (cli/app/desktop) |

### Authentication
| Variable | Purpose |
|----------|---------|
| `OPENCODE_SERVER_PASSWORD` | Server basic auth password |
| `OPENCODE_SERVER_USERNAME` | Server basic auth username |

### Feature Flags
| Variable | Purpose |
|----------|---------|
| `OPENCODE_AUTO_SHARE` | Auto-share sessions |
| `OPENCODE_DISABLE_AUTOUPDATE` | Disable auto-updates |
| `OPENCODE_DISABLE_AUTOCOMPACT` | Disable auto-compaction |
| `OPENCODE_DISABLE_PRUNE` | Disable output pruning |
| `OPENCODE_DISABLE_TERMINAL_TITLE` | Don't set terminal title |
| `OPENCODE_DISABLE_DEFAULT_PLUGINS` | Skip built-in plugins |
| `OPENCODE_DISABLE_LSP_DOWNLOAD` | Don't auto-download LSP servers |
| `OPENCODE_ENABLE_EXPERIMENTAL_MODELS` | Show alpha models |
| `OPENCODE_DISABLE_MODELS_FETCH` | Don't fetch model list |

### Claude Code Compatibility
| Variable | Purpose |
|----------|---------|
| `OPENCODE_DISABLE_CLAUDE_CODE` | Disable all Claude Code compat |
| `OPENCODE_DISABLE_CLAUDE_CODE_PROMPT` | Don't use Claude Code prompts |
| `OPENCODE_DISABLE_CLAUDE_CODE_SKILLS` | Don't load `.claude/skills/` |

### Experimental
| Variable | Purpose |
|----------|---------|
| `OPENCODE_EXPERIMENTAL` | Enable all experimental features |
| `OPENCODE_EXPERIMENTAL_FILEWATCHER` | File watching |
| `OPENCODE_ENABLE_EXA` | Enable Exa search |
| `OPENCODE_EXPERIMENTAL_BASH_MAX_OUTPUT_LENGTH` | Bash output limit |
| `OPENCODE_EXPERIMENTAL_BASH_DEFAULT_TIMEOUT_MS` | Bash timeout |
| `OPENCODE_EXPERIMENTAL_OUTPUT_TOKEN_MAX` | Token limit |
| `OPENCODE_EXPERIMENTAL_LSP_TOOL` | LSP tool |
| `OPENCODE_EXPERIMENTAL_PLAN_MODE` | Plan mode features |
| `OPENCODE_PERMISSION` | JSON permission overrides |

## HTTP Server API

OpenCode exposes a full REST API (96 operations) when running in server mode (`opencode serve`). The API uses Hono, supports Basic Auth, SSE for real-time events, and WebSocket for PTY sessions.

**Full API reference**: See `references/http-api.md`

### Key API Groups

| Group | Routes | Purpose |
|-------|--------|---------|
| Global | `GET /global/health`, `GET /global/event` | Health check, SSE event stream |
| Session | `GET/POST/DELETE /session/*` | CRUD, fork, share, abort, revert |
| Message | `POST /session/:id/message` | Send prompt (streaming response) |
| Permission | `GET/POST /permission/*` | Handle tool permission requests |
| Question | `GET/POST /question/*` | Handle user question prompts |
| Provider | `GET /provider`, OAuth routes | Model provider management |
| File | `GET /file/*`, `GET /find/*` | Read files, ripgrep search, LSP symbols |
| MCP | `GET/POST /mcp/*` | MCP server management + OAuth |
| PTY | `CRUD /pty/*`, `WS /pty/:id/connect` | Pseudo-terminal sessions |
| Config | `GET/PATCH /config` | Configuration CRUD |
| TUI | `POST /tui/*` | Remote TUI control |
| Experimental | `GET /experimental/tool/*`, worktree routes | Tool listing, git worktrees |

### Quick Examples

```bash
# Health check
curl http://localhost:4096/global/health

# List sessions
curl -u opencode:$PASSWORD http://localhost:4096/session

# Send a prompt (streaming)
curl -u opencode:$PASSWORD -X POST http://localhost:4096/session/$SID/message \
  -H 'Content-Type: application/json' \
  -d '{"parts":[{"type":"text","text":"fix the bug"}]}'

# Subscribe to events (SSE)
curl -u opencode:$PASSWORD -N http://localhost:4096/event

# PTY WebSocket
websocat ws://localhost:4096/pty/$PID/connect
```

## Typical Workflows

### Quick One-Shot Task

```bash
opencode run "fix the typo in README.md"
opencode run -m openai/gpt-4.1 "explain the auth flow"
opencode run --format json "list all API endpoints" > endpoints.json
```

### Interactive Development (TUI)

```bash
opencode                                 # start TUI
opencode -m anthropic/claude-sonnet-4-20250514  # with specific model
opencode -c                              # continue last session
opencode -s abc123                       # continue specific session
```

### Headless Server + Web

```bash
# Terminal 1: start server
OPENCODE_SERVER_PASSWORD=secret opencode serve --port 3000

# Terminal 2: attach
opencode attach http://localhost:3000

# Or use web interface
opencode web --port 3000
```

### PR Review

```bash
opencode pr 123                          # checkout PR #123 and start review
```

### Session Export/Import

```bash
opencode export abc123 > session.json    # export session
opencode import session.json             # import on another machine
```

### GitHub Actions Agent

```bash
opencode github install                  # set up GitHub Actions
# PRs and issues now get AI-assisted responses
```

### Custom Agent Workflow

```bash
# Create agent
cat > .opencode/agent/security-reviewer.md << 'EOF'
---
description: Security code review specialist
model: anthropic/claude-sonnet-4-20250514
mode: subagent
permission:
  read: allow
  edit: deny
  bash:
    "git*": allow
    "*": deny
---

You are a security-focused code reviewer. Analyze code for OWASP Top 10
vulnerabilities, credential leaks, injection risks, and authentication flaws.
EOF

# Use it
opencode run --agent security-reviewer "review the auth module"
```

### Multi-Provider Setup

```jsonc
// opencode.jsonc
{
  "provider": {
    "anthropic": { "options": { "apiKey": "{env:ANTHROPIC_API_KEY}" } },
    "openai": { "options": { "apiKey": "{env:OPENAI_API_KEY}" } },
    "google": { "options": { "apiKey": "{env:GOOGLE_API_KEY}" } }
  },
  "model": "anthropic/claude-sonnet-4-20250514",
  "small_model": "openai/gpt-4.1-mini"
}
```

## Anti-Patterns

| Anti-Pattern | Problem | Solution |
|--------------|---------|----------|
| Hardcoding API keys in config | Security risk | Use `{env:VAR}` substitution |
| Using `opencode run` for long tasks | No interactivity | Use TUI or `serve` + `attach` |
| Not setting permissions | Agent has unrestricted access | Configure `permission` per-agent |
| Ignoring `opencode stats` | Cost surprises | Check stats regularly |
| Manual session management | Losing context | Use `-c` to continue, `export`/`import` to share |
| Running without server password | Unauthenticated access | Set `OPENCODE_SERVER_PASSWORD` |
| Skipping model specification | Uses expensive default | Set `model` and `small_model` in config |
| Not using `--format json` | Unparseable output in scripts | Always use `--format json` for automation |
| Creating agents in config only | No system prompt | Use `.opencode/agent/*.md` with frontmatter + prompt body |
| Ignoring compaction | Context window fills up | Enable `compaction.auto: true` |

## Best Practices

### Configuration
- **Use project-level config** (`opencode.jsonc`) for project-specific settings
- **Use global config** (`~/.config/opencode/opencode.jsonc`) for personal preferences
- **Set `small_model`** to a cheap/fast model for titles and summaries
- **Enable `compaction.auto`** to handle long sessions gracefully
- **Use `{env:VAR}` and `{file:path}`** for secrets and dynamic values

### Agents
- **Use `plan` agent first** for complex tasks -- it's read-only and deliberate
- **Create custom agents** for repeated workflows (security review, docs, etc.)
- **Set appropriate permissions** -- deny edit/bash for review-only agents
- **Set `steps` limit** to prevent runaway agents

### Sessions
- **Use `-c` to continue** sessions rather than starting fresh
- **Export important sessions** before they're compacted
- **Use `--title`** for meaningful session names

### Server Mode
- **Always set `OPENCODE_SERVER_PASSWORD`** when using `serve`
- **Use `--mdns`** for LAN discovery in team settings
- **Use `opencode web`** for browser-based access

### Debugging
- **`opencode debug config`** to verify resolved configuration
- **`opencode debug paths`** to find data/config/cache locations
- **`opencode debug agent <name>`** to inspect agent setup
- **`--print-logs --log-level DEBUG`** for troubleshooting

## Troubleshooting

### "Model not found"
```
Cause: Provider not configured or model ID incorrect
Fix:
  opencode models                    # list available models
  opencode models --refresh          # refresh from providers
  opencode debug config              # check provider config
```

### "Permission denied" on tool use
```
Cause: Permission rules blocking tool access
Fix:
  opencode debug config              # check permission section
  # Update opencode.jsonc permission rules
```

### Server won't start
```
Cause: Port in use or missing password
Fix:
  opencode serve --port 3001         # try different port
  OPENCODE_SERVER_PASSWORD=pw opencode serve  # set password
```

### MCP server connection failed
```
Cause: Server not running or OAuth expired
Fix:
  opencode mcp ls                    # check server status
  opencode mcp debug <name>          # debug connection
  opencode mcp auth <name>           # re-authenticate
```

### Session won't continue
```
Cause: Session ID wrong or data corrupted
Fix:
  opencode session list              # find correct session
  opencode session list --format json  # get session IDs
  opencode run -s <correct-id>       # continue with right ID
```

### High token costs
```
Cause: Expensive model, no compaction, long sessions
Fix:
  opencode stats --days 7 --models   # identify costly models
  # Set small_model for titles/summaries
  # Enable compaction.auto and compaction.prune
  # Use cheaper models for simple tasks
```

## Guidelines

- When the user asks to run opencode, check if they want TUI (`opencode`), non-interactive (`opencode run`), or server mode (`opencode serve`)
- When configuring providers, always use `{env:VAR}` for API keys, never hardcode
- When creating agents, use `.opencode/agent/*.md` with YAML frontmatter and markdown prompt body
- When adding MCP servers, verify with `opencode mcp ls` after configuration
- When debugging issues, start with `opencode debug config` and `opencode debug paths`
- When the user mentions costs, point them to `opencode stats --days 7 --models --tools`
- When managing sessions, remind about `-c` for continuation and `export` for preservation
- For CI/automation, always use `opencode run --format json`
- OpenCode reads `.claude/skills/` by default -- disable with `OPENCODE_DISABLE_CLAUDE_CODE_SKILLS`

---

**Skill Version**: 1.0.0
**Last Updated**: January 2026
**OpenCode Version Support**: 1.1.31+
**Status**: Production-Ready
