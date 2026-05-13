# Architecture Specification Template

This is the output template for the architect skill. When the Double Diamond process completes, the architect produces this document. It becomes the input to the planner skill for bead decomposition.

Copy this template and fill every section. Sections marked (REQUIRED) cannot be empty. Sections marked (IF APPLICABLE) can be marked N/A with a reason.

---

```markdown
# Architecture Specification: [Feature/System Name]

## Meta
- **Date:** [YYYY-MM-DD]
- **Author:** Architect Skill v2.0.0
- **Quality Score:** [Overall]% (Completeness: [X]%, Consistency: [X]%, Testability: [X]%, Clarity: [X]%, Security: [X]%)
- **Status:** [Ready for Planner | Needs More Discovery | Blocked]
- **Scope Level:** [Component | Module | System | Product]

---

## 1. Problem Statement (REQUIRED)

[Single paragraph. Specific. Falsifiable. No weasel words.]

### 1.1 Context
- **Who:** [Specific user personas, not "everyone"]
- **What:** [The problem they face today]
- **Evidence:** [Data, user feedback, metrics that prove this problem exists]
- **Impact:** [Quantified impact of the problem]

### 1.2 Scope

**IN scope:**
- [Explicit list of what this feature DOES]
- [...]

**OUT of scope:**
- [Explicit list of what this feature does NOT do]
- [...]

**DEFERRED:**
- [What might be added later, with trigger conditions]
- [...]

---

## 2. EARS Requirements (REQUIRED)

### 2.1 Ubiquitous
- THE SYSTEM SHALL [behavior]
- THE SYSTEM SHALL [behavior]

### 2.2 Event-Driven
- WHEN [trigger] THE SYSTEM SHALL [behavior]
- WHEN [trigger] THE SYSTEM SHALL [behavior]

### 2.3 State-Driven
- WHILE [state] THE SYSTEM SHALL [behavior]
- WHILE [state] THE SYSTEM SHALL [behavior]

### 2.4 Optional (IF APPLICABLE)
- WHERE [condition] THE SYSTEM SHALL [behavior]

### 2.5 Unwanted (REQUIRED -- minimum 3)
- IF [condition] THEN THE SYSTEM SHALL NOT [behavior]
- IF [condition] THEN THE SYSTEM SHALL NOT [behavior]
- IF [condition] THEN THE SYSTEM SHALL NOT [behavior]

### 2.6 Complex (IF APPLICABLE)
- WHILE [state] WHEN [trigger] THE SYSTEM SHALL [behavior]

---

## 3. Domain Model (REQUIRED)

### 3.1 Entities
| Entity | Key Fields | Relationships |
|--------|-----------|---------------|
| [Name] | [field: Type, ...] | [belongs_to X, has_many Y] |

### 3.2 Value Objects
| Value Object | Fields | Validation Rules |
|-------------|--------|-----------------|
| [Name] | [field: Type, ...] | [constraints] |

### 3.3 States and Transitions
```
[State Machine Diagram -- ASCII art or description]

States: [S1, S2, S3, ...]

Legal Transitions:
  S1 -> S2: [trigger / condition]
  S2 -> S3: [trigger / condition]

ILLEGAL Transitions (and how prevented):
  S3 -> S1: [why illegal] -- prevented by [type system / runtime check]
  S1 -> S3: [why illegal] -- prevented by [type system / runtime check]
```

### 3.4 Illegal States
| Illegal State | Why Illegal | Prevention Mechanism |
|--------------|-------------|---------------------|
| [description] | [reason] | [compile-time type / runtime check / invariant] |

### 3.5 Domain Events
| Event | Trigger | Payload | Consumers |
|-------|---------|---------|-----------|
| [Name] | [What causes it] | [Data included] | [Who listens] |

---

## 4. KIRK Contracts (REQUIRED)

### Component: [Name]

**Preconditions:**
| # | Condition | Enforcement | Violation Error |
|---|-----------|-------------|-----------------|
| P1 | [condition] | [compile-time / runtime / error variant] | [Error::Variant] |
| P2 | [condition] | [compile-time / runtime / error variant] | [Error::Variant] |

**Postconditions:**
| # | Guarantee | Verification |
|---|-----------|-------------|
| Q1 | [what must be true after] | [how verified] |
| Q2 | [what must be true after] | [how verified] |

**Invariants:**
| # | Condition | Enforcement | Broken During |
|---|-----------|-------------|---------------|
| I1 | [always true] | [type system / middleware / check] | [never / migration / startup] |
| I2 | [always true] | [type system / middleware / check] | [never / migration / startup] |

**Violation Examples:**
- VIOLATES P1: `[concrete call]` -- produces `Err([Error::Variant])`
- VIOLATES P2: `[concrete call]` -- produces `Err([Error::Variant])`

[Repeat for each component]

---

## 5. Error Taxonomy (REQUIRED)

### 5.1 Error Variants
| Variant | When | HTTP Code | User Message | Internal Log |
|---------|------|-----------|-------------|-------------|
| Error::X | [condition] | [4xx/5xx] | [what user sees] | [what ops sees] |
| Error::Y | [condition] | [4xx/5xx] | [what user sees] | [what ops sees] |

### 5.2 Error Hierarchy (IF APPLICABLE)
```
AppError
  +-- AuthError
  |     +-- InvalidCredentials
  |     +-- TokenExpired
  |     +-- InsufficientPermissions
  +-- ValidationError
  |     +-- MissingField(field_name)
  |     +-- InvalidFormat(field_name, expected)
  +-- ResourceError
  |     +-- NotFound(resource_type, id)
  |     +-- Conflict(resource_type, reason)
  +-- InfraError
        +-- DatabaseUnavailable
        +-- ExternalServiceTimeout(service_name)
