---
name: rust-contract
description: "Design-by-contract plus TLA+/Verus-first verification-layer planning for Rust. Produces contracts, temporal model plans, Verus obligations, theorem-kernel projections, proof obligations, performance evidence obligations, verification layers, and Given-When-Then plans before implementation."
---

# Rust Contract (Contract + TLA+/Verus-First Verification Layers + Fowler Tests)

This skill generates **contract-first specifications**, **TLA+ temporal model plans**, **Verus-first Rust-core obligations**, **theorem-kernel projections**, **performance evidence obligations**, **defense-in-depth verification layers**, and **Martin Fowler style test plans** for Rust work. It does **not** implement production code, proof/model code, or tests.

```jsonl
{"kind":"meta","skill":"rust-contract","version":"2.6.0","updated":"2026-05","format":"markdown-with-embedded-jsonl"}
{"kind":"principle","id":"contract_first","text":"Define preconditions, postconditions, invariants, and error taxonomy before any implementation."}
{"kind":"principle","id":"tla_temporal_default","text":"Default to TLA+ for workflow, protocol, scheduler, queue, retry, claim/lease, lifecycle, concurrent, distributed, and other temporal/state-over-time behavior. If no temporal model applies, write that explicitly in tla-spec.md with rationale."}
{"kind":"principle","id":"verus_first","text":"Default to Verus for Rust-local pure/core logic whenever the property can be expressed with Verus specs, invariants, proof functions, decreases clauses, or trusted boundary wrappers. If Verus cannot cover a scoped critical clause, record an explicit waiver with owner, reason, expiry, limitation, and compensating evidence."}
{"kind":"principle","id":"verification_first","text":"Every contract clause must map to a verification layer before implementation: TLA+ for temporal models, Verus, Lean/Aeneas/Hax for tiny theorem kernels, Kani, Miri/cargo-careful, proptest, fuzzing/Bolero, Loom/Shuttle/Stateright/Lockbud, mutation, coverage, static scan, performance, assembly/IR, API compatibility, release provenance, manual QA, a gauntlet lane, or an explicit waiver."}
{"kind":"principle","id":"scope_aware_high_assurance","text":"High verification is mandatory for bead scope, new regressions, required obligations, touched dependencies/unsafe/API surfaces, and release/critical work. Pre-existing unrelated repo-wide debt is recorded as DEFERRED_GLOBAL follow-up evidence, not as bead-local failure."}
{"kind":"principle","id":"token_efficient_obligations","text":"Put verifier intent in compact artifacts, not prompts. Every proof obligation must name risk, scope, required, checker, exact command or mode, owner_state, rerun_from, and expected_evidence so subagents receive small failure packets."}
{"kind":"principle","id":"no_invented_formal_targets","text":"Do not invent crate names, harness names, proof modules, theorem names, Moon tasks, or CLI commands. If an exact target is unknown, mark the obligation BLOCKED with the missing discovery step instead of hallucinating it."}
{"kind":"principle","id":"theorem_kernel_only","text":"Use Lean, Aeneas-to-Lean, or Hax-to-Lean only for tiny theorem-critical kernels, algebraic state transitions, protocol lattices, arithmetic bounds, parser/codec specs, refinement claims, and invariants that need proof-assistant extraction beyond Verus. Do not theorem-prove I/O shells, UI, async runtimes, or storage adapters directly."}
{"kind":"principle","id":"verus_not_optional_for_rust_core","text":"For Rust-local pure critical clauses, Verus is mandatory unless the contract records an explicit waiver. Kani/proptest/fuzz are complementary implementation evidence, not substitutes for a Verus proof when Verus can express the property."}
{"kind":"principle","id":"mechanical_empathy_claims_are_contracts","text":"Performance, zero-cost abstraction, vectorization, API compatibility, and release-provenance claims are contract clauses. They must map to exact benchmark/profiler/assembly/API/SBOM evidence or be declared non-goals."}
{"kind":"principle","id":"proof_obligations_jsonl","text":"Emit one compact JSONL proof obligation per contract clause so downstream agents pass file paths instead of repeating long prose."}
{"kind":"principle","id":"review_required","text":"The contract and verification layers require an independent review artifact before test planning, test writing, or implementation may consume them."}
{"kind":"principle","id":"fowler_tests","text":"Tests are executable specifications: expressive names, Given-When-Then, happy/error/edge coverage."}
{"kind":"principle","id":"no_implementation","text":"Do not write production code, proof code, harness code, or tests. Output only contracts, verification plans, proof obligations, and test plans."}
{"kind":"principle","id":"railway_oriented","text":"All fallible operations must be expressed as Result<T, Error> in the contract signatures."}
```

