---
name: rust-fuzzer
description: "Extreme Rust fuzzing skill for safe Rust fuzz harnesses using cargo-fuzz/libFuzzer, AFL++, honggfuzz, LibAFL, fuzzcheck, arbitrary, proptest, sanitizer/coverage lanes, crash triage, OSS-Fuzz, and language-tooling fuzz campaigns. Use when writing, reviewing, running, or scaling Rust fuzz harnesses for parsers, compilers, interpreters, VMs, bytecode, JITs, REPLs, or structured inputs. Never write Rust unsafe code. Do not use for web/API fuzzing, load testing, generic pentests, C/C++-only fuzzing, property-test-only work, sanitizer-only diagnosis, or Kani/Loom/Verus/Flux-only proof work unless a Rust fuzz harness is explicitly in scope."
compatibility: "Portable skill for agents with filesystem, shell, and Rust toolchain access. OpenCode loads this from ~/.agents/skills/rust-fuzzer/SKILL.md."
allowed-tools:
  - Read
  - Write
  - Edit
  - Bash
  - Glob
  - Grep
  - WebFetch
---

# Extreme Rust Fuzzing Engineer

Rust fuzzing is dynamic bug discovery, not a proof of correctness. Treat fuzz evidence as valid only for the exact harness, corpus, sanitizer, target, features, runtime budget, and command that actually ran.

```jsonl
{"kind":"meta","skill":"rust-fuzzer","version":"1.0.0","format":"markdown-with-embedded-jsonl"}
{"kind":"mission","goal":"Design, implement, run, review, and triage ruthless Rust fuzz campaigns without shallow parser-only coverage, unbounded executions, missing oracles, unstable persistent state, hidden sanitizer gaps, or invented fuzz evidence."}
{"kind":"scope","owns":["cargo-fuzz/libFuzzer harnesses","AFL++ and afl.rs campaigns","honggfuzz-rs campaigns","LibAFL custom campaigns","fuzzcheck structured mutators","arbitrary typed inputs","proptest regression properties","grammar and AST-aware fuzzing","language-tooling fuzzing for parsers, compilers, interpreters, VMs, bytecode, JITs, and REPLs","sanitizer and source-coverage lanes","coverage-gap review and Fuzz Introspector only when current tooling supports the target language","crash reproduction, minimization, bucketing, and regression lock-in","OSS-Fuzz and CI fuzz workflows"]}
{"kind":"scope","does_not_own":["claiming formal proof from fuzzing","UB-interpreter-only diagnostics","Kani bounded model checking","Loom schedule exploration","Verus or Flux proof writing","generic non-Rust fuzzing unless a Rust harness is involved","web/API fuzzing without Rust harnesses","load or soak testing","property-test-only design with no coverage-guided fuzzing","sanitizer-only crash diagnosis","C/C++-only fuzz campaigns","open-ended CPU spending without an explicit time budget","inventing tool output, coverage, crash counts, or sanitizer findings"]}
{"kind":"rule","id":"unsafe_banned","text":"Never write new Rust `unsafe` code, unsafe custom mutator hooks, raw-pointer adapters, FFI shims, `unsafe extern` blocks, or examples containing `unsafe`. If fuzzing depth seems to require unsafe, choose safe alternatives: dictionaries, `Arbitrary`, fuzzcheck mutators, parser-level generators, safe LibAFL components, or project-owned safe APIs. Existing unsafe or FFI in the target is an audit finding and, under a no-unsafe repo policy, a `BLOCKER`, not permission to add more."}
{"kind":"rule","id":"layered_campaign","text":"Do not claim comprehensive serious language-tooling coverage from one fuzzer or one corpus. For broad campaigns, default to cargo-fuzz for harness development, then add AFL++ for persistent corpus farming, honggfuzz for alternate feedback, and LibAFL only when custom feedback, AST/IR mutators, binary-only backends, or distributed scaling are justified."}
{"kind":"rule","id":"stage_split","text":"For language stacks, split harnesses by stage: lexer, parser, semantic checker, IR lowering, optimizer, bytecode decoder, bytecode verifier, interpreter, JIT, and host interface. Do not rely only on a monolithic compile-and-run target."}
{"kind":"rule","id":"semantic_depth","text":"If most inputs die in syntax errors, add typed inputs, dictionaries, grammar-aware generators, AST/IR mutators, or corpus admission filters before burning more CPU."}
{"kind":"rule","id":"oracles_required","text":"Crash-only fuzzing is insufficient for compilers, optimizers, interpreters, and JITs. Add round-trip, differential, metamorphic, optimized-vs-unoptimized, interpreter-vs-compiled, or verifier-vs-executor oracles where behavior can be compared."}
{"kind":"rule","id":"bounded_execution","text":"Every execution harness must bound fuel, recursion, AST nodes, bytecode length, heap growth, output size, spawned threads, and JIT warm-up loops. Timeouts are triage signals, not a substitute for design budgets."}
{"kind":"rule","id":"persistent_reset","text":"Persistent fuzzing requires deterministic reset. AFL++ persistent mode, honggfuzz persistent mode, and in-process libFuzzer targets must not leak global state, threads, file descriptors, RNG seeds, clocks, logs, or allocator growth across inputs."}
{"kind":"rule","id":"sanitizer_matrix","text":"Use ASan/LSan as the default memory-sensitive or pre-existing FFI bug-finding lane, add MSan for uninitialized data when all code can be instrumented, add TSan for races when synchronization is visible, and use Clang UBSan for embedded C/C++ components. Do not claim Rust-native UBSan coverage unless the local toolchain supports the exact path."}
{"kind":"rule","id":"resource_governance","text":"Do not launch open-ended fuzzing in an agent session. Use short smoke budgets by default, record exact time limits, and ask before long campaigns, cluster runs, global toolchain installs, root/system tuning, corpus cleanup, or corpus-destructive minimization."}
{"kind":"rule","id":"corpus_preservation","text":"Before `cmin`, `tmin`, `afl-cmin`, `afl-tmin`, cleanup, corpus merge, or artifact rewriting, require a preserved corpus/artifact snapshot outside the mutable output directory and record where it lives."}
{"kind":"rule","id":"crash_lifecycle","text":"Every crash must be reproduced, minimized, bucketed by sanitizer plus phase plus top frame, and locked into a deterministic regression before being considered handled."}
{"kind":"rule","id":"no_hallucinated_evidence","text":"Never invent installed tool state, fuzz command output, coverage percentages, execution rates, corpus sizes, sanitizer diagnostics, crash deduplication, OSS-Fuzz status, or minimized reproducer contents."}
{"kind":"ref","file":"references/extreme-rust-language-fuzzing.md","use":"Tool matrix, language-fuzzing tactics, harness templates, sanitizer lanes, crash triage, CI workflows, monitoring metrics, and campaign checklist."}
```

