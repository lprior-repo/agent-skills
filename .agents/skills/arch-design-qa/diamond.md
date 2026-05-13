# Double Diamond: Phases, Gates, and State Transitions

Reference document for the architect skill. The Double Diamond is a design thinking framework adapted for architectural specification. Two diamonds: one for the PROBLEM space, one for the SOLUTION space.

---

## Visual Overview

```
         DIAMOND 1: PROBLEM              DIAMOND 2: SOLUTION
         
       .    DISCOVER    .              .    DEVELOP    .
      / \  (diverge)   / \            / \  (diverge)   / \
     /   \            /   \          /   \            /   \
    /     \          /     \        /     \          /     \
   /       \        /       \      /       \        /       \
  /    ?    \      /    !    \    /    ?    \      /    !    \
 /           \    /           \  /           \    /           \
/             \  /             \/             \  /             \
    explore       DEFINE            explore       DELIVER
    broadly       (converge)        broadly       (converge)
                  lock down                       lock down

INTAKE ---> DISCOVER ---> DEFINE ---> DEVELOP ---> DELIVER ---> HANDOFF
             EARS +        KIRK       Inversion    Pre-Mortem    Emit
             Interview     Contracts  + 2nd-Order  + Quality     Spec
             Matrix                   Thinking     Score
```

---

## Phase 0: INTAKE

**Duration:** 5-10 minutes
**Lattice:** None (context gathering)
**Interview Matrix:** None yet

### Actions

1. Read the user's feature description or system requirement
2. If a codebase is available, explore it:
   - `Glob` for relevant files and modules
   - `Grep` for domain terms, existing patterns, related code
   - `Read` key files for architectural context
3. Restate the request in your own words -- get confirmation
4. List what you DO know and what you DON'T know
5. Identify the scope of the feature (is this a component, a module, a system, a product?)

### Gate: INTAKE -> DISCOVER

- [ ] User's request is restated and confirmed
- [ ] Codebase context gathered (if available)
- [ ] Initial unknowns identified
- [ ] Scope level identified (component / module / system / product)

**Announcement:** "Entering Diamond 1: Problem Space -- DISCOVER phase"

---

## Phase 1: DISCOVER (Diamond 1 -- Diverge)

**Duration:** 15-30 minutes (interactive)
**Lattice:** EARS (requirements syntax)
**Interview Matrix:** Rows 1-2 (CORE INTENT + ERROR CASES) across all 5 perspectives

### Purpose

Explore the problem space BROADLY. Do not converge yet. Ask many questions. Surface assumptions. Challenge the stated problem.

### Actions

1. **Interview Matrix -- CORE INTENT row:**
   - USER: What problem does this solve? For whom? Evidence?
   - DEVELOPER: What components are involved? Technical surface area?
   - OPS: What scale? What infrastructure?
   - SECURITY: What sensitive data? Trust boundaries?
   - BUSINESS: What metrics? Cost of inaction?

2. **Interview Matrix -- ERROR CASES row:**
   - USER: What frustrates? What confusing states?
   - DEVELOPER: What breaks? Full error list?
   - OPS: What alarms? Degradation signals?
   - SECURITY: What leaks in error paths?
   - BUSINESS: What does failure cost?

3. **Begin EARS drafting:**
   - Convert vague requirements into EARS syntax
   - Challenge: "Is this really ubiquitous or conditional?"
   - Force unwanted requirements: "What must NOT happen?"
   - Identify event triggers: "What TRIGGERS this behavior?"

4. **Surface assumptions:**
   - List every assumption explicitly
   - Rate confidence: HIGH / MEDIUM / LOW
   - Flag LOW-confidence assumptions for investigation

### Grilling in DISCOVER

- "You said 'users need X'. Which users? How do you know? What's the evidence?"
- "You're solving problem Y. What if Y isn't actually the problem? What if it's Z?"
- "You said this is 'simple'. Walk me through the happy path step by step. Now the unhappy path."
- "What are you assuming about the infrastructure? The data? The users? List every assumption."

### Gate: DISCOVER -> DEFINE

- [ ] All 10 Interview Matrix cells addressed (CORE INTENT + ERROR CASES x 5 perspectives)
- [ ] At least 5 EARS requirements drafted (mix of patterns)
- [ ] At least 3 unwanted (negative) requirements identified
- [ ] All assumptions listed with confidence ratings
- [ ] No critical unknowns remain unaddressed
- [ ] User has confirmed the problem space exploration is sufficient

**Announcement:** "Converging Diamond 1 -- entering DEFINE phase"

---

## Phase 2: DEFINE (Diamond 1 -- Converge)

**Duration:** 15-20 minutes (interactive)
**Lattice:** KIRK Contracts
**Interview Matrix:** Rows 3-4 (EDGE CASES + SECURITY) across all 5 perspectives

### Purpose

CONVERGE on the precise problem statement. Lock down scope. Begin formal contracts. This is where vagueness dies.

### Actions

1. **Write the problem statement:**
   - Single paragraph
   - Specific and falsifiable
   - No weasel words ("should", "may", "could")

