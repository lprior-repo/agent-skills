# OpenCode HTTP Server API Reference

**Framework**: Hono (TypeScript)
**Default Port**: 4096
**Auth**: Basic Auth via `OPENCODE_SERVER_PASSWORD` / `OPENCODE_SERVER_USERNAME`
**OpenAPI Spec**: `packages/sdk/openapi.json` (96 operations)
**SDK**: `@opencode-ai/sdk`

---

## Real-Time Communication

### Server-Sent Events (SSE)

| Endpoint | Scope |
|----------|-------|
| `GET /event` | Instance-level events (current directory) |
| `GET /global/event` | Global events (all directories) |

Heartbeat every 30s. Events include: message updates, permission requests, session status, file edits, LSP diagnostics, PTY lifecycle, MCP tool changes, VCS updates.

### WebSocket

| Endpoint | Purpose |
|----------|---------|
| `GET /pty/:ptyID/connect` | Bidirectional PTY terminal I/O |

---

## Global Routes

| Method | Path | Operation | Description |
|--------|------|-----------|-------------|
| GET | `/global/health` | `global.health` | Server health check. Returns `{ healthy, version }` |
| GET | `/global/event` | `global.event` | SSE stream of global events across all directories |
| POST | `/global/dispose` | `global.dispose` | Clean up all OpenCode instances |

---

## Project Routes

| Method | Path | Operation | Description |
|--------|------|-----------|-------------|
| GET | `/project` | `project.list` | List all projects. Query: `directory?` |
| GET | `/project/current` | `project.current` | Get active project |
| PATCH | `/project/:projectID` | `project.update` | Update project (name, icon, commands) |

### Project Type

```typescript
{
  id: string
  worktree: string
  vcs: "git"
  name?: string
  icon?: { url?, override?, color? }
  commands?: { start?: string }
  time: { created: number, updated: number, initialized?: number }
  sandboxes: string[]
}
```

---

## Session Routes

| Method | Path | Operation | Description |
|--------|------|-----------|-------------|
| GET | `/session` | `session.list` | List sessions. Query: `directory?, roots?, start?, search?, limit?` |
| POST | `/session` | `session.create` | Create session. Body: `{ parentID?, title?, permission? }` |
| GET | `/session/status` | `session.status` | Status of all sessions |
| GET | `/session/:id` | `session.get` | Get session |
| DELETE | `/session/:id` | `session.delete` | Delete session |
| PATCH | `/session/:id` | `session.update` | Update session. Body: `{ title?, time?: { archived? } }` |
| GET | `/session/:id/children` | `session.children` | Get forked child sessions |
| GET | `/session/:id/todo` | `session.todo` | Get session TODO list |
| POST | `/session/:id/init` | `session.init` | Initialize session (creates AGENTS.md) |
| POST | `/session/:id/fork` | `session.fork` | Fork at message. Body: `{ messageID? }` |
| POST | `/session/:id/abort` | `session.abort` | Abort active session |
| POST | `/session/:id/share` | `session.share` | Create shareable link |
| DELETE | `/session/:id/share` | `session.unshare` | Remove shareable link |
| GET | `/session/:id/diff` | `session.diff` | File diffs. Query: `messageID` |
| POST | `/session/:id/summarize` | `session.summarize` | AI compaction. Body: `{ providerID, modelID, auto? }` |
| POST | `/session/:id/revert` | `session.revert` | Revert message. Body: `RevertInput` |
| POST | `/session/:id/unrevert` | `session.unrevert` | Restore reverted messages |

### Session Type

```typescript
{
  id: string               // pattern: ^ses.*
  parentID?: string
  directory: string
  title: string
  permission: PermissionRuleset
  time: {
    created: number
    updated: number
    archived?: number
  }
  shareID?: string
}
```

---

## Message Routes

| Method | Path | Operation | Description |
|--------|------|-----------|-------------|
| GET | `/session/:id/message` | `session.messages` | All messages. Query: `limit?` |
| POST | `/session/:id/message` | `session.prompt` | Send message (streaming). Body: `PromptInput` |
| POST | `/session/:id/prompt_async` | `session.prompt_async` | Send async (returns 204). Body: `PromptInput` |
| GET | `/session/:id/message/:msgID` | `session.message` | Get message + parts |
| POST | `/session/:id/command` | `session.command` | Send command. Body: `CommandInput` |
| POST | `/session/:id/shell` | `session.shell` | Execute shell. Body: `ShellInput` |

### PromptInput Type

```typescript
{
  parts: ContentPart[]      // text, image, file attachments
  providerID?: string
  modelID?: string
  agent?: string
  variant?: string
  system?: string
  tools?: Record<string, boolean>
}
```

### Message Types

