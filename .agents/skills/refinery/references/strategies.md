# Refinery Strategies

## Decision Tree

```
Is branch ahead of main?
├── NO (ahead == 0)
│   └── STALE: delete remote branch (safe, commits already in main)
├── YES, ahead <= 10
│   ├── behind <= 50?
│   │   ├── YES → DIRECT MERGE (git merge --no-edit)
│   │   └── NO → CHERRY-PICK (deep divergence)
│   └── behind > 50?
│       └── CHERRY-PICK (always)
└── YES, ahead > 10
    └── CHERRY-PICK (too many commits for safe merge)
```

## Cherry-Pick Pattern

```bash
# Get commits in chronological order
COMMITS=$(git log --reverse --format="%H" origin/main..origin/$branch)

# Cherry-pick each one
for c in $COMMITS; do
  if ! git cherry-pick $c; then
    # Conflict occurred
    CONFLICTS=$(git diff --name-only --diff-filter=U)
    # Try auto-resolution per conflict resolution rules
    # If still stuck: git cherry-pick --abort && break
  fi
done
```

## Conflict Resolution Examples

### Formatting conflict (keep HEAD)
```bash
# Typically <<<<<<< markers around whitespace/indentation
git checkout --ours <file>
git add <file>
git cherry-pick --continue
```

### New logic from incoming (keep theirs)
```bash
# Incoming adds new test functions, impl blocks, etc.
git checkout --theirs -- <file>
git add <file>
git cherry-pick --continue
```

### Both sides meaningful (SKIP)
```bash
git cherry-pick --abort
echo "SKIPPED: $branch - manual conflict resolution required"
```

## Build Check Commands

| Rig | Command | Pass Indicator |
|-----|---------|---------------|
| veloxide | `moon run :quick` | exit 0 |
| hardline | `cargo check` | exit 0 |
| oya_frontend | `cargo check` | exit 0 |
| twerk | `cargo check` | exit 0 |
| Seshat | `cargo check` | exit 0 |

## Revert Procedure

```bash
# If merge or cherry-pick fails build:
git reset --hard origin/main
# If already pushed (shouldn't happen due to build gate):
git revert HEAD  # Safe revert
```
