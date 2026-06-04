---
name: verus
description: "Verus proof-engineering skill for writing, reviewing, and repairing Verus specs, proofs, loop invariants, triggers, ghost/exec separation, trusted boundaries, and verifier-in-the-loop Rust proof obligations. Use whenever working with Verus, verusfmt, proof fn, spec fn, requires/ensures, recommends, tracked state, or Verus verifier failures."
allowed-tools:
  - Read
  - Write
  - Edit
  - Bash
  - Glob
  - Grep
---

# Verus Proof Engineer

Verus is not Rust plus annotations. It is a verifier-backed three-mode language: `spec`, `proof`, and `exec`. Optimize for verifier-accepted proof structure, not plausible syntax.

```jsonl
{"kind":"meta","skill":"verus","version":"1.0.0","format":"markdown-with-embedded-jsonl"}
{"kind":"mission","goal":"Write, review, and repair Verus code so it verifies without hidden trust expansion, brittle solver context, or hallucinated evidence."}
{"kind":"scope","owns":["Verus specs","proof functions","exec contracts","loop invariants","quantifier triggers","ghost/exec separation","trusted-boundary audits","verifier diagnostics repair","Verus proof-obligation execution evidence"]}
{"kind":"scope","does_not_own":["temporal models","production Rust implementation unrelated to proof repair","test helper/loop/table-style judgments","inventing external proofs or tool output"]}
{"kind":"rule","id":"mode_first","text":"Choose `spec`, `proof`, or `exec` first. Spec defines mathematics; proof establishes facts; exec runs. Do not mix ghost-only values into executable expressions."}
{"kind":"rule","id":"contract_shape","text":"Use `requires`/`ensures` on exec and proof functions. Do not put `requires`/`ensures` on `spec fn`; use `recommends` or redesign."}
{"kind":"rule","id":"math_layer","text":"State specs over `int`/`nat` by default. Keep executable locals and returns in compilable Rust integer types and prove conversions/bounds explicitly."}
{"kind":"rule","id":"loops_are_isolated","text":"Loop invariants must restate ambient facts needed inside the loop: bounds, preserved preconditions, functional relation, and decreases metric."}
{"kind":"rule","id":"trigger_discipline","text":"Quantifier triggers must be program terms that will appear at use sites, such as `s[i]`, not arithmetic side conditions. Bad triggers are proof bugs."}
{"kind":"rule","id":"abstraction_boundary","text":"Use closed spec functions plus proof lemmas for library abstractions. Do not make specs open just to get a call site to verify."}
{"kind":"rule","id":"unfolding_control","text":"Use `reveal`, `hide`, `reveal_with_fuel`, `by(compute)`, or `by(compute_only)` locally. Do not globally over-unfold recursive specs."}
{"kind":"rule","id":"solver_escalation_local","text":"Use `nonlinear_arith`, `integer_ring`, or extra fuel only around the exact assertion or lemma that needs it."}
{"kind":"rule","id":"extensional_equality","text":"For `Seq`, `Set`, and `Map`, prefer extensional equality (`=~=`) when elementwise equality is the proof obligation."}
{"kind":"rule","id":"type_invariants","text":"Type invariants are not magically in proof context. Invoke `use_type_invariant` or the project-approved invariant lemma at the point of use."}
{"kind":"rule","id":"trust_boundary_is_evidence","text":"`assume`, `#[verifier::external_body]`, `#[verifier::external]`, and axiomatic specs are trusted-base expansion. Report them separately from fully proved verification."}
{"kind":"rule","id":"no_hallucinated_evidence","text":"Never invent verifier output, proof names, command success, warning status, file paths, or tool availability."}
{"kind":"rule","id":"pipeline_boundary","text":"In the broader Rust proof stack, Verus is default for Rust-local pure/core logic. Lean/Aeneas/Hax own tiny theorem kernels beyond Verus; Kani/fuzz/Loom/etc. are risk-selected companions."}
{"kind":"ref","file":"references/verus-patterns.md","use":"Mode rules, proof idioms, anti-patterns, and failure signatures."}
{"kind":"ref","file":"references/verus-harness.md","use":"Verifier-in-the-loop commands, diagnostics, trust audit, and acceptance gates."}
{"kind":"ref","file":"references/verus-curriculum.md","use":"Training/evaluation curriculum and source priority map."}
```

## Mandatory Verification Gate

Run the repository's exact Verus command when one is named in `proof-obligations.planned.jsonl` or `verification-ledger.jsonl`. If no exact command exists, use the nearest repo script/task. If none exists, report a blocker instead of fabricating proof evidence.

```bash
command -v verus >/dev/null
verus --version
if command -v verusfmt >/dev/null; then verusfmt --check <verus-files-or-repo-path>; fi
verus <exact-verus-target-or-command-from-proof-obligations>
rg -n 'assume\(|#\[verifier::external_body\]|#\[verifier::external\]|axiom' --glob '*.rs' --glob '!**/target/**'
```

Accepted proof evidence must include command, exit status, verifier summary, warnings/notes if any, and trusted-boundary scan result. Missing `verus` or unknown exact target is `BLOCKER`, not pass.

## Workflow

1. Read `references/verus-patterns.md` and the relevant project Verus files before editing.
2. Classify target as `spec`, `proof`, or `exec` and write/repair the contract shape first.
3. Use smallest proof idiom: local lemma, `assert(goal) by { lemma_call(); }`, loop invariant, trigger, reveal/fuel, compute proof, or solver escalation.
4. Run the exact verifier command and repair from diagnostics.
5. Audit trust boundary and label `assume`/external bodies separately.
6. Report what was proved, what is trusted, command evidence, and any remaining proof debt.

## Output Contract

When writing or repairing Verus, return:

- Files changed.
- Exact Verus commands run and result.
- Proof idioms used.
- Trusted-base additions, if any.
- Residual obligations or blockers.
