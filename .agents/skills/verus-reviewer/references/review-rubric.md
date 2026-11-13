# Verus Review Rubric

## Phase 1 — Scope And Provenance

Reject if the review target is unclear. A valid review names:

- Production files and symbols.
- Verus proof/spec/harness files and symbols.
- Planned proof obligations or acceptance criteria.
- Evidence commands and raw summaries.
- Trust ledger entries for assumptions and external boundaries.
- Claimed proof grade: `PASS`, `ACCEPTABLE`, `PRODUCTION_USABLE`, or `UPSTREAM_GRADE`.
- Intactness evidence for executable code, signatures, specs, contracts, invariants, decreases, trusted code, and TCB manifests.
- Full CI/test evidence and human/domain-owner approval when claiming production or upstream grade.

## Phase 2 — Trust Boundary Audit

Run or inspect a scan using `../../verus/references/trust-marker-scan.md`. Parser-aware or token-tree scans are preferred. Regex fallback is acceptable only for same-line/simple attributes and must report `BLOCKED` for multiline attributes or complex `cfg_attr` cases without parser/token evidence.

Severity:

- `CRITICAL`: Marker hides production core logic, proves false, changes runtime behavior under `verus_only`, or is unledgered/unapproved in a proof claim.
- `HIGH`: Marker is an external boundary with incomplete owner/scope/expiry/compensating evidence, including external type/trait specs that assume future implementations.
- `MEDIUM`: Marker is old and scoped but not referenced from the proof evidence.
- `LOW`: Marker is ledgered and unrelated, but final report omitted explicit `TRUSTED` labeling.

## Phase 3 — AI Proof Intactness Gate

Use `../../verus/references/ai-proof-production-gate.md`. Raw Verus success is only `PASS`. Approval requires the claimed grade to match evidence:

| Grade | Minimum evidence |
|---|---|
| `PASS` | Exact Verus command passes for the scoped proof target. |
| `ACCEPTABLE` | `PASS` plus parser/token intactness evidence that `exec`, signatures, specs, contracts, invariants, decreases, trusted code, and TCB manifests are unchanged or approved. |
| `PRODUCTION_USABLE` | `ACCEPTABLE` plus no new unapproved trust markers and full repository CI/test evidence for the scoped policy. |
| `UPSTREAM_GRADE` | `PRODUCTION_USABLE` plus maintainer/domain-owner review of specs, APIs, and TCB impact. |

Reject if an AI proof patch changes immutable regions without explicit approval: executable code, public/private signatures, `requires`, `ensures`, `recommends`, spec/view functions, invariants, decreases, trusted modules, external boundary ledgers, or TCB manifests.

## Phase 4 — Contract And Code Parity

Reject if any behavior claim lacks a chain:

`contract clause -> production symbol -> proof artifact -> exact verifier command -> independent behavior test when runtime behavior changed`

Look for laundering:

- Stronger `requires` excluding hard cases or unsatisfiable production callers.
- Weaker `ensures`, deleted invariant, tautological postcondition, empty quantified domain, or contradictory state invariant.
- Proof helper reimplements production logic without proving correspondence.
- Extracted helper is verified but no bridge names production call sites, input/view equivalence, return/error/state equivalence, and visibility/bypass guarantees.
- Harness verifies a local model instead of the crate-wired function.
- State-machine proof proves abstract transition but no implementation bridge names actual event/state update sites.

## Phase 5 — Verus Proof Shape

Check:

- Modes: `spec`, `proof`, `exec`, `ghost`, and `tracked` values do not cross invalid boundaries.
- Contracts: `exec fn`/`proof fn` use `requires`/`ensures`; `spec fn` uses `recommends` only.
- Integer layer: specs use `int`/`nat`; exec conversions and arithmetic bounds are explicit.
- Loops: invariants restate preconditions, bounds, functional relation, and decreases metric.
- Quantifiers: triggers contain bound variables and match real use-site terms; no accidental matching loops.
- Collections: `Seq`, `Set`, `Map`, `ISet`, `IMap` equality uses extensional equality when needed.
- Abstraction: `closed`/`opaque` specs expose facts through lemmas, not by making everything open.
- Unfolding: `reveal`, `reveal_with_fuel`, `by(compute)`, and `by(compute_only)` are local and justified.
- Solver modes: `nonlinear_arith`, `integer_ring`, `bit_vector`, and high rlimits are scoped to exact goals.
- Type invariants: `use_type_invariant` or project lemmas appear at the point where invariant facts are used.
- State machines: tokens are linear/tracked, transitions match implementation events, storage protocols do not hide ownership transfer gaps.
- Safe API: public verified APIs callable from unverified Rust have `-V check-safe-api` output or an explicit dynamic-check/wrapper boundary.
- Non-vacuity: `requires`/`recommends` are satisfiable, quantified domains are nonempty when claimed, invariants are not contradictory, verified helpers are reachable from production, and postconditions constrain observable behavior.
- Parser-safe deltas: proof-affecting syntax changes enumerate `requires`, `ensures`, `recommends`, invariants, decreases, visibility, attributes, and exec body semantics; regex summaries alone are never sufficient for approval when parser-aware evidence is available.

## Phase 6 — Evidence Gate

Acceptable final evidence includes one of:

- Exact command from `proof-obligations.planned.jsonl` or `verification-ledger.jsonl`.
- Repository Verus script or task documented for the target.
- `cargo verus verify` for opted-in crates/workspace when cargo-verus is used.
- If `cargo verus` is used, evidence that the target crate opted into verification with `[package.metadata.verus] verify = true` and no “no crates opted into verification” warning appeared.
- Direct `verus <target>.rs` only when the target is the agreed proof unit.

Evidence must state:

- Command.
- Exit status.
- Verus version when available.
- Verifier summary and relevant warnings/notes.
- Trust scan result.
- Whether the command is final closure or iteration-only.
- Whether `--no-cheating` was used when supported, or why it could not be used for a ledgered boundary.
- Public-API proof claims include `-V check-safe-api` output or a stated non-public scope.
- Claimed grade and intactness result.
- Full CI/test result when claiming `PRODUCTION_USABLE` or `UPSTREAM_GRADE`.
- Maintainer/domain-owner review when claiming `UPSTREAM_GRADE`.

Reject as insufficient final evidence:

- `cargo verus focus` without a later full `cargo verus verify` or exact closure command.
- `cargo verus verify` output showing no crate opted into verification.
- `verus --version` alone.
- Formatting, cargo build/check, Rust tests, or documentation build without the Verus proof command.
- Logs quoted without command, exit status, or target.
- Proof evidence that omits required `--no-cheating` status or a ledgered reason.
- Regex-only safe-change summaries when parser-aware evidence is available and proof-affecting syntax changed.
- `PASS` presented as production-ready without intactness, no-new-trust, and CI/test evidence.
- `UPSTREAM_GRADE` presented without maintainer/domain-owner review.

## Finding Format

Use one line per finding plus repair detail when needed:

```text
path:line: CRITICAL: external_body verifies the contract without proving the body. Replace with a verified body or ledger it as TRUSTED and remove PROVED claim. Evidence: <command/scan>.
```

## Approval Standard

Approve only if:

1. No unresolved critical/high/medium/low findings remain.
2. Trust boundary is fully ledgered and labeled.
3. Every proof claim maps to actual source symbols or accepted extracted helpers with complete bridge evidence.
4. Exact verifier evidence is present and scoped correctly.
5. Claimed grade is supported by intactness, trust, CI/test, and human-review evidence.
6. No proof laundering, non-vacuity gap, unapproved trust marker, or contract delta remains.
