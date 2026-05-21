# Proof Pipeline Contract

Do not renumber Go-skill states. Rewire responsibilities inside the existing 1..16 lifecycle.

## State Ownership

- State 3: `rust-contract` emits domain/type/workflow/error/boundary/hazard artifacts and `proof-seeds.jsonl`; no verifier artifacts, tests, or final proof obligations.
- State 4: `proof-planner` emits lane decisions, proof obligations, trusted-base plan, and waiver candidates; `proof-plan-reviewer` must approve with independent provenance and write `verifier-lane-review.jsonl`.
- State 5: `proof-writer` writes proof/model/harness artifacts only and records smoke evidence plus trusted-base debt.
- State 6: `proof-reviewer` rejects weak, vacuous, unmapped, under-evidenced, or trust-heavy proofs.
- State 7: `proof-to-implementation` maps proof claims to Rust source refs, behavior tests, refinement harness refs, and exact evidence commands; `proof-reviewer` independently approves the bridge review.
- State 8: `test-planner` plans behavior tests.
- State 9: `test-writer` writes failing-first behavior tests.
- State 10: `test-reviewer` reviews behavior tests only.
- State 11: `holzman-rust` implements production Rust.
- State 12: `formal-verifier` executes final commands and closes ledgers.
- State 13: `black-hat-reviewer` attacks proof/test/source/code parity.
- State 14: `truth-serum` and evidence packaging audit the bundle.

## Hard Rules

- Validator beats Markdown.
- Behavior-affecting waivers are forbidden.
- Verifier harnesses are not behavior tests.
- TLA+ is temporal evidence, not Rust implementation evidence.
- `PENDING_FORMAL_EXECUTION`, `mapping_status: planned`, and pending trusted-base dispositions must be closed by State 12.
- Review approval requires `agent-invocation-ledger.jsonl`; Markdown headers alone are forgeable.