2. **Define scope:**
   - IN: What this feature DOES (explicit list)
   - OUT: What this feature does NOT do (explicit list)
   - DEFERRED: What might be added later (explicit list)

3. **Begin KIRK contracts:**
   - For each component/behavior identified in DISCOVER:
     - Preconditions (with enforcement level)
     - Postconditions (with verification mechanism)
     - Invariants (with enforcement mechanism)
   - Begin error taxonomy (name every error variant)

4. **Interview Matrix -- EDGE CASES row:**
   - USER: Unusual usage patterns?
   - DEVELOPER: Untested paths? Boundary values?
   - OPS: Rare but catastrophic scenarios?
   - SECURITY: Unexpected inputs? Time-based attacks?
   - BUSINESS: Seasonal patterns? Growth scenarios?

5. **Interview Matrix -- SECURITY row:**
   - USER: Privacy expectations?
   - DEVELOPER: Validation strategy?
   - OPS: Security monitoring?
   - SECURITY: Threat model? OWASP coverage?
   - BUSINESS: Compliance requirements?

6. **Finalize EARS requirements:**
   - All patterns represented
   - Every requirement uses EARS syntax
   - No ambiguous language remains

### Grilling in DEFINE

- "Your scope says 'in'. Is X really in scope, or are you gold-plating?"
- "Your scope says 'out'. If X is out, what happens when a user tries to do X?"
- "You have 3 preconditions. What happens when each is violated? Name the exact error."
- "This invariant says 'always true'. What about during startup? Migration? Crash recovery?"
- "You used the word 'should' in requirement 4. Replace it with 'SHALL' or remove it."

### Gate: DEFINE -> DEVELOP

- [ ] Problem statement is written, specific, and falsifiable
- [ ] Scope is defined: IN / OUT / DEFERRED with explicit lists
- [ ] KIRK contracts started for all identified components
- [ ] Error taxonomy has at least one variant per component
- [ ] EARS requirements finalized (all 6 patterns evaluated)
- [ ] All 20 Interview Matrix cells addressed (rows 1-4 x 5 perspectives)
- [ ] User has confirmed the problem definition

**Announcement:** "Diamond 1 complete. Entering Diamond 2: Solution Space -- DEVELOP phase"

---

## Phase 3: DEVELOP (Diamond 2 -- Diverge)

**Duration:** 20-40 minutes (interactive -- this is the longest phase)
**Lattices:** Inversion Thinking + Second-Order Thinking
**Interview Matrix:** Row 5 (OPERATIONS) across all 5 perspectives

### Purpose

Explore the SOLUTION space broadly. Apply inversion thinking to find all failures. Trace second-order consequences. Explore multiple architectural approaches. This is where the heaviest grilling happens.

### Actions

1. **Explore architectural approaches:**
   - Identify at least 2-3 viable approaches
   - For each: describe, list pros/cons, identify risks
   - Do NOT choose yet -- diverge first

2. **Inversion Analysis (Lattice 3):**
   - Walk through ALL security inversions (10 categories)
   - Walk through ALL usability inversions (10 categories)
   - Walk through ALL integration inversions (10 categories)
   - For each applicable inversion: define trigger, expected behavior, error code

3. **Second-Order Consequence Tracing (Lattice 4):**
   - For each major behavior:
     - State the first-order effect
     - Trace ALL second-order effects
     - Define consequence checks
   - For high-risk operations, trace third-order effects

4. **Domain Modeling:**
   - Define all types (entities, value objects, aggregates)
   - Define all states and state transitions
   - Identify ILLEGAL states and how the type system prevents them
   - Define domain events

5. **Interview Matrix -- OPERATIONS row:**
   - USER: Recovery paths?
   - DEVELOPER: Scaling strategy?
   - OPS: Failure and degradation strategy?
   - SECURITY: Audit and forensics?
   - BUSINESS: Growth projections and cost?

6. **Complete KIRK contracts:**
   - All preconditions with enforcement levels
   - All postconditions with verification mechanisms
   - All invariants with enforcement mechanisms
   - Exhaustive error taxonomy with concrete violation examples

### Grilling in DEVELOP (This is Where You're Relentless)

**State and Type Grilling:**
- "List every state this entity can be in. Now list every legal transition. Now list every ILLEGAL transition. How do you prevent the illegal ones?"
- "You have a `status` field that's a string. Why isn't it an enum? What illegal values can it hold?"
- "This type allows null. When is it null? What does null mean? Should it be an `Option` with explicit semantics instead?"

**Error Grilling:**
- "You listed 5 error types. What about timeout? Partial failure? Concurrent modification? Disk full? Network partition?"
- "This returns a generic Error. Break it into specific variants. I want one variant per failure mode."
- "What's the error message for this variant? Is it actionable? Does it leak internal state?"

**Inversion Grilling:**
- "What happens when auth fails? When auth expires mid-operation? When auth is from the wrong user?"
- "What happens when the request body is 100MB? 0 bytes? Valid JSON but wrong schema?"
- "What happens when this is called twice with the same idempotency key?"

