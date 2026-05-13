# Skill Templates

## 1. Portable Advisory Skill

```markdown
---
name: <skill-name>
description: "Use this skill when users need <outcome>, including <trigger phrases>, <contexts>, or <nearby synonyms>. Do not use for <important near-miss>."
---

# <Skill Title>

## Purpose

<One paragraph explaining what the skill helps the agent do and why this workflow matters.>

## Workflow

1. Inspect the user's inputs and current repository context.
2. Identify which variant of the workflow applies.
3. Follow the relevant procedure below.
4. Verify the output before responding.

## Output Contract

Return <format>. Include <required sections/files>. If verification cannot run, state why.

## Verification

- Run <command or check> when the required tooling exists.
- Do not invent command output or success claims.
```

## 2. Portable Executable Workflow Skill

```markdown
---
name: <skill-name>
description: "Execute <workflow>. Use when users ask to <side-effecting task>, <deploy/build/convert/run>, or troubleshoot <domain>. Requires <tools>."
compatibility: "Requires filesystem access and <tool/CLI>; host-specific permission metadata may be needed."
---

# <Skill Title>

## Safety Boundary

Confirm before destructive, irreversible, billing-impacting, or externally visible operations unless the user already gave explicit approval.

## Workflow

1. Validate prerequisites.
2. Snapshot or record current state when rollback matters.
3. Execute the smallest safe command sequence.
4. Capture real output.
5. Verify final state.

## Failure Handling

If a command fails, preserve the error, diagnose the likely cause, and choose the next safe step. Do not retry blindly.
```

## 3. Reference Overlay Skill

```markdown
---
name: <skill-name>
description: "Apply <domain/framework> conventions. Use whenever users mention <framework>, <APIs>, <error classes>, or ask design/debugging questions in this domain."
---

# <Skill Title>

## Use This Knowledge

Read only the reference file matching the user's domain:

- `references/<variant-a>.md` for <case A>
- `references/<variant-b>.md` for <case B>

## Decision Rules

1. Prefer existing project conventions over generic examples.
2. Cite exact files or docs used for non-obvious claims.
3. Ask one focused question if the selected variant is ambiguous.
```

## 4. Evals File

```json
{
  "skill_name": "<skill-name>",
  "evals": [
    {
      "id": 1,
      "prompt": "A realistic user prompt with enough context to exercise the skill",
      "expected_output": "What a successful run should produce",
      "files": [],
      "expectations": [
        "Expectation that checks correctness, not just file existence",
        "Expectation that would fail for a plausible bad output"
      ]
    }
  ]
}
```

## 5. Existing Skill Hardening Report

```markdown
## Baseline Contract

- Purpose: <what the original skill was meant to do>
- Triggers: <current trigger language and likely activation phrases>
- Non-triggers: <near-misses that should not load it>
- Outputs: <expected response/files/side effects>
- Host assumptions: <portable requirements and host-specific adapters>

## Weaknesses Found

- Trigger: <specific issue or "None found">
- Workflow: <specific issue or "None found">
- Safety: <specific issue or "None found">
- Portability: <specific issue or "None found">
- Evals: <specific issue or "None found">
- Packaging: <specific issue or "None found">

## Hardening Changes

- `<path>`: <change and why it improves reliability>

## Verification

- `<command or review>`: <real result>

## Remaining Risks

- <follow-up gap, skipped check, or "None known">
```

## 6. Strengthening Eval Additions

```json
{
  "skill_name": "<existing-skill-name>",
  "evals": [
    {
      "id": "happy-path",
      "prompt": "A realistic prompt that should use the skill and complete the core workflow",
      "expectations": [
        "Checks the primary output contract with concrete details"
      ]
    },
    {
      "id": "edge-path",
      "prompt": "A messy or incomplete prompt that should trigger clarification or safe fallback behavior",
      "expectations": [
        "Checks the skill handles ambiguity without inventing missing facts"
      ]
    },
    {
      "id": "near-miss",
      "prompt": "A prompt that shares vocabulary but should not use this skill",
      "should_trigger": false,
      "expectations": [
        "Checks the trigger boundary or selection evidence when the host exposes it"
      ]
    }
  ]
}
```

## 7. Trigger Eval Set

```json
{
  "queries": [
    {
      "id": 1,
      "query": "realistic user prompt that should load the skill",
      "should_trigger": true
    },
    {
      "id": 2,
      "query": "near-miss user prompt that shares vocabulary but should not load the skill",
      "should_trigger": false
    }
  ]
}
```

## 8. Runner Adapter Command

```sh
SKILL_WRITER_RUNNER='agent-run --skill {active_skill_path} --prompt-file {prompt_file} --output-dir {output_dir}' \
python scripts/run_eval.py . --evals evals/evals.json --trigger-regex 'selected skill: <skill-name>'
```

Replace `agent-run` and the trigger regex with the current host's real adapter. Do not invent a pass rate if the host runner cannot expose skill-selection evidence.

## 9. Final Report Template

```markdown
## Skill Contract

- Purpose: <what the skill enables>
- Triggers: <when it should load>
- Non-triggers: <near-misses>
- Outputs: <expected response/files>
- Compatibility: <host/tool assumptions>

## Files Changed

- `<path>`: <why changed>

## Verification

- `<command or eval>`: <result>

## Gaps

- <skipped check or known limitation, or "None known">
```
