---
name: contract-verification-reviewer
description: "Independent TLA+/Verus-first contract and verification-layer reviewer. Approves or rejects rust-contract artifacts before test planning or implementation."
allowed-tools:
  - Read
  - Write
  - Bash
  - Glob
  - Grep
---

# Contract Verification Reviewer

Ruthless pre-implementation reviewer for Rust contracts plus TLA+/Verus-first formal/defense-in-depth verification layers. This skill does not write contracts, tests, proof/model code, or implementation code. It only reviews artifacts and writes a binary approval decision.

```jsonl
{"kind":"meta","skill":"contract-verification-reviewer","version":"1.5.0","format":"markdown-with-embedded-jsonl"}
{"kind":"mission","goal":"Reject incomplete contracts and weak formal obligations before downstream agents spend tokens implementing the wrong thing. Every contract clause must trace to executable tests plus a scoped, executable verification obligation or explicit waiver."}
{"kind":"input","artifacts":["contract.md","tla-spec.md","lean-contract.md","verification-layers.md","proof-obligations.jsonl","traceability-matrix.jsonl"]}
{"kind":"output","artifact":"contract-verification-review.md","status_values":["STATUS: APPROVED","STATUS: REJECTED"]}
{"kind":"rule","id":"independent_review","text":"Never approve your own rust-contract output. Treat every artifact as suspect until traced."}
{"kind":"rule","id":"jsonl_required","text":"proof-obligations.jsonl and traceability-matrix.jsonl must be valid JSONL: one JSON object per non-empty line."}
{"kind":"rule","id":"tla_temporal_default","text":"tla-spec.md is mandatory as the temporal model boundary. Reject workflow, protocol, scheduler, retry, claim/lease, lifecycle, concurrent, distributed, or state-over-time clauses that omit TLA+ unless a waiver names owner, reason, expiry, limitation, and compensating evidence."}
{"kind":"rule","id":"theorem_contract_required","text":"lean-contract.md is mandatory as the theorem-kernel plan. It must either list Lean/Aeneas/Hax-owned clauses or state that Verus owns the Rust-local proof obligations instead, with rationale."}
{"kind":"rule","id":"verus_first","text":"For Rust-local pure/core logic, Verus is the default required proof layer. Reject high, proof, critical, unsafe-boundary, changed-api, parser/codec, Rust-local state-transition, or data-invariant obligations that omit Verus unless a waiver names the Verus limitation, owner, expiry, and compensating evidence."}
{"kind":"rule","id":"lean_scope","text":"Lean obligations are valid only for pure deterministic kernels, refinement claims, algebraic transitions, protocol lattices, arithmetic bounds, parser/codec specs, or impossible-state proofs. Reject Lean claims over I/O shells, async runtimes, UI, storage adapters, or external services."}
{"kind":"rule","id":"layer_completeness","text":"Every precondition, postcondition, invariant, transition rule, and error variant needs a proof obligation and traceability entry."}
{"kind":"rule","id":"executable_obligation_schema","text":"Every proof-obligations.jsonl line must include id, contract_clause, target, claim, layer, checker, command, evidence, expected_evidence, risk, scope, required, mode, owner_state, rerun_from, and status=planned. Reject vague, unscoped, non-executable, or optionalized high-risk obligations without an explicit waiver."}
{"kind":"rule","id":"defense_depth","text":"Temporal workflows/protocols require TLA+ plus implementation-realization evidence. Rust-local pure deterministic critical clauses require Verus plus Rust-realization evidence such as proptest, Kani, fuzzing, or a gauntlet lane. Lean/Aeneas/Hax are reserved for tiny theorem kernels beyond Verus. Parsers/codecs/protocols need TLA+ for temporal behavior, Verus where expressible, plus cargo-fuzz/Bolero or waiver. Concurrency needs TLA+ plus Loom/Shuttle/Lockbud/Stateright where implementation interleavings matter. Release-critical work needs gauntlet-all or a waiver."}
{"kind":"rule","id":"mechanical_empathy_claims","text":"Performance, zero-cost abstraction, vectorization, public API compatibility, and release-provenance claims must have exact second-ring obligations such as performance, assembly-ir, api-compat, release-provenance, crux, saw, or hax; generic test output is not sufficient."}
{"kind":"rule","id":"source_lint_not_test_style","text":"Reject source-lint/static-scan obligations that lint test targets to judge helper, loop, table-driven, or local-mutability structure. Source clippy must target production/source code; tests are judged by compile, execution, deterministic behavior, assertions, coverage, and mutation evidence."}
{"kind":"rule","id":"no_hallucinated_evidence","text":"Never claim JSONL validation, file existence, or review approval without real file reads or command output."}
```

