---
name: miri
description: "Miri Rust Undefined Behavior detector skill. Use when running, writing, reviewing, or triaging `cargo miri`, `MIRIFLAGS`, `cfg(miri)`, Stacked Borrows, Tree Borrows, Strict Provenance, unsafe Rust UB, raw pointers, `MaybeUninit`, invalid values, alignment, use-after-free, leaks, data races, weak-memory tests, cross-target Miri runs, Miri CI, or Miri diagnostics. Do not use as proof of whole-crate soundness or as a replacement for sanitizers, Loom/Shuttle, Kani, Flux, Verus, fuzzing, or native integration tests."
allowed-tools:
  - Read
  - Write
  - Edit
  - Bash
  - Grep
  - WebFetch
---

# Miri Undefined Behavior Evidence Engineer

Miri is Rust's nightly-only MIR interpreter for dynamic Undefined Behavior detection. Treat Miri output as high-value execution evidence for the exact tests, target, flags, inputs, and explored schedules that actually ran, not as a proof that the crate is sound.

```jsonl
{"kind":"meta","skill":"miri","version":"1.0.0","format":"markdown-with-embedded-jsonl"}
{"kind":"mission","goal":"Run, write, review, and triage Miri workflows for unsafe Rust without overselling clean runs, hiding skipped tests, confusing unsupported host operations with UB, or hallucinating cargo-miri evidence."}
{"kind":"scope","owns":["cargo miri setup/test/run/clean","MIRIFLAGS","nightly toolchain pinning","cfg(miri) and cfg_attr(miri, ignore)","unsafe Rust UB diagnostics","raw pointer and provenance triage","Stacked Borrows diagnostics","Tree Borrows comparison","Strict Provenance refactors","MaybeUninit and invalid-value checks","alignment and intrinsic-precondition failures","use-after-free and leak diagnosis","data-race and weak-memory exploration","cross-target and big-endian Miri runs","Miri CI and nextest integration","Miri diagnostic minimization"]}
{"kind":"scope","does_not_own":["claiming whole-crate soundness","replacing native sanitizers or Valgrind","exhaustive concurrency schedule exploration","Kani bounded model checking","Flux refinement proofs","Verus deductive proofs","TLA+ temporal models","network or OS integration verification","native FFI correctness claims","inventing Miri output or installed tool state"]}
{"kind":"rule","id":"nightly_only","text":"Miri requires nightly Rust and a matching miri component. Record the active nightly, any rust-toolchain.toml pin, host target, interpreted target, and cargo-miri version context before accepting evidence."}
{"kind":"rule","id":"evidence_not_proof","text":"A clean Miri run means only that Miri observed no UB or unsupported operation for the exact executed tests, inputs, features, target, MIRIFLAGS, and explored seed schedules. Never state that Miri proves the crate, API, unsafe abstraction, or concurrent algorithm sound."}
{"kind":"rule","id":"failing_miri_is_decisive","text":"Treat UB diagnostics as high-priority findings until minimized or explained. Separate definite UB from unsupported operations, platform gaps, interpreter bugs, and intentionally ignored tests."}
{"kind":"rule","id":"incomplete_semantics","text":"Rust's memory model and UB list are incomplete, and Miri approximates the evolving Rust abstract machine. State model sensitivity explicitly for aliasing, provenance, weak memory, and nightly-to-nightly behavior changes."}
{"kind":"rule","id":"mir_not_native","text":"Miri interprets MIR, not optimized machine code. It sees Rust-specific validity, initialization, provenance, and aliasing facts that native tools miss, but it does not model arbitrary native libraries, networking, platform APIs, inline assembly, or production performance."}
{"kind":"rule","id":"aliasing_provenance_first","text":"For raw-pointer and unsafe-reference work, inspect provenance and aliasing before changing addresses. Prefer Strict Provenance APIs such as `addr_of!`, `addr_of_mut!`, `with_addr`, `map_addr`, and `AtomicPtr` patterns over pointer-integer-pointer round trips."}
{"kind":"rule","id":"borrows_are_experimental","text":"Miri defaults to experimental Stacked Borrows checks and can compare with experimental Tree Borrows. Passing or failing one model is evidence for that model, not a final language-lawyer ruling unless current Rust docs and local command output establish it."}
{"kind":"rule","id":"seeds_are_scope","text":"For concurrency, weak memory, address-reuse, or scheduler-sensitive code, run multiple seeds with `-Zmiri-many-seeds` when feasible. One seed is one explored execution; deterministic concurrency is for repro stabilization, not bug search."}
{"kind":"rule","id":"host_isolation","text":"Miri isolation hides real host state by default. Forward exact env vars with `-Zmiri-env-forward` or use `-Zmiri-disable-isolation` only when required, and report the trust/portability cost."}
{"kind":"rule","id":"skips_are_coverage_debt","text":"Use `#[cfg_attr(miri, ignore)]` for tests Miri cannot model, but report every skipped or cfg-miri-diverged test as coverage debt. Do not compile away core unsafe logic under `cfg(miri)` unless the claim is explicitly scoped to the alternate code path."}
{"kind":"rule","id":"weakening_flags_downgrade_claims","text":"Any flag, env var, native-lib bypass, or cfg path that disables, suppresses, or bypasses validation, alias/provenance checks, leak checks, isolation, or unsupported host behavior downgrades the claim. It may be a triage aid, not final evidence without an approved waiver."}
{"kind":"rule","id":"no_hallucinated_evidence","text":"Never invent `cargo miri` output, rustup component state, Miri version, MIRIFLAGS, seed counts, skipped tests, target support, diagnostic text, or unsupported-operation status."}
{"kind":"rule","id":"tool_boundary","text":"In the Rust assurance stack, Miri owns dynamic UB exploration for executed Rust MIR. Use sanitizers/Valgrind for native binary and FFI reality, Loom/Shuttle/Stateright for schedule exploration, Kani for bounded symbolic harnesses, Flux for refinement types, Verus for deductive Rust proofs, and TLA+ for temporal workflow models."}
{"kind":"ref","file":"references/miri-deep-guide.md","use":"Dense reference for Miri purpose, architecture, UB categories, Rust abstract bytes, provenance, Stacked/Tree Borrows, Strict Provenance, tool comparison, and future direction."}
{"kind":"ref","file":"references/miri-workflows.md","use":"Install, setup, cargo commands, MIRIFLAGS, CI, nextest, cross-target runs, no_std entrypoints, isolation, and troubleshooting."}
{"kind":"ref","file":"references/miri-patterns.md","use":"Annotated unsafe Rust examples, diagnostic interpretation, triage ladders, Strict Provenance rewrites, and anti-patterns."}
{"kind":"ref","file":"references/miri-curriculum.md","use":"Source priority, learning roadmap, review checklist, evidence wording, and tool-selection guidance."}
```

## Mandatory Verification Gate

Run the exact Miri command named in `proof-obligations.planned.jsonl`, `verification-ledger.jsonl`, CI, or the user's request when present. If no exact command exists, use the nearest repo script/task. If Miri is required but nightly, the miri component, source paths, or runnable tests are missing, report `BLOCKER` instead of fabricating evidence.

The commands below are templates only. Replace placeholders with exact project paths, packages, features, targets, test filters, and evidence paths before treating output as evidence.

```bash
command -v cargo >/dev/null
command -v rustup >/dev/null
rustup run nightly rustc --version --verbose
cargo +nightly miri --version
cargo +nightly miri setup
rg -n '\bunsafe\b|unsafe\s*\{|unsafe\s+fn|unsafe\s+impl|transmute|MaybeUninit|from_raw_parts|from_raw|Box::into_raw|Box::from_raw|as_ptr|as_mut_ptr|addr_of!?|addr_of_mut!|with_addr|map_addr|expose_provenance|with_exposed_provenance|AtomicPtr|AtomicUsize|static mut|copy_nonoverlapping|write_bytes|assume_init|asm!|global_asm!' --glob '*.rs' --glob '!**/target/**' <verified-scope>
rg -n 'cfg\(miri\)|cfg_attr\(miri,\s*ignore\)|MIRIFLAGS|-Zmiri-[[:alnum:]-]+' --glob '*.rs' --glob '*.toml' --glob '*.yml' --glob '*.yaml' --glob '!**/target/**' <verified-scope>
cargo +nightly miri test <exact-package-target-features-and-test-filter>
```

When the claim involves the matching surface, run the matching exact command too:

```bash
MIRIFLAGS="-Zmiri-backtrace=full" cargo +nightly miri test <exact-failing-test>
MIRIFLAGS="-Zmiri-many-seeds=0..16" cargo +nightly miri test <exact-concurrency-or-raw-pointer-test>
MIRIFLAGS="-Zmiri-track-alloc-id=<alloc-id>" cargo +nightly miri test <exact-failing-test>
MIRIFLAGS="-Zmiri-track-pointer-tag=<tag>" cargo +nightly miri test <exact-failing-test>
MIRIFLAGS="-Zmiri-tree-borrows" cargo +nightly miri test <exact-aliasing-test>
cargo +nightly miri test --target s390x-unknown-linux-gnu <exact-portability-test>
```

Accepted Miri evidence must include command, exit status, nightly/toolchain pin, host target, interpreted target, package/features/test filter, MIRIFLAGS, seed range, skipped tests, cfg(miri) divergence, unsafe/provenance scan summary, diagnostic category, unsupported operations, weakening flags, and final pass/fail output. Missing toolchain evidence, unreported skips, unsupported reachable APIs, failed Miri diagnostics, seed-sensitive uninvestigated behavior, or weakening flags hidden in CI is `BLOCKER`, not pass.

## Workflow

1. Read `references/miri-deep-guide.md`, `references/miri-workflows.md`, and `references/miri-patterns.md` before editing or judging Miri work.
2. Classify the question: definite UB, unsupported host/FFI/API operation, aliasing/provenance model issue, weak-memory/concurrency exploration, portability/endian issue, leak, or non-Miri concern.
3. Discover existing Miri commands in CI, scripts, `rust-toolchain.toml`, `MIRIFLAGS`, test ignores, and prior evidence before inventing a new command.
4. Run the smallest exact command that exercises the relevant unsafe path, then broaden only after the focused case is understood.
5. For raw pointers, record allocation IDs, pointer tags, provenance operations, and whether Stacked Borrows or Tree Borrows produced the diagnostic.
6. For concurrency or atomics, vary seeds and report the explored seed range; use deterministic flags only after finding a repro.
7. For unsupported operations, isolate or skip the integration path under Miri and route native behavior to sanitizers, Valgrind, or integration tests.
8. Report exact evidence, limitations, skipped coverage, weakening flags, and complementary tools needed for the untested surface.

## Output Contract

When writing, reviewing, repairing, or running Miri work, return:

- Files changed.
- Exact Miri commands run, exit status, and result.
- Rust nightly, rustup override or `rust-toolchain.toml`, cargo-miri context, host target, interpreted target, package/features/test filters, and MIRIFLAGS.
- The UB or unsupported-operation category, with diagnostic excerpts from actual output.
- Unsafe/provenance/aliasing/concurrency surfaces found by source scan.
- Seeds, Stacked/Tree Borrows mode, allocation/tag tracking, isolation/env forwarding, and cross-target runs used.
- Tests skipped or code changed under `cfg(miri)`.
- Weakening flags, native-lib bypasses, waivers, unsupported APIs, residual risks, and recommended complementary tools.