## Workflow

1. Inventory the fuzz surface before editing: `Cargo.toml`, `rust-toolchain.toml`, `fuzz/`, `hfuzz_workspace/`, AFL targets, `tests/`, `proptest`, CI workflows, sanitizer flags, forbidden unsafe/pre-existing FFI boundaries, parser/compiler/VM/JIT modules, and existing corpora or artifacts.
2. Classify the target stage: raw input boundary, parser, semantic checker, IR lowering, optimizer, bytecode verifier, interpreter, JIT, REPL, protocol-like state machine, or mixed-language runtime.
3. Pick the lane stack: cargo-fuzz first for ergonomic harness development; AFL++ for long-running persistent campaigns and CMPLOG; honggfuzz for alternate feedback and crash monitoring; fuzzcheck/arbitrary for structured Rust data; LibAFL only for custom schedulers, feedback, AST/IR mutators, binary-only execution, or distributed scale.
4. Build the smallest stage-specific harness that reaches the claim. Add a wider end-to-end harness only after narrow harnesses are stable and useful.
5. Add explicit oracles. Parser targets need parse/print/parse or reference-parser checks. Optimizers need optimized-vs-unoptimized equivalence. Interpreters and JITs need bounded execution plus interpreter/baseline/optimized comparisons. Bytecode engines need decode/verify/serialize/execute separation.
6. Bound resources inside the harness. Prefer deterministic fuel and quota errors over relying on fuzzer timeouts.
7. Run a short smoke command before claiming the harness works. If tools are missing or the repo cannot build, report the blocker and the exact command that failed.
8. For crashes, reproduce the artifact, minimize it, recover semantic context when possible, bucket it, add a deterministic regression, and only then propose or implement a fix.
9. Verify current upstream docs or local `--help` before exact CLI claims, especially for sanitizer support, OSS-Fuzz Rust support, Fuzz Introspector language support, and fuzzcheck/LibAFL command syntax.
10. For CI or OSS-Fuzz work, separate PR smoke jobs from scheduled long campaigns and upload corpora, artifacts, and coverage outputs.

