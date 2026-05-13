# TLA+ Modeling Patterns

## Ownership Boundary

TLA+ models behavior over time. Use it for protocols, workflows, retries, schedulers, leases, queues, lifecycle transitions, concurrency contracts, distributed state, fairness, liveness, and deadlock freedom.

Do not use TLA+ as a replacement for Verus proof of local Rust functions. Use TLA+ to prove the state machine is coherent, then use Verus/Kani/tests/Loom/Stateright to connect implementation to the model.

## Minimum Model Shape

Every model needs these parts:

- `CONSTANTS`: bounded sets such as workers, nodes, messages, tasks, attempts, or model values.
- `VARIABLES`: mutable state such as owner, queue, lease, phase, log, clock, in_flight, failed, done.
- `vars`: tuple of all mutable variables.
- `Init`: complete initial state predicate.
- Actions: named transitions such as `Claim`, `Renew`, `Release`, `Timeout`, `Retry`, `Commit`, `Abort`, `Crash`, `Recover`.
- `Next`: disjunction of all actions, usually with unchanged variables per action.
- Type invariant: every variable stays in the intended finite set or record shape.
- Safety invariants: bad states never happen.
- Temporal properties: eventually, always, leads-to, fairness, and termination claims.
- Deadlock stance: TLC deadlock checking enabled or an explicit reason to disable it.
- Refinement map: how Rust/runtime events map to actions and variables.

Default complete safety form:

```tla
Spec == Init /\ [][Next]_vars
```

Do not write bare `[]Next` unless there is a deliberate, documented reason to reject stuttering. Stuttering is what keeps refinement practical when the implementation has extra internal steps.

## Behavior Spec Forms

Use `Init` plus `Next` for safety-only specs when no fairness or liveness is checked.

The canonical safety shape is:

```tla
Spec == Init /\ [][Next]_vars
```

Use temporal formula shape when checking fairness or liveness:

```tla
Spec == Init /\ [][Next]_vars /\ WF_vars(Claim) /\ WF_vars(Complete)
```

The exact fairness operators must match the intended scheduler assumption. Weak fairness says an action eventually happens if continuously enabled. Strong fairness says an action eventually happens if enabled infinitely often.

## Safety Property Patterns

- Type safety: variables always stay in finite domains.
- Mutual exclusion: two owners cannot hold the same lease.
- No loss: every accepted item is queued, processed, failed, or completed.
- Monotonic progress marker: generation, epoch, version, or committed index never decreases.
- Idempotency: duplicate retry does not duplicate committed effect.
- Ownership: only current owner can renew, commit, or release.
- Causal order: commit cannot precede prepare, ack cannot precede send.
- Conservation: counts of items across queues and terminal sets match original set.

## Temporal Property Patterns

- Eventually done: accepted work eventually reaches done or failed under fairness.
- No starvation: enabled worker eventually gets a claim under scheduler assumptions.
- Retry progress: retryable failure eventually attempts again until budget exhausted.
- Lease recovery: expired lease eventually becomes claimable.
- Shutdown drain: after stop, in-flight work eventually finalizes or is cancelled.
- No infinite limbo: state cannot remain forever in pending without enabled transition.

## Bounds And Constraints

TLC checks finite models. Constants and constraints are part of the claim.

Record these in the evidence:

- number of nodes, workers, tasks, messages, retries, epochs, or queue slots;
- model values and typed model values;
- state constraints and action constraints;
- symmetry sets and why symmetry is sound for safety;
- liveness disabled by symmetry or distributed mode;
- worker count and whether deterministic one-worker mode was used.

State constraints can hide bugs. Prefer small but meaningful finite sets over constraints that remove important states.

## Model Values And Symmetry

Model values represent uninterpreted constants such as workers or nodes. Typed model values can catch accidental cross-domain comparisons.

Symmetry can cut state space. Treat it as trusted reduction because TLC does not prove the symmetry assumption. Do not use symmetry for liveness checks unless the proof obligation explicitly accepts the limitation.

## Refinement Map To Rust

Every TLA+ model must say how it maps to implementation:

- Rust event or function that corresponds to each action.
- Persistent state corresponding to each variable.
- Inputs/environment modeled as nondeterministic actions.
- Failures modeled explicitly: timeout, crash, cancellation, duplicate message, retry exhaustion.
- Shell behavior excluded from the model and covered by tests, QA, Loom, Stateright, Kani, Miri, or fuzzing.

## PlusCal Guidance

PlusCal is useful for algorithmic workflows and for readers who prefer pseudo-code. Pure TLA+ is often better for protocols with many independent actions and nondeterministic environment steps.

Use PlusCal for algorithm-shaped specs: mutual exclusion, producer-consumer flows, request-response protocols, phase machines, leader election, and bounded protocol sketches.

Use raw TLA+ first for abstract contracts, refinement mappings, set/history/message models, and math-first design statements.

If using PlusCal, commit both the PlusCal source and generated TLA+ or document the translation command. Do not hand-edit generated sections without recording why.

## Syntax And TLC Traps

These are common failure modes from real TLC/SANY repair sessions:

| Trap | Wrong | Correct |
|------|-------|---------|
| Comment syntax | `* text` | `\* text` — bare `*` is multiplication |
| Else-if syntax | `ELSIF ...` | `ELSE IF ...` or `CASE ...` |
| Else-if indentation | `/\ ELSE IF ...` | `ELSE IF ...` continues the prior `IF`; do not start a new conjunction branch |
| Set cardinality | `\|set\|` | `Cardinality(set)` with `EXTENDS FiniteSets` |
| Unassigned marker | `Nil` | Use a concrete marker such as `0`, or declare `Nil` as a model value in the config |
| Underspecified action | assigning only changed variables | Every action must assign every variable or state `UNCHANGED <<...>>` |
| Primed action as invariant | `THEOREM Spec => []ActionWithPrimes` | Use action form `THEOREM Spec => [][Action]_vars`, or define an unprimed state invariant |
| Inline config constraint | `CONSTRAINT Len(journal) <= 4` in `.cfg` | Define `Bound == Len(journal) <= 4` in `.tla`, then use `CONSTRAINT Bound` or `STATE_CONSTRAINT Bound` |
| Unbounded append | `journal' = Append(journal, e)` with no bound/termination | Add a finite bound, terminal stutter action, or split the obligation |
| Tuple-keyed function | `f[a][b]` when keys are tuples | Use `f[<<a, b>>]` and `EXCEPT ![<<a, b>>] = v` |

## Anti-Patterns

- Modeling only the happy path.
- Missing environment/failure actions.
- Omitting `UNCHANGED` variables in actions.
- Safety invariants that restate the type invariant only.
- Liveness claim without fairness assumption.
- Disabling deadlock check without a written reason.
- State constraint that removes the failure being claimed impossible.
- Symmetry used with liveness.
- No refinement map from model actions to Rust/runtime events.
- Treating random simulation as proof.
- Reporting TLC success without state counts and bounds.
- Claiming liveness from a safety-only `Spec`.
- Using a strong invariant such as cache coherence while allowing transient asynchronous states that violate it, then pretending the model passed.
