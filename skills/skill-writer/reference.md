# Skill Writer Reference

## Frontmatter Defaults

- `name`: lowercase, hyphenated, command-safe
- `description`: include both what the skill does and when to invoke it
- `argument-hint`: add when arguments are expected
- `allowed-tools`: minimum required tools only
- `model`: set when the skill has clear performance requirements

## Invocation Controls

- Default: both user and model can invoke
- `disable-model-invocation: true`: manual-only for side effects
- `user-invocable: false`: model-only for background knowledge

## Context Isolation

Use `context: fork` only when the skill is an explicit executable task and should run without chat-history coupling.

Recommended pairings:
- Research/exploration: `agent: Explore`
- Planning/decomposition: `agent: Plan`
- Mixed implementation tasks: omit `agent` or use `general-purpose`

## Token Efficiency Pattern

Preferred structure:
1. Valid YAML frontmatter
2. Compact JSONL behavior block
3. Links to supporting docs

Avoid large narrative bodies in `SKILL.md` when data can be represented as JSONL records and offloaded to support files.
