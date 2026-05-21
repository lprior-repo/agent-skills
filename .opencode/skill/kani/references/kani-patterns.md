# Kani Patterns

Keep harnesses small, named after the claim, and explicit about bounds. Prefer one harness per critical property. Add comments only when a bound or assumption is not obvious from the code.

## Basic Harness Shape

```rust
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    #[kani::proof]
    fn verify_increment_no_overflow_for_small_inputs() {
        let x: u32 = kani::any();
        kani::assume(x < u32::MAX);
        kani::cover!(x == 0, "domain includes zero");
        kani::cover!(x == u32::MAX - 1, "domain includes boundary");

        let y = increment(x);
        assert_eq!(y, x + 1);
    }
}
```

Rules:

- Put Kani-only code behind `#[cfg(kani)]` unless the project intentionally exposes it.
- Use `kani::any()` for finite full-domain inputs.
- Use `kani::assume()` only for real preconditions or bounded proof scope.
- Add `kani::cover!` for critical domains and boundaries.
- Do not assert tautologies such as `result.is_ok() || result.is_err()`.

## Bounded Collections

```rust
#[kani::proof]
#[kani::unwind(17)]
fn verify_reverse_twice_for_vec_len_le_16() {
    let xs: Vec<bool> = kani::bounded_any::<_, 16>();

    kani::cover!(xs.len() == 0, "empty vector covered");
    kani::cover!(xs.len() == 16, "max vector length covered");

    let ys = reverse(reverse(xs.clone()));
    assert_eq!(xs, ys);
}
```

Rules:

- Include the bound in the harness name or report.
- Prove min, max, and representative boundary values are reachable.
- State the proof as `len <= 16`, not all vectors.
- Use unwind at least as high as the maximum loop iterations plus required loop-exit checks.

## Assumption Discipline

Bad:

```rust
#[kani::proof]
fn vacuous() {
    let x: u8 = kani::any();
    kani::assume(x > 250);
    kani::assume(x < 10);
    assert!(false);
}
```

Good:

```rust
#[kani::proof]
fn bounded_domain_is_nonempty() {
    let x: u8 = kani::any_where(|x| *x >= 10 && *x <= 20);
    kani::cover!(x == 10, "lower boundary reachable");
    kani::cover!(x == 20, "upper boundary reachable");

    assert!((10..=20).contains(&x));
}
```

Rules:

- Every `kani::assume`, `any_where`, and contract `requires` is proof context.
- Move real API preconditions into types or contracts where possible.
- Use cover points to make empty domains visible.
- Place assumptions before invoking the function they constrain.

## State Transitions

```rust
#[kani::proof]
fn verify_machine_preserves_invariant_one_step() {
    let state: MachineState = kani::any();
    let event: Event = kani::any();
    kani::assume(state.invariant());

    let next = step(state, event);
    assert!(next.invariant());
}
```

Rules:

- This checks one bounded implementation step, not temporal behavior over arbitrary traces.
- For workflows, first model the design in TLA+ when lifecycle, fairness, retries, leases, or distributed coordination matter.
- Use additional bounded trace harnesses only as implementation evidence, not as a replacement for temporal specs.

## Negative Harnesses

```rust
#[kani::proof]
fn init_once_succeeds() {
    let mut device = Device::new();
    assert_eq!(device.try_init(), Ok(()));
    assert!(device.is_initialized());
}

#[kani::proof]
fn init_twice_returns_exact_rejection() {
    let mut device = Device::new();
    assert_eq!(device.try_init(), Ok(()));
    assert!(device.is_initialized());

    assert_eq!(device.try_init(), Err(DeviceError::AlreadyInitialized));
    assert!(device.is_initialized());
}
```

Rules:

- `#[kani::should_panic]` proves some panic/assertion failure is reachable, not necessarily the intended one.
- Prefer exact `Result`/error-variant rejection harnesses over `#[kani::should_panic]` when the API exposes an error value.
- Pair negative harnesses with positive harnesses.
- Keep negative harnesses narrow enough that unrelated panics do not satisfy them.

