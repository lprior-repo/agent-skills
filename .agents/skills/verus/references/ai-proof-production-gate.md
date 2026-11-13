# AI Verus Proof Production Gate

Use this when an agent writes, repairs, reviews, benchmarks, or evaluates AI-generated Verus proofs in a production Rust/Verus repository.

Core doctrine: **lock executable code and specs; let AI edit proof material only**. Verus statically checks Rust code against specifications without runtime checks, but it supports a Rust subset and changes over time, so pinned toolchains, intactness checks, and regression tests are part of correctness.

## Production-Style Pipeline

```text
production Rust/Verus repo
        |
        v
spec + exec code frozen
        |
        v
AI proposes proof-only patch
        |
        v
intactness checker: no exec/spec/TCB drift
        |
        v
Verus verification in pinned toolchain
        |
        v
normal Rust tests, fuzz/property tests, benchmarks
        |
        v
human review for specs, trusted code, and public APIs
```

## Immutable And Mutable Regions

Default AI permissions:

| Region | Default policy | Human override required? |
|---|---|---|
| `exec` bodies and executable control flow | Immutable | Yes |
| public/private function signatures | Immutable | Yes |
| `requires`, `ensures`, `recommends`, invariants, decreases, view/spec definitions | Immutable unless the task explicitly approves a spec change | Yes |
| trusted files/modules, TCB manifests, axioms, external specs, external bodies | Immutable | Yes |
| proof blocks, proof functions, local assertions, lemmas, loop invariants, trigger hints, local reveal/fuel/scoped solver calls | Mutable | No, if they do not change obligations or runtime behavior |

Reject any AI patch that changes immutable regions unless the review packet includes explicit human approval and separate evidence for the contract/TCB/API delta.

## Intactness Gate

Verifier success is not enough. Classify results as:

| Grade | Required evidence |
|---|---|
| `Pass` | Verus verifier passes for the scoped target. |
| `Acceptable` | Verus passes **and** executable code/specifications/TCB are intact. |
| `Production-usable` | Acceptable **and** no new TCB/assume/admit/axiom/external boundary **and** full repository CI passes. |
| `Upstream-grade` | Production-usable **and** maintainers/domain owners reviewed the proof/spec/API impact. |

Intact means executable code, specs, APIs, and trusted boundaries are unchanged except for explicitly approved deltas. RVBench-style evaluation treats this separately from raw verifier pass rate; use an intactness checker or parser-safe diff before accepting AI proof output.

## Forbidden AI Moves

Hard reject or block when generated proof output includes any unapproved:

- executable code change
- signature/API shape change
- strengthened `requires` or `recommends`
- weakened `ensures`, invariant, decreases, or postcondition
- changed trusted module/file/manifest entry
- `assume`, `admit`, `assume_specification`, `assume_termination`
- new `axiom fn`, `broadcast axiom`, or axiom-like declaration
- new `#[verifier::external_body]`, `#[verifier(external_body)]`, `#[verifier::external]`, `#[verifier(external)]`, `#[verus_verify(external*)]`, or `exec_spec_unverified!`
- proof that only works by hiding obligations, changing public API shape, bypassing production code, or proving a copied model
- verifier-only runtime fork such as `cfg(verus_only)` changing executable behavior

## TCB Ledger

Keep a machine-readable trusted-computing-base ledger. Each trusted file, module, axiom, external body, external spec, unsafe boundary, FFI boundary, platform assumption, or verifier escape hatch needs:

- path and symbol/range
- owner
- reason
- scope
- expiry or revisit trigger
- compensating evidence
- affected proof claims
- label: `TRUSTED`, never `PROVED`

Useful convention from verified systems repos: separate trusted from untrusted/verified code by path/name policy, such as trusted `_t` files/directories versus untrusted `_u` implementation/proof files. Do not rely on naming alone; enforce it with manifest checks and review.

## Repository-Aware Retrieval

Single-function prompting is insufficient for production Verus. Retrieval must include:

- production module and dependency closure
- specs, views, datatype/type invariants, traits, and state-machine definitions
- existing proof lemmas and vstd lemmas
- current Verus guide/reference pages for syntax in use
- verifier diagnostics from the exact failed command
- project proof conventions and trusted-boundary ledger

Do not retrieve or expose the masked target proof body or near-duplicate future proof during evaluation.

## Golden Dataset Schema

Use this shape for internal proof-completion tasks:

```json
{
  "task_id": "repo_commit_file_symbol_hash",
  "repo": "your-org/your-repo",
  "repo_commit": "...",
  "verus_commit_or_release": "...",
  "z3_version": "...",
  "file": "src/foo/bar.rs",
  "symbol": "module::type::function_or_lemma",
  "task_type": "proof_completion",
  "immutable_regions": ["exec code", "specs", "trusted code"],
  "mutable_regions": ["proof blocks", "assertions", "lemmas", "loop invariants"],
  "input": "code with proof body masked",
  "golden": "human-accepted verified proof",
  "verify_command": "cargo verus verify ...",
  "test_command": "cargo test --workspace",
  "forbidden_patterns": "references/trust-marker-scan.md canonical grammar plus parser/token scan rule or scripts/trust_scan_adapter.py JSON output",
  "trust_scan_scope": ["changed proof-relevant files", "full verifier trust closure"],
  "dependency_closure": ["vstd::...", "crate::spec::...", "crate::proof::..."],
  "tags": ["loop", "quantifier", "memory", "linear", "temporal", "concurrency"]
}
```

