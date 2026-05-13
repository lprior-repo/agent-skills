# Verifier Commands And Evidence

TLA+ evidence requires model command, config, checked invariants, deadlock status, and counterexample file when failing.

Verus evidence requires verifier command, proof file, trusted boundary list, and pass/fail output.

Kani evidence requires harness name, unwind bounds, assumptions, stubs, and pass/fail output.

Flux evidence requires refinement annotations, checker command, unsupported features, and pass/fail output.

Loom evidence requires model name, represented synchronization primitives, schedule/preemption coverage when configured, and failure trace when failing.

Miri evidence requires nightly command, exercised unsafe/provenance path, unsupported-feature notes, and pass/fail output.

proptest evidence requires generators, properties, regression seeds, shrink output when failing, and pass/fail output.

Fuzz evidence requires target, run budget, corpus/crash artifacts, sanitizer config when used, and pass/fail output.
