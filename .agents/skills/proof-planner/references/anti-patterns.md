# Anti-Patterns Catalog

Bad planner habits. Each entry names the pattern, the failure mode, the finding code, and the correction. The catalog is paired with `references/plan-quality-gates.md`; each gate maps to one or more anti-patterns.

## A1 — Vague "not_applicable"

**Pattern.** A default-profile verifier is marked `not_applicable` with a reason like "not needed", "low risk", or "covered by other lane".

**Failure.** The reviewer rejects the plan; the seed loses coverage; downstream obligations lack paired evidence.

**Code.** `E_LANE_DECISION_WEAK`.

**Correction.** Cite concrete evidence refs (artifact hashes) and a typed `limitation_kind` (`surface_absent`, `risk_out_of_scope`, `superseded_by_other_lane_with_evidence`, `external_dependency_unavoidable`). See `references/lane-decision-guide.md`.

## A2 — Silent Lane Omission

**Pattern.** A seed's `risk_tags` map to a verifier that is missing from the plan entirely (no lane decision, no obligation).

**Failure.** The seed's coverage is incomplete; the reviewer catches it via `E_LANE_DECISION_MISSING`.

**Code.** `E_LANE_DECISION_MISSING`.

**Correction.** Emit one `verifier-lane-decision/v1` row per `(requirement_id, contract_clause, proof_seed_id, verifier)` in the seed's required profile. See `references/risk-taxonomy.md`.

## A3 — Generic Command

**Pattern.** The obligation's `command` is `cargo test`, `cargo build`, `cargo check`, `verus`, `cargo kani`, etc. — without a harness/test/target name.

**Failure.** The formal verifier cannot run the obligation; the evidence is missing.

**Code.** `E_COMMAND_EVIDENCE_MISSING`.

**Correction.** Name the exact harness, test, target, and feature flags. Examples in `references/obligation-schema.md`.

## A4 — Vacuous Kani

**Pattern.** A Kani obligation relies on `kani::cover!` alone to satisfy a property claim, or uses hardcoded structural inputs, or asserts `true`.

**Failure.** The obligation is non-proof; the reviewer rejects as `E_KANI_COVER_ONLY` or `E_KANI_ASSUMPTION_VACUITY`.

**Code.** `E_KANI_COVER_ONLY`, `E_KANI_ASSUMPTION_VACUITY`.

**Correction.** Use `kani::any`/`kani::any_where`/`kani::bounded_any` for symbolic input; assert the property with `kani::assert` or a contract postcondition; include `kani::cover` only as non-vacuity evidence. See `references/implementation-binding.md`.

## A5 — Verus Vacuum Proof

**Pattern.** A Verus obligation declares an `exec fn` with `#[verifier::external_body]`, or encodes the desired result in `requires`, or uses `axiom`/`assume` to short-circuit.

**Failure.** The obligation is non-proof; the reviewer rejects as `E_VERUS_DISCONNECTED_SPEC`.

**Code.** `E_VERUS_DISCONNECTED_SPEC`.

**Correction.** Prove the real function body. Use `assert(...)`, lemmas, `use_type_invariant`, and standard proof idioms. See `references/implementation-binding.md`.

## A6 — Toy Loom Model

**Pattern.** A Loom obligation uses `std::sync::*` directly instead of the crate's `cfg(loom)`-aware sync indirection.

**Failure.** The model does not exercise production sync; the reviewer rejects as `E_LOOM_TOY_MODEL`.

**Code.** `E_LOOM_TOY_MODEL`.

**Correction.** Route through `crate::sync::sync::*` (or equivalent) so `cfg(loom)` swaps in Loom types. See `references/implementation-binding.md`.

## A7 — (TLA+ removed; see A6 Loom Without Bridge for the temporal model anti-pattern)

**Pattern.** The temporal-workflow shape uses loom + proptest (TLA+ is no longer in scope). Loom harness without production sync indirection or without paired production-bound test falls under A6.

**Correction.** See A6 for Loom discipline. Temporal liveness/safety are proved by loom (interleavings) + proptest (randomized pressure) plus a paired `rust-refinement-obligation/v1` row.

## A8 — Miri Without Scoping

**Pattern.** A Miri obligation is marked `not_applicable` without a specialist scoping note, OR an unsafe-bearing obligation lacks a Miri row.

**Failure.** The reviewer rejects; the unsafe surface is unverified.

**Code.** `E_MIRI_PATH_SKIPPED`.

**Correction.** Always emit a Miri Specialist Scoping Note in `trusted-base-plan.md` when the seed's risk tags include `ub`, `unsafe`, `ffi`, `raw_pointer`, `aliasing`, `provenance`, or `layout`. See `references/implementation-binding.md`.

## A9 — Behavior Waiver

**Pattern.** A waiver row covers a behavior-affecting obligation, citing "too hard", "low risk", or "covered by tests".

**Failure.** Fatal. `E_BEHAVIOR_WAIVER`.

**Code.** `E_BEHAVIOR_WAIVER`.

**Correction.** Never waive behavior. File a debt bead, route to a tooling acquisition, or escalate. See `references/waiver-planning-guide.md`.

## A10 — Generic Waiver

**Pattern.** A waiver row has empty `boundary_proof`, empty `compensating_evidence`, weak `reason`, missing `expiry`, or missing `owner`.

