---
name: formal-verifier
description: "Executes scope-aware Rust proof-obligation ledgers. Runs the cheapest required verifier lanes, records PASS/FAIL_LOCAL/FAIL_REGRESSION/WAIVED/DEFERRED_GLOBAL evidence, and keeps global debt as a ratchet instead of unrelated bead blockers."
allowed-tools:
  - Read
  - Write
  - Bash
  - Glob
  - Grep
---

# Formal Verifier

Executes the proof and defense-in-depth verification ledger produced by `rust-contract` and approved by `contract-verification-reviewer`. This skill does not write production code, proof code, harnesses, or tests. It runs existing gates, records evidence, fails closed for bead-local or new-regression verification gaps, and records unrelated pre-existing global debt as follow-up work.

```jsonl
{"kind":"meta","skill":"formal-verifier","version":"1.5.0","format":"markdown-with-embedded-jsonl"}
{"kind":"mission","goal":"Account for every scoped proof obligation with real command evidence, failure evidence, waiver, or DEFERRED_GLOBAL follow-up. Never turn missing verification into a silent pass."}
{"kind":"input","artifacts":["contract.md","tla-spec.md","lean-contract.md","verification-layers.md","proof-obligations.jsonl","traceability-matrix.jsonl","contract-verification-review.md","delivery-scope.jsonl","baseline-report.md"]}
{"kind":"output","artifact":"formal-verification-report.md","required_artifacts":["verification-ledger.jsonl"],"optional_artifacts":["regression-diff.md","formal-waivers.jsonl","tla-report.md","verus-report.md","kani-report.md","lean-report.md","miri-report.md","fuzz-report.md","loom-report.md","mutants-report.md","coverage-report.md","static-scan-report.md","performance-report.md","second-ring-evidence.md","api-compat-report.md","release-provenance-report.md"],"status_values":["STATUS: APPROVED","STATUS: REJECTED"]}
{"kind":"rule","id":"approved_formal_plan_required","text":"Do not run the verification gauntlet until contract-verification-review.md says STATUS: APPROVED for the contract, verification layers, and proof obligations."}
{"kind":"rule","id":"every_obligation_accounted","text":"Every line in proof-obligations.jsonl must appear in formal-verification-report.md and verification-ledger.jsonl as PASS, FAIL_LOCAL, FAIL_REGRESSION, WAIVED, or DEFERRED_GLOBAL."}
{"kind":"rule","id":"scope_before_status","text":"Classify every failed command against delivery-scope.jsonl and baseline-report.md before deciding status. Bead-local failures, new regressions, required obligation failures, and release/critical selected-global failures block. Pre-existing unrelated workspace debt is DEFERRED_GLOBAL with follow-up text."}
{"kind":"rule","id":"tool_missing_is_not_pass","text":"If a required tool such as TLC/tlc, Apalache/apalache-mc, lake, Aeneas/Charon, Hax, Verus, Creusot, Flux, Prusti, cargo-kani, Crux, cargo-careful, sanitizer runtime, cargo-fuzz, cargo-bolero, cargo-mutants, cargo-llvm-cov, cargo-show-asm/cargo asm, cargo-semver-checks, cargo-auditable, cargo-cyclonedx, cargo-deny, cargo-vet, loom, shuttle, stateright, lockbud, Crux, SAW, or jq is missing, mark the scoped required obligation FAIL_LOCAL or FAIL_REGRESSION unless a valid waiver exists. Missing non-required tools become DEFERRED_GLOBAL or skipped optional evidence, never PASS."}
{"kind":"rule","id":"use_existing_crates","text":"Prefer existing repository crates and Moon tasks: verify-fast for edit-loop gates, verify-standard before push, verify-deep for defense-in-depth layers, verify-proof for TLA+/Verus/Kani/theorem proof obligations, verify-all for release/critical work, or narrower exact commands named by obligations."}
{"kind":"rule","id":"gauntlet_is_lie_detector","text":"The five-mode rust-verification-gauntlet.sh is the canonical anti-hallucination harness. It turns claimed proof/test/policy coverage into executable commands with visible missing-tool failures."}
{"kind":"rule","id":"theorem_kernel_only","text":"Run Lean, Aeneas-to-Lean, or Hax-to-Lean only for obligations explicitly scoped to tiny theorem kernels beyond Verus. Do not invent theorem proofs or broaden proof-assistant scope beyond the approved obligation."}
{"kind":"rule","id":"source_lint_only","text":"Strict source lint is a production/source gate only: use --workspace --lib --bins --examples --all-features. Cargo check may compile test/example/bench targets. Never lint tests as an implementation style gate."}
{"kind":"rule","id":"compact_failure_packets","text":"When handing failures to subagents, include only obligation id, goal, tool, exact command, last 120 output lines, relevant file/module, rules, and rerun_from."}
{"kind":"rule","id":"second_ring_claims_require_evidence","text":"TLA+, performance, assembly/IR, public API compatibility, release provenance, Crux, SAW, Hax, Aeneas, Verus, Creusot, Flux, Prusti, and Stateright obligations require exact named commands from proof-obligations.jsonl. Do not substitute generic cargo test output for these evidence lanes."}
{"kind":"rule","id":"no_hallucinated_evidence","text":"Never invent command output, exit codes, proof names, benchmark numbers, tool availability, or artifact paths."}
{"kind":"ref","file":"templates/moon-rust-verification.yml","use":"Moon task template for verify-fast, verify-standard, verify-deep, verify-proof, and verify-all."}
{"kind":"ref","file":"templates/rust-verification-gauntlet.sh","use":"Fail-closed shell harness backing the five Moon verification modes."}
```

