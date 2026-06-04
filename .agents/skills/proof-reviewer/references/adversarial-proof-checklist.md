# Adversarial Proof Checklist

General:
- Every obligation maps to a requirement or contract clause.
- Every required obligation has raw command evidence or a valid waiver.
- Every assumption is named and justified.
- No proof depends on deleted tests, fake paths, or unrun commands.

Verus:
- Trusted boundaries are minimal and listed.
- Specs connect to executable functions.
- Lemmas prove something stronger than their preconditions.
- Recursive/loop proof obligations have real decreases/invariants.

Kani:
- Harness reaches the target behavior.
- Unwind bounds match loops/recursion/input sizes.
- Assumptions do not encode the desired result.

Flux:
- Refinements exclude invalid states.
- Constructors enforce predicates.
- Unsupported gaps are explicit.

Loom:
- Production synchronization is represented.
- Cancellation/drop/error paths are modeled.
- Assertions check the contract, not just no panic.

proptest/fuzz:
- Exercised paths match the risk.
- Generators/corpus cover boundary classes.
- Oracles assert contract behavior.
