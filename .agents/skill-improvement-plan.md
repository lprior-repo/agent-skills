# Skill Improvement Plan

Date: 2026-04-21

## Scope

- Audited 72 materialized `SKILL.md` files across 38 unique skill names.
- Skill roots inspected: `/home/lewis/.agents/skills`, `/home/lewis/.claude/skills`, `/home/lewis/.opencode/skill`.
- Reference standard: `skill-writer` guidance in `/home/lewis/.agents/skills/skill-writer/`.

## Executive Summary

Your skill portfolio has two clearly different generations.

- Newer skills like `opencode-scheduler` and `hands-on-qa` already look like a real contract: they have input handling, explicit workflow, output shape, verification gates, and anti-hallucination rules.
- Most of the rest of the catalog is still older monolithic prompt text: huge `SKILL.md` files, weak frontmatter, missing least-privilege tool declarations, and inconsistent verification rules.

The main job is not inventing more skill ideas. The main job is normalizing the portfolio so every skill is predictable, testable, and cheap to maintain.

## Measured Findings

Across the 72 skill files:

- Only 10 include `allowed-tools`.
- Only 9 include `argument-hint`.
- Only 17 mention `Mandatory Verification Gate`.
- Only 8 include explicit anti-hallucination language.
- 34 files are longer than 150 lines.
- 24 files are longer than 250 lines.

Largest current `SKILL.md` files:

- `landing-skill`: 1675 lines
- `opencode`: 812 lines
- `moon-v2`: 806 lines
- `red-queen`: 604 lines
- `test-writer`: 450 lines
- `planner`: 416 lines

Mirror drift:

- 34 skill names exist in both `.agents` and `.claude`.
- 33 mirrored pairs are byte-identical.
- `go-skill` is already drifted between roots.

## Concrete Problems Found

### 1. No canonical source of truth

The mirror setup is mostly synchronized, but not guaranteed. `go-skill` already differs between:

- `/home/lewis/.agents/skills/go-skill/SKILL.md`
- `/home/lewis/.claude/skills/go-skill/SKILL.md`

That means future edits will silently fork behavior across agents.

### 2. Monolithic skills are too large to stay reliable

The biggest files are acting like manuals, runbooks, examples, and policy docs all at once. That breaks progressive disclosure and makes the actual skill contract harder to see.

### 3. Most skills do not declare tool contracts

If `allowed-tools` is missing, the skill contract is underspecified. That makes it harder to reason about least privilege and expected execution behavior.

### 4. Most skills do not define input shape

If `argument-hint` is missing, invocation becomes vague and routing quality drops.

### 5. Verification discipline is inconsistent

Some skills are strict about real execution. Many older skills still rely on narrative instructions without a standard verification gate.

### 6. Some skills still contain environment-coupled or stale execution guidance

Examples:

- `planner` hardcodes `~/.claude/skills/planner/planner.nu`.
- `go-skill` tells the agent to prefix commands with `cd ../<bead-id> &&`, which is brittle when the runtime already supports `workdir`.

### 7. Some machine-readable sections need validation

`qa-enforcer` contains malformed JSONL in its gate block, which means the "structured" section is not fully trustworthy until linted.

## What Good Looks Like

`opencode-scheduler` and `hands-on-qa` are the best current templates for the portfolio because they have:

- explicit mission
- explicit input handling
- least-surprise workflow
- tool selection rules
- output contract
- verification gate
- anti-hallucination rules
- support docs for progressive disclosure

That should become the default shape for every skill.

## Portfolio Standard I Will Apply

Every skill should converge on this structure:

1. YAML frontmatter
- `name`
- `description`
- `argument-hint` when parameters matter
- `allowed-tools` with least privilege
- `disable-model-invocation` or `user-invocable` when risk requires it

2. Compact JSONL contract block
- mission
- input contract
- core rules
- workflow
- output format
- references
- hard gates

3. Mandatory Verification Gate
- exact commands or tool checks required before sign-off

4. Anti-Hallucination Shield
- explicit ban on fabricated output, fake file claims, and fake state claims

5. Progressive disclosure support docs
- `reference.md`
- `examples.md`
- `checklist.md`
- optional domain-specific docs

## Upgrade Strategy

### Wave 0: Foundation

Apply once, then use everywhere.

- Pick `/home/lewis/.agents/skills` as the canonical source.
- Mirror into `/home/lewis/.claude/skills` from automation, not by hand.
- Add a portfolio audit script that checks:
  - frontmatter validity
  - presence of `allowed-tools`
  - presence of `argument-hint` when needed
  - presence of verification gate
  - presence of anti-hallucination section
  - `SKILL.md` size budget
  - JSONL parse validity
- Add a reusable skill skeleton based on `opencode-scheduler`.

