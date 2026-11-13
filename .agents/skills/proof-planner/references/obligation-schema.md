# Proof Obligation Schema

`proof-obligation/v1` is the contract between `proof-planner`, `proof-plan-reviewer`, `proof-writer`, `proof-to-implementation`, and `formal-verifier`. Every row is one JSON object per line of `proof-obligations.planned.jsonl`. Validator beats Markdown; a row that passes Markdown review but fails schema validation is rejected with `E_SCHEMA_MISSING_FIELD` or `E_SCHEMA_ALIAS_FIELD`.

## Required Fields (Binding)

| Field | Type | Notes |
|---|---|---|
| `schema_version` | string literal `"proof-obligation/v1"` | mandatory |
| `id` | string | unique within the plan, format `PO-NNN` |
| `requirement_id` | string | matches `delivery-scope.jsonl` and `proof-seeds.jsonl` |
| `contract_clause` | string | matches `contract.md` clause ID |
| `domain_claim` | string | one-sentence claim under test |
| `risk` | string | one of `bounded_transition`, `arithmetic_overflow`, `index_safety`, `panic_freedom`, `illegal_state`, `refinement`, `concurrency_interleaving`, `cancellation_safety`, `shutdown_drain`, `temporal_liveness` (loom+proptest), `temporal_safety` (loom+verus), `ub_safety`, `hostile_input`, `rejection`, `equality`, `ordering`, `field_sensitivity`, `parse_canonicalization` |
| `risk_tags` | array of strings | drawn from `references/risk-taxonomy.md` |
| `verifier` | string | one of `verus`, `kani`, `flux-rs`, `loom`, `miri`, `cargo-fuzz`, `proptest` |
| `artifact` | string | path relative to `workdir` of the artifact to be produced |
| `target` | string | production source symbol or extracted production helper (file-only is invalid) |
| `command` | string | exact shell command; placeholders are invalid |
| `workdir` | string | absolute path |
| `expected_evidence` | string | one-sentence description of the success criterion |
| `assumptions` | array of strings | each entry is a property the proof assumes; each triggers a `trusted-base-plan.md` entry |
| `model_bounds` | object | `{ "unwind": N, "input_size": N, "threads": N, "duration_seconds": N, "max_total_time": N, "rss_limit_mb": N, "mem_high": "20G", "mem_max": "24G" }`; required for bounded verifiers |
| `tool_metadata` | object | `{ "tool": "<verifier>", "version_pin": "<commit|version|date>", "feature_flags": ["..."], "solver": "...", "backend": "..." }` |
| `trusted_base_refs` | array of strings | `trusted-base-plan.md` note IDs that this obligation raises |
| `required` | boolean | `true` for behavior-affecting obligations on the default profile |
| `behavior_affecting` | boolean | `true` if the claim constrains production behavior |
| `mode` | string | `verify-proof` (binds to production) or `verify-smoke` (smoke only, must pair with `verify-proof` row) |
| `owner_state` | integer | current Go-skill state number (4 at planning) |
| `rerun_from` | integer | lowest state that must rerun on change |
| `status` | string | `planned` (planner output only); never `PASS` |

## Forbidden Fields (Aliases)

The following legacy aliases are invalid. Validator emits `E_SCHEMA_ALIAS_FIELD`.

- `layer` (use `verifier`)
- `checker` (use `verifier`)
- `claim` (use `domain_claim`)

## Status Discipline

| `status` | Allowed writer | Meaning |
|---|---|---|
| `planned` | `proof-planner` only | planner-owned row awaiting proof-writer |
| `blocked_tooling` | `proof-planner` | required tool unavailable; emits `E_LANE_BLOCKED_TOOLING`, blocks advance |
| `waived` | `formal-verifier` only | requires `behavior_affecting: false`, `formal-waiver/v1` row, and matching `verification-ledger/v1` `WAIVED` row |
| `verified` | `formal-verifier` only | requires `verification-ledger/v1` `PASS` row with raw command evidence |

`PASS` is forbidden in `proof-obligation/v1`. The planner never produces evidence; the verifier does.

## Per-Lane Examples

Each example is one row. Adapt the IDs, paths, and commands to the bead; do not copy literally.

### Verus — Rust-local invariant

