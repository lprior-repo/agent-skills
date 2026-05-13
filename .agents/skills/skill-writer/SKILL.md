---
name: skill-writer
description: "Create, strengthen, evaluate, and optimize portable agent skills. Use when users want to draft a new skill, harden or port an existing skill, remove vendor-specific assumptions, benchmark skill behavior, tune trigger descriptions, or package skills for a CLI-agnostic agent environment."
---

# Skill Writer

Create or strengthen skills through a portable lifecycle: capture intent, draft or audit the skill, run realistic evals, compare against a baseline, gather human feedback, iterate, optimize the trigger description, and package only when the result is validated.

## Core Rules

- Treat host-specific commands, slash commands, metadata fields, and runner names as adapters. Do not bake a specific assistant product or CLI into a general skill unless the skill is explicitly for that platform.
- Use neutral terms in portable skills: agent, model, host CLI, skill runner, subagent, workspace, tool, transcript, output, and skill registry.
- The portable frontmatter floor is `name` and `description`. Add optional fields only when the target host documents them, and describe host requirements in `compatibility` or the body instead of inventing metadata.
- Keep `SKILL.md` compact and operational. Move schemas, long examples, scripts, fixtures, and reference material into bundled resources with explicit links.
- Never report invented command output, benchmark data, trigger rates, or eval results. If a run was not executed, say it was not executed.
- Do not create skills that mislead users, hide behavior, exfiltrate data, bypass authorization, or surprise the user beyond the stated purpose.

## Lifecycle

### 1. Locate the User's Stage

First determine whether the user is creating a new skill, strengthening an existing skill, porting a skill between environments, optimizing a trigger description, adding evals, or packaging a finished skill. If the conversation already contains the desired workflow, extract the steps, tool use, corrections, edge cases, inputs, outputs, and success criteria before asking follow-up questions.

### 2. Capture Intent

Answer these before drafting:

1. What should the skill enable an agent to do?
2. When should it trigger, including natural user phrases and contextual cues?
3. What should it not trigger for?
4. What output format or side effects are expected?
5. Which tools, files, services, or credentials are required?
6. Can success be objectively tested, or is human review the right evaluation mode?

Ask only for missing information that changes the design. Research examples, host conventions, and dependencies proactively when tools are available.

### 3. Draft the Skill

Write a skill folder around this shape:

