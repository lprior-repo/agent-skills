---
name: decomposer
description: "Deterministic BEAM-style supervisor that shreds architecture specs into molecular tasks via plan-shredder loop."
---

# Decomposer Orchestrator

Think of this as the Erlang BEAM supervisor for architectural decomposition.
You are a deterministic skeptical orchestrator. You DO NOT write the tasks directly. You execute this pipeline strictly by launching sub-agents via the `Task` tool.

**Input:** An architecture specification markdown file.
**Output:** A JSON file containing an array of perfectly molecular, `planner`-compliant tasks.

---

## STATE 0: INITIALIZATION
Identify the input spec path and the target output JSON path from the user's prompt (e.g., `.forge/<id>/architecture-spec.md` and `.forge/<id>/final-tasks.json`).
Set up a temporary workspace for the drafting loop:
```bash
WORKSPACE=".decomposition/session_$(date +%s)"
mkdir -p $WORKSPACE
cp <INPUT_SPEC_PATH> $WORKSPACE/spec.md
```
Initialize `$WORKSPACE/STATE.md` with "STATE 1". Update this file at the start of every subsequent state.

---

## STATE 1: DRAFT DECOMPOSITION
**Action:** Launch `general` Sub-Agent via the `Task` tool.
**Prompt:** "Read `$WORKSPACE/spec.md` and `~/.claude/skills/decomposer/molecular-rules.md`. Shred the architecture spec into isolated, molecular tasks (max 1-2 hours each). You must strictly follow the JSON format in `~/.claude/skills/decomposer/handoff-template.md` ensuring ALL sections (ears, contracts, tests, etc.) are filled for CUE validation. NOTE: If this is a massive Epic like 'Add Multiplayer', you must break it down into dozens of tiny tasks. Do not try to compress it. Write the output to `$WORKSPACE/draft-tasks.json`."
**Gate:** The Orchestrator MUST explicitly verify the file exists (`ls $WORKSPACE/draft-tasks.json`). If missing, fail-closed (Abort).

---

## STATE 2: ADVERSARIAL REVIEW (THE IRON SHREDDER)
**Action:** Launch `plan-shredder` Sub-Agent via the `Task` tool.
**Prompt:** "Load the `plan-shredder` skill. Read `$WORKSPACE/draft-tasks.json` and `~/.claude/skills/decomposer/molecular-rules.md`. You are the ruthless Plan Shredder. Evaluate every single drafted task against Munger's Lattices and the Molecular Slicing Rules. Also verify that the JSON structure perfectly matches the handoff template (no missing contracts or EARS). 
If ANY task fails ANY heuristic, write a scathing critique to `$WORKSPACE/defects.md` and output exactly 'STATUS: REJECTED'. 
If all tasks are perfectly molecular and complete, output exactly 'STATUS: APPROVED'."
**Gate:**
- `STATUS: APPROVED`: Proceed to State 4.
- `STATUS: REJECTED`: Proceed to State 3.

---

## STATE 3: THE REPAIR LOOP
**Action:** Launch `general` Sub-Agent via `Task` tool.
**Prompt:** "Read `$WORKSPACE/defects.md`. Your previous decomposition was rejected by the plan-shredder. Re-slice the tasks in `$WORKSPACE/draft-tasks.json` to fix every defect. Ensure no task violates the boundaries, and ensure all planner fields are populated. Reply 'FIXES APPLIED'."
**Gate:** Return to STATE 2 (Re-run Plan Shredder).
**HARD LIMIT:** If looping > 3 times, ABORT.

---

## STATE 4: FINAL HANDOFF
Once the Plan Shredder approves:
1) Verify the JSON format is valid using `jq`.
```bash
cat $WORKSPACE/draft-tasks.json | jq . > /dev/null
```
2) Copy the approved draft to the requested target output path.
```bash
cp $WORKSPACE/draft-tasks.json <TARGET_OUTPUT_PATH>
echo "Decomposition complete: <TARGET_OUTPUT_PATH>"
```

Base directory for this skill: file:///home/lewis/.claude/skills/decomposer

<skill_files>
<file>/home/lewis/.claude/skills/decomposer/molecular-rules.md</file>
<file>/home/lewis/.claude/skills/decomposer/handoff-template.md</file>
</skill_files>
