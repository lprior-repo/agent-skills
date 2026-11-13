# Workflow: Refinement Newtype (Offset::new)

End-to-end example of a `proof-planner` run for a newtype with a refinement predicate. This seed shape exercises `flux-rs` (primary), `verus` (deeper functional proof), and `kani` (executable cross-check).

## Input (from `rust-contract`)

`proof-seeds.jsonl`:

```jsonl
{"schema_version": "proof-seed/v1", "id": "PS-120", "requirement_id": "REQ-120", "contract_clause": "C-022", "domain_claim": "Offset stays within [0, LEN) on construction", "risk_tags": ["refinement", "ownership", "index"], "suggested_layers": ["flux-rs", "verus", "kani"], "behavior_affecting": true, "model_boundary": "single_crate", "notes": "Refinement type."}
```

`contract.md` says (clause C-022):

> `Offset::new(value: usize) -> Result<Offset, OffsetError>` constructs an `Offset` whose value is in `[0, LEN)`. `Offset::get(self) -> usize` returns the value. The newtype MUST make invalid values unrepresentable after construction.

`type-contracts.md` shows the smart constructor pattern.

## Lane Profile

| Verifier | Decision | Reason |
|---|---|---|
| `flux-rs` | `required` | `refinement + ownership + index` |
| `verus` | `required` | Deeper functional proof of `Offset::new` |
| `kani` | `required` | Executable cross-check |
| `proptest` | `optional` | Property pressure on get/set |
| `loom` | `not_applicable` | Single-threaded newtype |
| (TLA+ removed) | n/a | No temporal surface |
| `miri` | `not_applicable` | No unsafe in the newtype |
| `cargo-fuzz` | `not_applicable` | No parser/codec boundary |

## Obligations

`proof-obligations.planned.jsonl`:

```jsonl
{"schema_version": "proof-obligation/v1", "id": "PO-047", "requirement_id": "REQ-120", "contract_clause": "C-022", "domain_claim": "Offset::new(value) accepts only value in [0, LEN)", "risk": "refinement", "risk_tags": ["refinement", "ownership", "index"], "verifier": "flux-rs", "artifact": "src/index.rs", "target": "crate::index::Offset::new", "command": "cargo flux --lib -p index --features=verified", "workdir": "/abs/path/to/bead-workspace", "expected_evidence": "Flux check reports no postcondition failures for Offset::new; paired negative test crate::index::Offset::new_negative reports should_fail as expected; no #[trusted]/#[ignore]/extern_spec/opaque broadened by this obligation.", "assumptions": [], "model_bounds": {}, "tool_metadata": {"tool": "cargo-flux", "version_pin": "flux@nightly-2026-02-15", "solver": "liquid-fixpoint", "backend": "z3"}, "trusted_base_refs": [], "required": true, "behavior_affecting": true, "mode": "verify-proof", "owner_state": 4, "rerun_from": 4, "status": "planned"}
{"schema_version": "proof-obligation/v1", "id": "PO-047-VERUS", "requirement_id": "REQ-120", "contract_clause": "C-022", "domain_claim": "Offset::new(value) returns Ok iff value in [0, LEN) and the Ok value's get() returns value", "risk": "refinement", "risk_tags": ["refinement", "ownership", "index"], "verifier": "verus", "artifact": "verification/verus/offset.rs", "target": "crate::index::Offset::new", "command": "verus --crate-type=lib --edition=2021 crates/index/src/lib.rs crates/index/verification/verus/offset.rs", "workdir": "/abs/path/to/bead-workspace", "expected_evidence": "Verus reports Verification SUCCEEDED for offset_new; spec_fn proves the postcondition: result.is_ok() == (value < LEN); no external_body, no assume(, no axiom.", "assumptions": [], "model_bounds": {}, "tool_metadata": {"tool": "verus", "version_pin": "verus@0.2024.10", "solver": "z3"}, "trusted_base_refs": [], "required": true, "behavior_affecting": true, "mode": "verify-proof", "owner_state": 4, "rerun_from": 4, "status": "planned"}
{"schema_version": "proof-obligation/v1", "id": "PO-047-KANI", "requirement_id": "REQ-120", "contract_clause": "C-022", "domain_claim": "Offset::new(value) executable check across bounded symbolic inputs", "risk": "refinement", "risk_tags": ["refinement", "index"], "verifier": "kani", "artifact": "harnesses/kani/offset_new.rs", "target": "crate::index::Offset::new", "command": "cargo kani -j 1 --output-format=regular --harness offset_new_bounded", "workdir": "/abs/path/to/bead-workspace", "expected_evidence": "Kani reports VERIFICATION:- SUCCESSFUL for offset_new_bounded; harness uses kani::any() for value (bounded < 1024) and asserts the property with kani::assert.", "assumptions": [], "model_bounds": {"unwind": 4, "input_size": 1024, "mem_high": "20G", "mem_max": "24G"}, "tool_metadata": {"tool": "cargo-kani", "version_pin": "cargo-kani@0.65", "feature_flags": [], "solver": "cbmc"}, "trusted_base_refs": [], "required": true, "behavior_affecting": true, "mode": "verify-proof", "owner_state": 4, "rerun_from": 4, "status": "planned"}
```

