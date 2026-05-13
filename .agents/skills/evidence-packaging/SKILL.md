---
name: evidence-packaging
description: "Build a truth-serum-audited assurance bundle for bead delivery. Use after formal/test execution and black-hat review, before landing, to prove every requirement maps to raw evidence."
argument-hint: "[bead-id or assurance artifact root]"
allowed-tools:
  - Bash
  - Read
  - Glob
  - Grep
  - Write
disable-model-invocation: true
---

```jsonl
{"kind":"meta","skill":"evidence-packaging","version":"1.0.1","format":"jsonl-progressive","mode":"artifact-writing-evidence-bundling"}
{"kind":"mission","goal":"Package bead acceptance evidence into an auditable assurance bundle and run truth-serum against hallucinated, missing, or laundered evidence before landing."}
{"kind":"rule","id":"raw_evidence_only","text":"Subagent summaries are not evidence. Only commands, exit statuses, raw logs, artifact files, reviewer findings, and explicit waivers count."}
{"kind":"rule","id":"traceability_kernel","text":"Every user requirement must map to contract clause, proof obligation or test, execution evidence, review status, and final disposition."}
{"kind":"rule","id":"truth_serum_required","text":"Run truth-serum in the active execution context before approval. Delegated truth-serum output is review input only and cannot approve the bundle."}
{"kind":"rule","id":"no_new_claims","text":"Do not create new correctness claims during packaging. Package only already-produced artifacts and mark gaps as blockers."}
{"kind":"workflow","id":"evidence_packaging","steps":["Read delivery-scope.jsonl, contract.md, traceability-matrix.jsonl, proof-review.md, test-plan-review.md, formal-verification-report.md, verification-ledger.jsonl, black-hat-review.md, and gate reports.","Check every required artifact exists and is non-empty.","Build assurance-bundle.md with requirement-to-evidence mapping and unresolved waiver/debt table.","Run truth-serum audit in the active execution context against the bundle and raw artifacts.","Write truth-serum-report.md and final-evidence-decision.md with STATUS: APPROVED or STATUS: REJECTED."]}
{"kind":"artifact","id":"outputs","items":[".beads/<bead-id>/assurance-bundle.md",".beads/<bead-id>/truth-serum-report.md",".beads/<bead-id>/final-evidence-decision.md"]}
{"kind":"gate","id":"mandatory_verification_gate","text":"Verify required artifacts, JSONL validity, status lines, and raw evidence pointers. Missing or invalid evidence blocks landing."}
{"kind":"gate","id":"anti_hallucination","text":"Never invent command output, test counts, verifier status, reviewer approval, commit IDs, paths, or waiver decisions. Mark absent proof as MISSING_EVIDENCE."}
```

# evidence-packaging

Use this skill immediately before landing. It is the acceptance kernel: no evidence, no merge.

This skill does not decide product design, write code, or fix tests. It packages and audits the evidence produced by earlier states.

Read these supporting docs as needed:
- [references/assurance-bundle-template.md](references/assurance-bundle-template.md) for the required final bundle shape.
- [references/evidence-audit-checklist.md](references/evidence-audit-checklist.md) for truth-serum audit inputs.

## Mandatory Verification Gate

Run these from the isolated workspace before approving evidence packaging.

```bash
pwd -P
test -s ".beads/<bead-id>/delivery-scope.jsonl"
test -s ".beads/<bead-id>/contract.md"
test -s ".beads/<bead-id>/traceability-matrix.jsonl"
test -s ".beads/<bead-id>/proof-review.md"
test -s ".beads/<bead-id>/test-plan-review.md"
test -s ".beads/<bead-id>/formal-verification-report.md"
test -s ".beads/<bead-id>/verification-ledger.jsonl"
test -s ".beads/<bead-id>/black-hat-review.md"
test -s ".beads/<bead-id>/machine-gate-report.md"
test -s ".beads/<bead-id>/regression-diff.md"
jq -c . ".beads/<bead-id>/delivery-scope.jsonl" >/dev/null
jq -c . ".beads/<bead-id>/traceability-matrix.jsonl" >/dev/null
jq -c . ".beads/<bead-id>/verification-ledger.jsonl" >/dev/null
rg -n '^STATUS: APPROVED$|^STATUS: PASS$' ".beads/<bead-id>/proof-review.md" ".beads/<bead-id>/test-plan-review.md" ".beads/<bead-id>/formal-verification-report.md" ".beads/<bead-id>/black-hat-review.md"
```

Then run `truth-serum` in audit mode from the same active execution context against `.beads/<bead-id>/assurance-bundle.md` and the raw artifacts it references. Delegated truth-serum output may inform findings, but it cannot approve the bundle. If active-context truth-serum cannot run, write `final-evidence-decision.md` with `STATUS: REJECTED` or `STATUS: UNVERIFIED`.

## Anti-Hallucination Shield

Forbidden:
- Packaging a subagent sentence as proof.
- Omitting failed gates from the bundle.
- Reporting missing tools as passed.
- Claiming a requirement is covered without a traceability row.
- Allowing landing before truth-serum evidence audit passes.

Required:
- `assurance-bundle.md` must name every requirement and its evidence.
- `truth-serum-report.md` must include command evidence or explicit blockers.
- `final-evidence-decision.md` must include `STATUS: APPROVED`, `STATUS: REJECTED`, or `STATUS: UNVERIFIED`.
