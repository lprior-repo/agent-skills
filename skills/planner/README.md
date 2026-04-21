# Planner Skill

Deterministic planning agent that decomposes complex work into atomic beads using the Intent CLI enhanced 16-section template.

## Files

- `SKILL.md` - Skill documentation and algorithm description
- `PROMPT.md` - AI usage guide with examples
- `planner.nu` - Nushell script implementing the state machine
- `BEAD_TEMPLATE.md` - Template reference (symlinked from Intent CLI)
- `enhanced-bead.cue` - CUE schema for validation (symlinked from Intent CLI)

## Quick Start

```bash
# Set script path
P="$HOME/.claude/skills/planner/planner.nu"

# Initialize planning session
nu $P init --session-id my-feature --description "Add user authentication"

# Add tasks (AI generates these)
echo '<task-json>' | nu $P add-task my-feature --task-json -

# Process all tasks (generate, validate, create beads)
nu $P process my-feature

# View results
nu $P report my-feature
```

## State Machine

```
INIT → ADD_TASKS → GENERATE → VALIDATE → CREATE → COMPLETE
```

Each phase is deterministic and produces auditable state in `~/.local/share/planner/sessions/`.

## Integration

- **Templates**: Uses `.beads/BEAD_TEMPLATE.md` from Intent CLI
- **Validation**: Uses `schema/enhanced-bead.cue` for CUE validation
- **Persistence**: Creates beads via `bd create` command
- **State**: YAML session files for idempotency

## Requirements

- Nushell (nu)
- CUE (`cue` command)
- Beads daemon (`bd` command)
- Intent CLI repository (for templates and schemas)

## See Also

- `/tdd15` - TDD implementation workflow
- `/red-queen` - Adversarial testing
- `bd` - Bead database management
