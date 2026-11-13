# Evidence Requirements

Every `proof-obligation/v1` row MUST specify enough evidence that `formal-verifier` can run a single command and observe a single, deterministic pass/fail. "Run the tests" is not evidence. "Verify it works" is not evidence. This reference is the per-lane minimum evidence contract.

## Evidence Discipline

For each required lane, the obligation MUST name:

- The exact command (no placeholders, no `<harness>`, no ellipsis).
- The exact workdir (absolute path, not relative).
- The expected evidence artifact path (where the raw log lands).
- The expected stdout/stderr marker (a literal string the tool emits on success, e.g., `VERIFICATION:- SUCCESSFUL`, `loom: PASSED`).
- The tool version pin (e.g., `cargo-kani@0.65`, `verus@0.2024.10`).
- The feature flags and solver flags (e.g., `--mem-predicates`, `-Zmiri-strict-provenance`, `--symmetry off`).

The reviewer (`proof-plan-reviewer`) rejects obligations whose `expected_evidence` is vague. The formal verifier (`formal-verifier`) rejects evidence rows whose command output does not contain the expected marker.

## Per-Lane Evidence Contract

### Verus

| Field | Required value |
|---|---|
| `command` | exact `verus` invocation with crate paths and entry files |
| `workdir` | absolute path |
| `expected_evidence` | literal `SUCCESS` or `Verification SUCCEEDED` line, plus a confirmation that no `external_body`, `assume`, or `axiom` markers are present |
| `flags` | any `--crate-type`, `--edition`, solver flags |
| `tool_metadata.tool` | `verus` |
| `tool_metadata.version_pin` | `verus@<version>` or `verus@<commit>` |
| `raw_log` | absolute path to the verifier's stdout/stderr |
| `evidence_artifact` | absolute path to the postcondition report or summary file |

Example `expected_evidence`: "Verus reports `Verification SUCCEEDED` for the named exec fn; `rg -n '#\[verifier::external_body\]|assume\(|axiom' verification/verus/` is empty."

### Kani

| Field | Required value |
|---|---|
| `command` | exact `cargo kani -j 1 --output-format=regular --harness <name>` plus any feature flags |
| `workdir` | absolute path |
| `expected_evidence` | literal `VERIFICATION:- SUCCESSFUL` line, harness name, check count, and a `kani::cover` reachability summary when assumptions are present |
| `flags` | `--mem-predicates`, `-Z stubbing`, `-Z function-contracts`, `--output-format=regular`; never `--no-default-checks`, `--no-memory-safety-checks`, `--no-overflow-checks`, `--no-unwinding-checks`, `--only-codegen`, `--no-codegen`, `--prove-safety-only` |
| `tool_metadata.tool` | `cargo-kani` |
| `tool_metadata.version_pin` | `cargo-kani@<version>` |
| `model_bounds` | unwind, input_size, mem_high, mem_max |
| `raw_log` | absolute path to Kani stdout/stderr |
| `evidence_artifact` | absolute path to SARIF or summary JSON when produced |

Example `expected_evidence`: "Kani reports `VERIFICATION:- SUCCESSFUL` for `frame_decode_rejects_truncation` with check_failure=0 and a paired `kani::cover` reachability entry proving the truncated-input branch is reachable."

### Flux

| Field | Required value |
|---|---|
| `command` | exact `cargo flux --lib -p <crate> [--features ...]` invocation |
| `workdir` | absolute path |
| `expected_evidence` | literal `check passed` or absence of postcondition failures, plus confirmation that no `#[trusted]`, `#[ignore]`, `extern_spec`, or `opaque` markers were broadened by this obligation |
| `flags` | any `-F` solver flags, include patterns, solver selection |
| `tool_metadata.tool` | `cargo-flux` |
| `tool_metadata.version_pin` | `flux@nightly-YYYY-MM-DD` |
| `raw_log` | absolute path to Flux output |
| `evidence_artifact` | absolute path to constraint dump or solver log when produced |

Example `expected_evidence`: "Flux check reports no postcondition failures for `crate::index::Offset::new`; a paired negative test `crate::index::Offset::new_negative` reports `should_fail` as expected."

### Loom

| Field | Required value |
|---|---|
| `command` | exact `RUSTFLAGS="--cfg loom" [LOOM_*] cargo test --test <name> --release` |
| `workdir` | absolute path |
| `expected_evidence` | literal `test result: ok` plus the harness count, plus confirmation that the production sync indirection is in use |
| `flags` | `RUSTFLAGS=--cfg loom`, `LOOM_MAX_PREEMPTIONS`, `LOOM_MAX_BRANCHES`, `--release` |
| `tool_metadata.tool` | `loom` |
| `tool_metadata.version_pin` | `loom@<version>` |
| `model_bounds` | threads, preemptions, branches |
| `raw_log` | absolute path to test stdout/stderr |
| `evidence_artifact` | optional `LOOM_CHECKPOINT_FILE` path when used |

Example `expected_evidence`: "Loom test passes across all explored schedules (`test result: ok. N passed`), harness uses `crate::sync::sync::*` (cfg(loom)-aware), and the harness is `--release`."

(TLA+ removed. Temporal workflows are proved by loom + proptest.)

### Miri