```

---

## 6. Inversion Analysis (REQUIRED)

### 6.1 Security Inversions
| Inversion | Applicable? | Trigger | Response | Test Scenario |
|-----------|------------|---------|----------|---------------|
| auth-bypass | [Y/N] | [how] | [code + behavior] | [test name] |
| expired-token | [Y/N] | [how] | [code + behavior] | [test name] |
| wrong-user-access | [Y/N] | [how] | [code + behavior] | [test name] |
| privilege-escalation | [Y/N] | [how] | [code + behavior] | [test name] |
| injection | [Y/N] | [how] | [code + behavior] | [test name] |
| xss-payload | [Y/N] | [how] | [code + behavior] | [test name] |
| rate-limit | [Y/N] | [how] | [code + behavior] | [test name] |

### 6.2 Usability Inversions
| Inversion | Applicable? | Trigger | Response | Test Scenario |
|-----------|------------|---------|----------|---------------|
| not-found | [Y/N] | [how] | [code + behavior] | [test name] |
| invalid-format | [Y/N] | [how] | [code + behavior] | [test name] |
| missing-required | [Y/N] | [how] | [code + behavior] | [test name] |
| duplicate | [Y/N] | [how] | [code + behavior] | [test name] |
| empty-result | [Y/N] | [how] | [code + behavior] | [test name] |
| stale-data | [Y/N] | [how] | [code + behavior] | [test name] |
| invalid-transition | [Y/N] | [how] | [code + behavior] | [test name] |

### 6.3 Integration Inversions
| Inversion | Applicable? | Trigger | Response | Test Scenario |
|-----------|------------|---------|----------|---------------|
| idempotency | [Y/N] | [how] | [code + behavior] | [test name] |
| timeout | [Y/N] | [how] | [code + behavior] | [test name] |
| concurrent-modification | [Y/N] | [how] | [code + behavior] | [test name] |
| partial-failure | [Y/N] | [how] | [code + behavior] | [test name] |
| downstream-unavailable | [Y/N] | [how] | [code + behavior] | [test name] |

---

## 7. Second-Order Consequences (REQUIRED for major behaviors)

### Behavior: [Name]

**First Order:** [Immediate effect]

**Second Order:**
| # | Cascade Effect | Affected Component | Consequence Check |
|---|---------------|-------------------|-------------------|
| 1 | [effect] | [component] | [how to verify it's handled] |
| 2 | [effect] | [component] | [how to verify it's handled] |

**Third Order (if high-risk):**
| # | Cascade Effect | Source | Affected Component |
|---|---------------|--------|-------------------|
| 1 | [downstream effect] | [from 2nd-order #X] | [component] |

[Repeat for each major behavior]

---

## 8. Pre-Mortem (REQUIRED)

**Scenario:** "[Feature name] failed catastrophically after [timeframe]"

| # | Cause | Probability | Severity | Detection | Mitigation | In Scope? |
|---|-------|------------|----------|-----------|------------|-----------|
| 1 | [cause] | HIGH/MED/LOW | CRIT/HIGH/MED/LOW | [how detected] | [strategy] | Y/N |
| 2 | [cause] | HIGH/MED/LOW | CRIT/HIGH/MED/LOW | [how detected] | [strategy] | Y/N |
| 3 | [cause] | HIGH/MED/LOW | CRIT/HIGH/MED/LOW | [how detected] | [strategy] | Y/N |
| 4 | [cause] | HIGH/MED/LOW | CRIT/HIGH/MED/LOW | [how detected] | [strategy] | Y/N |
| 5 | [cause] | HIGH/MED/LOW | CRIT/HIGH/MED/LOW | [how detected] | [strategy] | Y/N |

---

## 9. Architecture Decision (REQUIRED)

### 9.1 Chosen Approach
**Approach:** [Name/description]
**Rationale:** [Why this approach over alternatives]

### 9.2 Rejected Alternatives
| Alternative | Pros | Cons | Rejection Reason |
|------------|------|------|-----------------|
| [Approach B] | [pros] | [cons] | [why rejected] |
| [Approach C] | [pros] | [cons] | [why rejected] |

### 9.3 Key Design Decisions
| Decision | Choice | Rationale | Trade-off Accepted |
|----------|--------|-----------|-------------------|
| [What] | [Chosen option] | [Why] | [What we give up] |

---

## 10. Acceptance Criteria (REQUIRED)

### 10.1 Happy Path
| # | Scenario | Given | When | Then | Why |
|---|----------|-------|------|------|-----|
| 1 | [name] | [precondition] | [action] | [expected result] | [why this matters] |

### 10.2 Error Path
| # | Scenario | Given | When | Then | Why |
|---|----------|-------|------|------|-----|
| 1 | [name] | [precondition] | [action] | [expected error] | [why this matters] |

### 10.3 Edge Cases
| # | Scenario | Given | When | Then | Why |
|---|----------|-------|------|------|-----|
| 1 | [name] | [precondition] | [action] | [expected behavior] | [why this matters] |

---

## 11. Non-Functional Requirements (REQUIRED)

### 11.1 Performance
| Metric | Target | Measurement |
|--------|--------|-------------|
| [Latency / Throughput / etc.] | [target value] | [how measured] |

### 11.2 Security
| Requirement | Implementation | Verification |
|------------|----------------|-------------|
| [requirement] | [how implemented] | [how verified] |

### 11.3 Observability
| Signal | Type | Purpose | Alert Threshold |
|--------|------|---------|----------------|
| [metric/log/trace] | [metric/log/trace] | [what it tells you] | [when to alert] |

### 11.4 Scalability (IF APPLICABLE)
| Dimension | Current | Target | Strategy |
|-----------|---------|--------|----------|
| [users/requests/data] | [current] | [target] | [how to scale] |

---

## 12. Open Risks (REQUIRED -- even if empty)

| # | Risk | Source | Severity | Status | Revisit Trigger |
|---|------|--------|----------|--------|----------------|
| 1 | [risk] | [which lattice/phase] | HIGH/MED/LOW | OPEN/DEFERRED | [when to revisit] |

---

## 13. Interview Matrix Completion (REQUIRED)

```
             USER  DEV   OPS   SEC   BIZ
