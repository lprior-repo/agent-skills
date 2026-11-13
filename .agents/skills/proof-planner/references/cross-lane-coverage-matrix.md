# Cross-Lane Coverage Matrix

The matrix below maps canonical seed shapes to the union of verifier lanes each shape demands. This complements `references/defense-depth-matrix.md` (which gives per-risk-class minimums) and `references/risk-taxonomy.md` (which gives per-tag profile additions) by showing what a complete plan looks like for each archetypal seed.

The matrix is binding when a seed's `domain_claim` matches the shape description. Deviations require `non_applicability_evidence_refs` and reviewer acceptance.

## Pure Invariant (Counter::tick, Parse-Spec Round-Trip)

Shape: Rust-local pure/core function with arithmetic, indexing, or typestate.

```
risk_tags:   rust_local, pure_core, arithmetic | index | refinement
risk:        arithmetic_overflow | index_safety | panic_freedom | refinement
required:    verus (primary, depth=unbounded)
             kani  (companion, bounded symbolic)
             flux-rs (when refinement expresses the claim)
             proptest (companion, randomized property)
optional:    miri   (if unsafe in scope)
             loom  (NO — single-threaded)
             cargo-fuzz (NO — no parser/codec boundary)
```

## Parser / Codec / Hostile Input

Shape: byte/text input boundary that accepts arbitrary data.

```
risk_tags:   parser, codec, hostile_input, fuzzable_canonicalization
risk:        hostile_input | parse_canonicalization | rejection
required:    cargo-fuzz (primary, dynamic coverage)
             kani (companion, bounded rejection)
             verus (companion, parse-spec round-trip)
             proptest (companion, typed-input pressure)
optional:    flux-rs (NO unless refinement expresses the parse invariant)
             loom  (NO — single-threaded)
             miri  (NO unless the parser uses unsafe)
```

## Concurrency / Channel / Cancellation

Shape: threads, atomics, channels, locks, cancellation, shutdown.

```
risk_tags:   concurrency, channel, atomic, lock, cancellation, shutdown
risk:        concurrency_interleaving | cancellation_safety | shutdown_drain
required:    loom (primary, interleavings)
             proptest (companion, randomized pressure)
optional:    flux-rs (when ownership typestate)
             kani (when bounded drain)
             verus (when ordering spec is provable)
             miri (NO unless unsafe sync primitive)
             cargo-fuzz (NO — no hostile input)
```

## Unsafe / FFI / Raw Pointer

Shape: explicit `unsafe`, FFI crossing, raw-pointer arithmetic, layout assumptions.

```
risk_tags:   unsafe, ffi, raw_pointer, aliasing, provenance, layout, ub
risk:        ub_safety
required:    miri (primary, dynamic UB detection — specialist scoping note required)
             verus (companion, ownership spec)
             proptest (companion, randomized behavior)
optional:    kani (when the unsafe path is bounded — `-Z` flags required)
             native sanitizer (complementary to Miri for surfaces Miri cannot model)
             loom (NO — model checker for concurrency, not UB)
             cargo-fuzz (NO — not for UB)
             flux-rs (NO — refinement is about types, not UB)
```

## Temporal Workflow / Protocol / Scheduler

Shape: state-over-time behavior with retries, leases, claims, distributed coordination.

```
risk_tags:   temporal_workflow, protocol, scheduler, lease, liveness, distributed
risk:        temporal_liveness | temporal_safety
required:    loom (primary, interleavings + temporal surface)
             proptest (companion, randomized pressure)
optional:    flux-rs (when region/typestate refinement)
             verus (when the workflow has a Rust-local invariant)
             kani (when the workflow has a bounded sub-state)
             miri (NO — temporal, not UB)
             cargo-fuzz (NO — workflow, not parser)
```

## Refinement Type / Typestate

Shape: invariant expressed as a refinement type at the construction boundary.

```
risk_tags:   refinement, ownership, index
risk:        refinement | illegal_state | index_safety
required:    flux-rs (primary, refinement type)
             verus (companion, deeper functional proof)
             kani (companion, executable cross-check)
optional:    proptest (companion, randomized property pressure)
             loom (NO — refinement is about types, not concurrency)
             miri (NO — refinement is not UB)
             cargo-fuzz (NO — refinement is not hostile input)
```

## Property Pressure (Ordering, Equality, Field Sensitivity)

Shape: a property that holds across a strategy-generated input space.

```
risk_tags:   property, ordering, equality, field_sensitivity
risk:        equality | ordering | field_sensitivity
required:    proptest (primary, randomized property)
             verus (companion, the unbounded invariant)
             kani (companion, bounded exhaustive)
optional:    flux-rs (when refinement expresses the property)
             loom (NO — no concurrency surface)
             miri (NO — property is not UB)
             cargo-fuzz (NO — property is not hostile input)
```

## Rejection / Bounded State Machine

Shape: explicit rejection of invalid input; finite state machine with bounded transitions.

```
risk_tags:   bounded_state, rejection
risk:        rejection | bounded_transition | panic_freedom
required:    kani (primary, bounded symbolic)
             verus (companion, the unbounded invariant)
             proptest (companion, randomized rejection)
optional:    flux-rs (when typestate)
             loom (NO — single-threaded)
             miri (NO unless unsafe in scope)
             cargo-fuzz (NO unless the input is also a parser)
```

## Cross-Class Seeds

When a seed matches multiple shapes, the planner unions the required lanes and de-duplicates. For example:

- A parser inside a workflow: parser lanes (cargo-fuzz, kani, verus, proptest) ∪ workflow lanes (loom, proptest) = `{cargo-fuzz, kani, verus, proptest, loom}`. proptest is shared.
- An arena allocator exposed in a concurrent queue: UB lanes (miri, verus, proptest) ∪ concurrency lanes (loom, proptest) = `{miri, verus, proptest, loom}`. proptest is shared.

When two required lanes conflict (e.g., one shape demands `loom` and another demands `flux-rs` for the same target), the planner picks the lane that exercises the larger risk surface and documents the deprioritized lane as `not_applicable` with `limitation_kind: superseded_by_other_lane_with_evidence` and a citation.

## Self-Audit Checklist

- [ ] Every obligation's `verifier` is in the required-or-optional set for its seed shape.
- [ ] Every required lane for the seed's shape has at least one paired obligation.
- [ ] No shape-locked lane (e.g., `loom` for a pure invariant) appears without a typed `not_applicable`.
- [ ] Cross-class seeds have explicit union, with shared lanes named.
- [ ] The risk profile coverage check passes (`scripts/validate_plan.py` exit 0).

## Cross-Reference

- `references/risk-taxonomy.md` — per-tag lane profile.
- `references/defense-depth-matrix.md` — per-risk minimum lane count.
- `references/verifier-trigger-matrix.md` — per-verifier triggers and counterindications.
- `references/lane-decision-guide.md` — `not_applicable` and `blocked_tooling` rules.
