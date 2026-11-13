# Workflow: Unsafe Arena Allocator (Bump::alloc)

End-to-end example of a `proof-planner` run for an unsafe arena allocator exposing a raw-pointer API. This seed shape exercises `miri` (primary, dynamic UB detection), `verus` (ownership spec), and `proptest` (randomized behavior). It also requires a `Miri Specialist Scoping Note`.

## Input (from `rust-contract`)

`proof-seeds.jsonl`:

```jsonl
{"schema_version": "proof-seed/v1", "id": "PS-150", "requirement_id": "REQ-150", "contract_clause": "C-046", "domain_claim": "Bump::alloc returns a provenance-clean pointer for valid input", "risk_tags": ["unsafe", "raw_pointer", "provenance", "aliasing"], "suggested_layers": ["miri", "verus", "proptest"], "behavior_affecting": true, "model_boundary": "unsafe_block", "notes": "Specialist scoping required for miri lane."}
```

`contract.md` says (clause C-046):

> `Bump::alloc(&mut self, layout: Layout) -> *mut u8` returns a pointer to uninitialized memory aligned to `layout`. The pointer MUST have valid provenance; it MUST NOT alias another live allocation; it MUST remain valid until `Bump::dealloc` is called.

## Lane Profile

| Verifier | Decision | Reason |
|---|---|---|
| `miri` | `required` | `unsafe + raw_pointer + provenance + aliasing` |
| `verus` | `required` | Ownership spec companion |
| `proptest` | `required` | Randomized behavior companion |
| `kani` | `required` (optional) | Bounded UB harness where Miri cannot model |
| `loom` | `not_applicable` | Single-threaded allocator |
| (TLA+ removed) | n/a | No temporal surface |
| `flux-rs` | `not_applicable` | Refinement does not express UB |
| `cargo-fuzz` | `not_applicable` | No parser/codec boundary |

## Obligations

`proof-obligations.planned.jsonl`:

```jsonl
{"schema_version": "proof-obligation/v1", "id": "PO-066", "requirement_id": "REQ-150", "contract_clause": "C-046", "domain_claim": "Bump::alloc returns a provenance-clean pointer across Miri seeds", "risk": "ub_safety", "risk_tags": ["unsafe", "raw_pointer", "provenance"], "verifier": "miri", "artifact": "tests/miri/bump_alloc.rs", "target": "crate::arena::Bump::alloc", "command": "MIRIFLAGS=\"-Zmiri-strict-provenance -Zmiri-many-seeds=0..8\" cargo +nightly miri test --test bump_alloc --release", "workdir": "/abs/path/to/bead-workspace", "expected_evidence": "Miri reports no UB diagnostic across 8 seeded runs (test result: ok); provenance flags recorded; one cfg_attr(miri, ignore) test skipped and noted in trusted-base-plan.md under TB-066-skipped-tests.", "assumptions": ["nightly toolchain pinned in rust-toolchain.toml"], "model_bounds": {"seeds": 8}, "tool_metadata": {"tool": "cargo-miri", "version_pin": "nightly-2026-02-15", "feature_flags": ["-Zmiri-strict-provenance", "-Zmiri-many-seeds"]}, "trusted_base_refs": ["TB-066-skipped-tests", "TB-066-specialist-scoping"], "required": true, "behavior_affecting": true, "mode": "verify-proof", "owner_state": 4, "rerun_from": 4, "status": "planned"}
{"schema_version": "proof-obligation/v1", "id": "PO-066-VERUS", "requirement_id": "REQ-150", "contract_clause": "C-046", "domain_claim": "Bump::alloc ownership spec: returned pointer is fresh and unaliased", "risk": "ub_safety", "risk_tags": ["unsafe", "raw_pointer", "provenance", "ownership"], "verifier": "verus", "artifact": "verification/verus/bump_alloc.rs", "target": "crate::arena::Bump::alloc", "command": "verus --crate-type=lib --edition=2021 crates/arena/src/lib.rs crates/arena/verification/verus/bump_alloc.rs", "workdir": "/abs/path/to/bead-workspace", "expected_evidence": "Verus reports Verification SUCCEEDED for bump_alloc; the spec_fn proves the returned pointer is fresh (not in the allocator's live set) and aligned to layout.alignment.", "assumptions": [], "model_bounds": {}, "tool_metadata": {"tool": "verus", "version_pin": "verus@0.2024.10", "solver": "z3"}, "trusted_base_refs": [], "required": true, "behavior_affecting": true, "mode": "verify-proof", "owner_state": 4, "rerun_from": 4, "status": "planned"}
{"schema_version": "proof-obligation/v1", "id": "PO-066-PROPTEST", "requirement_id": "REQ-150", "contract_clause": "C-046", "domain_claim": "Bump::alloc property: allocations under random layout are aligned and unique", "risk": "ub_safety", "risk_tags": ["unsafe", "raw_pointer"], "verifier": "proptest", "artifact": "tests/proptest/bump_alloc.rs", "target": "crate::arena::Bump::alloc", "command": "PROPTEST_CASES=10000 cargo test --test bump_alloc --release", "workdir": "/abs/path/to/bead-workspace", "expected_evidence": "Test passes for 10000 cases (test result: ok); strategy uses (any::<usize>(), any::<usize>()) for layout size/align; anti-invariant asserts zero-size layout returns a non-null aligned pointer or an explicit error.", "assumptions": [], "model_bounds": {"cases": 10000, "input_size": 128}, "tool_metadata": {"tool": "proptest", "version_pin": "proptest@1.5"}, "trusted_base_refs": [], "required": true, "behavior_affecting": true, "mode": "verify-proof", "owner_state": 4, "rerun_from": 4, "status": "planned"}
```

