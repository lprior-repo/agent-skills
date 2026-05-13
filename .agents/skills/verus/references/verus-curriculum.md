# Verus Curriculum And Sources

## Source Priority

| Priority | Source | Use |
|---|---|---|
| Highest | https://verus-lang.github.io/verus/guide/ | Normative syntax, semantics, idioms |
| Highest | https://github.com/verus-lang/verus | Examples, tests, evolving practice |
| High | https://github.com/verus-lang/verus/blob/main/CONTRIBUTING.md | Formatting, logging, test workflow |
| High | https://verus-lang.github.io/verus/state_machines/ | Tracked state, tokens, concurrency patterns |
| High | https://arxiv.org/abs/2303.05491 | Conceptual model: modes, SMT, linear ghost state |
| Medium | https://arxiv.org/abs/2409.13082 | AutoVerus function-level proof synthesis lessons |
| Medium | https://arxiv.org/abs/2502.05344 | RAG-Verus repository-level retrieval lessons |
| Medium | https://microsoft.github.io/z3guide/docs/logic/Quantifiers/ | Trigger and SMT quantifier intuition |

## Staged Skill Progression

| Stage | Goal | Must learn |
|---|---|---|
| Foundations | Modes, contracts, integers, ghost/exec split | `spec` vs `proof` vs `exec`, `requires`/`ensures`, `int`/`nat` |
| Local proofs | Lemmas and proof blocks | `proof fn`, `assert(goal) by { lemma_call(); }`, closed specs |
| Recursion and loops | Termination and invariants | `decreases`, loop isolation, functional relation invariants |
| Quantifiers and collections | Solver instantiation | triggers, `Seq`, `Set`, `Map`, extensional equality |
| Automation control | Stable solver use | reveal/fuel, compute, nonlinear arithmetic, integer ring |
| Encapsulation and trust | Sound abstraction | type invariants, trusted boundary audit |
| Tracked state | Advanced ownership/concurrency | tokens, transition systems, linear ghost resources |
| Repository repair | Cross-file proof work | imports, visibility, existing lemmas, exact diagnostics |

## Training Transformations

- Proof ablation: remove one invariant, trigger, reveal, or lemma call.
- Contract completion: provide body and ask for `requires`/`ensures`/invariants.
- Proof repair: provide verifier diagnostic and require minimal fix.
- Visibility swap: convert open spec to closed spec plus lemma interface.
- Trigger edit: compare candidate triggers and label verifier behavior.
- Trust normalization: replace shortcuts with explicit trusted-boundary labels or remove them.
- Repository slicing: start local, then add cross-file dependency context.

## Evaluation Metrics

- Verification pass rate.
- Assumption-free pass rate.
- Warning-clean pass rate.
- Median verification time and tail latency.
- Robustness under stricter resource limits.
- Stability under harmless source perturbations.
- Minimal retrieved context needed.
- Repair iterations to pass.
