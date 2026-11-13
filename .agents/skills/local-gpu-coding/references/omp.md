# Native OMP dispatch

Use OMP's `task` tool. Do not start another coding CLI or use direct HTTP. The installation lives only under `~/.agents/skills/local-gpu-coding/`; it does not create custom agent definitions.

Scope inline first. Fill every bracketed field with actual context and exclusive ownership before dispatch. Two independent substantial slices go in one `tasks[]` batch. Omit `agent` for the default worker; never write `agent: "task"`. Set each task's exact approved model selector.

```json
{
  "i": "Implementing independent Rust slices",
  "context": "[Absolute workspace, shared contracts, essential repository rules, integration owner.] Both workers must read skill://holzman-rust before any Rust work; if unavailable, read ~/.agents/skills/holzman-rust/SKILL.md. Stop if neither is accessible. Follow its required references. Rust-only code/tooling; no Python. No recursive delegation, GPU endpoint queries, other coding harnesses, publication, or mid-flight build/test/lint/format commands. Coordinator verifies completed patches.",
  "tasks": [
    {
      "name": "Rust5090",
      "model": "local-5090-ninfer/qwen3.8-27b-uncensored:high",
      "task": "Before Rust work, read skill://holzman-rust; use the shared fallback path if needed; stop if neither works. [5090 goal, owned files, relevant inputs, acceptance and non-goals.] Return changed files, decisions, loaded skills, blockers, and checks not run.",
      "solutionSpace": "[Describe fixed choices and remaining design decisions.]"
    },
    {
      "name": "Rust3090",
      "model": "local-3090-llama/qwen3.8-27b-uncensored:high",
      "task": "Before Rust work, read skill://holzman-rust; use the shared fallback path if needed; stop if neither works. [3090 goal, owned files, relevant inputs, acceptance and non-goals.] Return changed files, decisions, loaded skills, blockers, and checks not run.",
      "solutionSpace": "[Describe fixed choices and remaining design decisions.]"
    }
  ]
}
```

Each `model` is one exact selector, never a list or role alias. If per-call model selection is disabled, a configured override conflicts, or a model is unavailable, stop and report; do not silently fall back or bypass policy.

OMP children do not automatically inherit `AGENTS.md` or the parent's conversation. Include essential constraints and known read paths. Each worker must perform its own successful `holzman-rust` read; this packet is not an automatic harness-wide skill hook.

The coordinator owns shared interfaces, manifests, integration, and checks. Do not assume task filesystem isolation. Keep one active request per GPU; workers must not delegate back onto their own endpoint. Consume delivered task results without polling. Run narrow checks on completed changes, then send actual diagnostics to the same lane for correction. Same-model review is not independent proof.
