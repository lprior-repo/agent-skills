---
name: verus
description: "Verus proof-engineering skill for writing, repairing, and diagnosing Verus specs, proofs, loop invariants, triggers, ghost/exec separation, trusted boundaries, verifier diagnostics, cargo-verus evidence, vstd lemma lookup, and Rust proof obligations. Use for Verus implementation or verifier repair. Review-only proof/evidence audits belong to verus-reviewer."
allowed-tools:
  - Read
  - Write
  - Edit
  - Bash
  - Glob
  - Grep
---

# Verus Proof Engineer

Verus is not Rust plus annotations. It is a verifier-backed three-mode language: `spec`, `proof`, and `exec`. Optimize for verifier-accepted proof structure tied to the real Rust code, not plausible syntax or toy models.

```jsonl
{"kind":"meta","skill":"verus","version":"2.1.0","format":"markdown-with-embedded-jsonl"}
{"kind":"mission","goal":"Write and repair Verus proof material against actual production Rust/Verus targets without exec/spec/TCB drift, hidden trust expansion, brittle solver context, or hallucinated evidence."}
{"kind":"scope","owns":["Verus proof functions","proof blocks","lemmas","assertions","loop invariants","quantifier triggers","ghost/exec/tracked separation","state-machine proof obligations","trusted-boundary audits","verifier diagnostics repair","cargo-verus evidence"]}
{"kind":"scope","does_not_own":["temporal models outside Verus","production Rust behavior changes unrelated to proof repair","approving proof reviews","inventing external proofs or tool output"]}
{"kind":"rule","id":"mode_first","text":"Choose spec, proof, or exec first. Spec defines mathematics; proof establishes facts; exec runs. Do not mix ghost-only values into executable expressions."}
{"kind":"rule","id":"contract_shape","text":"Use requires/ensures on exec and proof functions. Do not put requires/ensures on spec fn; use recommends or redesign."}
{"kind":"rule","id":"code_parity","text":"Every behavior-affecting proof claim must map to real production symbols or accepted extracted helpers. Copied models and local harness-only functions are not implementation proof."}
{"kind":"rule","id":"math_layer","text":"State specs over int/nat by default. Keep executable locals and returns in compilable Rust types and prove conversions, overflow bounds, and indexing bounds explicitly."}
{"kind":"rule","id":"loops_are_isolated","text":"Loop invariants must restate ambient facts needed inside the loop: bounds, preserved preconditions, functional relation, and decreases metric."}
{"kind":"rule","id":"trigger_discipline","text":"Quantifier triggers must be terms that appear at use sites, such as s[i]. Triggers on artificial/arithmetic side conditions are proof bugs or performance bugs."}
{"kind":"rule","id":"abstraction_boundary","text":"Use closed/opaque specs plus proof lemmas for library abstractions. Do not make specs open just to force a call site to verify."}
{"kind":"rule","id":"unfolding_control","text":"Use reveal, hide, reveal_with_fuel, by(compute), or by(compute_only) locally. Do not globally over-unfold recursive specs."}
{"kind":"rule","id":"solver_escalation_local","text":"Use nonlinear_arith, integer_ring, bit_vector, high rlimits, or extra fuel only around the exact assertion or lemma that needs it."}
{"kind":"rule","id":"extensional_equality","text":"For Seq, Set, ISet, Map, and IMap, prefer extensional equality (=~= or =~~= when applicable) when elementwise equality is the obligation."}
{"kind":"rule","id":"type_invariants","text":"Type invariants are not automatically in proof context. Invoke use_type_invariant or the project-approved invariant lemma at the point of use."}
{"kind":"rule","id":"trust_boundary_is_evidence","text":"assume, admit, assume_specification, assume_termination, external_body, external, verus_verify(external*), external_fn_specification, external_type_specification, external_trait_specification, external_trait_extension, exec_spec_unverified!, broadcast axioms, and cfg/verus_only runtime forks expand the trusted base. Report them separately from fully proved verification."}
{"kind":"rule","id":"no_laundering","text":"external_body, assume_specification, verus_verify(external*), exec_spec_unverified!, assumptions, admits, and axioms are forbidden as shortcuts for proving production core logic. Boundary code may be TRUSTED only with owner/scope/reason/expiry and must not be reported as PROVED."}
{"kind":"rule","id":"proof_only_ai_patch","text":"For AI-generated proof work, executable code, signatures, specs, requires/ensures/recommends, invariants, decreases, trusted modules, TCB ledgers, axioms, and external bodies are immutable unless a human explicitly approves the delta. Default mutable regions are proof blocks, lemmas, assertions, loop invariants, and trigger hints."}
{"kind":"rule","id":"intactness_before_success","text":"Verifier pass is only PASS. Acceptable proof output also requires intact exec/spec/TCB regions. Production-usable output additionally requires no new trust markers and full repo CI. Upstream-grade requires maintainer/domain-owner review."}
{"kind":"rule","id":"no_hallucinated_evidence","text":"Never invent verifier output, proof names, command success, warning status, file paths, lemma names, or tool availability."}
{"kind":"rule","id":"pipeline_boundary","text":"In the broader Rust proof stack, Verus is default for Rust-local pure/core logic. Kani/fuzz/Loom/etc. are risk-selected companions; reviewer approval belongs to proof-reviewer or verus-reviewer."}
{"kind":"ref","file":"references/verus-patterns.md","use":"Mode rules, proof idioms, anti-patterns, and failure signatures."}
{"kind":"ref","file":"references/verus-harness.md","use":"Verifier-in-the-loop commands, diagnostics, trust audit, and acceptance gates."}
{"kind":"ref","file":"references/verus-code-parity.md","use":"Proof-to-production mapping and verification laundering checks."}
{"kind":"ref","file":"references/verus-modern-sources.md","use":"Primary docs, local corpus paths, vstd/test-suite lookup, and modern research lessons."}
{"kind":"ref","file":"references/trust-marker-scan.md","use":"Canonical trust-marker scan pattern, scan scopes, host adapters, and parser-aware safe-change requirements."}
{"kind":"ref","file":"references/ai-proof-production-gate.md","use":"AI proof-only patch policy, intactness gate, TCB ledger, benchmark ladder, golden dataset schema, and production review checklist."}
{"kind":"ref","file":"references/verus-curriculum.md","use":"Training/evaluation curriculum and source priority map."}
```

