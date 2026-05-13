# Verus Harness

## Command Selection

Prefer commands in this order:

1. Exact `command` from `proof-obligations.jsonl`.
2. Repository script such as `./scripts/verify-verus.sh`.
3. Moon task such as `moon run :verify-proof` when it is documented for Verus.
4. Direct `verus <target>.rs` only when the target is known.

Do not invent targets, modules, proof names, or successful verifier output.

## Minimum Gate

```bash
command -v verus >/dev/null
verus --version
if command -v verusfmt >/dev/null; then verusfmt --check <verus-files-or-repo-path>; fi
verus <exact-target-or-approved-command>
rg -n 'assume\(|#\[verifier::external_body\]|#\[verifier::external\]|axiom' --glob '*.rs' --glob '!**/target/**'
```

If `verusfmt` is absent, record `VERUSFMT_MISSING` but do not treat formatting as proof evidence. If `verus` is absent and the obligation is required, report `BLOCKER`.

## Failure Enrichment

On verifier failure, rerun with the project's supported diagnostics. Common options include:

```bash
verus --expand-errors <target>.rs
VERUS_EXTRA_ARGS="--log-all" <repo-verus-test-command>
```

Use only options supported by the installed Verus version or repository harness. Capture last relevant diagnostics, not whole logs.

## Acceptance Criteria

- Verus command exits 0 for the scoped target.
- No unapproved `assume`, `external_body`, `external`, or axiomatic shortcut was introduced.
- Proof does not rely on broad global fuel/nonlinear context when a local proof suffices.
- Loop invariants, triggers, and reveals are local and maintainable.
- Evidence names exact command, exit status, verifier summary, and trusted-boundary scan result.

## Corpus Hygiene

For training or gold examples:

- Store accepted source, exact Verus version, command, stdout/stderr summary, and trust scan.
- Label examples using assumptions separately from assumption-free proofs.
- Keep repair traces: failing code, diagnostic, minimal fix, final verified code.
- Prefer verifier-accepted traces over hand-written snippets that were not executed.