```typescript
UserMessage {
  id: string
  sessionID: string
  role: "user"
  time: { created: number }
  agent: string
  model: { providerID, modelID }
  summary?: { title?, body?, diffs: FileDiff[] }
  system?: string
  tools?: Record<string, boolean>
  variant?: string
}

AssistantMessage {
  id: string
  sessionID: string
  role: "assistant"
  time: { created: number, completed?: number }
  error?: Error
  parentID?: string
  modelID: string
  providerID: string
  mode?: string
  agent?: string
  path?: { cwd, root }
}
```

---

## Part Routes

| Method | Path | Operation | Description |
|--------|------|-----------|-------------|
| DELETE | `/session/:id/message/:msgID/part/:partID` | `part.delete` | Delete part |
| PATCH | `/session/:id/message/:msgID/part/:partID` | `part.update` | Update part |

### Part Type

```typescript
{
  id: string
  messageID: string
  sessionID: string
  type: "text" | "tool-call" | "tool-result"
  data: any                 // type-specific payload
  time: { created: number }
}
```

---

## Permission Routes

| Method | Path | Operation | Description |
|--------|------|-----------|-------------|
| GET | `/permission` | `permission.list` | List pending permission requests |
| POST | `/permission/:requestID/reply` | `permission.reply` | Reply. Body: `{ reply: Reply, message? }` |

Reply values: `"allow"` | `"deny"` | `"always_allow"`

---

## Question Routes

| Method | Path | Operation | Description |
|--------|------|-----------|-------------|
| GET | `/question` | `question.list` | List pending questions |
| POST | `/question/:requestID/reply` | `question.reply` | Answer. Body: `{ answers: Record<string, string> }` |
| POST | `/question/:requestID/reject` | `question.reject` | Reject question |

---

## Provider Routes

| Method | Path | Operation | Description |
|--------|------|-----------|-------------|
| GET | `/provider` | `provider.list` | List providers. Returns `{ all, default, connected }` |
| GET | `/provider/auth` | `provider.auth` | Auth methods per provider |
| POST | `/provider/:id/oauth/authorize` | `provider.oauth.authorize` | Start OAuth. Body: `{ method }` |
| POST | `/provider/:id/oauth/callback` | `provider.oauth.callback` | OAuth callback. Body: `{ method, code? }` |

---

## Auth Routes

| Method | Path | Operation | Description |
|--------|------|-----------|-------------|
| PUT | `/auth/:providerID` | `auth.set` | Set auth credentials |
| DELETE | `/auth/:providerID` | `auth.remove` | Remove auth credentials |

---

## File Routes

| Method | Path | Operation | Description |
|--------|------|-----------|-------------|
| GET | `/file` | `file.list` | List directory. Query: `path` |
| GET | `/file/content` | `file.read` | Read file. Query: `path` |
| GET | `/file/status` | `file.status` | Git status of files |

---

## Find (Search) Routes

| Method | Path | Operation | Description |
|--------|------|-----------|-------------|
| GET | `/find` | `find.text` | Ripgrep text search. Query: `pattern` |
| GET | `/find/file` | `find.files` | Find files. Query: `query, dirs?, type?, limit?` |
| GET | `/find/symbol` | `find.symbols` | LSP symbol search. Query: `query` |

---

## MCP Routes

| Method | Path | Operation | Description |
|--------|------|-----------|-------------|
| GET | `/mcp` | `mcp.status` | Status of all MCP servers |
| POST | `/mcp` | `mcp.add` | Add MCP server. Body: `{ name, config }` |
| POST | `/mcp/:name/auth` | `mcp.auth.start` | Start OAuth flow |
| POST | `/mcp/:name/auth/callback` | `mcp.auth.callback` | Complete OAuth. Body: `{ code }` |
| POST | `/mcp/:name/auth/authenticate` | `mcp.auth.authenticate` | OAuth start + wait for callback |
| DELETE | `/mcp/:name/auth` | `mcp.auth.remove` | Remove OAuth credentials |
| POST | `/mcp/:name/connect` | `mcp.connect` | Connect server |
| POST | `/mcp/:name/disconnect` | `mcp.disconnect` | Disconnect server |

---

## PTY Routes

| Method | Path | Operation | Description |
|--------|------|-----------|-------------|
| GET | `/pty` | `pty.list` | List PTY sessions |
| POST | `/pty` | `pty.create` | Create PTY. Body: `{ command, args?, cwd?, title?, env? }` |
| GET | `/pty/:id` | `pty.get` | Get PTY details |
| PUT | `/pty/:id` | `pty.update` | Update PTY. Body: `{ title?, size?: { rows, cols } }` |
| DELETE | `/pty/:id` | `pty.remove` | Remove PTY |
| GET | `/pty/:id/connect` | `pty.connect` | **WebSocket** real-time PTY I/O |

