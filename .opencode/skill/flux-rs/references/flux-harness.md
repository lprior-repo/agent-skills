# Flux RS Harness

## Evidence Standard

Valid Flux evidence must include:

- Exact command.
- Working directory.
- Flux invocation mode: `cargo flux`, single-file `flux`, or repo script.
- Toolchain or installed Flux discovery output when available.
- Crate metadata and include/ignore scope.
- Solver/config flags such as Z3, cvc5, cache, or `FLUXFLAGS`.
- `liquid-fixpoint` or `fixpoint` availability.
- Exit status and real diagnostics.
- Trusted-boundary scan result.
- Negative or `#[should_fail]` evidence when claiming invalid states are rejected.
- Residual blockers or waivers.

Do not report verifier success from a code review, Rust compiler pass, test pass, or guessed output.

## Tool Discovery

Use CLI evidence, not editor integration.

```bash
command -v cargo >/dev/null
rustup show active-toolchain
command -v z3 >/dev/null
z3 --version
if command -v fixpoint >/dev/null; then
  fixpoint --version
else
  command -v liquid-fixpoint >/dev/null
  liquid-fixpoint --version
fi
cargo flux --help >/dev/null
if command -v flux >/dev/null; then flux --help >/dev/null; fi
```

If `cargo flux`, required solver tools, Liquid Fixpoint, or required Rust toolchain details are unavailable and Flux is required, report `BLOCKER`. If only single-file `flux` is available, use it only when the obligation explicitly targets a single file or the repo accepts that mode. If the repo selects cvc5, capture `cvc5 --version` too.

## Installation Notes

Official install path, subject to current upstream docs:

```bash
git clone https://github.com/flux-rs/flux
cd flux
cargo xtask install
```

Flux requires Rustup, the upstream-pinned nightly toolchain, solver dependencies such as Liquid Fixpoint, and Z3 4.15 or later. `cargo xtask install` installs `flux` and `cargo-flux` into the Cargo home and copies support files under `$HOME/.flux`. Nightly binary builds may exist on GitHub Releases; otherwise build from source. Verify against current upstream docs before changing installation steps.

## Crate Setup

Typical crate metadata:

```toml
[package.metadata.flux]
enabled = true

[dependencies]
flux-rs = { git = "https://github.com/flux-rs/flux.git" }
```

Import attributes in Rust code:

```rust
use flux_rs::attrs::*;
```

Single-file mode may use tool-style `flux::` prefixes in examples. Do not mix crate-mode and single-file conventions without checking the local docs and diagnostics.

VSCode integration is useful for feedback but is not evidence. A typical override is:

```json
{
  "rust-analyzer.check.overrideCommand": [
    "cargo", "flux", "--workspace",
    "--message-format=json-diagnostic-rendered-ansi"
  ]
}
```

## Command Selection

Preferred order:

1. Exact command from `proof-obligations.planned.jsonl` or `verification-ledger.jsonl`.
2. Repo script or task that runs Flux.
3. `cargo flux` in the target crate.
4. Single-file `flux --crate-type=lib path/to/file.rs` only when scoped and accepted.

Template commands are not evidence:

```bash
cargo flux <exact-crate-package-or-target>
flux --crate-type=lib <exact-file.rs>
FLUXFLAGS="-Ftimings" cargo flux <exact-target>
```

Replace every placeholder before running.

## Debug And Scale Flags

Useful flags and environment patterns seen in official material and source workflows:

```bash
FLUXFLAGS="-Ftimings" cargo flux <exact-target>
FLUX_DUMP_TIMINGS=true cargo flux <exact-target>
FLUX_CATCH_BUGS=1 cargo flux <exact-target>
flux --crate-type=lib -Fsolver=cvc5 <exact-file.rs>
flux --crate-type=lib -Fscrape-quals <exact-file.rs>
flux --crate-type=lib -Fdump-constraint <exact-file.rs>
flux --crate-type=lib -Fdump-checker-trace <exact-file.rs>
flux --crate-type=lib -Fdump-fhir <exact-file.rs>
flux --crate-type=lib -Fdump-rty <exact-file.rs>
flux --crate-type=lib -Fannots <exact-file.rs>
```

Use dumps only to diagnose. Keep large solver artifacts as raw evidence files and summarize only the decisive diagnostic in final reports.

When filing Flux bugs or tracing confusing diagnostics, use the local upstream-recommended `-Ztrack-diagnostics=y` path if available. Some Flux developer workflows enable it through `cargo xtask run attic/playground.rs`.

## Developer Test Commands

Inside the Flux repository, common workflows are:

```bash
cargo xtask test
cargo xtask test impl_trait
cargo xtask run path/to/test.rs
cargo x expand path/to/file.rs
```

Regression tests commonly live under `tests/pos/` for expected verification success and `tests/neg/` for expected failures. These are Flux project development commands, not automatically valid for downstream crates.

## Configuration Knobs

Flux supports incremental adoption through metadata/config such as enablement, cache, solver choice, include patterns, ignored scope, and version-specific trusted implementation settings. Exact keys may change; verify against the local Flux docs/source before editing config.

Example metadata from the practical guide:

```toml
[package.metadata.flux]
enabled = true
cache = true
solver = "cvc5"
```

Per-invocation flags may be supplied with `FLUXFLAGS`, for example `FLUXFLAGS="-Ftimings" cargo flux`. Flux flags use the `-Fname=value` or `-Fname` form. Run the local help/docs before relying on any flag in a report.

Treat broad trusted implementation settings, broad include exclusions, or crate-level ignore as verification limitations.

## Trusted Boundary Scan

Run a scan over the verified scope:

```bash
rg -n '#!?\[(flux_rs::|flux::)?(trusted|trusted_impl|extern_spec|ignore|no_panic|no_panic_if)(\([^]]*\))?\]|unsafe' --glob '*.rs' --glob '!**/target/**' <verified-scope>
```

Report each trusted or ignored item that affects the property being claimed. Trusted code that breaks global readiness is a global blocker; touched or scoped trusted code is local evidence debt.

## Failure Triage

1. Confirm Flux is enabled for the crate or file.
2. Reduce to the smallest failing function, type, or module.
3. Check whether the property belongs in a type invariant, function contract, enum variant, or `ensures` post-state.
4. Check whether local std specs model the relevant `Option`/`Vec` behavior.
5. Add a loop invariant, qualifier, or spec function only after the boundary contract is right.
6. Inspect constraint/checker dumps only for the failing item.
7. Avoid widening `#[trusted]`; if trust is necessary, make it a tiny wrapper and report it.

## Acceptance Gate

Accept only when the exact command exits successfully, diagnostics are reviewed, trusted/ignored scope is reported, and the property checked by Flux matches the contract claim.

When claiming illegal states are unrepresentable, also require an exact negative or `#[should_fail]` target that proves the bad constructor/call is rejected. If no negative target exists, the illegal-state claim is `BLOCKER`, not soft residual debt.

Reject or block when Flux is unavailable, the target is unknown, broad ignore/trust hides the property, the command checks a different crate/file, or the output is inferred rather than observed.