## Mandatory References

Before writing, repairing, or reviewing Verus proof code, read:

1. `references/verus-patterns.md`
2. `references/verus-harness.md`
3. `references/verus-code-parity.md`
4. Relevant project Verus/Rust files and proof obligations

Also read `references/verus-modern-sources.md` when the task involves vstd lemma lookup, newer syntax, `cargo verus`, state machines, LLM proof output, repository-scale proof repair, or review-quality claims. Read `references/ai-proof-production-gate.md` for AI-generated proof work, golden datasets, benchmark design, intactness checks, or production-style proof gates.

If the task is review-only, use `verus-reviewer` for the final adversarial review instead of self-approving.

## Mandatory Verification Gate

Run the repository's exact Verus command when one is named in `proof-obligations.planned.jsonl`, `verification-ledger.jsonl`, or project docs. If no exact command exists, use the nearest repo script/task. If none exists, report a blocker instead of fabricating proof evidence.

Prefer command selection from `references/verus-harness.md`. Typical gates:

```bash
command -v verus >/dev/null
verus --version
if command -v cargo-verus >/dev/null || cargo verus --help >/dev/null 2>&1; then cargo verus verify <scoped-target> -- --no-cheating; fi
if command -v verusfmt >/dev/null; then verusfmt --check <verus-files-or-repo-path>; fi
verus --no-cheating <exact-verus-target-or-command-from-proof-obligations>
# Run the canonical trust-marker scan from references/trust-marker-scan.md with host-native search or parser/token scan; use rg only as an allowed fallback.
```

Accepted proof evidence must include command, exit status, verifier summary, relevant warnings/notes, trusted-boundary scan result, and Cargo-Verus crate opt-in evidence when applicable. Missing `verus`, unknown exact target, or `cargo verus verify` output showing no opted-in crate is `BLOCKER`, not pass. `cargo verus focus` is iteration evidence only; final closure needs `cargo verus verify` or an exact approved command.
For AI-generated proof patches, verifier success is not enough: run or inspect intactness evidence showing no unapproved `exec`, signature, spec, contract, invariant, decreases, trusted-code, or TCB-ledger drift before reporting success.

## Workflow

1. Read the required references and the real source/proof target. If vstd or current syntax matters, inspect local `vstd`, guide, examples, or `rust_verify_test` before inventing lemma names.
2. Classify target as `spec`, `proof`, `exec`, `ghost`, or `tracked`; write/repair the contract shape first.
3. Preserve contract and intactness parity. Do not change executable code, signatures, `requires`, `ensures`, `recommends`, specs, invariants, decreases, trusted code, or TCB ledger entries unless the user explicitly approved that delta.
4. For AI proof generation, edit only proof material by default: proof blocks, proof functions, local lemmas, assertions, loop invariants, trigger hints, local reveal/fuel, and scoped solver hints.
5. Use the smallest proof idiom: local lemma, `assert(goal) by { lemma_call(); }`, loop invariant, trigger, reveal/fuel, compute proof, extensional equality, type-invariant use, or scoped solver escalation.
6. Keep proof-to-code parity: prove actual production symbols or accepted extracted helpers, not copied local models.
7. Run the exact verifier command, read diagnostics, and repair from the smallest failing obligation.
8. Run the trust scan and label every assumption/external boundary as TRUSTED, not PROVED.
9. Report proved claims, trusted claims, intactness status, commands, residual obligations, and blockers.

## Output Contract

When writing or repairing Verus, return:

- Files changed.
- Production symbols constrained by each proof artifact.
- Exact Verus commands run and result.
- Proof idioms used.
- Trusted-base additions or existing trusted boundaries, if any.
- Intactness status for AI-generated proof patches: unchanged exec/spec/TCB, approved deltas, or blocker.
- Residual obligations or blockers.

## Anti-Verification-Laundering Mandate

`#[verifier::external_body]`, `#[verifier(external_body)]`, `assume`, `admit`, `assume_specification`, `assume_termination`, external type/trait specs, `verus_verify(external*)`, `exec_spec_unverified!`, and axioms are strictly forbidden for creating vacuum proofs of production code. You cannot declare a contract on an `exec fn` and then skip proving the body. If a boundary is unavoidable, label it `TRUSTED`, ledger it, and do not claim the underlying behavior is proved.
