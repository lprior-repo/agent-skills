---
name: kani
description: "Kani bounded model checking skill for Rust. Use when writing, reviewing, repairing, or running Kani proof harnesses, `cargo kani`, `#[kani::proof]`, `kani::any`, `kani::assume`, `kani::cover`, `#[kani::unwind]`, `#[kani::should_panic]`, function contracts, stubs, bounded panic-freedom, arithmetic/index/state-transition verification, unsafe-code harnesses, or Kani counterexample triage."
allowed-tools:
  - Read
  - Write
  - Edit
  - Bash
  - Glob
  - Grep
---

# Kani Bounded Model Checking Engineer

Kani proves Rust proof harnesses by bounded symbolic execution over all modeled inputs in the harness domain. Treat Kani evidence as valid only when the exact harnesses, commands, bounds, assumptions, stubs/contracts, supported feature set, and verifier output are recorded.

```jsonl
{"kind":"meta","skill":"kani","version":"1.0.1","format":"markdown-with-embedded-jsonl"}
{"kind":"mission","goal":"Write, review, and repair Kani harnesses without hidden bounds, vacuous assumptions, unreported stubs/contracts, unsupported-feature blind spots, or hallucinated cargo-kani output."}
{"kind":"scope","owns":["Kani proof harnesses","cargo kani commands","harness inventory","symbolic inputs with kani::any and kani::bounded_any","assumption and cover discipline","loop unwinding bounds","panic/assertion/arithmetic/index checks","bounded state-transition checks","unsafe-code harness evidence","function contract harnesses","stubbing and verified stubbing audits","Kani counterexample and concrete-playback triage","kani-report.md evidence"]}
{"kind":"scope","does_not_own":["temporal or distributed design models","Verus proof bodies for Rust-local deductive proofs","Flux RS refinement annotations","UB-interpreter-only exploration","Loom/Shuttle/Stateright concurrency interleavings","runtime end-to-end CLI/API behavior","claiming unbounded whole-program correctness","inventing Kani output or tool availability"]}
{"kind":"rule","id":"bounded_claims_only","text":"Kani is bounded model checking. State claims as verified for named harnesses under recorded input bounds, unwind bounds, assumptions, feature flags, and supported constructs. Never generalize bounded Vec/string/depth proofs beyond their bounds."}
{"kind":"rule","id":"harness_inventory_first","text":"Every claim must map to named `#[kani::proof]` or `#[kani::proof_for_contract]` harnesses. Run or record `cargo kani list --format json` when available before claiming coverage."}
{"kind":"rule","id":"assumptions_are_debt","text":"`kani::assume`, `kani::any_where`, contract `requires`, manual `Arbitrary`, and bounded generators shrink or shape the state space. Audit each one and require `kani::cover` or equivalent non-vacuity evidence for critical domains."}
{"kind":"rule","id":"cover_not_proof","text":"`kani::cover!` is reachability/non-vacuity evidence only. It never proves safety, equality, injectivity, ordering, rejection, or field sensitivity without a corresponding `kani::assert`, contract postcondition, or checked panic/return property over production code."}
{"kind":"rule","id":"unwind_is_proof_context","text":"Loop and recursion bounds are part of the proof. Any unwinding assertion failure, under-unwind workaround, timeout, `UNDETERMINED`, unsupported-feature warning, or resource exhaustion invalidates success for that obligation."}
{"kind":"rule","id":"stubs_and_contracts_are_trust_boundaries","text":"Plain `#[kani::stub]` is an abstraction, not proof of equivalence. `#[kani::stub_verified]` requires separate successful contract harness evidence for the target. Report all stubs, contracts, modifies clauses, and feature flags separately from implementation checks."}
{"kind":"rule","id":"unsafe_caveat","text":"Kani checks many modeled panic, arithmetic, bounds, and pointer-dereference failures, but it does not prove full Rust UB freedom, aliasing-model compliance, data-race freedom, ABI correctness, inline assembly, or concurrency correctness. Label unsafe residual risks explicitly."}
{"kind":"rule","id":"negative_evidence","text":"When claiming invalid inputs or states are rejected, provide an exact negative harness, `#[kani::should_panic]` harness, contract precondition call-site failure, or other explicit rejection evidence. Missing negative evidence for such a claim is `BLOCKER`, not soft debt."}
{"kind":"rule","id":"experimental_features","text":"Function contracts, stubbing, loop contracts, concrete playback, source coverage, valid-value checks, uninit checks, and memory predicate APIs are feature-flagged or experimental unless current Kani docs and `cargo kani --help` prove otherwise. Evidence must include the exact `-Z` flags used."}
{"kind":"rule","id":"disabled_checks_are_blockers","text":"Kani flags that disable, skip, or weaken verification, including `--no-default-checks`, `--no-memory-safety-checks`, `--no-overflow-checks`, `--no-undefined-function-checks`, `--no-unwinding-checks`, `--no-assertion-reach-checks`, `--prove-safety-only`, `--only-codegen`, `--no-codegen`, and `--ignore-global-asm`, are `BLOCKER` for any claim relying on that property class. Waived use must downgrade the claim and record owner, reason, and compensating evidence."}
{"kind":"rule","id":"resource_governance","text":"CBMC can consume tens of GiB per harness. Default broad or unknown Kani runs to `-j 1` and execute them inside a cgroup memory cap such as `systemd-run --user --scope --collect -p MemoryHigh=20G -p MemoryMax=24G -p MemorySwapMax=0 ...`. Running unbounded `cargo kani`, especially with `-j > 1`, is a review blocker unless the user explicitly accepts the machine risk."}
{"kind":"rule","id":"no_hallucinated_evidence","text":"Never invent Kani version, harness list, command success, property counts, SARIF paths, concrete playback output, counterexamples, unsupported-feature status, or CBMC/Kani diagnostics."}
{"kind":"rule","id":"pipeline_boundary","text":"In the Rust proof stack, Kani owns bounded implementation evidence for numeric, indexing, panic-freedom, state-transition, and selected unsafe-code harnesses. Verus owns Rust-local deductive proofs; Flux owns refinement-type obligations; fuzz/Loom/Shuttle/Stateright remain complementary where scoped."}
{"kind":"ref","file":"references/kani-practice.md","use":"Practical mental model, official scope, tool boundaries, evidence wording, and non-claims."}
{"kind":"ref","file":"references/kani-patterns.md","use":"Harness idioms, bounded inputs, assumptions, cover points, contracts, stubs, unsafe harnesses, and anti-patterns."}
{"kind":"ref","file":"references/kani-harness.md","use":"CLI-first command selection, install/setup, evidence capture, scans, artifact contract, and failure triage."}
{"kind":"ref","file":"references/kani-curriculum.md","use":"Source priority, staged learning path, evaluation tasks, and black-hat checklist."}
```

## Mandatory Verification Gate

Run the exact Kani command named in `proof-obligations.planned.jsonl` or `verification-ledger.jsonl` when present. If no exact command exists, use the nearest repo script/task. If no Kani target or harness exists for the claim, report a blocker instead of fabricating evidence.

## Resource Governance

CBMC is the solver backend behind Kani and may grow to tens of GiB per harness. Treat memory, swap, and solver parallelism as part of the proof execution context.

- Prefer exact package/harness commands over full-workspace `cargo kani`.
- Default broad, unknown, or CI-parity Kani runs to `-j 1`; use `-j 2+` only with recorded memory headroom.
- Run broad or unknown Kani commands inside a cgroup cap. Example:

```bash
systemd-run --user --scope --collect \
  -p WorkingDirectory=<workspace> \
  -p MemoryHigh=20G \
  -p MemoryMax=24G \
  -p MemorySwapMax=0 \
  cargo kani -j 1 --output-format=regular <exact-package-or-harness-args>
