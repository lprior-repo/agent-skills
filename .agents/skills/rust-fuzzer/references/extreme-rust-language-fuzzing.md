# Extreme Rust Language Fuzzing Reference

Use this reference when planning or executing Rust fuzz campaigns for language tooling. Keep the active answer concise, but pull details from here when choosing engines, harness patterns, or evidence requirements.

## Research Snapshot

Verified against upstream docs on 2026-05-24. Re-check links and local `--help` when exact flags or platform support matter.

- The Rust Fuzz Book calls `cargo-fuzz` the recommended tool for fuzz testing Rust code and states that cargo-fuzz currently invokes libFuzzer.
- The Rust Fuzz Book setup page requires nightly for sanitizer-backed cargo-fuzz flows and documents support on x86_64 Linux, x86_64 macOS, Apple Silicon macOS, and Windows through MSVC AddressSanitizer.
- The Rust Fuzz Book guide documents `#[cfg(fuzzing)]` for the fuzz target crate, project crate, and dependency tree; use it for deterministic fuzz-only adaptations.
- The Rust Fuzz Book CI page recommends short CI smoke fuzzing, pins `cargo-fuzz` through `CARGO_FUZZ_VERSION`, installs with `--locked`, and uses `-max_total_time`.
- LLVM libFuzzer docs say fuzz targets run many times in the same process, must tolerate arbitrary inputs, must not call `exit`, should join threads, should be deterministic and fast, and should avoid global state. libFuzzer runs indefinitely unless bounded with flags such as `-max_total_time` or `-runs`.
- LLVM libFuzzer docs also document corpus seeding, `-merge=1`, dictionaries, trace-cmp/value profile, custom mutators, and returning `-1` to reject unwanted corpus inputs. This skill may use dictionaries and safe corpus shaping, but must not implement Rust custom mutator ABI hooks.
- afl.rs now fronts AFL++ for Rust, enables CMPLOG by default, sets `cfg(fuzzing)` by default, and provides `fuzz_with_reset!` for persistent-mode static-state reset.
- honggfuzz-rs documents Linux/macOS/BSD/Android/WSL support, stable/beta/nightly Rust compatibility, `cargo hfuzz run`, `HFUZZ_RUN_ARGS`, sanitizer use through `RUSTFLAGS`, and `cfg(fuzzing)` deterministic adaptations.
- fuzzcheck documents itself as evolutionary and structure-aware, Linux/macOS plus nightly-only, and explicitly says it continues indefinitely; bound it in agent sessions.
- Rust's unstable sanitizer docs list supported Rust sanitizer flags and target caveats. MSan requires all program code to be instrumented; TSan needs visible synchronization and does not support atomic fences.
- OSS-Fuzz Rust integration currently expects `cargo fuzz` and states Rust `project.yaml` supports `libfuzzer` and `address` for that Rust path. Do not claim OSS-Fuzz Rust AFL++/honggfuzz lanes unless current docs or project config prove it.
- ClusterFuzzLite documents Rust support, libFuzzer, ASan/MSan/UBSan, PR fuzzing, batch fuzzing, crashing testcases, corpus use, and coverage reports.
- OSS-Fuzz Fuzz Introspector docs currently describe supported report generation for C/C++, Python, and Java. For Rust, prefer `cargo fuzz coverage`/source coverage unless current Fuzz Introspector docs prove Rust support for the project.

## No Unsafe Code Policy

This skill must never cause an agent to write new Rust `unsafe` code. That ban includes `unsafe` blocks, `unsafe fn`, `unsafe extern`, raw pointer manipulation, FFI adapter shims, `extern "C"` custom mutator hooks, transmute-based helpers, and snippets that teach unsafe patterns.

Allowed work under this policy:

- Write safe Rust fuzz targets that call safe public APIs.
- Use `arbitrary`, fuzzcheck mutators, dictionaries, parser-level generators, proptest regressions, and safe LibAFL components.
- Detect pre-existing unsafe or FFI surfaces and report them as audit findings. Under a no-unsafe repo policy, report them as `BLOCKER` unless they are explicitly out of scope and untouched.
- Run sanitizer-backed fuzz lanes against existing code when the project already exposes a safe harness boundary.

Disallowed work under this policy:

- Adding unsafe code to reach a private parser, allocator, VM, JIT, or C ABI.
- Writing `LLVMFuzzerCustomMutator` in Rust, because the ABI requires raw pointer handling.
- Adding FFI shims just to fuzz deeper internals.
- Copying unsafe examples from upstream docs into project code.

