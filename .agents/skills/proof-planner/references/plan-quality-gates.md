# Plan Quality Gates

The planner runs these gates against its own output before handoff to `proof-plan-reviewer`. Any gate failure is a self-rejection: the planner must repair the plan and re-run the gates. The reviewer will run the same gates independently; agreement is the only path to `STATUS: APPROVED`.

The gates are mechanical where possible. The validator (`tools/go-skill-v9-validate`) implements most of them; the planner also runs them as a mental checklist.

## Gate 1 — Schema Compliance

Every artifact is schema-valid and versioned.

| Check | Failure |
|---|---|
| `proof-obligations.planned.jsonl` rows have `schema_version: proof-obligation/v1` | `E_SCHEMA_VERSION_MISSING` |
| `verifier-lane-decisions.jsonl` rows have `schema_version: verifier-lane-decision/v1` | `E_SCHEMA_VERSION_MISSING` |
| `waiver-candidates.jsonl` rows have `schema_version: waiver-candidate/v1` | `E_SCHEMA_VERSION_MISSING` |
| No `layer`, `checker`, or alias-only `claim` fields | `E_SCHEMA_ALIAS_FIELD` |
| All required fields present per schema | `E_SCHEMA_MISSING_FIELD` |

The planner MUST NOT submit a plan with schema errors. Run the validator locally first.

## Gate 2 — Lane Decision Coverage

Every seed has lane decisions for its required profile.

| Check | Failure |
|---|---|
| Every `(requirement_id, contract_clause, proof_seed_id, verifier)` tuple in the default profile has exactly one lane decision | `E_LANE_DECISION_MISSING` |
| Two active lane decisions exist for the same tuple | `E_LANE_DECISION_DUPLICATE` |
| A default-profile verifier has `not_applicable` without `non_applicability_evidence_refs` containing at least one hash | `E_LANE_DECISION_WEAK` |
| `blocked_tooling` rows have a `tooling_acquisition_ref` | `E_LANE_BLOCKED_TOOLING` |

## Gate 3 — Obligation Pairing

Every required lane decision has at least one paired obligation, and vice versa.

| Check | Failure |
|---|---|
| Lane decision `applicability: required` has at least one `proof-obligation/v1` ID in `required_obligation_ids` | `E_LANE_OBLIGATION_MISSING` |
| `proof-obligation/v1` row references a lane decision by inverse index in `proof-to-implementation-input.md` | `E_LANE_OBLIGATION_MISMATCH` |
| Obligation `verifier` matches the lane decision's `verifier` | `E_LANE_OBLIGATION_MISMATCH` |

## Gate 4 — Implementation Binding

Every behavior-affecting obligation binds to production code.

| Check | Failure |
|---|---|
| `target` parses as `path::symbol` or `crate::module::symbol` | `E_SOURCE_REF_SHAPE` |
| Verus obligation forbids `external_body`, `assume(`, `axiom` in `command`/`target`/`expected_evidence` | `E_VERUS_DISCONNECTED_SPEC` |
| Kani obligation has paired `kani::assert` or contract postcondition; `cover!` only is invalid for property claims | `E_KANI_COVER_ONLY` |
| Kani obligation uses `kani::any`/`kani::any_where`/`kani::bounded_any` for symbolic input; hardcoded structural inputs invalid | `E_KANI_ASSUMPTION_VACUITY` |
| Loom obligation uses production sync indirection | `E_LOOM_TOY_MODEL` |
| Miri obligation has a specialist scoping note when risk_tags include `ub`, `unsafe`, `ffi`, `raw_pointer`, `aliasing`, `provenance`, `layout` | `E_MIRI_PATH_SKIPPED` |
| Flux obligation does not broaden `#[trusted]`/`#[ignore]`/`extern_spec`/`opaque` without paired trusted-base entry | `E_FLUX_TRUST_ABUSE` |
| Temporal-workflow obligation (loom + proptest) has paired Rust refinement | `E_PROOF_TO_RUST_MISSING` |
| cargo-fuzz obligation has an explicit oracle | `E_FUZZ_NO_ORACLE` |
| proptest obligation has a non-vacuous strategy and an anti-invariant | `E_PROPTEST_NO_ANTI_INVARIANT` |

## Gate 5 — Evidence Specificity

Every obligation has exact, runnable evidence.

| Check | Failure |
|---|---|
| `command` has no placeholders (`<...>`, `...`, `<harness>`) | `E_COMMAND_EVIDENCE_MISSING` |
| `command` is not generic (`cargo test`, `cargo check`, `cargo build`) | `E_COMMAND_EVIDENCE_MISSING` |
| `workdir` is absolute | `E_COMMAND_EVIDENCE_MISSING` |
| `expected_evidence` cites a concrete tool marker (e.g., `VERIFICATION:- SUCCESSFUL`, `Model checking completed`, `test result: ok`) | `E_PROOF_SMOKE_MISSING` |
| `tool_metadata.version_pin` is set | `E_PROOF_SMOKE_MISSING` |
| Bounded verifiers have `model_bounds` populated | `E_PROOF_PLAN_MISSING_NONVACUITY` |