## Inputs

- Bead ID or feature description
- Any existing constraints, APIs, or domain language

If information is missing, list **open questions** and **assumptions** explicitly.

## Outputs

Produce these artifacts under `.beads/<bead-id>/` when a bead ID exists, or in the current working directory for standalone use:

1) `contract.md` - Design by contract specification
2) `tla-spec.md` - TLA+ temporal model boundary, properties, commands, or explicit non-applicability rationale
3) `lean-contract.md` - theorem-kernel projection or explicit statement that Verus owns the Rust-local proof obligations
4) `verification-layers.md` - Defense-in-depth verification plan
5) `proof-obligations.jsonl` - One machine-readable proof obligation per contract clause
6) `traceability-matrix.jsonl` - Contract clause to test/proof/tool mapping
7) `martin-fowler-tests.md` - Test plan with Given-When-Then scenarios

The independent reviewer writes `contract-verification-review.md` with `STATUS: APPROVED` or `STATUS: REJECTED`. `rust-contract` must not approve its own work.

## Workflow

### Step 1: Gather Context

- Read relevant docs or bead description
- Identify domain terms and constraints
- List open questions (if any)

### Step 2: Design by Contract

Define the contract **before** tests:

- Preconditions (what must be true before)
- Postconditions (what must be true after)
- Invariants (what is always true)
- Error taxonomy (exhaustive, semantic error variants)
- Function signatures (Result<T, Error> for all fallible ops)

### Step 3: TLA+/Verus Kernel Split

Before assigning broader verification layers, split the contract into TLA+-owned temporal behavior, Verus-owned Rust core logic, optional theorem-owned kernels, and the Rust/runtime shell:

- TLA+-owned temporal model: workflows, protocols, schedulers, queues, retries, claims/leases, lifecycle transitions, distributed coordination, inter-agent orchestration, eventuality/liveness, fairness, deadlock freedom, and state-over-time invariants.
- Verus-owned Rust core: pure functions, data-structure invariants, state transitions, typestate refinements, arithmetic/indexing bounds, panic/overflow freedom, parser/codec invariants, and loop invariants expressible in Verus.
- Theorem-owned kernel: tiny algebraic or extracted models that need Lean, Aeneas-to-Lean, or Hax-to-Lean beyond Verus.
- Rust shell behavior: I/O, async scheduling, networking, databases, filesystem, wall-clock time, FFI surfaces, UI, and orchestration glue.

For every workflow/protocol/concurrent/distributed temporal clause, write a TLA+ obligation unless there is an explicit waiver. A valid TLA+ obligation must name:

- Contract clause ID.
- TLA+ module/model path.
- Variables and state shape.
- Init/action/next relation names.
- Safety invariants.
- Temporal properties such as liveness, fairness, eventually, always, until, and deadlock freedom.
- State constraints, symmetry sets, and bounded model limits when TLC/Apalache needs them.
- Refinement relation connecting Rust/runtime behavior to the TLA+ model.
- Evidence command, usually an exact `tlc ...`, `apalache-mc check ...`, or `moon run :verify-proof` command.

For every Rust-local pure/core critical clause, write a Verus obligation unless there is an explicit waiver. A valid Verus obligation must name:

- Contract clause ID.
- Rust module/function/type target.
- Verus spec function, proof function, invariant, precondition, postcondition, decreases clause, or trusted-boundary wrapper.
- Abstract input/output model when the Rust representation is too concrete.
- Refinement or abstraction relation connecting runtime Rust behavior to the Verus proof surface.
- Runtime shell exclusions.
- Evidence command, usually an exact `verus ...` command or `moon run :verify-proof`.

