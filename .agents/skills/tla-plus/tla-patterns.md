# TLA+ Patterns

Canonical guidance lives in `references/effective-tla-practice.md` and `references/tla-patterns.md`. Use this file as a quick snippet sheet only.

## Minimal Complete Spec

```
---- MODULE MySpec ----
EXTENDS Integers, Sequences, FiniteSets, TLC

CONSTANT RunId, ShardId  \* model values in .cfg

VARIABLES run_owner, shard_runs

Init ==
    /\ run_owner = [run \in RunId |-> 0]
    /\ shard_runs = [shard \in ShardId |-> {}]

Next ==
    \/ \E run \in RunId, shard \in ShardId : AssignShard(run, shard)

Spec == Init /\ [][Next]_<<run_owner, shard_runs>>

THEOREM Spec => []TypeInvariant
====
```

## .cfg for Above

```
SPECIFICATION Spec
CONSTANTS
    RunId = {1, 2}
    ShardId = {1, 2}
```

## State Machine with Terminal States

```
TerminalStates == {Succeeded, Failed, Cancelled}

ValidTransition(src, dst) ==
    \/ src = Pending /\ dst = Running
    \/ src = Running /\ dst \in TerminalStates \cup {Waiting, Asking}
    \/ src = dst  \* idempotent re-mark

TypeInvariant ==
    step_state \in {Pending, Running, Succeeded, Failed, Skipped, Waiting, Asking, Cancelled}
```

## Bounded Queue

```
MAX_QUEUE == 65536

QueueBounded ==
    \A s \in ShardId : Len(queues[s]) <= MAX_QUEUE
```

## Bounded Journal with Termination

```
Init ==
    /\ journal = <<>>
    /\ replay_index = 1

ReplayComplete ==
    replay_index > Len(journal)
    /\ UNCHANGED <<journal, replay_index>>

Spec == Init /\ [][Next \/ ReplayComplete]_<<journal, replay_index>>
```

## Nested EXCEPT for 2D State

```
\* WRONG: latest_attempt[run][step] = attempt
latest_attempt = [<<run, step>> \in (RunId \X StepId) |-> -1]
latest_attempt' = [latest_attempt EXCEPT ![<<run, step>>] = attempt]
```

## Set Cardinality

```
Cardinality(shard_runs[shard]) <= MaxRunsPerShard  \* CORRECT
```

## Anti-Patterns

### Unbounded append without termination
```
journal' = Append(journal, event)  \* grows forever
```
Fix: add a terminal stutter action, or define a named bound operator in `.tla` and reference it from `.cfg`:
```tla
JournalBound == Len(journal) <= MaxJournalLen
```
```cfg
CONSTRAINT JournalBound
```

### Forward reference
```
Spec == Init /\ [][Next]_vars
Next == ...  \* defined AFTER — ERROR
```

### Else-if inside conjunction
```
/\ IF x = 0 THEN A
ELSE IF x = 1 THEN B
```
Fix: use `ELSE IF` continuation (no leading `/\ `), or rewrite multi-way branching as `CASE`.

### Inline cfg constraints
```
CONSTRAINT Len(journal) <= 4  \* WRONG in .cfg
```
Fix: define a named operator in `.tla`, then reference it from `.cfg`:
```tla
JournalBound == Len(journal) <= 4
```
```cfg
CONSTRAINT JournalBound
```

### THEOREM with primed variables
```
THEOREM Spec => []TickOneCommand  \* TickOneCommand has Len(q)' — ERROR
THEOREM Spec => [][TickOneCommand]_vars  \* CORRECT
```
