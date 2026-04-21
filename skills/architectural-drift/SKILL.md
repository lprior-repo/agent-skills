---
name: architectural-drift
description: "Runbook for checking architectural drift, enforcing <300 line files, and applying Scott Wlaschin DDD principles."
---

# Architectural Drift Protocol

1. **Verify Line Counts**:
   Check all `.rs` files in the current workspace.
   Any file > 300 lines MUST be split. Use your bash tools to count lines if needed.

2. **Enforce DDD (Scott Wlaschin)**:
   - Identify primitive obsession (e.g., `String`, `i32` for IDs). Refactor into NewTypes.
   - Check workflows: Are they explicit state transitions modeled as functions?
   - Verify `Parse, don't validate` is actually adhered to.

3. **Refactor & Split**:
   - If refactoring is needed: Create new files, update `mod.rs` or `lib.rs`/`main.rs`, move code, and ensure it still logically aligns with the contract.

4. **Status Outputs**:
   - State `STATUS: REFACTORED` if edits occurred.
   - State `STATUS: PERFECT` if no edits were needed.