**Failure.** The waiver is invalid; the reviewer rejects.

**Code.** `E_WAIVER_LIFECYCLE_INVALID`.

**Correction.** Provide a concrete boundary proof citing a sibling obligation or external system. Provide compensating evidence as a sibling ID, a verification-ledger hash, or an upstream spec. Use ISO-8601 expiry and name an owner.

## A11 — Resource-Ungoverned Run

**Pattern.** A Kani obligation uses `-j > 1` without recorded memory headroom; a fuzz obligation has no `-max_total_time`; a Miri obligation has no `MIRIFLAGS`; a proptest obligation has no `PROPTEST_CASES`.

**Failure.** The runner can OOM the workstation or run unbounded.

**Code.** `E_RESOURCE_UNGOVERNED`, `E_FUZZ_BUDGET_ABSENT`, `E_MIRI_TOOLCHAIN_UNPINNED`, `E_PROPTEST_BUDGET_ABSENT`.

**Correction.** Encode budgets in `model_bounds` and `tool_metadata`. Use `-j 1` by default. See `references/resource-governance.md`.

## A12 — Flux Trust Widening

**Pattern.** A Flux obligation broadens `#[trusted]`, `#[ignore]`, `extern_spec`, or `opaque` to make a failing refinement pass without a paired trusted-base entry.

**Failure.** The trust surface grows; the reviewer rejects.

**Code.** `E_FLUX_TRUST_ABUSE`.

**Correction.** Each broadened trust marker has a paired `trusted-base-plan.md` entry and a paired `negative` or `#[should_fail]` target. See `references/implementation-binding.md`.

## A13 — Copied Harness Model

**Pattern.** A Kani/Loom/fuzz/proptest harness copies production logic into a local model builder and calls that a proof.

**Failure.** The harness is a smoke check, not an implementation proof; the bridge cannot name the production target.

**Code.** `E_MODEL_COPIES_PRODUCTION`.

**Correction.** The harness calls production code directly or via an extracted production helper. The bridge row names the production symbol.

## A14 — Self-Approval

**Pattern.** The planner writes `verifier-lane-review/v1` rows or stamps `STATUS: APPROVED` on its own plan.

**Failure.** Fatal provenance violation.

**Code.** `E_REVIEW_SELF_APPROVAL`.

**Correction.** The planner never writes reviewer disposition. The reviewer (`proof-plan-reviewer`) is the only writer of `verifier-lane-review/v1` and `STATUS:` lines.

## A15 — PASS in Plan

**Pattern.** A `proof-obligation/v1` row has `status: PASS`.

**Failure.** Fatal. `PASS` is verifier output only.

**Code.** `E_SCOPE_MISCLASSIFIED_BEHAVIOR`.

**Correction.** Planner rows are `status: planned`. `PASS` is written only by `formal-verifier` into `verification-ledger/v1`.

## A16 — Smoke-Only Proof

**Pattern.** An obligation is `mode: verify-proof` but the harness is hardcoded structural inputs, `assert(true)`, or `cover!` only.

**Failure.** The mode is mislabeled; the harness is smoke only.

**Code.** `E_PROOF_SMOKE_MISSING`, `E_KANI_COVER_ONLY`.

**Correction.** Either rename to `mode: verify-smoke` and pair with a `verify-proof` obligation that actually binds to production, or fix the harness.

## A17 — Unpaired Trust Marker

**Pattern.** An obligation uses `assume(`, `axiom`, `#[trusted]`, `#[ignore]`, `extern_spec`, `opaque`, `cover!`-as-property, or a model bound but has no `trusted_base_refs` entry.

**Failure.** The trust surface is unledgerged.

**Code.** `E_TRUST_UNLEDGERED_MARKER`, `E_TRUST_LEDGER_INCOMPLETE`.

**Correction.** Each trust marker has a `trusted-base-plan.md` row with `marker`, `trusted_kind`, `scope`, `impact`, `compensating_evidence`, `owner`, `expiry`.

## A18 — (removed with TLA+)

**Correction.** Specify `-symmetry off` (or equivalent) when liveness is checked. Document the symmetry stance explicitly.

## A19 — Disabled Check

**Pattern.** A Kani obligation uses `--no-default-checks`, `--no-memory-safety-checks`, `--no-overflow-checks`, `--no-unwinding-checks`, `--prove-safety-only`, `--only-codegen`, `--no-codegen`.

**Failure.** The check is disabled; the obligation is non-proof.

**Code.** `E_KANI_COVER_ONLY` (closest), `E_BLOCKED_TOOLING_ADVANCE` (general).

**Correction.** Remove the disabled-check flag. The obligation is invalid with the flag.

## A20 — Comment-Only Proof

**Pattern.** The "proof" is a comment: `// proof: this is safe because ...`.

**Failure.** The obligation is non-proof.

**Code.** `E_PROOF_COMMENT_ONLY`.

**Correction.** The proof must be an executable artifact with raw command evidence. See `references/evidence-requirements.md`.

## Cross-Reference

- `references/plan-quality-gates.md` — gate failures map to anti-patterns.
- `references/implementation-binding.md` — anti-laundering rules.
- `references/evidence-requirements.md` — evidence discipline.
- `references/waiver-planning-guide.md` — waiver discipline.
- `references/lane-decision-guide.md` — lane decision discipline.
