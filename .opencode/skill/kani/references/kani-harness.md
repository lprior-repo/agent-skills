# Kani Harness And Evidence Guide

This skill is CLI-first. Editor integration is irrelevant to proof evidence. Exact commands, outputs, bounds, and artifacts are the evidence.

## Install And Discovery

Template commands:

```bash
cargo install --locked kani-verifier
cargo kani setup
cargo kani --version
if command -v kani >/dev/null; then kani --version; fi
cargo kani --help
rustc --version --verbose
cargo --version
rustup show active-toolchain
```

Do not install or mutate the toolchain in this workflow. If Kani is required and missing, report `BLOCKER` with the required install command for operator action.

## Harness Inventory

Use Kani's list command when available:

```bash
cargo kani list --format json > kani-evidence/harnesses.json
ruby -rjson -e 'JSON.parse(File.read(ARGV.fetch(0)))' kani-evidence/harnesses.json
cargo kani list --format markdown
```

Validate that `harnesses.json` is actual JSON harness inventory, not a status line or generated-path message. If the local Kani version writes JSON to a different path, record that path and validate that artifact instead.

For single-file mode:

```bash
kani list <file.rs> --format json
```

Use source scans as a cross-check:

```bash
rg -n '#\[kani::proof(\([^\]]*\))?\]|#\[kani::proof_for_contract(\([^\]]*\))?\]' --glob '*.rs' --glob '!**/target/**' <verified-scope>
rg -n '#\[kani::should_panic(\([^\]]*\))?\]' --glob '*.rs' --glob '!**/target/**' <verified-scope>
```

Every accepted claim needs a harness-to-claim mapping.

## Standard Commands

Run all package harnesses:

```bash
cargo kani
```

Run a workspace:

```bash
cargo kani --workspace
```

Run harnesses under test modules:

```bash
cargo kani --tests
```

Run one harness:

```bash
cargo kani --harness <harness-name>
```

Run an exact fully qualified harness:

```bash
cargo kani --harness <module::path::harness> --exact
```

Select package, target, and features:

```bash
cargo kani --package <package>
cargo kani --manifest-path <Cargo.toml>
cargo kani --lib
cargo kani --bin <name>
cargo kani --features <features>
cargo kani --all-features
cargo kani --no-default-features
```

Record solver and backend options as proof context when present:

```bash
cargo kani --solver <solver> --harness <harness-name>
cargo kani --harness <harness-name> --cbmc-args <cbmc-args>
```

Use the exact command from `proof-obligations.planned.jsonl` or `verification-ledger.jsonl` when present. Do not silently broaden or narrow package, feature, or harness scope.

## Unwinding

Prefer per-harness attributes for stable proof context:

```rust
#[kani::proof]
#[kani::unwind(11)]
fn verify_len_le_10() { }
```

Fallback command options:

```bash
cargo kani --default-unwind <N>
cargo kani --harness <harness> --unwind <N>
```

Evidence must report:

- Problem bound, such as `len <= 10`.
- Unwind source, such as `#[kani::unwind(11)]` or `--default-unwind 11`.
- Whether all unwinding assertions passed.

Any unwinding assertion failure is a failed proof. Do not disable or ignore unwind checks to get a green run.

## Assumption, Stub, Contract, And Unsafe Scans

Run these scans over the verified scope:

```bash
rg -n 'kani::assume|any_where|#\[kani::requires' --glob '*.rs' --glob '!**/target/**' <verified-scope>
rg -n 'kani::cover!?\(' --glob '*.rs' --glob '!**/target/**' <verified-scope>
rg -n '#\[kani::stub|#\[kani::stub_verified|stub_|mock_' --glob '*.rs' --glob '!**/target/**' <verified-scope>
rg -n '#\[kani::(requires|ensures|modifies|recursion|proof_for_contract|stub_verified|loop_invariant|loop_modifies|loop_decreases|solver)' --glob '*.rs' --glob '!**/target/**' <verified-scope>
rg -n 'kani::mem::|derive\((kani::)?Arbitrary\)|derive\((kani::)?BoundedArbitrary\)|impl([[:space:]]*<[^>]+>)?[[:space:]]+(kani::)?Arbitrary[[:space:]]+for|impl([[:space:]]*<[^>]+>)?[[:space:]]+(kani::)?BoundedArbitrary[[:space:]]+for|#\[bounded' --glob '*.rs' --glob '!**/target/**' <verified-scope>
rg -n '\bunsafe\b|unsafe\s*\{|unsafe\s+fn|unsafe\s+impl|transmute|MaybeUninit|from_raw_parts|from_raw|as_ptr|as_mut_ptr|asm!|global_asm!' --glob '*.rs' --glob '!**/target/**' <verified-scope>
rg -n 'bounded_any|BoundedArbitrary|const\s+\w+\s*:\s*usize|LIMIT|MAX' --glob '*.rs' --glob '!**/target/**' <verified-scope>
```

