# Tooling Availability Gate

Every required verifier lane must produce runnable evidence. When the tool is missing, broken, or out-of-version, the planner MUST emit a `verifier-lane-decision/v1` row with `applicability: blocked_tooling`, not `not_applicable`, and not silently omit the lane.

This reference is the per-lane tooling availability matrix. It pairs with `references/evidence-requirements.md` (which describes what evidence each lane must produce) and `references/resource-governance.md` (which describes resource bounds).

## Tooling Detection

Before emitting any `proof-obligation/v1` row, the planner (or a delegated `proof-writer` smoke pass) MUST run the detection command below. The exit status determines the lane decision.

| Verifier | Detection command | Pass criterion |
|---|---|---|
| `verus` | `command -v verus && verus --version` | binary present, version pinned |
| `kani` | `command -v cargo-kani && cargo kani --version` | binary present, version pinned |
| `flux-rs` | `command -v cargo-flux && cargo flux --version` plus `rustup show active-toolchain` is nightly | toolchain + cargo-flux |
| `loom` | `cargo metadata --format-version=1 \| jq -r '.packages[] \| select(.name=="loom")'` | dependency declared in `Cargo.toml` |
| `miri` | `rustup run nightly rustc --version` plus `cargo +nightly miri --version` | nightly toolchain + miri component |
| `cargo-fuzz` | `command -v cargo-fuzz && cargo fuzz --version` plus `rustup show active-toolchain` is nightly | nightly toolchain + cargo-fuzz |
| `proptest` | `cargo metadata --format-version=1 \| jq -r '.packages[] \| select(.name=="proptest")'` | dependency declared in `Cargo.toml` |

Missing or failing detection ⇒ `applicability: blocked_tooling`. The planner MUST NOT mark a default-profile verifier `not_applicable` because the tool is missing; the absence is a tooling gap, not a non-applicability.

## Per-Lane Tooling Requirements

### Verus

- Tool: `verus` binary; rustup toolchain may need custom install.
- Pin: `verus@<version>` or `verus@<commit>`.
- Required components: `verus`, optional `verusfmt`.
- Detection: see table above.
- When unavailable: emit `blocked_tooling` and route to a Verus install bead.

### Kani

- Tool: `cargo-kani`.
- Pin: `cargo-kani@<version>`.
- Required components: `cargo-kani`, CBMC backend.
- Detection: see table above.
- When unavailable: emit `blocked_tooling` and route to a Kani install bead. Common blockers: Kani requires nightly Rust or pinned stable; older Kani versions may not support the current Rust version.

### Flux

- Tool: `cargo-flux`, nightly toolchain, `liquid-fixpoint` solver, optional `z3`.
- Pin: `flux@nightly-YYYY-MM-DD`.
- Required components: nightly toolchain, `flux` crate, `cargo-flux`.
- Detection: see table above.
- When unavailable: emit `blocked_tooling`. Flux is research-grade and version-sensitive; nightly breakage is a known failure mode.

### Loom

- Tool: `loom` dependency + `cfg(loom)` build.
- Pin: `loom@<version>`.
- Detection: declared as `[target.'cfg(loom)'.dependencies] loom = ...`.
- When unavailable: emit `blocked_tooling` and add the Loom dependency.

### Miri

- Tool: nightly toolchain + miri component.
- Pin: `nightly-YYYY-MM-DD` with `miri` component installed (`rustup component add miri --toolchain nightly`).
- Detection: see table above.
- When unavailable: emit `blocked_tooling` and route to a Miri setup bead. Miri is nightly-only.

### cargo-fuzz

- Tool: nightly toolchain + `cargo-fuzz`.
- Pin: `cargo-fuzz@<version>`.
- Required components: nightly toolchain, `cargo-fuzz`, optional `llvm-tools-preview`.
- Detection: see table above.
- When unavailable: emit `blocked_tooling` and route to a cargo-fuzz install bead.

### proptest

- Tool: `proptest` dependency.
- Pin: `proptest@<version>`.
- Detection: declared as dev-dependency.
- When unavailable: emit `blocked_tooling` and add the proptest dependency. proptest is stable and rarely blocks.

## `blocked_tooling` Row Shape

```json
{
  "schema_version": "verifier-lane-decision/v1",
  "id": "VLD-018",
  "requirement_id": "REQ-104",
  "contract_clause": "C-019",
  "proof_seed_id": "PS-104",
  "verifier": "kani",
  "risk_tags": ["bounded_state", "rejection"],
  "applicability": "blocked_tooling",
  "decision_reason": "cargo-kani 0.65 not installed in this workspace; rustup does not show miri/nightly",
  "tooling_acquisition_ref": "BEAD-TOOL-KANI-INSTALL",
  "non_applicability_evidence_refs": [],
  "required_obligation_ids": [],
  "limitation_kind": "external_dependency_unavoidable",
  "owner_state": 4,
  "status": "blocked_tooling"
}
```

The `tooling_acquisition_ref` MUST be a bead ID or runbook step that resolves the gap. The row MUST NOT have `required_obligation_ids` populated; the obligation is added after the tool is available.

## Tooling Acquisition Bead Template

When the planner emits `blocked_tooling`, it also creates a tooling acquisition bead. The bead follows the same EARS/BDD template used in `landing-skill`.

```
Title: "[tooling] Install <verifier> <version> in workspace <path>"
Type: task
Priority: 1 (blocking)
Labels: smell:tooling, severity:blocking, gate:tooling

Description:
  Context: proof planning blocked; required verifier <name> unavailable.
  Smell type: tooling
  Severity: blocking
  Gate failed: tooling detection
  
  Acceptance:
    - Detection command exits 0
    - Version matches the pin
    - Smoke command for at least one minimal harness passes
```

The bead is closed only when the detection command passes AND the smoke command for at least one minimal harness in the same lane passes.

## Interaction With Other Lanes

When a default-profile verifier is blocked, the planner MUST also evaluate whether a companion lane can substitute. Substitution is allowed only when:

- The substitute lane's `risk_tags` include the seed's relevant tag.
- The substitute's `references/verifier-trigger-matrix.md` row explicitly lists the seed's risk as a primary or valid companion.
- The substitute is not itself blocked.

Substitution is documented by:

- `applicability: not_applicable` on the blocked lane with `limitation_kind: external_dependency_unavoidable` and a citation of the substitute lane's `verifier-lane-decision/v1` ID.
- The substitute lane retains `applicability: required` with a `decision_reason` that cites the blocked lane's ID and the substitution rationale.

Substitution without a documented cite is `E_LANE_SUBSTITUTION_UNJUSTIFIED`.

## Blocked Tooling at State 12

`blocked_tooling` rows at State 12 are `E_FORMAL_PENDING_AT_CLOSURE`. The closure gate rejects the bead. The tooling acquisition bead MUST close before the proof obligation closes.

## Cross-Reference With Verifier Triggers

`references/verifier-trigger-matrix.md` defines what each verifier can and cannot do. When a verifier is blocked, consult the matrix for a substitute that does not require the same tool. If no substitute exists, the bead is fundamentally blocked and the planner MUST escalate.

## Cross-Reference With Resource Governance

`references/resource-governance.md` defines resource budgets per lane. A lane may be available but unable to run within the workspace's resource budget (e.g., Kani exceeds the cgroup cap). This is distinct from `blocked_tooling`:

- `blocked_tooling`: tool is missing or wrong version.
- `resource_constrained`: tool is present but resource budget cannot honor the obligation.

`resource_constrained` rows use the same schema with `applicability: blocked_tooling`, `limitation_kind: external_dependency_unavoidable`, and a `decision_reason` citing the resource cap.