If the only apparent path to a fuzz target requires new unsafe code, return `BLOCKER` and propose a safe API boundary, dictionary, typed input model, grammar generator, or refactor that exposes the target safely.

## Cold Recommendation

Default to `cargo-fuzz` for first harnesses and local iteration. Graduate stable high-value harnesses to AFL++ for persistent long-haul campaigns, honggfuzz for alternate feedback and crash monitoring, and LibAFL only when off-the-shelf engines cannot express the needed scheduler, feedback, mutator, backend, or distributed campaign. Do not describe that as proof or as comprehensive coverage unless the evidence actually supports it.

For Rust language tooling, the high-yield plan is layered:

- Byte/coverage-guided fuzzing for lexers, parsers, decoders, and file boundaries.
- Typed or grammar-aware fuzzing for parsers, semantic checkers, and AST consumers.
- IR/program/sequence-aware fuzzing for optimizers, interpreters, VMs, JIT tiers, and REPLs.
- Semantic oracles layered on top: round-trip, differential, metamorphic, optimized-vs-unoptimized, interpreter-vs-compiled, verifier-vs-executor.
- Sanitizer, coverage, and regression lanes that convert found failures into durable tests.

## Tool Matrix

| Tool | Best Use | Strength | Weak Point |
| --- | --- | --- | --- |
| `cargo-fuzz` + libFuzzer | First Rust harness, parser/front-end libraries, local triage | Ergonomic setup, corpus/artifact layout, `Arbitrary`, `fmt`, `tmin`, `cmin`, coverage | In-process targets expose reset leaks; libFuzzer is maintained rather than feature-growing |
| AFL++ via `afl.rs` | Long-running persistent campaigns, compare-heavy grammars, corpus farming | Strong mutators and power schedules, CMPLOG, persistent mode, monitoring | More operational complexity; target must be truly resettable |
| honggfuzz-rs | Alternate feedback lane, Linux hardware feedback, crash-heavy targets | Multi-process/threaded, persistent mode, replay/debug support | Best features are Linux-centric; smaller Rust mindshare |
| LibAFL | Custom AST/IR mutators, custom feedback, binary-only backends, distributed scaling | Modular, scalable, supports many executors/backends | Higher engineering cost; evolving API surface |
| fuzzcheck | Rust-native structured values and recursive grammars | Evolutionary mutators, grammar mutators, corpus and stats | Smaller ecosystem, nightly-oriented setup |
| `arbitrary` | Bridge raw fuzzer bytes into structured Rust inputs | Fast typed inputs for libFuzzer | Shape-valid does not mean semantically valid |
| proptest | Shrinking-friendly invariants and deterministic regressions | Excellent oracle and regression tool | Not coverage-guided; not a native fuzzer replacement |

Linux x86_64 remains the least surprising baseline for production fuzzing because it aligns best with Rust sanitizers, AFL++, honggfuzz hardware feedback, and OSS-Fuzz infrastructure.

## Practical Toolchain

Use project pins when present. Ask before global installs or toolchain changes. Otherwise, after approval, this is a practical starting point:

```bash
rustup toolchain install nightly --component llvm-tools-preview
cargo install --locked cargo-fuzz
cargo install --locked cargo-afl
cargo install --locked honggfuzz
cargo install --locked cargo-llvm-cov
cargo install --locked cargo-fuzzcheck
```

If `--locked` is unsupported or fails because the crate does not publish usable lock metadata, record the failure and ask before retrying without it. Do not run `cargo afl system-config` or any root/system tuning without explicit user approval.

## Minimal Entry Commands

```bash
# cargo-fuzz
cargo fuzz init
cargo fuzz add parser
cargo fuzz run parser --sanitizer address -- -max_len=4096 -max_total_time=120 -rss_limit_mb=2048 -print_final_stats=1

# AFL++ / afl.rs
cargo afl build
timeout 120s cargo afl fuzz -i in -o out target/debug/parser_target

# honggfuzz
HFUZZ_RUN_ARGS="-t 1 -n 2 -N 100000 --exit_upon_crash" timeout 120s cargo hfuzz run parser

# fuzzcheck
timeout 120s cargo fuzzcheck tests::my_fuzz_test
```

The `timeout` wrapper is a smoke-run guard, not campaign engineering. For a real campaign, create a named run plan with duration, machine/resource owner, corpus backup policy, sanitizer lane, monitoring metrics, and stop conditions.

