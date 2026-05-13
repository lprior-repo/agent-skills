# The Five Mental Lattices

Reference document for the architect skill. Each lattice is a structured thinking tool applied at specific phases of the Double Diamond.

"I think it is undeniably true that the human brain works in models. The trick is to have your brain work better than the other person's brain because it understands the most fundamental models -- ones that do the most work." -- Charlie Munger

---

## Lattice 1: EARS (Easy Approach to Requirements Syntax)

**Applied in:** DISCOVER + DEFINE phases
**Purpose:** Eliminate natural language ambiguity from requirements

EARS forces every requirement into one of six structured patterns. No wiggle room. No "should", "may", "could". Only "SHALL" and "SHALL NOT".

### The Six Patterns

| Pattern | Template | When to Use |
|---------|----------|-------------|
| Ubiquitous | `THE SYSTEM SHALL [behavior]` | Always-true requirements |
| Event-Driven | `WHEN [trigger] THE SYSTEM SHALL [behavior]` | Triggered behaviors |
| State-Driven | `WHILE [state] THE SYSTEM SHALL [behavior]` | State-dependent behaviors |
| Optional | `WHERE [condition] THE SYSTEM SHALL [behavior]` | Conditional features |
| Unwanted | `IF [condition] THEN THE SYSTEM SHALL NOT [behavior]` | Negative requirements |
| Complex | `WHILE [state] WHEN [trigger] THE SYSTEM SHALL [behavior]` | Combined conditions |

### How the Architect Uses EARS

During DISCOVER, the architect converts vague statements into EARS:

**Vague**: "Users should be able to log in"

**EARS conversion**:
- Ubiquitous: `THE SYSTEM SHALL provide a login endpoint that accepts email and password`
- Event-Driven: `WHEN a user submits valid credentials THE SYSTEM SHALL return a JWT token within 200ms`
- State-Driven: `WHILE the user is authenticated THE SYSTEM SHALL include the JWT in all API responses`
- Unwanted: `IF the password has been attempted 5 times incorrectly THEN THE SYSTEM SHALL NOT accept further login attempts for 15 minutes`
- Unwanted: `IF the JWT has expired THEN THE SYSTEM SHALL NOT grant access to protected resources`
- Complex: `WHILE the system is under rate limiting WHEN a user submits valid credentials THE SYSTEM SHALL queue the authentication request and respond within 5 seconds`

### Grilling Questions for EARS

- "You said 'users can do X'. Is that ubiquitous (always) or conditional (sometimes)?"
- "What TRIGGERS this behavior? If nothing, it's ubiquitous. If something, write the WHEN clause."
- "What state must the system be in for this to apply? If any state, it's ubiquitous. If specific, write the WHILE clause."
- "What MUST NOT happen? You've written 5 positive requirements. Write 5 negative ones."
- "You used the word 'should'. Replace it with 'SHALL' or remove the requirement."

### Completeness Check

A feature is EARS-complete when:
- [ ] At least 1 ubiquitous requirement exists
- [ ] All triggered behaviors have WHEN clauses
- [ ] All state-dependent behaviors have WHILE clauses
- [ ] At least 3 unwanted behaviors are specified
- [ ] No requirement contains "should", "may", "could", "might"

---

## Lattice 2: KIRK Contracts (Design by Contract)

**Applied in:** DEFINE + DELIVER phases
**Purpose:** Lock down formal contracts for every component

KIRK (Knowledge-Informed Requirements & Kontract) applies Bertrand Meyer's Design by Contract. Every function, component, and system boundary gets:

### The Three Contract Elements

```
PRECONDITIONS    -- What must be true BEFORE execution
  |
  v
FUNCTION BODY    -- The actual behavior (AI generates this later)
  |
  v
POSTCONDITIONS   -- What must be true AFTER execution
  |
  v
INVARIANTS       -- What must ALWAYS be true
```

### Contract Specification Format

For each component/behavior:

```
Component: [name]
  Preconditions:
    - [condition] -- enforced by: [compile-time type | runtime check | error variant]
    - [condition] -- violation produces: [exact error variant]
  Postconditions:
    - [state change] -- verified by: [assertion | return value | side effect check]
    - [guarantee] -- verified by: [mechanism]
  Invariants:
    - [always-true condition] -- enforced by: [type system | middleware | runtime check]
    - [always-true condition] -- broken during: [never | migration | startup]
  Error Taxonomy:
    - Error::VariantA -- when [condition], produces [HTTP code / error message]
    - Error::VariantB -- when [condition], produces [HTTP code / error message]
```

### Enforcement Hierarchy

Always prefer the strongest enforcement mechanism:

| Level | Mechanism | Example | Strength |
|-------|-----------|---------|----------|
| 1 (strongest) | Compile-time type | `NonZeroU64`, `NonEmptyString` | Impossible to violate |
| 2 | Newtype with validation | `Email::new(s) -> Result<Email, InvalidEmail>` | Caught at construction |
| 3 | Runtime assertion | `debug_assert!(x > 0)` | Caught in development |
| 4 (weakest) | Error variant | `Result<T, Error::PreconditionViolated>` | Caught at call site |