```

- If `systemd-run` is unavailable, use the host's equivalent cgroup/container memory limit or explicitly report that the run was skipped to avoid machine exhaustion.
- Do not run unbounded `cargo kani`, `cargo kani -j 4`, or full harness inventories on a developer workstation unless the user explicitly asks for the risk.
- If an existing Kani/CBMC process is already exhausting memory, identify the process group first, then prefer cgroup termination or a temporary `prlimit --as=<bytes> --pid <pid>` cap over letting the machine swap storm.

The commands below are templates only. Replace placeholders with exact project paths, package names, harnesses, feature flags, and evidence paths before treating output as proof evidence.

```bash
command -v cargo >/dev/null
cargo --version
rustc --version --verbose
rustup show active-toolchain
cargo kani --version
if command -v kani >/dev/null; then kani --version; fi
cargo kani --help >/dev/null
cargo kani list --format json > <evidence-dir>/kani-harnesses.json
ruby -rjson -e 'JSON.parse(File.read(ARGV.fetch(0)))' <evidence-dir>/kani-harnesses.json
rg -n '#!?\[kani::(proof|proof_for_contract|should_panic|unwind|stub|stub_verified|requires|ensures|modifies|recursion|loop_invariant|loop_modifies|loop_decreases|solver)(\([^\]]*\))?\]|kani::(any|any_where|bounded_any|assume|assert|cover|mem::[[:alnum:]_]+)(!|::[^()]*)?\(|derive\((kani::)?Arbitrary\)|derive\((kani::)?BoundedArbitrary\)|impl([[:space:]]*<[^>]+>)?[[:space:]]+(kani::)?Arbitrary[[:space:]]+for|impl([[:space:]]*<[^>]+>)?[[:space:]]+(kani::)?BoundedArbitrary[[:space:]]+for|#\[bounded' --glob '*.rs' --glob '!**/target/**' <verified-scope>
rg -n '\bunsafe\b|unsafe\s*\{|unsafe\s+fn|unsafe\s+impl|transmute|MaybeUninit|from_raw_parts|from_raw|as_ptr|as_mut_ptr|asm!|global_asm!' --glob '*.rs' --glob '!**/target/**' <verified-scope>
cargo kani <exact-package-target-or-harness-command-from-proof-obligations> --output-format=regular
```

When these surfaces are present, run the matching exact commands too:

```bash
cargo kani -Z stubbing --harness <harness-with-kani-stub>
cargo kani -Z function-contracts --harness <proof_for_contract-harness>
cargo kani -Z function-contracts -Z stubbing --harness <caller-using-stub_verified>
cargo kani -Z mem-predicates --harness <harness-using-kani-mem-predicates>
cargo kani -Z concrete-playback --concrete-playback=print --harness <failing-harness>
```

Accepted Kani evidence must include command, exit status, Kani version, harness inventory, harness names, checked packages/features, solver/backend/CBMC args, disabled or weakening flags, bounds/unwind source, assumption/stub/contract/unsafe scans, property summary, and final `VERIFICATION:- SUCCESSFUL` output for required harnesses. Missing `cargo-kani`, invalid harness inventory JSON, unknown harnesses, missing required `-Z` support, absent contract proof for `stub_verified`, unaccounted disabled-check flags, failed/unwound/undetermined checks, unsupported reachable features, or missing negative target for rejection claims is `BLOCKER`, not pass.

## Workflow

1. Read `references/kani-practice.md`, `references/kani-patterns.md`, and `references/kani-harness.md` before editing or judging Kani work.
2. Classify the target: panic freedom, arithmetic/index safety, bounded collection behavior, state transition, unsafe boundary, contract proof, stubbed abstraction, or non-Kani concern.
3. Build a harness inventory and map each claimed property to exact `#[kani::proof]` or `#[kani::proof_for_contract]` harnesses.
4. Record assumptions, input bounds, `bounded_any` sizes, manual `Arbitrary`, and unwind sources before interpreting success.
5. Add or demand `kani::cover` evidence for critical assumption domains and boundary values.
6. Reject `cover!`-only harnesses, `assert(true)`, copied production models, and hardcoded structural inputs for behavior/property obligations.
7. Keep stubs, contracts, FFI models, unsafe code, and experimental `-Z` flags explicit in the trusted surface.
8. Run the exact Kani command, then triage counterexamples, unwinding failures, unsupported features, and vacuity before changing code.
9. Report bounded evidence, limitations, trusted abstractions, residual unsafe risk, and blockers.

## Output Contract

When writing, reviewing, repairing, or running Kani work, return:

- Files changed.
- Exact Kani commands run, exit status, and result.
- Kani version, Rust toolchain, packages/features/targets, solver/backend/CBMC args, disabled-check flags, and harness inventory.
- Resource controls used: `-j` value, cgroup/container memory cap, swap policy, timeout, or an explicit reason no Kani command was run.
- Harness-to-claim map with bounds, unwind policy, and checked property classes.
- Assumptions, `any_where`, `bounded_any`, `Arbitrary`, and cover/non-vacuity evidence.
- Stubs, contracts, `stub_verified`, FFI models, experimental flags, and trusted surfaces.
- Counterexample or concrete-playback summary from actual output when failing.
- Unsupported constructs, unsafe residual risks, waivers, and blockers.