Lean/Aeneas/Hax, Kani, proptest, fuzzing, and examples still matter, but they complement Verus. They do not replace Verus for Rust-local pure/core critical behavior unless the waiver says why Verus cannot express the property or cannot be introduced at acceptable scope.

### Step 4: Verification Layer Plan

For every precondition, postcondition, invariant, transition rule, and error variant, assign at least one verification layer:

- `tla-plus` - default temporal model layer for workflows, protocols, schedulers, queues, retry/claim/lease logic, distributed state, deadlock freedom, fairness, and liveness
- `verus` - default Rust-native proof layer for pure/core Rust contracts: preconditions, postconditions, refinements, data-structure invariants, state transitions, panic/overflow freedom, indexing bounds, and loop invariants
- `lean`, `aeneas-lean`, or `hax-lean` - tiny theorem kernel, extraction/refinement relation, algebraic transition proof, protocol lattice proof, arithmetic bound theorem, parser grammar theorem, codec theorem, or impossible-state proof beyond Verus
- `creusot`, `flux`, or `prusti` - secondary Rust-native contract/refinement tools when Verus is a poor fit or the repo already standardizes on them
- `kani` - bounded model check for numeric, indexing, state transition, or panic-freedom properties
- `crux-mir` - symbolic Rust test/model check for assertions and bounded state spaces
- `miri` - UB, aliasing, invalid layout, and interpreter-level Rust checks
- `sanitizer` - ASan/TSan/MSan/LSan or other nightly sanitizer evidence for unsafe, FFI, allocation, race, or memory-sensitive paths
- `proptest` - broad invariant exploration over generated inputs
- `cargo-fuzz` or `bolero` - malformed input, parser, codec, protocol, and state-machine adversarial input
- `loom`, `shuttle`, `stateright`, or `lockbud` - thread interleavings, races, cancellation, deadlocks, distributed protocol state spaces, and linearizability
- `cargo-careful` - unsafe/FFI-sensitive runtime checking where applicable
- `cargo-mutants` - test strength and mutation kill proof
- `cargo-llvm-cov` - branch/line coverage evidence
- `static-scan` - unsafe, panic, indexing, unchecked arithmetic, source clippy, and supply-chain gates
- `performance` - before/after benchmark, profiler, `perf`, `criterion`, `iai-callgrind`, `hyperfine`, or load-test evidence
- `assembly-ir` - `cargo asm`, `cargo llvm-ir`, `cargo llvm-lines`, `cargo bloat`, or equivalent symbol-level proof for zero-cost/vectorization/code-size claims
- `api-compat` - `cargo semver-checks` or equivalent public API compatibility evidence
- `release-provenance` - `cargo auditable`, `cargo cyclonedx`, `cargo deny`, `cargo vet`, or equivalent release artifact and SBOM evidence
- `crux`, `saw`, or `hax` - obligation-specific second-ring formal tools for bit-precise, unsafe, extracted, or proof-heavy code
- `manual-qa` - end-to-end user workflow proof where machine checks cannot observe behavior
- `gauntlet-fast`, `gauntlet-standard`, `gauntlet-deep`, `gauntlet-proof`, or `gauntlet-all` - Moon rollup lanes when one lane is the right evidence boundary
- `waiver` - explicit written justification when no verification layer applies

TLA+ obligations must name the model, variables, actions, invariants, temporal properties, model-check command, and Rust/refinement boundary. Verus obligations must name the Rust target, spec/proof surface, invariants, trusted boundary, and exact command. Keep proof-assistant work small: use Lean/Aeneas/Hax only when a tiny theorem kernel is actually better than Verus. Then verify the Rust shell by Kani/Crux, Miri/cargo-careful/sanitizers, fuzzing/Bolero/proptest, Loom/Shuttle/Lockbud/Stateright where concurrent or distributed, mutation, coverage, static scans, and manual QA. Use `gauntlet-proof` for proof-targeted changes, `gauntlet-deep` for high-risk defense-in-depth layers, and `gauntlet-all` for release/critical work.