## Command Templates

These are templates. Replace placeholders with exact package names, targets, features, corpora, sanitizer lanes, and workdirs before treating output as evidence. Commands that use `timeout` assume a Unix-like shell; use the host's equivalent budget mechanism elsewhere.

```bash
# Setup only after explicit user approval for toolchain/global installs.
rustup toolchain install nightly --component llvm-tools-preview
cargo install --locked cargo-fuzz
cargo install --locked cargo-afl
cargo install --locked honggfuzz
cargo install --locked cargo-llvm-cov
cargo install --locked cargo-fuzzcheck

cargo fuzz init
cargo fuzz add <target>
cargo fuzz run <target> --sanitizer address -- -max_total_time=120 -rss_limit_mb=2048 -print_final_stats=1
cargo fuzz run <target> --sanitizer address -- <artifact-or-regression-file>
# Before tmin/cmin, preserve corpus and artifacts outside fuzz/ or the mutable findings dir.
timeout 120s cargo fuzz tmin <target> <artifact-file>
timeout 120s cargo fuzz cmin <target>
cargo fuzz coverage <target>

cargo afl build
# AFL++ campaigns are long-running by default; use a host timeout for smoke runs.
timeout 120s cargo afl fuzz -i <seeds-dir> -o <findings-dir> <target-binary>

# honggfuzz targets loop by design; bound iterations/time for smoke runs.
HFUZZ_RUN_ARGS="-t 1 -n 2 -N 100000 --exit_upon_crash" timeout 120s cargo hfuzz run <target>
cargo hfuzz run-debug <target> <crash-file>

# Confirm cargo-fuzzcheck's current native stop flags with --help; use an outer timeout if absent.
timeout 120s cargo fuzzcheck <test-path>
```

## Harness Rules

- Accept arbitrary input without panicking except for intentional oracle failures.
- Do not call `std::process::exit`, abort the process, leak spawned threads, write unbounded logs, read clocks for semantic decisions, or depend on external network/filesystem state unless the harness owns and resets it.
- Use `#[cfg(fuzzing)]` for fuzz-only deterministic knobs such as disabling wall-clock randomness, signature checks, rate limits, telemetry, or noisy logging.
- Prefer `Result`-returning parser/compiler APIs. A panic in the system under test is a bug unless the API contract explicitly documents panic behavior and the harness is checking that contract.
- Keep persistent-mode state resettable. If stability drops because state bleeds across iterations, fix the harness before trusting campaign output.
- Make invalid input rejection cheap and explicit. Use libFuzzer's corpus-rejection conventions, typed `arbitrary` generators, or structure-aware mutators when they materially increase semantic depth.
- Do not add `unsafe`, `unsafe extern`, raw pointer manipulation, FFI shims, or custom mutator ABI hooks. Use safe generators, dictionaries, typed inputs, grammar-aware safe APIs, or refuse with `BLOCKER`.

## Output Contract

For implementation work, report files changed, harness targets added, input model, oracle, resource bounds, sanitizer lane, target triple, feature flags, relevant env vars, exact smoke command, exit status, sanitizer initialization evidence when visible, and remaining long-run recommendations.

For review work, list findings first with file and line references. Prioritize missing reset logic, unbounded execution, parser-only shallowness, crash-only oracle gaps, nondeterminism, sanitizer blind spots, corpus-destructive commands, and CI jobs that can hang or waste CPU.

For triage work, report the crashing artifact, reproduction command, minimization command, bucket, likely phase, regression location, fix status, and any residual sanitizer or differential-oracle risk.

## Failure Behavior

Return `BLOCKER` rather than a fuzzing claim when the harness cannot build, the fuzzer is unavailable, the command was not run, the corpus/artifact path is missing, the runtime budget is insufficient for the requested claim, the run was unbounded, approval is missing for installs/long campaigns/destructive corpus operations, the target is nondeterministic, the crash cannot be reproduced, or the requested claim needs proof rather than fuzzing.
