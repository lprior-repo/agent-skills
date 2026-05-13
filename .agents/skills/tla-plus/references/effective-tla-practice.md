# Effective TLA+ In Practice

This reference is the local operating guide for TLA+ work. It is deliberately CLI-first: edit `.tla` and `.cfg` files in any editor, then run SANY/TLC from the command line. Do not require VS Code for this workflow. Treat the Toolbox as legacy/unmaintained unless a project explicitly inherits a Toolbox-specific model.

## Executive Summary

TLA+ is the practical specification language built around TLA, the Temporal Logic of Actions, for reasoning about concurrent, distributed, reactive, and state-over-time systems.

The highest-leverage workflow is:

1. Model the design before code.
2. Choose a small abstraction boundary.
3. Make the model finite.
4. Write `TypeOK` and one strong semantic invariant first.
5. Run TLC early and read counterexamples immediately.
6. Stabilize safety and deadlock behavior before liveness.
7. Add fairness, refinement, or TLAPS proof obligations only after the safety story is solid.

Most practical value comes from safety properties: invariants, deadlock checks, and counterexamples. Liveness is still important when the requirement is progress, no starvation, eventual handling, or recovery, but it is more expensive and easier to get wrong.

## Core Mental Model

TLA is the underlying temporal logic. TLA+ is the engineering language used to write specifications in ordinary math plus temporal operators.

| Term | Practical meaning | Write first |
|------|-------------------|-------------|
| TLA | Temporal Logic of Actions, the mathematical foundation | Read the intro and simple examples |
| TLA+ | Engineering language built from math plus temporal operators | Start with a tiny checked module |
| State | Assignment of values to all variables right now | `VARIABLES` and `Init` |
| State predicate | Boolean expression over one state | `TypeOK`, semantic invariant |
| Action | Relation between old variables and primed next-state variables | `Send`, `Receive`, `Grant`, `Timeout` |
| Behavior | Sequence of states | What `Spec` constrains |
| Invariant | Predicate true in every reachable state | `TypeOK`, then one semantic invariant |
| Safety | Nothing bad happens | Invariants, action properties, deadlock checks |
| Liveness | Something good eventually happens | Fairness and temporal properties after safety |
| Refinement | Lower-level spec implements a higher-level spec | Abstraction variables and refinement maps |
| Stuttering | Steps that leave viewed variables unchanged | Use `[Next]_vars`, not bare `[]Next` |

## Canonical Spec Shape

Internalize this shape:

```tla
Spec == Init /\ [][Next]_vars
```

`Init` constrains the first state. `Next` constrains non-stuttering transitions. `[Next]_vars` means either a `Next` step occurs or all variables in `vars` are unchanged. This explicit stuttering is what makes refinement practical across different atomicity choices.

Use `Spec == Init /\ [][Next]_vars` for the complete safety behavior. Add fairness and liveness only when the requirement demands progress:

```tla
LiveSpec == Spec /\ WF_vars(Handle) /\ SF_vars(Retry)
```

Weak fairness says an action must eventually happen if it remains continuously enabled. Strong fairness says an action must eventually happen if it becomes enabled again and again. Prefer liveness as a conjunction of `WF_vars(A)` and `SF_vars(A)` assumptions over ad hoc temporal formulas.

## Constant Formula Cheat Sheet

| Formula | Read it as | Use |
|---------|------------|-----|
| `Init` | Initial states | Entry conditions |
| `Next` | Allowed transitions | Core behavior |
| `[Next]_vars` | `Next` or stutter on `vars` | Robust safety spec |
| `UNCHANGED x` | `x' = x` | Variables untouched by an action |
| `[]P` | Always `P` | Safety and invariants |
| `<>P` | Eventually `P` | Liveness goals |
| `WF_vars(A)` | Weak fairness of `A` | Progress under continuous enablement |
| `SF_vars(A)` | Strong fairness of `A` | Progress under repeated enablement |
| `ENABLED A` | There exists a next state satisfying `A` | Fairness/progress reasoning |
| `INSTANCE M WITH ...` | Reuse module with substitutions | Reuse and refinement |

