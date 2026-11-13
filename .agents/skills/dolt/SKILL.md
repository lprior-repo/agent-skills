---
name: dolt
description: "Dolt data-plane repair for beads. Use when `bd` fails, `.beads/` server mode or embedded mode is suspect, DoltHub remotes drift, tracker state is missing, sync/push/pull fails, permission errors occur, or bead IDs are absent from live Dolt data. Covers all rigs and repo-specific remotes."
allowed-tools:
  - Read
  - Bash
  - Glob
  - Grep
---

# Skill: Dolt for Beads

Dolt is the data plane for beads. Issues, mail, and metadata live in Dolt
databases served by a bd-managed SQL server. The active port and database are
repository metadata, not constants: read `.beads/metadata.json`, `.beads/config.yaml`,
and `bd context`/`bd where` before repairing.

```jsonl
{"k":"meta","s":"dolt","v":"1.0.0","f":"jsonl-progressive"}
{"k":"mission","g":"Keep beads dolt layer operational: server, routing, remotes, metadata."}
{"k":"rule","id":"metadata_required","t":"Every .beads/ MUST have metadata.json with dolt_database matching server. Missing = all bd commands fail."}
{"k":"rule","id":"diagnostics_first","t":"NEVER restart Dolt without capturing goroutine dump + status first. Blind restart destroys hang evidence."}
{"k":"rule","id":"never_rm_dolt_data","t":"NEVER rm -rf ~/.dolt-data/ directories. Use gt dolt cleanup instead."}
{"k":"rule","id":"always_priorlewis43","t":"ALL DoltHub remotes use priorlewis43/ prefix. NEVER use lprior-repo/. The creds auth as priorlewis43."}
{"k":"rule","id":"metadata_is_authoritative","t":"Do not hardcode port 3307 or <rig>-database. Read metadata/config and preserve repo-specific remotes such as priorlewis43/velvet-ballistics."}
{"k":"rule","id":"naming_pattern","t":"Default DoltHub repo name is often <rig>-database, but local AGENTS/config overrides win."}
{"k":"anti_hallucination","id":"no_fabricated_output","t":"NEVER fabricate dolt CLI output, database names, or query results. Run actual commands."}
```

## Rig Registry — ALL DoltHub Remotes

Most rigs use the pattern: `https://doltremoteapi.dolthub.com/priorlewis43/<rig>-database`.
Treat local AGENTS/config as authoritative when it names a different remote.

Repo-specific overrides are authoritative when documented in local AGENTS/config.
For `velvet-ballistics`, the active remote is
`https://doltremoteapi.dolthub.com/priorlewis43/velvet-ballistics` and the
backend must stay in server mode.

| Rig | DoltHub Remote | Dolt DB Name | Source Dir |
|-----|---------------|--------------|------------|
| hardline | `priorlewis43/hardline-database` | `hardline` | `/home/lewis/src/hardline` |
| twerk | `priorlewis43/twerk-database` | `twerk` | `/home/lewis/src/twerk` |
| veloxide | `priorlewis43/veloxide-database` | `veloxide` | `/home/lewis/src/veloxide` |
| velvet-ballistics | `priorlewis43/velvet-ballistics` | `velvet_ballistics` | `/home/lewis/src/velvet-ballistics` |
| Seshat | `priorlewis43/Seshat-database` | `Seshat` | `/home/lewis/src/Seshat` |
| oya-frontend | TBD (DoltHub repo not yet created) | `oya_frontend` | `/home/lewis/src/oya-frontend` |