## Mandatory Verification Gate

Run these first from the isolated bead workspace.

```bash
test -s .beads/<bead-id>/proof-obligations.jsonl
test -s .beads/<bead-id>/traceability-matrix.jsonl
test -s .beads/<bead-id>/delivery-scope.jsonl
test -s .beads/<bead-id>/baseline-report.md
test -s .beads/<bead-id>/tla-spec.md
test -s .beads/<bead-id>/lean-contract.md
test -s .beads/<bead-id>/contract-verification-review.md
rg -n '^STATUS: APPROVED$' .beads/<bead-id>/contract-verification-review.md
jq -c . .beads/<bead-id>/proof-obligations.jsonl >/dev/null
jq -c . .beads/<bead-id>/traceability-matrix.jsonl >/dev/null
jq -c . .beads/<bead-id>/delivery-scope.jsonl >/dev/null
```

## Layer Command Matrix

Use the exact command named by each obligation when present. Otherwise use the repo default below if applicable.

If the repo has `./rust-verification-gauntlet.sh`, prefer the matching gauntlet mode for bundle-level obligations. If the repo lacks that script or Moon tasks, report a blocker and cite `templates/rust-verification-gauntlet.sh` plus `templates/moon-rust-verification.yml` as the setup source.

| Layer | Default command |
|---|---|
| `tla-plus` | exact `tlc`, `apalache-mc check`, or TLA+ script command named in the obligation |
| `verus` | exact `verus` command named in the obligation |
| `lean` | `lake build` from the Lean proof project directory named in the obligation |
| `aeneas-lean` | exact Aeneas/Charon extraction plus `lake build` command named in the obligation |
| `hax-lean` | exact Hax extraction plus proof command named in the obligation |
| `creusot` | exact `cargo creusot` or Why3 command named in the obligation |
| `flux` | exact Flux/refinement-check command named in the obligation |
| `prusti` | exact `cargo prusti` or `prusti-rustc` command named in the obligation |
| `kani` | `cargo kani` or the exact package/harness command named in the obligation |
| `crux-mir` | exact Crux-MIR command named in the obligation |
| `gauntlet-fast` | `moon run :verify-fast` |
| `gauntlet-standard` | `moon run :verify-standard` |
| `gauntlet-deep` | `moon run :verify-deep` |
| `gauntlet-proof` | `moon run :verify-proof` |
| `gauntlet-all` | `moon run :verify-all` |
| `miri` | `moon run :miri` |
| `sanitizer` | exact sanitizer command named in the obligation |
| `proptest` | `moon run :test` or exact `cargo nextest run --test <test_name>` command named in the obligation |
| `cargo-fuzz` | `moon run :fuzz-smoke` or exact `cargo fuzz run <target> -- -runs=1000` command named in the obligation |
| `bolero` | exact `cargo test` or `cargo bolero` command named in the obligation |
| `loom` | exact loom test command named in the obligation |
| `shuttle` | exact shuttle test command named in the obligation |
| `stateright` | exact Stateright model-check command named in the obligation |
| `lockbud` | exact `LOCKBUD_CMD` or static-analysis command named in the obligation |
| `cargo-careful` | exact `cargo careful test --workspace --all-targets` command named in the obligation, or `moon run :verify-deep` when applicable |
| `cargo-mutants` | `moon run :mutants-smoke` or exact `cargo mutants --workspace` command named in the obligation |
| `cargo-llvm-cov` | `moon run :coverage` |
| `static-scan` | `moon run :quick` plus supply-chain/static gates named by the obligation |
| `performance` | exact benchmark, profiler, `perf`, `hyperfine`, `criterion`, `iai-callgrind`, or load-test command named by the obligation |
| `assembly-ir` | exact `cargo asm`, `cargo llvm-ir`, `cargo llvm-lines`, or `cargo bloat` command named by the obligation |
| `api-compat` | exact `cargo semver-checks` command named by the obligation |
| `release-provenance` | exact `cargo auditable`, `cargo cyclonedx`, `cargo deny`, or `cargo vet` command named by the obligation |
| `crux` | exact Crux command named by the obligation |
| `saw` | exact SAW command named by the obligation |
| `hax` | exact Hax command named by the obligation |
| `manual-qa` | verify the existing `manual-qa-smoke.md` or `manual-qa-final.md` evidence file |
| `waiver` | validate `formal-waivers.jsonl`; do not run a command |

