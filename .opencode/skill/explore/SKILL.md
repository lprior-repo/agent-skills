---
name: explore
description: "Codebase scout for bead delivery. Use before contract/proof/test work to map relevant files, APIs, crates, risks, dependencies, and existing verification artifacts without modifying production code."
argument-hint: "[bead-id, delivery goal, module, or scope path]"
allowed-tools:
  - Bash
  - Read
  - Grep
  - Write
disable-model-invocation: true
---

```jsonl
{"kind":"meta","skill":"explore","version":"1.0.0","format":"jsonl-progressive","mode":"artifact-writing-codebase-scout"}
{"kind":"mission","goal":"Map the smallest relevant codebase scope for a bead and write scout artifacts that downstream contract, proof, test, and implementation agents can trust."}
{"kind":"rule","id":"no_code_edits","text":"Do not edit production code, tests, proof artifacts, configs, or dependency files. During bead delivery, write only scout artifacts under `.beads/<bead-id>/`; for non-bead Q&A, answer without writing artifacts."}
{"kind":"rule","id":"scope_first","text":"Prefer focused Grep/Read and scoped shell discovery over broad repository scans. Start from bead text, changed files, module names, or user-provided paths, then widen only when evidence demands it."}
{"kind":"rule","id":"fact_evidence_only","text":"Every map entry must be backed by a file path, symbol name, command result, or explicit UNKNOWN marker. Do not infer ownership or behavior from naming alone."}
{"kind":"rule","id":"handoff_ready","text":"Outputs must be specific enough for rust-contract, proof-planner, test-planner, and holzman-rust to work without repeating open-ended discovery."}
{"kind":"workflow","id":"scout","steps":["Read bead request, STATE.md, baseline-report.md, and any provided scope hints.","Locate relevant crates/modules/APIs/tests/proofs/configs using Grep/Read and scoped shell discovery.","Identify risk tags: temporal, concurrency, unsafe/UB, persistence, auth/security, parser/codec, dependency, performance, public API, migration, or user-visible behavior.","Write `.beads/<bead-id>/codebase-map.md` with paths, symbols, existing tests/proofs, open questions, and recommended downstream owners.","Write or update `.beads/<bead-id>/delivery-scope.jsonl` with touched or suspected crates/files/APIs/dependencies/contracts/risk tags/required verifier modes."]}
{"kind":"artifact","id":"outputs","items":[".beads/<bead-id>/codebase-map.md",".beads/<bead-id>/delivery-scope.jsonl"]}
{"kind":"gate","id":"mandatory_verification_gate","text":"Before finalizing, prove the isolated workspace and scoped search inputs exist, then verify scout artifacts are non-empty and JSONL parses."}
{"kind":"gate","id":"anti_hallucination","text":"Never invent files, symbols, dependency changes, test coverage, proof coverage, or ownership. Mark unknowns as UNKNOWN and missing artifacts as MISSING."}
```

# explore

Use this skill at `go-skill` State 2 to create the scout packet for downstream agents.

This skill is artifact-writing but not code-writing. It may write only `.beads/<bead-id>/codebase-map.md` and `.beads/<bead-id>/delivery-scope.jsonl` from the isolated workspace.

## Mandatory Verification Gate

Run from the isolated workspace before finalizing State 2 artifacts.

```bash
pwd -P
test -s ".beads/<bead-id>/STATE.md"
test -s ".beads/<bead-id>/baseline-report.md"
rg -n "<bead keyword>|<module>|<public API>|<error type>" <scope-path>
test -s ".beads/<bead-id>/codebase-map.md"
test -s ".beads/<bead-id>/delivery-scope.jsonl"
jq -c . ".beads/<bead-id>/delivery-scope.jsonl" >/dev/null
```

If the scope is unknown, record `DISCOVERY_BLOCKED` in `codebase-map.md` with the exact missing input rather than guessing.

## Anti-Hallucination Shield

Forbidden:
- Claiming a file, crate, API, test, or proof exists without reading or locating it.
- Treating a filename match as behavioral proof.
- Marking verifier lanes unnecessary without a risk-based reason.
- Writing or modifying production code, tests, proofs, dependencies, or CI config.

Required:
- Include exact paths for every relevant file.
- Include unknowns and excluded paths explicitly.
- Include risk tags that downstream proof and test planning can consume.
- Keep `delivery-scope.jsonl` valid JSONL with one scoped row per file/API/dependency/contract cluster.
