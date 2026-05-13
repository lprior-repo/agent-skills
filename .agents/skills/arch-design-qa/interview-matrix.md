# The 5x5 Interview Matrix

Reference document for the architect skill. The Interview Matrix is a systematic requirement-gathering tool that ensures comprehensive coverage across 5 rounds of questioning from 5 distinct perspectives.

## How It Works

The architect walks through 25 cells (5 rounds x 5 perspectives). Each cell targets a specific intersection of concern and viewpoint. The user must address each cell or explicitly defer it with a reason.

## The Matrix

### Round 1: CORE INTENT

The foundation. What are we actually building and why?

| Perspective | Question | What You're Looking For |
|-------------|----------|------------------------|
| **USER** | What problem does this solve for real users? Who are they? What do they do today without this? | Evidence of actual user need, not assumed need. Specific personas, not "everyone". |
| **DEVELOPER** | What components/modules are involved? What's the technical surface area? | Boundaries, interfaces, dependencies. Where does new code live? What existing code is touched? |
| **OPS** | What scale is expected? What infrastructure is needed? | Concurrency, throughput, storage, compute. Day-1 vs Day-90 expectations. |
| **SECURITY** | What data is sensitive? What trust boundaries are crossed? | PII, credentials, tokens, audit requirements. Where does data flow across trust zones? |
| **BUSINESS** | What metrics define success? What's the cost of not doing this? | KPIs, revenue impact, competitive pressure. Quantified value, not vague "it'd be nice". |

**Grilling examples:**
- "You said 'users need this'. Which users? How many? What's the evidence?"
- "You said this touches the auth module. What other modules depend on auth? Did you account for them?"
- "What's the expected request rate? Don't say 'not much'. Give me a number."

---

### Round 2: ERROR CASES

Everything that can go wrong. This is where most specs fail.

| Perspective | Question | What You're Looking For |
|-------------|----------|------------------------|
| **USER** | What frustrates users? What confusing states can they reach? What do bad error messages look like? | UX failures, dead ends, unclear feedback. Every error the user sees must be actionable. |
| **DEVELOPER** | What breaks? What exceptions/errors can each component throw? What's the full error taxonomy? | Exhaustive error variant list. Not "it might fail" but "it fails with ErrorA, ErrorB, ErrorC, and here's when each one happens". |
| **OPS** | What alarms should fire? What does degraded look like? What's the runbook? | Alerting thresholds, degradation signals, incident response. If this breaks at 3am, what do you check first? |
| **SECURITY** | What sensitive data could be exposed in error responses? What do error paths leak? | Stack traces, internal IDs, SQL queries, file paths in error messages. Error paths are the #1 information disclosure vector. |
| **BUSINESS** | What does failure cost? Per-incident cost? Reputational damage? SLA impact? | Quantified cost of downtime, data loss, or degraded experience. This determines investment in resilience. |

**Grilling examples:**
- "You have 3 error types. What about timeout? What about partial success? What about concurrent modification?"
- "What does the user see when this fails? 'Something went wrong' is not acceptable. What EXACTLY do they see?"
- "This error path returns a 500. What sensitive information is in that response body?"

---

### Round 3: EDGE CASES

The unusual-but-valid scenarios that break assumptions.

| Perspective | Question | What You're Looking For |
|-------------|----------|------------------------|
| **USER** | What unusual things do users do? What if they use it "wrong"? What about accessibility? | Double-clicks, back-button, refresh mid-operation, screen readers, keyboard-only navigation. |
| **DEVELOPER** | What's untested? What code paths have zero coverage? What about empty/null/max values? | Boundary conditions, empty collections, maximum sizes, Unicode edge cases, zero-length strings, negative numbers. |
| **OPS** | What's rare but catastrophic? What happens during deployment? During migration? | Blue-green deploy mid-transaction, database migration with active connections, cache cold-start after restart. |
| **SECURITY** | What unexpected inputs could arrive? What about time-based attacks? | Timing attacks, race conditions in auth, TOCTOU (time-of-check-time-of-use), clock skew exploitation. |
| **BUSINESS** | What's seasonal? What about holidays, timezone changes, leap years? | Black Friday load spikes, DST transitions, leap second handling, fiscal year boundaries, currency precision. |

