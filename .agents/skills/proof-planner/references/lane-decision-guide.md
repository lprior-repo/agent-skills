# Lane Decision Guide

A `verifier-lane-decision/v1` row is the planner's binding commitment for one `(requirement_id, contract_clause, proof_seed_id, verifier)` tuple. The decision is the unit of accountability: omitting a row blocks the seed's coverage, duplicating rows create conflicts, and weak justifications block reviewer acceptance.

This guide is the algorithm. Read `references/risk-taxonomy.md` first to know which lanes a seed's risk profile demands, then walk the algorithm below.

## Algorithm (Binding)

```
for each proof_seed in proof-seeds.jsonl:
    for each verifier in mandatory_profile(seed.risk_tags):
        row = new verifier-lane-decision/v1
        row.applicability = "required"
        row.required_obligation_ids = [one_or_more_proof_obligation_v1_ids]
        row.decision_reason = "<why this verifier is required by risk_tags X, Y>"
        emit row

    for each verifier in optional_profile(seed.risk_tags):
        if seed.risk_tags intersect optional_profile:
            row = new verifier-lane-decision/v1
            row.applicability = "required"
            row.required_obligation_ids = [proof_obligation_v1_id]
            row.decision_reason = "<why this verifier is required by risk_tag Z>"
            emit row

    for each verifier in default_profile(seed) not yet emitted:
        # only allowed with concrete evidence
        row = new verifier-lane-decision/v1
        row.applicability = "not_applicable"
        row.non_applicability_evidence_refs = [<at least one SHA-256 hash>]
        row.decision_reason = "<concrete reason; not 'not needed'>"
        emit row

# after the loop, every row has at least one paired proof-obligation/v1 row
# (or a `blocked_tooling` row that explains why not)
```

## Applicability Decisions

### `required`

- Used when the seed's `risk_tags` include a tag in `references/risk-taxonomy.md` that mandates the verifier.
- `required_obligation_ids` MUST name one or more `proof-obligation/v1` IDs.
- `decision_reason` MUST cite at least one `risk_tag` by name and explain why the verifier is the right tool.
- `limitation_kind` is empty for required lanes.

### `not_applicable`

- Only allowed when the seed's `risk_tags` provably exclude the verifier's primary surface.
- `non_applicability_evidence_refs` MUST list at least one concrete artifact hash (e.g., a `proof-strategy.md` section SHA-256, a `domain-model.md` SHA-256, or a `contract.md` clause SHA-256).
- `decision_reason` MUST be a concrete sentence, not "not needed", "too hard", "not practical", or "covered by other lane".
- The reviewer (`proof-plan-reviewer`) independently verifies the evidence refs and may reject the decision.

Examples of valid `not_applicable`:

| Verifier | Reason | Evidence ref |
|---|---|---|
| `loom` (temporal) | seed is a pure Rust function with no temporal surface | `domain-model.md` SHA-256 shows no state machine |
| `loom` | seed operates on a single-threaded calculator; no `Send`/`Sync` boundaries | `workflow-model.md` SHA-256 shows no async/thread surface |
| `cargo-fuzz` | seed is purely numeric and has no parser/codec boundary | `boundary-map.md` SHA-256 shows parser is upstream |
| `miri` | seed's target crate forbids `unsafe` and uses no FFI | `boundary-map.md` SHA-256 plus `grep -n unsafe crates/foo/src` empty |

Examples of invalid `not_applicable`:

- "Not needed; trivial enough."
- "Covered by Kani."
- "Too expensive to run."
- "We'll add this later."
- "Low risk; skip for now."

These are rejected with `E_LANE_DECISION_WEAK`.

### `blocked_tooling`

- Used when the seed's risk profile requires the verifier but the verifier tool is unavailable in the workspace.
- `decision_reason` MUST name the exact missing tool (binary, component, version pin, or env var).
- `tooling_acquisition_ref` MUST be set to a bead ID or runbook step that will acquire the tool.
- The row never advances past State 4; the planner MUST emit `E_LANE_BLOCKED_TOOLING` and route the bead to a tooling acquisition branch.
- See `references/tooling-availability-gate.md`.

## Required Fields

`verifier-lane-decision/v1` is validated by `tools/go-skill-v9-validate`. See `../go-skill/references/proof-schemas.md` for the canonical schema. Required fields:

- `schema_version`
- `id` — unique, format `VLD-NNN`.
- `requirement_id`
- `contract_clause`
- `proof_seed_id`
- `verifier`
- `risk_tags` — the subset that drove this decision.
- `applicability` — one of `required`, `not_applicable`, `blocked_tooling`.
- `decision_reason`
- `required_obligation_ids` — non-empty when `applicability: required`.
- `non_applicability_evidence_refs` — non-empty when `applicability: not_applicable`.
- `tooling_acquisition_ref` — required when `applicability: blocked_tooling`.
- `limitation_kind` — required when `applicability: not_applicable`; one of `surface_absent`, `risk_out_of_scope`, `superseded_by_other_lane_with_evidence`, `external_dependency_unavoidable`.
- `owner_state`
- `status`

## Pairing With Proof Obligations

Every `verifier-lane-decision/v1` row with `applicability: required` MUST have at least one matching `proof-obligation/v1` row whose `verifier` equals the lane decision's `verifier`, whose `id` appears in `required_obligation_ids`, and whose `target` is a production source symbol (file-only is invalid).

Pairing is bidirectional: an obligation without a lane decision is `E_LANE_OBLIGATION_MISSING`; a lane decision without a paired obligation is `E_LANE_OBLIGATION_MISSING` (planner forgot the obligation).

## Cross-Lane Discipline

When two lanes target the same claim, the planner MUST distinguish their roles:

- Verus and Kani often overlap on Rust-local invariants. The planner MUST decide which is the *primary* lane and which is the *companion*. The companion's `decision_reason` MUST cite the primary by ID and explain what unique evidence it adds (e.g., "Kani bounds the input space; Verus proves the invariant for arbitrary input").
- Verus and Flux overlap on refinements. Flux owns the construction-time check; Verus owns deeper functional proof.
- Kani and proptest overlap on property obligations. Kani owns bounded symbolic proof; proptest owns randomized property pressure. Both required lanes are valid only when each has a unique target or distinct evidence (e.g., Kani covers exhaustive bounded inputs; proptest covers adversarial inputs across a strategy).

Stacking the same lane twice under different names is `E_LANE_DECISION_DUPLICATE`.

## Not-Applicable Discipline (Refusing Vagueness)

The validator enforces a vocabulary. Replace these weak phrases with concrete reasons.

| Weak | Strong replacement |
|---|---|
| "Not needed" | "The seed's target is `crate::foo::bar`, which is a single-threaded calculator with no concurrency surface; see `domain-model.md#bar` SHA-256:abc..." |
| "Too hard" | "Kani's CBMC backend is unavailable in this workspace; see `references/tooling-availability-gate.md` for the blocked_tooling branch" |
| "Covered by Kani" | "Kani provides bounded symbolic evidence for input length <= 1024; proptest provides randomized pressure up to 10000 cases and a non-vacuity strategy" |
| "Low risk" | "The seed's `domain_claim` constrains only the public API surface and is bounded by construction; no parser/codec/hostile input boundary is in scope per `boundary-map.md` SHA-256:abc..." |
| "We'll add this later" | Not allowed. Add a debt bead and re-emit later; do not silently omit. |

## Limitation Kind Vocabulary

`limitation_kind` is the typed reason for `not_applicable`. Validator enforces one of:

- `surface_absent` — the risk surface is provably absent (e.g., no parser, no concurrency).
- `risk_out_of_scope` — the risk is present but addressed outside this bead (e.g., upstream parser).
- `superseded_by_other_lane_with_evidence` — a different lane covers the claim with documented evidence; the row must cite the lane decision ID.
- `external_dependency_unavoidable` — the verifier depends on a system that is intentionally out of scope (e.g., native FFI in a `no-ffi` bead).

Vague or missing `limitation_kind` is `E_LANE_DECISION_WEAK`.

## Self-Audit Checklist (Before Handoff)

Run this checklist against your own `verifier-lane-decisions.jsonl` before submitting to the reviewer. Any miss is a self-rejection.

- [ ] Every `(requirement_id, contract_clause, proof_seed_id, verifier)` tuple in the default profile has exactly one lane decision.
- [ ] No default-profile verifier has `not_applicable` without `non_applicability_evidence_refs` containing at least one hash.
- [ ] Every `required` lane decision has at least one paired `proof-obligation/v1` ID, and the obligation exists in `proof-obligations.planned.jsonl`.
- [ ] Every `proof-obligation/v1` row references a lane decision ID by way of the inverse index in `proof-to-implementation-input.md`.
- [ ] No `blocked_tooling` row advances past State 4.
- [ ] All `decision_reason` strings cite concrete `risk_tags` and avoid the weak vocabulary above.
- [ ] All `not_applicable` rows have a typed `limitation_kind`.
- [ ] No two rows duplicate `(requirement_id, contract_clause, proof_seed_id, verifier)` with conflicting `applicability`.