Source lint obligations must target production/source code only, for example `cargo clippy --workspace --lib --bins --examples --all-features -- -D warnings`. Test compile/execution uses `cargo test` or `cargo nextest`; test implementation style such as helpers, loops, table-driven cases, or local mutability is not a formal-lint gate.

Performance and second-ring obligations must name the exact benchmark, profiler command, symbol, package, baseline revision, or release artifact expected from downstream agents. Do not use generic `cargo test` as evidence for a speed, assembly, API, or SBOM claim.

### Step 5: Proof Obligations JSONL

Write one JSON object per line. Do not write prose paragraphs in `proof-obligations.jsonl`.

Required fields:

- `id` - stable ID such as `INV-001`, `PRE-002`, `ERR-003`, or `THM-004`
- `contract_clause` - exact contract clause ID
- `target` - source module/function/type or proof module
- `claim` - concrete property to prove or verify
- `layer` - one of the verification layers above
- `checker` - exact tool, Moon gauntlet command such as `moon run :verify-proof`, or `waiver`
- `command` - exact command when known, or the Moon mode when the gauntlet lane owns the evidence
- `evidence` - expected artifact or command output file
- `expected_evidence` - exact stdout marker, artifact path, report field, or proof name the verifier must observe
- `risk` - `low`, `medium`, `high`, `proof`, `critical`, or `release`
- `scope` - `bead-local`, `touched-crate`, `changed-api`, `new-dependency`, `unsafe-boundary`, `concurrent`, `protocol`, or `workspace`
- `required` - boolean; required obligations must pass or have an approved waiver
- `mode` - `verify-fast`, `verify-standard`, `verify-deep`, `verify-proof`, `verify-all`, or `exact-command`
- `owner_state` - Go-skill state that owns the artifact or code if this fails
- `rerun_from` - smallest Go-skill state to rerun after repair
- `status` - always `planned` at contract time; formal-verifier writes execution result later
- TLA+-only fields when `layer` is `tla-plus`: `tla_module`, `model`, `config`, `variables`, `actions`, `invariants`, `temporal_properties`, `fairness`, `state_constraints`, and `refinement`
- Verus-only fields when `layer` is `verus`: `verus_target`, `spec_fn`, `proof_fn`, `invariants`, `trusted_boundary`, and `shell_exclusions`
- Lean-only fields when `layer` is `lean`, `aeneas-lean`, or `hax-lean`: `lean_module`, `theorem`, `model`, `refinement`, and `shell_exclusions`
- Second-ring fields when `layer` is `performance`, `assembly-ir`, `api-compat`, or `release-provenance`: `command`, `artifact`, `baseline`, and `acceptance_threshold` when applicable

Scope policy:

- Required `bead-local`, `touched-crate`, `changed-api`, `new-dependency`, `unsafe-boundary`, `concurrent`, and `protocol` obligations block on failure.
- `workspace` obligations block only for release/critical beads or explicit workspace-scoped contract clauses.
- Pre-existing unrelated workspace failures should be represented as `DEFERRED_GLOBAL` follow-up evidence by formal-verifier, not as failed bead-local obligations.

### Step 6: Martin Fowler Test Plan

Create test cases that fully specify behavior:

- Happy path tests (expressive names)
- Error path tests (each failure mode)
- Edge case tests (boundaries, empty, extremes)
- Contract verification tests (pre/post/invariants)
- At least one end-to-end scenario (if applicable)

### Step 7: Exit Criteria

Only finalize if:

- Every failure mode has a corresponding error variant
- Every pre/post/invariant has at least one test
- Every pre/post/invariant has at least one verification layer or explicit waiver
- `tla-spec.md` exists and either lists TLA+-owned temporal clauses or states why no temporal/state-over-time model applies
- `lean-contract.md` exists and either lists theorem-owned clauses or states why Verus owns the Rust-local proof obligations instead
- Every workflow, protocol, scheduler, retry, claim/lease, lifecycle, concurrent, or distributed transition has a TLA+ obligation or explicit waiver
- Every TLA+ obligation includes module/model, variables, actions, invariants, temporal properties, fairness/deadlock stance, exact command, and refinement boundary
- Every Rust-local pure deterministic critical behavior has a Verus obligation or explicit waiver
- Every Verus obligation includes target, spec/proof surface, invariants, trusted boundary, exact command, and shell exclusions
- Every Lean/Aeneas/Hax obligation is scoped to a tiny theorem kernel and includes module, theorem, model, refinement, and shell exclusions
- Every obligation includes risk, scope, required, command or mode, owner_state, rerun_from, and expected_evidence
- Every parser/codec/protocol boundary has fuzzing/Bolero or a waiver
- Every concurrent implementation path has Loom/Shuttle/Stateright/Lockbud or a waiver, in addition to TLA+ temporal coverage for model-level behavior
- Every non-trivial pure invariant has Verus plus proptest or Kani, unless a waiver explains why Verus cannot cover it
- Every performance, zero-cost, vectorization, public API, and release-provenance claim has an exact evidence layer or is explicitly listed as a non-goal
- `proof-obligations.jsonl` and `traceability-matrix.jsonl` are valid JSONL
- Test names describe behavior unambiguously

When a repository lacks verification tasks, point downstream agents to the templates in `../formal-verifier/templates/` instead of inventing commands.

## Output Templates

### contract.md

```markdown
# Contract Specification

## Context
- Feature:
- Domain terms:
- Assumptions:
- Open questions:

## Preconditions
- PRE-001: [ ]

## Postconditions
- POST-001: [ ]

## Invariants
- INV-001: [ ]

## Error Taxonomy
- Error::InvalidInput - when input violates PRE-001
- Error::NotFound - when the requested entity is absent
- Error::PreconditionViolation - when a caller bypasses validated construction

## Contract Signatures
- fn create_resource(input: ValidInput) -> Result<Resource, DomainError>

## Verus-Owned Clauses
- INV-001: [Rust-local pure critical behavior proven by Verus]

## TLA+-Owned Clauses
- None, or INV-002: [temporal workflow/protocol behavior model-checked by TLA+]

## Theorem-Owned Clauses
- None, or list tiny kernels projected in lean-contract.md

## Non-goals
- [ ]
```

### tla-spec.md

```markdown
# TLA+ Temporal Model Plan

## Boundary
- Temporal/workflow behavior:
- Rust/core behavior excluded from TLA+ and handled by Verus/Kani/tests:
- External systems abstracted:
- Non-applicability rationale: [only if no temporal model applies]

## TLA+-Owned Clauses
- None, or INV-002 -> specs/Workflow.tla::NoDoubleClaim

## Model Shape
- Module/model path:
- Variables:
- Init action:
- Next/actions:
- State constraints:
- Symmetry sets:
- Bounded model limits:

## Properties
- Safety invariants:
- Liveness/eventuality:
- Fairness assumptions:
- Deadlock freedom:
- Refinement to Rust/runtime behavior:

## Evidence Command
- tlc -config specs/Workflow.cfg specs/Workflow.tla
- or apalache-mc check --config specs/Workflow.cfg specs/Workflow.tla

## Waivers
- None, or list temporal clauses where TLA+ does not apply with owner, reason, expiry, and compensating evidence.
```

### lean-contract.md

```markdown
# Theorem Kernel Projection

## Boundary
- TLA+-owned temporal model:
- Verus-owned Rust core:
- Theorem-owned kernel:
- Rust/runtime shell:
- External systems excluded from theorem proof:

## Theorem-Owned Clauses
- None, or INV-001 -> proofs/pi_core/State.lean::transition_preserves_invariants

## Theorem Obligations
### THM-INV-001
- Contract clause: INV-001
- Rust/spec target: crate::core::State::transition
- Lean module: PiCore.State
- Theorem shape: transition_preserves_invariants
- Model: abstract State and Command values
- Refinement: Rust State validates into Lean State before transition and reifies after transition
- Shell exclusions: I/O, async scheduling, storage, wall-clock time
- Evidence command: lake build or moon run :verify-proof

## Waivers
- None, or list clauses where Verus/Lean/Aeneas/Hax do not apply with owner, reason, expiry, and compensating evidence.
```

### verification-layers.md

