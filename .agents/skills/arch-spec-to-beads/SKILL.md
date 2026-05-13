---
name: arch-spec-to-beads
description: "Master pipeline orchestrator. Takes architecture-spec.md, runs it through Decomposer, and persists validated beads."
---

# Arch-Spec-to-Beads: The Automated Master Pipeline

You are the overarching deterministic supervisor that drives an existing architectural specification into fully validated and persisted molecular beads.

Note: The interactive Architectural specification phase happens BEFORE this orchestrator runs (using the `arch-design-qa` OpenCode agent).

Artifact root: `.beads/forge/<session-id>/`

## Exactly-Once Principle

This orchestrator is **idempotent and resumable**. Every state transition reads the current state first and skips completed work. Re-running the pipeline on the same `architecture-spec.md` is a no-op after the first successful run.

The state file (`STATE.yml`) is the source of truth. **Read before write.** Never assume. Always check.

---

## STATE 0: INITIALIZATION (Idempotent)

### Step 0a: Compute content-addressable session ID

```bash
# Hash the spec content to get a deterministic session ID
# Same spec = same session = no duplicates
SPEC_HASH=$(sha256sum architecture-spec.md | cut -d' ' -f1)
SESSION_ID="forge_${SPEC_HASH:0:16}"
FORGE_DIR=".beads/forge/$SESSION_ID"
STATE_FILE="$FORGE_DIR/STATE.yml"
```

### Step 0b: Check for existing session (resume or new)

```bash
if [ -f "$STATE_FILE" ]; then
  CURRENT_STATE=$(yq '.state' "$STATE_FILE")

  if [ "$CURRENT_STATE" = "3" ]; then
    echo "✅ Session $SESSION_ID already completed. Nothing to do."
    echo "   Run 'bd ready' to see the work."
    exit 0
  fi

  echo "↻ Resuming session $SESSION_ID from STATE $CURRENT_STATE"
  # Jump directly to the stored state — skip everything before it
  # The state dispatcher below handles this
else
  # New session — set up workspace
  mkdir -p "$FORGE_DIR"
  cp architecture-spec.md "$FORGE_DIR/architecture-spec.md"
fi
```

### Step 0c: Write initial state (only for new sessions)

```bash
# Only write if this is a fresh session
if [ ! -f "$STATE_FILE" ]; then
  cat > "$STATE_FILE" <<EOF
state: 1
session_id: $SESSION_ID
spec_hash: $SPEC_HASH
created_at: $(date -Iseconds)
updated_at: $(date -Iseconds)
decomposer:
  status: pending
  attempts: 0
planner:
  status: pending
  tasks_total: 0
  tasks_created: 0
EOF
  echo "✅ New session: $SESSION_ID"
fi
```

---

## STATE DISPATCHER (Read-then-act)

After initialization, **read the state file** and dispatch to the correct state. Never fall through blindly.

```bash
CURRENT_STATE=$(yq '.state' "$STATE_FILE")

case "$CURRENT_STATE" in
  1) # Fall through to STATE 1 below
     ;;
  2) # Fall through to STATE 2 below
     ;;
  3) echo "Already complete."; exit 0 ;;
  *) echo "Unknown state: $CURRENT_STATE"; exit 1 ;;
esac
```

---

## STATE 1: THE DECOMPOSER (Idempotent Gate)

**Pre-condition:** `STATE.yml` exists with `state: 1`.

### Step 1a: Check if decomposer already completed

```bash
if [ -f "$FORGE_DIR/final-tasks.json" ]; then
  TASK_COUNT=$(jq 'length' "$FORGE_DIR/final-tasks.json" 2>/dev/null || echo "0")
  if [ "$TASK_COUNT" -gt 0 ]; then
    echo "↻ Decomposer already completed ($TASK_COUNT tasks). Skipping."
    # Advance state and skip to STATE 2
    yq -i ".state = 2" "$STATE_FILE"
    yq -i ".decomposer.status = \"skipped_already_done\"" "$STATE_FILE"
    yq -i ".decomposer.tasks = $TASK_COUNT" "$STATE_FILE"
    yq -i ".updated_at = \"$(date -Iseconds)\"" "$STATE_FILE"
    # Proceed to STATE 2
  fi
fi
```

### Step 1b: Run decomposer (only if gate not passed)

**Action:** Launch `decomposer` Sub-Agent via the `Task` tool.

