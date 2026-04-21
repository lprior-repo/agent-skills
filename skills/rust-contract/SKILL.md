---
name: rust-contract
description: "Design-by-contract and Martin Fowler test planning for Rust. Produces contracts, invariants, and Given-When-Then plans."
---

# Rust Contract (Design by Contract + Martin Fowler Tests)

This skill generates **contract-first specifications** and **Martin Fowler style test plans** for Rust work. It does **not** implement code.

```jsonl
{"kind":"meta","skill":"rust-contract","version":"1.0.0","updated":"2026-02","format":"markdown-with-embedded-jsonl"}
{"kind":"principle","id":"contract_first","text":"Define preconditions, postconditions, invariants, and error taxonomy before any implementation."}
{"kind":"principle","id":"fowler_tests","text":"Tests are executable specifications: expressive names, Given-When-Then, happy/error/edge coverage."}
{"kind":"principle","id":"no_implementation","text":"Do not write production code. Output only contracts and test plans."}
{"kind":"principle","id":"railway_oriented","text":"All fallible operations must be expressed as Result<T, Error> in the contract signatures."}
```

## Inputs

- Bead ID or feature description
- Any existing constraints, APIs, or domain language

If information is missing, list **open questions** and **assumptions** explicitly.

## Outputs

Produce two artifacts:

1) `contract-spec.md` - Design by contract specification
2) `martin-fowler-tests.md` - Test plan with Given-When-Then scenarios

## Workflow

### Step 1: Gather Context

- Read relevant docs or bead description
- Identify domain terms and constraints
- List open questions (if any)

### Step 2: Design by Contract

Define the contract **before** tests:

- Preconditions (what must be true before)
- Postconditions (what must be true after)
- Invariants (what is always true)
- Error taxonomy (exhaustive, semantic error variants)
- Function signatures (Result<T, Error> for all fallible ops)

### Step 3: Martin Fowler Test Plan

Create test cases that fully specify behavior:

- Happy path tests (expressive names)
- Error path tests (each failure mode)
- Edge case tests (boundaries, empty, extremes)
- Contract verification tests (pre/post/invariants)
- At least one end-to-end scenario (if applicable)

### Step 4: Exit Criteria

Only finalize if:

- Every failure mode has a corresponding error variant
- Every pre/post/invariant has at least one test
- Test names describe behavior unambiguously

## Output Templates

### contract-spec.md

```markdown
# Contract Specification

## Context
- Feature:
- Domain terms:
- Assumptions:
- Open questions:

## Preconditions
- [ ]

## Postconditions
- [ ]

## Invariants
- [ ]

## Error Taxonomy
- Error::InvalidInput - when ...
- Error::NotFound - when ...
- Error::PreconditionViolation - when ...

## Contract Signatures
- fn ... -> Result<..., Error>

## Non-goals
- [ ]
```

### martin-fowler-tests.md

```markdown
# Martin Fowler Test Plan

## Happy Path Tests
- test_returns_success_when_valid_input_provided
- test_creates_resource_when_preconditions_met

## Error Path Tests
- test_returns_error_when_invalid_input
- test_returns_error_when_resource_not_found

## Edge Case Tests
- test_handles_empty_input_gracefully
- test_handles_boundary_values_correctly

## Contract Verification Tests
- test_precondition_<name>
- test_postcondition_<name>
- test_invariant_<name>

## Given-When-Then Scenarios
### Scenario 1: <name>
Given: ...
When: ...
Then:
- ...
```

## Notes

- Do not implement code in this skill.
- Use ASCII only unless the repo already uses non-ASCII.
- Keep outputs precise, testable, and unambiguous.
