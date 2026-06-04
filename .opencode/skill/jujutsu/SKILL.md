---
name: jujutsu
description: "JJ (Jujutsu) distributed version control. Use when working with JJ repositories, commits, branches, conflicts, operation log, or multi-agent VCS workflows. JJ is lock-free, has undo capability, and anonymous branches. Triggers: jj, jujutsu, VCS, version control, jj log/jj describe/jj new/jj git push, operation log, undo."
---

# Jujutsu (JJ) Skill

JJ is a **distributed version control system** designed for lock-free concurrency and complete undo capability.

## Why JJ over Git for Multi-Agent?

| Feature | Git | JJ |
|---------|-----|-----|
| **Concurrency** | Locking required, corrupts at scale | Lock-free parallel agents |
| **Undo** | Destructive, permanent | Operation log, undo ANYTHING |
| **Conflicts** | Block until resolved | First-class, commit and resolve later |
| **Branches** | Namespace pollution | Anonymous branches |
| **State** | Confusing index/staging | Auto-committed working copy |

## Core Commands

```bash
# View history
jj log

# Create new change
jj new

# Describe (commit) current change
jj describe -m "feat: description"

# Push to remote (GitHub)
jj git push

# View changes
jj diff

# Undo last operation
jj operation undo

# View operations
jj operation log
```

## Key Concepts

### Operation Log
Every operation is recorded. Undo anything:
```bash
jj operation undo    # Undo last operation
jj operation log     # View operation history
jj operation restore <op-id>  # Restore to specific operation
```

### Anonymous Branches
Workspaces don't pollute branch namespace:
```bash
jj new              # Creates anonymous branch
jj describe -m "wip"  # Add description
```

### Conflicts as Commits
Conflicts are first-class, not blockers:
```bash
# Conflict is automatically committed as a conflict commit
jj resolve          # Resolve conflicts
```

## JSON Output
All commands support `--json` for machine parsing.

## Error Codes
- 0: Success
- 1: Validation error
- 2: Not found
- 3: System error
- 4: External command error

## Integration with Isolate

Isolate wraps JJ to provide workspace isolation:
```bash
isolate spawn <bead-id>   # Creates isolated JJ workspace
isolate done               # Merges back to main
isolate sync               # Rebases on latest main
```

## Common Workflows

### Start Feature Work
```bash
jj new
jj describe -m "feat: my feature"
# ... do work ...
jj git push
```

### Undo a Mistake
```bash
jj operation log    # Find the bad operation
jj operation undo   # Undo last operation
```

### Handle Conflicts
```bash
jj log              # See conflict commits
jj edit <conflict-commit>
# ... resolve files ...
jj resolve
jj describe -m "resolve conflict"
```
