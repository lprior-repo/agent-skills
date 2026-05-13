# The Five Perspectives (Interview Matrix Personas)

Reference document for the architect skill. Each perspective represents a distinct viewpoint that surfaces different types of requirements, risks, and concerns.

The architect rotates through these perspectives during each phase of the Double Diamond. No single perspective is sufficient. All five must be heard.

---

## Perspective 1: USER

**Voice:** "I'm the person who has to live with this every day."
**Focus:** Value, experience, pain points, trust, recovery
**Bias to counter:** Developer tendency to optimize for technical elegance over user experience

### Core Questions

1. Who exactly is the user? (Be specific -- not "everyone")
2. What problem do they have right now? What do they do today without this feature?
3. What does success look like from their perspective? How do they know it worked?
4. What's the worst experience they could have with this feature?
5. When something goes wrong, what do they see? What can they do about it?
6. What do they expect to happen that we haven't thought of?
7. How do they discover this feature exists?
8. What makes them trust this feature? What makes them stop trusting it?

### Error-Focused Questions

- "The user clicks the button and nothing happens. Then what?"
- "The operation takes 30 seconds. What does the user see during that time?"
- "The user's data is lost. How do they find out? How do they recover?"
- "The user made a mistake. Can they undo it? How?"
- "The error message says 'Error 500'. That's not acceptable. What SHOULD it say?"

### Edge Case Questions

- "The user double-clicks. What happens?"
- "The user hits back mid-operation. What happens?"
- "The user is on a slow 3G connection. What happens?"
- "The user is using a screen reader. Does this work?"
- "The user copy-pastes from Word with smart quotes. Does this work?"
- "The user has 10,000 items. Does the UI still work?"

### The USER Perspective is Satisfied When:

- [ ] Specific user personas are identified (not "all users")
- [ ] Every error has a user-facing message that is actionable
- [ ] Every long operation has a loading/progress indicator defined
- [ ] Every destructive action has an undo or confirmation defined
- [ ] Accessibility requirements are specified
- [ ] The user recovery path for every failure mode is documented

---

## Perspective 2: DEVELOPER

**Voice:** "I'm the one who has to build and maintain this."
**Focus:** Components, interfaces, contracts, error types, testability, maintainability
**Bias to counter:** Tendency to under-specify error handling and edge cases

### Core Questions

1. What components/modules are involved? What's the surface area?
2. What are the interfaces between components? What's the contract?
3. What's the full error taxonomy? List every error variant.
4. What are the preconditions, postconditions, and invariants?
5. What states can this system be in? Which transitions are legal? Which are illegal?
6. What existing code is affected? What's the blast radius of this change?
7. What tests need to exist before this is "done"?
8. What's the simplest thing that could possibly work?

### Error-Focused Questions

- "You said this can fail. Name every way it can fail. Now name one more."
- "This function returns `Result<T, E>`. What are ALL the variants of E?"
- "What happens when the database is unavailable? The cache? The queue?"
- "Two requests hit this endpoint simultaneously. What happens?"
- "The response is 50MB. Does anything break?"

### Type-System Questions

- "What types enforce this constraint at compile time?"
- "Can you represent an illegal state with these types? If yes, fix the types."
- "This string could be anything. Should it be a newtype instead?"
- "This `Option<T>` -- when is it `None` and what does that mean?"
- "This boolean parameter -- should it be an enum with named variants instead?"

### The DEVELOPER Perspective is Satisfied When:

- [ ] All components and their interfaces are defined
- [ ] Error taxonomy is exhaustive with concrete variants
- [ ] KIRK contracts (pre/post/invariants) are specified
- [ ] State machine (if any) has all transitions enumerated
- [ ] Types make illegal states unrepresentable
- [ ] Test scenarios cover happy path, error path, and edge cases
- [ ] Dependencies and affected code are mapped

---

## Perspective 3: OPS

**Voice:** "I'm the one who gets paged at 3am when this breaks."
**Focus:** Deployment, monitoring, scaling, failure recovery, runbooks
**Bias to counter:** Developer tendency to assume infrastructure "just works"

### Core Questions

1. How is this deployed? What's the deployment strategy?
2. What do you monitor? What alerts exist? What are the thresholds?
3. What breaks under 10x load? 100x load?
4. What's the rollback plan if deployment goes wrong?
5. What logs exist? Can you reconstruct what happened from logs alone?
6. What's the disaster recovery plan? RPO? RTO?
7. What resource limits exist? (CPU, memory, connections, disk, network)
8. What happens during a deployment? Are there any gaps in service?

### Failure Recovery Questions

- "The database is full. What happens? How do you fix it at 3am?"
- "Memory usage grows 1% per hour. When does it crash? How do you detect it?"
- "A bad deployment went out. How long to detect? How long to roll back?"
- "The cache cluster dies. What's the impact? Cold-start time?"
- "DNS goes down for 5 minutes. What breaks? What recovers automatically?"

### Observability Questions

- "Show me the dashboard for this feature. What graphs are on it?"
- "An SLO is breached. How do you know? What fires? What's the runbook?"
- "A user reports slowness. Walk me through the debugging process."
- "I need to know if this feature is working right now. Where do I look?"
- "How do you correlate a user complaint to a specific request in logs?"

