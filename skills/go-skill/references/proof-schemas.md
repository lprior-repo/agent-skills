# Proof Pipeline Schemas

Canonical schemas for the Go-skill proof pipeline. `schema_version` is mandatory. Validator output beats Markdown.

## Required Schema Versions

- `proof-seed/v1`
- `verifier-lane-decision/v1`
- `verifier-lane-review/v1`
- `proof-obligation/v1`
- `waiver-candidate/v1`
- `formal-waiver/v1`
- `trusted-base-ledger/v1`
- `rust-refinement-obligation/v1`
- `verification-ledger/v1`
- `agent-invocation/v1`
- `finding/v1`

## Verifier Lane Profiles

Every `(requirement_id, contract_clause, proof_seed_id)` tuple needs lane decisions for the seed's profile. No verifier is silently omitted: if a profile requires a verifier, the lane decision is `required`, `not_applicable`, or `blocked_tooling` with reviewer acceptance.

Default Rust behavior profile:

- `kani`
- `verus`
- `flux-rs`
- `proptest`

Conditional profile additions:

- `loom` for implementation concurrency, cancellation, shutdown, atomics, channels, locks, task ownership, or interleaving risk.
- `cargo-fuzz` for parsers, codecs, binary/persisted payloads, hostile input, YAML/IPC/storage decoding, or fuzzable canonicalization boundaries.

## `proof-seed/v1`

Required fields: `schema_version`, `id`, `requirement_id`, `contract_clause`, `domain_claim`, `risk_tags`, `suggested_layers`, `behavior_affecting`, `model_boundary`, `notes`.

Purpose: domain-level proof hints from `rust-contract`. These are not proof obligations.

## `verifier-lane-decision/v1`

Required fields: `schema_version`, `id`, `requirement_id`, `contract_clause`, `proof_seed_id`, `verifier`, `risk_tags`, `applicability`, `decision_reason`, `required_obligation_ids`, `non_applicability_evidence_refs`, `limitation_kind`, `owner_state`, `status`.

Allowed `applicability`: `required`, `not_applicable`, `blocked_tooling`.

Rules: `required` rows name planned obligations. `not_applicable` rows cite concrete evidence. `blocked_tooling` blocks; it never passes. Missing or duplicate active lane rows fail validation.

## `verifier-lane-review/v1`

Required fields: `schema_version`, `id`, `lane_decision_id`, `requirement_id`, `contract_clause`, `proof_seed_id`, `verifier`, `reviewer_disposition`, `finding_refs`, `planner_invocation_id`, `reviewer_invocation_id`, `owner_state`, `status`.

Purpose: independent `proof-plan-reviewer` disposition for each planner-owned lane decision. Planner artifacts must not self-stamp reviewer disposition. Downstream proof writing requires `reviewer_disposition: accepted` for every lane row.

`reviewer_disposition` is lane-review status, not `finding/v1.disposition`. Findings must use only the canonical `finding/v1.disposition` values below.

## `proof-obligation/v1`

Required fields: `schema_version`, `id`, `requirement_id`, `contract_clause`, `domain_claim`, `risk`, `risk_tags`, `verifier`, `artifact`, `target`, `command`, `workdir`, `expected_evidence`, `assumptions`, `model_bounds`, `tool_metadata`, `trusted_base_refs`, `required`, `behavior_affecting`, `mode`, `owner_state`, `rerun_from`, `status`.

`target` is canonical. Legacy aliases `layer`, `checker`, and alias-only `claim` are invalid.

For implementation-bound behavior claims, proof obligations must target production code directly or an extracted production helper. Duplicated harness models are allowed only as model evidence and require bridge rows that state the copy/reality risk.

## `trusted-base-ledger/v1`

Required fields: `schema_version`, `id`, `obligation_id`, `artifact`, `location`, `marker`, `trusted_kind`, `reason`, `scope`, `impact`, `behavior_affecting`, `compensating_evidence`, `owner`, `expiry`, `reviewer_disposition`, `status`.

Every `assume`, `axiom`, `admit`, `external_body`, `trusted`, `ignore`, stub, disabled check, model bound, or model reduction needs one row.