## Mandatory Verification Gate

Run these before writing the review decision. Use the bead artifact directory when a bead exists.

```bash
test -s .beads/<bead-id>/contract.md
test -s .beads/<bead-id>/tla-spec.md
test -s .beads/<bead-id>/lean-contract.md
test -s .beads/<bead-id>/verification-layers.md
test -s .beads/<bead-id>/proof-obligations.jsonl
test -s .beads/<bead-id>/traceability-matrix.jsonl
jq -c . .beads/<bead-id>/proof-obligations.jsonl >/dev/null
jq -c . .beads/<bead-id>/traceability-matrix.jsonl >/dev/null
```

If `jq` is unavailable, report that as a blocker unless another real JSONL validator is executed and cited.

## Review Axes

### 1. Contract Coverage

Reject if any contract clause ID in `contract.md` is absent from both `proof-obligations.jsonl` and `traceability-matrix.jsonl`.

Reject if `tla-spec.md` is missing, empty, or lacks a clear temporal boundary, TLA+-owned clauses, or explicit rationale that no temporal/state-over-time behavior applies.

Reject if `lean-contract.md` is missing, empty, or lacks a clear theorem-kernel boundary or a clear statement that Verus owns all Rust-local proof obligations.

Reject if any error variant has no exact expected error scenario.

### 2. Verification Layer Fit

Reject if an obligation uses a weak layer for a high-risk clause:

- Rust-local pure deterministic critical behavior: `verus` plus Rust-realization evidence such as `proptest`, `kani`, `cargo-fuzz`, `bolero`, `gauntlet-proof`, or waiver.
- Workflow/protocol/scheduler/retry/claim/lease/lifecycle/distributed temporal behavior: `tla-plus` plus realization evidence such as `stateright`, `loom`, `shuttle`, integration scenarios, or waiver.
- Tiny theorem kernel beyond Verus: `lean`, `aeneas-lean`, or `hax-lean` plus Rust-realization evidence, or waiver.
- Pure non-critical invariant with non-trivial input space: `verus` plus `proptest`, `kani`, or waiver.
- Numeric/indexing/arithmetic safety: `verus` or `kani`, with Verus preferred when the bound/invariant is Rust-local and expressible.
- UB/layout/aliasing risk: `miri` or waiver.
- Parser/codec/protocol/hostile input: `tla-plus` for temporal protocol behavior plus `cargo-fuzz` or `bolero` for input space, or waiver.
- Concurrent state/interleaving/cancellation: `tla-plus` for model-level temporal properties plus `loom`, `shuttle`, `stateright`, or `lockbud` for implementation interleavings, or waiver.
- Release-critical or cross-layer assurance: `gauntlet-all` or waiver.
- Test strength claim: `cargo-mutants` or waiver.
- Coverage claim: `cargo-llvm-cov` or waiver.
- Supply-chain/static safety: `static-scan` or waiver.
- Performance claim: `performance` with exact benchmark/profiler/load-test command, baseline, and acceptance threshold, or waiver.
- Zero-cost/vectorization/bounds-check/code-size claim: `assembly-ir` with exact symbol-level `cargo asm`, `cargo llvm-ir`, `cargo llvm-lines`, or `cargo bloat` command, or waiver.
- Public API compatibility claim: `api-compat` with exact `cargo semver-checks` baseline command, or waiver.
- Release artifact/provenance claim: `release-provenance` with exact `cargo auditable`, `cargo cyclonedx`, `cargo deny`, or `cargo vet` evidence, or waiver.
- Bit-precise or extracted proof claim beyond Verus/Lean/Kani/Miri/fuzz: `crux`, `saw`, or `hax` only when the obligation names the exact target and command.

