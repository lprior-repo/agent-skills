# Workflow: Pure Rust Invariant (Counter::tick)

End-to-end example of a `proof-planner` run for a pure Rust arithmetic obligation. This is the simplest seed shape; everything builds on it.

## Input (from `rust-contract`)

`proof-seeds.jsonl`:

```jsonl
{"schema_version": "proof-seed/v1", "id": "PS-042", "requirement_id": "REQ-042", "contract_clause": "C-007", "domain_claim": "Counter::tick never underflows at usize zero", "risk_tags": ["rust_local", "pure_core", "arithmetic"], "suggested_layers": ["verus", "kani", "proptest"], "behavior_affecting": true, "model_boundary": "single_crate", "notes": "Default profile."}
```

`contract.md` says (clause C-007):

> `Counter::tick` decrements an internal `usize` count. It MUST NOT panic, wrap, or underflow when the count is at zero.

`traceability-matrix.jsonl`:

```jsonl
{"requirement_id": "REQ-042", "contract_clause": "C-007", "source_ref": "crates/counter/src/lib.rs::counter::Counter::tick"}
```

## Lane Profile

Per `references/risk-taxonomy.md` and `references/cross-lane-coverage-matrix.md`:

| Verifier | Decision | Reason |
|---|---|---|
| `verus` | `required` | `rust_local + pure_core + arithmetic` |
| `kani` | `required` | Default profile companion for `arithmetic_overflow` |
| `proptest` | `required` | Default profile companion for `arithmetic_overflow` |
| `flux-rs` | `not_applicable` | `refinement` not in `risk_tags`; see waiver if applicable |
| `loom` | `not_applicable` | Single-threaded calculator |
| (TLA+ removed) | n/a | Temporal workflows are covered by loom + proptest, not applicable to pure invariants |
| `miri` | `not_applicable` | Crate forbids `unsafe` |
| `cargo-fuzz` | `not_applicable` | No parser/codec boundary |

## Lane Decisions

`verifier-lane-decisions.jsonl`:

```jsonl
{"schema_version": "verifier-lane-decision/v1", "id": "VLD-001", "requirement_id": "REQ-042", "contract_clause": "C-007", "proof_seed_id": "PS-042", "verifier": "verus", "risk_tags": ["rust_local", "pure_core", "arithmetic"], "applicability": "required", "decision_reason": "rust_local + arithmetic demand verus per references/risk-taxonomy.md.", "required_obligation_ids": ["PO-014"], "non_applicability_evidence_refs": [], "limitation_kind": "", "owner_state": 4, "status": "planned"}
{"schema_version": "verifier-lane-decision/v1", "id": "VLD-002", "requirement_id": "REQ-042", "contract_clause": "C-007", "proof_seed_id": "PS-042", "verifier": "kani", "risk_tags": ["rust_local", "pure_core", "arithmetic"], "applicability": "required", "decision_reason": "Default profile for arithmetic_overflow requires kani as bounded companion.", "required_obligation_ids": ["PO-014-KANI"], "non_applicability_evidence_refs": [], "limitation_kind": "", "owner_state": 4, "status": "planned"}
{"schema_version": "verifier-lane-decision/v1", "id": "VLD-003", "requirement_id": "REQ-042", "contract_clause": "C-007", "proof_seed_id": "PS-042", "verifier": "proptest", "risk_tags": ["rust_local", "pure_core", "arithmetic"], "applicability": "required", "decision_reason": "Default profile for arithmetic_overflow requires proptest as property-pressure companion.", "required_obligation_ids": ["PO-014-PROPTEST"], "non_applicability_evidence_refs": [], "limitation_kind": "", "owner_state": 4, "status": "planned"}
{"schema_version": "verifier-lane-decision/v1", "id": "VLD-004", "requirement_id": "REQ-042", "contract_clause": "C-007", "proof_seed_id": "PS-042", "verifier": "flux-rs", "risk_tags": [], "applicability": "not_applicable", "decision_reason": "Flux is not selected because the risk_tags do not include refinement or ownership; the seed is purely arithmetic and the existing type (Counter) is already a structural newtype.", "required_obligation_ids": [], "non_applicability_evidence_refs": ["contract.md#C-007@sha256:abc...", "domain-model.md#Counter@sha256:def..."], "limitation_kind": "surface_absent", "owner_state": 4, "status": "planned"}
{"schema_version": "verifier-lane-decision/v1", "id": "VLD-005", "requirement_id": "REQ-042", "contract_clause": "C-007", "proof_seed_id": "PS-042", "verifier": "loom", "risk_tags": [], "applicability": "not_applicable", "decision_reason": "Single-threaded calculator; no Send/Sync boundary; see workflow-model.md SHA-256.", "required_obligation_ids": [], "non_applicability_evidence_refs": ["workflow-model.md@sha256:..."], "limitation_kind": "surface_absent", "owner_state": 4, "status": "planned"}
{"schema_version": "verifier-lane-decision/v1", "id": "VLD-006", "requirement_id": "REQ-042", "contract_clause": "C-007", "proof_seed_id": "PS-042", "verifier": "loom", "risk_tags": [], "applicability": "not_applicable", "decision_reason": "No temporal surface; tick is a single pure transition with no retries, leases, or workflows.", "required_obligation_ids": [], "non_applicability_evidence_refs": ["contract.md#C-007@sha256:..."], "limitation_kind": "surface_absent", "owner_state": 4, "status": "planned"}
{"schema_version": "verifier-lane-decision/v1", "id": "VLD-007", "requirement_id": "REQ-042", "contract_clause": "C-007", "proof_seed_id": "PS-042", "verifier": "miri", "risk_tags": [], "applicability": "not_applicable", "decision_reason": "The crate sets #![forbid(unsafe_code)] and uses no FFI; see boundary-map.md SHA-256.", "required_obligation_ids": [], "non_applicability_evidence_refs": ["boundary-map.md@sha256:..."], "limitation_kind": "surface_absent", "owner_state": 4, "status": "planned"}
{"schema_version": "verifier-lane-decision/v1", "id": "VLD-008", "requirement_id": "REQ-042", "contract_clause": "C-007", "proof_seed_id": "PS-042", "verifier": "cargo-fuzz", "risk_tags": [], "applicability": "not_applicable", "decision_reason": "No parser/codec boundary; Counter::tick accepts no external input.", "required_obligation_ids": [], "non_applicability_evidence_refs": ["boundary-map.md@sha256:..."], "limitation_kind": "surface_absent", "owner_state": 4, "status": "planned"}
```

## Obligations

`proof-obligations.planned.jsonl`:

```jsonl
{"schema_version": "proof-obligation/v1", "id": "PO-014", "requirement_id": "REQ-042", "contract_clause": "C-007", "domain_claim": "Counter::tick never underflows at usize zero", "risk": "arithmetic_overflow", "risk_tags": ["rust_local", "pure_core", "arithmetic"], "verifier": "verus", "artifact": "verification/verus/counter_tick.rs", "target": "crate::counter::Counter::tick", "command": "verus --crate-type=lib --edition=2021 crates/counter/src/lib.rs crates/counter/verification/verus/counter_tick.rs", "workdir": "/abs/path/to/bead-workspace", "expected_evidence": "Verus reports Verification SUCCEEDED for counter_tick; post-check confirms no external_body, no assume(, no axiom.", "assumptions": [], "model_bounds": {}, "tool_metadata": {"tool": "verus", "version_pin": "verus@0.2024.10", "solver": "z3"}, "trusted_base_refs": [], "required": true, "behavior_affecting": true, "mode": "verify-proof", "owner_state": 4, "rerun_from": 4, "status": "planned"}
{"schema_version": "proof-obligation/v1", "id": "PO-014-KANI", "requirement_id": "REQ-042", "contract_clause": "C-007", "domain_claim": "Counter::tick never underflows across bounded symbolic inputs", "risk": "arithmetic_overflow", "risk_tags": ["rust_local", "pure_core", "arithmetic"], "verifier": "kani", "artifact": "harnesses/kani/counter_tick_bounded.rs", "target": "crate::counter::Counter::tick", "command": "cargo kani -j 1 --output-format=regular --harness counter_tick_bounded", "workdir": "/abs/path/to/bead-workspace", "expected_evidence": "Kani reports VERIFICATION:- SUCCESSFUL for counter_tick_bounded; harness uses kani::any() symbolic input and asserts property with kani::assert.", "assumptions": [], "model_bounds": {"unwind": 8, "input_size": 64, "mem_high": "20G", "mem_max": "24G"}, "tool_metadata": {"tool": "cargo-kani", "version_pin": "cargo-kani@0.65", "feature_flags": [], "solver": "cbmc"}, "trusted_base_refs": [], "required": true, "behavior_affecting": true, "mode": "verify-proof", "owner_state": 4, "rerun_from": 4, "status": "planned"}
{"schema_version": "proof-obligation/v1", "id": "PO-014-PROPTEST", "requirement_id": "REQ-042", "contract_clause": "C-007", "domain_claim": "Counter::tick property: never panics for arbitrary usize inputs within size bound", "risk": "arithmetic_overflow", "risk_tags": ["rust_local", "pure_core", "arithmetic"], "verifier": "proptest", "artifact": "tests/proptest/counter_tick.rs", "target": "crate::counter::Counter::tick", "command": "PROPTEST_CASES=10000 cargo test --test counter_tick --release", "workdir": "/abs/path/to/bead-workspace", "expected_evidence": "Test passes for 10000 cases (test result: ok); strategy uses prop::collection::vec(any::<u8>(), 0..1024); anti-invariant asserts tick at usize::MAX-1 does not panic.", "assumptions": [], "model_bounds": {"cases": 10000, "input_size": 1024}, "tool_metadata": {"tool": "proptest", "version_pin": "proptest@1.5"}, "trusted_base_refs": [], "required": true, "behavior_affecting": true, "mode": "verify-proof", "owner_state": 4, "rerun_from": 4, "status": "planned"}
```

## Bridge Input

`proof-to-implementation-input.md`:

> **PO-014 (verus).** Source: `crates/counter/src/lib.rs::counter::Counter::tick`. Behavior test: `tests/counter_tick_test.rs::tick_at_zero_does_not_panic` (planned by test-planner). Refinement harness: `verification/verus/counter_tick.rs::exec_counter_tick`. Evidence command: `verus --crate-type=lib --edition=2021 crates/counter/src/lib.rs crates/counter/verification/verus/counter_tick.rs`. The exec fn mirrors the production body and is proven directly.
>
> **PO-014-KANI (kani).** Same source. Behavior test: same. Refinement harness: `harnesses/kani/counter_tick_bounded.rs`. Evidence: `cargo kani -j 1 --harness counter_tick_bounded`.
>
> **PO-014-PROPTEST (proptest).** Same source. Behavior test: `tests/proptest/counter_tick.rs`. Refinement harness: n/a (property test IS the refinement). Evidence: `PROPTEST_CASES=10000 cargo test --test counter_tick --release`.

## Self-Audit

Run `python3 scripts/validate_plan.py <plan_dir>`:

- Schema: PASS
- Lane decisions: 8 rows (3 required, 5 not_applicable)
- Obligations: 3 rows, all paired
- Default risk profile: arithmetic_overflow requires `verus + kani + proptest` — all present.
- Resource governance: Kani has `-j 1` and `mem_high/mem_max`; proptest has `PROPTEST_CASES`.
- Anti-laundering: Verus obligation forbids `external_body/assume/axiom`; Kani uses `kani::assert` not `cover!` as proof.
- Waivers: none.

Expected: `status: PASS, findings_total: 0, exit 0`.

## Next Steps

1. Submit to `proof-plan-reviewer`. The reviewer writes `verifier-lane-review/v1` rows with `reviewer_disposition: accepted` and `STATUS: APPROVED` in `proof-plan-review.md`.
2. `proof-writer` authors the proof artifacts.
3. `proof-reviewer` approves the proof artifacts.
4. `proof-to-implementation` materializes the bridge.
5. `holzman-rust` implements production Rust (already in place if this is a planning bead for existing code).
6. `formal-verifier` runs `verus`, `cargo kani`, `cargo test`, and closes the ledger.
