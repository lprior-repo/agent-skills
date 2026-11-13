# Native OMP dispatch

Use OMP's `task` tool. Do not start another coding CLI or use direct HTTP. The installation lives only under `~/.agents/skills/local-gpu-coding/`; it does not create custom agent definitions.

Scope inline first. Fill every bracketed field with actual context and exclusive ownership before dispatch. Two independent substantial slices go in one `tasks[]` batch. Omit `agent` for the default worker; never write `agent: "task"`. Set each task's exact approved model selector.

```json
{
  "i": "Implementing independent Rust slices",
  "context": "[Absolute workspace, shared contracts, essential repository rules, integration owner.] Both workers must read skill://holzman-rust and skill://functional-rust before Rust work; use their ~/.agents/skills/<name>/SKILL.md paths if needed and stop if required skills cannot be loaded. [Read plan: concrete paths for applicable skill references, target instructions/manifests and selected evidence, with reasons.] Complete the read plan before editing; return read/unread/failed status. [Exact task-selected specialist skill URIs and compact grounding packets prepared using references/grounding.md.] Read relevant implementation/test examples; do not claim tests passed without execution. Rust-only code/tooling; no Python. No recursive delegation, GPU endpoint queries, other coding harnesses, publication, edits to reference repos, or mid-flight build/test/lint/format commands. Coordinator verifies completed patches and actual read evidence.",
  "tasks": [
    {
      "name": "Rust5090",
      "model": "local-5090-ninfer/qwen3.8-27b-uncensored:high",
      "task": "Preflight: first read skill://local-gpu-coding, then complete the shared mandatory read plan before this task's case-specific steps. If shared context or any required read is missing, report blocked rather than skipping it. [5090 goal, owned files, specialist skill URIs, relevant APIs, implementation/test pair, version constraints, acceptance and non-goals.] Read the actual selected evidence before Rust edits. Return read-plan status for BOTH common and case-specific paths, changed files, decisions, actual loaded skills, source citations, conflicts, blockers, and checks not run.",
      "solutionSpace": "[Describe fixed choices and remaining design decisions.]"
    },
    {
      "name": "Rust3090",
      "model": "local-3090-llama/qwen3.8-27b-uncensored:high",
      "task": "Preflight: first read skill://local-gpu-coding, then complete the shared mandatory read plan before this task's case-specific steps. If shared context or any required read is missing, report blocked rather than skipping it. [3090 goal, owned files, specialist skill URIs, relevant APIs, implementation/test pair, version constraints, acceptance and non-goals.] Read the actual selected evidence before Rust edits. Return read-plan status for BOTH common and case-specific paths, changed files, decisions, actual loaded skills, source citations, conflicts, blockers, and checks not run.",
      "solutionSpace": "[Describe fixed choices and remaining design decisions.]"
    }
  ]
}
```

Each `model` is one exact selector, never a list or role alias. If per-call model selection is disabled, a configured override conflicts, or a model is unavailable, stop and report; do not silently fall back or bypass policy.

OMP children do not automatically inherit `AGENTS.md` or the parent's conversation. Include essential constraints and concrete source locations. Each worker must perform its own successful mandatory/specialist skill reads and read its selected evidence; this packet is not an automatic harness-wide skill hook. Preserve usable context during same-task repairs; rehydrate required sources if context was lost.

The coordinator owns shared interfaces, manifests, integration, and checks. Do not assume task filesystem isolation. Keep one active request per GPU; workers must not delegate back onto their own endpoint. Consume delivered task results without polling. Preserve the first submission; run narrow checks on completed changes, then send all actionable diagnostics together to the same lane for minimal correction. After executable-code repairs, verify affected behavior again. Same-model review and borrowed examples are not independent proof.
