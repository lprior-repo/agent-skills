---
name: dolt
description: "Dolt database operations for beads. Use when bd commands fail, when setting up new rig beads, fixing dolt remotes, or diagnosing dolt corruption. Covers ALL rigs."
allowed-tools:
  - Read
  - Bash
  - Glob
  - Grep
---

# Skill: Dolt for Beads

Dolt is the data plane for beads. All issues, mail, and metadata live in Dolt
databases served by a shared SQL server on port 3307.

```jsonl
{"k":"meta","s":"dolt","v":"1.0.0","f":"jsonl-progressive"}
{"k":"mission","g":"Keep beads dolt layer operational: server, routing, remotes, metadata."}
{"k":"rule","id":"metadata_required","t":"Every .beads/ MUST have metadata.json with dolt_database matching server. Missing = all bd commands fail."}
{"k":"rule","id":"diagnostics_first","t":"NEVER restart Dolt without capturing goroutine dump + status first. Blind restart destroys hang evidence."}
{"k":"rule","id":"never_rm_dolt_data","t":"NEVER rm -rf ~/.dolt-data/ directories. Use gt dolt cleanup instead."}
{"k":"rule","id":"always_priorlewis43","t":"ALL DoltHub remotes use priorlewis43/ prefix. NEVER use lprior-repo/. The creds auth as priorlewis43."}
{"k":"rule","id":"naming_pattern","t":"DoltHub repo name = <rig>-database. e.g. hardline -> priorlewis43/hardline-database."}
{"k":"anti_hallucination","id":"no_fabricated_output","t":"NEVER fabricate dolt CLI output, database names, or query results. Run actual commands."}
```

## Rig Registry — ALL DoltHub Remotes

Every rig uses the pattern: `https://doltremoteapi.dolthub.com/priorlewis43/<rig>-database`

| Rig | DoltHub Remote | Dolt DB Name | Source Dir |
|-----|---------------|--------------|------------|
| hardline | `priorlewis43/hardline-database` | `hardline` | `/home/lewis/src/hardline` |
| twerk | `priorlewis43/twerk-database` | `twerk` | `/home/lewis/src/twerk` |
| veloxide | `priorlewis43/veloxide-database` | `veloxide` | `/home/lewis/src/veloxide` |
| Seshat | `priorlewis43/Seshat-database` | `Seshat` | `/home/lewis/src/Seshat` |
| oya-frontend | TBD (DoltHub repo not yet created) | `oya_frontend` | `/home/lewis/src/oya-frontend` |

