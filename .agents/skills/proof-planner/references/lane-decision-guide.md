# Lane Decision Guide

Write one planner-owned `verifier-lane-decision/v1` row per `(requirement_id, contract_clause, proof_seed_id, verifier)`. `required` rows point to obligation IDs. `not_applicable` rows cite concrete evidence. `blocked_tooling` blocks and never passes. Do not write reviewer dispositions; the independent reviewer writes `verifier-lane-review.jsonl`.
