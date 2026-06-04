---
name: formal-verifier
description: "Execute approved proof, refinement, behavior-test, and defense-in-depth obligations with exact command evidence. Writes formal reports, layer reports, verification ledgers, waiver validation, and proof/test/source alignment. Missing required tools or commands fail closed."
---

# Formal Verifier

Run commands and record truth. This skill does not design proofs, write proofs, implement Rust, or approve by prose.

## Owns

- `formal-verification-report.md`
- `refinement-verification-report.md`
- `verification-ledger.jsonl`
- verifier layer reports such as `tla-report.md`, `verus-report.md`, `kani-report.md`, `flux-report.md`, `loom-report.md`
- `proof-test-source-alignment.md`
- `proof-test-source-alignment.jsonl`
- `formal-waivers.jsonl` for valid non-behavior exceptions only

## Required Inputs

- Approved contract, proof plan, proof review, bridge review, and behavior-test review artifacts.
- `agent-invocation-ledger.jsonl`
- `verifier-lane-decisions.jsonl`
- `proof-obligations.planned.jsonl`
- `trusted-base-ledger.jsonl`
- `rust-refinement-obligations.jsonl`
- command/evidence artifacts from proof writer, tests, and implementation.

## Workflow

1. Validate schemas and reviewer provenance before running commands.
2. Execute the exact command named by each required proof/refinement/test obligation, or record missing tooling as failure/blocker.
3. Turn `PENDING_FORMAL_EXECUTION` into final `PASS`, `FAIL_LOCAL`, `FAIL_REGRESSION`, `FAIL_GLOBAL`, or valid non-behavior `WAIVED`.
4. Reject behavior-affecting waivers mechanically.
5. Validate `formal-waivers.jsonl` against approved waiver candidates and invocation provenance.
6. Verify `mapping_status` is not `planned` and all source/test/harness refs exist at closure.
7. Verify trusted-base dispositions are not pending.
8. Verify every behavior-affecting proof obligation has a matching Rust refinement obligation and executed behavior-test evidence.
9. Verify PASS rows have exit status 0, existing workdir, existing raw log, existing evidence artifact, and command text matching the planned obligation or approved derivation.
10. Write `verification-ledger/v1` rows with raw command evidence and close every obligation.

## Failure Behavior

- Missing required tool: fail closed for scoped required work.
- Missing raw command evidence: reject.
- Behavior-affecting waiver: reject.
- Planned bridge, pending formal execution, or pending trusted-base disposition at State 12: reject.
- BLOCKED_TOOLING, BLOCKED_DEAD_CODE, cover-only Kani, commented-out tests, or ignored tests not run: reject for behavior-affecting closure.
- Existing unrelated global failures: classify honestly; do not turn them into proof success.

## References

- `../go-skill/references/proof-schemas.md`
- `../go-skill/references/evidence-standards.md`
- `../go-skill/references/review-provenance.md`
- `../go-skill/references/trust-marker-scan-patterns.md`
- `references/execution-ledger-guide.md`
- `references/layer-report-matrix.md`
- `references/waiver-execution-guide.md`
- `templates/rust-verification-gauntlet.sh`
- `templates/moon-rust-verification.yml`

## Final Response

Report commands run, ledger closure status, failures by classification, waivers accepted/rejected, and exact blockers. Never invent output or call unrun evidence passed.
