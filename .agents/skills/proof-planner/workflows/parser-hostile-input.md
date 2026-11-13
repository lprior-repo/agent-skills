# Workflow: Parser / Hostile Input (Frame::decode)

End-to-end example of a `proof-planner` run for a binary parser that accepts arbitrary bytes. This seed shape exercises `cargo-fuzz` (primary), `kani` (bounded rejection), `verus` (parse-spec round-trip), and `proptest` (typed-input pressure).

## Input (from `rust-contract`)

`proof-seeds.jsonl`:

```jsonl
{"schema_version": "proof-seed/v1", "id": "PS-104", "requirement_id": "REQ-104", "contract_clause": "C-019", "domain_claim": "Frame::decode rejects truncated input without panic", "risk_tags": ["parser", "bounded_state", "rejection", "hostile_input", "fuzzable_canonicalization"], "suggested_layers": ["cargo-fuzz", "kani", "verus", "proptest"], "behavior_affecting": true, "model_boundary": "binary_io", "notes": "Parser surface; fuzz is primary."}
```

`contract.md` says (clause C-019):

> `Frame::decode(bytes: &[u8]) -> Result<Frame, DecodeError>` MUST parse a complete frame or return `DecodeError::Truncated`/`DecodeError::Invalid`. It MUST NOT panic on any input, including truncated, malformed, or oversized bytes.

`traceability-matrix.jsonl`:

```jsonl
{"requirement_id": "REQ-104", "contract_clause": "C-019", "source_ref": "crates/parser/src/lib.rs::parser::Frame::decode"}
```

## Lane Profile

| Verifier | Decision | Reason |
|---|---|---|
| `cargo-fuzz` | `required` | `parser + hostile_input + fuzzable_canonicalization` |
| `kani` | `required` | Default profile for `bounded_state + rejection` |
| `verus` | `required` | Parse-spec round-trip companion |
| `proptest` | `required` | Default profile companion for `rejection` |
| `flux-rs` | `not_applicable` | Refinement does not express the parse invariant |
| `loom` | `not_applicable` | Single-threaded parser |
| (TLA+ removed) | n/a | Bounded, not temporal |
| `miri` | `not_applicable` | No unsafe in the parser |

## Obligations

`proof-obligations.planned.jsonl`:

```jsonl
{"schema_version": "proof-obligation/v1", "id": "PO-031", "requirement_id": "REQ-104", "contract_clause": "C-019", "domain_claim": "Frame::decode rejects truncated input without panic across bounded symbolic inputs", "risk": "rejection", "risk_tags": ["bounded_state", "rejection"], "verifier": "kani", "artifact": "harnesses/kani/frame_rejection.rs", "target": "crate::parser::Frame::decode", "command": "cargo kani -j 1 --output-format=regular --harness frame_decode_rejects_truncation", "workdir": "/abs/path/to/bead-workspace", "expected_evidence": "Kani reports VERIFICATION:- SUCCESSFUL for frame_decode_rejects_truncation; harness uses kani::any() symbolic bytes and asserts rejection property with kani::assert; paired kani::cover proves truncated branch reachable.", "assumptions": [], "model_bounds": {"unwind": 16, "input_size": 1024, "mem_high": "20G", "mem_max": "24G"}, "tool_metadata": {"tool": "cargo-kani", "version_pin": "cargo-kani@0.65", "feature_flags": [], "solver": "cbmc"}, "trusted_base_refs": [], "required": true, "behavior_affecting": true, "mode": "verify-proof", "owner_state": 4, "rerun_from": 4, "status": "planned"}
{"schema_version": "proof-obligation/v1", "id": "PO-104-FUZZ", "requirement_id": "REQ-104", "contract_clause": "C-019", "domain_claim": "Frame::decode never panics on arbitrary bytes across fuzz coverage", "risk": "hostile_input", "risk_tags": ["parser", "hostile_input", "fuzzable_canonicalization"], "verifier": "cargo-fuzz", "artifact": "fuzz/fuzz_targets/frame_decode.rs", "target": "crate::parser::Frame::decode", "command": "cargo fuzz run frame_decode --sanitizer=address -- -max_total_time=120 -rss_limit_mb=2048 -print_final_stats=1", "workdir": "/abs/path/to/bead-workspace", "expected_evidence": "Fuzz run reaches 120s budget with no crash; #<N> NEW cov recorded; corpus preserved at <workdir>/fuzz/corpus/frame_decode; oracle is round-trip (encode then decode then assert equal).", "assumptions": [], "model_bounds": {"max_total_time": 120, "rss_limit_mb": 2048}, "tool_metadata": {"tool": "cargo-fuzz", "version_pin": "cargo-fuzz@0.13", "feature_flags": ["--sanitizer=address"]}, "trusted_base_refs": [], "required": true, "behavior_affecting": true, "mode": "verify-proof", "owner_state": 4, "rerun_from": 4, "status": "planned"}
{"schema_version": "proof-obligation/v1", "id": "PO-104-VERUS", "requirement_id": "REQ-104", "contract_clause": "C-019", "domain_claim": "Frame::decode round-trip: encode(decode(x)) == x for valid x", "risk": "equality", "risk_tags": ["parser", "codec"], "verifier": "verus", "artifact": "verification/verus/frame_decode.rs", "target": "crate::parser::Frame::decode", "command": "verus --crate-type=lib --edition=2021 crates/parser/src/lib.rs crates/parser/verification/verus/frame_decode.rs", "workdir": "/abs/path/to/bead-workspace", "expected_evidence": "Verus reports Verification SUCCEEDED for the round-trip exec fn; the spec_fn proves encode(decode(x)) == x for valid x.", "assumptions": [], "model_bounds": {}, "tool_metadata": {"tool": "verus", "version_pin": "verus@0.2024.10", "solver": "z3"}, "trusted_base_refs": [], "required": true, "behavior_affecting": true, "mode": "verify-proof", "owner_state": 4, "rerun_from": 4, "status": "planned"}
{"schema_version": "proof-obligation/v1", "id": "PO-104-PROPTEST", "requirement_id": "REQ-104", "contract_clause": "C-019", "domain_claim": "Frame::decode property: arbitrary byte input within bound is rejected or accepted without panic", "risk": "rejection", "risk_tags": ["bounded_state", "rejection"], "verifier": "proptest", "artifact": "tests/proptest/frame_decode.rs", "target": "crate::parser::Frame::decode", "command": "PROPTEST_CASES=10000 cargo test --test frame_decode --release", "workdir": "/abs/path/to/bead-workspace", "expected_evidence": "Test passes for 10000 cases (test result: ok); strategy uses prop::collection::vec(any::<u8>(), 0..1024); anti-invariant asserts zero-length input is rejected without panic.", "assumptions": [], "model_bounds": {"cases": 10000, "input_size": 1024}, "tool_metadata": {"tool": "proptest", "version_pin": "proptest@1.5"}, "trusted_base_refs": [], "required": true, "behavior_affecting": true, "mode": "verify-proof", "owner_state": 4, "rerun_from": 4, "status": "planned"}
```

