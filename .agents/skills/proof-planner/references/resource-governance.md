# Resource Governance

Verification tools are resource-bound. CBMC, fuzzing, mutation testing, and Miri can each consume tens of GiB per run. Without resource governance, a single broad run can swap-storm a workstation. This reference defines the minimum resource budget per lane and the rules for exceeding them.

The planner MUST encode each budget in `proof-obligation/v1.model_bounds` and `proof-obligation/v1.tool_metadata`. The formal verifier MUST honor the budget. Resource relaxation is a finding.

## Universal Rules

- Default to single-worker parallel execution for bounded symbolic and fuzz lanes. `-j > 1` is allowed only with recorded memory headroom and a per-obligation justification.
- Encode `mem_high` and `mem_max` in `model_bounds` for any lane that uses CBMC, fuzzing, or Miri.
- Encode `max_total_time` and `rss_limit_mb` in `model_bounds` for fuzz and cargo-fuzz.
- Encode `unwind` for any lane that uses Kani or Loom with recursion/loop bounds.
- Encode `threads` for any lane that uses Loom or concurrent proptest.
- Encode `cases` for any lane that uses proptest.
- Never run a broad or unknown command without a cgroup cap or container memory limit. See the cgroup template below.

## Per-Lane Budgets

### Verus

| Field | Default |
|---|---|
| CPU parallelism | solver default; typically 1-2 |
| Memory | solver-bound; `mem_high=8G`, `mem_max=16G` |
| Time | unbounded (solver may take hours for deep proofs) |
| Special | none |

Verus is solver-bound. The planner MAY allow unbounded wall time when the obligation's `expected_evidence` explicitly states "verifier may take hours; budget not enforced". The reviewer accepts the unbounded budget only when the obligation's `tool_metadata.solver` is named and the `tool_metadata.solver_version` is pinned.

### Kani (CBMC)

| Field | Default |
|---|---|
| CPU parallelism | `-j 1` mandatory |
| Memory | `mem_high=20G`, `mem_max=24G` |
| Time | 1 hour per harness (smoke); 4 hours per harness (deep) |
| Unwind | per-obligation in `model_bounds.unwind` |
| Input size | per-obligation in `model_bounds.input_size` |
| Special | never use `--no-default-checks`, `--no-memory-safety-checks`, `--no-overflow-checks`, `--no-unwinding-checks`, `--only-codegen`, `--no-codegen`, `--prove-safety-only` |

CBMC memory grows superlinearly. A broad Kani run on a 4-core developer workstation can OOM the machine in minutes. Default to cgroup-capped execution:

```bash
systemd-run --user --scope --collect \
  -p WorkingDirectory=<workspace> \
  -p MemoryHigh=20G \
  -p MemoryMax=24G \
  -p MemorySwapMax=0 \
  cargo kani -j 1 --output-format=regular --harness <name>
```

If `systemd-run` is unavailable, the planner MUST report an equivalent cgroup, container, or `prlimit` mechanism. Running a broad Kani without a memory cap is `E_RESOURCE_UNGOVERNED`.

### Flux

| Field | Default |
|---|---|
| CPU parallelism | 1 |
| Memory | `mem_high=8G`, `mem_max=12G` |
| Time | 30 minutes per crate |
| Special | nightly toolchain required |

Flux is nightly-only and version-sensitive. The planner MUST pin the nightly date and report it in `tool_metadata.version_pin`. Running Flux on stable or floating nightly is `E_FLUX_TOOLCHAIN_UNPINNED`.

### Loom

| Field | Default |
|---|---|
| CPU parallelism | 1 (Loom is single-threaded by design) |
| Memory | standard |
| Time | bounded by `LOOM_MAX_PERMUTATIONS` and `LOOM_MAX_DURATION` |
| Threads | `threads` in `model_bounds`; `max_threads ≤ MAX_THREADS = 5` |
| Preemptions | `LOOM_MAX_PREEMPTIONS=2` smoke; `3` deep |
| Branches | `LOOM_MAX_BRANCHES=10000` smoke; `20000` deep |
| Permutations | `LOOM_MAX_PERMUTATIONS` per claim |
| Special | `--release` mandatory; `RUSTFLAGS="--cfg loom"` mandatory |

