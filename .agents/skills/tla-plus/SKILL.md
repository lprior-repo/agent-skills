---
name: tla-plus
description: "TLA+ and TLC model-checking skill for writing, reviewing, and repairing temporal specs, PlusCal models, .tla/.cfg files, invariants, liveness/fairness properties, deadlock checks, counterexample traces, optional Apalache symbolic checks, and Rust proof-obligation evidence. Use for TLA+, TLC, PlusCal, tla2tools.jar, Apalache, temporal properties, state machines, protocols, schedulers, queues, retries, leases, and distributed or concurrent workflows."
argument-hint: "[spec_file] [command]"
allowed-tools:
  - Read
  - Write
  - Edit
  - Bash
  - Glob
  - Grep
---

# TLA+ Temporal Model Engineer

TLA+ is the default design model for temporal and state-over-time behavior. TLC proves finite bounded models by exhaustive reachable-state exploration. Treat TLA+ output as evidence only when the exact model, config, bounds, properties, and command are recorded.

```jsonl
{"kind":"meta","skill":"tla-plus","version":"1.1.0","format":"markdown-with-embedded-jsonl"}
{"kind":"mission","goal":"Write, review, and repair TLA+/PlusCal models with TLC-first evidence, optional Apalache bounded evidence, and no hidden bounds, liveness caveats, or hallucinated model-checker output."}
{"kind":"scope","owns":["TLA+ specs","PlusCal models",".cfg model configs","Init/Next behavior specs","state variables","actions","safety invariants","temporal properties","fairness/liveness","deadlock checks","finite model bounds","TLC commands","optional Apalache commands","counterexample trace interpretation","Rust/runtime refinement maps"]}
{"kind":"scope","does_not_own":["Rust-local pure proof bodies owned by Verus","tiny theorem kernels owned by Lean/Aeneas/Hax","implementation interleaving tests owned by Loom/Shuttle/Stateright/Lockbud","test helper/loop/table-style judgments","inventing model-checker output"]}
{"kind":"rule","id":"temporal_default","text":"Use TLA+ by default for workflows, protocols, schedulers, queues, retries, claim/lease logic, lifecycle transitions, distributed coordination, concurrency protocols, eventuality, fairness, and deadlock freedom."}
{"kind":"rule","id":"cli_first_local_workflow","text":"In this environment, do not make VS Code or Toolbox part of the operating path. Use editor-agnostic files plus command-line Java/tla2tools/TLC. Mention VS Code only as external ecosystem context when asked; treat Toolbox as legacy/unmaintained."}
{"kind":"rule","id":"canonical_mental_model","text":"A state assigns values to variables; a behavior is a sequence of states; a state predicate talks about one state; an action relates old unprimed variables to new primed variables; the default complete safety spec is Init /\\ [][Next]_vars."}
{"kind":"rule","id":"safety_first_practice","text":"Model the design before code, start with a tiny finite model, write TypeOK plus one strong semantic invariant first, let TLC find counterexamples early, and add fairness/liveness only after safety and deadlock behavior are stable."}
{"kind":"rule","id":"behavior_shape","text":"Every model must name variables, constants, Init, action predicates, Next, vars, invariants, temporal properties, deadlock stance, fairness stance, finite bounds, and refinement relation to runtime events."}
{"kind":"rule","id":"tlc_evidence","text":"TLC evidence must include exact command, module, config, constants, invariants/properties checked, states generated, distinct states, diameter when reported, deadlock status, liveness status, and counterexample trace path when failing."}
{"kind":"rule","id":"apalache_optional_defense","text":"Apalache is optional bounded/symbolic defense-in-depth. It is useful for invariant counterexamples and type discipline, but it does not replace TLC for exhaustive finite-state checks or liveness/fairness evidence. Baseline TLC evidence still stands when Apalache is unavailable; only obligations that explicitly select apalache-mc require it."}
{"kind":"rule","id":"finite_model_bounds","text":"State constraints, action constraints, constants, model values, worker count, symmetry, and simulation depth are part of the proof context. Record them because tight bounds can hide bugs."}
{"kind":"rule","id":"symmetry_is_trusted","text":"Symmetry reduction is user-supplied trust. TLC does not prove symmetry soundness. Do not use symmetry when checking liveness unless the obligation explicitly accepts that limitation."}
{"kind":"rule","id":"liveness_caveat","text":"Liveness and fairness need a temporal behavior spec. Distributed TLC and some reductions do not support liveness; simulation is bug-finding only, not proof."}
{"kind":"rule","id":"trace_first_debugging","text":"On TLC failure, read the shortest counterexample trace before changing the model. Prefer JSON trace export when available for reproducible diagnosis."}
{"kind":"rule","id":"no_hallucinated_evidence","text":"Never invent TLC, Apalache, SANY, Toolbox, TLAPS, state counts, trace steps, deadlock status, temporal success, or tool availability."}
{"kind":"rule","id":"pipeline_boundary","text":"In the Rust proof stack, TLA+ owns temporal workflow/protocol behavior. Verus owns Rust-local pure/core invariants. Lean/Aeneas/Hax own tiny theorem kernels beyond Verus. Kani/Miri/fuzz/Loom/Stateright and related tools are risk-selected implementation evidence."}
{"kind":"ref","file":"references/effective-tla-practice.md","use":"CLI-first practical TLA+ guide: core concepts, canonical spec shape, safety-first workflow, PlusCal/raw TLA+ choice, counterexample debugging, learning path, examples, and black-hat review checklist."}
{"kind":"ref","file":"references/tla-patterns.md","use":"Spec shape, modeling idioms, temporal properties, refinement maps, and anti-patterns."}
{"kind":"ref","file":"references/tlc-harness.md","use":"TLC-first command selection, optional Apalache bounded evidence, trace export, tool availability, and failure triage."}
{"kind":"ref","file":"references/tla-curriculum.md","use":"Source priority, staged learning path, and evaluation tasks for TLA+/TLC work."}
```

