# Workflow: Concurrency Channel (MpscQueue::shutdown_drain)

End-to-end example of a `proof-planner` run for a multi-producer single-consumer queue with a shutdown drain protocol. This seed shape exercises `loom` (primary, interleavings + temporal surface), `kani` (bounded drain harness), and `proptest` (randomized pressure). TLA+ is removed.

## Input (from `rust-contract`)

`proof-seeds.jsonl`:

```jsonl
{"schema_version": "proof-seed/v1", "id": "PS-133", "requirement_id": "REQ-133", "contract_clause": "C-031", "domain_claim": "MpscQueue drains all messages on shutdown without losing items", "risk_tags": ["concurrency", "channel", "shutdown", "task_ownership"], "suggested_layers": ["loom", "kani", "proptest"], "behavior_affecting": true, "model_boundary": "async_shell", "notes": "Shutdown drain protocol; loom primary (temporal surface + interleavings)."}
```

`contract.md` says (clause C-031):

> `MpscQueue::shutdown(self)` MUST drain all in-flight messages and signal the receiver that no more messages will arrive. It MUST NOT lose messages; it MUST NOT deadlock; it MUST NOT leave the receiver blocked forever.

`boundary-map.md` notes that the queue uses `cfg(loom)`-aware sync indirection.

## Lane Profile

| Verifier | Decision | Reason |
|---|---|---|
| `loom` | `required` | `concurrency + channel + shutdown` |
| (TLA+ removed; loom covers the temporal surface) | n/a | `shutdown` is proved by loom (interleavings) + proptest |
| `kani` | `required` | Bounded drain harness |
| `proptest` | `required` | Default profile companion |
| `verus` | `not_applicable` | No unbounded Rust-local invariant in scope |
| `flux-rs` | `not_applicable` | No refinement expressible |
| `miri` | `not_applicable` | No unsafe in the queue |
| `cargo-fuzz` | `not_applicable` | No parser/codec boundary |

## Obligations

`proof-obligations.planned.jsonl`:

```jsonl
{"schema_version": "proof-obligation/v1", "id": "PO-052", "requirement_id": "REQ-133", "contract_clause": "C-031", "domain_claim": "MpscQueue drains all messages on shutdown across explored interleavings", "risk": "concurrency_interleaving", "risk_tags": ["concurrency", "channel", "shutdown"], "verifier": "loom", "artifact": "tests/loom/mpsc_drain.rs", "target": "crate::queue::MpscQueue", "command": "RUSTFLAGS=\"--cfg loom\" LOOM_MAX_PREEMPTIONS=3 LOOM_MAX_BRANCHES=20000 cargo test --test mpsc_drain --release", "workdir": "/abs/path/to/bead-workspace", "expected_evidence": "Loom test passes across all explored schedules (test result: ok); harness uses crate::sync::sync::* (cfg(loom)-aware); cancellation, drop, timeout, and shutdown paths exercised.", "assumptions": ["bounded message count", "bounded waiters"], "model_bounds": {"threads": 4, "unwind": 8, "preemptions": 3, "branches": 20000}, "tool_metadata": {"tool": "loom", "version_pin": "loom@0.7", "feature_flags": ["futures", "checkpoint"]}, "trusted_base_refs": [], "required": true, "behavior_affecting": true, "mode": "verify-proof", "owner_state": 4, "rerun_from": 4, "status": "planned"}
{"schema_version": "proof-obligation/v1", "id": "PO-052-KANI", "requirement_id": "REQ-133", "contract_clause": "C-031", "domain_claim": "MpscQueue bounded drain: shutdown drains all messages within bounded operations", "risk": "bounded_transition", "risk_tags": ["bounded_state", "concurrency"], "verifier": "kani", "artifact": "harnesses/kani/mpsc_drain_bounded.rs", "target": "crate::queue::MpscQueue", "command": "cargo kani -j 1 --output-format=regular --harness mpsc_drain_bounded", "workdir": "/abs/path/to/bead-workspace", "expected_evidence": "Kani reports VERIFICATION:- SUCCESSFUL for mpsc_drain_bounded; harness models shutdown as a deterministic sequence and asserts the queue is empty after shutdown; kani::assert enforces the property.", "assumptions": ["bounded senders (3)", "bounded messages per sender (4)"], "model_bounds": {"unwind": 32, "input_size": 12, "mem_high": "20G", "mem_max": "24G"}, "tool_metadata": {"tool": "cargo-kani", "version_pin": "cargo-kani@0.65", "feature_flags": [], "solver": "cbmc"}, "trusted_base_refs": [], "required": true, "behavior_affecting": true, "mode": "verify-proof", "owner_state": 4, "rerun_from": 4, "status": "planned"}
{"schema_version": "proof-obligation/v1", "id": "PO-052-PROPTEST", "requirement_id": "REQ-133", "contract_clause": "C-031", "domain_claim": "MpscQueue property: shutdown drains all messages across randomized interleavings", "risk": "concurrency_interleaving", "risk_tags": ["concurrency", "channel"], "verifier": "proptest", "artifact": "tests/proptest/mpsc_drain.rs", "target": "crate::queue::MpscQueue", "command": "PROPTEST_CASES=1000 cargo test --test mpsc_drain --release", "workdir": "/abs/path/to/bead-workspace", "expected_evidence": "Test passes for 1000 cases (test result: ok); strategy uses prop::collection::vec(any::<u8>(), 0..64) for message bodies; anti-invariant asserts that sending after shutdown is rejected.", "assumptions": [], "model_bounds": {"cases": 1000, "input_size": 64}, "tool_metadata": {"tool": "proptest", "version_pin": "proptest@1.5"}, "trusted_base_refs": [], "required": true, "behavior_affecting": true, "mode": "verify-proof", "owner_state": 4, "rerun_from": 4, "status": "planned"}
```

