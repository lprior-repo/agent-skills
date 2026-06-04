# Contract Specification

## Context
- **Feature**: Eliminate all panic violations in clock_skew_chaos.rs test suite
- **Domain terms**:
  - `ChaosTestError`: Test-specific error type with semantic variants
  - `assert_chaos!`: Custom macro that returns `ChaosTestError` instead of panicking
  - `assert_eq_chaos!`: Custom macro for equality checks that returns `ChaosTestError`
  - Clock skew simulation: Testing RPC timeout behavior under system clock adjustments
  - Monotonic invariant: `Instant::now()` never decreases (immune to clock skew)
- **Assumptions**:
  - Test implementation style is not a rejection gate; exact behavior and error evidence are mandatory
  - The file already has custom assertion macros defined (lines 38-54)
  - Standard `assert!`, `assert_eq!`, and `.expect()` are the violations
  - All test functions return `Result<(), ChaosTestError>`
  - The custom macros are fully functional and tested
- **Open questions**: None

## Preconditions
- Test file must compile and execute through the normal test command
- Custom assertion macros (`assert_chaos!`, `assert_eq_chaos!`) must be defined
- All test functions must return `Result<(), ChaosTestError>`
- `ChaosTestError` must have appropriate error variants for different failure modes

## Postconditions
- Zero calls to standard `assert!` macro in test code
- Zero calls to standard `assert_eq!` macro in test code
- Zero calls to `.expect()` in test code
- All assertions use `assert_chaos!` or `assert_eq_chaos!` macros
- All `.expect()` calls replaced with a typed `.map_err` pattern
- Test compile check passes: `cargo test --all-features --no-run`

## Invariants
- All test failures must return `Err(ChaosTestError)` instead of panicking
- Test failure evidence must preserve the semantic intent of the original assertion
- No production code changes (only test code modifications)

## Error Taxonomy

### Existing `ChaosTestError` Variants (Already Defined)
- `ChaosTestError::RpcTimeout { timeout_ms }` - RPC call exceeded timeout
- `ChaosTestError::SpuriousTimeout { elapsed_ms, timeout_ms }` - Timeout fired when it shouldn't have
- `ChaosTestError::MissedTimeout { elapsed_ms, timeout_ms }` - Timeout didn't fire when it should have
- `ChaosTestError::StateMismatch { details }` - Actor state inconsistent
- `ChaosTestError::ClockSkewFailed { reason }` - Clock skew simulation failed
- `ChaosTestError::RpcFailed { reason }` - Actor RPC call failed
- `ChaosTestError::SetupFailed { reason }` - Test environment setup failed
- `ChaosTestError::SupervisorMeltdown` - Supervisor restarted too many times
- `ChaosTestError::InvariantViolated { details }` - Generic invariant violation

### Error Variant Mapping
- `assert!` failures → `InvariantViolated` variant
- `assert_eq!` failures → `InvariantViolated` variant with details
- `.expect()` failures → Appropriate variant based on context:
  - For setup operations → `SetupFailed`
  - For RPC operations → `RpcFailed`
  - For state checks → `InvariantViolated`

## Contract Signatures

### Test Function Signature
```rust
async fn test_<name>() -> Result<(), ChaosTestError>
```

### Custom Macro Signatures (Already Defined)
```rust
macro_rules! assert_chaos {
    ($condition:expr, $error:expr) => {
        if !$condition {
            return Err($error);
        }
    };
}

macro_rules! assert_eq_chaos {
    ($left:expr, $right:expr, $error:expr) => {
        if $left != $right {
            return Err($error);
        }
    };
}
```

### Error Conversion Pattern
```rust
// Replace .expect() with:
.some_operation()
    .map_err(|e| ChaosTestError::<Variant> {
        reason: format!("operation failed: {e}")
    })?
```

## Non-goals
- Do NOT add test-style lint gates as acceptance criteria
- Do NOT modify custom assertion macro implementations
- Do NOT change test logic or behavior
- Do NOT add new error variants (use existing `InvariantViolated`)
- Do NOT modify production code (only test modifications)
