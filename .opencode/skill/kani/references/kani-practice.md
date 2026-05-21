# Kani Practice Guide

Kani is a bounded model checker for Rust proof harnesses. It uses symbolic execution and a CBMC backend to check modeled executions for assertions, panics, arithmetic and indexing failures, many pointer-dereference failures, and explicit properties written in harnesses or function contracts.

Use Kani when the question is implementation-local and bounded: numeric safety, array/slice bounds, parser state transitions over small inputs, panic freedom for finite domains, constructor rejection, unsafe boundary smoke proofs, or contract/stub validation. Do not use Kani as the design model for temporal/distributed behavior, concurrency interleavings, liveness, protocol fairness, or unbounded mathematical proof.

## Source Priority

Prefer current official sources and local evidence in this order:

1. `cargo kani --version`, `kani --version`, and `cargo kani --help` from the machine running the proof.
2. Official Kani Book: `https://model-checking.github.io/kani/`.
3. Kani crate docs: `https://model-checking.github.io/kani/crates/doc/kani/`.
4. Kani GitHub repository and release notes: `https://github.com/model-checking/kani`.
5. Local repository proof obligations, scripts, CI config, and prior evidence artifacts.

If local commands and docs disagree, report the conflict and fail closed for required obligations.

## Mental Model

- A Kani proof harness is a zero-argument Rust function annotated with `#[kani::proof]` or `#[kani::proof_for_contract(target)]`.
- `kani::any::<T>()` creates a symbolic valid value for every modeled value of `T` that implements `kani::Arbitrary`.
- `kani::bounded_any::<_, N>()` creates bounded symbolic collections; the proof is only valid up to `N`.
- `kani::assume(pred)` removes paths where `pred` is false after that statement. It can make proofs vacuous.
- `kani::cover!(cond, msg)` asks Kani to show that a condition is reachable; use it to defend against impossible assumptions.
- Loops are checked up to unwind bounds. All unwinding assertions must pass.
- `VERIFICATION:- SUCCESSFUL` means the selected harnesses passed under the recorded model, not that the whole crate is correct.

## What Kani Can Prove Well

- Panics and assertions are unreachable for all modeled harness inputs.
- Integer arithmetic does not overflow under the modeled inputs and enabled checks.
- Slice, array, and pointer dereference checks are safe in the modeled executions.
- State-transition invariants hold over bounded event domains.
- Invalid call sequences panic or violate a contract in an explicit negative harness.
- A function contract holds for the target function when `#[kani::proof_for_contract]` evidence passes with `-Z function-contracts`.
- A caller is safe under a stub or contract abstraction when the stub/trust boundary is documented and the relevant flags are used.

## What Kani Must Not Be Claimed To Prove

- Unbounded correctness for arbitrary-length vectors, strings, trees, protocol histories, or recursion depths.
- Full Rust UB freedom, RustBelt-style memory safety, aliasing model compliance, or data-race freedom.
- Async/concurrent behavior, scheduler fairness, atomics ordering, or thread interleavings.
- ABI correctness, inline assembly behavior, external service behavior, file system/network behavior, or CLI output formatting.
- Verus-style deductive proofs of Rust-local pure logic when Verus is the required proof layer.
- TLA+ temporal workflow invariants or liveness.
- Flux RS refinement obligations.
- Security absence, side-channel freedom, performance, or production runtime behavior without a dedicated model for that claim.

## Evidence Wording

Use precise language:

> Kani verified harness `verify_len_4_parser` with `bytes: [u8; 4]`, `len <= 4`, `#[kani::unwind(6)]`, no failed/unwound/undetermined checks, and cover points for `len == 0` and `len == 4` satisfied.

Avoid broad language:

> Kani proved the parser correct.

## Tool Boundaries

- Use TLA+ for temporal state machines, liveness, fairness, distributed coordination, leases, queues, and protocol design.
- Use Verus for Rust-local deductive proof obligations, algebraic invariants, loop invariants that need unbounded reasoning, and proof functions.
- Use Flux RS for refinement types, legal-state encodings, range/length/index relationships at compile time, and lightweight contract checking.
- Use Miri/cargo-careful/sanitizers for UB exploration that Kani does not model well.
- Use Loom/Shuttle/Stateright/Lockbud for implementation interleavings and concurrency schedules.
- Use fuzzing/proptest/Bolero for hostile input spaces and long-running bug discovery.

## Black-Hat Rules

Reject Kani evidence if any of these are true:

- The command did not name or discover the harnesses relevant to the claim.
- The proof has assumptions without non-vacuity or boundary evidence.
- The proof depends on a hidden stub, FFI model, or unverified contract.
- The report omits unwind bounds or ignores unwinding assertion failures.
- The command uses disabled, skipped, or weakening verification flags such as `--no-default-checks`, `--no-memory-safety-checks`, `--no-overflow-checks`, `--no-unwinding-checks`, `--prove-safety-only`, `--only-codegen`, `--no-codegen`, or `--ignore-global-asm` while still claiming those property classes.
- The report omits solver/backend context such as `#[kani::solver(...)]`, `--solver`, or `--cbmc-args`.
- The harness uses `kani::mem::*` predicate APIs without the current required `-Z mem-predicates` evidence.
- The result contains `FAILURE`, `UNDETERMINED`, unsupported-feature diagnostics, solver/resource exhaustion, or failed cover points needed for non-vacuity.
- A bounded proof is described as unbounded.
- Unsafe code is declared sound without a residual-risk section.