## Fuzz Harness (oracle: round-trip)

`fuzz/fuzz_targets/frame_decode.rs`:

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let parsed = crate::parser::Frame::decode(data);
    // Oracle: round-trip. If decode succeeded, encode must reproduce the bytes.
    if let Ok(frame) = parsed {
        let encoded = frame.encode();
        assert_eq!(crate::parser::Frame::decode(&encoded).unwrap(), frame);
    }
});
```

The oracle is named explicitly in `expected_evidence` (round-trip). Crash-only fuzzing would fail `E_FUZZ_NO_ORACLE`.

## Bridge Input

`proof-to-implementation-input.md`:

> **PO-104-FUZZ (cargo-fuzz).** Source: `crates/parser/src/lib.rs::parser::Frame::decode`. Behavior test: `tests/frame_decode_test.rs::decode_known_good` (planned by test-planner). Refinement harness: `fuzz/fuzz_targets/frame_decode.rs`. Evidence: `cargo fuzz run frame_decode --sanitizer=address -- -max_total_time=120 -rss_limit_mb=2048`.
>
> **PO-031 (kani).** Same source. Behavior test: `tests/frame_decode_test.rs::decode_truncated_input`. Refinement harness: `harnesses/kani/frame_rejection.rs`. Evidence: `cargo kani -j 1 --harness frame_decode_rejects_truncation`.
>
> **PO-104-VERUS (verus).** Same source. Behavior test: `tests/frame_decode_test.rs::roundtrip_property`. Refinement harness: `verification/verus/frame_decode.rs`. Evidence: `verus --crate-type=lib --edition=2021 crates/parser/src/lib.rs crates/parser/verification/verus/frame_decode.rs`.
>
> **PO-104-PROPTEST (proptest).** Same source. Behavior test: `tests/proptest/frame_decode.rs`. Refinement harness: n/a. Evidence: `PROPTEST_CASES=10000 cargo test --test frame_decode --release`.

## Self-Audit

- Cargo-fuzz: `-max_total_time=120` (budget), `-rss_limit_mb=2048` (memory cap), `--sanitizer=address`. ✓
- Cargo-fuzz: oracle "round-trip" named in `expected_evidence`. ✓
- Kani: `-j 1`, `mem_high=20G`, `mem_max=24G`, paired `kani::assert`. ✓
- Verus: forbids `external_body/assume/axiom` in command/target. ✓
- proptest: `PROPTEST_CASES=10000`, anti-invariant "zero-length input is rejected". ✓

Expected: `status: PASS, findings_total: 0, exit 0`.

## Next Steps

Same as the pure-invariant workflow: reviewer → writer → reviewer → bridge → formal-verifier → closure. The writer authors the Kani harness, the fuzz target, the Verus round-trip, and the proptest strategy.
