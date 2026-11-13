---
name: local-gpu-coding
description: "OMP-only Rust coding on the RTX 5090 and RTX 3090 using repository code and tests. Use exact native task model selectors, mandatory holzman-rust loading and task-specific specialist skills. Installed only under ~/.agents. No Python or other coding harnesses."
---

# Local GPU Coding — OMP Only

Use OMP's native `task` tool to give substantive Rust implementation work to both local GPUs. The harness is OMP; this skill's only installation is `~/.agents/skills/local-gpu-coding/`. Do not create integrations or links for other coding harnesses, install agent definitions elsewhere, or replace native tasks with direct HTTP generation.

## Local lanes

| GPU | Exact per-task model selector |
| --- | --- |
| RTX 5090 | `local-5090-ninfer/qwen3.8-27b-uncensored:high` |
| RTX 3090 | `local-3090-llama/qwen3.8-27b-uncensored:high` |

Set `model` explicitly on each task item. Do not use role aliases such as `tiny`, `smol`, or `commit` to select a GPU. OMP's exact per-call model guard prevents silent fallback and model promotion. If host policy rejects or conflicts with a selector, report it rather than changing policy or using HTTP.

## Mandatory Holzman Rust loading

Every 5090/3090 worker must load **`holzman-rust` before writing, fixing, refactoring, or reviewing Rust**. The parent's skill read is not a substitute. Copy this startup requirement into every Rust worker task:

```text
Before any Rust work, read skill://holzman-rust in this worker and follow it.
If that URI cannot resolve, read ~/.agents/skills/holzman-rust/SKILL.md.
If neither can be read, stop and report the blocker; do not code without it.
Follow the skill's required references and applicable repository rules.
Also read skill://functional-rust and the task-selected specialist skills.
Read the target repository instructions and relevant implementation/tests.
Complete the packet's explicit read plan before editing; report unread/failed paths.
Use Rust for code and reusable tooling; no Python scripts or validators.
Do not delegate, query GPU endpoints, start another coding harness, publish,
or run builds/tests/linters/formatters mid-flight. The coordinator verifies.
```

This is an explicit worker startup requirement, not an OMP autoload hook. OMP's `autoloadSkills` mechanism requires agent definitions outside this installation; none are installed. Do not claim universal enforcement for tasks that bypass this skill. A worker's assertion that it loaded a skill is not evidence; the worker transcript's successful skill read is.

## Ground before dispatch

Read [references/grounding.md](references/grounding.md) to select skills and a relevant implementation/test pair. Prefer the target crate; use [references/local-rust-map.md](references/local-rust-map.md) only for a specific gap. These are discovery sources, not approved templates.

Pass a compact evidence packet with exact skill URIs, applicable references and target instructions, code/test paths, version constraints and unresolved conflicts. Each worker reads its assigned sources itself. The coordinator checks successful tool reads before accepting a packet or patch; a claimed skill list is insufficient. Keep context focused on the assigned behavior and relevant source.

## Dispatch discipline

1. Scope inline first and follow OMP's delegation rules. Assign one complete behavior and exclusive file ownership to each worker. Use both GPUs for two genuinely independent substantial slices; do not manufacture busywork.
2. Both GPUs implement. Do not reserve the 3090 for commits or administration. Launch at most one outstanding request per GPU from this coordinator; other sessions may still occupy the servers.
3. Workers must not recursively delegate or start another coding CLI. Keep shared interfaces, manifests, integration, and verification with one owner. Do not assume task workers have isolated filesystems.
4. Give workers relevant types, source paths, and essential repository constraints. OMP children do not automatically inherit `AGENTS.md`. Do not send the whole conversation or unrelated skill library.
5. Preserve explicit GPU reservations for gameplay/performance testing. Do not stop services, modify inference settings, or steal desktop focus.

Read [references/omp.md](references/omp.md) for the native task shape. Include the mandatory startup requirement above plus:

```text
Goal: one consumer-visible behavior.
Workspace and ownership: absolute directory and exact allowed files.
Context: relevant APIs, existing patterns, dependencies, repository rules.
Grounding: required skill URIs, repository instructions,
one relevant implementation/test pair, version checks and evidence limitations.
Acceptance: concrete behavior, edge cases, narrow verification commands.
Non-goals: unrelated changes and unrequested publication.
Return: changed files, decisions, actual loaded skills, source citations,
blockers, checks run by the coordinator or explicitly not run.
```

## Rust-only tooling and verification

Implementation code, reusable helpers, orchestration, and test harnesses must be Rust. Do not introduce or invoke Python scripts, adapters, dependencies, or validators, including pre-existing Python validators. Verify with the repository's Rust-native commands or `cargo`/`rustc`; do not rewrite unrelated tooling.

Workers skip builds, tests, linters, and formatters mid-flight. The integration owner preserves the first submission and runs narrow checks after completed patches, including the real CLI/UI surface where appropriate. Once code compiles, collect both behavior failures and applicable lint diagnostics in one repair packet. Ask for a minimal correction, not a rewrite of working code; keep acceptance tests coordinator-owned and rerun affected behavior after executable-code repairs. Let Rust formatting tools handle mechanical formatting. Return actual diagnostics and the violated contract to the same local lane; do not take over implementation merely because it needs feedback.

Task results arrive automatically: do not poll, stack duplicate requests, or treat a timeout as proof that a GPU is idle. On unavailable models, missing skills, policy rejection, or inference failure, report the exact lane and observed error. Do not silently substitute models/providers, bypass policy, or start/reconfigure services.

Report the actual worker/model, changes, observed checks, and unresolved blockers. A worker's prose is not verification, and a smoke test does not establish general parity with a stronger model.

## Discovery

Canonical file: `~/.agents/skills/local-gpu-coding/SKILL.md`. OMP discovers this shared skill root; no other harness is configured by this skill. Read `skill://local-gpu-coding` in OMP. If an existing session has a stale skill catalog, start a fresh OMP session. Keep future changes under this directory; do not recreate a managed-skills copy or foreign-harness symlinks.
