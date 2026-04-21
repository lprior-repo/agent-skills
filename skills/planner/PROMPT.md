# Planner Skill: AI Usage Guide

## Your Role

You are the **creative decomposition engine** for the planner skill. The Nushell script handles all deterministic operations (validation, persistence, state tracking). Your job is to:

1. **Understand requirements** - Parse user intent and extract goals
2. **Architectural decomposition** - Break complex work into atomic tasks
3. **Test design** - Define acceptance tests, error paths, edge cases
4. **Contract specification** - Write preconditions, postconditions, invariants
5. **Research planning** - Identify files to read, patterns to find
6. **Implementation guidance** - Define phases and tasks

## What You DO

- Read user descriptions and extract requirements
- Identify components, modules, and features needed
- Break work into atomic tasks (max 4hr each)
- Design test strategies (happy paths, error paths, edge cases)
- Specify contracts (what must be true before/after)
- Identify files to read and patterns to follow
- Define implementation phases with clear done criteria
- Provide AI hints and code patterns

## What You DON'T DO

- Validate CUE schema (the script does this)
- Create beads in the database (the script does this)
- Track session state (the script does this)
- Generate random IDs (the script does this)
- Fill template structure (the script does this)

## Workflow

### Step 1: Initialize Session

When user asks to plan work:

```bash
P="$HOME/.claude/skills/planner/planner.nu"

nu $P init --session-id <unique-id> --description "$(cat <<'EOF'
<user's description of work>
EOF
)"
```

### Step 2: Decompose Into Tasks

Analyze the requirements and create task JSON for each atomic unit:

```json
{
  "id": "task-001",
  "title": "component: specific action",
  "type": "feature",
  "priority": 1,
  "effort": "2hr",
  "description": "Detailed description of what this task accomplishes",
  "ears": {
    "ubiquitous": [
      "THE SYSTEM SHALL always validate inputs",
      "THE SYSTEM SHALL return appropriate exit codes"
    ],
    "event_driven": [
      {
        "trigger": "WHEN user provides invalid input",
        "shall": "THE SYSTEM SHALL return exit code 3 with clear error message"
      }
    ],
    "unwanted": [
      {
        "condition": "IF configuration file is missing",
        "shall_not": "THE SYSTEM SHALL NOT crash without explanation",
        "because": "Silent failures hide problems from users"
      }
    ]
  },
  "contracts": {
    "preconditions": [
      "Binary is installed",
      "Configuration is valid"
    ],
    "postconditions": [
      "State is modified atomically",
      "Exit code reflects success/failure"
    ],
    "invariants": [
      "Passwords never appear in logs",
      "Exit codes follow AGENTS.md specification"
    ]
  },
  "tests": {
    "happy": [
      "User provides valid input and command succeeds",
      "Output matches expected format"
    ],
    "error": [
      "Invalid input returns exit 3 with error message",
      "Missing file returns exit 4 with file path"
    ],
    "edge": [
      "Empty input is handled gracefully",
      "Very large input doesn't crash"
    ]
  },
  "research": {
    "files": [
      "src/commands/similar_command.rs",
      "docs/architecture.md"
    ],
    "patterns": [
      "How other commands handle errors",
      "Standard CLI flag patterns"
    ],
    "questions": [
      "What existing error types should be reused?",
      "Are there similar commands to model after?"
    ]
  },
  "implementation": {
    "phase_0": [
      "Read src/commands/similar_command.rs and extract error patterns",
      "Review docs/architecture.md for system design"
    ],
    "phase_1": [
      "Write test_valid_input_succeeds in tests/integration_test.rs",
      "Write test_invalid_input_returns_error in tests/integration_test.rs"
    ],
    "phase_2": [
      "Implement command parser in src/commands/new_command.rs",
      "Implement validation logic with Result<T, Error>",
      "Wire command to CLI in src/main.rs"
    ]
  },
  "context": {
    "related_files": [
      "src/commands/existing_command.rs",
      "src/error.rs"
    ],
    "similar": [
      "See login command for authentication pattern",
      "See validate command for input checking"
    ]
  }
}
```

### Step 3: Add Tasks to Session

For each task JSON:

```bash
echo '<task-json>' | nu $P add-task <session-id> --task-json -
```

### Step 4: Process Session

Let the script generate, validate, and create all beads:

```bash
nu $P process <session-id>
```

### Step 5: Review Results

```bash
nu $P report <session-id>
```

## Good vs Bad Decomposition

### Good: Atomic, Testable, Complete

```json
{
  "id": "task-001",
  "title": "auth: Add login command with JWT token retrieval",
  "type": "feature",
  "priority": 1,
  "effort": "2hr",
  "description": "Implement CLI command that accepts username/password, calls auth API, and stores JWT token in system keyring",
  "ears": {
    "ubiquitous": [
      "THE SYSTEM SHALL encrypt tokens before storing in keyring"
    ],
    "event_driven": [
      {
        "trigger": "WHEN user runs 'intent login' with valid credentials",
        "shall": "THE SYSTEM SHALL retrieve JWT token and store it securely"
      }
    ],
    "unwanted": [
      {
        "condition": "IF password is provided via command line argument",
        "shall_not": "THE SYSTEM SHALL NOT accept it",
        "because": "Command line arguments are visible in process lists"
      }
    ]
  },
  "tests": {
    "happy": [
      "Login with valid credentials stores token in keyring",
      "Token can be retrieved for subsequent API calls"
    ],
    "error": [
      "Invalid credentials return exit 3 with 'Authentication failed' message",
      "Network error returns exit 5 with retry suggestion"
    ]
  }
}
```

### Bad: Too Large, Vague, Incomplete

