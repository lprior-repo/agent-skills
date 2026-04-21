---
name: velocity
description: "Raw throughput via good coding discipline. High-velocity outcome-driven development with TDD, functional core, Modern SWE principles, and CI/CD automation."
---

# Skill: Velocity

**Raw throughput through good coding discipline.**

Velocity is the only metric. Ship incredibly fast through small consistent changes with TDD, functional architecture, and automated validation.

## Core Philosophy

**Code is not an asset. It is a disposable traverse.** We define a destination (Output/Contract) and burn compute to find a path to it. Once the data arrives, the path is irrelevant.

Based on Rich Sutton's "The Bitter Lesson": General methods that leverage massive computation ultimately crush human-designed, domain-specific cleverness.

## Three Pillars of Velocity

### 1. Outcome-Driven Development (The Bitter Truth)
- **Velocity is king**: One-piece flow by default
- **Small slice → validate → next slice**
- **If time-to-green slows, slice is too big**
- **Inventory is waste**: unvalidated code is liability

### 2. Engineering Rigor (Modern SWE)
- **TDD-First Loop**: Test-driven development cycle
- **Functional Architecture**: Pure core, impure shell
- **Hard Constraints**: ≤25 lines per function, ≤5 parameters
- **Commit Frequently**: Small, atomic commits with clear messages

### 3. Continuous Flow (CI/CD)
- **Slice then gate**: After every slice, run validation
- **Small batch caps**: Validate every 15 minutes or 100 LOC
- **Moon gates**: `moon run :ci` is the only truth
- **jj workflow**: Use jj for VCS porcelain

## Quick Reference

### Hard Constraints (Non-Negotiable)
- **≤25 lines per function**
- **≤5 parameters per function** (use objects/structs if exceeded)
- **Zero implementation without failing test first**
- **One behavior per test**
- **One concept per commit**

### TDD Core Loop
1. Declare constraints and invariants upfront
2. Write failing test that specifies desired behavior
3. Predict how the test will fail before running
4. Write minimal implementation to pass (halt if >25 lines)
5. Run skeptical validation
6. Refactor for clarity while tests stay green

### CI/CD Flow
```bash
# Tight loop (velocity defaults)
jj diff
moon run :ci
# if :ci fails unrelated
moon run :<crate>:test || cargo test -p <crate>
moon run :<crate>:lint || cargo clippy -p <crate> -- -D warnings
moon run :<crate>:fmt || cargo fmt --check
```

## Functional Core / Imperative Shell

**Mandatory architecture pattern.** All systems MUST separate pure logic from side effects.

```
┌─────────────────────────────────────┐
│      IMPERATIVE SHELL               │
│      (I/O, side effects)            │
│                                     │
│  ┌───────────────────────────────┐  │
│  │   FUNCTIONAL CORE             │  │
│  │   (Pure logic, no I/O)        │  │
│  │                               │  │
│  │   - Business rules            │  │
│  │   - Transformations           │  │
│  │   - Validations               │  │
│  │   - Calculations              │  │
│  └───────────────────────────────┘  │
│                                     │
└─────────────────────────────────────┘
```

**Data flow:**
1. Shell reads data (impure I/O)
2. Core transforms data (pure logic)
3. Shell writes result (impure I/O)

## Six Mandatory Workflow Gates

**GATE-1: Acceptance Test Exists**
- [ ] RED acceptance test defines WHAT (not HOW)
- [ ] Uses domain language only
- [ ] Expresses user-visible behavior

**GATE-2: Unit Test RED**
- [ ] Failing unit test for current step
- [ ] Single assertion
- [ ] Tests behavior, not implementation

**GATE-3: Function Purity**
- [ ] Core functions are pure (no I/O)
- [ ] Shell contains all side effects
- [ ] Same input → same output

**GATE-4: Function Size**
- [ ] ≤25 lines per function
- [ ] Extract if exceeded
- [ ] Single responsibility

**GATE-5: GREEN Before Refactor**
- [ ] All tests passing
- [ ] No RED tests
- [ ] Safe to refactor

**GATE-6: Commit Frequently**
- [ ] Small, atomic commits
- [ ] Clear commit messages
- [ ] Commit when GREEN
- [ ] Fix RED before moving on

## CI/CD Best Practices

### Slice Then Gate
- After every slice, run `moon run :ci` before expanding scope
- Transaction cost is validation. Drive it toward zero by keeping slices tiny.

### Small Batch Caps
- Soft cap: avoid >100 LOC OR >15 minutes of work without a green `moon run :ci`
- Continuous flow ratio drifts above 1 when you batch. Shrink the slice.

