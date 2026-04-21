---
name: planner
description: "Deterministic planning agent that decomposes complex work into atomic beads using enhanced 16-section template."
---

# Planner: Deterministic Bead Decomposition Agent

## Script Path

The deterministic state machine lives at:

```
~/.claude/skills/planner/planner.nu
```

All commands use this absolute path:

```bash
P="$HOME/.claude/skills/planner/planner.nu"
nu $P plan --description-file /tmp/plan-desc.md
nu $P validate --bead-id intent-cli-a1b2
nu $P status
```

**Always set `P` at the start of every session.**

## Design Principle: Deterministic Planning

The planner has a clear separation of concerns:

| Operation | Who Does It | Why |
|-----------|------------|-----|
| **Parse requirements** | Script (deterministic) | Extract sections from user input |
| **Break down into tasks** | AI (creative) | Requires understanding of architecture and dependencies |
| **Generate bead YAML** | Script (deterministic) | Template filling from structured input |
| **Validate against schema** | CUE (deterministic) | `cue vet` ensures completeness |
| **Create beads** | `bd create` (deterministic) | Persists to bead database |
| **Track state** | Script (deterministic) | YAML state file for idempotency |

**AI decides**: What tasks are needed, how to break them down, dependencies, test strategies.
**Script ensures**: All beads follow template, pass validation, are properly persisted.

## The Algorithm

```
PLANNER_DETERMINISTIC(user_input, planner_script):

  # PHASE 0: Initialize planning session
  nu $P init --session-id <id>

  # PHASE 1: Parse user input (AI + deterministic)
  # AI: Read user description, understand requirements
  # Script: Extract sections and structure

  requirements = AI_analyze_requirements(user_input)
  nu $P set-requirements <session-id> <requirements.json>

  # PHASE 2: Decompose into atomic tasks (AI)
  # AI generates list of atomic beads needed
  tasks = AI_decompose_into_beads(requirements)

  for task in tasks:
    nu $P add-task <session-id> <task.json>

  # PHASE 3: Generate beads (deterministic)
  # For each task, expand to full 16-section template
  for task in session.tasks:
    bead_yaml = expand_template(task)
    nu $P generate-bead <session-id> <task-id>

  # PHASE 4: Validate beads (deterministic)
  for bead in session.beads:
    # Validate against CUE schema
    cue_result = cue vet schema/enhanced-bead.cue <bead.cue>
    if cue_result.exit_code != 0:
      # Validation failed - report errors
      nu $P mark-invalid <session-id> <bead-id> <error>
    else:
      nu $P mark-valid <session-id> <bead-id>

  # PHASE 5: Create beads in database (deterministic)
  for valid_bead in session.valid_beads:
    bd create --title <bead.title> \
              --type <bead.type> \
              --priority <bead.priority> \
              --description <bead.full_yaml>

    nu $P mark-created <session-id> <bead-id> <bd-id>

  # PHASE 6: Report results
  nu $P report <session-id>
```

## What AI Does vs What Script Does

| Step | AI | Deterministic Script |
|------|----|--------------------|
| Parse user intent | Reads description, extracts goals | — |
| Identify components | Names modules/features needed | — |
| Break into tasks | Creates atomic task list | `add-task` stores them |
| Define contracts | Writes preconditions, postconditions | — |
| Design tests | Writes acceptance tests, E2E tests | — |
| Fill template | Provides content for each section | `expand_template` formats YAML |
| Validate structure | — | `cue vet` checks completeness |
| Check dependencies | — | Validates task ordering |
| Persist beads | — | `bd create` stores to database |
| Track progress | — | YAML state file tracks status |
| Report results | — | Structured output of created beads |

**AI touches**: Understanding requirements, architectural decomposition, test design, technical details.
**Script handles**: Template expansion, validation, persistence, state tracking, idempotency.

## State Machine

```
UNINITIALIZED → init → REQUIREMENTS_PENDING
  → set-requirements → DECOMPOSITION_PENDING
  → add-task (repeat) → GENERATION_PENDING
  → generate-bead (repeat) → VALIDATION_PENDING
  → validate (all beads) → CREATION_PENDING
  → create (all valid beads) → COMPLETE
```

Each phase has deterministic gates and produces auditable state.

## Template Sections (16 Total)

Every bead generated must have all 16 sections from `BEAD_TEMPLATE.md`:

1. **Clarifications** (Section 0) - Resolved questions, open questions, assumptions
2. **EARS Requirements** (Section 1) - Ubiquitous, event-driven, state-driven, optional, unwanted, complex
3. **KIRK Contracts** (Section 2) - Preconditions, postconditions, invariants
4. **Research Requirements** (Section 2.5) - Files to read, patterns to find, prior art
5. **Inversions** (Section 3) - Security, usability, data integrity, integration failures
6. **ATDD Tests** (Section 4) - Happy paths, error paths, edge cases, contract tests
7. **E2E Tests** (Section 5) - Pipeline test, scenarios
8. **Verification Checkpoints** (Section 5.5) - Research, tests, implementation, integration gates
9. **Implementation Tasks** (Section 6) - Phase 0-4 with parallelization markers
10. **Failure Modes** (Section 7) - Symptoms, causes, debugging commands
11. **Anti-Hallucination** (Section 7.5) - Read-before-write rules, API existence checks
12. **Context Survival** (Section 7.6) - Progress files, recovery instructions
13. **Completion Checklist** (Section 8) - Tests, code, CI, documentation
14. **Context** (Section 9) - Related files, similar implementations, patterns
15. **AI Hints** (Section 10) - Do/don't lists, code patterns, constitution

## Validation Rules

All beads must pass CUE validation from `schema/enhanced-bead.cue`:

- ID matches pattern: `^intent-cli-[a-z0-9]+$`
- Title format: `"Component: Description"`
- Type: `feature | bug | task | epic | chore`
- Priority: 0-4
- Effort estimate: `15min | 30min | 1hr | 2hr | 4hr`
- All required sections present with proper structure
- At least one ubiquitous, event-driven, and unwanted requirement
- At least one happy path and one error path test
- At least one invariant defined
- Completion checklist includes standard items

## Command Reference

### Planning Session Commands

```bash
# Initialize a new planning session
nu $P init --session-id my-feature --description "Add user authentication"

# Set parsed requirements
nu $P set-requirements my-feature requirements.json

# Add atomic task to session
nu $P add-task my-feature task.json

# Generate bead YAML from task
nu $P generate-bead my-feature task-001

# Validate bead against CUE schema
nu $P validate my-feature task-001

# Create bead in bd database
nu $P create my-feature task-001

# Process entire session (generate, validate, create all)
nu $P process my-feature

# Show session status
nu $P status my-feature

# Report session results
nu $P report my-feature

# List all planning sessions
nu $P list

# Reset/clear session
nu $P reset my-feature
```

### Query Commands

```bash
# Show all tasks in session
nu $P show-tasks my-feature

# Show generated bead YAML
nu $P show-bead my-feature task-001

# Show validation errors
nu $P show-errors my-feature

# Show created bead IDs
nu $P show-created my-feature
```

## Execution Protocol

### Step 1: Initialize Planning Session

```bash
P="$HOME/.claude/skills/planner/planner.nu"

# Create session
nu $P init --session-id user-auth --description "$(cat <<'EOF'
Add JWT-based user authentication to the CLI.

Requirements:
- Login command with username/password
- Token storage in keyring
- Automatic token refresh
- Logout command
- Token validation on all API calls
EOF
)"
```

### Step 2: AI Decomposes Into Tasks

AI analyzes the description and generates atomic tasks:

```json
[
  {
    "id": "task-001",
    "title": "auth: Add login command with JWT token retrieval",
    "type": "feature",
    "priority": 1,
    "effort": "2hr",
    "description": "Implement login command that accepts credentials and retrieves JWT token",
    "contracts": {
      "preconditions": ["Network connectivity", "Valid API endpoint"],
      "postconditions": ["Token stored in keyring", "User logged in"],
      "invariants": ["Password never logged", "Token encrypted at rest"]
    },
    "tests": {
      "happy": ["Login with valid credentials succeeds"],
      "error": ["Login with invalid credentials returns exit 3", "Login without network returns exit 5"]
    }
  },
  {
    "id": "task-002",
    "title": "auth: Add token storage using system keyring",
    "type": "feature",
    "priority": 1,
    "effort": "1hr",
    ...
  }
]
```

### Step 3: Generate and Validate Beads