CORE INTENT  [x]   [x]   [x]   [x]   [x]
ERROR CASES  [x]   [x]   [x]   [x]   [x]
EDGE CASES   [x]   [x]   [x]   [x]   [x]
SECURITY     [x]   [x]   [x]   [x]   [x]
OPERATIONS   [x]   [x]   [x]   [x]   [x]
```

Deferred cells: [list any deferred cells with reasons]

---

## 14. Assumptions Log (REQUIRED)

| # | Assumption | Confidence | Impact if Wrong | Validation Plan |
|---|-----------|-----------|-----------------|----------------|
| 1 | [assumption] | HIGH/MED/LOW | [what breaks] | [how to verify] |

---

## 15. Glossary (IF APPLICABLE)

| Term | Definition | Context |
|------|-----------|---------|
| [term] | [definition] | [where used] |

---

## 16. Handoff Notes

**Recommendation:** [Ready for Planner | Needs More Discovery on X | Blocked on Y]

**For the Planner:**
- Suggested decomposition boundaries: [where to split into beads]
- Dependency ordering: [what must be built first]
- Parallel work opportunities: [what can be built simultaneously]
- Estimated total effort: [rough range]

**For the Quality Engineer:**
- Key test scenarios: [most important tests]
- Risk areas requiring extra coverage: [where to focus testing]
- Performance test requirements: [load/stress test parameters]
```

---

## Template Usage Notes

1. Every REQUIRED section must be filled. Empty REQUIRED sections mean the spec is incomplete.
2. IF APPLICABLE sections can be marked `N/A: [reason]`.
3. The quality score in the Meta section must reflect actual content, not aspirational targets.
4. Tables should have real content, not placeholder text. If you don't know, write "TBD: [what's needed]".
5. This document should be self-contained. A reader should not need to ask questions.
6. The Handoff Notes section is specifically for the planner skill's consumption.