```markdown
# Verification Layers

## Boundary
- Verus-owned kernel:
- TLA+ temporal model:
- Theorem projection:
- Runtime shell:
- External systems excluded from formal proof:

## Layer Assignment
- INV-002 -> tla-plus + stateright/loom when implementation concurrency exists
- PRE-001 -> verus + proptest + kani
- INV-001 -> verus + proptest
- ERR-001 -> Fowler scenario + mutation
- PERF-001 -> performance + assembly-ir
- API-001 -> api-compat
- REL-001 -> release-provenance

## Verus Scope
- Rust target:
- Spec/proof function:
- Invariants:
- Trusted boundary:
- Shell exclusions:

## TLA+ Scope
- Module/model path:
- Variables:
- Actions:
- Safety invariants:
- Temporal properties:
- Fairness/deadlock stance:
- Refinement boundary:
- Evidence command:

## Theorem Scope
- Theorem module:
- Rust target:
- Abstraction relation:
- Shell exclusions:
- Non-goals:

## Waivers
- None, or list clause IDs with reason and compensating evidence.
```

### proof-obligations.jsonl

```jsonl
{"id":"TLA-WF-001","contract_clause":"INV-002","target":"specs/Workflow.tla","claim":"workflow never double-claims a bead and every claimed bead eventually reaches terminal cleanup or explicit failure","layer":"tla-plus","checker":"tlc","command":"tlc -config specs/Workflow.cfg specs/Workflow.tla","evidence":"tla-report.md","expected_evidence":"TLC reports no invariant violations, no deadlock, and temporal properties satisfied for Workflow.cfg bounds","risk":"proof","scope":"protocol","required":true,"mode":"verify-proof","owner_state":3,"rerun_from":3,"status":"planned","tla_module":"Workflow","model":"specs/Workflow.tla","config":"specs/Workflow.cfg","variables":["beads","claims","states","workers"],"actions":["Init","Claim","Advance","Fail","Cleanup"],"invariants":["NoDoubleClaim","ValidState"],"temporal_properties":["EventuallyTerminalOrFailed"],"fairness":"weak fairness on Advance and Cleanup under enabled actions","state_constraints":["finite beads/workers for TLC bounds"],"refinement":"Rust bead lifecycle events refine TLA+ actions by bead ID and state"}
{"id":"VERUS-INV-001","contract_clause":"INV-001","target":"crate::core::State::transition","claim":"invalid transitions are unrepresentable and transition preserves core invariants","layer":"verus","checker":"verus","command":"verus crates/core/src/state_verus.rs","evidence":"verus-report.md","expected_evidence":"Verus verified crate::core::State::transition proof obligations with 0 errors","risk":"proof","scope":"bead-local","required":true,"mode":"verify-proof","owner_state":3,"rerun_from":3,"status":"planned","verus_target":"crate::core::State::transition","spec_fn":"spec_transition","proof_fn":"proof_transition_preserves_invariants","invariants":["valid_state","valid_transition"],"trusted_boundary":"validated State and Command constructors","shell_exclusions":["I/O","async scheduling","storage","wall-clock time"]}
{"id":"ERR-001","contract_clause":"ERR-001","target":"crate::parser::parse","claim":"arbitrary bytes never panic","layer":"cargo-fuzz","checker":"cargo fuzz run parse","command":"cargo fuzz run parse -- -runs=1000","evidence":"formal-verification-report.md","expected_evidence":"fuzz target parse completes 1000 runs without panic or sanitizer failure","risk":"high","scope":"touched-crate","required":true,"mode":"verify-deep","owner_state":6,"rerun_from":8,"status":"planned"}
{"id":"PERF-001","contract_clause":"PERF-001","target":"crate::parser::parse_hot_path","claim":"parser remains below the accepted p99 latency budget","layer":"performance","checker":"cargo bench --bench parser_hot_path","command":"cargo bench --bench parser_hot_path","evidence":"performance-report.md","expected_evidence":"criterion report shows p99 regression within acceptance_threshold against baseline","risk":"high","scope":"changed-api","required":true,"mode":"exact-command","owner_state":6,"rerun_from":8,"status":"planned","artifact":"target/criterion/parser_hot_path/report/index.html","baseline":"origin/main","acceptance_threshold":"no more than 10 percent p99 regression"}
{"id":"ASM-001","contract_clause":"PERF-001","target":"crate::parser::parse_hot_path","claim":"hot parser loop has no dynamic dispatch in the release assembly","layer":"assembly-ir","checker":"cargo asm --lib crate::parser::parse_hot_path","command":"cargo asm --lib crate::parser::parse_hot_path","evidence":"second-ring-evidence.md","expected_evidence":"inspected release assembly for crate::parser::parse_hot_path contains no dynamic dispatch in the hot loop","risk":"high","scope":"changed-api","required":true,"mode":"exact-command","owner_state":6,"rerun_from":8,"status":"planned","artifact":"second-ring-evidence.md","baseline":"origin/main","acceptance_threshold":"no dynamic dispatch in the inspected hot symbol"}
```