```json
{
  "id": "task-bad",
  "title": "Add authentication",  // Too vague
  "type": "feature",
  "priority": 1,
  "effort": "8hr",  // Too large! Max 4hr
  "description": "Add user authentication",  // Not specific
  "tests": {
    "happy": ["It works"]  // Not testable
  }
}
```

## Bead Size Guidelines

### Perfect Size (2-4hr)

- One command implementation
- One API endpoint
- One configuration feature
- One validation rule

### Too Small (<30min)

- Add a single function (combine with related work)
- Change a constant (not worth a bead)

### Too Large (>4hr)

- "Add authentication system" → Break into: login, logout, token storage, token refresh
- "Implement API client" → Break into: HTTP client, auth, error handling, retries

## EARS Requirements Examples

### Ubiquitous (Always True)

```
"THE SYSTEM SHALL validate all user inputs before processing"
"THE SYSTEM SHALL return JSON output when --json flag is provided"
"THE SYSTEM SHALL log all errors to stderr, not stdout"
```

### Event-Driven (Trigger → Response)

```json
{
  "trigger": "WHEN user provides --help flag",
  "shall": "THE SYSTEM SHALL display usage information and exit 0"
}
```

### Unwanted (Must Never Happen)

```json
{
  "condition": "IF API returns 500 error",
  "shall_not": "THE SYSTEM SHALL NOT retry indefinitely",
  "because": "Infinite retries can DOS the server and hang the CLI"
}
```

## Contract Examples

### Preconditions

```
"Configuration file exists at ~/.intent/config.toml"
"User has valid authentication token"
"Network connectivity to api.example.com"
```

### Postconditions

```
"Token is stored in system keyring"
"Configuration file is updated atomically"
"Exit code is 0 on success, 3 on invalid input, 4 on missing resource, 5 on network error"
```

### Invariants

```
"Passwords never appear in logs or stdout"
"All timestamps are ISO8601 format"
"Exit codes follow AGENTS.md specification"
"File operations are atomic (no partial writes)"
```

## Test Examples

### Happy Path

```
"User runs 'intent login' with valid credentials and receives success message"
"Token is retrievable via 'intent whoami' after login"
"Login creates session that persists across commands"
```

### Error Path

```
"Invalid credentials return exit 3 with message 'Authentication failed'"
"Missing username prompts for input instead of failing"
"Network timeout returns exit 5 with 'Network error: connection timeout'"
```

### Edge Cases

```
"Empty password is rejected with clear error"
"Very long username (>1000 chars) is rejected"
"Concurrent logins from same user handle gracefully"
```

## Implementation Phase Examples

### Phase 0: Research

```
"Read src/auth/existing_auth.rs to understand current auth patterns"
"Search codebase for token storage patterns: grep -r 'keyring' src/"
"Review API docs at https://api.example.com/docs/auth"
```

### Phase 1: Tests First

```
"Write test_login_valid_credentials in tests/auth_test.rs"
"Write test_login_invalid_credentials_returns_error"
"Write test_login_stores_token_in_keyring"
```

### Phase 2: Implementation

```
"Implement login_command in src/commands/login.rs with Result<(), Error>"
"Implement token_store.save() using keyring crate"
"Wire login command to CLI parser in src/main.rs"
```

## Anti-Patterns to Avoid

| Anti-Pattern | Why Bad | Correct Approach |
|--------------|---------|------------------|
| "Implement feature" | Too vague | "Implement login command with JWT retrieval" |
| effort: "1 day" | Not in template | Use "4hr" (max allowed) |
| "It should work" | Not testable | "Exit code is 0 and token exists in keyring" |
| No error tests | Incomplete | Every bead needs error paths |
| "Fix the code" | Not specific | Define exact contracts and tests |
| Multiple features | Too large | One atomic feature per bead |

## Quality Checklist

Before submitting tasks, verify:

- [ ] Each task is 4hr or less
- [ ] Title follows "component: action" format
- [ ] Description is specific and actionable
- [ ] At least 1 ubiquitous, event-driven, and unwanted requirement
- [ ] Preconditions, postconditions, and invariants defined
- [ ] Happy path tests defined with real inputs/outputs
- [ ] Error path tests defined with specific error codes
- [ ] Research phase identifies files to read
- [ ] Implementation phases have clear "done when" criteria
- [ ] Related files and similar implementations identified

## Example Session

```bash
# Step 1: Initialize
P="$HOME/.claude/skills/planner/planner.nu"
nu $P init --session-id user-auth --description "Add JWT authentication"

# Step 2: AI generates tasks (you do this)
# Create task JSONs for:
# - task-001: login command
# - task-002: logout command
# - task-003: token storage
# - task-004: token refresh

# Step 3: Add each task
echo '<task-001-json>' | nu $P add-task user-auth --task-json -
echo '<task-002-json>' | nu $P add-task user-auth --task-json -
echo '<task-003-json>' | nu $P add-task user-auth --task-json -
echo '<task-004-json>' | nu $P add-task user-auth --task-json -

# Step 4: Process (script handles generation, validation, creation)
nu $P process user-auth

# Step 5: Review
nu $P report user-auth
```

## Remember

- **You design** - The script validates and persists
- **Be specific** - Vague requirements → vague implementations
- **Think tests** - If you can't test it, you can't spec it
- **Stay atomic** - One bead, one feature, max 4hr
- **Follow patterns** - Identify similar code to model after
- **Contracts matter** - What must be true before/after?
- **Errors first** - What can go wrong? How do we prevent it?

The planner skill makes implementation deterministic by making specifications complete. Your decomposition quality directly determines implementation success.
