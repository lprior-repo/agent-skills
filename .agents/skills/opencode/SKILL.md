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