**CRITICAL**: The DoltHub user is ALWAYS `priorlewis43`. NEVER use `lprior-repo` (that's a GitHub org, not DoltHub).

## Diagnosis Protocol

When ANY `bd` command fails for ANY rig, run these checks IN ORDER:

```bash
# 1. Server alive?
gt dolt status

# 2. metadata.json exists and has correct database?
bd context
bd where

# 3. embeddeddolt trap?
test ! -e <rig-source-dir>/.beads/embeddeddolt

# 4. Dolt remote correct?
(cd <rig-source-dir>/.beads/dolt && dolt remote -v)
# MUST match the repo's authoritative remote; do not assume <rig>-database.
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
  "dolt_server_port": <port-from-bd-context-or-config>
}
EOF
```

### Fix 2: Wrong or Missing Remote

```bash
cd <rig-source-dir>/.beads/dolt
dolt remote remove origin
dolt remote add origin <authoritative-dolthub-https-remote>
dolt push origin main
```

If the local Dolt sync checkout is corrupted (panic on every dolt command),
capture status first and only rebuild the sync checkout when metadata confirms
the live server-mode database is healthy:
```bash
gt dolt status
bd context
bd where
rm -rf <rig-source-dir>/.beads/dolt
mkdir -p <rig-source-dir>/.beads/dolt
(cd <rig-source-dir>/.beads/dolt && dolt init --name Lewis --email priorlewis43@gmail.com)
(cd <rig-source-dir>/.beads/dolt && dolt remote add origin <authoritative-dolthub-https-remote>)
bd dolt push
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
3. Verify the authoritative DoltHub repo exists under `dolthub.com/priorlewis43/`
4. If repo doesn't exist, create it on dolthub.com first

### Fix 6: Corrupted local .beads/dolt/

The local `.beads/dolt/` is only for DoltHub remote sync. The live beads data
is on the bd-managed server-mode database. It is safe to rebuild the sync
checkout only after `bd context`/`bd where` prove server mode is healthy:

```bash
rm -rf <rig-source-dir>/.beads/dolt
mkdir -p <rig-source-dir>/.beads/dolt
cd <rig-source-dir>/.beads/dolt
dolt init --name Lewis --email priorlewis43@gmail.com
dolt remote add origin <authoritative-dolthub-https-remote>
bd dolt push
```

Never force-push DoltHub unless the user explicitly authorizes destructive remote replacement after you show the live `bd` state and remote divergence evidence.

## New Rig Setup

Three files required for beads on a new rig:

1. **metadata.json** — see Fix 1 above (set dolt_database to rig name)
2. **config.yaml** — `prefix: <pfx>`, `issue-prefix: <pfx>`, `dolt.idle-timeout: "0"`
3. **Dolt remote** — create repo on dolthub.com using the local authoritative remote name, then:
   ```bash
   cd <rig-source-dir>/.beads/dolt
   dolt init --name Lewis --email priorlewis43@gmail.com
   dolt remote add origin <authoritative-dolthub-https-remote>
   dolt push origin main
   ```

## Mandatory Verification Gate

After ANY dolt configuration change:

```bash
gt dolt status                              # Server must be healthy
cd <rig-source-dir> && bd ready -n 1        # Must return beads (not error)
cd <rig-source-dir> && bd context           # Must report server mode when the repo requires server mode
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
| `permission denied` | Wrong org prefix (`lprior-repo` instead of `priorlewis43`) | Use the authoritative `priorlewis43/...` remote |
| `permission denied` | Repo doesn't exist on DoltHub | Create at dolthub.com first |
| `no common ancestor` | Local/remote diverged | Stop, show divergence evidence, and ask before any force push |
| `database not found` | Repo doesn't exist on DoltHub | Create it on dolthub.com first |
| `unknown url scheme: dolt://` | Using dolt:// instead of https:// | Use https:// URL |
| `dial tcp [::1]:43001` | DoltHub cred proxy not running | Check `dolt creds check`, restart if needed |
| `corrupted journal` | Local .beads/dolt/ noms DB damaged | Nuke and re-init (Fix 6 above) |
| `non-fast-forward` | Fresh local behind remote | Pull/inspect first; force push only with explicit user authorization |

## Progressive Disclosure

- [references/cli-reference.md](references/cli-reference.md) — Full Dolt CLI command catalog
- [references/beads-integration.md](references/beads-integration.md) — bd + Dolt sync, backup, cross-machine workflows
