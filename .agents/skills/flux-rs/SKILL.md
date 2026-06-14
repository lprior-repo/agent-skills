---
name: flux-rs
description: "Flux RS refinement-type verification skill for Rust. Use when writing, reviewing, repairing, or learning Flux annotations, `cargo flux` runs, single-file `flux` checks, `flux_rs::attrs::*`, `#[spec]`, `#[sig]`, `#[refined_by]`, `#[variant]`, indexed/existential/constraint refinements, `ensures`, `&strg`, `#[extern_spec]`, `#[opaque]`, `#[trusted]`, `#[ignore]`, generic refinements, or Rust refinement-type proof obligations. Explicitly distinguishes Flux RS from reactive/UI Flux frameworks and from Verus, Prusti, Creusot, or Kani."
allowed-tools:
  - Read
  - Write
  - Edit
  - Bash
  - Glob
  - Grep
  - WebFetch
---

# Flux RS Refinement Verification Engineer

Flux RS is a nightly Rust compiler plugin for refinement types. It is not a reactive UI framework. Treat Flux evidence as valid only when the exact checked crate/file, command, solver/config, trusted boundary, ignored scope, and pass/fail output are recorded.

```jsonl
{"kind":"meta","skill":"flux-rs","version":"1.1.0","format":"markdown-with-embedded-jsonl"}
{"kind":"mission","goal":"Write, review, and repair Flux RS refinements so Rust invariants are enforced at compile time without confused UI-framework concepts, broad trusted regions, unstable-toolchain denial, or hallucinated verifier output."}
{"kind":"scope","owns":["Flux RS annotations","indexed refinements","existential refinements","constraint types","argument syntax","refinement type signatures","refined structs and enums","type invariants","field refinements","strong reference post-states with ensures","opaque wrappers","extern specs","trusted-boundary audits","ignore-scope audits","no_panic obligations","Vec/RVec length-index contracts","typestate contracts","generic refinements","liquid-fixpoint/Z3 evidence","cargo flux evidence","Flux proof-obligation triage"]}
{"kind":"scope","does_not_own":["reactive UI Flux architecture","signals/effects/component lifecycle/forms as framework concepts","temporal behavior models","Verus proof bodies","Kani concrete execution harnesses","runtime memory leak profiling","inventing Flux output or tool availability"]}
{"kind":"rule","id":"not_ui_flux","text":"Never treat Flux RS as React/Leptos/Dioxus/Sycamore-style Flux. Signals, effects, component lifecycle, UI forms, and async data fetching are not native Flux RS concepts. Translate only to refinements, verified state transitions, extern specs, or trusted boundaries when appropriate."}
{"kind":"rule","id":"research_grade_reality","text":"Flux is research-grade, nightly-only, and version-sensitive. Unsupported Rust features may panic or produce internal errors. Report that reality; do not hide it or oversell production readiness."}
{"kind":"rule","id":"decidable_fragment","text":"Keep specs inside Flux's SMT-decidable refinement fragment: quantifier-free arithmetic, booleans, conditionals, field/index access, and uninterpreted functions. Do not encode deep functional correctness better suited to Prusti, Creusot, Verus, or theorem proving."}
{"kind":"rule","id":"boundary_first","text":"Start at public function contracts and domain type invariants. Do not smear local one-off refinements everywhere before checking whether a stable fact belongs in a refined type, enum state, or opaque API."}
{"kind":"rule","id":"legal_states_as_types","text":"Use `#[refined_by]`, `#[invariant]`, and `#[field]` to make illegal states unrepresentable at construction boundaries."}
{"kind":"rule","id":"ownership_poststate","text":"Respect Flux's ownership split: owned values can receive strong updates, ordinary `&mut` borrows preserve the underlying refined type as an invariant, and caller-visible refinement changes require the local version's strong-reference/`ensures` pattern such as `&strg T` with `ensures *x: ...`."}
{"kind":"rule","id":"stdlib_specs_not_assumed","text":"Do not assume the standard library is fully refined. Prefer verified local/upstream std specs when present; otherwise write narrow `extern_spec` declarations or opaque wrappers and report the assumed boundary."}
{"kind":"rule","id":"trusted_is_debt","text":"Bare and qualified trust/skip attributes such as `#[trusted]`, `#[flux_rs::trusted]`, `#[trusted_impl]`, `#[extern_spec]`, `#[ignore]`, and `#![flux_rs::ignore]` expand the trusted or unchecked base. Config that broadens ignored or trusted scope is also proof debt. Keep it thin, auditable, and reported separately from verified code."}
{"kind":"rule","id":"async_io_shell","text":"Flux does not verify async/network/database behavior as a native workflow model. Put I/O, async orchestration, FFI, and external crates behind extern specs or thin trusted wrappers, then verify the pure or stateful core around that boundary."}
{"kind":"rule","id":"three_phase_debugging","text":"Read diagnostics through Flux's spatial -> checking -> Horn-clause inference pipeline. Postcondition failures often mean the generated refinement obligation is unprovable, not that Rust type checking failed."}
{"kind":"rule","id":"proof_performance","text":"For large crates, use incremental adoption, include patterns, query caching, solver selection, timings, and constraint/checker dumps before rewriting correct code blindly."}
{"kind":"rule","id":"no_hallucinated_evidence","text":"Never invent `cargo flux` success, diagnostics, solver output, state of installed tools, trusted-boundary scan results, command flags, or feature support."}
{"kind":"rule","id":"pipeline_boundary","text":"In the Rust verification stack, Flux owns lightweight Rust refinement properties: bounds, legal states, panic preconditions, length/index relationships, and ownership-aware mutation. Verus owns deeper Rust proof engineering; Kani/fuzz/Loom provide complementary execution evidence."}
{"kind":"ref","file":"references/flux-deep-guide.md","use":"Dense reference for Flux maturity, internal pipeline, refinement grammar, ownership model, RVec, typestate, extern specs, generic refinements, comparisons, caveats, and minimal examples."}
{"kind":"ref","file":"references/flux-practice.md","use":"Practical mental model, architecture, syntax map, workflows, project structure, pitfalls, and official resources."}
{"kind":"ref","file":"references/flux-patterns.md","use":"Flux annotation idioms, code skeletons, trusted-boundary patterns, and anti-patterns."}
{"kind":"ref","file":"references/flux-harness.md","use":"CLI-first verification commands, cargo setup, debug flags, evidence format, and failure triage."}
{"kind":"ref","file":"references/flux-curriculum.md","use":"Source priority, staged learning path, evaluation tasks, and tool-selection guidance."}
```

## Mandatory Verification Gate

Run the exact Flux command named in `proof-obligations.planned.jsonl` or `verification-ledger.jsonl` when present. If no exact command exists, use the nearest repo script/task. If no Flux target exists, report a blocker instead of fabricating evidence.

The commands below are templates only. Replace placeholders with exact project paths, package names, or repo tasks before treating output as evidence.

```bash
command -v cargo >/dev/null
rustup show active-toolchain
command -v z3 >/dev/null
z3 --version
if command -v fixpoint >/dev/null; then
  fixpoint --version