Report each match as verified context, trusted context, or irrelevant to the selected harnesses.

## Experimental Feature Commands

Use only when current Kani docs and local help support the feature.

Contracts:

```bash
cargo kani -Z function-contracts --harness <contract-harness>
```

Plain stubs:

```bash
cargo kani -Z stubbing --harness <stubbed-harness>
```

Verified stubs are still active abstractions in the caller harness. Require both contract proof evidence and caller evidence. Use `-Z function-contracts -Z stubbing` unless local help or official docs for the installed Kani version prove a different exact flag set:

```bash
cargo kani -Z function-contracts --harness <contract-harness>
cargo kani -Z function-contracts -Z stubbing --harness <caller-using-stub_verified>
```

Loop contracts:

```bash
cargo kani -Z loop-contracts --harness <harness>
```

Coverage/non-vacuity exploration:

```bash
cargo kani --coverage -Z source-coverage --harness <harness>
```

Invalid-value or uninitialized-memory checks:

```bash
cargo kani -Z valid-value-checks --harness <harness>
cargo kani -Z uninit-checks --harness <harness>
```

Memory predicate APIs such as `kani::mem::can_dereference`:

```bash
cargo kani -Z mem-predicates --harness <harness-using-kani-mem-predicates>
```

Concrete playback for failures:

```bash
cargo kani -Z concrete-playback --concrete-playback=print --harness <failing-harness>
cargo kani playback -Z concrete-playback -- <generated-test-name>
```

Do not use `--concrete-playback=inplace` in this workflow; it mutates source. Use printed playback or route source-edit needs through the owning implementation workflow.

## Evidence Artifact Set

For serious work, create or request an evidence directory:

```text
kani-evidence/
  manifest.json
  commands.jsonl
  harnesses.json
  kani.stdout.txt
  kani.stderr.txt
  kani.exitcode.txt
  kani.sarif
  proof-results.json
  property-summary.json
  concrete-playback/
```

Minimum reproducibility fields:

- Working directory and exact argv.
- Git commit and dirty status when available.
- `rustc --version --verbose`.
- `cargo kani --version` and `kani --version` when available.
- `cargo metadata --format-version=1` for Cargo projects.
- `Cargo.lock` hash when present.
- Package, target, feature, and harness selection.
- Solver selection, backend options, and `--cbmc-args` when present.
- Disabled, skipped, or weakening verification flags, including any `--no-*checks`, `--prove-safety-only`, `--only-codegen`, `--no-codegen`, or `--ignore-global-asm`.
- Environment variables that affect Rust/Kani, such as `KANI_HOME`, `RUSTFLAGS`, and `RUSTUP_TOOLCHAIN`.

## Accept / Reject Triage

Accept only when required harnesses show `VERIFICATION:- SUCCESSFUL`, all expected cover points are satisfied, all unwinding assertions pass, and no required property is failed, undetermined, unsupported, or timed out.

Reject or block on these output signals:

```text
VERIFICATION:- FAILED
Failed Checks
unwinding assertion
UNDETERMINED
UNREACHABLE cover point needed for non-vacuity
UNSATISFIABLE cover point needed for a boundary
unsupported feature
out of memory
timeout
```

Reject or downgrade any command that weakens the checked property classes unless the waiver is explicit and the final claim no longer relies on the disabled checks:

```text
--no-default-checks
--no-memory-safety-checks
--no-overflow-checks
--no-undefined-function-checks
--no-unwinding-checks
--no-assertion-reach-checks
--prove-safety-only
--only-codegen
--no-codegen
--ignore-global-asm
```

If output contains active stubs, contract abstractions, or experimental feature warnings, the report must account for them before claiming success.

## Report Template

```markdown
## Kani Evidence

- Verdict: PASS | FAIL | BLOCKED | INCONCLUSIVE
- Commands: `<exact command>`
- Kani/Rust: `<versions>`
- Harness inventory: `<artifact or summary>`
- Harnesses checked: `<names>`
- Solver/backend: `<solver, cbmc args, or none>`
- Disabled checks: `<flags or none>`
- Bounds/unwind: `<input bounds and unwind source>`
- Assumptions/covers: `<assumptions and non-vacuity evidence>`
- Stubs/contracts/unsafe: `<trusted or verified surfaces>`
- Result: `<actual summary>`
- Counterexample: `<actual trace/playback or none>`
- Limitations: `<bounded/unsupported/residual risks>`
```
