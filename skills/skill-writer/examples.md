# Skill Design Examples

## Good: Portable Focused Description

- "Create reviewer-ready PR summaries from Git history and issue context. Use when users ask to summarize a branch, explain changes for reviewers, draft release notes from PRs, or turn `git`/GitHub output into a concise review brief."

Why it works: clear behavior, clear trigger conditions, no dependency on a specific agent CLI.

## Bad: Vague Description

- "Helps with GitHub stuff."

Why it fails: weak trigger language, ambiguous behavior.

## Bad: Vendor-Coupled Generic Skill

- "Use this AcmeAgent CLI skill with `/deploy-prod` whenever AcmeAgent needs to run the deployment hook."

Why it fails: it can only work in one host, and the trigger describes the implementation instead of the user intent. If the skill is truly host-specific, say that explicitly in the skill name and compatibility notes.

## Good: Progressive Disclosure

- `SKILL.md`: compact workflow and output contract
- `references/schemas.md`: detailed JSON schemas
- `scripts/validate.py`: deterministic validation
- `evals/evals.json`: realistic test prompts

Why it works: fast load path for common calls, deep docs available when needed.

## Good: Strong Eval Expectation

- "The generated CSV has the columns `account_id`, `invoice_total`, and `currency`; row count matches the input JSON invoice count; and `invoice_total` equals the sum of each invoice's line items."

Why it works: a plausible wrong output cannot pass by merely creating a CSV file.

## Bad: Weak Eval Expectation

- "Creates a CSV file."

Why it fails: an empty or malformed CSV would pass.

## Good: CLI-Agnostic Runner Wording

- "Run the eval with the skill enabled and again against the baseline using the strongest isolated runner available in this environment. Capture transcript, outputs, timing if exposed, and grading evidence."

Why it works: it gives the workflow without assuming a specific command name.

## Good: Host Adapter Boundary

- `SKILL_WRITER_RUNNER='agent-run --skill {active_skill_path} --prompt-file {prompt_file} --output-dir {output_dir}'`

Why it works: the script owns artifact layout while the host-specific command stays outside the portable skill logic.

## Good: Existing Skill Strengthening

- Keep the original skill name and purpose, sharpen the description from "helps with deploys" to "Plan and verify staged application deployments. Use when users ask to prepare a deploy checklist, run preflight checks, compare staging vs production readiness, or diagnose deployment blockers. Do not use for writing application code." Add evals for a normal deploy plan, a missing-environment edge case, and a coding-task near-miss.

Why it works: it preserves the user's existing workflow while improving trigger precision, safety, and testability.

## Bad: Cosmetic Rewrite Masquerading as Hardening

- Replace the whole skill with a new structure, rename it, remove working helper scripts, and add generic best-practice prose without running validation or comparing against the original.

Why it fails: hardening should reduce risk with evidence. A rewrite that changes the contract without proof can regress behavior.

## Good: Hardening Weak Evals

- Change "produces a report" to "the report includes the requested service name, all failing checks from `preflight.json`, an owner for each blocker, and a clear skipped-check section when a dependency is unavailable."

Why it works: the expectation now catches missing substance instead of rewarding any output file.

## Bad: Hidden Host Default

- A script silently runs `AcmeAgent --auto-load-skill` when no runner is configured.

Why it fails: portability is fake if the fallback path depends on one product or host registry.

## Bad: Hallucinated Verification

- "The benchmark passed" when no benchmark command was run and no benchmark file exists.

Why it fails: skill quality depends on evidence. Report skipped checks explicitly.

## Bad: Monolithic SKILL.md

- One file with hundreds of lines of mixed policy, templates, examples, and scripts.

Why it fails: high token cost, low scanability, brittle updates.
