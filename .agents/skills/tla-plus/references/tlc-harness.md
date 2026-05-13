# TLC Harness And Evidence

## Command Selection

Use commands in this order:

1. Exact command from `.beads/<bead-id>/proof-obligations.jsonl`.
2. Repo script `./scripts/verify-tla.sh` if it exists and covers the obligation.
3. Moon task `moon run :verify-proof` or narrower task named by the repo.
4. Direct TLC command when module/config paths are known.
5. Direct Apalache command only when obligation explicitly names bounded/symbolic checking.

Do not invent module names, config names, jar paths, state counts, or successful output.

This environment is CLI-first. Do not require VS Code or Toolbox to parse, translate, or model-check. A GUI may be useful elsewhere, but local evidence comes from command-line SANY/TLC output.

## Tool Availability

Minimum tool check:

```bash
command -v java >/dev/null
java --version
if command -v tlc >/dev/null; then tlc -version || true; fi
if command -v apalache-mc >/dev/null; then apalache-mc version || true; fi
```

Treat Java 11+ as the practical baseline for current stable TLC usage. If Java is absent or too old for the selected `tla2tools.jar`, report a blocker for required TLA+ evidence.

Portable TLC discovery helper:

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

Known jar path check:

```bash
java -cp <path-to-tla2tools.jar> tla2sany.SANY <module.tla>
java -cp <path-to-tla2tools.jar> tlc2.TLC -config <model.cfg> <module.tla>
```

Jar alias check:

```bash
java -jar <path-to-tla2tools.jar> -config <model.cfg> <module.tla>
```

Missing required TLC, missing `tla2tools.jar`, or unknown target is a blocker unless an approved waiver exists. Apalache availability matters only for obligations that explicitly name `apalache-mc` or bounded/symbolic checking.

## Apalache Is Optional Defense-In-Depth

Apalache is not the baseline TLA+ gate. Use it when an obligation asks for bounded or symbolic invariant checking, type discipline, or a second checker for a finite model.

Do not use Apalache as replacement evidence for TLC liveness or fairness. Apalache may ignore fairness constraints, so any liveness/fairness claim must be carried by TLC or another checker explicitly approved for that property.

## Minimum Model Check Gate

For a required TLA+ obligation, capture:

- command;
- exit status;
- module path;
- config path;
- constants and model values;
- invariants checked;
- temporal properties checked;
- deadlock setting;
- worker count;
- state constraints and action constraints;
- symmetry and view settings;
- generated states;
- distinct states;
- diameter when reported;
- trace path if failed;
- exact failure class: parse error, type/config error, invariant violation, deadlock, liveness violation, state explosion, missing tool.

## Trace Export

When a counterexample matters, prefer reproducible trace output:

```bash
tlc -config <model.cfg> -dumpTrace json <trace.json> <module.tla>
```

If using the Java jar form:

```bash
java -cp <path-to-tla2tools.jar> tlc2.TLC -config <model.cfg> -dumpTrace json <trace.json> <module.tla>
```

Read the trace before changing the model. Most fixes should change the spec, invariant, bound, or implementation mapping only after the trace explains the actual behavior.

## Simulation Is Not Proof

Simulation generates random behaviors. It is useful for quick bug finding and smoke checks. It does not exhaust the reachable state graph and must not satisfy required proof obligations unless the obligation explicitly asks for simulation-only evidence.

## Liveness Caveats

Liveness and fairness checks need a temporal spec such as `Init /\ [][Next]_vars /\ WF_vars(Action)`. Do not claim liveness from safety-only checking.

Avoid symmetry with liveness. Distributed TLC does not check liveness. If liveness is required, check a smaller non-distributed model first.

## Distributed And Profiling Caveats

Distributed TLC is for larger safety checks. It can increase coverage for big state spaces but lacks liveness checking, depth-first mode, simulation mode, and coverage details. Use normal TLC on a small model before distributed mode.

Profiling helps diagnose disabled actions, expensive expressions, and state explosion. It slows large checks, so use it when diagnosing, not as default evidence.

## Failure Triage

Parse/config error:
- Run SANY and fix syntax/config names first.

Invariant violation:
- Read shortest trace, identify bad action, then decide whether model or implementation contract is wrong.

Deadlock:
- Decide whether terminal deadlock is intended. If intended, encode terminal condition or disable deadlock with written rationale.

Liveness violation:
- Check fairness assumptions, stuttering, enabledness, symmetry, and constraints.

State explosion:
- Reduce constants, add safe symmetry for safety-only checks, add `VIEW`, improve model structure, or split obligations.

Missing tool:
- Report blocker for required obligation, or `DEFERRED_GLOBAL` only when obligation is non-required and unrelated to current bead scope.

Broken example/spec claim:
- If a pedagogical example violates its own invariant, say so. Do not preserve a broken model as authoritative guidance; either mark it as a bug-finding example or repair the invariant/action semantics.
