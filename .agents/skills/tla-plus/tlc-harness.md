# TLC Harness

Canonical CLI-first evidence rules live in `references/tlc-harness.md`. Use this file as a quick command/error snippet sheet only.

## Find TLC

```bash
if command -v tlc >/dev/null 2>&1; then
    TLC=(tlc)
elif command -v tla2tools >/dev/null 2>&1; then
    TLC=(tla2tools)
elif [ -n "${TLA2TOOLS_JAR:-}" ] && [ -f "$TLA2TOOLS_JAR" ]; then
    TLC=(java -cp "$TLA2TOOLS_JAR" tlc2.TLC)
else
    echo "BLOCKER: tlc not found"
    exit 1
fi

"${TLC[@]}" -version
```

## Run TLC

```bash
rm -rf states/
"${TLC[@]}" -config <spec>.cfg <spec>.tla
```

## .cfg Directives

| Directive | Purpose | Example |
|-----------|---------|---------|
| `SPECIFICATION` | Required — names the spec operator | `SPECIFICATION Spec` |
| `CONSTANTS` | Model values for constants | `RunId = {1, 2}` |
| `CONSTRAINT` | Name of a Boolean state predicate after each step | `CONSTRAINT JournalBound` |
| `STATE_CONSTRAINT` | Same; useful when diagnosing constraint parsing | `STATE_CONSTRAINT JournalBound` |
| `INVARIANT` | Safety property checked at every state | `TypeInvariant` |
| `DEADLOCK_DEFINE false` | Disable deadlock detection | `DEADLOCK_DEFINE false` |

### Constants as Model Values

```
CONSTANTS
    RunId = {1, 2}
    ShardId = {1, 2}
    Nil = Nil  \* model value for unassigned
```

Define inline bounds in the `.tla` module, not directly in the `.cfg`:

```tla
JournalBound == Len(journal) <= MaxJournalLen
```

```cfg
CONSTRAINT JournalBound
```

## Common Errors

### ConfigFileException: constant not assigned
```
Error: The constant parameter X is not assigned a value by the configuration file.
```
Fix: add `X = <model value>` to CONSTANTS in .cfg

### ConfigFileException: Len in CONSTRAINT
```
Error: The constraint of Len is equal to <Java Method>
```
Fix: define a named operator in the `.tla` module, then reference that name in `.cfg`:

```tla
JournalBound == Len(journal) <= 4
```

```cfg
CONSTRAINT JournalBound
```

### Parse error: not properly indented inside conjunction
```
Item at line N, col N is not properly indented inside conjunction
```
Fix: `/\ ` prefix must align at column of parent `/\`. `ELSE IF` must NOT have `/\ ` prefix.

### Semantic error: Unknown operator Nil
```
Unknown operator: `Nil'
```
Fix: `Nil` is not built-in. Use `0` or declare a model value such as `Nil = Nil` in .cfg CONSTANTS.

### Semantic error: cardinality on non-enumerable
```
Attempted to enumerate a set of the form [l1 : v1, ...]
but can't enumerate the value of the `slots' field: Nat
```
Fix: replace `Nat` with bounded set: `0..MaxSlotsPerRun`

### Underspecified action
```
Successor state is not completely specified by action X
The following variable is not assigned: shard_status
```
Fix: every action must assign ALL variables — use `UNCHANGED <<vars>>` for unchanging ones

### Deadlock reached
```
Error: Deadlock reached.
```
Fix: add termination action with `UNCHANGED vars`, or add `DEADLOCK_DEFINE false` to .cfg

## Reporting TLC Results

### Success
```
=== <spec>.tla ===
TLC: PASS
Command: "${TLC[@]}" -config <spec>.cfg <spec>.tla
States: <N> generated, <M> distinct, depth <D>
Finished: No error has been found.
```

### Failure
```
=== <spec>.tla ===
TLC: FAIL
Command: "${TLC[@]}" -config <spec>.cfg <spec>.tla
Error: <error type>
State: <N>
Trace: <steps to reproduce>
```

## Cleanup

TLC writes to `states/<timestamp>` and dies if it exists:
```bash
rm -rf states/
```
Always clean before running if running multiple specs.
