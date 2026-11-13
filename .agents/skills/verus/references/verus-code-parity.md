# Verus Code Parity

Proof evidence only matters when it constrains the code that will run or the verified abstraction the project explicitly accepts. A Verus proof that verifies a local copy, toy model, or strengthened contract is not implementation evidence.

## Required Mapping

For every behavior-affecting proof claim, require all of the following:

| Field | Required content | Reject when |
|---|---|---|
| Production target | Exact Rust/Verus symbol, file, and line/range for the function/type/state transition being constrained | Only a file, module, comment, copied model, or helper name is named |
| Spec target | Exact `spec fn`, invariant, pre/postcondition, state-machine predicate, or view `@` relation | Spec is prose-only or hidden behind an unreviewed external/trusted boundary |
| Proof artifact | Exact proof fn/block/invariant/lemma/harness and the command that verifies it | Artifact is not compiled/verified by the named command |
| Behavior evidence | Unit/integration/BDD/proptest evidence independent from the verifier when production behavior changes | Only verifier output is provided for runtime behavior |
| Trust boundary | Every `assume`, `admit`, `assume_specification`, `assume_termination`, axiom, `external_body`, `external`, `verus_verify(external_body|external)`, `external_fn_specification`, `external_type_specification`, `external_trait_specification`, `external_trait_extension`, `exec_spec_unverified!`, and `verus_only` conditional is ledgered | Trust marker is unledgered, unapproved, broad, or hides production logic |

## Proof-To-Code Review Questions

1. Does the proof call or verify the production function, or a separately maintained clone?
2. Did any precondition become stronger or postcondition weaker since the obligation was planned?
3. Does the implementation still execute the same branches, arithmetic, data movement, and state transitions?
4. Are loop invariants strong enough to prove the postcondition and weak enough to be maintained for real loop exits?
5. Do quantified specs use triggers that can fire at real use sites rather than on artificial helper terms only?
6. Are type invariants used at the point of need with `use_type_invariant` or project lemmas?
7. Are `open` specs exposing internals only where abstraction requires it, or did the proof writer open/inline to force verification?
8. Is solver escalation local to the exact lemma/assertion, or did it increase global proof context/rlimit/fuel?
9. Does the final command prove all relevant crates/modules, not only a focused or partial target?
10. Would deleting the production behavior or replacing it with a different result make at least one proof or behavior test fail?
11. Is every `requires`/`recommends` satisfiable by a named production caller or witness?
12. Are quantified domains nonempty when the proof claims concrete coverage?
13. Would an extracted helper proof fail to matter if the production path bypassed the helper?
14. Does bridge evidence show production call sites pass equivalent inputs, observe equivalent outputs/errors, and cannot bypass the verified helper?

## Verification Laundering Patterns

Reject as high or critical unless explicitly approved as an unverified boundary:

- `#[verifier::external_body]` on an `exec fn` whose contract claims real production behavior.
- A proof harness that reimplements the algorithm in `spec` or local helper code instead of constraining the real function.
- Strengthened `requires` clauses that exclude failing inputs without a contract change approval.
- Weakened `ensures`, removed invariants, or replaced postconditions with tautologies.
- `assume(false)`, `admit()`, `assume_specification`, `assume_termination`, external type/trait specifications/extensions, broadcast axioms, or `#[verifier::external]`/`#[verus_verify(external)]` use without a trust ledger entry.
- `#[cfg(verus_only)]` changing executable behavior rather than guarding ghost imports/config.
- `cargo verus focus`, `verus --version`, `cargo build/check`, or syntax-only checks presented as final proof closure.
- `exec_spec_unverified!` or spec/exec bridge macros presented as proved spec-to-exec equivalence without macro-expanded or parser-aware evidence.

## Acceptable Boundary Pattern

A boundary can remain if all are true:

1. The boundary is unavoidable I/O, foreign code, platform behavior, or intentionally unverified component.
2. The contract states only what the project is willing to trust.
3. The trust ledger names owner, reason, scope, expiry/revisit trigger, and compensating evidence.
4. No verified core function depends on unstated facts from the boundary.
5. The final report says `TRUSTED`, not `PROVED`, for claims that cross the boundary.

## Extracted Helper Bridge

An accepted extracted helper is implementation evidence only when all are present:

1. Exact production call-site refs.
2. Argument/view equivalence between production inputs and helper inputs.
3. Return, error, and state-transition equivalence.
4. Visibility/bypass analysis proving production code cannot skip the verified helper.
5. Verifier evidence for any wrapper contract.
6. Independent behavior evidence for the production path.

Reject helper-only proofs when this bridge is absent.