**CRITICAL**: The DoltHub user is ALWAYS `priorlewis43`. NEVER use `lprior-repo` (that's a GitHub org, not DoltHub).

## Diagnosis Protocol

When ANY `bd` command fails for ANY rig, run these checks IN ORDER:

```bash
# 1. Server alive?
gt dolt status

# 2. metadata.json exists and has correct database?
cat <rig-source-dir>/.beads/metadata.json

# 3. embeddeddolt trap?
ls <rig-source-dir>/.beads/embeddeddolt/ 2>/dev/null && echo "DELETE THIS"

# 4. Dolt remote correct?
cd <rig-source-dir>/.beads/dolt && dolt remote -v
# MUST be: origin https://doltremoteapi.dolthub.com/priorlewis43/<rig>-database
```

## Fix Procedures

### Fix 1: Missing metadata.json

```bash
cat > <rig-source-dir>/.beads/metadata.json << 'EOF'
{
  "backend": "dolt",
  "database": "dolt",
  "dolt_database": "<rig-name>",
  "dolt_mode": "server",
  "dolt_server_host": "127.0.0.1",
  "dolt_server_port": 3307
}
EOF
```

### Fix 2: Wrong or Missing Remote

```bash
cd <rig-source-dir>/.beads/dolt
dolt remote remove origin
dolt remote add origin https://doltremoteapi.dolthub.com/priorlewis43/<rig>-database
dolt push origin main
```

If the local dolt database is corrupted (panic on every dolt command):
```bash
rm -rf <rig-source-dir>/.beads/dolt
cd <rig-source-dir>/.beads
dolt init --name Lewis --email priorlewis43@gmail.com dolt
cd dolt
dolt remote add origin https://doltremoteapi.dolthub.com/priorlewis43/<rig>-database
dolt push origin main --force
```

### Fix 3: embeddeddolt trap

```bash
rm -rf <rig-source-dir>/.beads/embeddeddolt
```

### Fix 4: Dolt hang (NOT just restart)

```bash
kill -QUIT $(cat ~/gt/.dolt-data/dolt.pid)      # goroutine dump
gt dolt status 2>&1 | tee /tmp/dolt-hang-$(date +%s).log  # capture state
gt escalate -s HIGH "Dolt: <symptom>"             # THEN escalate
```

### Fix 5: PermissionDenied on push

1. Verify creds: `dolt creds check` — must show `priorlewis43`
2. Verify remote uses `priorlewis43/` prefix, NOT `lprior-repo/`
3. Verify DoltHub repo exists at `dolthub.com/priorlewis43/<rig>-database`
4. If repo doesn't exist, create it on dolthub.com first

### Fix 6: Corrupted local .beads/dolt/

The local `.beads/dolt/` is only for DoltHub remote sync. The live beads data
is on the shared Dolt server (port 3307). It is safe to nuke and re-initialize:

```bash
rm -rf <rig-source-dir>/.beads/dolt
mkdir -p <rig-source-dir>/.beads/dolt
cd <rig-source-dir>/.beads/dolt
dolt init --name Lewis --email priorlewis43@gmail.com
dolt remote add origin https://doltremoteapi.dolthub.com/priorlewis43/<rig>-database
dolt push origin main --force
```

## New Rig Setup

Three files required for beads on a new rig:

1. **metadata.json** — see Fix 1 above (set dolt_database to rig name)
2. **config.yaml** — `prefix: <pfx>`, `issue-prefix: <pfx>`, `dolt.idle-timeout: "0"`
3. **Dolt remote** — create repo on dolthub.com as `priorlewis43/<rig>-database`, then:
   ```bash
   cd <rig-source-dir>/.beads/dolt
   dolt init --name Lewis --email priorlewis43@gmail.com
   dolt remote add origin https://doltremoteapi.dolthub.com/priorlewis43/<rig>-database
   dolt push origin main
   ```

## Mandatory Verification Gate

After ANY dolt configuration change:

```bash
gt dolt status                              # Server must be healthy
cd <rig-source-dir> && bd ready -n 1        # Must return beads (not error)
```

## Push Workflow

```bash
# Always run from the .beads/dolt directory
cd <rig-source-dir>/.beads/dolt

# Push to DoltHub
dolt push origin main
```

## Common Errors and Fixes

| Error | Cause | Fix |
|-------|-------|-----|
| `permission denied` | Wrong org prefix (`lprior-repo` instead of `priorlewis43`) | Use `priorlewis43/<rig>-database` |
| `permission denied` | Repo doesn't exist on DoltHub | Create at dolthub.com first |
| `no common ancestor` | Local/remote diverged | `dolt push origin main --force` |
| `database not found` | Repo doesn't exist on DoltHub | Create it on dolthub.com first |
| `unknown url scheme: dolt://` | Using dolt:// instead of https:// | Use https:// URL |
| `dial tcp [::1]:43001` | DoltHub cred proxy not running | Check `dolt creds check`, restart if needed |
| `corrupted journal` | Local .beads/dolt/ noms DB damaged | Nuke and re-init (Fix 6 above) |
| `non-fast-forward` | Fresh local behind remote | `dolt push origin main --force` |

## Progressive Disclosure

- [references/cli-reference.md](references/cli-reference.md) — Full Dolt CLI command catalog
- [references/beads-integration.md](references/beads-integration.md) — bd + Dolt sync, backup, cross-machine workflows