**Prompt:** "Load the `decomposer` skill. Read `$FORGE_DIR/architecture-spec.md`. Run your deterministic pipeline (Draft -> Iron Shredder review -> Repair). Output the final JSON task array to `$FORGE_DIR/final-tasks.json`. Note that for large Epics, this may result in dozens or even hundreds of tasks."

### Step 1c: Verify gate (fail-closed)

```bash
if [ ! -f "$FORGE_DIR/final-tasks.json" ]; then
  echo "❌ FAIL-CLOSED: Decomposer did not produce final-tasks.json"
  exit 1
fi

TASK_COUNT=$(jq 'length' "$FORGE_DIR/final-tasks.json" 2>/dev/null || echo "0")
if [ "$TASK_COUNT" -eq 0 ]; then
  echo "❌ FAIL-CLOSED: final-tasks.json is empty or invalid JSON"
  exit 1
fi
```

### Step 1d: Advance state

```bash
yq -i ".state = 2" "$STATE_FILE"
yq -i ".decomposer.status = \"complete\"" "$STATE_FILE"
yq -i ".decomposer.tasks = $TASK_COUNT" "$STATE_FILE"
yq -i ".updated_at = \"$(date -Iseconds)\"" "$STATE_FILE"

echo "✅ Decomposer complete: $TASK_COUNT tasks"
```

---

## STATE 2: THE PLANNER (Idempotent Gate)

**Pre-condition:** `STATE.yml` exists with `state: 2`.

### Step 2a: Check if planner session already exists

```bash
P="$HOME/.claude/skills/planner/planner.nu"
PLANNER_SESSION_EXISTS=$(nu $P list 2>/dev/null | grep -c "$SESSION_ID" || echo "0")
```

### Step 2b: Initialize planner session (only if new)

```bash
if [ "$PLANNER_SESSION_EXISTS" -eq 0 ]; then
  nu $P init --session-id "$SESSION_ID" --description "$(cat "$FORGE_DIR/architecture-spec.md")"
fi
```

### Step 2c: Add tasks (idempotent — planner rejects duplicate task IDs)

```bash
cat "$FORGE_DIR/final-tasks.json" | jq -c '.[]' | while read task; do
  # planner.nu add-task rejects duplicate IDs, so this is safe to re-run
  echo "$task" | nu $P add-task "$SESSION_ID" - 2>/dev/null || true
done
```

### Step 2d: Process, validate, and create beads

```bash
nu $P process "$SESSION_ID"
```

### Step 2e: Verify gate

```bash
REPORT=$(nu $P report "$SESSION_ID" 2>&1)
CREATED=$(echo "$REPORT" | grep -oP 'Successfully created: \K\d+' || echo "0")

if [ "$CREATED" -eq 0 ]; then
  echo "❌ FAIL-CLOSED: No beads were created"
  echo "$REPORT"
  exit 1
fi
```

### Step 2f: Advance state

```bash
TASKS_TOTAL=$(jq 'length' "$FORGE_DIR/final-tasks.json")
yq -i ".state = 3" "$STATE_FILE"
yq -i ".planner.status = \"complete\"" "$STATE_FILE"
yq -i ".planner.tasks_total = $TASKS_TOTAL" "$STATE_FILE"
yq -i ".planner.tasks_created = $CREATED" "$STATE_FILE"
yq -i ".updated_at = \"$(date -Iseconds)\"" "$STATE_FILE"

echo "✅ Planner complete: $CREATED beads created"
```

---

## STATE 3: SUCCESS

```bash
nu $P report "$SESSION_ID"
```

Output to the user:
```
Spec-to-Beads complete. The architecture spec has been shredded into molecular tasks, validated by the Iron Shredder, and persisted by the Planner.
Run `bd ready` to see the work.
```

---

## Failure Modes & Recovery

| Symptom | Cause | Recovery |
|---------|-------|----------|
| `STATE.yml` says state 1, no `final-tasks.json` | Decomposer crashed | Re-run orchestrator — it will retry STATE 1 |
| `STATE.yml` says state 2, planner init fails | Corrupted planner session | `nu $P reset $SESSION_ID` then re-run |
| `STATE.yml` says state 2, some beads failed | Partial planner run | Re-run — `add-task` skips duplicates, `process` retries failed |
| Re-run says "already completed" | Previous run succeeded | Check `bd ready` — work is there |
| `architecture-spec.md` changed | Content hash differs | New session created automatically |

Base directory for this skill: file:///home/lewis/.claude/skills/spec-to-beads