```json
{
  "schema_version": "proof-obligation/v1",
  "id": "PO-014",
  "requirement_id": "REQ-042",
  "contract_clause": "C-007",
  "domain_claim": "crate::counter::Counter::tick never underflows at usize zero",
  "risk": "arithmetic_overflow",
  "risk_tags": ["rust_local", "pure_core", "arithmetic"],
  "verifier": "verus",
  "artifact": "verification/verus/counter_tick.rs",
  "target": "crate::counter::Counter::tick",
  "command": "verus --crate-type=lib --edition=2021 crates/counter/src/lib.rs crates/counter/verification/verus/counter_tick.rs",
  "workdir": "/abs/path/to/bead-workspace",
  "expected_evidence": "Verus reports SUCCESS for the named exec fn, no `external_body`, no `assume(`, no unproved `axiom`",
  "assumptions": [],
  "model_bounds": {},
  "tool_metadata": { "tool": "verus", "version_pin": "verus@0.2024.10", "solver": "z3" },
  "trusted_base_refs": [],
  "required": true,
  "behavior_affecting": true,
  "mode": "verify-proof",
  "owner_state": 4,
  "rerun_from": 4,
  "status": "planned"
}
```

### Kani — bounded state machine

```json
{
  "schema_version": "proof-obligation/v1",
  "id": "PO-031",
  "requirement_id": "REQ-104",
  "contract_clause": "C-019",
  "domain_claim": "crate::parser::Frame::decode rejects truncated input without panic",
  "risk": "rejection",
  "risk_tags": ["parser", "bounded_state", "hostile_input"],
  "verifier": "kani",
  "artifact": "harnesses/kani/frame_rejection.rs",
  "target": "crate::parser::Frame::decode",
  "command": "cargo kani -j 1 --output-format=regular --harness frame_decode_rejects_truncation --mem-predicates",
  "workdir": "/abs/path/to/bead-workspace",
  "expected_evidence": "Kani reports VERIFICATION:- SUCCESSFUL for frame_decode_rejects_truncation, with check_failure=0, and a paired `kani::cover` reachability entry",
  "assumptions": ["input length <= 1024"],
  "model_bounds": { "unwind": 16, "input_size": 1024, "mem_high": "20G", "mem_max": "24G" },
  "tool_metadata": { "tool": "cargo-kani", "version_pin": "cargo-kani@0.65", "feature_flags": ["--mem-predicates"], "solver": "cbmc" },
  "trusted_base_refs": ["TB-031-cover-reachability"],
  "required": true,
  "behavior_affecting": true,
  "mode": "verify-proof",
  "owner_state": 4,
  "rerun_from": 4,
  "status": "planned"
}
```

### Flux — refinement type

```json
{
  "schema_version": "proof-obligation/v1",
  "id": "PO-047",
  "requirement_id": "REQ-120",
  "contract_clause": "C-022",
  "domain_claim": "crate::index::Offset stays within [0, crate::index::LEN) on construction",
  "risk": "refinement",
  "risk_tags": ["refinement", "ownership", "index"],
  "verifier": "flux-rs",
  "artifact": "src/index.rs",
  "target": "crate::index::Offset::new",
  "command": "cargo flux --lib -p index --features=verified",
  "workdir": "/abs/path/to/bead-workspace",
  "expected_evidence": "Flux check reports no postcondition failures, no `#[trusted]` widening introduced by the obligation, and a paired negative test for invalid construction",
  "assumptions": [],
  "model_bounds": {},
  "tool_metadata": { "tool": "cargo-flux", "version_pin": "flux@nightly-2026-02-15", "solver": "liquid-fixpoint", "backend": "z3" },
  "trusted_base_refs": [],
  "required": true,
  "behavior_affecting": true,
  "mode": "verify-proof",
  "owner_state": 4,
  "rerun_from": 4,
  "status": "planned"
}
```

### Loom — concurrency interleaving

```json
{
  "schema_version": "proof-obligation/v1",
  "id": "PO-052",
  "requirement_id": "REQ-133",
  "contract_clause": "C-031",
  "domain_claim": "crate::queue::MpscQueue drains all messages on shutdown without losing items",
  "risk": "concurrency_interleaving",
  "risk_tags": ["concurrency", "channel", "shutdown"],
  "verifier": "loom",
  "artifact": "tests/loom/mpsc_drain.rs",
  "target": "crate::queue::MpscQueue",
  "command": "RUSTFLAGS=\"--cfg loom\" LOOM_MAX_PREEMPTIONS=3 LOOM_MAX_BRANCHES=20000 cargo test --test mpsc_drain --release",
  "workdir": "/abs/path/to/bead-workspace",
  "expected_evidence": "Loom test passes across all explored schedules; the harness uses the production sync indirection (`crate::sync::sync::*`), not bare `std::sync::*`",
  "assumptions": ["bounded message count", "bounded waiters"],
  "model_bounds": { "threads": 4, "unwind": 8 },
  "tool_metadata": { "tool": "loom", "version_pin": "loom@0.7", "feature_flags": ["futures", "checkpoint"] },
  "trusted_base_refs": [],
  "required": true,
  "behavior_affecting": true,
  "mode": "verify-proof",
  "owner_state": 4,
  "rerun_from": 4,
  "status": "planned"
}
```

### Miri — UB detection

```json
{
  "schema_version": "proof-obligation/v1",
  "id": "PO-066",
  "requirement_id": "REQ-150",
  "contract_clause": "C-046",
  "domain_claim": "crate::arena::Bump::alloc returns a provenance-clean pointer for valid input",
  "risk": "ub_safety",
  "risk_tags": ["ub", "raw_pointer", "provenance"],
  "verifier": "miri",
  "artifact": "tests/miri/bump_alloc.rs",
  "target": "crate::arena::Bump::alloc",
  "command": "MIRIFLAGS=\"-Zmiri-strict-provenance -Zmiri-many-seeds=0..8\" cargo +nightly miri test --test bump_alloc --release",
  "workdir": "/abs/path/to/bead-workspace",
  "expected_evidence": "Miri reports no UB diagnostic across the seeded runs; provenance flags are recorded; skipped tests are listed in `trusted-base-plan.md`",
  "assumptions": ["nightly toolchain pinned in rust-toolchain.toml"],
  "model_bounds": { "seeds": 8 },
  "tool_metadata": { "tool": "cargo-miri", "version_pin": "nightly-2026-02-15", "feature_flags": ["-Zmiri-strict-provenance", "-Zmiri-many-seeds"] },
  "trusted_base_refs": ["TB-066-skipped-tests"],
  "required": true,
  "behavior_affecting": true,
  "mode": "verify-proof",
  "owner_state": 4,
  "rerun_from": 4,
  "status": "planned"
}
```

### cargo-fuzz — hostile input

```json
{
  "schema_version": "proof-obligation/v1",
  "id": "PO-074",
  "requirement_id": "REQ-160",
  "contract_clause": "C-052",
  "domain_claim": "crate::codec::Postcard::decode never panics on arbitrary bytes",
  "risk": "hostile_input",
  "risk_tags": ["parser", "codec", "hostile_input", "fuzzable_canonicalization"],
  "verifier": "cargo-fuzz",
  "artifact": "fuzz/fuzz_targets/postcard_decode.rs",
  "target": "crate::codec::Postcard::decode",
  "command": "cargo fuzz run postcard_decode --sanitizer=address -- -max_total_time=120 -rss_limit_mb=2048 -print_final_stats=1",
  "workdir": "/abs/path/to/bead-workspace",
  "expected_evidence": "Fuzz run reaches the time budget with no crash; coverage or coverage-equivalent counter recorded; corpus preservation path noted",
  "assumptions": ["fuzz build uses --cfg fuzzing"],
  "model_bounds": { "max_total_time": 120, "rss_limit_mb": 2048 },
  "tool_metadata": { "tool": "cargo-fuzz", "version_pin": "cargo-fuzz@0.13", "feature_flags": ["--sanitizer=address"] },
  "trusted_base_refs": [],
  "required": true,
  "behavior_affecting": true,
  "mode": "verify-proof",
  "owner_state": 4,
  "rerun_from": 4,
  "status": "planned"
}
```

### proptest — property pressure

```json
{
  "schema_version": "proof-obligation/v1",
  "id": "PO-080",
  "requirement_id": "REQ-171",
  "contract_clause": "C-061",
  "domain_claim": "sort is idempotent over arbitrary Vec<i64> within size bound",
  "risk": "ordering",
  "risk_tags": ["property", "ordering"],
  "verifier": "proptest",
  "artifact": "tests/proptest/sort_idempotent.rs",
  "target": "crate::sort::stable_sort",
  "command": "PROPTEST_CASES=10000 cargo test --test sort_idempotent --release",
  "workdir": "/abs/path/to/bead-workspace",
  "expected_evidence": "Test passes for the documented case count, with a non-vacuity strategy (size 0..1024, any i64) and an explicit anti-invariant",
  "assumptions": [],
  "model_bounds": { "input_size": 1024, "cases": 10000 },
  "tool_metadata": { "tool": "proptest", "version_pin": "proptest@1.5" },
  "trusted_base_refs": [],
  "required": true,
  "behavior_affecting": true,
  "mode": "verify-proof",
  "owner_state": 4,
  "rerun_from": 4,
  "status": "planned"
}
```

## Validator Behavior

`tools/go-skill-v9-validate` checks each row for:

- `schema_version` literal match.
- All required fields present and non-empty (unless the field is an array/object that may be empty).
- `verifier` in the allowed set.
- `mode` in `{verify-proof, verify-smoke}`; `verify-smoke` requires a paired `verify-proof` row whose `id` is referenced in `smoke_pair_id`.
- `target` parses as `path::symbol` or `crate::module::symbol`.
- `command` does not contain `<`, `>`, `...`, or unquoted `placeholder`.
- `behavior_affecting: true` ⇒ a paired `rust-refinement-obligation/v1` row exists in `proof-to-implementation-input.md` or the bridge MUST materialize one.
- `assumptions` non-empty ⇒ each assumption ID exists in `trusted-base-plan.md`.
- `model_bounds` for bounded verifiers (`kani`, `loom`, `cargo-fuzz`, `proptest`) MUST include `mem_high`/`mem_max` for Kani, `max_total_time`/`rss_limit_mb` for fuzz, `threads`/`unwind` for Loom, and `cases`/`input_size` for proptest.
- For `behavior_affecting: true`, `waiver-candidates.jsonl` MUST NOT contain a row with the same `requirement_id` and `behavior_affecting: true`.