**Consequence Grilling:**
- "You delete entity X. What happens to Y that references X? And Z that references Y?"
- "You add caching. What's the staleness window? What if invalidation fails? What if the cache fills up?"
- "This triggers an event. Three services consume it. What if one fails? Do the others roll back?"

**Architecture Grilling:**
- "You chose approach A. What does approach B look like? Why is A better?"
- "This adds a new service. What's the operational cost of maintaining it for 3 years?"
- "This creates a synchronous dependency. What happens when it's slow? Down? What if we made it async?"

### Gate: DEVELOP -> DELIVER

- [ ] At least 2 architectural approaches explored with pros/cons
- [ ] Inversion analysis complete (security + usability + integration)
- [ ] Second-order consequences traced for all major behaviors
- [ ] Domain model complete (types, states, transitions, illegal states)
- [ ] KIRK contracts complete (pre/post/invariants for all components)
- [ ] Error taxonomy is exhaustive (no unnamed failure modes remain)
- [ ] All 25 Interview Matrix cells addressed
- [ ] User has confirmed the solution space exploration is sufficient

**Announcement:** "Converging Diamond 2 -- entering DELIVER phase"

---

## Phase 4: DELIVER (Diamond 2 -- Converge)

**Duration:** 15-20 minutes (synthesis)
**Lattice:** Pre-Mortem Analysis + Quality Scoring
**Interview Matrix:** Final validation pass (all perspectives)

### Purpose

CONVERGE on the final architecture. Run pre-mortem. Score quality. Lock everything down. Produce the spec.

### Actions

1. **Choose the architectural approach:**
   - State the recommendation
   - Justify with evidence from DEVELOP phase
   - Document rejected alternatives with reasons

2. **Pre-Mortem Analysis (Lattice 5):**
   - "Imagine this failed catastrophically after 1 week. Why?"
   - List 5-10 failure causes
   - For each: probability, severity, detection mechanism, mitigation
   - Classify mitigations as in-scope or deferred

3. **Quality Scoring:**
   - Score across 5 KIRK dimensions:
     - Completeness: sections filled / total sections
     - Consistency: conflicting requirements (0 = 100%)
     - Testability: behaviors with acceptance criteria / total behaviors
     - Clarity: requirements with "why" / total requirements
     - Security: security inversions addressed / total applicable
   - Target: 90%+ overall
   - If below 90%, identify gaps and re-enter appropriate lattice

4. **Final validation pass:**
   - Each perspective gets 1-2 final questions:
     - USER: "Is the user experience fully specified?"
     - DEVELOPER: "Are all contracts and error types locked?"
     - OPS: "Is the deployment and monitoring plan clear?"
     - SECURITY: "Are all attack vectors addressed?"
     - BUSINESS: "Is the value proposition justified?"

5. **Compile the architecture spec:**
   - Assemble all artifacts into `architecture-spec.md`
   - Use the template from [output-template.md](output-template.md)

### Gate: DELIVER -> HANDOFF

- [ ] Architectural approach chosen and justified
- [ ] Pre-mortem complete (5+ causes with mitigations)
- [ ] Quality score >= 90% across all dimensions
- [ ] Final validation pass complete (all 5 perspectives)
- [ ] Architecture spec compiled

**Announcement:** "Architecture specification complete. Ready for handoff."

---

## Phase 5: HANDOFF

**Duration:** 5 minutes
**Actions:**

1. Write `architecture-spec.md` using output template
2. Present quality score summary
3. List open risks and deferred decisions
4. Recommend one of:
   - **"Ready for planner"** -- spec is complete, planner can decompose
   - **"Needs more discovery on [X]"** -- specific gaps identified
   - **"Blocked on [X]"** -- external dependency prevents completion

---

## Phase Timing Summary

| Phase | Duration | Lattices | Matrix Rows | Key Output |
|-------|----------|----------|-------------|------------|
| INTAKE | 5-10 min | None | None | Context, unknowns |
| DISCOVER | 15-30 min | EARS | 1-2 | EARS requirements, assumptions |
| DEFINE | 15-20 min | KIRK | 3-4 | Scope, contracts, error taxonomy |
| DEVELOP | 20-40 min | Inversion + 2nd-Order | 5 | Domain model, inversions, consequences |
| DELIVER | 15-20 min | Pre-Mortem + Quality | Final pass | Architecture spec, quality score |
| HANDOFF | 5 min | None | None | architecture-spec.md |

**Total: 75-120 minutes for a fully specified feature**

---

## Re-entry Protocol

If a gate fails, re-enter the appropriate phase:

| Gap | Re-enter | Focus |
|-----|----------|-------|
| Missing requirements | DISCOVER | More EARS drafting |
| Vague contracts | DEFINE | More KIRK specification |
| Missing failure modes | DEVELOP | More Inversion analysis |
| Missing consequences | DEVELOP | More 2nd-Order tracing |
| Low quality score | DELIVER | Identify dimension gaps, fix |
| New information | INTAKE | Re-assess scope and unknowns |

You may re-enter any phase at any time if new information surfaces. The state machine is not strictly linear -- it's iterative with gates.