Bundle-level obligations may use gauntlet layers when the lane is the artifact being certified:

```jsonl
{"id":"GATE-001","contract_clause":"INV-001","target":"workspace","claim":"proof-targeted change passes TLA+/Verus/Kani/theorem proof lane","layer":"gauntlet-proof","checker":"moon run :verify-proof","command":"moon run :verify-proof","evidence":"formal-verification-report.md","expected_evidence":"verify-proof exits 0 and records all scoped proof obligations as PASS or WAIVED","risk":"proof","scope":"bead-local","required":true,"mode":"verify-proof","owner_state":12,"rerun_from":12,"status":"planned"}
{"id":"GATE-002","contract_clause":"REL-001","target":"workspace","claim":"release-critical change passes full verification gauntlet","layer":"gauntlet-all","checker":"moon run :verify-all","command":"moon run :verify-all","evidence":"formal-verification-report.md","expected_evidence":"verify-all exits 0 with no blocking local, regression, release, or required-obligation failures","risk":"release","scope":"workspace","required":true,"mode":"verify-all","owner_state":12,"rerun_from":12,"status":"planned"}
```

### traceability-matrix.jsonl

```jsonl
{"contract_clause":"INV-002","tests":["given_two_workers_when_claim_same_bead_then_only_one_claim_succeeds"],"proofs":["TLA-WF-001"],"review":"contract-verification-review.md"}
{"contract_clause":"INV-001","tests":["given_valid_transition_when_applied_then_next_state_is_exact"],"proofs":["VERUS-INV-001"],"review":"contract-verification-review.md"}
{"contract_clause":"ERR-001","tests":["given_invalid_input_when_parsed_then_exact_error_variant"],"proofs":["ERR-001"],"review":"contract-verification-review.md"}
```

### martin-fowler-tests.md

```markdown
# Martin Fowler Test Plan

## Happy Path Tests
- test_returns_success_when_valid_input_provided
- test_creates_resource_when_preconditions_met

## Error Path Tests
- test_returns_error_when_invalid_input
- test_returns_error_when_resource_not_found

## Edge Case Tests
- test_handles_empty_input_gracefully
- test_handles_boundary_values_correctly

## Contract Verification Tests
- test_precondition_<name>
- test_postcondition_<name>
- test_invariant_<name>

## Given-When-Then Scenarios
### Scenario 1: <name>
Given: all named preconditions hold
When: the contracted operation executes
Then:
- the exact postcondition holds
```

## Notes

- Do not implement code in this skill.
- Do not write TLA+ model code, Verus, Lean, Kani, fuzz, loom, or test harness code in this skill unless the user explicitly changes scope.
- Do write TLA+ temporal model obligations, Verus obligations, theorem-kernel obligations, and abstraction/refinement contracts; proof/model code belongs to later proof/harness work, not this skill.
- Do not invent formal targets, exact commands, or proof names. Mark unknown targets as blockers or discovery obligations.
- Do not approve your own contract or verification plan.
- Use ASCII only unless the repo already uses non-ASCII.
- Keep outputs precise, testable, and unambiguous.

## References

- `../formal-verifier/templates/moon-rust-verification.yml` - Moon task template for the five verification modes.
- `../formal-verifier/templates/rust-verification-gauntlet.sh` - fail-closed shell gauntlet used by those Moon tasks.