## Mutation Strategies

| Strategy | Best Targets | Rust Route | Unlocks | Downside |
| --- | --- | --- | --- | --- |
| Coverage-guided byte mutation | Lexers, decoders, front ends, bytecode readers | cargo-fuzz, AFL++, honggfuzz | Fast baseline exploration | Wastes cycles on invalid syntax |
| Compare and dictionary guidance | Keywords, magic constants, tokenized grammars | libFuzzer `-dict`, trace-cmp; AFL++ CMPLOG | Cracks string/number gates | Still grammar-blind alone |
| Typed mutation | AST-like enums/structs, recursive IR | `arbitrary`, fuzzcheck | Keeps shapes valid enough for deeper code | Derived shapes can violate semantics |
| Grammar-aware generation | Source languages, configs, interpreters | fuzzcheck grammar mutators, custom generators | Higher parser acceptance and semantic reach | Requires grammar work |
| IR/program synthesis | Compilers, optimizers, JITs | Custom generators, LibAFL inputs | Hits optimization and codegen passes directly | Harder oracle design |
| Corpus distillation | Large long-running campaigns | `cargo fuzz cmin`, libFuzzer merge, `afl-cmin` | Removes redundant seeds | Can discard semantic scaffolding if overused |
| Taint/data-flow guidance | Hard comparisons, checksums, validators | Specialized tools or LibAFL feedback | Reaches guarded paths | Higher overhead and complexity |
| Stateful sequence mutation | REPLs, protocols, incremental parsers | Command-sequence inputs, reset hooks | Finds history-gated bugs | Repro and reset are harder |

If coverage plateaus for 24 to 72 hours with no meaningful new edges, states, or buckets, do not keep burning CPU. Add grammar, add a dictionary, add differential oracles, add taint/state feedback, or write a better harness.

## Stage-Split Harness Design

Prefer narrow targets:

- `lex_only`: arbitrary bytes to token stream, no semantic assumptions.
- `parse_only`: source bytes/string to AST, round-trip if printer exists.
- `parse_and_validate`: admit only syntax-valid inputs and drive name/type/scope checks.
- `lower_to_ir`: AST to IR, assert structural invariants.
- `optimize_ir`: compare optimized and unoptimized behavior or verifier invariants.
- `verify_bytecode`: fuzz bytecode verifier directly, no execution needed.
- `execute_interpreter`: execute verified bytecode with fuel and output limits.
- `execute_jit`: force tiering/warm-up and compare interpreter, baseline JIT, and optimized JIT.
- `repl_sequence`: fuzz sequences of commands/messages with explicit reset.

Keep end-to-end `compile_and_run` as a final integration lane, not the only lane.

## Parser-Compiler-VM Harness Template

```rust
#![no_main]

use libfuzzer_sys::{arbitrary::Arbitrary, fuzz_target};

#[derive(Debug, Arbitrary)]
struct ProgramInput {
    source: String,
    optimize: bool,
    fuel: u32,
}

fuzz_target!(|input: ProgramInput| {
    let fuel = input.fuel.min(50_000);

    if let Ok(ast) = my_lang::parse_module(&input.source) {
        let pretty = my_lang::print_module(&ast);
        let reparsed = match my_lang::parse_module(&pretty) {
            Ok(value) => value,
            Err(err) => panic!("printer emitted unparseable source: {err:?}"),
        };
        assert_eq!(ast, reparsed, "parse/print/parse mismatch");

        if let Ok(bytecode) = my_lang::compile_module(&reparsed, input.optimize) {
            let mut vm = my_lang::Vm::with_fuel(fuel as usize);
            let _ = vm.run(&bytecode);
        }

        let a = my_lang::eval_source(&input.source, false).ok();
        let b = my_lang::eval_source(&input.source, true).ok();
        assert_eq!(a, b, "optimization changed observable behavior");
    }
});
```

This template demonstrates the pattern, not a universal API contract. In production code, prefer fallible APIs and exact domain errors. A panic in a fuzz target should be an intentional oracle failure with a message that explains the violated property, not incidental unwrap/expect convenience.

## Safe Mutation Bias

Do not write libFuzzer custom mutator ABI hooks in Rust under this skill. They require raw pointer handling and violate the no-unsafe-code policy. Use safe biasing techniques instead:

- `-dict=<tokens.dict>` for keywords, operators, bytecode opcodes, pragmas, magic headers, and JIT intrinsics.
- `arbitrary` input structs and enums for typed shape preservation.
- fuzzcheck mutators and grammar mutators for recursive syntax and structured values.
- Safe parser-level generators that produce `String`, `Vec<u8>`, AST enums, or command sequences without raw pointers.
- Safe LibAFL components only when the project can express the executor/mutator without adding unsafe code.

Example dictionary file content:

```text
"fn"
"let"
"if"
"else"
"while"
"return"
"=>"
"=="
"{"
"}"
```

Example safe libFuzzer invocation with a dictionary:

```bash
cargo fuzz run parser --sanitizer address -- -dict=fuzz/dictionaries/parser.dict -max_total_time=120 -rss_limit_mb=2048 -print_final_stats=1
```

## Language-Specific Oracles

Parser oracles:

- `parse(print(parse(x))) == parse(x)` for accepted inputs.
- Formatting preserves AST shape or documented semantic shape.
- Reference parser accepts/rejects consistently where the grammars overlap.
- Error spans are bounded and do not panic on malformed Unicode or byte offsets.

Interpreter oracles:

- Desugared and original programs produce the same result.
- Constant folding preserves result.
- Evaluation is deterministic under fixed fuel and fixed environment.
- Step count, recursion, heap, and output limits are enforced as ordinary errors.

Compiler and optimizer oracles:

- Optimization levels preserve observable behavior.
- Different backends agree on UB-free generated programs.
- IR verifier accepts produced IR before and after every transform.
- Serialization round-trips preserve IR or bytecode semantics.

VM and bytecode oracles:

- Decode/encode/decode is stable for valid bytecode.
- Verifier rejection prevents execution.
- Executing generated bytecode and source-compiled bytecode agree when both are valid.
- Stack, register, heap, and host-call bounds are enforced.

JIT oracles:

- Interpreter, baseline JIT, and optimized JIT agree under identical budgets.
- Warm-up and tier transitions are forced intentionally.
- Side effects, host calls, output size, and nondeterminism are controlled.

Stateful oracles:

- Command sequences preserve documented state-machine invariants.
- Reset returns the system to the initial state.
- Invalid transitions return errors without poisoning later valid transitions.

## Sanitizer and Coverage Lanes

| Lane | Use | Caveat |
| --- | --- | --- |
| ASan + LSan | Existing memory-sensitive code, pre-existing FFI, allocator misuse, OOB, UAF, leaks | Default bug-finding lane, but not a proof |
| MSan | Uninitialized data in mixed runtimes | All code, including C/C++ deps and std, must be instrumented |
| TSan | Parser/interpreter/compiler races | Synchronization must be visible; atomics/fences can limit signal |
| Clang UBSan | Embedded C/C++ runtimes and native helpers | Treat as mixed-language adjunct, not Rust-native coverage by default |
| Rust source coverage | Harness blind spots and corpus quality | Coverage alone does not prove semantic depth |
| Miri repro lane | Minimized failures involving pre-existing unsafe code | Slow, not coverage-guided, and not a replacement for fuzzing; do not add unsafe for Miri |

Record sanitizer name, sanitizer flag or default, sanitizer initialization/report evidence when visible, toolchain, target triple, Cargo features, `RUSTFLAGS`/`RUSTDOCFLAGS`/engine env vars, corpus path, artifact path, time budget, exact command, and exit status for every evidence claim.

For coverage-gap review in Rust, prefer `cargo fuzz coverage`, source-based coverage, and `cargo-llvm-cov` unless current Fuzz Introspector docs prove Rust support for the project. The OSS-Fuzz Fuzz Introspector page currently names C/C++, Python, and Java for report generation.

## Crash Lifecycle

1. Reproduce the exact artifact with the exact target.
2. Preserve the original artifact and relevant corpus outside the mutable engine output directory.
3. Minimize the artifact with `cargo fuzz tmin`, libFuzzer `-minimize_crash=1`, `afl-tmin`, or honggfuzz replay/debug workflows under a recorded budget.
4. Recover semantic context with `cargo fuzz fmt` or a project-specific formatter when structured inputs are used.
5. Bucket by sanitizer type plus language phase plus top frame.
6. Add a deterministic regression test or regression corpus file.
7. Fix the bug.
8. Replay the minimized artifact and relevant regression suite.
9. Keep the artifact if the fix cannot be landed immediately.

Do not deduplicate only by filename hash. Language bugs cluster by phase, not by artifact name.

## CI Shape