### The OPS Perspective is Satisfied When:

- [ ] Deployment strategy is defined (blue-green, canary, rolling)
- [ ] Monitoring and alerting thresholds are specified
- [ ] Rollback plan is documented and tested
- [ ] Scaling strategy is defined (horizontal, vertical, auto)
- [ ] Log format and log levels are specified for key operations
- [ ] Runbooks exist for top 5 failure scenarios
- [ ] Resource limits and capacity planning are documented

---

## Perspective 4: SECURITY

**Voice:** "I'm the attacker. How do I break this?"
**Focus:** Threat modeling, attack vectors, trust boundaries, data protection, compliance
**Bias to counter:** Optimism bias -- assuming good faith from all inputs

### Core Questions

1. What's the threat model? Who are the adversaries? What do they want?
2. Where are the trust boundaries? Where does untrusted data enter?
3. What sensitive data does this feature handle? Where does it flow?
4. What's the authentication model? Authorization model?
5. What happens if credentials are compromised? What's the blast radius?
6. What OWASP Top 10 vulnerabilities apply?
7. What compliance requirements exist? (GDPR, SOC2, HIPAA, PCI-DSS)
8. What's the incident response plan for a security breach in this feature?

### Attack Vector Questions

- "I have a valid JWT. What's the worst I can do?"
- "I control the input. What injection attacks are possible?"
- "I intercept the network traffic. What do I learn?"
- "I'm a malicious insider with developer access. What can I exfiltrate?"
- "I flood this endpoint. What breaks? At what rate?"
- "I send a request with 1MB of JSON in a field expecting a name. What happens?"

### Data Protection Questions

- "Where is this data stored? Encrypted at rest? What key management?"
- "Who can access this data? Is access logged? Can access be revoked?"
- "If I dump the database, what sensitive data is in cleartext?"
- "If I read the logs, what sensitive data appears?"
- "If I capture the API response, what sensitive data is included?"
- "How long is this data retained? Is there a deletion mechanism?"

### The SECURITY Perspective is Satisfied When:

- [ ] Threat model is documented (adversaries, capabilities, goals)
- [ ] Trust boundaries are identified and mapped
- [ ] All sensitive data flows are traced end-to-end
- [ ] Input validation strategy is defined for every entry point
- [ ] Authentication and authorization model is specified
- [ ] OWASP Top 10 checklist is evaluated
- [ ] Compliance requirements are identified
- [ ] Error responses don't leak sensitive information

---

## Perspective 5: BUSINESS

**Voice:** "I'm paying for this. Show me the value."
**Focus:** ROI, metrics, cost, competitive advantage, risk, compliance, growth
**Bias to counter:** Engineering tendency to build for technical interest rather than business value

### Core Questions

1. What metric moves if this succeeds? By how much?
2. What's the cost of building this? (Engineer-hours, infrastructure, opportunity cost)
3. What's the cost of NOT building this? (Lost revenue, churn, competitive disadvantage)
4. What's the simplest version that delivers 80% of the value?
5. How do we measure success after launch? What's the feedback loop?
6. What's the regulatory/compliance impact?
7. What happens to existing customers during this transition?
8. What's the competitive landscape? Are we behind, ahead, or differentiated?

### Cost Questions

- "At 10x users, what's the infrastructure cost? Is it linear or exponential?"
- "This feature needs a new service. What's the operational cost of maintaining it?"
- "You want to use service X. What's the vendor lock-in risk? What's the exit cost?"
- "This adds complexity. What's the ongoing maintenance cost in engineer-hours per month?"
- "If we defer this 6 months, what do we lose? What do we gain?"

### Value Questions

- "Which users pay more because of this feature? How much more?"
- "Which users leave without this feature? What's their LTV?"
- "What's the competitive moat this creates? How long until competitors copy it?"
- "What does this enable that wasn't possible before? What's that worth?"
- "If this feature didn't exist in 2 years, would anyone notice?"

### The BUSINESS Perspective is Satisfied When:

- [ ] Success metrics are defined and measurable
- [ ] Cost estimates exist (build + run + maintain)
- [ ] ROI is justified (value delivered > cost)
- [ ] MVP scope is defined (80% value at 20% cost)
- [ ] Competitive context is understood
- [ ] Compliance requirements are identified and costed
- [ ] Go/no-go criteria are clear

---

## Rotation Protocol

The architect rotates perspectives based on the current Diamond phase:

| Phase | Primary Perspectives | Secondary |
|-------|---------------------|-----------|
| DISCOVER | USER, BUSINESS | DEVELOPER |
| DEFINE | DEVELOPER, SECURITY | OPS |
| DEVELOP | DEVELOPER, SECURITY, OPS | USER, BUSINESS |
| DELIVER | ALL (final pass) | -- |

In each phase, the primary perspectives ask 3-5 questions each. Secondary perspectives ask 1-2. During DELIVER, all perspectives get a final validation pass.

## The Grilling Mindset

Every perspective shares these principles:

1. **"What else?"** -- The first answer is never complete
2. **"How does this fail?"** -- Every feature has failure modes
3. **"Show me, don't tell me"** -- Demand concrete examples, not abstractions
4. **"What if you're wrong?"** -- Challenge every assumption
5. **"And then what?"** -- Trace consequences beyond the immediate effect
