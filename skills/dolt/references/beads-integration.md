# Beads + Dolt Integration

## How bd Uses Dolt

`bd` (beads) uses Dolt as its storage backend. Every write (create, update, close,
dep) goes through the Dolt SQL server. The `metadata.json` file tells bd which
database to use.

## bd Dolt Sync Commands

```bash
bd dolt push          # Push beads changes to DoltHub remote
bd dolt pull          # Pull beads changes from DoltHub remote
bd dolt commit        # Commit pending beads changes
```

## Directory Structure

```
.beads/
├── config.yaml          — Issue prefix, backup settings
├── metadata.json        — CRITICAL: Dolt connection info (must exist)
├── dolt/
│   └── .dolt/
│       └── config.json  — Dolt remote config (use HTTPS, not dolt://)
├── embeddeddolt/        — DELETE IF PRESENT: causes bd to use empty local DB
├── dolt-server.port     — Port file (should be 3307)
├── routes.jsonl         — Prefix-to-path routing (HQ level only)
└── issues.jsonl         — JSONL backup of issues
```

## Beads Backup

Beads automatically backs up to `issues.jsonl` on every write. The Dolt database
provides version control for all bead changes.

```bash
# View change history
cd /home/lewis/gt/<rig>/mayor/rig/.beads/dolt
dolt log

# Check what changed
dolt diff HEAD~1

# Push to DoltHub for off-machine backup
dolt push origin main
```

## Cross-Machine Sync

```bash
# Before ending work session:
bd dolt push

# When starting on a new machine:
bd dolt pull
```

## Common Failure Modes

| Symptom | Cause | Fix |
|---------|-------|-----|
| "no database selected" | Missing metadata.json | Create with correct dolt_database |
| "unknown url scheme: 'dolt'" | dolt:// remote in config.json | Change to HTTPS remote |
| "embeddeddolt: init schema" | embeddeddolt/ directory exists | rm -rf embeddeddolt/ |
| bd shows wrong beads | Routing to wrong database | Check routes.jsonl + metadata.json |
| Connection refused | Dolt server down | gt dolt start |
| Slow queries (>5s) | Orphan databases bloating server | gt dolt cleanup |
| "no beads configuration found" | metadata.json missing or malformed | Recreate with correct database name |
