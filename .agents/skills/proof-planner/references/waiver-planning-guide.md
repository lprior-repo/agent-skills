# Waiver Planning Guide

A waiver is the planner's tool for handling obligations that cannot be discharged by the available verifiers in the current bead. Waivers are narrowly scoped, never behavior-affecting, and always time-bound. A waiver that tries to skip behavior proof is `E_BEHAVIOR_WAIVER` (fatal).

This guide is the canonical shape for `waiver-candidate/v1` and the rules for what is and is not waivable.

## What Is Waivable

Only non-behavior obligations. The canonical examples:

- A trusted abstraction (`#[verifier::external_body]`, `#[trusted]`, `#[ignore]`, opaque, extern_spec) where the boundary is necessary and the abstraction is verified elsewhere.
- A model bound (`model_bounds.unwind`, `model_bounds.input_size`) where exceeding the bound exceeds the workspace's resource budget.
- A model reduction (TLC symmetry, Miri `-Z` flag) where the reduction is sound for the claim.
- A dependency on an external system (e.g., a third-party crate without Flux spec) where the boundary is intentional.
- A diagnostic skip (`#[cfg_attr(miri, ignore)]`) for a test that cannot be modeled.

A waivable obligation MUST have:

- A `boundary_proof` that explains why the obligation holds outside the waiver.
- A `compensating_evidence` row that names a sibling obligation or external system that covers the waived claim.
- An `owner`, an ISO-8601 `expiry`, and a `review_status` of `proposed` (planner) or `approved`/`rejected` (reviewer).

## What Is NOT Waivable

Behavior-affecting obligations. Specifically:

- A property obligation over production behavior (Verus `exec fn`, Kani production harness, Flux production refinement, Loom production model, Miri production unsafe, cargo-fuzz production parser, proptest production property, temporal-workflow obligation).
- A lane decision with `applicability: required` for a default-profile verifier.
- A trust marker (`assume`, `axiom`, `admit`, `sorry`, `external_body`, `#[trusted]`, `#[ignore]`, `opaque`, `extern_spec`, stub) that affects production behavior.
- A disabled check (`--no-default-checks`, `--no-memory-safety-checks`, `--no-overflow-checks`, `--no-unwinding-checks`, `--prove-safety-only`, `--only-codegen`, `--no-codegen`).
- A refinement or model reduction that hides a missing proof.

These are `E_BEHAVIOR_WAIVER` and the planner MUST NOT emit them.

## Waiver Candidate Schema

`waiver-candidate/v1` is the planner's row. Required fields:

- `schema_version`
- `id` — unique, format `WVR-NNN`.
- `requirement_id`
- `contract_clause`
- `proof_seed_id` — optional; some waivers are bead-scoped, not seed-scoped.
- `reason` — concrete sentence citing the limitation.
- `behavior_affecting` — must be `false`. `true` is fatal.
- `boundary_proof` — explanation of why the obligation holds outside the waiver.
- `compensating_evidence` — at least one reference to a sibling obligation, external system, or property.
- `owner`
- `expiry` — ISO-8601 date; waivers expire and must be re-justified.
- `review_status` — `proposed` (planner), `approved`/`rejected` (reviewer), `expired` (lifecycle).

## Valid Waiver Reasons

| Reason | Valid? | Notes |
|---|---|---|
| "Tooling unavailable in this workspace" | yes | paired with `references/tooling-availability-gate.md` and a tooling acquisition bead |
| "Model bound exceeds workspace resource budget" | yes | paired with `references/resource-governance.md` |
| "Trusted abstraction: crate X has verified spec upstream" | yes | paired with `compensating_evidence` pointing to the upstream spec |
| "Third-party crate has no upstream Flux spec" | yes | paired with `compensating_evidence` showing the same crate is verified by Verus/Kani in the same bead |
| "Test cannot be modeled under Miri" | yes | paired with a paired native sanitizer run |
| "Symmetry reduction used; liveness not claimed" | yes | paired with `boundary_proof` showing the model is symmetric and the claim is safety-only |
| "Too hard" | no | vague; `E_WAIVER_LIFECYCLE_INVALID` |
| "Covered by other lane" | no | vague without citation; `E_WAIVER_LIFECYCLE_INVALID` |
| "We'll add this later" | no | `E_WAIVER_LIFECYCLE_INVALID`; file a debt bead instead |
| "Behavior is hard to prove" | no | `E_BEHAVIOR_WAIVER` |

## Lifecycle

