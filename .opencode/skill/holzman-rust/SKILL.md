---
name: holzman-rust
description: "OpenCode skill bridge for NASA/JPL Power-of-Ten Rust implementation, repair, review, and performance work. Uses the canonical Holzman Rust doctrine mirrored in ~/.agents and ~/.claude."
argument-hint: "[target path, crate, diff, bead id, or optimization goal]"
agent: holzman-rust
allowed-tools:
  - read
  - grep
  - bash
---

# Holzman Rust OpenCode Skill

This is the OpenCode-side skill bridge for the canonical Holzman Rust doctrine. It routes Rust implementation, repair, review, and performance work to the `holzman-rust` OpenCode agent.

Canonical sources:
- `/home/lewis/.agents/skills/holzman-rust/SKILL.md`
- `/home/lewis/.claude/skills/holzman-rust/SKILL.md`
- `/home/lewis/.opencode/agent/holzman-rust.md`

Before any Rust implementation, repair, review, async hot-path change, low-level systems change, or performance claim, read the applicable canonical source plus these reference files and list the exact files used:
- `/home/lewis/.agents/skills/holzman-rust/references/nasa-jpl-standards.md`
- `/home/lewis/.agents/skills/holzman-rust/references/latency-throughput-playbook.md`
- `/home/lewis/.agents/skills/holzman-rust/references/runtime-performance-architecture.md`
- `/home/lewis/.agents/skills/holzman-rust/references/zero-cost-abstractions.md`
- `/home/lewis/.agents/skills/holzman-rust/references/simd-patterns.md`
- `/home/lewis/.agents/skills/holzman-rust/references/mechanical-empathy-toolchain.md`

Rust implementation, repair, review, async hot-path work, low-level systems work, and performance optimization MUST route to the `holzman-rust` OpenCode agent. No alternate Rust implementation or performance agent may be used unless the user explicitly grants a per-task override.

Non-negotiables:
- No production `unsafe`, `unwrap`, `expect`, `panic`, `todo`, `unimplemented`, `unreachable!`, production `assert!` macros, unchecked indexing, unchecked arithmetic, lossy `as` conversions, or ignored fallible results.
- Use typed errors, proof-carrying newtypes, checked access, checked arithmetic, bounded resources, and explicit failure modes.
- Keep CPU work out of async workers; use Tokio for I/O and Rayon or bounded CPU pools for CPU work.
- Require benchmark or profiler evidence for performance claims.
- Treat performance as a contract layer: name workload, hot path, target hardware, baseline command, baseline number, threshold, and variance before optimizing.
- Require second-ring evidence for zero-cost, vectorization, bounds-check removal, public API compatibility, or release-provenance claims when those claims are made.
- Never invent command output, benchmark numbers, profiler evidence, or file paths.
- Bead delivery is strict on touched production Rust, new regressions, required proof obligations, and global-readiness gates. Already-present repo-wide failures are `BLOCK_GLOBAL` prerequisite repair with proof before advancement.
- Compile tests/examples/benches with `cargo check --workspace --all-targets --all-features`. Strict source lint never includes test targets as an implementation style gate.

## Mandatory Verification Gate

Run the repository's canonical gate first. If no stronger gate exists, run the Holzman fallback gate or report exact blockers:

```bash
cargo fmt --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --lib --bins --examples --all-features -- -D warnings -D unsafe_code -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::panic_in_result_fn -D clippy::todo -D clippy::unimplemented -D clippy::dbg_macro -D clippy::indexing_slicing -D clippy::string_slice -D clippy::get_unwrap -D clippy::arithmetic_side_effects -D clippy::as_conversions -D clippy::let_underscore_must_use -D clippy::await_holding_lock
cargo test --workspace --all-features --no-run
cargo test --workspace --all-features
if rg -n '(^|[^A-Za-z0-9_])(assert!|assert_eq!|assert_ne!|unreachable!)' --glob '*.rs' --glob '!**/tests/**' --glob '!**/benches/**' --glob '!**/examples/**' --glob '!build.rs'; then exit 1; else true; fi
```

When running inside a bead workflow, classify failures against `delivery-scope.jsonl`, `baseline-report.md`, and `global-readiness-report.md`. `BLOCK_LOCAL`, `BLOCK_REGRESSION`, `BLOCK_GLOBAL`, and required-obligation failure stop delivery until repaired; do not substitute bookkeeping for repair.

For performance claims, replace template commands with actual repo benchmark names or report `no benchmark exists` as a blocker. For second-ring claims, replace assembly/API/SBOM templates with actual symbols, packages, baseline revisions, and artifact paths.