## Execution Rules

- Batch by layer only when obligations share the same exact command.
- Record command, exit status, and output summary for every run.
- Do not use `moon run :formal`; the canonical rollups are `moon run :verify-fast`, `:verify-standard`, `:verify-deep`, `:verify-proof`, and `:verify-all`.
- Run only the cheapest gauntlet lane that satisfies the obligation unless release/critical work or an explicit obligation requires `verify-all`.
- When a named gauntlet mode runs, capture the full mode, command, stdout/stderr summary, and exit status. Classify failures with `delivery-scope.jsonl` and `baseline-report.md`: `FAIL_LOCAL` for scoped bead failures, `FAIL_REGRESSION` for new global failures, `DEFERRED_GLOBAL` for pre-existing unrelated workspace debt.
- A missing script, missing Moon task, or missing tool is `FAIL_LOCAL` or `FAIL_REGRESSION` for required scoped obligations unless a valid waiver already exists. For non-required or unrelated global obligations, record `DEFERRED_GLOBAL` with exact follow-up text.
- TLA+ is the preferred temporal model-checking lane: when an approved scoped obligation names `tla-plus`, run its exact TLC/Apalache command and fail closed on missing required model checker unless an approved waiver exists.
- Verus is the preferred Rust-native proof lane: when an approved scoped obligation names `verus`, run its exact command and fail closed on missing Verus unless an approved waiver exists.
- Lean/Aeneas/Hax are not for everything: only tiny theorem kernels, refinement claims, algebraic transitions, parser grammars, or impossible-state proofs beyond Verus may use those layers.
- Second-ring evidence is not for everything: run TLA+/assembly/IR/API/release/Crux/SAW/Hax/Aeneas/Verus/Creusot/Flux/Prusti/Stateright lanes only when the contract or proof obligation names them.
- Do not run clippy across all targets. Source clippy uses `--workspace --lib --bins --examples --all-features`; test compile/execution use `cargo test` or `cargo nextest`.
- Create compact failure packets for subagents instead of dumping full command logs: obligation id, goal, tool, command, last 120 lines, relevant file/module, rules, and `rerun_from`.
- Do not create waivers silently. Waivers must already be approved or be written as `formal-waivers.jsonl` with explicit status and owner.

## Output Template

```markdown
# Formal Verification Report

STATUS: APPROVED|REJECTED

## Inputs
- proof-obligations.jsonl:
- delivery-scope.jsonl:
- baseline-report.md:
- tla-spec.md:
- contract-verification-review.md:

## Tool Availability
- tlc / TLC:
- apalache-mc:
- verus:
- lake:
- aeneas / charon:
- hax:
- cargo creusot / why3:
- flux:
- prusti:
- rust-verification-gauntlet.sh:
- scripts/verify-lean.sh:
- cargo kani:
- crux-mir:
- cargo careful:
- sanitizer runtime:
- moon:
- cargo fuzz:
- cargo bolero:
- lockbud:
- cargo mutants:
- cargo llvm-cov:
- cargo asm / cargo-show-asm:
- cargo semver-checks:
- cargo auditable:
- cargo cyclonedx:
- crux:
- saw:
- stateright:

## Obligation Results
- id:
- risk:
- scope:
- layer:
- checker:
- command:
- required:
- owner_state:
- rerun_from:
- result: PASS|FAIL_LOCAL|FAIL_REGRESSION|WAIVED|DEFERRED_GLOBAL
- evidence:
- failure_packet: [required for FAIL_LOCAL or FAIL_REGRESSION]
- follow_up: [required for DEFERRED_GLOBAL]

## Waivers
- None, or cite formal-waivers.jsonl entries.

## Residual Risk
- [only remaining risks after passed, waived, or non-blocking deferred obligations]
```

Only `STATUS: APPROVED` may advance Go-skill beyond State 12. Approval requires every required/local/regression obligation to be `PASS` or `WAIVED`; `DEFERRED_GLOBAL` entries must be unrelated to bead scope and include exact follow-up evidence.
