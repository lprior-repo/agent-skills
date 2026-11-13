# Verus Curriculum And Sources

## Source Priority

| Priority | Source | Use |
|---|---|---|
| Highest | `references/verus-modern-sources.md` | Local corpus paths, current source map, and modern research index |
| Highest | <https://verus-lang.github.io/verus/guide/> | Normative syntax, semantics, idioms |
| Highest | <https://github.com/verus-lang/verus> | Examples, tests, vstd, evolving practice |
| Highest | `source/vstd/` in a local Verus checkout | Real standard-library definitions and lemma names |
| Highest | `source/rust_verify_test/tests/` in a local Verus checkout | Newer syntax, regression patterns, feature behavior |
| High | <https://github.com/verus-lang/verus/blob/main/CONTRIBUTING.md> | Formatting, logging, test workflow, verusfmt/vargo |
| High | <https://verus-lang.github.io/verus/state_machines/> | Tracked state, tokens, state-machine and concurrency patterns |
| High | <https://arxiv.org/abs/2303.05491> | Conceptual model: modes, SMT, linear ghost state |
| High | <https://verus-lang.github.io/verus/guide/llmforverusproof.html> | Modern agent workflow, vstd/test-suite lookup, verifier loop, cheat checker |
| Medium | <https://arxiv.org/abs/2409.13082> | AutoVerus function-level proof synthesis lessons |
| Medium | <https://arxiv.org/abs/2502.05344> | RAG-Verus repository-level retrieval lessons |
| Medium | <https://arxiv.org/abs/2512.18436> | Agent-based system verification benchmark lessons |
| Medium | <https://arxiv.org/abs/2603.25810> | Counterexample-guided proof repair and invariant discovery lessons |
| Medium | <https://arxiv.org/abs/2605.03822> | Dependency-aware, resilient repository-level Verus repair lessons |
| Medium | <https://microsoft.github.io/z3guide/docs/logic/Quantifiers/> | Trigger, pattern, matching-loop, and SMT quantifier intuition |

## Staged Skill Progression

| Stage | Goal | Must learn | Evidence |
|---|---|---|---|
| Foundations | Modes, contracts, integers, ghost/exec split | `spec` vs `proof` vs `exec`, `requires`/`ensures`, `recommends`, `int`/`nat` | Small file verifies without assumptions |
| Local proofs | Lemmas and proof blocks | `proof fn`, `assert(goal) by { lemma_call(); }`, closed specs | Lemma call proves a concrete exec postcondition |
| Recursion and loops | Termination and invariants | `decreases`, loop isolation, functional relation invariants | Loop proof restates ambient facts and exits cleanly |
| Quantifiers and collections | Solver instantiation | triggers, `Seq`, `Set`, `Map`, extensional equality | Trigger fires at real use site; no matching-loop smell |
| Automation control | Stable solver use | reveal/fuel, compute, nonlinear arithmetic, integer ring, bit-vector | Escalation scoped to exact assertion/lemma |
| Encapsulation and trust | Sound abstraction | type invariants, closed/opaque specs, trust-boundary audit | Trust scan clean or ledgered; PROVED/TRUSTED labels correct |
| Tracked state | Advanced ownership/concurrency | tokens, state machines, transition systems, linear ghost resources | Abstract transition maps to actual implementation event |
| Repository repair | Cross-file proof work | imports, visibility, existing lemmas, exact diagnostics, cargo-verus | Full exact command verifies target; no copied-model gap |
| Review | Adversarial proof assessment | proof-to-code parity, verifier evidence, laundering patterns | `verus-reviewer` review has zero unresolved findings |

## Training Transformations

- Proof ablation: remove one invariant, trigger, reveal, or lemma call.
- Contract completion: provide body and ask for `requires`/`ensures`/invariants without changing semantics.
- Proof repair: provide verifier diagnostic and require minimal fix.
- Visibility swap: convert open spec to closed spec plus lemma interface.
- Trigger edit: compare candidate triggers and label verifier behavior/performance.
- Trust normalization: replace shortcuts with explicit trusted-boundary labels or remove them.
- Repository slicing: start local, then add cross-file dependency context.
- Code parity check: compare proof artifact to production symbol and reject copied model.
- Evidence audit: distinguish `cargo verus focus` iteration evidence from final `cargo verus verify`.

## Evaluation Metrics

- Verification pass rate.
- Assumption-free pass rate.
- Code-parity pass rate.
- Warning/notes-clean pass rate.
- Trusted-boundary ledger completeness.
- Median verification time and tail latency.
- Robustness under stricter resource limits.
- Stability under harmless source perturbations.
- Minimal retrieved context needed.
- Repair iterations to pass.
- Reviewer rejection rate for intentionally laundered proofs.