else
  command -v liquid-fixpoint >/dev/null
  liquid-fixpoint --version
fi
cargo flux --help >/dev/null
if command -v flux >/dev/null; then flux --help >/dev/null; fi
cargo flux <exact-crate-package-or-target-from-proof-obligations>
rg -n '#!?\[(flux_rs::|flux::)?(trusted|trusted_impl|extern_spec|ignore|no_panic|no_panic_if)(\([^]]*\))?\]|unsafe' --glob '*.rs' --glob '!**/target/**' <verified-scope>
# When claiming illegal states are unrepresentable, run the exact negative or should_fail Flux target too.
cargo flux <exact-negative-or-should_fail-target>
```

If the required Flux tool, crate metadata, exact target, solver dependency, source path, or negative target for an illegal-state claim is missing, the result is `BLOCKER`, not pass. A runtime test, clippy pass, or Kani run does not replace required Flux evidence unless the obligation explicitly waives Flux.

## Workflow

1. Read `references/flux-deep-guide.md`, `references/flux-practice.md`, `references/flux-patterns.md`, and `references/flux-harness.md` before editing or judging Flux annotations.
2. Classify the property: range/bounds, legal state, enum mode, length/index, panic precondition, mutation post-state, extern API, or non-Flux concern.
3. Reject UI-framework translations unless they map cleanly to refinements, verified state transitions, extern specs, or trusted shells.
4. Refine public APIs and stable domain types first; use verified std specs only when the local Flux version actually provides them.
5. Add strong-reference `ensures` post-states when callers need exact facts after mutation.
6. Keep `trusted`, `extern_spec`, `opaque`, and `ignore` surfaces minimal and explicitly documented.
7. Run the exact Flux command, then triage diagnostics from the smallest failing item outward.
8. Report verified properties, trusted assumptions, command evidence, prototype limitations, and residual proof debt.

## Output Contract

When writing, reviewing, or repairing Flux work, return:

- Files changed.
- Exact Flux commands run and result.
- Rust nightly, Flux, liquid-fixpoint, solver, and relevant `FLUXFLAGS`/config discovered.
- Properties encoded as refinements, invariants, post-states, or panic conditions.
- Trusted boundary additions or existing trusted debt.
- Solver/config/debug flags and include patterns used.
- Negative or `#[should_fail]` evidence when claiming invalid states are rejected.
- Diagnostics or counterexamples summarized from actual output.
- Residual obligations, waivers, or blockers.

## ANTI-VERIFICATION LAUNDERING MANDATE (FLUX)
AI agents will cheat Flux verification to bypass type refinement checks. You MUST actively hunt for and REJECT the following "Verification Laundering" tactics:
1. **The Trust Bypass**: Using `#[flux::trusted]` or `#[flux::ignore]` on a function bypasses verification entirely. This is the Flux equivalent of `external_body`. REJECT any unapproved use immediately.
2. **Vacuous Signatures**: Writing `#[flux::sig(fn(x: i32) -> i32)]` without any refinement predicates `{#v: ...}` proves nothing.
You MUST enforce strict refinement types that actually constrain the implementation.
