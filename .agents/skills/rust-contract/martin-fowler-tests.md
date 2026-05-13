# Martin Fowler Test Plan

## Happy Path Tests
- test_returns_success_when_all_assertions_use_custom_macros
- test_compiles_without_changing_behavior
- test_maintains_original_test_semantics_after_refactoring

## Error Path Tests
- test_returns_invariant_error_when_assertion_condition_fails
- test_returns_invariant_error_when_equality_assertion_fails
- test_preserves_error_message_from_original_assertion

## Edge Case Tests
- test_handles_nested_expressions_in_assertions
- test_handles_formatting_in_error_messages
- test_handles_multiple_assertions_in_single_test_function
- test_preserves_exact_failure_details_with_long_condition_expressions

## Contract Verification Tests
- test_precondition_no_standard_assert_macros_remain
- test_precondition_no_expect_calls_remain
- test_postcondition_all_tests_return_result_type
- test_postcondition_tests_compile_and_execute
- test_invariant_no_panics_in_test_execution

## Given-When-Then Scenarios

### Scenario 1: Replace assert! with assert_chaos!
**Given**: A test function uses `assert!(condition, "message")`
**When**: The assertion is replaced with `assert_chaos!(condition, ChaosTestError::InvariantViolated { details: "message".to_string() })`
**Then**:
- Test compiles successfully
- Test fails with `Err(ChaosTestError::InvariantViolated)` when condition is false
- Test succeeds with `Ok(())` when condition is true
- Error message preserves the original assertion intent

### Scenario 2: Replace assert_eq! with assert_eq_chaos!
**Given**: A test function uses `assert_eq!(left, right, "message")`
**When**: The assertion is replaced with `assert_eq_chaos!(left, right, ChaosTestError::InvariantViolated { details: format!("message: {} != {}", left, right) })`
**Then**:
- Test compiles successfully
- Test fails with `Err(ChaosTestError::InvariantViolated)` when values differ
- Test succeeds with `Ok(())` when values are equal
- Error message includes both values for debugging

### Scenario 3: Replace .expect() with proper error handling
**Given**: A test function uses `some_operation().expect("message")`
**When**: The expect call is replaced with `some_operation().map_err(|e| ChaosTestError::SetupFailed { reason: format!("message: {e}") })?`
**Then**:
- Test compiles successfully
- Test fails with appropriate `ChaosTestError` variant when operation fails
- Error message includes both context and underlying error
- Test succeeds with `Ok(())` when operation succeeds

### Scenario 4: Verify comprehensive test coverage
**Given**: The clock_skew_chaos.rs file has multiple test functions with various assertion types
**When**: All assertions are converted to custom macros
**Then**:
- `test_instant_monotonicity_invariant` uses `assert_chaos!` for monotonicity check
- `test_timeout_duration_accuracy` uses `assert_chaos!` for duration check
- `test_actor_state_consistent_after_clock_skew` uses `assert_eq_chaos!` for state comparisons
- `test_supervisor_healthy_after_clock_skew` uses `assert_eq_chaos!` for status checks
- `test_comprehensive_clock_skew_scenarios` uses `.map_err()?` for error handling
- All tests return `Result<(), ChaosTestError>`

### Scenario 5: Compile and behavior validation
**Given**: The modified test file
**When**: Tests are compiled with `cargo test --all-features --no-run` and executed with the suite's normal test command
**Then**:
- All tests compile successfully
- Test behavior is unchanged from original implementation

## Specific Test Refactorings

### 1. test_instant_monotonicity_invariant (lines 568-590)
**Original**:
```rust
for window in instants.windows(2) {
    assert!(
        window[1] >= window[0],
        "Instant should be monotonic: {:?} >= {:?}",
        window[1],
        window[0]
    );
}
```

**Refactored**:
```rust
for window in instants.windows(2) {
    assert_chaos!(
        window[1] >= window[0],
        ChaosTestError::InvariantViolated {
            details: format!("Instant should be monotonic: {:?} >= {:?}", window[1], window[0])
        }
    );
}
```

### 2. test_timeout_duration_accuracy (lines 615-618)
**Original**:
```rust
assert!(
    timed.elapsed < Duration::from_millis(timeout_ms),
    "Elapsed time should be less than timeout"
);
```

**Refactored**:
```rust
assert_chaos!(
    timed.elapsed < Duration::from_millis(timeout_ms),
    ChaosTestError::InvariantViolated {
        details: "Elapsed time should be less than timeout".to_string()
    }
);
```

### 3. test_actor_state_consistent_after_clock_skew (lines 681-684, 687-690)
**Original**:
```rust
assert_eq!(
    status_before.workflow_id, status_after.workflow_id,
    "Workflow ID should remain consistent"
);
assert_eq!(
    status_before.total_beads, status_after.total_beads,
    "Total bead count should remain consistent"
);
```

**Refactored**:
```rust
assert_eq_chaos!(
    status_before.workflow_id,
    status_after.workflow_id,
    ChaosTestError::InvariantViolated {
        details: format!(
            "Workflow ID should remain consistent: {} != {}",
            status_before.workflow_id, status_after.workflow_id
        )
    }
);
assert_eq_chaos!(
    status_before.total_beads,
    status_after.total_beads,
    ChaosTestError::InvariantViolated {
        details: format!(
            "Total bead count should remain consistent: {} != {}",
            status_before.total_beads, status_after.total_beads
        )
    }
);
```

### 4. test_supervisor_healthy_after_clock_skew (lines 734-738, 752-757)
**Original**:
```rust
assert_eq!(
    ctx.supervisor.get_status(),
    ActorStatus::Running,
    "Supervisor should be running initially"
);
// later in the same scenario
assert_eq!(
    ctx.supervisor.get_status(),
    ActorStatus::Running,
    "Supervisor should remain running after skew: {:?}",
    skew
);
```

**Refactored**:
```rust
assert_eq_chaos!(
    ctx.supervisor.get_status(),
    ActorStatus::Running,
    ChaosTestError::InvariantViolated {
        details: "Supervisor should be running initially".to_string()
    }
);
// later in the same scenario
assert_eq_chaos!(
    ctx.supervisor.get_status(),
    ActorStatus::Running,
    ChaosTestError::InvariantViolated {
        details: format!(
            "Supervisor should remain running after skew: {:?}",
            skew
        )
    }
);
```

### 5. test_comprehensive_clock_skew_scenarios (lines 799, 809, 816)
**Original**:
```rust
simulator.apply().expect("Failed to apply clock skew");
// later in the same scenario
.expect("Failed to execute timed RPC");
// later in the same scenario
.expect("Timeout behavior should be correct");
```

**Refactored**:
```rust
simulator.apply()
    .map_err(|e| ChaosTestError::ClockSkewFailed {
        reason: format!("Failed to apply clock skew: {e}")
    })?;
// later in the same scenario
.map_err(|e| ChaosTestError::RpcFailed {
    reason: format!("Failed to execute timed RPC: {e}")
    })?;
// later in the same scenario
.map_err(|e| ChaosTestError::InvariantViolated {
    details: format!("Timeout behavior should be correct: {e}")
})?;
```

## Exit Criteria
- All standard assertions replaced with custom macros
- All `.expect()` calls replaced with proper error handling
- Tests compile and execute successfully
- Test behavior preserved (same success/failure outcomes)
- Error messages maintain debugging clarity