Reject if a high, proof, critical, release, unsafe-boundary, changed-api, new-dependency, concurrent, or protocol obligation is marked `required:false` without a waiver that names owner, reason, expiry, and compensating evidence.

Reject if `command` is generic (`cargo test`, `moon run :test`, or `lake build`) when the obligation needs a named package, target, theorem, harness, fuzz target, benchmark, symbol, or Moon mode to prove the claim.

Reject if `expected_evidence` is missing, vague, or not mechanically observable by `formal-verifier`.

### 3. TLA+, Verus, and Theorem Scope

Approve TLA+ obligations only when the artifact names:

- TLA+ module/model path and config.
- Variables, Init, Next/actions, and state constraints.
- Safety invariants and temporal properties.
- Fairness, liveness, eventually/always/until, and deadlock-freedom stance when applicable.
- Refinement relation to Rust/runtime events.
- Exact `tlc`, `apalache-mc`, or `moon run :verify-proof` command and mechanically observable expected evidence.

Reject waived TLA+ coverage for workflow/protocol/concurrent/distributed behavior unless the waiver explains the concrete modeling limitation and names compensating evidence.

Approve Verus obligations only when the artifact names:

- Rust module/function/type target.
- Spec function, proof function, invariant, requires/ensures clause, decreases clause, or trusted-boundary wrapper.
- Runtime shell exclusions.
- Exact `verus` or `moon run :verify-proof` command and mechanically observable expected evidence.

Reject waived Verus coverage for Rust-local pure/core behavior unless the waiver explains the concrete Verus limitation and names compensating evidence.

### 4. Lean/Aeneas/Hax Scope

Approve Lean/Aeneas/Hax only when the artifact names:

- Pure Rust target or extracted/specification target.
- Lean module/theorem shape.
- Abstraction or refinement relation.
- Inputs, outputs, and excluded runtime shell behavior.

Reject Lean/Aeneas/Hax over live I/O, async scheduling, UI, storage adapter behavior, wall-clock time, network, filesystem, or external service behavior.

Reject obligations that try to theorem-prove runtime shells directly instead of proving a tiny pure model and separately verifying the Rust/runtime shell.

### 5. Executable Obligation Shape

Reject any `proof-obligations.jsonl` entry missing any of:

- `id`
- `contract_clause`
- `target`
- `claim`
- `layer`
- `checker`
- `command`
- `evidence`
- `expected_evidence`
- `risk`
- `scope`
- `required`
- `mode`
- `owner_state`
- `rerun_from`
- `status`

Reject if `status` is not `planned` at review time.

Reject any `tla-plus` obligation that lacks `tla_module`, `model`, `config`, `variables`, `actions`, `invariants`, `temporal_properties`, `fairness`, `state_constraints`, or `refinement` fields.

Reject if source-lint/static-scan commands include test-target linting to reject test helper structure, loops, table-driven cases, or local mutability. Tests must compile/run and assert behavior; production/source lint remains strict.

### 6. Waiver Quality

Reject any waiver missing:

- Clause ID.
- Verification layer waived.
- Reason.
- Compensating evidence.
- Owner.
- Expiration or follow-up condition.

## Output Template

```markdown
# Contract Verification Review

STATUS: APPROVED|REJECTED

## Files Reviewed
- contract.md
- tla-spec.md
- lean-contract.md
- verification-layers.md
- proof-obligations.jsonl
- traceability-matrix.jsonl

## Command Evidence
- <command> -> <exit/status summary>

## Findings
- Severity: LETHAL|MAJOR|MINOR
- Clause:
- Problem:
- Required fix:

## Coverage Decision
- Contract clauses traced:
- TLA+-owned clauses covered:
- Verus-owned clauses covered:
- Theorem-owned clauses covered:
- Proof obligations traced:
- TLA+ scope valid:
- Verus scope valid:
- Lean/Aeneas/Hax scope valid:
- Waivers valid:
```

Only `STATUS: APPROVED` may unlock downstream test planning, red tests, implementation, or formal verification work.