### Wave 1: Highest-Risk Skills

These should be upgraded first because they are central or drift-prone.

- `go-skill`
- `master`
- `planner`
- `landing-skill`
- `qa-enforcer`
- `red-queen`

Goals for this wave:

- remove hardcoded path assumptions
- split manuals into support docs
- normalize state-machine language
- add strict output contracts
- add verification gates that match actual tool behavior
- run `truth-serum` against each revised skill

### Wave 2: Tooling and Environment Skills

- `opencode`
- `moon-v2`
- `gastown`
- `jj`
- `dolt`
- `rtk`
- `omarchy`
- `dioxus`
- `dioxus-qa`

Goals for this wave:

- move giant command catalogs into `reference.md`
- keep `SKILL.md` focused on routing, rules, and gates
- add environment prerequisite sections
- reduce absolute path and machine-specific assumptions

### Wave 3: Rust Delivery Stack

- `holzman-rust`
- `rust-contract`
- `test-planner`
- `test-reviewer`
- `test-writer`
- `async-rust-reviewer`
- `scott-ddd-refactor`
- `truth-serum`

Goals for this wave:

- unify artifact path conventions
- normalize verification commands
- align severity language and output formats
- reduce overlap between reviewer skills

### Wave 4: Planning and Spec Skills

- `arch-design-qa`
- `architectural-drift`
- `arch-spec-to-beads`
- `doc-to-beads`
- `decomposer`
- `plan-shredder`
- `beads`
- `bdd-enforcer`
- `velocity`
- `femdation`
- `refinery`

Goals for this wave:

- make handoff artifacts explicit
- define exact start conditions and finish conditions
- separate interactive questioning skills from non-interactive pipeline skills
- standardize what file each skill writes and how the next skill consumes it

## How I Would Improve Each Skill Family

### Orchestrators

Skills:

- `go-skill`
- `master`
- `landing-skill`
- `planner`
- `arch-spec-to-beads`
- `doc-to-beads`
- `femdation`
- `refinery`

Improvements:

- Replace giant inline procedures with short state summaries plus linked runbooks.
- Standardize artifact names, state transitions, and retry budgets.
- Require one output table per state: input artifact, action, verification, output artifact.
- Remove shell patterns that fight the runtime, especially hardcoded `cd ... &&` habits.

### Reviewers and QA

Skills:

- `qa-enforcer`
- `truth-serum`
- `hands-on-qa`
- `black-hat-reviewer`
- `test-reviewer`
- `red-queen`
- `dioxus-qa`

Improvements:

- Standardize severity levels and report sections.
- Make evidence requirements machine-checkable.
- Separate "review only" skills from "review and auto-fix" skills.
- Add a uniform anti-fabrication block to every reviewer.

### Rust Implementers and Spec Authors

Skills:

- `holzman-rust`
- `rust-contract`
- `test-planner`
- `test-writer`
- `async-rust-reviewer`
- `scott-ddd-refactor`
- `bdd-enforcer`

Improvements:

- Reduce duplication around contracts, tests, and invariants.
- Define exactly which artifact each skill reads and writes.
- Keep doctrine in support docs and leave only enforceable constraints in `SKILL.md`.

### Tooling and Platform Guides

Skills:

- `gastown`
- `opencode`
- `opencode-scheduler`
- `jj`
- `moon-v2`
- `dolt`
- `rtk`
- `omarchy`
- `dioxus`

Improvements:

- Move command encyclopedias into `reference.md`.
- Keep top-level `SKILL.md` under a small routing budget.
- Add "when not to use this skill" sections where routing confusion is likely.
- Pin commands to actual current tool behavior and supported flags.

## Acceptance Criteria For The Portfolio

I would consider the portfolio upgraded when all of this is true:

- every skill has valid YAML frontmatter
- every side-effecting skill declares least-privilege tools
- every parameterized skill has `argument-hint`
- every skill has an explicit verification gate or explicitly says it is advisory only
- every skill has an anti-hallucination section
- every `SKILL.md` is compact, with long references moved out
- every mirrored skill is generated from one canonical source
- every updated skill passes a `truth-serum` audit
- every JSONL block parses cleanly

## Recommended First 10 Upgrades

If we do this incrementally, this is the best order:

1. `go-skill`
2. `master`
3. `planner`
4. `landing-skill`
5. `qa-enforcer`
6. `red-queen`
7. `opencode`
8. `moon-v2`
9. `gastown`
10. `jj`

## Short Version

The portfolio does not need more personality. It needs stronger contracts.

The fastest win is:

- one canonical skill source
- one reusable contract template
- one audit script
- one refactor pass over the core orchestrator and QA skills

After that, improving the rest of the catalog becomes repetitive, safe, and fast instead of bespoke.
