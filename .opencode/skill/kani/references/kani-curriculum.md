# Kani Curriculum And Evaluation

## Stage 0: CLI Setup

Goal: know whether local Kani evidence is possible.

Tasks:

- Run `cargo kani --version` or report missing Kani as `BLOCKER` for required obligations.
- Run `cargo kani --help` and check support for selected flags.
- Record solver/backend options and reject disabled-check flags unless the claim is explicitly downgraded.
- Know when `cargo kani setup` is an install step, not proof evidence.
- Never require an editor, VS Code, or UI integration for local proof evidence.

## Stage 1: Basic Harnesses

Goal: write small, non-vacuous proof harnesses.

Tasks:

- Add `#[cfg(kani)] mod kani_proofs`.
- Write a zero-argument `#[kani::proof]` harness.
- Use `kani::any()` for finite input domains.
- Assert a real domain invariant or result property.
- Run `cargo kani --harness <name>` and report exact output.

Evaluation:

- The harness name states the claim.
- The assertion is not a tautology.
- The output includes `VERIFICATION:- SUCCESSFUL` or a real failure is triaged.

## Stage 2: Assumptions And Cover

Goal: avoid vacuous proofs.

Tasks:

- Use `kani::assume` only for real preconditions or bounded model scope.
- Replace repeated assumptions with `any_where` or domain types when clearer.
- Add `kani::cover!` for domain existence and boundary values.
- Explain every failed, unreachable, or unsatisfiable cover result.

Evaluation:

- Impossible assumptions are caught.
- Boundary covers match the claimed input space.
- The proof report separates assumptions from verified facts.

## Stage 3: Bounds And Unwinding

Goal: account for loop and input bounds.

Tasks:

- Use `kani::bounded_any::<_, N>()` for bounded collections.
- Add `#[kani::unwind(N)]` or record the exact command-line unwind policy.
- Interpret unwinding assertion failures as proof failures.
- State claims as bounded, such as `len <= 16`.

Evaluation:

- Harness names or reports include bounds.
- Max/min cover points are present for collection sizes.
- No green result is accepted if unwind checks failed.

## Stage 4: Negative Evidence

Goal: prove invalid states are rejected when making rejection claims.

Tasks:

- Add positive harnesses for valid flows.
- Add `#[kani::should_panic]` or contract-precondition failure harnesses for invalid flows.
- Keep negative harnesses narrow enough to avoid accidental panic sources.

Evaluation:

- Invalid-state claims have exact negative evidence.
- Missing negative evidence is `BLOCKER`.
- `should_panic` results are not overinterpreted as a specific panic site unless independently checked.

## Stage 5: Contracts And Stubs

Goal: use abstractions without lying about trust.

Tasks:

- Use `#[kani::requires]`, `#[kani::ensures]`, and `#[kani::modifies]` only with current feature flags.
- Verify contracted functions with `#[kani::proof_for_contract(target)]`.
- Use `#[kani::stub_verified(target)]` only after contract proof evidence exists.
- Treat `#[kani::stub_verified(target)]` as an active abstraction in the caller harness and record the exact `-Z function-contracts` / `-Z stubbing` evidence.
- Use `#[kani::stub(original, replacement)]` only with a documented abstraction argument.

Evaluation:

- Contract harness command includes `-Z function-contracts` when required.
- Stub harness command includes `-Z stubbing` when required.
- Every stub and contract is listed in the report.

## Stage 6: Unsafe And Unsupported Areas

Goal: report exact coverage and residual risk.

Tasks:

- Scan unsafe blocks, unsafe functions, `transmute`, raw pointer constructors, `MaybeUninit`, and assembly.
- Use pointer helpers and opt-in checks only when supported by current Kani.
- Use `-Z mem-predicates` for `kani::mem::*` predicate APIs when the installed Kani version gates them.
- State which UB-like failures Kani checked and which it did not.

Evaluation:

- No report claims full unsafe soundness.
- Unsupported features become blockers, waivers, or residual risk, not silent pass.

## Final Evaluation Task

Given a crate with:

- One arithmetic overflow risk.
- One parser over bounded byte slices.
- One invalid state transition.
- One over-constrained `kani::assume`.
- One plain stub and one verified stub.
- One unsafe raw-pointer helper.

A competent Kani agent must:

- Inventory all harnesses with `cargo kani list --format json` or a justified fallback.
- Add or repair non-vacuity cover points.
- Report exact input and unwind bounds.
- Run the relevant exact harness commands.
- Reject the over-constrained assumption until fixed.
- Require contract proof evidence before accepting `stub_verified`.
- Report the plain stub as trusted abstraction debt.
- Reject any disabled-check command that still claims the disabled property class.
- Record solver/backend/CBMC-args and memory-predicate feature gates.
- Avoid claiming full UB or concurrency correctness.

## Black-Hat Checklist

Reject a Kani plan or report if it:

- Has no exact command evidence.
- Has no harness inventory.
- Omits bounds or unwind information.
- Uses assumptions without non-vacuity evidence.
- Ignores `UNREACHABLE`, `UNSATISFIABLE`, `UNDETERMINED`, or unwinding failures.
- Hides stubs, FFI models, contracts, or experimental flags.
- Uses `--no-*checks`, `--prove-safety-only`, `--only-codegen`, `--no-codegen`, or `--ignore-global-asm` without downgrading the claim.
- Omits `#[kani::solver(...)]`, `--solver`, `--cbmc-args`, or `-Z mem-predicates` when those surfaces are present.
- Claims unbounded correctness from bounded inputs.
- Claims full unsafe-code soundness.
- Treats `cargo test`, clippy, Verus, Flux, or UB-tooling output as a substitute for required Kani evidence.
