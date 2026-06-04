# Skill Shipping Checklist

- YAML frontmatter parses cleanly and has matching `---` markers
- `name` is stable kebab-case with lowercase letters, numbers, and hyphens
- `description` states what the skill does, when to use it, and important near-misses
- No generic skill hard-codes a specific assistant product, CLI, slash command, or host-only metadata
- Host-specific requirements are isolated in `compatibility`, support docs, or a host adapter section
- Runner and improver commands are explicit adapters; no script defaults to a fixed vendor CLI
- `SKILL.md` is concise enough to read in one pass; bulky material is in linked resources
- Every referenced file, script, asset, and eval path exists
- `scripts/quick_validate.py <skill_path>` passes when Python is available
- Output format requirements are explicit when consistency matters
- Side effects, destructive actions, credentials, and external calls have clear safety boundaries
- Scripts are deterministic, scoped to the skill, and runnable in the stated environment
- Evals exist for objectively checkable workflows, or the reason for qualitative-only review is stated
- Baseline comparison is defined: no skill for new skills, original snapshot for skill updates
- Grading expectations check real substance and include evidence requirements
- Verification results are real; skipped checks are reported as skipped, not implied as passing
- Packages exclude eval workspaces, generated reports, caches, secrets, and local-only artifacts

## Existing Skill Strengthening Checks

- Original skill contract is identified before edits: purpose, triggers, outputs, side effects, and host assumptions
- Existing `name`, directory path, and user-facing purpose are preserved unless the user explicitly asks for a rename or redesign
- Hardening changes target observed weaknesses instead of rewriting working sections for style alone
- Trigger description includes specific intent phrases and near-miss boundaries without broad keyword grabs
- Workflow steps include prerequisites, decision points, failure handling, and final response requirements where relevant
- Safety boundaries cover destructive, external, credential, billing, privacy, and irreversible actions
- Host-specific commands, registry paths, and metadata are isolated as adapters or compatibility notes
- Evals compare against the original skill snapshot when behavior changed
- New or revised expectations would fail for at least one plausible bad output
- Trigger optimization evals use boolean `should_trigger` labels; unlabeled cases are rejected instead of scored as success
- Final report states what was strengthened, what was verified, and what hardening remains

## GEPA-Style Optimization Checks

- Text components to optimize are named explicitly, such as description, workflow, safety policy, output contract, or review rubric
- Eval tasks define inputs, expected outputs, forbidden outputs, required evidence, and trigger labels where relevant
- Baseline comparison is fixed before optimization starts and the final test set is held out
- Trajectories capture actionable side information from real transcripts, command output, changed files, reviews, timing, tokens, or cost when available
- `evaluate` returns per-example scores and records individual task failures instead of aborting the whole run silently
- Reflective dataset records include `Inputs`, `Generated Outputs`, and actionable `Feedback` for each component being updated
- Multi-objective scoring preserves specialists across task success, trigger behavior, evidence strength, safety, minimality, latency, and cost
- Local GEPA docs are used as targeted references; the full mirrored HTML site is not dumped into model context