## Miri Specialist Scoping Note (mandatory)

In `trusted-base-plan.md`:

> **TB-066-specialist-scoping.** Required for `risk_tags` containing `unsafe, raw_pointer, provenance`. Contents:
>
> - Unsafe API surface under test: `crate::arena::Bump::alloc` and `crate::arena::Bump::dealloc`.
> - Provenance/aliasing model: `-Zmiri-strict-provenance` (Strict Provenance).
> - Skipped tests: `tests/bump_alloc_stress.rs` is `#[cfg_attr(miri, ignore)]` because it spawns threads; Miri does not model thread spawning. Native sanitizer lane (TSan) covers the threading risk.
> - Planned Miri command: `MIRIFLAGS="-Zmiri-strict-provenance -Zmiri-many-seeds=0..8" cargo +nightly miri test --test bump_alloc --release`.
> - Complementary native sanitizer: `cargo +nightly miri test --test bump_alloc_threaded --release` is impossible; instead, `RUSTFLAGS="-Z sanitizer=thread" cargo test --test bump_alloc_stress` (when nightly+TSan is available) or `cargo +stable asan test --test bump_alloc_stress` covers aliasing across threads.
>
> Owner: bead-150-owner. Expiry: 2026-12-31. Reviewer disposition: pending.

> **TB-066-skipped-tests.** Marker: `#[cfg_attr(miri, ignore)]`. Trusted kind: model gap. Scope: tests/bump_alloc_stress.rs. Impact: 1 test skipped under Miri; covered by TSan lane. Owner: bead-150-owner. Expiry: 2026-12-31.

Without these notes, the obligation is `E_MIRI_PATH_SKIPPED`.

## Bridge Input

`proof-to-implementation-input.md`:

> **PO-066 (miri).** Source: `crates/arena/src/lib.rs::arena::Bump::alloc`. Behavior test: `tests/bump_alloc_test.rs::alloc_returns_aligned_pointer`. Refinement harness: `tests/miri/bump_alloc.rs`. Evidence: `MIRIFLAGS="..." cargo +nightly miri test --test bump_alloc --release`. Specialist scoping: see TB-066 in trusted-base-plan.md.
>
> **PO-066-VERUS (verus).** Same source. Behavior test: `tests/bump_alloc_test.rs::alloc_returns_unique_pointer`. Refinement harness: `verification/verus/bump_alloc.rs`. Evidence: `verus --crate-type=lib --edition=2021 ...`.
>
> **PO-066-PROPTEST (proptest).** Same source. Behavior test: `tests/proptest/bump_alloc.rs`. Refinement harness: n/a. Evidence: `PROPTEST_CASES=10000 cargo test --test bump_alloc --release`.

## Self-Audit

- Miri: `MIRIFLAGS` present, `nightly-2026-02-15` pinned, seeds=8. ✓
- Miri: trusted_base_refs `["TB-066-skipped-tests", "TB-066-specialist-scoping"]` named. ✓
- Verus: no `external_body/assume/axiom` in command/target. ✓
- proptest: `PROPTEST_CASES=10000`, anti-invariant present. ✓
- Lane decision `loom`/`flux-rs`/`cargo-fuzz`: not_applicable with concrete evidence refs. ✓

Expected: `status: PASS, findings_total: 0, exit 0`.

## Next Steps

Standard pipeline. The Miri obligation is unusual: the writer must run Miri under nightly and record the seeded outputs in `proof-evidence.md`. The bridge row cites the Miri Specialist Scoping Note explicitly so the reviewer can verify the trust surface is bounded.