## Gate 6 — Resource Governance

Every bounded obligation has a resource budget.

| Check | Failure |
|---|---|
| Kani uses `-j 1` and `mem_high`/`mem_max` | `E_RESOURCE_UNGOVERNED` |
| cargo-fuzz uses `-max_total_time` and `-rss_limit_mb` | `E_FUZZ_BUDGET_ABSENT` |
| Miri uses pinned nightly and `MIRIFLAGS` | `E_MIRI_TOOLCHAIN_UNPINNED` |
| Flux uses pinned nightly | `E_FLUX_TOOLCHAIN_UNPINNED` |
| Loom uses `--release` and `RUSTFLAGS="--cfg loom"` | `E_LOOM_NOT_MODELED` |
| proptest uses `PROPTEST_CASES=<N>` | `E_PROPTEST_BUDGET_ABSENT` |

## Gate 7 — Waiver Discipline

Every waiver row is non-behavior, has a boundary proof, and has compensating evidence.

| Check | Failure |
|---|---|
| `behavior_affecting: true` on any waiver row | `E_BEHAVIOR_WAIVER` |
| `boundary_proof` is empty or vague | `E_WAIVER_LIFECYCLE_INVALID` |
| `compensating_evidence` is empty | `E_WAIVER_LIFECYCLE_INVALID` |
| `reason` uses weak vocabulary (see `references/waiver-planning-guide.md`) | `E_WAIVER_LIFECYCLE_INVALID` |
| Waiver without ISO-8601 `expiry` | `E_WAIVER_LIFECYCLE_INVALID` |
| Waiver without `owner` | `E_WAIVER_LIFECYCLE_INVALID` |

## Gate 8 — Trust Marker Ledger

Every trust marker has a `trusted-base-plan.md` row.

| Check | Failure |
|---|---|
| Obligation with `assumptions` non-empty has a `trusted_base_refs` entry per assumption | `E_TRUST_LEDGER_INCOMPLETE` |
| Obligation with `model_bounds` reducing a default bound has a `trusted_base_refs` entry explaining the reduction | `E_TRUST_LEDGER_INCOMPLETE` |
| Obligation with `kani::cover!` as property evidence has a `trusted_base_refs` entry | `E_TRUST_LEDGER_INCOMPLETE` |
| Obligation with Flux `#[trusted]`/`#[ignore]`/`extern_spec`/`opaque` has a `trusted_base_refs` entry | `E_TRUST_LEDGER_INCOMPLETE` |
| Miri obligation with `cfg_attr(miri, ignore)` skipped tests has a `trusted_base_refs` entry | `E_TRUST_LEDGER_INCOMPLETE` |

## Gate 9 — Cross-Reference Integrity

The bridge input is complete and the refinement obligations are planned.

| Check | Failure |
|---|---|
| Behavior-affecting obligation has a paired `rust-refinement-obligation/v1` row stub or a `proof-to-implementation-input.md` requirement | `E_PROOF_TO_RUST_MISSING` |
| `source_refs` are not file-only | `E_SOURCE_REF_SHAPE` |
| `behavior_test_refs` are independent of `refinement_harness_refs` | `E_BEHAVIOR_TEST_NOT_INDEPENDENT` |
| `refinement_harness_refs` are disjoint from `behavior_test_refs` | `E_BRIDGE_REFS_NOT_DISJOINT` |

## Gate 10 — Mirror Parity

If this skill is mirrored to `.opencode/skill/proof-planner/`, both trees MUST be byte-identical.

| Check | Failure |
|---|---|
| `diff -rq ~/.agents/skills/proof-planner/ ~/.opencode/skill/proof-planner/` reports no differences | `E_MIRROR_DRIFT` |

The mirror gate is run after every change and before every handoff.

## Self-Reject Procedure

When a gate fails, the planner MUST:

1. Identify the failing check by finding code.
2. Repair the affected artifact(s).
3. Re-run all gates.
4. Only then submit to `proof-plan-reviewer`.

A plan that fails any gate at self-audit MUST NOT be submitted. Submitting a self-rejected plan is `E_PLAN_SELF_REJECTED_BYPASS` and the reviewer is expected to reject without review.

## Cross-Reference

- `../go-skill/references/finding-codes.md` — full finding-code vocabulary.
- `../go-skill/references/proof-schemas.md` — schemas.
- `../go-skill/references/proof-pipeline-contract.md` — state ownership.
- `references/lane-decision-guide.md`, `references/implementation-binding.md`, `references/evidence-requirements.md`, `references/resource-governance.md`, `references/waiver-planning-guide.md` — gate source documents.
