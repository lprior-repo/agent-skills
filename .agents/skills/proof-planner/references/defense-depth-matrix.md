# Defense Depth Matrix

Minimum lane coverage per risk class. "Defense depth" here is the minimum count of distinct verifiers that must appear for a behavior-affecting claim in that risk class. Each row is binding; deviating requires `non_applicability_evidence_refs` and reviewer acceptance.

## How to Read

The matrix below is keyed on `proof-obligation/v1.risk`. For each risk, the planner MUST emit at least the listed lanes as `applicability: required`. Adding more lanes is encouraged when the seed's `risk_tags` motivate it (see `references/risk-taxonomy.md`). Removing a lane is forbidden without a `not_applicable` row that cites concrete evidence and is independently accepted.

## Matrix

| Risk class | Required lanes (minimum) | Optional companions |
|---|---|---|
| `arithmetic_overflow` | `verus` + `kani` + `proptest` | `miri` if `unsafe` is in scope |
| `index_safety` | `verus` + `kani` + `proptest` | `flux-rs` if the index is a refinement |
| `panic_freedom` | `verus` + `kani` + `proptest` | `miri` if `unsafe` is in scope |
| `rejection` | `kani` + `proptest` | `verus` for the parse-spec round-trip |
| `bounded_transition` | `kani` + `verus` | `flux-rs` for typestate |
| `equality` | `proptest` + `verus` | `kani` for bounded exhaustive |
| `ordering` | `proptest` + `verus` | `kani` for bounded exhaustive |
| `field_sensitivity` | `proptest` + `kani` | `verus` for the property invariant |
| `illegal_state` | `flux-rs` + `verus` | `kani` for executable cross-check |
| `refinement` | `flux-rs` + `verus` | `kani` for runtime cross-check |
| `concurrency_interleaving` | `loom` + `proptest` | `verus` for ordering spec |
| `cancellation_safety` | `loom` + `proptest` | `flux-rs` for typestate |
| `shutdown_drain` | `loom` + `proptest` | `flux-rs` for region ownership |
| `temporal_liveness` | `loom` + `proptest` | randomized pressure |
| `temporal_safety` | `verus` + `loom` | `kani` for bounded |
| `ub_safety` | `miri` + `verus` + `proptest` | `kani` for bounded UB; native sanitizer |
| `hostile_input` | `cargo-fuzz` + `kani` + `proptest` | `verus` for parse spec |
| `parse_canonicalization` | `cargo-fuzz` + `verus` + `kani` | `proptest` for typed-input pressure |

## Stacking Discipline

Stacking two lanes that produce the same evidence is `E_LANE_DECISION_DUPLICATE`. Each required lane MUST have a distinct role:

- Verus proves the unbounded invariant.
- Kani provides bounded symbolic evidence with explicit bounds.
- Flux proves the type-level refinement.
- Loom proves the interleaving under bounded exploration.
- TLA+ removed; temporal behavior is proved by loom + proptest.
- Miri detects UB across the seeded runs.
- cargo-fuzz generates adversarial input coverage.
- proptest generates randomized property pressure with shrink.

Two lanes that produce overlapping evidence need a `decision_reason` that explains the unique evidence each adds (see `references/lane-decision-guide.md`).

## Risk Class → Behavior-Affecting Default

Every risk in this matrix is behavior-affecting by default. The planner MUST set `behavior_affecting: true` on every required obligation in the matrix unless the obligation is explicitly scoped to a non-behavior surface (e.g., an internal trait bound that is structurally enforced). Internal `not_applicable` decisions for behavior-affecting rows are invalid; `E_BEHAVIOR_WAIVER` is fatal.

## Depth Scaling by Seed Size

For seeds that span more than one production source file, the planner MUST scale the lane count as follows:

- 1 file, ≤ 100 LoC: minimum matrix applies; one obligation per lane is sufficient.
- 1-3 files, ≤ 500 LoC: minimum matrix applies; consider per-file proptest strategies.
- 4+ files, > 500 LoC: minimum matrix applies; consider per-module proptest strategies and multiple Kani harnesses.
- Workspace-wide seeds: minimum matrix applies per affected crate; cross-crate invariants require explicit `verus` obligations in each affected crate and a `kani` cross-crate harness.

These are floors, not ceilings. The planner SHOULD add lanes when the seed's `risk_tags` motivate them.

## Cross-Class Seeds

When a seed's `risk_tags` map to more than one risk class, the planner MUST union the required lanes. For example, a parser with arithmetic on input length maps to `hostile_input` (cargo-fuzz + kani + proptest) and `arithmetic_overflow` (verus + kani + proptest). The union is `verus` + `kani` + `cargo-fuzz` + `proptest`, with `kani` and `proptest` shared.

If two risk classes conflict on a lane (e.g., one demands `loom` and another demands `flux-rs` for the same target), the planner MUST choose the lane that exercises the larger risk surface and document the deprioritized lane as a `not_applicable` with `limitation_kind: superseded_by_other_lane_with_evidence` and a citation.

## Defense Depth and Resource Governance

Each required lane has a minimum resource budget. See `references/resource-governance.md`:

| Lane | Default resource budget |
|---|---|
| `verus` | no resource cap; Verus is solver-bound |
| `kani` | `-j 1`, `mem_high=20G`, `mem_max=24G`, `unwind` per claim |
| `flux-rs` | nightly toolchain; `liquid-fixpoint` solver memory |
| `loom` | `LOOM_MAX_PREEMPTIONS=3`, `LOOM_MAX_BRANCHES=20000`, `--release` |
| (TLA+ removed) | n/a |
| `miri` | nightly + miri component; `MIRIFLAGS` per claim; seed range |
| `cargo-fuzz` | `-max_total_time=120` smoke; `-rss_limit_mb=2048`; sanitizer per claim |
| `proptest` | `PROPTEST_CASES=10000` minimum; size strategy per claim |

Larger budgets require `references/tooling-availability-gate.md` and reviewer acceptance.

## Cross-Reference With Lane Decision Guide

Every row in this matrix produces one or more `verifier-lane-decision/v1` rows with `applicability: required` and at least one paired `proof-obligation/v1` row. See `references/lane-decision-guide.md` for the row schema and pairing rules.
