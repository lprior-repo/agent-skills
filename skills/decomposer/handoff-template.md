# Planner Handoff Template

The Decomposer Drafter must output tasks matching this EXACT JSON schema. The `planner` skill uses CUE validation (`schema/enhanced-bead.cue`) which strictly requires ALL of these fields to be present. Do not skip any arrays or objects.

**CRITICAL RULE:** The `effort` field MUST be `"15min"`, `"30min"`, `"1hr"`, or `"2hr"`. 
**DO NOT USE `"4hr"`**. If a task feels like it needs 4 hours, your slicing failed. Slice it smaller.

```json
[
  {
    "id": "task-001",
    "title": "layer: action description (e.g., db: Add UNIQUE constraint to events)",
    "type": "feature|bug|task|epic|chore",
    "priority": 1,
    "effort": "30min|1hr|2hr", 
    "description": "Specific, molecular description of the isolated change.",
    "clarifications": {
      "resolved": ["Resolved architectural question"],
      "open": [],
      "assumptions": ["Must be backward compatible with existing rows"]
    },
    "ears": {
      "ubiquitous": ["THE SYSTEM SHALL ..."],
      "event_driven": [{"trigger": "WHEN ...", "shall": "THE SYSTEM SHALL ..."}],
      "unwanted": [{"condition": "IF ...", "shall_not": "THE SYSTEM SHALL NOT ...", "because": "..."}]
    },
    "contracts": {
      "preconditions": ["List of verifiable preconditions"],
      "postconditions": ["List of verifiable postconditions"],
      "invariants": ["List of system invariants that must hold"]
    },
    "tests": {
      "happy": ["Isolated test scenario..."],
      "error": ["Expected isolated failure mode..."],
      "edge": ["Edge case..."]
    },
    "research": {
      "files": ["path/to/specific/file.rs"],
      "patterns": ["pattern to mimic"],
      "questions": ["Any open questions"]
    },
    "implementation": {
      "phase_0": ["Research/setup..."],
      "phase_1": ["Write failing test..."],
      "phase_2": ["Implement isolated fix..."]
    },
    "context": {
      "related_files": ["..."],
      "similar": ["..."]
    }
  }
]
```
