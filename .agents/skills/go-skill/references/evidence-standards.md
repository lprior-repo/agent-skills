# Evidence Standards

Proof by prose is invalid. Every PASS needs raw command evidence.

## Command Evidence

Record exact command, workdir, exit status, tool version, flags, bounds, model constants, seeds, target filters, stdout/stderr summary, raw log path, evidence artifact path, and scope classification.

## Pending Execution

`PENDING_FORMAL_EXECUTION` is only for expensive deep runs. Every proof/model/harness artifact needs cheap syntax/typecheck/smoke evidence before proof review approval unless tooling blocks forward progress.

## Closure

State 12 must close every required obligation as `PASS`, `FAIL_LOCAL`, `FAIL_REGRESSION`, `FAIL_GLOBAL`, or valid non-behavior `WAIVED`. Pending execution, planned mappings, and pending trusted-base dispositions fail closure.