**Grilling examples:**
- "What happens when the input is empty? When it's the maximum allowed size? When it contains emoji? RTL text?"
- "You deploy a new version while a user is mid-transaction. What happens to their state?"
- "It's February 29th and your date validation rejects it. Did you test that?"

---

### Round 4: SECURITY

Dedicated security analysis. Not just "is it secure" but "how specifically can it be attacked".

| Perspective | Question | What You're Looking For |
|-------------|----------|------------------------|
| **USER** | What do users fear? Data theft? Account takeover? What privacy expectations exist? | User trust model, privacy expectations, consent requirements. What would make a user leave? |
| **DEVELOPER** | What validates input? What sanitizes output? Where are the trust boundaries in code? | Input validation strategy, output encoding, parameterized queries, CSRF tokens, CSP headers. |
| **OPS** | What's monitored for security? What alerts on anomalous behavior? | Failed login tracking, anomaly detection, audit logging, rate limiting dashboards. |
| **SECURITY** | What's the threat model? What attack vectors exist? OWASP Top 10 coverage? | Injection, broken auth, sensitive data exposure, XXE, broken access control, misconfig, XSS, insecure deserialization, vulnerable components, insufficient logging. |
| **BUSINESS** | What's the liability? Regulatory requirements? Breach notification obligations? | GDPR, SOC2, HIPAA, PCI-DSS applicability. Fine amounts, notification timelines, data residency requirements. |

**Grilling examples:**
- "You validate email format. Do you also check for email header injection? What about plus-addressing for enumeration?"
- "Show me every trust boundary in this feature. Where does untrusted data enter? Where does it exit?"
- "If an attacker gets a valid JWT, what's the blast radius? What can they access?"

---

### Round 5: OPERATIONS

How this lives in production. The part most specs forget entirely.

| Perspective | Question | What You're Looking For |
|-------------|----------|------------------------|
| **USER** | How do users recover from problems? Self-service? Support ticket? Retry? | Recovery paths, self-service tools, support escalation, data recovery options. |
| **DEVELOPER** | What scales? What's the horizontal scaling strategy? What's stateful vs stateless? | Stateless compute, externalized state, connection pooling, sharding strategy. |
| **OPS** | What fails gracefully? What's the degradation strategy? What's the rollback plan? | Circuit breakers, feature flags, canary deployments, database rollback, cache fallback. |
| **SECURITY** | What's audited? What logs exist for forensics? What's the incident response? | Audit trail completeness, log retention, tamper-proof logging, forensic capability. |
| **BUSINESS** | What grows? What are the scaling cost projections? At 10x users, what breaks first? | Cost per user, infrastructure scaling curve, feature flags for gradual rollout, capacity planning. |

**Grilling examples:**
- "You said it's stateless. Where does the session live? Where does the upload buffer live? Those are state."
- "Deploy goes wrong. How do you roll back? What about database migrations -- are they reversible?"
- "At 10x current load, what breaks first? Don't say 'nothing'. Something always breaks first."

---

## Tracking Completion

Mark each cell as you go:

```
             USER  DEV   OPS   SEC   BIZ
CORE INTENT  [ ]   [ ]   [ ]   [ ]   [ ]
ERROR CASES  [ ]   [ ]   [ ]   [ ]   [ ]
EDGE CASES   [ ]   [ ]   [ ]   [ ]   [ ]
SECURITY     [ ]   [ ]   [ ]   [ ]   [ ]
OPERATIONS   [ ]   [ ]   [ ]   [ ]   [ ]
```

**Gate**: All 25 cells must be addressed (answered or explicitly deferred with reason) before the architecture spec is considered complete.

## Deferred Cells

When a cell is deferred, document it as:

```
DEFERRED: [Round] x [Perspective]
Reason: [Why this is deferred]
Risk: [What risk does deferral introduce]
Revisit: [When/what triggers revisiting this]
```

Deferred cells become "Open Risks" in the architecture spec.
