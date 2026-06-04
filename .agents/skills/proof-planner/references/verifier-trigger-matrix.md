# Verifier Trigger Matrix

| Risk trigger | Primary lane | Required when |
|---|---|---|
| Rust-local invariant, API contract, illegal state, pure/core function correctness | Verus | The invariant is safety-critical or cheaper to prove than exhaustively test |
| Bounded state machine, parser, codec, arithmetic/index bounds, finite transition system | Kani | The input/state space can be bounded honestly |
| Refinement/type-state, numeric/data predicate, constructor-enforced invalid-state exclusion | Flux | The property is naturally expressible as a refinement |
| Threads, atomics, channels, locks, async shutdown, scheduler races | Loom | Correctness depends on concurrent interleavings |
| Unsafe, FFI, raw pointers, aliasing, provenance, interior-mutability UB risk | specialist scoping required | Runtime UB or provenance risk is in scope |
| Broad input space, serialization, domain invariants over generated values | proptest | Examples cannot cover the relevant input classes |
| Untrusted/crash/security input boundary, parser/protocol frame | fuzz | Robustness against adversarial input matters |
| Dependency files changed | cargo audit/deny/vet/geiger via existing gates | Dependency risk is touched by the bead |

Default to the cheapest lane that kills the real risk. Escalate when failure is catastrophic, security-sensitive, persistent, concurrent, unsafe, distributed, or too broad for honest examples.