## Loom Harness (production sync indirection)

`tests/loom/mpsc_drain.rs`:

```rust
use crate::sync::sync::{Arc, mpsc, thread};
use crate::queue::MpscQueue;

#[test]
fn mpsc_drain_loom() {
    loom::model(|| {
        let queue = Arc::new(MpscQueue::new(16));
        let q1 = queue.clone();
        let producer = thread::spawn(move || {
            for i in 0..4u8 {
                q1.send(i).unwrap();
            }
        });

        let q2 = queue.clone();
        let drainer = thread::spawn(move || {
            q2.shutdown();
            // Drain must yield all 4 messages.
            let drained: Vec<u8> = q2.into_iter().collect();
            assert_eq!(drained.len(), 4);
        });

        producer.join().unwrap();
        drainer.join().unwrap();
    });
}
```

The harness uses `crate::sync::sync::*` (not `std::sync::*`) so `cfg(loom)` swaps in `loom::sync::*`. Without this, the harness is `E_LOOM_TOY_MODEL`.

## Trusted-Base Plan Entry

(TLA+ removed. Loom covers the temporal surface; the trusted-base ledger does not need a TLC fairness assumption.)

## Bridge Input

`proof-to-implementation-input.md`:

> **PO-052 (loom).** Source: `crates/queue/src/lib.rs::queue::MpscQueue::shutdown`. Behavior test: `tests/mpsc_drain_test.rs::shutdown_drains_all`. Refinement harness: `tests/loom/mpsc_drain.rs` (uses `crate::sync::sync::*`). Evidence: `RUSTFLAGS="--cfg loom" LOOM_MAX_PREEMPTIONS=3 cargo test --test mpsc_drain --release`.
>
> **PO-052-KANI (kani).** Same source. Behavior test: `tests/mpsc_drain_bounded_test.rs`. Refinement harness: `harnesses/kani/mpsc_drain_bounded.rs`. Evidence: `cargo kani -j 1 --harness mpsc_drain_bounded`.
>
> **PO-052-PROPTEST (proptest).** Same source. Behavior test: `tests/proptest/mpsc_drain.rs`. Refinement harness: n/a. Evidence: `PROPTEST_CASES=1000 cargo test --test mpsc_drain --release`.

## Self-Audit

- Loom: `--cfg loom`, `--release`, threads=4, preemptions=3, harness uses `crate::sync::sync::*`. ✓
- Kani: `-j 1`, `mem_high/mem_max`, paired `kani::assert`. ✓
- proptest: `PROPTEST_CASES=1000`, anti-invariant "sending after shutdown is rejected". ✓

Expected: `status: PASS, findings_total: 0, exit 0`.

## Next Steps

Standard pipeline: reviewer → writer → reviewer → bridge → formal-verifier. The writer authors the Loom harness, the Kani harness, and the proptest strategy.