## Source (sketch, with Flux annotations)

```rust
#![cfg_attr(flux, flux_rs::check)]

pub const LEN: usize = 1024;

#[flux_rs::refined_by(value: usize { v < LEN })]
pub struct Offset {
    #[flux_rs::field(usize { v < LEN })]
    value: usize,
}

impl Offset {
    #[flux_rs::sig(fn (value: usize) -> Result<Offset{v: v.value < LEN}, OffsetError>)]
    pub fn new(value: usize) -> Result<Self, OffsetError> {
        if value < LEN {
            Ok(Offset { value })
        } else {
            Err(OffsetError::OutOfRange)
        }
    }

    pub fn get(&self) -> usize {
        self.value
    }
}
```

The refinement `value < LEN` is encoded at the type boundary via `#[flux_rs::refined_by]`. After construction, invalid `Offset` values are unrepresentable.

## Negative Test (paired Flux target)

```rust
#[cfg(flux)]
#[flux_rs::check]
pub fn offset_new_negative(value: usize) -> Result<Offset, OffsetError> {
    // This should_fail: LEN is not enforced.
    Offset::new(LEN)
}
```

`cargo flux --lib -p index --features=verified` should report `should_fail` for this target, providing non-vacuity evidence that the refinement is actually enforced.

## Bridge Input

`proof-to-implementation-input.md`:

> **PO-047 (flux-rs).** Source: `crates/index/src/lib.rs::index::Offset::new`. Behavior test: `tests/offset_test.rs::new_accepts_in_range` (planned by test-planner). Refinement harness: `crates/index/src/index.rs` with `#[flux_rs::refined_by]` and `#[flux_rs::field]`. Negative test: `crates/index/src/index.rs::offset_new_negative`. Evidence: `cargo flux --lib -p index --features=verified`.
>
> **PO-047-VERUS (verus).** Same source. Behavior test: same. Refinement harness: `verification/verus/offset.rs`. Evidence: `verus --crate-type=lib --edition=2021 ...`.
>
> **PO-047-KANI (kani).** Same source. Behavior test: same. Refinement harness: `harnesses/kani/offset_new.rs`. Evidence: `cargo kani -j 1 --harness offset_new_bounded`.

## Self-Audit

- Flux: nightly pinned, paired negative test, no `#[trusted]`/`#[ignore]`/`extern_spec`/`opaque` broadened. ✓
- Verus: no `external_body/assume/axiom` in command/target. ✓
- Kani: `-j 1`, `mem_high/mem_max`, paired `kani::assert`. ✓
- Lane decisions for not-required verifiers cite concrete evidence. ✓

Expected: `status: PASS, findings_total: 0, exit 0`.

## Next Steps

Standard pipeline. The writer authors the Flux annotations, the Verus spec, and the Kani harness. The negative Flux target is required for non-vacuity; without it, the Flux obligation is `E_FLUX_TRUST_ABUSE`.