Kani `cover!` is non-vacuity evidence only. It cannot be the sole satisfaction evidence for a safety, equality, injectivity, ordering, panic-freedom, or field-sensitivity obligation.

## `rust-refinement-obligation/v1`

Required fields: `schema_version`, `id`, `proof_id`, `requirement_id`, `contract_clause`, `proof_claim_ref`, `rust_target`, `behavior_affecting`, `source_refs`, `behavior_test_refs`, `refinement_harness_refs`, `refinement_claim`, `verifier`, `evidence_command`, `evidence_workdir`, `evidence_artifact`, `expected_evidence`, `mapping_status`, `required`, `owner_state`, `rerun_from`, `status`.

Allowed `mapping_status`: `planned`, `materialized`, `verified`. `planned` is allowed at State 7 and rejected at State 12 closure.

Every behavior-affecting proof obligation needs a matching Rust refinement obligation with concrete source refs, independent behavior tests, separate refinement harness refs, and executed command evidence by State 12.

## Waivers

`waiver-candidate/v1` required fields: `schema_version`, `id`, `requirement_id`, `contract_clause`, `reason`, `behavior_affecting`, `boundary_proof`, `compensating_evidence`, `owner`, `expiry`, `review_status`.

`formal-waiver/v1` required fields: `schema_version`, `id`, `waiver_candidate_id`, `waiver_candidate_hash`, `requirement_id`, `contract_clause`, `behavior_affecting`, `approved_candidate_review_invocation_id`, `formal_verifier_invocation_id`, `owner`, `expiry`, `boundary_proof`, `compensating_evidence`, `ledger_result_ref`, `status`.

Behavior-affecting waivers are invalid. Final waivers require `review_status: approved`, `status: approved`, future ISO-8601 expiry, canonical SHA-256 hashes of the candidate/formal rows, completed `proof-plan-reviewer` and `formal-verifier` invocation rows, and a matching `WAIVED` verification-ledger row.

## `verification-ledger/v1`

Required fields: `schema_version`, `id`, `obligation_id`, `obligation_kind`, `requirement_id`, `contract_clause`, `behavior_affecting`, `verifier`, `result`, `command`, `workdir`, `exit_status`, `tool_version`, `flags`, `bounds`, `seeds`, `raw_log`, `evidence_artifact`, `formal_waiver_id`, `formal_waiver_hash`, `formal_verifier_invocation_id`, `classification`, `rerun_from`, `status`.

Allowed `result`: `PASS`, `FAIL_LOCAL`, `FAIL_REGRESSION`, `FAIL_GLOBAL`, `WAIVED`. `WAIVED` rows require `behavior_affecting: false` and a matching `formal-waivers.jsonl` row.

## `agent-invocation/v1`

Required fields: `schema_version`, `ledger_sequence`, `previous_entry_hash`, `entry_hash`, `host_session_id`, `invocation_id`, `parent_invocation_id`, `skill`, `state`, `workdir`, `input_artifacts`, `input_artifact_hashes`, `output_artifacts`, `output_artifact_hashes`, `transcript_artifact`, `transcript_hash`, `reviewed_artifacts_existed_before_start`, `started_at`, `completed_at`, `status`.

Independent review requires control-plane or otherwise non-writer-controlled invocation evidence. Workspace Markdown headers alone are not proof.

## `finding/v1`

Required fields: `schema_version`, `reviewer_skill`, `review_state`, `finding_code`, `severity`, `artifact`, `owner_state`, `message`, `required_fix`, `disposition`.

Allowed `disposition` values:
- `fixed_with_evidence`: requires `evidence_refs` pointing to repair evidence or reviewed artifact rows.
- `owner_approved_debt`: requires `owner`, `approval_ref`, `rationale`, and `debt_ref`; only valid for non-behavior-affecting findings.
- `owner_approved_no_action`: requires `owner`, `approval_ref`, and `rationale`; only valid when the reviewer confirms the finding is non-blocking.
- `blocker`: requires `blocker_reason`; the owning review status must be `STATUS: REJECTED` or `STATUS: UNVERIFIED`.
