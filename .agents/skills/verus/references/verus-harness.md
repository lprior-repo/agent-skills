# Verus Harness

## Command Selection

Prefer commands in this order:

1. Exact `command` from `proof-obligations.planned.jsonl`, `verification-ledger.jsonl`, or equivalent project proof ledger.
2. Repository script such as `./scripts/verify-verus.sh`, `moon run :verify-proof`, or project-specific Verus task.
3. `cargo verus verify` for opted-in Cargo crates/workspaces.
4. Direct `verus <target>.rs` only when the target is the agreed proof unit.

Do not invent targets, modules, proof names, lemma names, successful verifier output, or command summaries.

## Cargo-Verus Rules

- `cargo verus verify` verifies all selected crates with `[package.metadata.verus] verify = true` and their verified dependencies. This is normal final closure for Cargo-integrated projects.
- `cargo verus focus` is for iteration; it skips re-verification of dependencies and is not final proof closure unless a project ledger explicitly accepts that scope.
- `cargo verus build` verifies and compiles artifacts. Use it when the deliverable is a verified binary/library.
- Arguments after `--` are forwarded to Verus invocations. Avoid project-specific `--verify-module` or `--verify-function` unless the project documents the exact target.
- For `cargo verus verify`, confirm the changed crate is opted in with `[package.metadata.verus] verify = true` or that command output proves the intended crate was verified; the “no crates opted into verification” warning is not proof evidence.

## Minimum Gate

```bash
command -v verus >/dev/null
verus --version
if command -v verusfmt >/dev/null; then verusfmt --check <verus-files-or-repo-path>; fi
verus --no-cheating <exact-target.rs>
# Run the canonical trust-marker scan from references/trust-marker-scan.md with host-native search or parser/token scan; use rg only as an allowed fallback.
```

If Cargo-Verus is the project harness:

```bash
cargo verus verify <scoped-target> -- --no-cheating <project-approved-verus-args>
```

If `verusfmt` is absent, record `VERUSFMT_MISSING` but do not treat formatting as proof evidence. If `verus` is absent and the obligation is required, report `BLOCKER`.
Where the installed Verus supports it, run final proof commands with `--no-cheating` or record why the project cannot. `--no-cheating` is not a replacement for a diff-based trust scan; it is an additional guard.
Use `references/trust-marker-scan.md` as the canonical scan pattern and scan-scope rule. Host-native search or parser/token-tree scans are preferred; `rg` is only a fallback adapter.

## Failure Enrichment

On verifier failure, rerun with the project's supported diagnostics:

```bash
verus --expand-errors <target>.rs
VERUS_EXTRA_ARGS="--log-all" <repo-verus-test-command>
cargo verus verify -p <crate> -- --expand-errors
```

Use only options supported by the installed Verus version or repository harness. Capture relevant diagnostics, not whole logs. For Verus repository tests, `VERUS_EXTRA_ARGS="--log-all" vargo test ...` writes VIR/AIR/SMT logs; run one test at a time to avoid log overwrite.

## Cheat Checker

Before approval or final success claims, scan both changed proof-relevant files and the full verifier trust closure for:

- `assume(` and `admit(`
- `assume_specification` and `#[verifier::assume_termination]`
- `#[verifier::external_body]`, `#[verifier(external_body)]`, `#[verifier::external]`, `#[verifier(external)]`, and `#[verus_verify(external_body|external)]`
- `external_fn_specification`, `external_type_specification`, `external_trait_specification`, and `external_trait_extension`
- `exec_spec_unverified!` generated code, which is `TRUSTED` unless separately proved
- `broadcast axiom` or other axiom declarations
- `cfg(verus_only)` blocks that affect executable behavior
- public verified APIs with unchecked preconditions exposed to unverified Rust; use `-V check-safe-api` where the project treats this as required evidence
- contract deltas in `requires`, `ensures`, invariants, decreases, and spec visibility

Trust markers are not always illegal, but they must be explicit boundaries with owner/reason/scope/expiry and final claims must say `TRUSTED`, not `PROVED`.

## Acceptance Criteria

- Exact Verus command exits 0 for the scoped target.
- Final closure command is not iteration-only unless explicitly scoped by the obligation.
- No unapproved `assume`, `admit`, unchecked external specification, axiom, trusted attribute, `exec_spec_unverified!`, or `verus_only` runtime fork was introduced.
- Proof constrains the actual production symbol or accepted extracted helper.
- Contract shape did not launder the obligation.
- Proof does not rely on broad global fuel/nonlinear/rlimit context when a local proof suffices.
- Loop invariants, triggers, reveals, type-invariant uses, and solver modes are local and maintainable.
- Evidence names exact command, exit status, verifier summary, warnings/notes, and trusted-boundary scan result.
- Where supported, final evidence either uses `--no-cheating` or explains why a required, ledgered trust boundary prevents that flag.

## Corpus Hygiene

For training or gold examples:

- Store accepted source, exact Verus version, command, stdout/stderr summary, and trust scan.
- Label examples using assumptions separately from assumption-free proofs.
- Keep repair traces: failing code, diagnostic, minimal fix, final verified code.
- Prefer verifier-accepted traces over hand-written snippets that were not executed.
- Keep source/proof mappings to actual symbols, not just files.