| Field | Required value |
|---|---|
| `command` | exact `MIRIFLAGS="..." cargo +nightly miri test --test <name> --release` |
| `workdir` | absolute path |
| `expected_evidence` | literal `test result: ok` plus the absence of UB diagnostics, plus the MIRIFLAGS used and the seed range |
| `flags` | `-Zmiri-strict-provenance`, `-Zmiri-many-seeds=0..N`, `-Zmiri-tree-borrows`, `-Zmiri-track-alloc-id`, `-Zmiri-backtrace=full` |
| `tool_metadata.tool` | `cargo-miri` |
| `tool_metadata.version_pin` | `nightly-YYYY-MM-DD` with miri component |
| `model_bounds` | seeds |
| `raw_log` | absolute path to Miri stdout/stderr |
| `evidence_artifact` | optional Miri trace path when used |

Example `expected_evidence`: "Miri reports no UB diagnostic across 8 seeded runs (`test result: ok. N passed`) for `crate::arena::Bump::alloc`; provenance flags `-Zmiri-strict-provenance` recorded; one test (`cfg_attr(miri, ignore)`) skipped and noted in `trusted-base-plan.md`."

### cargo-fuzz

| Field | Required value |
|---|---|
| `command` | exact `cargo fuzz run <target> --sanitizer=<address|memory|thread|undefined> -- -max_total_time=<N> -rss_limit_mb=<N> -print_final_stats=1` |
| `workdir` | absolute path |
| `expected_evidence` | literal fuzz completion line (`#<N> NEW    cov: ...`) and absence of crashes, plus corpus preservation path |
| `flags` | `--sanitizer=address`, `-max_total_time`, `-rss_limit_mb`, `-print_final_stats=1` |
| `tool_metadata.tool` | `cargo-fuzz` |
| `tool_metadata.version_pin` | `cargo-fuzz@<version>` with `nightly-YYYY-MM-DD` |
| `model_bounds` | max_total_time, rss_limit_mb |
| `raw_log` | absolute path to fuzz stdout/stderr |
| `evidence_artifact` | absolute path to preserved corpus directory |

Example `expected_evidence`: "Fuzz run reaches the time budget (120s) with no crash; `#<N> NEW    cov: <X>` recorded; corpus preserved at `<workdir>/fuzz/corpus/postcard_decode`."

### proptest

| Field | Required value |
|---|---|
| `command` | exact `PROPTEST_CASES=<N> cargo test --test <name> --release` |
| `workdir` | absolute path |
| `expected_evidence` | literal `test result: ok` plus the case count, plus a confirmation that the strategy is non-vacuous and the anti-invariant is exercised |
| `flags` | `PROPTEST_CASES`, `--release` |
| `tool_metadata.tool` | `proptest` |
| `tool_metadata.version_pin` | `proptest@<version>` |
| `model_bounds` | cases, input_size |
| `raw_log` | absolute path to test stdout/stderr |
| `evidence_artifact` | optional proptest regression file when produced |

Example `expected_evidence`: "Test passes for 10000 cases (`test result: ok. 1 passed; 0 failed`) with a non-vacuous strategy (`prop::collection::vec(any::<i64>(), 0..1024)`) and an anti-invariant that exercises at least one invalid input."

## Anti-Evidence Patterns

The following patterns are rejected by `proof-plan-reviewer` and `formal-verifier`:

- "Verify it works" — no command, no marker.
- "Run cargo test" — generic, no harness.
- "All tests pass" — no evidence of which tests.
- "We expect Verus to accept it" — no failure marker, no version pin.
- "Kani reports success" — no command, no harness, no bounds.
- "TLC verifies the model" — no model, no config, no constants.
- "The fuzz run completed" — no `-max_total_time`, no `-rss_limit_mb`, no sanitizer.
- "Miri passes" — no `MIRIFLAGS`, no nightly pin, no seed range.
- "Loom explored all schedules" — no `LOOM_MAX_PREEMPTIONS`, no `--release`, no harness name.

These are caught by `E_COMMAND_EVIDENCE_MISSING` and `E_PROOF_SMOKE_MISSING`.

## Multi-Lane Evidence

When a seed is covered by multiple lanes, each lane has its own evidence. The `formal-verifier` writes one `verification-ledger/v1` row per obligation. The `evidence-packaging` skill produces one requirement-to-evidence row per required lane.

Cross-lane evidence (e.g., one Kani run satisfying both `bounded_state` and `rejection`) is allowed only when:

- The same harness satisfies both claims (named in `expected_evidence`).
- The harness binds to the same production target.
- The `expected_evidence` explicitly lists both claims satisfied.

Cross-lane evidence without this disambiguation is `E_LANE_EVIDENCE_UNDISAMBIGUATED`.

## Resource Budgets in Evidence

`model_bounds` is part of the evidence contract. The formal verifier MUST honor `mem_high`, `mem_max`, `max_total_time`, `rss_limit_mb`, `unwind`, `cases`, and `threads` from the obligation. If the verifier cannot honor the bound (e.g., Kani cannot complete within `mem_max`), the row is `FAIL_GLOBAL` with a classification citing the bound.

The formal verifier MUST NOT silently relax bounds to make a run succeed. Bound relaxation is a `E_BOUND_RELAXED_UNAUTHORIZED` finding.