Loom without `--release` is impractical (test takes 10x longer). The obligation MUST specify `--release` in `command`. Loom without `cfg(loom)` is `E_LOOM_NOT_MODELED`.

### Miri

| Field | Default |
|---|---|
| CPU parallelism | 1 |
| Memory | standard (Miri is interpreter-bound) |
| Time | 30 minutes per test (smoke); 2 hours per test (deep) |
| Seeds | `seeds` in `model_bounds`; recommend `0..8` smoke, `0..32` deep |
| Special | nightly toolchain + miri component pinned |

Miri is interpreter-only; it scales with interpreted code size, not optimized binary size. Wall time can be 10-100x slower than native. The planner MUST NOT use Miri to substitute for native sanitizer runs; both are complementary.

### cargo-fuzz

| Field | Default |
|---|---|
| CPU parallelism | `-j 1` smoke; `-j 2` deep with recorded memory headroom |
| Memory | `rss_limit_mb=2048` smoke; `rss_limit_mb=4096` deep |
| Time | `-max_total_time=120` smoke; `-max_total_time=1800` deep |
| Corpus size | unbounded but bounded by `-rss_limit_mb` |
| Sanitizer | `--sanitizer=address` default; `--sanitizer=memory` for uninit; `--sanitizer=thread` for races |
| Special | persistent mode is allowed with deterministic reset; `-print_final_stats=1` recommended |

Long-running fuzz campaigns (deep) require an outer shell `timeout` and a cgroup cap. Smoke runs at 120s are the default; longer runs require a bead-level justification and reviewer acceptance.

### proptest

| Field | Default |
|---|---|
| CPU parallelism | test default |
| Memory | standard |
| Time | bounded by `cases` and `input_size` |
| Cases | `cases=10000` minimum; `cases=100000` deep |
| Input size | per-obligation in `model_bounds.input_size` |
| Special | `PROPTEST_CASES=<N>` mandatory in `command` |

A proptest run without an explicit case count is `E_PROPTEST_BUDGET_ABSENT`. Default is 10000; deeper runs require reviewer acceptance.

## Cgroup Template

For CBMC, Miri, fuzz, and any other lane that can grow unbounded:

```bash
systemd-run --user --scope --collect \
  -p WorkingDirectory=<abs-workspace> \
  -p MemoryHigh=<mem_high> \
  -p MemoryMax=<mem_max> \
  -p MemorySwapMax=0 \
  <command>
```

When `systemd-run` is unavailable, the equivalent container or `prlimit` invocation is acceptable. The cgroup cap MUST be present in the raw log evidence path; absence is `E_CGROUP_CAP_ABSENT`.

## Broad-Run Discipline

The following patterns are blocked unless explicitly accepted by the reviewer:

- `cargo kani -j 4` (multi-worker CBMC) without recorded memory headroom.
- `cargo kani --workspace` without per-harness scoping.
- `cargo fuzz run <target>` without `-max_total_time`.
- `cargo +nightly miri test` without `MIRIFLAGS`.
- (TLA+ removed.)
- `cargo mutants` without `--timeout` and per-file scoping.

Each blocked pattern has a finding code:

| Pattern | Finding |
|---|---|
| Kani `-j > 1` without cap | `E_RESOURCE_UNGOVERNED` |
| Fuzz without time budget | `E_FUZZ_BUDGET_ABSENT` |
| Miri without nightly pin | `E_MIRI_TOOLCHAIN_UNPINNED` |
| (TLA+ removed) | n/a |
| proptest without `PROPTEST_CASES` | `E_PROPTEST_BUDGET_ABSENT` |

## Recording the Budget

Each `proof-obligation/v1.model_bounds` is the binding budget. The formal verifier records the actual values used in `verification-ledger/v1.bounds` and `verification-ledger/v1.flags`. A row that ran with relaxed bounds (e.g., `mem_max=64G` when the obligation said `mem_max=24G`) is `E_BOUND_RELAXED_UNAUTHORIZED`.

## Cross-Reference

- `references/evidence-requirements.md` — what evidence each lane must produce.
- `references/tooling-availability-gate.md` — what to do when the tool is missing.
- `references/implementation-binding.md` — anti-laundering rules that govern what the harness actually tests.
