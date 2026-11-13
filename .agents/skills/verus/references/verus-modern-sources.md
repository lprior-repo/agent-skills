# Verus Modern Sources

Use this as the source map before writing or reviewing Verus proof work. Prefer local source, then primary URLs. Do not rely on memory when a page or file is available.

## Local Corpus Discovery

Prefer a local Verus checkout when available, but do not require a workstation-specific path. Discover in this order:

1. `VERUS_REPO` environment variable.
2. Workspace-local `verus-modern-corpus/` checkout.
3. Optional research-pass path `/home/lewis/tmp/verus-modern-corpus` when it exists.
4. Official primary URLs listed below.

Once a checkout is located, important paths include:

- `README.md` — project status, docs, playground, verusfmt, active-development warning.
- `CONTRIBUTING.md` — `vargo fmt`, `vargo test`, `VERUS_EXTRA_ARGS`, logging, vstd conventions.
- `source/docs/guide/src/SUMMARY.md` — current guide table of contents and exact page names.
- `source/docs/guide/src/llmforverusproof.md` — LLM proof workflow, resources, verifier loop, cheat checker.
- `source/docs/guide/src/cargo_verus.md` — `cargo verus verify/focus/build` and workspace opt-in.
- `source/docs/guide/src/tcb.md` — assumptions and trusted components.
- `source/docs/guide/src/erasure.md` — ghost erasure and `verus_only` soundness hazard.
- `source/docs/guide/src/forall.md` and `quantproofs.md` — trigger and quantified proof practice.
- `source/docs/guide/src/while.md` and `invariants.md` — loop isolation and invariant design.
- `source/docs/guide/src/opaque.md` and `assert_by.md` — reveal/hide and local proof-context control.
- `source/docs/guide/src/nonlinear.md` — nonlinear arithmetic, `integer_ring`, context minimization.
- `source/docs/guide/src/reference-type-invariants.md` — type invariant rules and `use_type_invariant`.
- `source/vstd/` — standard-library definitions and lemmas; check real lemma names here before using them.
- `source/rust_verify_test/tests/` — evolving Verus syntax/features and regression examples.
- `examples/` — standalone patterns, including BST, iterators, atomics, state-machine, quantifier, bit-vector, and `compute_only` examples.
- `source/docs/state_machines/src/` — `state_machine!`, `tokenized_state_machine!`, tracked tokens, storage protocols.

## Primary Online Sources

- Official guide: <https://verus-lang.github.io/verus/guide/>
- Verus repository: <https://github.com/verus-lang/verus>
- Verus standard library docs: <https://verus-lang.github.io/verus/verusdoc/vstd/>
- Transition systems guide: <https://verus-lang.github.io/verus/state_machines/>
- Publications/projects index: <https://verus-lang.github.io/verus/publications-and-projects/>
- Verus paper: <https://arxiv.org/abs/2303.05491>
- AutoVerus: <https://arxiv.org/abs/2409.13082>
- RAG-Verus: <https://arxiv.org/abs/2502.05344>
- VeruSAGE: <https://arxiv.org/abs/2512.18436>
- ExVerus: <https://arxiv.org/abs/2603.25810>
- KVerus: <https://arxiv.org/abs/2605.03822>
- Z3 quantifier guide: <https://microsoft.github.io/z3guide/docs/logic/Quantifiers/>

## Research Findings To Encode In Skills

1. Verus statically verifies Rust code against specifications and erases ghost code; it does not add runtime checks.
2. Verus uses a three-mode model: `spec` for mathematical definitions, `proof` for ghost proof steps, and `exec` for compiled code. Mode confusion is a root cause of bad proof edits.
3. `exec fn` and `proof fn` use `requires`/`ensures`; `spec fn` uses `recommends`, not hard preconditions.
4. `spec` code can use mathematical `int`/`nat`; executable code must use compilable Rust integer types and prove conversions/bounds.
5. Verus verifies functions and loops modularly; loop invariants must restate needed surrounding facts because loop isolation prevents inherited preconditions by default.
6. Quantifier triggers are programmatic instantiation hints. Triggers must contain bound variables and match terms that appear at use sites; broad triggers and matching loops damage proof performance.
7. `assert(...) by { ... }`, auxiliary lemmas, `opaque`/`reveal`, fuel, and specialized prover modes should be local. Global fuel or broad nonlinear context is a maintenance smell.
8. `assume`, `admit`, `assume_specification`, `assume_termination`, axioms, `external_body`, `external_fn_specification`, `external_type_specification`, `external_trait_specification`, `external_trait_extension`, and indirect `external`/`verus_verify(external*)` use expand the trusted base. They are not full proof evidence.
9. `external_body` is a legitimate boundary for unverified I/O or foreign code only when explicit, ledgered, and not used to avoid proving production core logic.
10. `--no-cheating` is a useful final-evidence guard where supported; it catches `assume`, `admit`, `external_body`, `assume_specification`, and `assume_termination`, but it does not replace source review or trust-ledger classification.
11. `verus_only` should guard imports/config attributes only; conditional executable behavior under `verus_only` can break verification guarantees.
12. Public verified APIs callable from unverified safe Rust need safe-API evidence such as `-V check-safe-api` or an explicit wrapper/dynamic-check rationale.
13. `cargo verus verify` is the end-to-end workspace gate for opted-in crates. `cargo verus focus` is an iteration shortcut and must not be final evidence.
14. `cargo verus` only verifies crates that opt in with `[package.metadata.verus] verify = true`; review command output and manifests so “nothing verified” does not masquerade as success.
15. Modern Verus LLM guidance explicitly recommends providing `vstd`, tests, guide sources, a Verus binary, iterative verifier feedback, and a cheat checker.
16. Modern automation papers reinforce repository-level retrieval, dependency-aware proof repair, counterexample-guided invariant repair, and agent/tool loops. Reviewer skills must therefore demand source refs, exact commands, and proof-to-code traceability rather than accepting isolated snippets.
