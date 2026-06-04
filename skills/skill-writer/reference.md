# Skill Writer Reference

## Portable Frontmatter

Use the smallest frontmatter that works across hosts:

```yaml
---
name: example-skill
description: "Use this skill when users need a specific outcome in a defined context."
---
```

Required fields:

- `name`: lowercase kebab-case, command-safe, stable across updates
- `description`: primary trigger surface; include what the skill does, when to use it, and distinctive trigger language

Portable optional fields when the target host accepts them:

- `license`: package/distribution metadata
- `compatibility`: human-readable requirements such as shell, Python, browser, network, or specific CLIs
- `metadata`: host-neutral structured metadata
- `allowed-tools`: only when the host enforces tool permissions; keep least privilege

Avoid host-only fields unless the target environment documents them. If a field controls manual-only invocation, model-only invocation, context isolation, agent selection, or hooks, treat it as a host adapter and explain it in `compatibility` or a host-specific appendix.

## Description Design

The description is usually all the agent sees before deciding whether to load the skill. Write it as trigger guidance, not a passive summary.

Strong description pattern:

```text
Create and validate release notes from Git history. Use when users ask for changelogs, release summaries, PR-to-release aggregation, or reviewer-ready release notes, especially when they provide a version range, branch, tag, or GitHub PR list.
```

Good descriptions:

- State the user intent the skill handles
- Include natural synonyms and messy phrasing users might type
- Include enough boundary language to avoid obvious near-miss triggers
- Stay under the strictest known host length limit, commonly 1024 characters

Weak descriptions:

- Say only "helps with X"
- List implementation details instead of user intent
- Depend on the user knowing the skill name
- Trigger on broad keywords that belong to many other skills

## Skill Body Structure

Keep `SKILL.md` operational and readable:

1. Mission or purpose
2. Activation/boundary rules not already covered by the description
3. Workflow steps
4. Output contract
5. Verification gates
6. Links to bundled resources

Move bulky material out of `SKILL.md`:

- `references/`: long docs, API notes, schemas, decision tables
- `scripts/`: deterministic helpers, validators, packagers, report generators
- `assets/`: templates, static HTML, icons, seed files
- `evals/`: prompts, fixtures, expected outputs, rubrics

For large references, add a table of contents and tell the agent when to read each section.

## Existing Skill Strengthening Audit

Use this audit before changing an existing skill. The goal is to make the current contract stronger without losing useful behavior.

Contract inventory:

- Purpose: what job the skill currently claims to do
- Trigger surface: description wording, natural trigger phrases, and near-misses
- Inputs: files, prompts, services, credentials, or repository state the skill expects
- Outputs: final response shape, generated files, side effects, and failure reports
- Host assumptions: command names, registry paths, metadata fields, tool permissions, and environment variables
- Evidence: evals, transcripts, benchmarks, verification commands, examples, and human feedback

Strengthening categories:

| Category | Weak signal | Stronger replacement |
| --- | --- | --- |
| Trigger | Broad keywords or name-only activation | Intent phrases, synonyms, and near-miss exclusions |
| Workflow | Advice-only prose | Ordered actions, decision points, and stop conditions |
| Output | "Summarize results" | Required sections, files, evidence, and skipped-check reporting |
| Safety | Silent side effects | Confirmation gates and explicit risk boundaries |
| Portability | Fixed host command or metadata | Adapter note, compatibility requirement, or caller-supplied template |
| Evals | No tests or weak expectations | Real prompts with assertions that catch plausible wrong outputs |
| Maintenance | Long mixed `SKILL.md` | Compact entry file with linked references, scripts, assets, and evals |

When hardening, prefer small targeted edits. Preserve the existing `name`, directory path, user-facing purpose, and working helpers unless there is clear evidence they are wrong.

Suggested hardening order:

1. Validate the current skill and record existing failures.
2. Snapshot or preserve the original text as the baseline for comparison.
3. Fix unsafe or misleading behavior first.
4. Tighten the description and near-miss boundaries.
5. Strengthen workflow and output contracts.
6. Add evals or improve weak expectations.
7. Move bulky details into support files.
8. Re-run validation and compare against the baseline.

## Host Adapter Matrix

| Capability | Portable behavior | If unavailable |
| --- | --- | --- |
| Subagents or isolated runs | Run with-skill and baseline evals independently, preferably in parallel | Run serially with the same prompt and capture transcripts |
| CLI prompt runner | Execute eval prompts with explicit skill path/enabled state | Manual execution with documented steps |
| Browser or viewer | Generate a review UI for human feedback | Present outputs and grades inline |
| Timing/token telemetry | Save real values to `timing.json` | Mark fields unavailable |
| Packager | Build a distributable archive after validation | Leave the skill folder in place and report the path |
| Tool permissions | Encode least privilege in host metadata | Document required tools in `compatibility` |

## Bundled Script Interfaces

The scripts in this skill are host agnostic. They never assume a fixed assistant CLI or skill registry. Runner integration is provided through command templates.

Command template environment variables:

- `SKILL_WRITER_RUNNER`: host command used by `scripts/run_eval.py` and `scripts/run_loop.py` to execute a user prompt.
- `SKILL_WRITER_MODEL_CMD`: model command used by `scripts/improve_description.py` and `scripts/run_loop.py` to propose a revised trigger description from evidence.

Common placeholders available to runner templates:

- `{prompt}` or `{query}`: the eval prompt, shell-quoted
- `{prompt_file}`: path to a file containing the prompt
- `{skill_path}`: path to the skill under test
- `{active_skill_path}`: path to the enabled skill for with-skill runs, or empty for no-skill runs
- `{baseline_skill_path}`: optional old skill snapshot
- `{configuration}` or `{mode}`: current configuration name such as `with_skill` or `without_skill`
- `{output_dir}`: directory where the runner should place generated files
- `{run_dir}`: directory for all artifacts from this run
- `{model}`: opaque model label, if the host command needs one

Example runner template:

```sh
agent-run --skill {active_skill_path} --prompt-file {prompt_file} --output-dir {output_dir}
```

The template is executed through the local shell with placeholder values quoted. Prefer file placeholders such as `{prompt_file}` over inline `{prompt}` for long or multiline prompts.

Script entry points:

- `scripts/quick_validate.py <skill_path>`: validate `SKILL.md` frontmatter and portable structure.
- `scripts/run_eval.py <skill_path> --evals evals/evals.json --command-template '<template>'`: run with-skill and baseline evals.
- `scripts/aggregate_benchmark.py <workspace>`: write `benchmark.json` and `benchmark.md` from `grading.json` artifacts.
- `scripts/improve_description.py <skill_path> --results results.json --command-template '<template>'`: propose a new description from evidence.
- `scripts/run_loop.py <skill_path> trigger-evals.json --trigger-regex '<regex>'`: iterate trigger-description optimization with explicit runner and improver adapters.
- `scripts/package_skill.py <skill_path> -o dist`: create a `.skill` archive after validation.
- `eval-viewer/generate_review.py <workspace> --static`: generate a portable human review page.

No script should fabricate trigger decisions. `run_eval.py` only writes `grading.json` for trigger cases when `--trigger-regex` is supplied and the eval item has `should_trigger`.

## Eval Artifacts

Preferred workspace layout:

```text
example-skill-workspace/
|-- skill-snapshot/             optional original baseline
`-- iteration-1/
    |-- benchmark.json
    |-- benchmark.md
    `-- eval-name/
        |-- eval_metadata.json
        |-- with_skill/
        |   |-- transcript.md
        |   |-- timing.json
        |   |-- grading.json
        |   `-- outputs/
        `-- without_skill/
            |-- transcript.md
            |-- timing.json
            |-- grading.json
            `-- outputs/
```

Initial `evals/evals.json` shape:

```json
{
  "skill_name": "example-skill",
  "evals": [
    {
      "id": 1,
      "prompt": "User's realistic task prompt",
      "expected_output": "Human-readable success condition",
      "files": [],
      "expectations": [
        "Output contains the required fields with values matching the input",
        "Validation command succeeds with exit code 0"
      ]
    }
  ]
}
```

Run metadata shape:

```json
{
  "eval_id": 1,
  "eval_name": "descriptive-name",
  "prompt": "User's realistic task prompt",
  "expectations": []
}
```

Grading shape:

```json
{
  "expectations": [
    {
      "text": "Output contains the required fields with values matching the input",
      "passed": true,
      "evidence": "Compared output.json fields against fixture input.json"
    }
  ],
  "summary": {"passed": 1, "failed": 0, "total": 1, "pass_rate": 1.0},
  "claims": []
}
```

Benchmark run entries should keep configuration names stable, typically `with_skill`, `without_skill`, or `old_skill`, so comparisons are easy to aggregate.

## Grading Standards

Pass an expectation only when evidence proves meaningful completion. Fail when evidence is absent, contradicted, superficial, unverifiable, or only coincidentally true.

During grading, check:

- Transcript: what the agent actually did
- Outputs: produced files and their contents
- Claims: factual, process, and quality claims made by the agent
- User notes: uncertainties, workarounds, missing dependencies
- Eval strength: whether expectations would catch wrong outputs

## Trigger Eval Design

Create about 20 trigger queries after the skill behavior is stable:

- 8-10 should-trigger queries covering varied phrasing, task sizes, and edge cases
- 8-10 should-not-trigger queries using tricky near-misses and adjacent domains
- Each trigger eval item must include a JSON boolean `should_trigger`; unlabeled or string-labeled cases cannot produce a valid optimization score.

Bad negative query: unrelated programming trivia.

Good negative query: shares vocabulary with the skill but needs a different workflow or should be handled by a more specific skill.

Optimize descriptions against held-out queries when automation exists. If automation is unavailable, review the trigger set manually and explain the tradeoffs.

## Packaging Rules

Package only validated skill content. Exclude:

- eval workspaces and generated reports
- caches and compiled artifacts
- local credentials or `.env` files
- temporary downloads
- test-only fixtures unless the skill needs them at runtime

When updating an existing skill, preserve the directory name and frontmatter `name` unless the user explicitly requests a rename.