```
proposed (planner)
  ↓ reviewer accepts
approved (proof-plan-reviewer)
  ↓ formal-verifier executes
  ├── obligation is closed with WAIVED (verification-ledger/v1)
  │   └── requires matching formal-waiver/v1 row
  ├── expiry passes
  │   └── review_status flips to expired; obligation MUST be re-justified
  └── reviewer rejects
      └── review_status flips to rejected; obligation MUST be discharged or re-justified
```

A `waiver-candidate/v1` row that is `approved` by the reviewer becomes a `formal-waiver/v1` row owned by `formal-verifier`. The formal verifier writes the matching `WAIVED` row in `verification-ledger/v1`. Without a `formal-waiver/v1` row, the obligation is `E_WAIVER_LIFECYCLE_INVALID` at closure.

## Boundary Proof Discipline

`boundary_proof` is the heart of the waiver. It MUST:

- Name the obligation being waived.
- Cite at least one of:
  - A sibling `proof-obligation/v1` ID.
  - A `verification-ledger/v1` PASS row.
  - An upstream spec or test from a verified crate.
  - A model reduction that is sound for the claim.
- Explain why the boundary holds outside the waiver.

Examples:

> "Waiving Kani for `crate::foo::bar::baz`: the function is bounded by `len <= 32`; proptest PO-080 covers `len <= 1024`; the bound is sound because the upstream parser truncates at 32 bytes. Boundary: parser boundary."

> "Waiving TLC liveness check: the workflow is checked only for safety; liveness is not claimed. Boundary: `LeaseClaim.tla#safety_invariants`. Symmetry on workers is sound because the safety claim is permutation-invariant."

> "Waiving Flux for `std::vec::Vec`: no upstream Flux spec exists; the equivalent Verus obligation PO-014 covers the same invariant. Boundary: Verus proof."

These pass. The following fail:

> "The function is simple enough."

> "We'll add the bound later."

> "The risk is low."

Each of these is `E_WAIVER_LIFECYCLE_INVALID`.

## Compensating Evidence

`compensating_evidence` MUST be at least one concrete reference. Acceptable forms:

- `proof-obligation/v1` ID.
- `verification-ledger/v1` row hash.
- Upstream spec URL or commit hash.
- External test suite name and version.
- Native sanitizer lane (`cargo +nightly miri`, `cargo +stable asan`).

Vague forms ("we'll add this", "covered by tests") are `E_WAIVER_LIFECYCLE_INVALID`.

## Expiry Discipline

`expiry` is ISO-8601. Default rules:

- Tooling waivers expire when the tooling acquisition bead is scheduled to close.
- Bound waivers expire when the resource budget is projected to expand.
- Trusted-abstraction waivers expire when the upstream spec is scheduled to be added or replaced.
- Symmetry waivers expire when the model is scheduled to be checked under full state exploration.

A waiver that has not been re-justified by `expiry` is `expired`. The obligation is open again.

## Cross-Lane Waivers

When two lanes both have a waived obligation for the same claim, the waiver rows are coupled:

- Both rows cite the same `boundary_proof` and `compensating_evidence`.
- Both rows have the same `expiry`.
- The reviewer accepts them as a single decision (single `verifier-lane-review/v1` row covering both waivers).

## Interaction With Trust Markers

When a waiver covers a trust marker (`#[trusted]`, `extern_spec`, `opaque`, `assume`, `axiom`), the `trusted-base-plan.md` MUST include a matching row. The waiver and the trusted-base entry share the obligation ID.

## Self-Audit Checklist

- [ ] Every waiver row has `behavior_affecting: false`.
- [ ] Every waiver row has a concrete `reason` (no weak vocabulary).
- [ ] Every waiver row has a `boundary_proof` that cites a sibling obligation or external system.
- [ ] Every waiver row has at least one `compensating_evidence` entry.
- [ ] Every waiver row has an ISO-8601 `expiry`.
- [ ] Every waiver row has an `owner` named.
- [ ] No waiver row covers a default-profile verifier on a behavior-affecting seed.
- [ ] Every waiver that covers a trust marker has a matching `trusted-base-plan.md` row.

Any miss is `E_WAIVER_LIFECYCLE_INVALID` or `E_BEHAVIOR_WAIVER`. The planner self-rejects.

## Cross-Reference

- `../go-skill/references/proof-schemas.md` — waiver-candidate/v1 schema.
- `references/implementation-binding.md` — anti-laundering rules that govern what can be trusted.
- `references/tooling-availability-gate.md` — tooling-unavailable waivers.
- `references/resource-governance.md` — bound-budget waivers.