## Mandatory Verification Gate

Run the exact TLA+ command named in `proof-obligations.planned.jsonl` or `verification-ledger.jsonl` when present. If no exact command exists, use the nearest repo script/task. If no model target exists, report a blocker instead of fabricating evidence.

Template only; replace placeholders with exact project paths before treating output as evidence.

```bash
command -v java >/dev/null
java --version
if command -v tlc >/dev/null; then tlc -version || true; fi
if command -v apalache-mc >/dev/null; then apalache-mc version || true; fi
if test -x ./scripts/verify-tla.sh; then ./scripts/verify-tla.sh; else true; fi
tlc -config <model.cfg> <module.tla>
```

If `tlc` is absent, an equivalent Java command is acceptable only when the exact jar path is known:

```bash
java -cp <path-to-tla2tools.jar> tlc2.TLC -config <model.cfg> <module.tla>
java -jar <path-to-tla2tools.jar> -config <model.cfg> <module.tla>
```

Required TLA+ baseline obligation with missing TLC, missing `tla2tools.jar`, unknown module, or unknown config is `BLOCKER`, not pass. Apalache availability is irrelevant to baseline TLA+ success unless the obligation explicitly names `apalache-mc` or symbolic bounded checking.

## Workflow

1. Read `references/effective-tla-practice.md`, `references/tla-patterns.md`, and `references/tlc-harness.md` before editing or judging a model.
2. Classify scope: workflow, protocol, scheduler, queue, retry, lease, lifecycle, distributed coordination, concurrency, or non-temporal.
3. Define behavior boundary: variables, constants, `Init`, actions, `Next`, `vars`, model bounds, and Rust/runtime refinement map.
4. Encode safety first: invariants, type invariants, mutual exclusion, ownership, monotonicity, conservation, no-loss, and deadlock stance.
5. Encode liveness only with explicit fairness and temporal behavior spec.
6. Run exact command-line TLC command and capture evidence; run Apalache only when the obligation explicitly names bounded/symbolic checking.
7. Interpret counterexample trace before changing model or implementation.
8. Report bounded evidence, limitations, trusted reductions, and residual obligations.

## Output Contract

When writing, reviewing, or repairing TLA+ work, return:

- Files changed.
- Exact TLC/SANY commands run and results, plus Apalache result when explicitly selected.
- Model bounds, constants, constraints, and symmetry settings.
- Invariants and temporal properties checked.
- Counterexample trace summary or success evidence.
- Runtime/Rust refinement relation.
- Trusted reductions, liveness limitations, and blockers.