### PTY Type

```typescript
{
  id: string               // pattern: ^pty.*
  title: string
  command: string
  args: string[]
  cwd: string
  status: "running" | "exited"
  pid: number
}
```

---

## Config Routes

| Method | Path | Operation | Description |
|--------|------|-----------|-------------|
| GET | `/config` | `config.get` | Get full config |
| PATCH | `/config` | `config.update` | Update config |
| GET | `/config/providers` | `config.providers` | List providers + defaults |

---

## Experimental Routes

| Method | Path | Operation | Description |
|--------|------|-----------|-------------|
| GET | `/experimental/tool/ids` | `tool.ids` | List all tool IDs |
| GET | `/experimental/tool` | `tool.list` | List tools with JSON schemas. Query: `provider, model` |
| POST | `/experimental/worktree` | `worktree.create` | Create git worktree |
| GET | `/experimental/worktree` | `worktree.list` | List worktrees |
| DELETE | `/experimental/worktree` | `worktree.remove` | Remove worktree |
| POST | `/experimental/worktree/reset` | `worktree.reset` | Reset worktree branch |
| GET | `/experimental/resource` | `experimental.resource.list` | Get MCP resources |

---

## TUI Control Routes

| Method | Path | Operation | Description |
|--------|------|-----------|-------------|
| POST | `/tui/append-prompt` | `tui.appendPrompt` | Append text to prompt |
| POST | `/tui/submit-prompt` | `tui.submitPrompt` | Submit current prompt |
| POST | `/tui/clear-prompt` | `tui.clearPrompt` | Clear prompt |
| POST | `/tui/execute-command` | `tui.executeCommand` | Run TUI command. Body: `{ command }` |
| POST | `/tui/show-toast` | `tui.showToast` | Show toast notification |
| POST | `/tui/publish` | `tui.publish` | Publish TUI event |
| POST | `/tui/select-session` | `tui.selectSession` | Navigate to session |
| POST | `/tui/open-help` | `tui.openHelp` | Open help dialog |
| POST | `/tui/open-sessions` | `tui.openSessions` | Open sessions dialog |
| POST | `/tui/open-themes` | `tui.openThemes` | Open themes dialog |
| POST | `/tui/open-models` | `tui.openModels` | Open models dialog |
| GET | `/tui/control/next` | `tui.control.next` | Get next TUI request from queue |
| POST | `/tui/control/response` | `tui.control.response` | Submit TUI response |

---

## Utility Routes

| Method | Path | Operation | Description |
|--------|------|-----------|-------------|
| GET | `/path` | `path.get` | Get paths: `{ home, state, config, worktree, directory }` |
| GET | `/vcs` | `vcs.get` | Get VCS info: `{ branch }` |
| GET | `/command` | `command.list` | List available commands |
| GET | `/agent` | `app.agents` | List agents |
| GET | `/skill` | `app.skills` | List skills |
| GET | `/lsp` | `lsp.status` | LSP server status |
| GET | `/formatter` | `formatter.status` | Formatter status |
| POST | `/log` | `app.log` | Write log. Body: `{ service, level, message, extra? }` |
| POST | `/instance/dispose` | `instance.dispose` | Dispose current instance |
| GET | `/doc` | - | OpenAPI documentation |

---

## Error Handling

### Standard Error Response

```json
{
  "data": {},
  "errors": [{ "message": "...", "code": "..." }],
  "success": false
}
```

### 404 Not Found

```json
{
  "name": "NotFoundError",
  "data": { "message": "Resource not found" }
}
```

### Message Error Types

| Error | Meaning |
|-------|---------|
| `ProviderAuthError` | API key invalid or expired |
| `MessageOutputLengthError` | Output token limit exceeded |
| `MessageAbortedError` | Session was aborted |
| `UnknownError` | Unexpected failure |
| `APIError` | Provider API error |

---

## Event Types (SSE)

Events emitted via `/event` and `/global/event`:

| Category | Events |
|----------|--------|
| Installation | `installation.updated` |
| Project | `project.updated` |
| Session | `session.created`, `session.updated`, `session.deleted`, `session.error` |
| Message | `message.updated`, `message.removed` |
| Part | `part.updated`, `part.removed` |
| Permission | `permission.updated`, `permission.replied` |
| Question | `question.updated`, `question.replied`, `question.rejected` |
| File | `file.edited` |
| PTY | `pty.created`, `pty.updated`, `pty.exited`, `pty.deleted` |
| MCP | `mcp.tools.changed`, `mcp.browser.open.failed` |
| LSP | `lsp.diagnostics` |
| VCS | `vcs.branch.updated` |
| Worktree | `worktree.created`, `worktree.removed` |
| TUI | `tui.prompt.append`, `tui.toast.show`, `tui.command.execute`, etc. |
