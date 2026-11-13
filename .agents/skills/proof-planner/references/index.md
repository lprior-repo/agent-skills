# Proof Planner Reference Index

Read order for a fresh planner. Each reference cross-links to others.

## Core Workflow

1. **[obligation-schema.md](obligation-schema.md)** — `proof-obligation/v1` field-by-field schema with per-lane JSON examples. The binding contract.
2. **[risk-taxonomy.md](risk-taxonomy.md)** — `rust-contract` proof-seed `risk_tags` → mandatory lane profile. The classification source.
3. **[lane-decision-guide.md](lane-decision-guide.md)** — decision algorithm; `not_applicable` and `blocked_tooling` rules.
4. **[verifier-trigger-matrix.md](verifier-trigger-matrix.md)** — risk trigger → primary lane + counterindications.
5. **[defense-depth-matrix.md](defense-depth-matrix.md)** — minimum lane count per risk class.

## Discipline References

6. **[implementation-binding.md](implementation-binding.md)** — anti-laundering mandate (no `external_body`, `assume`, `axiom`, `cover!`-as-proof, copied models).
7. **[tooling-availability-gate.md](tooling-availability-gate.md)** — `BLOCKED_TOOLING` per lane and tooling acquisition beads.
8. **[evidence-requirements.md](evidence-requirements.md)** — minimum command/flag/output per lane.
9. **[resource-governance.md](resource-governance.md)** — CBMC memory caps, fuzz timeouts, mutation budgets, `-j 1` discipline.
10. **[waiver-planning-guide.md](waiver-planning-guide.md)** — valid waiver shape; behavior-affecting waivers are fatal.

## Quality and Process

11. **[plan-quality-gates.md](plan-quality-gates.md)** — 10 gates the planner runs against its own output before handoff. Maps to `finding-codes.md`.
12. **[anti-patterns.md](anti-patterns.md)** — 20 bad planner habits with corrections.
13. **[handoff-contract.md](handoff-contract.md)** — contract with `proof-plan-reviewer`, `proof-writer`, `proof-to-implementation`, and `formal-verifier`.
14. **[state-machine-integration.md](state-machine-integration.md)** — how proof-planner fits into Go-skill state 4; inputs and outputs.
15. **[cross-lane-coverage-matrix.md](cross-lane-coverage-matrix.md)** — canonical seed shapes and the union of lanes each shape demands.

## Scripts (Deterministic Helpers)

- `scripts/validate_plan.py` — runs all 10 gates against a plan directory; emits JSONL findings; exits 1 on blockers.
- `scripts/quick_validate.py` — checks skill tree structure, frontmatter, references, templates, workflows, evals, and mirror parity.

## Templates (Per-Verifier JSON)

- `templates/verus.json`, `kani.json`, `flux-rs.json`, `loom.json`, `miri.json`, `cargo-fuzz.json`, `proptest.json` — copy and adapt.

## Workflows (End-to-End Examples)

- `workflows/pure-invariant.md` — Counter::tick-style Rust-local arithmetic.
- `workflows/parser-hostile-input.md` — Frame::decode-style binary parser.
- `workflows/concurrency-channel.md` — MpscQueue-style shutdown drain.
- `workflows/unsafe-arena-allocator.md` — Bump::alloc-style raw-pointer allocator.
- `workflows/temporal-lease-claim.md` — LeaseClaim-style workflow with retries.
- `workflows/refinement-offset-newtype.md` — Offset::new-style refinement type.

## External (Go-Skill)

- `../go-skill/references/proof-schemas.md`
- `../go-skill/references/proof-pipeline-contract.md`
- `../go-skill/references/verification-lane-policy.md`
- `../go-skill/references/evidence-standards.md`
- `../go-skill/references/review-provenance.md`
- `../go-skill/references/finding-codes.md`
- `../go-skill/references/trust-marker-scan-patterns.md`