Pull-request smoke fuzzing should run 1 to 5 minutes per selected target and replay known regressions. Scheduled jobs should run longer matrix campaigns with sanitizer lanes and upload corpora, artifacts, and coverage snapshots.

Minimal PR smoke shape:

```yaml
name: fuzz-smoke

on:
  pull_request:

jobs:
  fuzz:
    runs-on: ubuntu-latest
    timeout-minutes: 20

    env:
      CARGO_FUZZ_VERSION: 0.12.0
      FUZZ_TIME: 120

    strategy:
      fail-fast: false
      matrix:
        target: [parser, compiler_frontend, bytecode_verifier]

    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
        with:
          components: llvm-tools-preview
      - name: Install native deps
        run: |
          sudo apt-get update
          sudo apt-get install -y clang llvm lld
      - uses: actions/cache@v4
        with:
          path: ${{ runner.tool_cache }}/cargo-fuzz
          key: cargo-fuzz-bin-${{ env.CARGO_FUZZ_VERSION }}
      - name: Install cargo-fuzz
        run: |
          echo "${{ runner.tool_cache }}/cargo-fuzz/bin" >> $GITHUB_PATH
          cargo install --root "${{ runner.tool_cache }}/cargo-fuzz" --version "${{ env.CARGO_FUZZ_VERSION }}" cargo-fuzz --locked
      - name: Replay known regressions
        run: |
          if ls fuzz/regressions/${{ matrix.target }}/* >/dev/null 2>&1; then
            timeout 180s cargo fuzz run ${{ matrix.target }} --sanitizer address -- fuzz/regressions/${{ matrix.target }}/*
          fi
      - name: Smoke fuzz
        run: timeout 180s cargo fuzz run ${{ matrix.target }} --sanitizer address -- -max_total_time=${{ env.FUZZ_TIME }} -rss_limit_mb=2048 -print_final_stats=1
      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: fuzz-smoke-${{ matrix.target }}
          path: |
            fuzz/artifacts/${{ matrix.target }}
            fuzz/corpus/${{ matrix.target }}
```

Scheduled jobs should add sanitizer matrix entries, longer budgets, and coverage snapshots such as `cargo fuzz coverage <target>` for address-sanitized lanes. For Rust OSS-Fuzz integration, verify the current Rust project guide before promising engines or sanitizers; current guidance expects `cargo fuzz` and lists `libfuzzer` plus `address` for the Rust `project.yaml` path.

## Monitoring Metrics

Track at least:

- Executions per second.
- Current RSS.
- Edges found.
- Corpus count.
- Favored and pending corpus counts where supported.
- Unique crashes.
- Unique hangs and timeouts.
- Time since last new edge.
- Coverage-gap findings from source coverage or Fuzz Introspector.

If only total coverage percentage is monitored, campaigns can plateau while huge statically reachable regions remain untouched.

## Extreme Campaign Checklist

- Split harnesses by stage.
- Make persistent targets resettable.
- Run at least one byte lane and one structure-aware lane for syntax-consuming components.
- Maintain dictionaries for keywords, operators, pragmas, magic headers, bytecode opcodes, and JIT intrinsics.
- Bound recursion, AST nodes, bytecode length, interpreter fuel, heap growth, JIT warm-up, thread count, and output size.
- Use a sanitizer matrix instead of one default lane.
- Add differential or metamorphic oracles for compiler and optimizer targets immediately.
- Force tiering and optimization states for JIT campaigns.
- Distill corpora regularly, but preserve semantically rich seeds before minimization.
- Replay every minimized bug as a deterministic regression.
- Switch lanes when coverage or state discovery plateaus.
- Move stable public Rust project harnesses to OSS-Fuzz when maintainers can handle continuous triage.

## Source Priority for Fresh Facts

When exact commands, flags, or platform support matter, verify against current upstream docs instead of relying on memory:

- Rust Fuzz Book for cargo-fuzz conventions.
- cargo-fuzz and libFuzzer docs for flags, corpus merge/minimize, dictionaries, and custom mutators.
- AFL++ and afl.rs docs for persistent mode, CMPLOG, minimization, and monitoring.
- honggfuzz-rs docs for run, persistent mode, and replay/debug.
- LibAFL docs for custom executors, feedback, mutators, and distributed setup.
- fuzzcheck book for structured mutators and corpus layout.
- Rust sanitizer docs for current sanitizer support and caveats.
- OSS-Fuzz and ClusterFuzzLite docs for CI, batch, coverage, and managed fuzzing.
