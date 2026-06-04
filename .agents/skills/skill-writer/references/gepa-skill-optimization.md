# GEPA Workflow For Skill Optimization

Use this reference when improving local agent skills with GEPA-style evaluation. It maps the GEPA adapter guide to `SKILL.md` authoring, hardening, and benchmarking.

## When To Use This

Use this workflow when a user asks to improve, optimize, learn, benchmark, or evolve skills, prompts, agents, rubrics, subagent instructions, tool policies, or skill-trigger descriptions using measured outcomes.

Do not use it for a one-off skill edit that has no evaluator, no task set, and no realistic way to compare behavior.

## Core Mapping

| GEPA Concept | Skill Workflow Equivalent |
|---|---|
| `candidate: dict[str, str]` | The text components being evolved: `description`, workflow steps, review rubric, output contract, tool policy, eval expectations, or proposer prompt. |
| `DataInst` | A realistic skill eval task: user prompt, fixture repo/path, expected behavior, allowed tools, non-trigger label, and verification commands. |
| `Trajectory` | Full run trace: selected skills, transcript, tool calls, changed files, command outputs, review findings, timing, token/cost, and final result. |
| `RolloutOutput` | Structured result of one run: pass/fail, produced artifacts, defects, safety events, and residual blockers. |
| `EvaluationBatch` | Batch of skill-task runs with per-task scores, optional trajectories, and objective-level scores. |
| `make_reflective_dataset` | Converts raw run traces into compact examples for the reflection/proposer model. |
| Actionable Side Information | Compiler/test output, reviewer notes, failed expectations, missing evidence, bad trigger selection, latency/cost, and diff statistics. |
| Pareto frontier | Preserve skill variants that excel at different task families instead of averaging them away. |

## Prefer `optimize_anything` First

Use GEPA `optimize_anything` when the artifact can be represented as one string or a small dictionary and the evaluator can run one task at a time.

Good candidates:

- One `SKILL.md` body.
- A trigger `description`.
- A review rubric.
- A test-plan prompt.
- A Rust implementation prompt.
- A gate-order policy.

Use a full `GEPAAdapter` only when you need batched evaluation, custom trace capture, per-component reflective datasets, or adapter state persistence.

## Candidate Shape For Skills

Split a skill into named text components so GEPA can update one component at a time:

```python
seed_candidate = {
    "description": current_description,
    "workflow": ordered_workflow_text,
    "safety_policy": safety_boundaries,
    "output_contract": final_response_shape,
    "evaluation_policy": eval_and_grading_rules,
    "trigger_examples": trigger_and_non_trigger_examples,
}
```

For Rust harness skills, add components such as:

```python
seed_candidate = {
    "contract_prompt": rust_contract_prompt,
    "decomposition_reviewer": dag_split_review_rubric,
    "implementation_prompt": holzman_rust_prompt,
    "test_prompt": test_writer_prompt,
    "proof_lane_policy": proof_lane_rules,
    "gate_policy": command_gate_policy,
    "error_triage_prompt": compiler_failure_triage_prompt,
}
```

## Eval Task Shape

Each task should be realistic and verifiable:

```python
@dataclass
class SkillEvalTask:
    prompt: str
    fixture_path: str | None
    expected_outputs: list[str]
    forbidden_outputs: list[str]
    required_commands: list[str]
    should_trigger: bool | None
    risk_tags: list[str]
```

Use `should_trigger` for trigger optimization. Use `None` when the task evaluates behavior after the skill is already selected.

## Trace Shape

Capture enough detail that another model can diagnose why the candidate failed:

```python
@dataclass
class SkillRunTrace:
    task_id: str
    candidate_id: str
    selected_skills: list[str]
    transcript_path: str
    tool_calls: list[dict]
    files_changed: list[str]
    command_results: list[dict]
    review_findings: list[dict]
    timing_ms: int | None
    token_count: int | None
    cost_usd: float | None
    final_status: str
```

Never invent timing, token, or cost fields. Use `None` or `unavailable` when the runner does not expose them.

## Evaluation Scores

Return a scalar score for acceptance, but also expose objective scores so Pareto search can preserve specialists.

Suggested objectives:

- `task_success`: passed the task-specific verifier.
- `trigger_precision`: selected the skill only when it should trigger.
- `trigger_recall`: selected the skill when it should trigger.
- `evidence_strength`: produced raw command or artifact evidence, not claims.
- `safety`: avoided destructive, credential, privacy, or policy violations.
- `minimality`: avoided unnecessary workflow steps or overbroad rewrites.
- `artifact_quality`: final files are structured, concise, and reusable.
- `latency`: inverted duration score, when measured.
- `cost`: inverted token/cost score, when measured.

For Rust harness optimization, add:

- `rust_gate_pass`: formatting, check, clippy, tests, and policy scans pass.
- `forbidden_constructs`: zero new forbidden Rust constructs.
- `proof_test_source_parity`: proof/test/source mappings close.
- `mutation_resistance`: mutants killed or explicitly justified.
- `performance_evidence`: benchmark/profiler evidence exists when speed is claimed.

## Reflective Dataset Records

Each component gets concise records. Keep the raw evidence pointers and the actionable diagnosis.

```python
record = {
    "Inputs": {
        "user_prompt": task.prompt,
        "fixture": task.fixture_path,
        "component_under_test": component_name,
    },
    "Generated Outputs": {
        "selected_skills": trace.selected_skills,
        "final_status": trace.final_status,
        "files_changed": trace.files_changed,
    },
    "Feedback": "Score: 0.40\nFailed: clippy gate was claimed but not run\nEvidence: transcript.md lines 81-116\nRepair direction: require exact command evidence before final response.",
}
```

Good feedback explains why the run failed and what kind of text change could improve it. Bad feedback only says `failed` or only reports the scalar score.

## Error Handling

For individual task failures, do not raise and abort the whole optimization run. Return a failed output with score `0.0` and a trajectory containing the exception, command error, or missing tool.

Reserve exceptions for systemic failures such as missing runner, invalid candidate schema, or inaccessible fixture repository.

## Data Splits

Keep separate sets:

- `train`: used for reflective updates.
- `val`: used for candidate selection and generalization checks.
- `test`: held out until final reporting.

Do not tune against the final test set. Do not report test performance unless the run was actually executed.

## Baselines

For a new skill, compare against no skill or the host's default behavior.

For an existing skill, snapshot the original `SKILL.md` and compare against that snapshot.

For a Rust coding harness, compare against the current `holzman-rust`, `functional-rust`, `go-skill`, and review pipeline before claiming improvement.

## Local Artifact Layout

Use this layout for skill optimization runs:

```text
skill-name-workspace/
|-- iteration-000-baseline/
|   |-- candidate.json
|   |-- runs/<task-id>/transcript.md
|   |-- runs/<task-id>/grading.json
|   `-- benchmark.json
|-- iteration-001/
|   |-- candidate.json
|   |-- reflective_dataset.json
|   |-- proposer_notes.md
|   `-- benchmark.json
`-- final-report.md
```

Exclude these workspaces from skill packages unless the user explicitly asks for eval artifacts to ship.

## How To Use The Local GEPA Site Mirror

The local mirror is stored at:

```text
skill-writer/references/gepa-site/gepa-ai.github.io/gepa/
```

Use it for local reading of GEPA docs and examples. Prefer compact local reference files for prompt context:

- `references/gepa-adapter-guide.md` for adapter protocol details.
- `references/gepa-skill-optimization.md` for skill workflow mapping.
- `references/gepa-site/README.md` for scrape scope and key pages.

Do not load the entire mirrored HTML site into a model prompt. Read specific pages relevant to the current task.

## Acceptance Checklist

- Candidate components are named and independently updateable.
- Eval tasks are realistic and include pass/fail expectations.
- `evaluate` returns per-example scores and does not mutate candidates in place.
- Traces are captured when `capture_traces=True` and align with outputs and scores.
- Reflective records include inputs, generated outputs, and actionable feedback.
- Failures are returned as scored examples rather than silently swallowed.
- Multi-objective scores cover quality, safety, evidence, trigger behavior, and cost/latency where measured.
- Baseline and candidate runs use the same task set and runner conditions.
- Final claims cite real commands, artifacts, and score deltas.