Recommended splits:

1. public training examples
2. public validation examples
3. private held-out golden tasks
4. future-commit tasks from later production revisions

The future-commit split catches memorization and toolchain brittleness.

## Evaluation Metrics

Report metrics in this order:

1. Verus pass rate
2. Intact pass rate: exec/spec/TCB unchanged
3. Forbidden construct rate: `assume`, `admit`, axiom, external body/spec, verifier escape hatches
4. Full CI pass rate: verifier, Rust tests, formatting/lints, project scripts
5. Proof robustness: harmless refactors or pinned toolchain bump still verify
6. Dependency retrieval quality: right lemmas/modules/specs found
7. Human review score: maintainability, minimality, idiomatic Verus, proof locality

## Benchmark Ladder

Use a staged ladder instead of jumping straight to production code:

| Layer | Use | Sources |
|---|---|---|
| Verus syntax and local proof skill | Basic invariants, arithmetic, loops, assertions, forall/triggers, lemma style | Verus upstream examples, tutorial examples, feature tests, unit tests |
| Algorithm-level proof generation | Fast isolated proof regression | Verus-Bench from `microsoft/verus-proof-synthesis` |
| Repository-level proof completion | Cross-file dependencies and real code structure | VeruSAGE-Bench, RVBench |
| Production-like repository benchmark | Cross-module lemmas, conventions, dependency hunting | RVBench; internal masked proofs from production repos |
| Smaller pilot repo benchmark | VeriSMo-heavy dependency splits | RepoVBench |
| Broad vericoding benchmark | General verified-code/spec generation | `Beneficial-AI-Foundation/vericoding-benchmark` |
| Cross-language algorithm sanity | Compare Verus/Dafny/Lean under related contracts | AlgoVeri |

## Production-Style Reference Repos

Prioritize current, production-adjacent systems repos when building golden tasks or review examples:

| Repo | Why it matters |
|---|---|
| `asterinas/vostd` | Production-adjacent OS standard-library verification; memory management and synchronization; tracks Verus closely. |
| `microsoft/verified-storage` | Persistent-memory logs, transactions, key-value storage, crash consistency, corruption detection. |
| `anvil-verifier/anvil` | Distributed/control-plane verification for Kubernetes controllers and examples. |
| `microsoft/verismo` | Low-level AMD confidential-VM security-module verification; use as reference and soundness-cleanup warning, not as clean production code. |
| `verus-lang/verified-memory-allocator` | Mimalloc-inspired allocator; pointer/memory reasoning and large verification scripts. |
| `unsoundsystem/rlsf-verified` | Real-time allocator verification; intrusive structures, pointer lists, bitmap reasoning. |
| `matthias-brun/verified-nrkernel` | OS page-table management; hardware memory translation; trusted/untrusted boundary discipline. |
| `verus-lang/verified-node-replication` | Verified node-replication library; concurrency/distributed proof style. |
| `verus-lang/verified-ironkv` | IronFleet-inspired sharded hash-table KV host program in Verus. |
| `anvil-verifier/verus-tla` | Verus proof library for temporal logic, state machines, weak fairness, liveness, and safety. |

## Rollout

1. Run candidate models on Verus upstream examples, Verus-Bench, and AlgoVeri.
2. Move to VeruSAGE-Bench and RVBench for repository-level reasoning.
3. Create an internal golden set by masking proof bodies from accepted production proofs.
4. Freeze exec/spec/TCB regions and let AI edit only proof material.
5. Enforce intactness before verifier success is counted.
6. Run pinned Verus, full Rust CI, fuzz/property tests, and benchmarks where relevant.
7. Require human/domain-owner review for specs, TCB changes, public APIs, and accepted upstream-grade proofs.

## Reviewer Checklist

Reject if:

- exec code changed
- signatures changed
- specs, `requires`, `ensures`, `recommends`, invariants, or decreases were weakened/rewritten without approval
- trusted code or TCB manifest changed without approval
- new `assume`, `admit`, axiom, external body/spec, or verifier escape hatch exists
- proof only works by changing API shape or hiding obligations
- proof passes only outside full repo CI
- dependency context is missing or hallucinated
- verifier pass is reported without intactness status

Approve only if the requested grade's evidence is present:

- `PASS`: Verus passes on the pinned/project-approved toolchain.
- `ACCEPTABLE`: Verus passes, exec/spec/TCB intactness is shown, no unledgered trust expansion exists, and any scoped repo policy raising the bar is cited.
- `PRODUCTION_USABLE` or `UPSTREAM_GRADE`: full Rust/project CI passes for the scoped repo policy.
- diff is proof-only or explicitly approved.
- TCB manifest is unchanged or approved and ledgered
- proof uses existing lemmas where appropriate
- generated proof is readable, local, and maintainable
- source/proof/evidence mapping is exact