## Local Toolchain Policy

Use three layers:

1. Any editor for `.tla` and `.cfg` files.
2. Java plus `tla2tools.jar` or a `tlc` wrapper for SANY/TLC.
3. Optional TLAPS only for machine-checked safety proofs when scoped.

Do not require VS Code here. The broader TLA+ ecosystem may point new users toward the VS Code extension, and the Toolbox exists for legacy/tutorial workflows, but this local skill is CLI-first and editor-independent.

| Tool | Local role | Rule |
|------|------------|------|
| Java 11+ | Runtime for `tla2tools.jar` | Treat Java 11+ as baseline |
| `tla2tools.jar` / `tlc` | Required baseline checker | Record exact command and output |
| Any editor | Text editing only | No editor-specific dependency |
| Toolbox | Legacy only | Do not introduce for new work |
| TLAPS | Optional safety proof layer | Do not use as replacement for TLC model checking |
| Apalache | Optional bounded/symbolic defense | Only when obligation explicitly selects it |

Minimal CLI path:

```bash
java --version
java -cp <path-to-tla2tools.jar> tla2sany.SANY MySpec.tla
java -cp <path-to-tla2tools.jar> tlc2.TLC -config MySpec.cfg MySpec.tla
```

Jar alias form is acceptable when known to work:

```bash
java -jar <path-to-tla2tools.jar> -config MySpec.cfg MySpec.tla
```

If the project has a `tlc` executable or wrapper, use it and still record the exact command:

```bash
tlc -config MySpec.cfg MySpec.tla
```

## Starting Skeleton

Start with the smallest possible complete safety skeleton:

```tla
---- MODULE MySystem ----
EXTENDS Naturals, Sequences

VARIABLES x, y
vars == <<x, y>>

Init == /\ x = 0
        /\ y = <<>>

DoSomething == /\ x' = x + 1
               /\ y' = Append(y, x)

Next == DoSomething

TypeOK == /\ x \in Nat
          /\ y \in Seq(Nat)

Inv == Len(y) = x

Spec == Init /\ [][Next]_vars
====
```

This skeleton is intentionally incomplete for TLC because `Nat` and unbounded `Append` explode. The next step is always to replace the unbounded action and type invariant with bounded versions:

```tla
CONSTANTS MaxX

DoSomething == /\ x < MaxX
               /\ x' = x + 1
               /\ y' = Append(y, x)

Done == /\ x = MaxX
        /\ UNCHANGED <<x, y>>

Next == DoSomething \/ Done

TypeOK == /\ x \in 0..MaxX
          /\ y \in Seq(0..MaxX)

Inv == Len(y) = x
```

Basic config:

```cfg
SPECIFICATION Spec
CONSTANTS MaxX = 3
INVARIANT TypeOK
INVARIANT Inv
```

Do not leave infinite domains in a model you expect TLC to exhaust.

## PlusCal Versus Raw TLA+

Use PlusCal when the system is naturally algorithmic: mutual exclusion, producer-consumer logic, request-response protocols, phase machines, leader election, and executable protocol sketches.

Use raw TLA+ first when:

1. The spec should be very abstract.
2. You are modeling sets of messages, histories, leases, or possible environment actions.
3. You need refinement relations.
4. You want a clean math-first statement of the design contract.

PlusCal labels determine atomic steps. Atomicity is part of the model, not a formatting detail. When using PlusCal, commit the source algorithm and record the translation method. Do not hand-edit generated TLA+ without saying why.

## TLC Workflow

The practical loop is:

1. Choose abstraction boundary.
2. Write variables, constants, `vars`, and `Init`.
3. Decompose `Next` into named actions.
4. Write `TypeOK` and one semantic invariant.
5. Create `.cfg` with tiny finite constants.
6. Run SANY/TLC.
7. If TLC fails, read the trace before editing.
8. Shrink constants until the counterexample is minimal.
9. Fix one thing and rerun.
10. Add liveness/refinement/proofs only after safety stabilizes.

Basic `.cfg` using named `Spec`:

```cfg
SPECIFICATION Spec
INVARIANT TypeOK
INVARIANT Inv
```

Alternative `.cfg` using separate init/next:

```cfg
INIT Init
NEXT Next
INVARIANT TypeOK
```

Keep deadlock checking on unless terminal deadlock is intentional. If deadlock is intentional, encode a terminal stutter action or document why deadlock checking is disabled.

## Counterexample Discipline

A TLC counterexample is the main debugging artifact.

When TLC fails:

1. Identify the first surprising state, not the last state.
2. Name the failing action.
3. Restate the bug in plain English.
4. Decide whether the problem is in the model, property, config, or implementation design.
5. Shrink constants to a minimal reproducer.
6. Fix exactly one thing.
7. Rerun the same property.

Use reduced/diff traces when available, and export traces when the failure is important:

```bash
tlc -config MySpec.cfg -dumpTrace json trace.json MySpec.tla
```

Simulation mode is bug-finding only. It is not proof unless the obligation explicitly asks for simulation-only evidence. If using simulation, record depth, seed, and aril.

## Model-Checking Strategy

Use this order:

1. Type correctness: domains, shapes, finite bounds.
2. Core safety: no duplicate grant, at-most-one leader, coherence, monotonicity, ownership, no loss.
3. Action properties: every step preserves important structure.
4. Deadlock behavior: keep checking on unless there is a written reason.
5. Liveness: only after safety and enabledness are understood.
6. Refinement/proofs: last.

This order prevents the common failure mode where a model has fancy temporal formulas but no useful safety evidence.

## Testing Strategy For Real Projects

Every useful project model should pass these review tests:

| Test | Question |
|------|----------|
| Abstraction test | Can you explain what is omitted and why it is safe to omit? |
| Finiteness test | Are all sets, queues, histories, domains, and counters bounded? |
| Invariant test | Is `TypeOK` present and is there at least one domain-specific invariant? |
| Mutation test | If a guard or action is deliberately broken, does TLC catch it? |
| Trace quality test | Can a teammate explain the first bad step from the trace? |
| Scale-up test | Do constants grow one notch at a time? |
| Liveness test | Are fairness assumptions minimal and explicit? |

## Worked Miniatures

These examples teach patterns. Do not paste them into production without adding project-specific bounds, configs, and refinement maps.

### Consensus Abstraction

```tla
---- MODULE MiniConsensus ----
EXTENDS FiniteSets

CONSTANTS Value
ASSUME /\ Value # {}
       /\ IsFiniteSet(Value)

VARIABLE chosen
vars == <<chosen>>

TypeOK == chosen \subseteq Value

Init == chosen = {}

Choose == /\ chosen = {}
          /\ \E v \in Value : chosen' = {v}

Next == Choose
Spec == Init /\ [][Next]_vars

Inv == /\ TypeOK
       /\ Cardinality(chosen) <= 1

Success == <>(chosen # {})
LiveSpec == Spec /\ WF_vars(Choose)
====
```

What this teaches: model the contract before the protocol, prove at-most-one chosen value first, add liveness only after the safety model is stable. If `Value = {"A", "B"}`, breaking `Choose` so it can add a second value should produce a counterexample to `Inv`.

### Peterson-Style PlusCal Bridge

```tla
---- MODULE PetersonMini ----
EXTENDS Naturals, TLC

CONSTANTS Proc
ASSUME Proc = {0, 1}

(* --algorithm peterson
variables
    flag = [i \in Proc |-> FALSE],
    turn = 0;

process (P \in Proc)
variable me = self, other = 1 - self;
begin
Try:
    flag[me] := TRUE;
    turn := other;
Wait:
    while flag[other] /\ turn = other do
        skip;
    end while;
CS:
    assert ~flag[other] \/ turn = me;
Exit:
    flag[me] := FALSE;
    goto Try;
end process;
*)
====
```

After translation, check a generated-control-state property like:

```tla
MutualExclusion ==
    \A i, j \in Proc : (i /= j) => ~(pc[i] = "CS" /\ pc[j] = "CS")
```

