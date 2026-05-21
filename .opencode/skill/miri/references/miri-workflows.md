# Miri Workflows And Evidence Guide

This skill is command-evidence-first. Exact commands, toolchains, flags, targets, skipped tests, and diagnostics are the evidence.

## Install And Setup

Template commands:

```bash
rustup +nightly component add miri
cargo +nightly miri setup
cargo +nightly miri test
```

Do not install or mutate the toolchain in this workflow. If Miri is required and missing, report `BLOCKER` with the required install command for operator action.

For reproducible CI, prefer a pinned nightly in `rust-toolchain.toml`:

```toml
[toolchain]
channel = "nightly-2026-05-01"
components = ["miri", "clippy", "rustfmt"]
profile = "minimal"
```

## Standard Commands

Run setup:

```bash
cargo +nightly miri setup
```

Run all tests in the current package:

```bash
cargo +nightly miri test
```

Select package, target, features, or test filter exactly as with Cargo:

```bash
cargo +nightly miri test -p <package>
cargo +nightly miri test --lib
cargo +nightly miri test --test <test-name>
cargo +nightly miri test --features <features>
cargo +nightly miri test --no-default-features
cargo +nightly miri test <test-filter>
```

Run a binary or example:

```bash
cargo +nightly miri run --bin <bin-name>
cargo +nightly miri run --example <example-name>
```

Clean Miri state when the custom sysroot drifts:

```bash
cargo +nightly miri clean
```

## High-Value MIRIFLAGS

Use these flags only when the local Miri help or README supports them for the current nightly.

Diagnostics:

```bash
MIRIFLAGS="-Zmiri-backtrace=full" cargo +nightly miri test <test-filter>
MIRIFLAGS="-Zmiri-report-progress" cargo +nightly miri test <test-filter>
MIRIFLAGS="-Zmiri-track-alloc-id=<alloc-id>" cargo +nightly miri test <test-filter>
MIRIFLAGS="-Zmiri-track-pointer-tag=<tag>" cargo +nightly miri test <test-filter>
```

Concurrency and weak memory:

```bash
MIRIFLAGS="-Zmiri-many-seeds=0..16" cargo +nightly miri test <test-filter>
MIRIFLAGS="-Zmiri-track-weak-memory-loads" cargo +nightly miri test <test-filter>
MIRIFLAGS="-Zmiri-deterministic-concurrency" cargo +nightly miri test <test-filter>
```

Aliasing and provenance:

```bash
MIRIFLAGS="-Zmiri-tree-borrows" cargo +nightly miri test <test-filter>
MIRIFLAGS="-Zmiri-permissive-provenance" cargo +nightly miri test <test-filter>
```

Isolation and host state:

```bash
MIRIFLAGS="-Zmiri-env-forward=RUST_BACKTRACE" cargo +nightly miri test <test-filter>
MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test <test-filter>
MIRIFLAGS="-Zmiri-isolation-error=warn" cargo +nightly miri test <test-filter>
```

Floating point stabilization:

```bash
MIRIFLAGS="-Zmiri-no-extra-rounding-error" cargo +nightly miri test <test-filter>
MIRIFLAGS="-Zmiri-deterministic-floats" cargo +nightly miri test <test-filter>
```

Do not treat `-Zmiri-permissive-provenance`, native-library bypasses, disabled checks, or broad `cfg(miri)` substitutions as final clean evidence without an explicit waiver.

## Cross-Target Runs

Miri can cross-interpret Rust targets without running on that host OS. Use this for portability checks:

```bash
cargo +nightly miri test --target s390x-unknown-linux-gnu <test-filter>
cargo +nightly miri test --target x86_64-unknown-linux-gnu <test-filter>
```

`s390x-unknown-linux-gnu` is the common big-endian Miri target. On Windows, a Linux interpreted target is often less noisy for host-API support than interpreting as Windows.

## cfg(miri) Discipline

Miri sets `cfg(miri)` while interpreting. Prefer keeping tests visible and ignoring unsupported integrations:

```rust
#[test]
#[cfg_attr(miri, ignore)]
fn real_network_or_os_integration_test() {
    // Keep test discovery and IDE visibility, but skip APIs Miri cannot model.
}
```

Avoid replacing core unsafe logic with a fake safe implementation under `cfg(miri)` unless the evidence claim is explicitly scoped to the fake path. Every skip or divergence is coverage debt.

## no_std Entry Point

For `no_std` binaries, provide the Miri-specific entry point documented by the current Miri README, such as a `#[cfg(miri)]` `miri_start` symbol. Record `MIRI_NO_STD` or `MIRI_SYSROOT` if advanced sysroot control is used.

## CI Pattern

Minimal GitHub Actions pattern:

```yaml
jobs:
  miri:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: rustup toolchain install nightly --component miri
      - run: rustup override set nightly
      - run: cargo miri setup
      - run: cargo miri test
```

For unsafe-heavy crates, add one focused job with many seeds and one big-endian target job if runtime permits.

## nextest Integration

`cargo miri test` normally runs tests more slowly and with less parallelism than native tests. `cargo miri nextest run` can restore process-level parallelism when configured. Tradeoff: per-test processes cannot catch races or shared-resource interactions between tests that a single interpreted test binary could expose.

## Troubleshooting

`cargo miri` missing:

- Cause: not using nightly or miri component is absent.
- Fix: verify `rustup run nightly rustc --version --verbose`; ask before installing `rustup +nightly component add miri`.

`found crate std compiled by an incompatible version of rustc`:

- Cause: custom sysroot drift.
- Fix: run `cargo +nightly miri clean`, then `cargo +nightly miri setup`.

`RUST_BACKTRACE=1` appears ignored:

- Cause: isolation hides host environment.
- Fix: use `-Zmiri-backtrace=full`, `-Zmiri-env-forward=RUST_BACKTRACE`, or carefully disable isolation.

Foreign function or syscall unsupported:

- Cause: API outside Miri's modeled world.
- Fix: isolate with `cfg_attr(miri, ignore)`, use native tests or sanitizers, or document an experimental native-library waiver.

Run is very slow:

- Cause: interpreter cost, many seeds, single-process execution, or heavy diagnostics.
- Fix: narrow test filters, use nextest where appropriate, and reserve weakening flags for temporary triage only.

Run hangs:

- Cause: true infinite loop, unlucky scheduler path, or slow interpretation.
- Fix: add `-Zmiri-report-progress`; for Miri development, use `MIRI_LOG`, `MIRI_BACKTRACE=1`, or tracing.

Clean Miri pass but suspicion remains:

- Cause: insufficient inputs, schedules, targets, or unsupported surface.
- Fix: add focused tests, vary seeds, add cross-target runs, fuzz inputs, and complement with sanitizers or concurrency tools.

## Evidence Checklist

Accepted reports include:

- Exact command and exit status.
- Nightly date or active override, cargo-miri context, host target, and interpreted target.
- Package, target kind, features, and test filter.
- MIRIFLAGS and relevant env vars.
- Seed range and whether deterministic concurrency was used.
- Skipped tests and `cfg(miri)` divergences.
- Source scan for unsafe/provenance/concurrency surfaces.
- Diagnostic category and exact excerpt for failures.
- Unsupported APIs and host/FFI boundaries.
- Weakening flags or waivers.
- Complementary tool recommendation when Miri is not enough.
