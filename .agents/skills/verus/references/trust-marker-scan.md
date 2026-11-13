# Verus Trust Marker Scan

Canonical trust-marker detection for Verus proof review. Use this as a semantic requirement, not as a shell-only mandate.

## Required Scan Scope

Run or inspect both scopes before approving a proof claim:

1. **Changed proof-relevant files** — catches newly introduced trust markers or contract deltas.
2. **Verifier trust closure** — all Rust/Verus files compiled or imported by the exact final verifier command, including helper crates/modules and visible external specifications.

Every hit must map to a trust-ledger row with owner, reason, scope, expiry/revisit trigger, compensating evidence, and `PROVED`/`TRUSTED` claim separation.

## Canonical Text Pattern

Use host-native structured search, parser-aware, or token-tree scanning first. Use this regex only as a same-line text fallback; it is not sufficient for multiline attributes or nested `cfg_attr`. Any `cfg_attr` hit in changed proof-relevant files or verifier closure must be classified with parser/token-tree evidence, proved unrelated by a scoped source line, or reported `BLOCKED`.

```regex
assume\s*\(|admit\s*\(|assume_specification|assume_termination|external_type_specification|external_trait_specification|external_trait_extension|exec_spec_unverified\s*!|#\s*\[\s*verifier\s*(::\s*(external_body|external)|\(\s*(external_body|external)\s*\))\s*\]|#\s*\[\s*verus_verify\s*\(\s*(external_body|external)\s*\)\s*\]|verifier\s*::\s*(external_body|external)|verifier\s*\(\s*(external_body|external)\s*\)|verus_verify\s*\(\s*(external_body|external)\s*\)|external_fn_specification|broadcast\s+axiom|\baxiom\b|cfg_attr\s*\(|cfg\s*\([^)]*verus_only
```

## Code-Parity Signals

These are not automatically trusted-base expansions, but they require proof-to-code parity review:

```regex
when_used_as_spec|dual_spec|spec_exec|exec_spec
```

If these appear, require a parser-aware or token-tree safe-change artifact that lists changed contract clauses, invariants, decreases clauses, spec visibility, attributes, and executable body/CFG changes for each changed symbol.

## Adapter Rule

Prefer host-native structured search when available. If a reusable scanner is allowed, run `../scripts/trust_scan_adapter.py --changed <changed-files> --closure <verifier-closure-files> --json` from this reference directory or the equivalent skill-root-relative path. If only shell tools are available, run `rg -n '<canonical-pattern>' <scope>` as a fallback and separately classify every `cfg_attr` hit in the changed files plus verifier closure. Do not skip the scan because `rg` is unavailable; use the host search tool, AST/token scanner, bundled adapter, or report `BLOCKED`.

The bundled adapter is intentionally lightweight: it scans Rust attributes as balanced token trees, catches parenthesized `#[verifier(external_body)]`, spaced attributes, and multiline/nested `cfg_attr(...)`, then emits JSON with `file`, `line`, `column`, `scope`, `kind`, and `scan`. It is not a verifier and does not prove a trust marker is safe; every hit still needs a trust-ledger row or a `BLOCKED` finding.
