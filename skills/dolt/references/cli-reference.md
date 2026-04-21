# Dolt CLI Reference for Beads

All dolt CLI commands run from the rig's dolt directory:
```bash
cd /home/lewis/gt/<rig>/mayor/rig/.beads/dolt
```

## Version Control

```bash
dolt status                    # Check for uncommitted bead changes
dolt log                       # View bead change history
dolt diff                      # See pending changes
dolt diff HEAD~1               # Compare with last commit
dolt add .                     # Stage all changes
dolt commit -m "description"   # Commit bead changes
dolt push origin main          # Push to DoltHub
dolt pull origin main          # Pull from DoltHub
```

## Branch Operations

```bash
dolt branch                    # List branches
dolt checkout -b <branch>      # Create and switch branch
dolt merge <branch>            # Merge branch
```

## SQL Operations

```bash
dolt sql                       # Interactive SQL session
dolt sql -q "SELECT * FROM issues WHERE status='open'"  # Query beads
```

## Testing with dolt_test

```bash
# Create a test
dolt sql -q "INSERT INTO dolt_tests VALUES ('test_name', 'validation', 'SELECT COUNT(*) FROM issues;', 'row_count', '>', '0');"

# Run tests
dolt sql -q "SELECT * FROM dolt_test_run();"

# Run specific test
dolt sql -q "SELECT * FROM dolt_test_run('test_name');"
```

## Checkout Behavior with Running SQL Servers

`dolt checkout` on CLI only affects the shell process. When a dolt sql-server is running,
existing SQL connections keep their current branch until they explicitly switch.

Include `CALL dolt_checkout('<branch>');` at the start of every SQL session/script.
Or use `dolt --branch <branch> sql` to connect to a specific branch.

## System Tables

```sql
SELECT * FROM dolt_log;           -- Commit history
SELECT * FROM dolt_status;        -- Current changes
SELECT * FROM dolt_branches;      -- Branch info
SELECT * FROM dolt_diff_<table>;  -- Table diffs
SELECT * FROM dolt_remotes;       -- Remote configuration
```

## Schema Design

Use UUID keys instead of auto-increment (prevents merge conflicts):

```sql
CREATE TABLE users (
    id varchar(36) default(uuid()) primary key,
    name varchar(255)
);
```
