# proof-planner

Contract-bound, machine-checkable architect of proof coverage for Rust beads in the Go-skill
pipeline. Produces planned artifacts at State 4; never writes proof/model/harness code or
production Rust. TLA+ is **removed** from this skill (the temporal surface is proved by
`loom` + `proptest`); a temporal-workflow risk class is still defined but has no TLA+ lane.

## Layout

```
proof-planner/
├── SKILL.md                     # doctrine (compact; cross-links references)
├── evals/evals.json             # 16 risk-class + discipline eval cases
├── examples/
│   ├── good-plan/               # PASS exit 0; reference shape
│   └── bad-plan/                # FAIL exit 1; 54+ findings across every gate
├── references/                  # 16 reference docs
├── templates/                   # 7 JSON templates (one per verifier)
├── workflows/                   # 5 end-to-end planning examples
└── scripts/                     # Rust crate (NO Python validator)
    ├── Cargo.toml
    ├── src/
    │   ├── lib.rs               # pure core: schemas, checks, Patterns
    │   ├── main.rs              # validate-plan binary (IO shell)
    │   └── quick_validate.rs    # quick-validate binary (tree + mirror)
    └── tests/integration.rs     # spawns binaries against the example plans
```

## Build

```bash
cd scripts
cargo build --release
```

Outputs `scripts/target/release/validate-plan` and `scripts/target/release/quick-validate`.

## Run

```bash
# Validate a plan directory (writes JSONL findings + summary to stdout)
./scripts/target/release/validate-plan <plan_dir>

# Structural check of the skill tree + optional mirror parity
./scripts/target/release/quick-validate [--mirror <mirror_path>]
```

Exit codes:
- `0` — PASS (no blocker findings; --strict also requires zero major)
- `1` — FAIL (one or more blocker findings; or major findings under --strict)
- `2` — internal error (IO, serialization)

## Test

```bash
cd scripts
cargo test --release          # 49 tests (43 unit + 6 integration)
cargo clippy --release --all-targets -- \
    -D warnings \
    -D clippy::unwrap_used \
    -D clippy::expect_used \
    -D clippy::panic
```

## Gates Enforced

The validator mechanically enforces the 10 gates documented in
`references/plan-quality-gates.md` and surfaces them as `finding/v1` JSONL with codes
defined in `../go-skill/references/finding-codes.md`:

- Schema (22 fields per obligation, 12 fields per lane decision, 11 per waiver)
- Lane pairing (one required lane per obligation, no duplicates)
- Anti-laundering (verifies, kani cover/assert, loom sync indirection, miri scoping)
- Resource governance (CBMC `-j 1` + `mem_high/mem_max`, fuzz `-max_total_time`, proptest
  `PROPTEST_CASES`, loom `--release` + `RUSTFLAGS=--cfg loom`)
- Waiver lifecycle (no behavior waivers, ISO-8601 expiry, named owner, boundary proof,
  compensating evidence)
- Artifact-source checks (fuzz oracle, flux negative, verus exec fn, kani assert, loom
  sync, proptest anti-invariant)

## Default Risk Profile

`lib.rs::DEFAULT_RISK_PROFILE` is the canonical mapping; every obligation whose
`risk_tags` map to a covered class is required to be paired with the named verifier lanes.
Replacing or skipping a default-profile verifier requires a non-behavior waiver citing
concrete evidence refs and reviewer acceptance.

| Risk | Required lanes |
|---|---|
| `arithmetic_overflow`, `index_safety`, `panic_freedom` | `verus` + `kani` + `proptest` |
| `rejection` | `kani` + `proptest` |
| `bounded_transition` | `kani` + `verus` |
| `equality`, `ordering` | `proptest` + `verus` |
| `field_sensitivity` | `proptest` + `kani` |
| `illegal_state`, `refinement` | `flux-rs` + `verus` |
| `concurrency_interleaving`, `cancellation_safety`, `shutdown_drain` | `loom` + `proptest` |
| `temporal_liveness` | `loom` + `proptest` |
| `temporal_safety` | `verus` + `loom` |
| `ub_safety` | `miri` + `verus` + `proptest` |
| `hostile_input`, `parse_canonicalization` | `cargo-fuzz` + `kani` (+ `proptest`) |

## Mirror Discipline

This skill is mirrored between `~/.agents/skills/proof-planner/` and
`~/.opencode/skill/proof-planner/`. After every change, both trees MUST be byte-identical
(except `examples/` and `evals/evals.json`, which may diverge). Use:

```bash
rsync -a --delete ~/.agents/skills/proof-planner/ ~/.opencode/skill/proof-planner/
./scripts/target/release/quick-validate --mirror ~/.opencode/skill/proof-planner
```

The Go-skill mirror gate (`tools/go-skill-v9-validate`) enforces byte-identity at the
validator level.

## Discipline

- **Zero panic / unwrap / expect** in production code. `Patterns::new()` is the only
  exception; it is gated by the `patterns_new_compiles_all_regexes` unit test.
- **Zero clippy warnings** under `-D warnings -D clippy::unwrap_used -D clippy::expect_used
  -D clippy::panic`. Run as part of the mandatory verification gate.
- **TLA+ removed**: there is no `tla-plus` verifier. Do not reintroduce it. Temporal
  workflows use `loom` for interleavings + `proptest` for randomized pressure; the bridge
  to production source symbols is still required.