### Grilling Questions for KIRK

- "What are the preconditions for this function? List ALL of them."
- "For each precondition, what enforcement level are you using? Can you make it compile-time?"
- "What postconditions must hold after this operation? What state changed?"
- "What invariants span this entire module? What is ALWAYS true?"
- "You listed the error taxonomy. Give me a concrete violation example for EACH error variant."
- "This takes `&mut self`. What fields does it mutate? Those mutations are postconditions."
- "What happens when precondition X is violated? Show me the exact error type and message."

### Completeness Check

A component is KIRK-complete when:
- [ ] Every function has preconditions defined
- [ ] Every function has postconditions defined
- [ ] Every module has invariants defined
- [ ] Every precondition has an enforcement level assigned
- [ ] Every precondition has a concrete violation example
- [ ] Every violation example maps to an error variant
- [ ] Every `&mut` parameter has mutation postconditions listed
- [ ] Error taxonomy is exhaustive (no unnamed failure modes)

---

## Lattice 3: Inversion Thinking (Failure Analysis)

**Applied in:** DEVELOP phase
**Purpose:** Systematically enumerate everything that can fail

"Invert, always invert." -- Carl Jacobi / Charlie Munger

Instead of asking "what should work?", systematically ask "what could fail?" across three categories.

### Category 1: Security Inversions

| Inversion | Description | Expected Response |
|-----------|-------------|-------------------|
| auth-bypass | Accessing without authentication | 401 Unauthorized |
| expired-token | Using expired credentials | 401 Unauthorized |
| wrong-user-access | Accessing another user's resources | 403 Forbidden |
| privilege-escalation | Admin actions as regular user | 403 Forbidden |
| sql-injection | Malicious query parameters | 400 Bad Request |
| xss-payload | XSS in user-controlled fields | 400 Bad Request |
| rate-limit-exceeded | Too many requests | 429 Too Many Requests |
| csrf-attack | Cross-site request forgery | 403 Forbidden |
| path-traversal | `../../etc/passwd` in file paths | 400 Bad Request |
| mass-assignment | Sending unexpected fields to update | 400 Bad Request |

### Category 2: Usability Inversions

| Inversion | Description | Expected Response |
|-----------|-------------|-------------------|
| not-found | Non-existent resources | 404 Not Found |
| invalid-format | Malformed request data | 400 Bad Request |
| missing-required | Omitted required fields | 400 Bad Request |
| duplicate-create | Creating duplicates | 409 Conflict |
| empty-list | Edge case for empty results | 200 OK (empty) |
| too-large | Payload exceeds limits | 413 Payload Too Large |
| unsupported-media | Wrong content type | 415 Unsupported Media |
| stale-data | Acting on outdated state | 409 Conflict |
| invalid-state-transition | Illegal operation for current state | 422 Unprocessable |
| partial-input | Incomplete multi-step operation | 400 Bad Request |

### Category 3: Integration Inversions

| Inversion | Description | Expected Response |
|-----------|-------------|-------------------|
| idempotency | Retry behavior for duplicate requests | 200 OK (same result) |
| timeout-handling | Long operation timeout | 504 Gateway Timeout |
| version-mismatch | API version compatibility | 400 Bad Request |
| method-not-allowed | Wrong HTTP method | 405 Method Not Allowed |
| partial-failure | Some sub-operations fail | 207 Multi-Status |
| downstream-unavailable | Dependency is down | 503 Service Unavailable |
| concurrent-modification | Race condition | 409 Conflict |
| network-partition | Connection lost mid-operation | Retry with idempotency key |
| clock-skew | Time-sensitive operations with drift | 400 Bad Request |
| data-inconsistency | Eventual consistency window | Documented behavior |

### Grilling Questions for Inversion

- "Walk me through each security inversion. Which ones apply to this feature?"
- "You handled auth-bypass. What about expired tokens? What about token theft?"
- "What happens when the database is down? The cache? The message queue?"
- "This operation takes 30 seconds. User retries after 5. Is it idempotent?"
- "Two users modify the same resource simultaneously. What happens?"
- "The network drops mid-write. What state is the data in?"
- "You have 10 usability inversions. I count 3 you missed. What about [X]?"

### Completeness Check

A feature is Inversion-complete when:
- [ ] All 10 security inversions evaluated (applicable or N/A with reason)
- [ ] All 10 usability inversions evaluated
- [ ] All 10 integration inversions evaluated
- [ ] Each applicable inversion has a defined response code/behavior
- [ ] Each applicable inversion has a test scenario

---

## Lattice 4: Second-Order Thinking (Consequence Tracing)

**Applied in:** DEVELOP phase
**Purpose:** Trace cascade effects beyond the immediate operation

Every action has consequences beyond its immediate effect. First-order thinking asks "what happens?" Second-order thinking asks "and then what?"

### Consequence Tracing Format

For each major behavior:

```
BEHAVIOR: [action name]

FIRST ORDER:
  [Immediate effect]

SECOND ORDER:
  - [Cascade effect 1] -- affects: [component/system]
  - [Cascade effect 2] -- affects: [component/system]
  - [Cascade effect 3] -- affects: [component/system]

THIRD ORDER (if applicable):
  - [Downstream of cascade 1] -- affects: [component/system]

CONSEQUENCE CHECKS:
  - [How to verify cascade 1 was handled correctly]
  - [How to verify cascade 2 was handled correctly]
```

### Common Second-Order Patterns

| Primary Action | Second-Order Effects |
|---------------|---------------------|
| Delete entity | Orphaned references, broken links, audit trail gaps, cache staleness |
| Change schema | Migration requirements, API version bump, client updates, rollback plan |
| Add dependency | Build time increase, supply chain risk, license compatibility, version conflicts |
| Scale horizontally | Session affinity issues, distributed state, cache coherence, cost increase |
| Add caching | Staleness windows, invalidation complexity, memory pressure, debugging difficulty |
| Add encryption | Key management, performance overhead, debugging difficulty, compliance requirements |
| Change auth model | Token migration, session invalidation, client updates, backwards compatibility |

### Grilling Questions for Second-Order

- "You delete a user. What happens to their posts? Comments? Shared files? Active sessions? Pending transactions?"
- "You add caching. What happens when the cache is stale? How do you invalidate? What if invalidation fails?"
- "You change the schema. What happens to existing data? In-flight requests? Cached responses?"
- "This feature adds a new dependency. What's the license? What if it's abandoned? What's the fallback?"
- "You scale to 10 instances. What about distributed locks? Session state? File uploads in progress?"
- "Trace the consequence chain 3 levels deep. First this happens, then that, then what?"

### Completeness Check

A feature is Consequence-complete when:
- [ ] Every major behavior has first and second-order effects listed
- [ ] Each second-order effect identifies the affected component/system
- [ ] Consequence checks are defined for each cascade effect
- [ ] No obvious cascade chains are left unexplored
- [ ] Third-order effects are traced for high-risk operations

---

## Lattice 5: Pre-Mortem Analysis (Risk Prediction)

**Applied in:** DELIVER phase
**Purpose:** Predict failure before it happens

Gary Klein's prospective hindsight: "Imagine the project has failed. Now work backwards to figure out why."

### Pre-Mortem Format

```
PRE-MORTEM: "[Feature name] failed catastrophically after [timeframe]"

LIKELY CAUSES:

1. [Cause description]
   Probability: [HIGH | MEDIUM | LOW]
   Severity: [CRITICAL | HIGH | MEDIUM | LOW]
   Detection: [How you would discover this failure]
   Mitigation: [What prevents or handles this]
   In Scope: [YES | DEFERRED]

2. [Next cause...]
```

### Risk Categories to Explore

| Category | Example Failures |
|----------|-----------------|
| Performance | Latency spike under load, memory leak, connection pool exhaustion |
| Data | Corruption, loss, inconsistency, migration failure |
| Security | Token theft, privilege escalation, data exposure |
| Integration | Dependency failure, API breaking change, version incompatibility |
| Operational | Deployment failure, rollback failure, monitoring blind spot |
| User Experience | Confusing error messages, data loss perception, silent failure |
| Business | Regulatory violation, SLA breach, cost overrun |
| Scale | Unexpected growth, viral load, bot traffic |

### Grilling Questions for Pre-Mortem

- "It's 3am. This feature is broken in production. PagerDuty fires. What went wrong?"
- "A customer lost data because of this feature. How did that happen?"
- "The feature worked fine for 6 months then broke. What changed?"
- "An attacker exploited this feature. What was the vector?"
- "The feature caused a 10x cost increase. Why?"
- "The feature passed all tests but users hate it. What did we miss?"
- "Deployment of this feature failed and rolled back. What went wrong with the rollback?"
- "You listed 5 risks. I want 10. What are the other 5?"

### Completeness Check

A feature is PreMortem-complete when:
- [ ] At least 5 failure causes identified
- [ ] Each cause has probability and severity rated
- [ ] Each cause has a detection mechanism
- [ ] Each cause has a mitigation strategy
- [ ] At least one cause from each of: performance, security, data, integration
- [ ] High-probability causes have in-scope mitigations
- [ ] Deferred mitigations are documented as open risks

---

## Quality Scoring (KIRK 5 Dimensions)

After all lattices are applied, score the specification:

| Dimension | Weight | Target | How to Measure |
|-----------|--------|--------|----------------|
| Completeness | 20% | 100% | All sections filled / total sections |
| Consistency | 20% | 100% | Zero conflicting requirements |
| Testability | 25% | 100% | Behaviors with acceptance criteria / total behaviors |
| Clarity | 15% | 100% | Requirements with "why" / total requirements |
| Security | 20% | 80%+ | Security inversions addressed / total applicable |

**Overall target: 90%+**

A spec scoring below 90% is not ready for planner handoff. Identify the gaps and re-enter the appropriate lattice.