What this teaches: PlusCal is useful for algorithm-shaped specs, labels define atomicity, and the generated `pc` variable becomes part of the state model.

### Cache Coherence Warning

A naive asynchronous bus model often violates this invariant transiently:

```tla
Coherence ==
    \A p, q \in Proc :
      \A a \in Addr :
        ((cache[p][a] # NoVal) /\ (cache[q][a] # NoVal)) =>
        (cache[p][a] = cache[q][a])
```

Black-hat rule: do not claim this invariant passes if a writer can update its cache while another processor still holds an older cached value. Either model invalidation/update as part of the write action, weaken the invariant to quiescent states, or add an explicit pending-bus invariant that describes allowed transient inconsistency.

Quiescent form:

```tla
QuiescentCoherence ==
    bus = <<>> => Coherence
```

What this teaches: write the semantic safety property first, but be honest about asynchrony. A strong invariant that fails may be the design bug you needed TLC to show.

## Learning Path

Use official and primary sources first, then community material.

| Stage | Goal | Exercises |
|-------|------|-----------|
| Foundations | States, actions, traces, invariants, `[Next]_vars` | One-variable counter with `TypeOK` and invariant |
| First PlusCal | Translation, labels, atomicity, configs | Bounded queue or lock with 2 processes |
| Raw TLA+ | Stop depending on PlusCal | Rewrite queue or lock directly |
| Safety/liveness | Invariants, deadlock, `WF`, `SF` | Add eventual response or starvation freedom |
| Abstraction/refinement | Relate levels | Abstract spec plus lower-level implementing spec |
| Real examples | Read realistic specs | Run official examples with tiny constants |
| Proofs | TLAPS safety proofs | Prove a tiny invariant, then mutual exclusion |

Exercise ladder:

1. Bounded counter.
2. Bounded queue or stack.
3. Reader-writer lock or semaphore.
4. Peterson or Dijkstra mutual exclusion.
5. Simple cache with coherence invariant.
6. Tiny choose-once consensus abstraction.
7. High-level Paxos skeleton.

Source priority:

1. Official TLA+ tools and TLC docs.
2. Lamport TLA+ materials and Specifying Systems.
3. Official PlusCal tutorial/manual and paper.
4. Official `tlaplus/Examples` corpus.
5. TLAPS docs for safety proofs.
6. Community explanations only after primary sources.

## Advanced Cautions

TLC checks finite models and only a practical subset of TLA+. Stay close to standard spec patterns until there is a reason not to.

Symmetry reduction is trusted input. It is useful for safety state-space control, but do not use symmetry for liveness unless the obligation explicitly accepts that limitation. Liveness under symmetry is a known danger zone.

Distributed TLC is for larger safety checks, not liveness evidence. Use normal TLC on a small model first.

State constraints can hide the bug. Prefer small meaningful constants over constraints that remove the very state you are claiming impossible.

TLAPS is strongest for safety proofs. Do not casually promise machine-checked temporal/liveness proofs.

## Black-Hat Review Checklist

Reject the model or mark it incomplete if any of these are true:

1. No `vars` tuple or inconsistent `vars` tuple.
2. Bare `[]Next` instead of `[][Next]_vars` without a written reason.
3. Missing `TypeOK`.
4. No semantic invariant beyond type correctness.
5. Infinite domains passed to TLC as if exhaustive checking were possible.
6. Action leaves a variable unconstrained instead of assigning it or using `UNCHANGED`.
7. Deadlock checking disabled without a written terminal-state rationale.
8. Liveness property exists without explicit fairness assumptions.
9. Fairness is attached to the wrong action or to an action with surprising `ENABLED` behavior.
10. Symmetry is used for liveness.
11. Random simulation is reported as proof.
12. Counterexample is ignored or hand-waved.
13. Model has no environment/failure actions: timeout, crash, cancellation, retry, duplicate message, partial commit, lost response.
14. No refinement map from model actions to runtime events/functions/storage.
15. TLC evidence omits command, config, constants, state counts, deadlock status, or liveness status.
