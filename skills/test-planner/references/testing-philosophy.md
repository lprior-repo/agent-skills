# Testing Philosophy Reference

## The Authorities

### Martin Fowler — Behavior-Driven Contracts
- Test behaviors (guarantees), not methods (implementations)
- Tests survive refactoring: if internal structure changes but behavior doesn't, NO test should break
- Test via public APIs only — never peer into private state
- Tests are executable documentation of the system's contract
- "Test state, not interactions" — verify WHAT the system produces, not HOW it arrived there

### Dave Farley — ATDD and Design Through Tests
- Tests drive API design. Difficulty writing tests signals bad design.
- Strict separation: WHAT (the test's intent/assertion) vs HOW (the SUT's implementation)
- Acceptance tests are the highest fidelity specification
- Every test tells a story: setup → action → assertion (no mixing)
- Small, incremental, always-green: the 15-minute TDD cycle

### Dan North — BDD and Living Specifications
- "Given-When-Then" is not syntax; it is a thinking tool
- Test names are sentences describing system behavior: `should_reject_transfer_when_balance_insufficient`
- Feature files and scenarios are the boundary between business and engineering
- The scenario is the specification; the code is the implementation of it

### Kent Beck — TDD Fundamentals
- Red-Green-Refactor: no production code without a failing test
- One logical assertion per test — focused, atomic
- Fast (<1ms for unit), isolated (no shared state), deterministic (same result every time)
- DAMP over DRY: "Descriptive And Meaningful Phrases" — a little duplication is OK in tests if it
  makes each test self-contained and readable. Tests don't need tests of their own.

### Testing Trophy (Hodges / Kent C. Dodds / Justin Searls)
vs the classic Test Pyramid (Mike Cohn):

```
Classic Pyramid:          Testing Trophy:
    /E2E\                     /E2E\
   /-----\                   /-----\
  /Integr.\    →        /Integration \   ← widest layer
 /----------\          /--------------\
/  Unit      \        /  Unit (Calc)   \
--------------        /------------------\
                      / Static Analysis   \
```

The Trophy recognizes that:
- Integration tests provide the best ROI: they test real interactions without full E2E cost
- Unit tests are still critical for Calc (pure logic) layer — exhaustive combinatorial coverage
- Static analysis (clippy, cargo-deny, type checker) catches classes of bugs for free
- E2E tests validate user journeys but are slow and fragile — use sparingly

### Google SWE Book — Engineering at Scale

**Prefer Real Implementations** (Ch. 13)
Prefer classical testing (real objects) over mockist testing. Real implementations:
- Have higher fidelity — bugs in dependencies surface
- Don't go stale the way mocks do
- Don't require understanding the dependency's internals
Use a test double only when: the real implementation is too slow, nondeterministic, or
has problematic external side effects (network, billing, etc.)

**Fake > Stub > Mock** (Ch. 13)
1. Fake: lightweight but realistic implementation (in-memory DB, local filesystem). Best.
2. Stub: hardcoded return values. OK for triggering specific code paths.
3. Mock: interaction verification ("was this method called?"). Leads to brittle tests.
   Use only for state-changing calls to external systems (sendEmail, saveRecord).
   Never use for non-state-changing queries.

**DAMP > DRY** (Ch. 12)
Tests should be "Descriptive And Meaningful Phrases." A test body should contain all
information needed to understand it — no chasing setup methods.
- Shared test infrastructure: OK for constructing objects and fakes
- Shared assertions: only if they assert a single conceptual fact
- Shared fixtures: OK if tests don't depend on the specific values

**Test Size vs Scope** (Ch. 11)
Size = resources consumed. Scope = code being validated.
- Small: single process, no I/O, no network, no sleep → fastest, deterministic
- Medium: single machine, can hit localhost → integration sweet spot
- Large: multi-machine, network → E2E territory

**The Beyoncé Rule**: If you care about a behavior, put a test on it. Every property
you want to preserve must have an automated test. Otherwise, it will eventually break.

**Hermetic SUTs**: Tests should contain all information needed to set up, run, and
tear down their environment. Tests must not depend on external state or test ordering.

---

## Rust Testing Ecosystem Reference

| Tool | Purpose | Layer |
|------|---------|-------|
| `#[test]` + nextest | Unit + integration runner | All |
| `tdd-guard-rust` | TDD enforcement gate | All |
| `proptest` | Property-based testing | Unit/Calc |
| `cargo-fuzz` (libFuzzer) | Fuzzing parsers/deserializers | Unit |
| `kani` | Formal model checking | Critical invariants |
| `cargo-mutants` | Mutation testing | Coverage quality |
| `insta` | Snapshot testing for complex outputs | Unit/Integration |
| `mockall` | Mocking external I/O (use sparingly) | Integration |
| `criterion` | Benchmarking performance-critical paths | Performance |
| `cucumber` | BDD acceptance scenarios | E2E/Acceptance |
| `cargo-llvm-cov` | Line/branch coverage metrics | All |

## Anti-Patterns to Reject

- `result.is_ok()` without asserting the value inside → **REJECT**
- Mocking the database when an in-memory fake exists → **REJECT**
- Test that passes when the implementation is deleted → **REJECT** (mutation survivor)
- Test named `test_foo()` instead of `foo_returns_x_when_y()` → **REJECT**
- Interaction test for a query (non-state-changing) function → **REJECT**
- Single test covering multiple behaviors → **REJECT** (split it)
- Logic (loops, conditionals) inside test bodies → **REJECT**
- `sleep()` inside tests → **REJECT** (use event polling or channels)
- Brittle test that breaks on refactoring internal structure → **REJECT**