### Validation Over Trust
- Only green moon gates count as truth. No green, no claims.
- If `moon run :ci` fails unrelated, validate your changed crate with targeted commands.

### JJ Workflow
- Use `jj` only for VCS porcelain (status, diff, log, commit)
- When using zjj workspaces, validate and diff inside the workspace
- Sync early/often to keep slices clean

## Anti-Patterns (FORBIDDEN)

### Code Anti-Patterns
- ❌ Functions >25 lines
- ❌ Functions with >5 parameters
- ❌ Implementation before test
- ❌ Core functions with I/O
- ❌ Global mutable state
- ❌ Persisting broken code

### Workflow Anti-Patterns
- ❌ "I'll write tests later"
- ❌ Debugging without understanding
- ❌ Refactoring on RED
- ❌ Skipping gates
- ❌ Large commits with multiple concepts
- ❌ Batching multiple slices before gates

### CI/CD Anti-Patterns
- ❌ Continuing while gates are red
- ❌ 200+ LOC without validation
- ❌ 30+ minutes without gates
- ❌ Expanding lint work beyond your changes

## When to Use This Skill

Invoke Velocity when:
- Writing new functions, modules, or services
- Refactoring existing code
- Designing system architecture
- Implementing features
- Running CI/CD pipelines
- Optimizing for throughput

## Success Metrics

You know you're doing it right when:
- ✅ Every commit is GREEN
- ✅ Functions are ≤25 lines
- ✅ Core is pure (no I/O)
- ✅ Tests express WHAT, not HOW
- ✅ Moon gates pass within minutes
- ✅ JJ diff stays tiny
- ✅ Flow through the system, not optimization of the system itself

## Trigger Language

Activate when:
- User mentions velocity, throughput, shipping, deployment, CI/CD
- Writing or refactoring code
- Running tests or validation
- TDD, test-driven development
- Functional programming, purity
- Continuous integration, deployment

```jsonl
{"kind":"meta","skill":"velocity","version":"1.0.0","format":"markdown-with-embedded-jsonl","mode":"contract-first"}
{"kind":"input","arguments":"$ARGUMENTS","rule":"Infer target from current request context. Trigger on velocity, throughput, tdd, testing, deployment, ci/cd, functional programming, purity."}
{"kind":"mission","goal":"Maximize raw throughput through good coding discipline: TDD, functional architecture, and automated validation."}
{"kind":"rule","id":"velocity_is_king","text":"VELOCITY IS KING: one-piece flow by default. Small slice -> moon run :ci -> next slice. If time-to-green slows, slice is too big."}
{"kind":"rule","id":"validation_over_trust","text":"VALIDATION OVER TRUST: only green moon gates count as truth. No green, no claims."}
{"kind":"rule","id":"inventory_is_waste","text":"Inventory is waste: unvalidated jj diff is liability. Keep edit->green close to zero."}
{"kind":"rule","id":"function_size","text":"≤25 lines per function. Extract if exceeded."}
{"kind":"rule","id":"parameter_count","text":"≤5 parameters per function. Use objects/structs if exceeded."}
{"kind":"rule","id":"test_first","text":"Zero implementation without failing test first."}
{"kind":"rule","id":"functional_core","text":"Core functions must be pure (no I/O). Shell contains all side effects."}
{"kind":"rule","id":"slice_then_gate","text":"After every slice, run moon run :ci before expanding scope."}
{"kind":"rule","id":"small_batch","text":"Validate every 15 minutes or 100 LOC. Avoid 200+ LOC or 30+ minutes without gates."}
{"kind":"workflow","id":"tdd_loop","steps":["Write failing test","Predict failure","Write minimal implementation","Run validation","Refactor if green"]}
{"kind":"workflow","id":"ci_loop","steps":["Write code slice","moon run :ci","Fix if red","Commit if green","Next slice"]}
{"kind":"gate","id":"moon_ci","commands":["moon run :ci"],"notes":["Green is the only signal. If fails unrelated, validate specific crate."]}
{"kind":"ref","file":"/home/lewis/references/atdd-four-layer.md","use":"Complete ATDD specification"}
{"kind":"ref","file":"/home/lewis/references/design-constraints.md","use":"Functional core / imperative shell rules"}
{"kind":"ref","file":"/home/lewis/references/workflow-gates.md","use":"Six mandatory checkpoints"}
```

## Philosophy

This skill embodies the fusion of:
1. **Bitter Truth**: High-velocity outcome-driven development
2. **Coding Rigor**: Dave Farley's Modern Software Engineering
3. **Continuous Deployment**: Lean one-piece-flow, small-batch CI/CD

**The goal:** Flow through the system, not optimization of the system itself.

Every gate is a quality checkpoint. Every commit is a proven step forward.