## Function Contracts

Function contracts are experimental and require current Kani feature-flag evidence, typically `-Z function-contracts`.

```rust
#[kani::requires(divisor != 0)]
#[kani::ensures(|result: &usize| *result <= dividend)]
fn safe_div(dividend: usize, divisor: usize) -> usize {
    dividend / divisor
}

#[kani::proof_for_contract(safe_div)]
fn verify_safe_div_contract() {
    let dividend: usize = kani::any();
    let divisor: usize = kani::any();
    safe_div(dividend, divisor);
}
```

Rules:

- `#[kani::proof_for_contract(target)]` is contract evidence; a normal `#[kani::proof]` harness is not enough.
- `ensures` receives the result by reference.
- Restrictions hidden in `kani::assume` inside a contract harness are suspicious; real preconditions belong in `requires`.
- Mutating functions need accurate `#[kani::modifies(...)]` clauses.
- Generic contracts prove only the monomorphizations exercised by the harness.

## Stubbing

Plain stubbing is an unstable abstraction and normally requires `-Z stubbing`.

```rust
#[cfg(kani)]
fn random_stub<T: kani::Arbitrary>() -> T {
    kani::any()
}

#[kani::proof]
#[kani::stub(rand::random, random_stub)]
fn caller_handles_all_random_values() {
    let result = caller();
    assert!(result.invariant());
}
```

Rules:

- `#[kani::stub(original, replacement)]` is a harness-local replacement.
- Report every active stub from source and output.
- Explain whether the stub over-approximates, under-approximates, or exactly models production behavior.
- FFI stubs and randomness/time/I/O stubs are trusted verification models.
- `#[kani::stub_verified(target)]` is still an active caller-side abstraction. Report it with stubs, require a separately passing `#[kani::proof_for_contract(target)]` harness, and require local evidence for the exact `-Z function-contracts` / `-Z stubbing` flags used.

## Unsafe Harnesses

Memory predicate APIs such as `kani::mem::can_dereference` are experimental in current local Kani help; run the harness with `-Z mem-predicates` unless current local help proves the gate changed.

```rust
#[kani::proof]
fn verify_raw_pointer_guard() {
    let mut value: u32 = kani::any();
    let pointer: *mut u32 = &mut value;

    kani::assert(kani::mem::can_dereference(pointer), "local pointer is dereferenceable");

    let observed = unsafe { *pointer };
    assert_eq!(observed, value);
}
```

Rules:

- State exactly which unsafe failure classes are modeled.
- Do not claim aliasing-model, data-race, ABI, invalid-value, or uninitialized-memory freedom unless current Kani flags and docs support that exact claim.
- Use `-Z mem-predicates` for `kani::mem::*` predicate APIs when the installed Kani version gates them.
- Consider `-Z valid-value-checks` or `-Z uninit-checks` only when the current Kani version supports them; label them experimental.

## Anti-Patterns

- Running `cargo kani` and claiming every critical invariant was checked without a harness inventory.
- Lowering `#[kani::unwind]` until the proof is green.
- Ignoring failed cover checks because the final summary is green for assertions.
- Using `kani::assume` for invalid production states that should be rejected by the API.
- Replacing hard code with a friendly stub and calling the result verified.
- Treating `#[kani::should_panic]` as proof of a specific panic site.
- Using `--no-*checks`, `--prove-safety-only`, `--only-codegen`, `--no-codegen`, or `--ignore-global-asm` while claiming the disabled/skipped property class was proved.
- Using `kani::mem::*` APIs without recording the required `-Z mem-predicates` evidence.
- Omitting `#[kani::solver(...)]`, `--solver`, or `--cbmc-args` from the proof context.
- Claiming Kani verifies async/concurrent behavior, atomics ordering, or thread schedules.