```bash
# Process entire session (AI provides task JSONs first)
echo '$task_json_1' | nu $P add-task user-auth -
echo '$task_json_2' | nu $P add-task user-auth -

# Generate, validate, create all beads
nu $P process user-auth

# Check results
nu $P report user-auth
```

### Step 4: Review Results

```bash
# Show created beads
nu $P show-created user-auth

# Each bead is now in bd database:
bd list --filter "auth:"
```

## Quality Gates

Every bead must pass:

- [ ] CUE schema validation (`cue vet`)
- [ ] Template completeness (all 16 sections present)
- [ ] EARS coverage (ubiquitous + event-driven + unwanted requirements)
- [ ] Test coverage (happy path + error path)
- [ ] Contract definition (preconditions + postconditions + invariants)
- [ ] Implementation tasks defined (with parallelization markers)
- [ ] Failure modes documented
- [ ] AI hints provided

## Task JSON Format

When AI generates tasks, use this structure:

```json
{
  "id": "task-XXX",
  "title": "component: action description",
  "type": "feature|bug|task|epic|chore",
  "priority": 0-4,
  "effort": "15min|30min|1hr|2hr|4hr",
  "description": "detailed description",
  "ears": {
    "ubiquitous": ["THE SYSTEM SHALL ..."],
    "event_driven": [{"trigger": "WHEN ...", "shall": "THE SYSTEM SHALL ..."}],
    "unwanted": [{"condition": "IF ...", "shall_not": "THE SYSTEM SHALL NOT ...", "because": "..."}]
  },
  "contracts": {
    "preconditions": ["..."],
    "postconditions": ["..."],
    "invariants": ["..."]
  },
  "tests": {
    "happy": ["test scenario..."],
    "error": ["error scenario..."],
    "edge": ["edge case..."]
  },
  "research": {
    "files": ["path/to/file.rs"],
    "patterns": ["existing pattern to find"],
    "questions": ["what to research?"]
  },
  "implementation": {
    "phase_0": ["research task..."],
    "phase_1": ["test task..."],
    "phase_2": ["implementation task..."]
  },
  "context": {
    "related_files": ["path/to/file"],
    "similar": ["reference to similar code"]
  }
}
```

## Anti-Patterns

| Anti-Pattern | Problem | Correct Approach |
|--------------|---------|------------------|
| Beads too large | >4hr effort, multiple features | Break into smaller beads (max 4hr) |
| Missing tests | No error paths defined | Every bead needs happy + error tests |
| Vague contracts | "Should work correctly" | Specific preconditions/postconditions |
| No validation | Skip CUE check | Always validate before creating |
| Manual template | Copy-paste sections | Use script to ensure consistency |
| No dependencies | Tasks done in wrong order | Define task dependencies clearly |

## Integration with Bead System

The planner creates beads that are ready for:

1. **bd ready** - Shows beads ready to implement
2. **bd show** - View full bead specification
3. **tdd15** - TDD implementation workflow
4. **red-queen** - Adversarial testing

All beads generated by planner follow the same 16-section template and can be processed by any Intent CLI skill.

## Session State Format

Planning sessions are stored in `~/.local/share/planner/sessions/<session-id>.yml`:

```yaml
session_id: user-auth
description: "Add JWT-based user authentication..."
created_at: "2026-01-31T12:00:00"
status: COMPLETE
tasks:
  task-001:
    title: "auth: Add login command..."
    status: CREATED
    bead_id: intent-cli-a1b2
  task-002:
    title: "auth: Add token storage..."
    status: CREATED
    bead_id: intent-cli-a1b3
summary:
  total_tasks: 2
  generated: 2
  valid: 2
  created: 2
  failed: 0
```

## Rules of Engagement

1. **Atomic Tasks** - Each bead is a single, implementable unit (max 4hr)
2. **Complete Specifications** - All 16 sections filled with real content
3. **Deterministic Validation** - CUE schema must pass, no exceptions
4. **Idempotency** - Re-running with same input creates same beads
5. **Audit Trail** - Session state tracks all decisions and outcomes
6. **Quality First** - Invalid beads are rejected, not created
7. **AI for Creativity** - Let AI decompose and design, not validate
8. **Script for Rigor** - Let script enforce structure and completeness

---

**Skill Version**: 1.0.0
**Last Updated**: January 2026
**Status**: Production-Ready
**Model**: Deterministic Planning — AI designs, script validates and persists