```text
skill-name/
|-- SKILL.md
|-- scripts/       optional deterministic helpers
|-- references/    optional long-form docs and schemas
|-- assets/        optional templates or static files
`-- evals/         optional test prompts and fixtures, excluded from packages when appropriate
```

In `SKILL.md`, include valid YAML frontmatter and imperative instructions. The `description` is the primary trigger surface in most skill systems, so make it specific and a little assertive: what the skill does, when to use it, and near-miss cases where it should or should not win.

Prefer explaining why instructions matter over stacking rigid `MUST` rules. Reserve hard requirements for safety, data integrity, externally visible side effects, output contracts, and verification.

### 4. Create Evals

For objectively checkable skills, create 2-3 realistic initial prompts before large-scale testing. Use prompts a real user would type, including messy details, file paths, ambiguous wording, and edge cases. Save them in `evals/evals.json` when a filesystem is available.

Draft expectations after the intent is clear. Good expectations are hard to satisfy accidentally: they check substance, not just filenames or superficial wording.

For subjective skills, use human review prompts and qualitative rubrics instead of fake precision.

### 5. Run With-Skill and Baseline Cases

Use the strongest runner available in the current environment:

- If subagents or isolated sessions are available, run each eval with the skill and with the baseline in parallel.
- If only a CLI prompt runner is available, run the same prompt once with the skill enabled and once without it, capturing transcripts and outputs.
- If no automation is available, manually execute the evals, record exactly what was done, and mark timing/token fields as unavailable instead of inventing them.

Use `<skill-name>-workspace/iteration-N/` as the default results area. For a new skill, the baseline is no skill. For an existing skill, snapshot the original skill before editing and use that snapshot as the baseline.

Capture these artifacts when possible:

- `eval_metadata.json` with prompt and expectations
- `outputs/` with produced files and `user_notes.md` for uncertainties
- `transcript.md` or equivalent runner log
- `timing.json` with real timing/token data if the runner exposes it
- `grading.json` with pass/fail evidence
- `benchmark.json` and `benchmark.md` for aggregate comparisons

Bundled helpers are available when local files can be executed:

- `scripts/quick_validate.py` validates portable frontmatter and basic structure.
- `scripts/run_eval.py` runs eval prompts through a caller-supplied host command template.
- `scripts/aggregate_benchmark.py` aggregates `grading.json` artifacts.
- `eval-viewer/generate_review.py` creates a local review UI for outputs and feedback.

### 6. Grade and Analyze

Grade each run against expectations using evidence from transcripts and output files. A pass requires real task completion, not surface compliance. Also extract claims made by the executor and verify them where possible.

After grading, analyze patterns the aggregate score hides: weak assertions, high variance, failures that occur in both baseline and with-skill runs, resource overhead, and cases where the skill hurts performance.

If an eval viewer or review generator exists in the target environment, use it. If not, present a concise inline review with links to outputs, grades, benchmark summaries, and open questions for the user.

### 7. Improve the Skill

Revise from evidence and user feedback. Generalize from failures instead of overfitting to a single eval. Remove instructions that cause wasted work. Add scripts only when repeated runs recreate the same deterministic helper logic. Keep the skill lean enough that future agents actually read and follow it.

Repeat the eval loop until the user is satisfied, the feedback is empty, or the changes stop improving outcomes.

### 8. Optimize Trigger Description

After the behavior is stable, tune the `description` field:

- Create roughly 20 trigger eval queries with a balanced mix of should-trigger and should-not-trigger cases.
- Make negative cases hard near-misses, not irrelevant prompts.
- Test against the host's actual skill-selection mechanism when possible.
- Split training and held-out cases if an automated loop is available to avoid overfitting.
- Keep the final description distinctive, intent-focused, and under the target host's length limit.

### 9. Package or Install

Package only after validation. Exclude eval workspaces, caches, generated reports, secrets, and local-only artifacts. Preserve the original name when updating an existing skill unless the user explicitly requests a rename.

Use `scripts/package_skill.py` for local `.skill` archives when Python is available. It intentionally excludes eval workspaces and common local artifacts.

## Part II: Strengthen Existing Skills

When the user asks to harden, improve, strengthen, audit, refactor, modernize, or make an existing skill more reliable, treat the current skill as the baseline. Do not rewrite it from scratch unless the current structure blocks the requested outcome.

### Strengthening Workflow

1. Preserve the skill contract: identify the current purpose, triggers, non-triggers, outputs, side effects, host assumptions, and bundled resources before editing.
2. Find evidence gaps: check whether the skill has evals, verification commands, output contracts, safety boundaries, examples, and package exclusions that prove the behavior.
3. Tighten the trigger surface: make the description more specific, add natural synonyms, remove broad keywords, and document near-misses that should lose to other skills.
4. Sharpen the operational path: replace vague advice with ordered actions, decision points, failure handling, and explicit final response requirements.
5. Push bulk out of `SKILL.md`: move long references, schemas, examples, prompts, and generated assets into linked support files.
6. Isolate host coupling: convert fixed CLI names, slash commands, registry paths, and product metadata into adapter notes or `compatibility` requirements.
7. Add or improve evals: cover happy path, edge path, near-miss trigger path, and one plausible failure path. Use the original skill snapshot as the baseline.
8. Verify the hardening: run validation and the cheapest relevant eval or manual review. Report skipped checks explicitly.

### Strengthening Moves

- Trigger hardening: split broad skills, add near-miss exclusions, remove keyword-only descriptions, and tune against trigger evals.
- Contract hardening: state required inputs, outputs, side effects, failure behavior, and what evidence proves success.
- Workflow hardening: add prerequisites, checkpoints, rollback notes, and deterministic helper scripts only where they remove repeated manual work.
- Safety hardening: require confirmation for destructive, externally visible, credential, billing, or privacy-sensitive actions.
- Portability hardening: move host-specific behavior behind explicit adapters and keep portable instructions neutral.
- Evaluation hardening: strengthen expectations so plausible bad outputs fail, capture transcripts, and compare against the original skill.
- Maintainability hardening: keep names stable, keep `SKILL.md` compact, link support docs by purpose, and remove stale or contradictory guidance.

### Existing-Skill Final Report

For strengthening work, report:

1. Baseline contract found in the original skill.
2. Weaknesses fixed, grouped by trigger, workflow, safety, portability, eval, and packaging concerns.
3. Files changed and why each change strengthens the skill.
4. Verification performed against the original or current skill.
5. Remaining risks or follow-up hardening opportunities.

## Required Final Response

When finishing a skill authoring task, report:

1. Files created or changed.
2. The skill contract: purpose, trigger conditions, expected outputs, and host compatibility assumptions.
3. Verification performed with real commands, eval runs, or manual checks.
4. Known gaps, skipped checks, or environment limits.

Additional resources for progressive disclosure:
- [reference.md](reference.md)
- [templates.md](templates.md)
- [checklist.md](checklist.md)
- [examples.md](examples.md)
- [scripts/](scripts/)
- [eval-viewer/](eval-viewer/)
